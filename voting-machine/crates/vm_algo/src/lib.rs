//! crates/vm_algo/src/lib.rs
//! Public surface for pure algorithm primitives (tabulation, allocation, gates/frontier).
//! No I/O, no JSON; deterministic ordering; RNG only for ties per 050/052.
//!
//! Ordering rules (normative reminders):
//! - Units iterate by ascending `unit_id`; options by `(order_index, option_id)`; allocations mirror registry order. :contentReference[oaicite:4]{index=4}
//! - RNG is used only for random tie policy (050) and seeded from 052 by the pipeline. :contentReference[oaicite:5]{index=5}

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub use vm_core::{
    ids::{OptionId, UnitId},
    rng::TieRng,
    // For API clarity, alias vm_core's TallyTotals as Turnout here.
    entities::TallyTotals as Turnout,
    // Ratio type (integer rational compare/formatting) lives in vm_core.
    rounding::Ratio,
    // Variable enums (tie policy, allocation method, etc.) are defined in vm_core::variables.
    variables::{AllocationMethod, OverhangPolicy, TiePolicy, TotalSeatsModel},
    // Registry option metadata (order_index) used for canonical ordering.
    entities::OptionItem,
};

/// Raw scores per unit, ready for allocation/aggregation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitScores {
    pub unit_id: UnitId,
    pub turnout: Turnout,                 // valid/invalid/total ballots
    pub scores: BTreeMap<OptionId, u64>,  // plurality=votes; approval=approvals; score=score sums
}

/// Per-unit allocation result; deterministic ordering by (order_index, OptionId).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allocation {
    pub unit_id: UnitId,
    /// For PR methods: seats per option. For WTA single-member: winner may be represented as 1 seat
    /// (pipeline interprets accordingly). Ordering mirrors registry option order. :contentReference[oaicite:6]{index=6}
    pub seats_or_power: BTreeMap<OptionId, u32>,
    /// True iff a tie policy decided a last seat / winner. (Pipeline logs tie details.) :contentReference[oaicite:7]{index=7}
    pub last_seat_tie: bool,
}

/// IRV round transfer/audit data (minimal, engine-agnostic).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrvRound {
    pub eliminated: OptionId,
    pub transfers: BTreeMap<OptionId, u64>,
    pub exhausted: u64,
}

/// IRV log across rounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrvLog {
    pub rounds: Vec<IrvRound>,
    pub winner: OptionId,
}

/// Pairwise wins map for Condorcet completions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pairwise {
    /// (A,B) = votes preferring A over B.
    pub wins: BTreeMap<(OptionId, OptionId), u64>,
}

/// Gate outcome (ratios are integer rationals; no floats).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GateOutcome {
    pub pass: bool,
    pub observed: Ratio,
    pub threshold_pct: u8,
}

/// Double-majority composition (e.g., national + regional).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoubleMajority {
    pub national: GateOutcome,
    pub regional: GateOutcome,
    pub pass: bool,
}

// ---- Module layout (stubs). Implementations live in sibling modules. ---------------------------
// The pipeline calls these in the step order defined in Doc 4A/5A; tests conform to Doc 6A–6C.
// (We provide function signatures here with `unimplemented!()` to make the public API explicit.)

/// Tabulation primitives (plurality/approval/score, IRV, Condorcet).
pub mod tabulation {
    use super::*;

    /// Plurality: raw vote counts per option for a unit.
    pub fn tabulate_plurality(
        unit_id: UnitId,
        votes: &BTreeMap<OptionId, u64>,
        turnout: Turnout,
    ) -> UnitScores {
        let _ = (unit_id, votes, turnout);
        unimplemented!("tabulate_plurality (4A S1)"); // :contentReference[oaicite:8]{index=8}
    }

    /// Approval: approvals per option; denominator for shares is valid_ballots. :contentReference[oaicite:9]{index=9}
    pub fn tabulate_approval(
        unit_id: UnitId,
        approvals: &BTreeMap<OptionId, u64>,
        turnout: Turnout,
    ) -> UnitScores {
        let _ = (unit_id, approvals, turnout);
        unimplemented!("tabulate_approval (4A S1)");
    }

    /// Score: sums of scores per option. Scale domain validated upstream; integer-only here. :contentReference[oaicite:10]{index=10}
    pub fn tabulate_score(
        unit_id: UnitId,
        score_sums: &BTreeMap<OptionId, u64>,
        turnout: Turnout,
    ) -> UnitScores {
        let _ = (unit_id, score_sums, turnout);
        unimplemented!("tabulate_score (4A S1)");
    }

    /// IRV on compressed ranked ballots; fixed exhaustion policy per spec; deterministic ties via 050/052. :contentReference[oaicite:11]{index=11}
    pub fn tabulate_ranked_irv(
        ballots: &[(Vec<OptionId>, u64)],   // unique options per ranking; multiplicity
        options: &[OptionItem],             // ordered by (order_index, id)
    ) -> (UnitScores, IrvLog) {
        let _ = (ballots, options);
        unimplemented!("tabulate_ranked_irv (4A S1 + 4C ties)");
    }

