//! crates/vm_report/src/structure.rs
//! Pure report data model + mappers from pipeline artifacts, per Doc 7.
//! No I/O, no recomputation, no floats. Deterministic ordering only.

// -------------------- Public model root & sections (Doc 7 order) --------------------

#[derive(Clone, Debug)]
pub struct ReportModel {
    pub cover: CoverSnapshot,
    pub eligibility: EligibilityBlock,
    pub ballot: BallotBlock,
    pub panel: LegitimacyPanel,
    pub outcome: OutcomeBlock,
    pub frontier: Option<FrontierBlock>,
    pub sensitivity: Option<SensitivityBlock>,
    pub integrity: IntegrityBlock,
    pub footer: FooterIds,
}

#[derive(Clone, Debug)]
pub struct CoverSnapshot {
    pub label: String,                       // Decisive|Marginal|Invalid
    pub reason: Option<String>,
    pub snapshot_vars: Vec<SnapshotVar>,     // VM-VAR key/value pairs
    pub registry_name: String,
    pub registry_published_date: String,
}
#[derive(Clone, Debug)]
pub struct SnapshotVar { pub key: String, pub value: String }

#[derive(Clone, Debug)]
pub struct EligibilityBlock {
    pub roll_policy: String,                 // VM-VAR-028, pretty
    pub totals_eligible_roll: u64,
    pub totals_ballots_cast: u64,
    pub totals_valid_ballots: u64,
    pub per_unit_quorum_note: Option<String>,// VM-VAR-021 (+scope)
    pub provenance: String,                  // source/edition text
}

#[derive(Clone, Debug)]
pub struct BallotBlock {
    pub ballot_type: String,                 // VM-VAR-001
    pub allocation_method: String,           // VM-VAR-010
    pub weighting_method: String,            // VM-VAR-030
    pub approval_denominator_sentence: bool, // “approval rate = approvals / valid ballots”
}

#[derive(Clone, Debug)]
pub struct GateRow {
    pub value_pct_1dp: String,               // e.g., "55.0%"
    pub threshold_pct_0dp: String,           // e.g., "55%"
    pub pass: bool,
    pub denom_note: Option<String>,          // for approval majority note
    pub members_hint: Option<Vec<String>>,   // for double-majority family
}

#[derive(Clone, Debug)]
pub struct LegitimacyPanel {
    pub quorum: GateRow,
    pub majority: GateRow,
    pub double_majority: Option<(GateRow, GateRow)>, // (national, family)
    pub symmetry: Option<bool>,
    pub pass: bool,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct OutcomeBlock {
    pub label: String,
    pub reason: String,
    pub national_margin_pp: String,          // signed “±pp”
}

#[derive(Clone, Debug)]
pub struct FrontierCounters {
    pub changed: u32, pub no_change: u32, pub mediation: u32,
    pub enclave: u32, pub protected_blocked: u32, pub quorum_blocked: u32,
}

#[derive(Clone, Debug)]
pub struct FrontierBlock {
    pub mode: String,                        // VM-VAR-040
    pub edge_types: String,                  // VM-VAR-047 summary
    pub island_rule: String,                 // VM-VAR-048
    pub bands_summary: Vec<String>,          // ladder/sliding descriptors (declared order)
    pub counters: FrontierCounters,
}

#[derive(Clone, Debug)]
pub struct SensitivityBlock { pub table_2x3: Vec<Vec<String>> }

#[derive(Clone, Debug)]
pub struct IntegrityBlock {
    pub engine_vendor: String, pub engine_name: String,
    pub engine_version: String, pub engine_build: String,
    pub formula_id_hex: String,
    pub tie_policy: String, pub tie_seed: Option<String>,
    pub started_utc: String, pub finished_utc: String,
}

// Footer IDs: keep strong types where available, otherwise String.
use vm_core::ids::{ResultId, RunId, FrontierMapId as FrontierId};
#[derive(Clone, Debug)]
pub struct FooterIds {
    pub result_id: ResultId,
    pub run_id: RunId,
    pub frontier_id: Option<FrontierId>,
    pub reg_id: String,
    pub param_set_id: String,
    pub tally_id: Option<String>,
}

// -------------------- Artifact “view” traits (no schema guesses here) --------------------
// The pipeline may wrap concrete DB structs to implement these views. This file depends only
// on what we need to *display*, not on storage layout. All methods are pure getters.

use core::fmt::Display;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

extern crate alloc;

/// Minimal view of Result artifact (RES).
pub trait ResultView {
    // Cover / label
    fn label(&self) -> &str;                 // "Decisive" | "Marginal" | "Invalid"
    fn label_reason(&self) -> Option<&str>;
    fn registry_name(&self) -> &str;
    fn registry_published_date(&self) -> &str;

