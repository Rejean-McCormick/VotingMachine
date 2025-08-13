//! crates/vm_pipeline/src/tabulate.rs
//! TABULATE stage: compute per-Unit `UnitScores` deterministically from the loaded,
//! canonicalized inputs according to VM-VAR-001. Integer-only; no RNG here.
//!
//! Inputs are provided through a minimal `LoadedContext` view (defined here to keep
//! this module self-contained). The pipeline may later replace it with a richer type.

use std::collections::BTreeMap;

use vm_core::{
    entities::{OptionItem, TallyTotals as Turnout},
    ids::{OptionId, UnitId},
    variables::Params,
};
use vm_algo::{
    tabulation,      // pure algorithm entry points
    IrvLog, Pairwise, UnitScores,
};

// ----- Context & unit inputs --------------------------------------------------------------------

/// Minimal, stable view of what TABULATE needs.
#[derive(Clone, Debug, Default)]
pub struct LoadedContext {
    /// Units to process, already canonicalized upstream (units ↑ by UnitId; options ↑ by (order_index, id)).
    pub units: Vec<UnitInput>,
}

/// Per-unit input shape supporting all ballot types; unused fields are ignored
/// by the relevant per-type dispatcher.
#[derive(Clone, Debug, Default)]
pub struct UnitInput {
    pub unit_id: UnitId,
    pub options: Vec<OptionItem>,                  // canonical order
    pub turnout: Turnout,                          // valid/invalid/total (as aggregated upstream)

    // Plurality/Approval/Score
    pub plurality_votes: BTreeMap<OptionId, u64>,  // plurality
    pub approvals: BTreeMap<OptionId, u64>,        // approval
    pub score_sums: BTreeMap<OptionId, u64>,       // score (already domain-checked upstream)

    // Ranked (IRV/Condorcet): compressed ballots (ranking, multiplicity)
    pub ranked_ballots: Vec<(Vec<OptionId>, u64)>,
}

// ----- Audit sidecar ---------------------------------------------------------------------------

/// Optional, lightweight log of a Condorcet completion (kept here until vm_algo exposes one).
#[derive(Clone, Debug, Default)]
pub struct CondorcetLog {
    pub steps: Vec<String>,
}

/// Audit payload emitted by TABULATE. Downstream stages (allocation, reporting, ties)
/// may read from these sidecars.
#[derive(Clone, Debug, Default)]
pub struct TabulateAudit {
    pub irv_logs: BTreeMap<UnitId, IrvLog>,
    pub condorcet_pairwise: BTreeMap<UnitId, Pairwise>,
    pub condorcet_logs: BTreeMap<UnitId, CondorcetLog>,
    /// Pending tie contexts (e.g., IRV elimination deadlocks) are collected here by a later
    /// revision once `crate::ties::TieContext` lands.
    pub pending_ties: Vec<()>, // placeholder; replaced by crate::ties::TieContext later
}

// ----- Param view (only what we need from VM-VAR-001..007 here) --------------------------------

trait TabulateParamView {
    fn ballot_is_plurality_001(&self) -> bool;
    fn ballot_is_approval_001(&self) -> bool;
    fn ballot_is_score_001(&self) -> bool;
    fn ballot_is_ranked_irv_001(&self) -> bool;
    fn ballot_is_ranked_condorcet_001(&self) -> bool;
}

impl TabulateParamView for Params {
    // These forwarders assume `Params` exposes the same-named getters (Doc 2 / Annex A).
    #[inline] fn ballot_is_plurality_001(&self) -> bool { self.ballot_is_plurality_001() }
    #[inline] fn ballot_is_approval_001(&self) -> bool { self.ballot_is_approval_001() }
    #[inline] fn ballot_is_score_001(&self) -> bool { self.ballot_is_score_001() }
    #[inline] fn ballot_is_ranked_irv_001(&self) -> bool { self.ballot_is_ranked_irv_001() }
    #[inline] fn ballot_is_ranked_condorcet_001(&self) -> bool { self.ballot_is_ranked_condorcet_001() }
}