    /// Condorcet: pairwise tallies + completion; deterministic ordering. :contentReference[oaicite:12]{index=12}
    pub fn tabulate_ranked_condorcet(
        ballots: &[(Vec<OptionId>, u64)],
        options: &[OptionItem],
    ) -> (UnitScores, Pairwise) {
        let _ = (ballots, options);
        unimplemented!("tabulate_ranked_condorcet (4A S1)");
    }
}

/// Allocation methods within a unit (WTA, divisors, largest remainder).
pub mod allocation {
    use super::*;

    /// Winner-take-all for single-member magnitude. Ties via policy 050; RNG only if Random (052). :contentReference[oaicite:13]{index=13}
    pub fn allocate_wta(
        scores: &UnitScores,
        magnitude: u32,
        options: &[OptionItem],
        tie_policy: TiePolicy,
        mut rng: Option<&mut TieRng>,
    ) -> Allocation {
        let _ = (scores, magnitude, options, tie_policy, &mut rng);
        unimplemented!("allocate_wta (4A S4 + 4C ties)");
    }

    /// D’Hondt (Jefferson). Deterministic order per registry options. :contentReference[oaicite:14]{index=14}
    pub fn allocate_dhondt(
        seats: u32,
        scores: &BTreeMap<OptionId, u64>,
        options: &[OptionItem],
    ) -> BTreeMap<OptionId, u32> {
        let _ = (seats, scores, options);
        unimplemented!("allocate_dhondt (4A S4)");
    }

    /// Sainte-Laguë (Webster). Deterministic order per registry options. :contentReference[oaicite:15]{index=15}
    pub fn allocate_sainte_lague(
        seats: u32,
        scores: &BTreeMap<OptionId, u64>,
        options: &[OptionItem],
    ) -> BTreeMap<OptionId, u32> {
        let _ = (seats, scores, options);
        unimplemented!("allocate_sainte_lague (4A S4)");
    }

    /// Largest Remainder with threshold (% of valid ballots before quota). :contentReference[oaicite:16]{index=16}
    pub fn allocate_largest_remainder(
        seats: u32,
        scores: &BTreeMap<OptionId, u64>,
        threshold_pct: u8,
        options: &[OptionItem],
    ) -> BTreeMap<OptionId, u32> {
        let _ = (seats, scores, threshold_pct, options);
        unimplemented!("allocate_largest_remainder (4A S4)");
    }
}

/// Mixed-member proportional helpers (targets & top-ups).
pub mod mmp {
    use super::*;

    /// Compute per-option seat targets from vote totals using a PR method baseline. :contentReference[oaicite:17]{index=17}
    pub fn mmp_target_shares(
        total_seats: u32,
        vote_totals: &BTreeMap<OptionId, u64>,
        method: AllocationMethod,
    ) -> BTreeMap<OptionId, u32> {
        let _ = (total_seats, vote_totals, method);
        unimplemented!("mmp_target_shares");
    }

    /// Compute top-ups given local seats and targets; policies control overhang & total seats. :contentReference[oaicite:18]{index=18}
    pub fn mmp_topups(
        local_seats: &BTreeMap<OptionId, u32>,
        targets: &BTreeMap<OptionId, u32>,
        overhang_policy: OverhangPolicy,
        total_seats_model: TotalSeatsModel,
    ) -> BTreeMap<OptionId, u32> {
        let _ = (local_seats, targets, overhang_policy, total_seats_model);
        unimplemented!("mmp_topups");
    }
}

/// Gates & frontier helpers (integer ratio math; no floats).
pub mod gates_frontier {
    use super::*;

    /// Quorum: observed = valid_ballots / eligible_roll; rational compare against threshold. :contentReference[oaicite:19]{index=19}
    pub fn gate_quorum(valid_ballots: u64, eligible_roll: u64, threshold_pct: u8) -> GateOutcome {
        let _ = (valid_ballots, eligible_roll, threshold_pct);
        unimplemented!("gate_quorum (4B)");
    }

    /// Majority: observed = approvals_for_change / valid_ballots (denominator is valid ballots). :contentReference[oaicite:20]{index=20}
    pub fn gate_majority(
        valid_ballots: u64,
        approvals_for_change: u64,
        threshold_pct: u8,
    ) -> GateOutcome {
        let _ = (valid_ballots, approvals_for_change, threshold_pct);
        unimplemented!("gate_majority (4B)");
    }

    /// Double-majority composition (national + regional). :contentReference[oaicite:21]{index=21}
    pub fn gate_double_majority(national: GateOutcome, regional: GateOutcome) -> DoubleMajority {
        let _ = (national, regional);
        unimplemented!("gate_double_majority (4B)");
    }

    /// Frontier support ratio helper (approval rate), used by frontier diagnostics. :contentReference[oaicite:22]{index=22}
    pub fn frontier_support_ratio(approvals_for_change: u64, valid_ballots: u64) -> Ratio {
        let _ = (approvals_for_change, valid_ballots);
        unimplemented!("frontier_support_ratio (4C)");
    }
}

// Re-exports for ergonomic use by pipeline/tests.
pub use allocation::*;
pub use gates_frontier::*;
pub use tabulation::*;
pub use mmp::*;