    // Snapshot VM-VARs (already stringified by pipeline snapshot)
    fn snapshot_vars(&self) -> &[(String, String)]; // preserved order

    // Totals
    fn totals_eligible_roll(&self) -> u64;
    fn totals_ballots_cast(&self) -> u64;
    fn totals_valid_ballots(&self) -> u64;

    // Eligibility / provenance
    fn roll_policy_pretty(&self) -> &str;    // VM-VAR-028
    fn per_unit_quorum_note(&self) -> Option<String>; // VM-VAR-021 (+scope)
    fn provenance(&self) -> &str;            // source/edition

    // Ballot / methods
    fn ballot_type(&self) -> &str;           // VM-VAR-001
    fn allocation_method(&self) -> &str;     // VM-VAR-010
    fn weighting_method(&self) -> &str;      // VM-VAR-030

    // Gates (precomputed ratios & thresholds; no recomputation here)
    fn gate_quorum_ratio(&self) -> (i128, i128);     // (num, den)
    fn gate_quorum_threshold_pct(&self) -> u8;
    fn gate_quorum_pass(&self) -> bool;

    fn gate_majority_ratio(&self) -> (i128, i128);   // approval / valid_ballots
    fn gate_majority_threshold_pct(&self) -> u8;
    fn gate_majority_pass(&self) -> bool;

    fn gate_double_majority_enabled(&self) -> bool;
    fn gate_double_majority_national_ratio(&self) -> (i128, i128);
    fn gate_double_majority_family_ratio(&self) -> (i128, i128);
    fn gate_double_majority_threshold_pct(&self) -> u8; // assume same cutoff both sides
    fn gate_double_majority_family_members(&self) -> Option<Vec<String>>;
    fn gate_double_majority_pass_both(&self) -> Option<(bool, bool)>;

    fn symmetry_enabled(&self) -> bool;
    fn symmetry_pass(&self) -> Option<bool>;

    // Outcome
    fn national_margin_pp(&self) -> i32;

    // IDs
    fn result_id(&self) -> &ResultId;
    fn frontier_id(&self) -> Option<FrontierId>;
    fn reg_id_str(&self) -> &str;
    fn param_set_id_str(&self) -> &str;
    fn tally_id_str(&self) -> Option<&str>;
}

/// Minimal view of RunRecord artifact (RUN).
pub trait RunRecordView {
    fn engine_vendor(&self) -> &str;
    fn engine_name(&self) -> &str;
    fn engine_version(&self) -> &str;
    fn engine_build(&self) -> &str;

    fn formula_id_hex(&self) -> &str;

    fn tie_policy(&self) -> &str;            // "deterministic" | "random" | ...
    fn tie_seed_opt(&self) -> Option<u64>;   // only meaningful if policy == random

    fn started_utc(&self) -> &str;           // "YYYY-MM-DDTHH:MM:SSZ"
    fn finished_utc(&self) -> &str;

    fn run_id(&self) -> &RunId;
}

/// Minimal view of FrontierMap artifact (FR).
pub trait FrontierMapView {
    fn mode_pretty(&self) -> &str;           // VM-VAR-040
    fn edge_types_summary(&self) -> &str;    // VM-VAR-047
    fn island_rule_pretty(&self) -> &str;    // VM-VAR-048
    fn bands_summary(&self) -> &[String];    // declared order, already pretty