// ----- Public entry point ----------------------------------------------------------------------

/// Tabulate all units in canonical order, producing per-unit `UnitScores` and audit sidecars.
/// No ratios/shares are computed here; those happen downstream (aggregation/reporting).
pub fn tabulate_all(
    ctx: &LoadedContext,
    p: &Params,
) -> (BTreeMap<UnitId, UnitScores>, TabulateAudit) {
    let mut out_scores: BTreeMap<UnitId, UnitScores> = BTreeMap::new();
    let mut audit = TabulateAudit::default();

    if p.ballot_is_plurality_001() {
        for u in &ctx.units {
            let sc = tabulate_unit_plurality(u);
            out_scores.insert(u.unit_id.clone(), sc);
        }
        return (out_scores, audit);
    }

    if p.ballot_is_approval_001() {
        for u in &ctx.units {
            let sc = tabulate_unit_approval(u);
            out_scores.insert(u.unit_id.clone(), sc);
        }
        return (out_scores, audit);
    }

    if p.ballot_is_score_001() {
        for u in &ctx.units {
            let sc = tabulate_unit_score(u, p);
            out_scores.insert(u.unit_id.clone(), sc);
        }
        return (out_scores, audit);
    }

    if p.ballot_is_ranked_irv_001() {
        for u in &ctx.units {
            let (sc, maybe_log /*, maybe_tie */) = tabulate_unit_ranked_irv(u, p);
            if let Some(log) = maybe_log {
                audit.irv_logs.insert(u.unit_id.clone(), log);
            }
            // if let Some(tc) = maybe_tie { audit.pending_ties.push(tc); }
            out_scores.insert(u.unit_id.clone(), sc);
        }
        return (out_scores, audit);
    }

    if p.ballot_is_ranked_condorcet_001() {
        for u in &ctx.units {
            let (sc, maybe_pw, maybe_log) = tabulate_unit_ranked_condorcet(u, p);
            if let Some(pw) = maybe_pw {
                audit.condorcet_pairwise.insert(u.unit_id.clone(), pw);
            }
            if let Some(lg) = maybe_log {
                audit.condorcet_logs.insert(u.unit_id.clone(), lg);
            }
            out_scores.insert(u.unit_id.clone(), sc);
        }
        return (out_scores, audit);
    }

    // Unknown ballot type – return empty with no scores; a higher layer should surface an error.
    (out_scores, audit)
}

// ----- Per-type dispatchers (thin wrappers around vm_algo::tabulation) --------------------------

fn tabulate_unit_plurality(unit: &UnitInput) -> UnitScores {
    tabulation::tabulate_plurality(
        unit.unit_id.clone(),
        &unit.plurality_votes,
        unit.turnout,
    )
}

fn tabulate_unit_approval(unit: &UnitInput) -> UnitScores {
    tabulation::tabulate_approval(
        unit.unit_id.clone(),
        &unit.approvals,
        unit.turnout,
    )
}

fn tabulate_unit_score(unit: &UnitInput, _p: &Params) -> UnitScores {
    // Score scale/normalization (002/003/004) are enforced in vm_algo; inputs are already validated.
    tabulation::tabulate_score(
        unit.unit_id.clone(),
        &unit.score_sums,
        unit.turnout,
    )
}

fn tabulate_unit_ranked_irv(
    unit: &UnitInput,
    _p: &Params,
) -> (UnitScores, Option<IrvLog> /*, Option<crate::ties::TieContext>*/) {
    let (sc, log) = tabulation::tabulate_ranked_irv(&unit.ranked_ballots, &unit.options);
    (sc, Some(log) /*, None*/)
}

fn tabulate_unit_ranked_condorcet(
    unit: &UnitInput,
    _p: &Params,
) -> (UnitScores, Option<Pairwise>, Option<CondorcetLog>) {
    let (sc, pw) = tabulation::tabulate_ranked_condorcet(&unit.ranked_ballots, &unit.options);
    (sc, Some(pw), None) // CondorcetLog placeholder until vm_algo exposes one
}