    fn counters(&self) -> FrontierCounters;  // changed / no_change / mediation / enclave / protected_blocked / quorum_blocked
}

// Public aliases to match function signatures in the spec (trait objects).
pub type ResultDb     = dyn ResultView;
pub type RunRecordDb  = dyn RunRecordView;
pub type FrontierMapDb= dyn FrontierMapView;

// -------------------- Top-level mapping API (pure) --------------------

pub fn model_from_artifacts(
    result: &ResultDb,
    run: &RunRecordDb,
    frontier: Option<&FrontierMapDb>,
) -> ReportModel {
    let cover = map_cover_snapshot(result);
    let eligibility = map_eligibility(result);
    let ballot = map_ballot(result);
    let panel = map_panel_from_gates(result);
    let outcome = map_outcome_from_result(result);
    let frontier_block = frontier.map(|fr| map_frontier(fr, result));
    let sensitivity = map_sensitivity(result);
    let (integrity, footer) = map_integrity_footer(run, result, frontier);

    ReportModel {
        cover,
        eligibility,
        ballot,
        panel,
        outcome,
        frontier: frontier_block,
        sensitivity,
        integrity,
        footer,
    }
}

// -------------------- Mapping helpers (pure) --------------------

fn map_cover_snapshot(result: &ResultDb) -> CoverSnapshot {
    CoverSnapshot {
        label: result.label().to_string(),
        reason: result.label_reason().map(|s| s.to_string()),
        snapshot_vars: result.snapshot_vars().iter().cloned().collect(),
        registry_name: result.registry_name().to_string(),
        registry_published_date: result.registry_published_date().to_string(),
    }
}

fn map_eligibility(result: &ResultDb) -> EligibilityBlock {
    EligibilityBlock {
        roll_policy: result.roll_policy_pretty().to_string(),
        totals_eligible_roll: result.totals_eligible_roll(),
        totals_ballots_cast: result.totals_ballots_cast(),
        totals_valid_ballots: result.totals_valid_ballots(),
        per_unit_quorum_note: result.per_unit_quorum_note(),
        provenance: result.provenance().to_string(),
    }
}

fn map_ballot(result: &ResultDb) -> BallotBlock {
    let ballot_type = result.ballot_type().to_string();
    let approval_sentence = ballot_type.to_ascii_lowercase() == "approval";
    BallotBlock {
        ballot_type,
        allocation_method: result.allocation_method().to_string(),
        weighting_method: result.weighting_method().to_string(),
        approval_denominator_sentence: approval_sentence,
    }
}

fn map_panel_from_gates(result: &ResultDb) -> LegitimacyPanel {
    // Quorum
    let (q_num, q_den) = result.gate_quorum_ratio();
    let quorum = GateRow {
        value_pct_1dp: pct_1dp(q_num, q_den),
        threshold_pct_0dp: pct0(result.gate_quorum_threshold_pct()),
        pass: result.gate_quorum_pass(),
        denom_note: None,
        members_hint: None,
    };

    // National approval majority (approval / valid ballots)
    let (m_num, m_den) = result.gate_majority_ratio();
    let majority = GateRow {
        value_pct_1dp: pct_1dp(m_num, m_den),
        threshold_pct_0dp: pct0(result.gate_majority_threshold_pct()),
        pass: result.gate_majority_pass(),
        denom_note: Some("approval rate = approvals / valid ballots".to_string()),
        members_hint: None,
    };

    // Double-majority (if enabled)
    let dm = if result.gate_double_majority_enabled() {
        let (n_num, n_den) = result.gate_double_majority_national_ratio();
        let (f_num, f_den) = result.gate_double_majority_family_ratio();
        let th = pct0(result.gate_double_majority_threshold_pct());
        let (pass_nat, pass_fam) = result.gate_double_majority_pass_both().unwrap_or((false, false));
        let hint = result.gate_double_majority_family_members();

        let nat = GateRow {
            value_pct_1dp: pct_1dp(n_num, n_den),
            threshold_pct_0dp: th.clone(),
            pass: pass_nat,
            denom_note: Some("approval rate = approvals / valid ballots".to_string()),
            members_hint: None,
        };
        let fam = GateRow {
            value_pct_1dp: pct_1dp(f_num, f_den),
            threshold_pct_0dp: th,
            pass: pass_fam,
            denom_note: Some("approval rate = approvals / valid ballots".to_string()),
            members_hint: hint,
        };
        Some((nat, fam))
    } else {
        None
    };

    let symmetry = if result.symmetry_enabled() { result.symmetry_pass() } else { None };

    // Overall pass/reasons mirror result label & gate passes (no recompute).
    let pass = quorum.pass && majority.pass && dm.as_ref().map_or(true, |(n, f)| n.pass && f.pass)
        && symmetry.unwrap_or(true);

    let mut reasons: Vec<String> = Vec::new();
    if !quorum.pass { reasons.push("Quorum failed".into()); }
    if !majority.pass { reasons.push("National majority failed".into()); }
    if let Some((n, f)) = &dm {
        if !n.pass || !f.pass { reasons.push("Double-majority failed".into()); }
    }
    if let Some(false) = symmetry { reasons.push("Symmetry failed".into()); }

    LegitimacyPanel { quorum, majority, double_majority: dm, symmetry, pass, reasons }
}

fn map_outcome_from_result(result: &ResultDb) -> OutcomeBlock {
    OutcomeBlock {
        label: result.label().to_string(),
        reason: result.label_reason().unwrap_or_default().to_string(),
        national_margin_pp: pp_signed(result.national_margin_pp()),
    }
}

fn map_frontier(fr: &FrontierMapDb, _result: &ResultDb) -> FrontierBlock {
    FrontierBlock {
        mode: fr.mode_pretty().to_string(),
        edge_types: fr.edge_types_summary().to_string(),
        island_rule: fr.island_rule_pretty().to_string(),
        bands_summary: fr.bands_summary().to_vec(),
        counters: fr.counters(),
    }
}

fn map_sensitivity(_result: &ResultDb) -> Option<SensitivityBlock> {
    // v1: no scenarios packaged ⇒ None
    None
}

fn map_integrity_footer(
    run: &RunRecordDb,
    result: &ResultDb,
    frontier: Option<&FrontierMapDb>,
) -> (IntegrityBlock, FooterIds) {
    // Integrity
    let (policy, seed_opt) = (run.tie_policy().to_string(), run.tie_seed_opt());
    let tie_seed = match (policy.as_str(), seed_opt) {
        ("random", Some(s)) => Some(s.to_string()),
        _ => None,
    };
    let integrity = IntegrityBlock {
        engine_vendor: run.engine_vendor().to_string(),
        engine_name: run.engine_name().to_string(),
        engine_version: run.engine_version().to_string(),
        engine_build: run.engine_build().to_string(),
        formula_id_hex: run.formula_id_hex().to_string(),
        tie_policy: policy,
        tie_seed,
        started_utc: run.started_utc().to_string(),
        finished_utc: run.finished_utc().to_string(),
    };

    // Footer IDs
    let footer = FooterIds {
        result_id: result.result_id().clone(),
        run_id: run.run_id().clone(),
        frontier_id: frontier.map(|_| result.frontier_id()).flatten(),
        reg_id: result.reg_id_str().to_string(),
        param_set_id: result.param_set_id_str().to_string(),
        tally_id: result.tally_id_str().map(|s| s.to_string()),
    };

    (integrity, footer)
}

// -------------------- Formatting helpers (integer math only) --------------------

fn pct_1dp(num: i128, den: i128) -> String {
    use vm_core::rounding::percent_one_decimal_tenths;
    if den <= 0 { return "0.0%".into(); }
    let tenths = percent_one_decimal_tenths(num, den); // e.g., 553 → 55.3%
    let sign = if tenths < 0 { "-" } else { "" };
    let abs = tenths.abs();
    let whole = abs / 10;
    let dec = (abs % 10) as i128;
    format!("{sign}{whole}.{dec}%")
}

fn pct0(value_u8: u8) -> String {
    format!("{value_u8}%")
}

fn pp_signed(pp_i32: i32) -> String {
    match pp_i32.cmp(&0) {
        core::cmp::Ordering::Greater => format!("+{pp_i32} pp"),
        core::cmp::Ordering::Equal => "±0 pp".to_string(),
        core::cmp::Ordering::Less => format!("{pp_i32} pp"),
    }
}
