//! vm_report/src/structure.rs
//! Pure report model + mappers (no I/O, no RNG).
//!
//! Inputs are artifact JSONs (Result / RunRecord / FrontierMap). This module
//! builds a renderer-friendly `ReportModel` mirroring Doc 7, with all human-
//! visible numbers preformatted (one-decimal percent; signed pp).
//!
//! Determinism: stable field order, BTree maps when needed, no floats.

#![deny(unsafe_code)]

use std::collections::BTreeMap;

use vm_core::ids::{FrontierId, ParamSetId, RegId, ResultId, RunId, TallyId};
use vm_core::rounding::percent_one_decimal_tenths;

// ----- Artifact aliases (decoupled from vm_pipeline/vm_io concrete types) -----
pub type ResultDb = serde_json::Value;
pub type RunRecordDb = serde_json::Value;
pub type FrontierMapDb = serde_json::Value;

// ===================== Model root =====================

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

// ---------------- Sections ----------------

#[derive(Clone, Debug)]
pub struct CoverSnapshot {
    pub label: String,               // Decisive|Marginal|Invalid
    pub reason: Option<String>,
    pub snapshot_vars: Vec<SnapshotVar>, // key/value VM-VARs for cover box
    pub registry_name: String,
    pub registry_published_date: String,
}
#[derive(Clone, Debug)]
pub struct SnapshotVar { pub key: String, pub value: String }

#[derive(Clone, Debug)]
pub struct EligibilityBlock {
    pub roll_policy: String,                // pretty VM-VAR-028
    pub totals_eligible_roll: u64,
    pub totals_ballots_cast: u64,
    pub totals_valid_ballots: u64,
    pub per_unit_quorum_note: Option<String>, // VM-VAR-021 + scope
    pub provenance: String,                 // source/edition string
}

#[derive(Clone, Debug)]
pub struct BallotBlock {
    pub ballot_type: String,                // VM-VAR-001
    pub allocation_method: String,          // VM-VAR-010
    pub weighting_method: String,           // VM-VAR-030
    pub approval_denominator_sentence: bool,
}

#[derive(Clone, Debug)]
pub struct GateRow {
    pub value_pct_1dp: String,              // e.g., "55.0%"
    pub threshold_pct_0dp: String,          // e.g., "55%"
    pub pass: bool,
    pub denom_note: Option<String>,         // “approval rate = approvals / valid ballots”
    pub members_hint: Option<Vec<String>>,  // double-majority family (ids/names) if present
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
    pub national_margin_pp: String,         // signed "±pp"
}

#[derive(Clone, Debug)]
pub struct FrontierCounters {
    pub changed: u32, pub no_change: u32, pub mediation: u32,
    pub enclave: u32, pub protected_blocked: u32, pub quorum_blocked: u32,
}

#[derive(Clone, Debug)]
pub struct FrontierBlock {
    pub mode: String,                       // VM-VAR-040
    pub edge_types: String,                 // VM-VAR-047 summary
    pub island_rule: String,                // VM-VAR-048
    pub bands_summary: Vec<String>,         // ladder/sliding descriptors
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

#[derive(Clone, Debug)]
pub struct FooterIds {
    pub result_id: ResultId,
    pub run_id: RunId,
    pub frontier_id: Option<FrontierId>,
    pub reg_id: RegId,
    pub param_set_id: ParamSetId,
    pub tally_id: Option<TallyId>,
}

// ===================== Top-level mapping API =====================

/// Build the full model from artifacts (pure, deterministic; no I/O).
pub fn model_from_artifacts(
    result: &ResultDb,
    run: &RunRecordDb,
    frontier: Option<&FrontierMapDb>
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

// ===================== Mapping helpers =====================

pub fn map_cover_snapshot(result: &ResultDb) -> CoverSnapshot {
    let label = j_str(result, "/label").unwrap_or_else(|| "Invalid".into());
    let reason = j_str(result, "/label_reason");
    let mut snapshot_vars = Vec::<SnapshotVar>::new();

    if let Some(bt) = j_str(result, "/params/ballot_type") {
        snapshot_vars.push(SnapshotVar{ key: "ballot_type".into(), value: bt });
    }
    if let Some(am) = j_str(result, "/params/allocation_method") {
        snapshot_vars.push(SnapshotVar{ key: "allocation_method".into(), value: am });
    }
    if let Some(wm) = j_str(result, "/aggregates/weighting_method") {
        snapshot_vars.push(SnapshotVar{ key: "weighting_method".into(), value: wm });
    }
    if let Some(th) = j_u64(result, "/params/pr_entry_threshold_pct") {
        snapshot_vars.push(SnapshotVar{ key: "pr_entry_threshold_pct".into(), value: format!("{}%", th) });
    }
    if let Some(dm) = j_bool(result, "/params/double_majority_enabled") {
        snapshot_vars.push(SnapshotVar{ key: "double_majority_enabled".into(), value: dm.to_string() });
    }
    if let Some(sym) = j_bool(result, "/params/symmetry_enabled") {
        snapshot_vars.push(SnapshotVar{ key: "symmetry_enabled".into(), value: sym.to_string() });
    }
    if let Some(fm) = j_str(result, "/params/frontier_mode") {
        snapshot_vars.push(SnapshotVar{ key: "frontier_mode".into(), value: fm });
    }

    let registry_name = j_str(result, "/provenance/registry_name").unwrap_or_else(|| "registry".into());
    let registry_published_date = j_str(result, "/provenance/registry_published_date").unwrap_or_else(|| "".into());

    CoverSnapshot { label, reason, snapshot_vars, registry_name, registry_published_date }
}

pub fn map_eligibility(result: &ResultDb) -> EligibilityBlock {
    let roll_policy = j_str(result, "/params/roll_inclusion_policy").unwrap_or_else(|| "unspecified".into());
    let totals_eligible_roll = j_u64(result, "/aggregates/turnout/eligible_roll").unwrap_or(0);
    let totals_ballots_cast  = j_u64(result, "/aggregates/turnout/ballots_cast").unwrap_or(0);
    let totals_valid_ballots = j_u64(result, "/aggregates/turnout/valid_ballots").unwrap_or(0);

    let per_unit_quorum_note = j_u64(result, "/params/quorum_per_unit_pct")
        .and_then(|q| if q > 0 {
            let scope = j_str(result, "/params/quorum_per_unit_scope").unwrap_or_else(|| "units".into());
            Some(format!("Per-unit quorum applied at {}% (scope: {})", q, scope))
        } else { None });

    let provenance = j_str(result, "/provenance/registry_source")
        .or_else(|| j_str(result, "/provenance/registry_name"))
        .unwrap_or_else(|| "registry".into());

    EligibilityBlock {
        roll_policy,
        totals_eligible_roll,
        totals_ballots_cast,
        totals_valid_ballots,
        per_unit_quorum_note,
        provenance,
    }
}

pub fn map_ballot(result: &ResultDb) -> BallotBlock {
    let ballot_type = j_str(result, "/params/ballot_type").unwrap_or_else(|| "unspecified".into());
    let allocation_method = j_str(result, "/params/allocation_method").unwrap_or_else(|| "unspecified".into());
    let weighting_method  = j_str(result, "/aggregates/weighting_method").unwrap_or_else(|| "unspecified".into());
    let approval_denominator_sentence = ballot_type == "approval";

    BallotBlock { ballot_type, allocation_method, weighting_method, approval_denominator_sentence }
}

pub fn map_panel_from_gates(result: &ResultDb) -> LegitimacyPanel {
    let gates = result.pointer("/gates").cloned().unwrap_or_default();

    let quorum = GateRow {
        value_pct_1dp: percent_number_to_1dp_str(gates.pointer("/quorum/observed")),
        threshold_pct_0dp: j_u64(&gates, "/quorum/threshold_pct").map(|v| format!("{}%", v)).unwrap_or_else(|| "0%".into()),
        pass: j_bool(&gates, "/quorum/pass").unwrap_or(false),
        denom_note: None,
        members_hint: None,
    };

    let majority = GateRow {
        value_pct_1dp: percent_number_to_1dp_str(gates.pointer("/majority/observed")),
        threshold_pct_0dp: j_u64(&gates, "/majority/threshold_pct").map(|v| format!("{}%", v)).unwrap_or_else(|| "0%".into()),
        pass: j_bool(&gates, "/majority/pass").unwrap_or(false),
        denom_note: Some("approval rate = approvals / valid ballots".into()),
        members_hint: None,
    };

    let double_majority = gates.pointer("/double_majority").and_then(|dm| {
        let nat = GateRow {
            value_pct_1dp: percent_number_to_1dp_str(dm.pointer("/national/observed")),
            threshold_pct_0dp: j_u64(dm, "/national/threshold_pct").map(|v| format!("{}%", v)).unwrap_or_else(|| "0%".into()),
            pass: j_bool(dm, "/national/pass").unwrap_or(false),
            denom_note: Some("approval rate = approvals / valid ballots".into()),
            members_hint: None,
        };
        let fam = GateRow {
            value_pct_1dp: percent_number_to_1dp_str(dm.pointer("/regional/observed")),
            threshold_pct_0dp: j_u64(dm, "/regional/threshold_pct").map(|v| format!("{}%", v)).unwrap_or_else(|| "0%".into()),
            pass: j_bool(dm, "/regional/pass").unwrap_or(false),
            denom_note: Some("approval rate = approvals / valid ballots".into()),
            members_hint: dm.pointer("/members")
                .and_then(|lst| lst.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
        };
        Some((nat, fam))
    });

    let symmetry = gates.pointer("/symmetry").and_then(|s| s.get("pass")).and_then(|v| v.as_bool());
    let pass = j_bool(&gates, "/pass").unwrap_or(false);
    let reasons = gates.pointer("/reasons")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
        .unwrap_or_default();

    LegitimacyPanel { quorum, majority, double_majority, symmetry, pass, reasons }
}

pub fn map_outcome_from_result(result: &ResultDb) -> OutcomeBlock {
    let label  = j_str(result, "/label").unwrap_or_else(|| "Invalid".into());
    let reason = j_str(result, "/label_reason").unwrap_or_else(|| "gates_failed".into());
    let nmargin = j_i64(result, "/aggregates/national_margin_pp").unwrap_or(0) as i32;

    OutcomeBlock {
        label,
        reason,
        national_margin_pp: pp_signed(nmargin),
    }
}

pub fn map_frontier(fr: &FrontierMapDb, result: &ResultDb) -> FrontierBlock {
    // Mode/edges/island can be read from params echoed in Result; fallback to FR if present.
    let mode        = j_str(result, "/params/frontier_mode")
        .or_else(|| j_str(fr, "/mode")).unwrap_or_else(|| "none".into());
    let edge_types  = j_str(result, "/params/contiguity_edge_types")
        .or_else(|| j_str(fr, "/edge_policy")).unwrap_or_else(|| "land".into());
    let island_rule = j_str(result, "/params/island_exception_rule")
        .or_else(|| j_str(fr, "/island_rule")).unwrap_or_else(|| "none".into());

    let bands_summary = fr.pointer("/bands_summary")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect::<Vec<_>>())
        .unwrap_or_default();

    let counters = FrontierCounters {
        changed: j_u64(fr, "/summary/changed").unwrap_or(0) as u32,
        no_change: j_u64(fr, "/summary/no_change").unwrap_or(0) as u32,
        mediation: j_u64(fr, "/summary/mediation").unwrap_or(0) as u32,
        enclave: j_u64(fr, "/summary/enclave").unwrap_or(0) as u32,
        protected_blocked: j_u64(fr, "/summary/protected_blocked").unwrap_or(0) as u32,
        quorum_blocked: j_u64(fr, "/summary/quorum_blocked").unwrap_or(0) as u32,
    };

    FrontierBlock { mode, edge_types, island_rule, bands_summary, counters }
}

pub fn map_sensitivity(_result: &ResultDb) -> Option<SensitivityBlock> {
    // v1: no scenario compare; return None for a lean model.
    None
}

pub fn map_integrity_footer(
    run: &RunRecordDb,
    result: &ResultDb,
    frontier: Option<&FrontierMapDb>
) -> (IntegrityBlock, FooterIds) {
    let engine_vendor  = j_str(run, "/engine/vendor").unwrap_or_else(|| "vm-engine".into());
    let engine_name    = j_str(run, "/engine/name").unwrap_or_else(|| "vm".into());
    let engine_version = j_str(run, "/engine/version").unwrap_or_else(|| "0.1.0".into());
    let engine_build   = j_str(run, "/engine/build").unwrap_or_else(|| "dev".into());
    let formula_id_hex = j_str(run, "/formula_id")
        .or_else(|| j_str(result, "/formula_id"))
        .unwrap_or_else(|| "unknown".into());
    let tie_policy     = j_str(run, "/determinism/tie_policy").unwrap_or_else(|| "deterministic".into());
    let tie_seed       = if tie_policy == "random" { j_str(run, "/determinism/rng_seed") } else { None };
    let started_utc    = j_str(run, "/started_utc").unwrap_or_else(|| "1970-01-01T00:00:00Z".into());
    let finished_utc   = j_str(run, "/finished_utc").unwrap_or_else(|| "1970-01-01T00:00:00Z".into());

    let integrity = IntegrityBlock {
        engine_vendor, engine_name, engine_version, engine_build,
        formula_id_hex, tie_policy, tie_seed, started_utc, finished_utc,
    };

    let result_id   : ResultId   = j_str(run, "/outputs/result_id").unwrap_or_else(|| "RES:unknown".into()).into();
    let run_id      : RunId      = j_str(run, "/id").unwrap_or_else(|| "RUN:unknown".into()).into();
    let frontier_id : Option<FrontierId> = frontier
        .and_then(|_| j_str(run, "/outputs/frontier_map_id"))
        .map(Into::into);

    let reg_id      : RegId      = j_str(run, "/inputs/reg_id").unwrap_or_else(|| "REG:unknown".into()).into();
    let param_set_id: ParamSetId = j_str(run, "/inputs/parameter_set_id").unwrap_or_else(|| "PS:unknown".into()).into();
    let tally_id    : Option<TallyId> = j_str(run, "/inputs/ballot_tally_id").map(Into::into);

    let footer = FooterIds {
        result_id, run_id, frontier_id, reg_id, param_set_id, tally_id
    };

    (integrity, footer)
}

// ===================== Formatting helpers (integer math; no floats) =====================

/// Format % to one decimal using integer tenths from `percent_one_decimal_tenths`.
pub fn pct_1dp(num: i128, den: i128) -> String {
    if den <= 0 { return "0.0%".into(); }
    let tenths = percent_one_decimal_tenths(num, den);
    let whole = tenths / 10;
    let frac  = (tenths % 10).abs();
    format!("{}.{}%", whole, frac)
}
/// "55%"
pub fn pct0(value_u8: u8) -> String { format!("{}%", value_u8) }

/// "+3 pp" / "-2 pp"
pub fn pp_signed(pp_i32: i32) -> String {
    if pp_i32 >= 0 { format!("+{} pp", pp_i32) } else { format!("{} pp", pp_i32) }
}

// ===================== Small JSON helpers (pure) =====================

fn j_str(root: &serde_json::Value, ptr: &str) -> Option<String> {
    root.pointer(ptr).and_then(|v| v.as_str().map(|s| s.to_string()))
}
fn j_u64(root: &serde_json::Value, ptr: &str) -> Option<u64> {
    root.pointer(ptr).and_then(|v| v.as_u64())
}
fn j_i64(root: &serde_json::Value, ptr: &str) -> Option<i64> {
    root.pointer(ptr).and_then(|v| v.as_i64())
}
fn j_bool(root: &serde_json::Value, ptr: &str) -> Option<bool> {
    root.pointer(ptr).and_then(|v| v.as_bool())
}

/// Convert a JSON number 0..=100 (stringified) to one-decimal percent **without**
/// float arithmetic. If not present, returns "0.0%".
fn percent_number_to_1dp_str(maybe: Option<&serde_json::Value>) -> String {
    let s = match maybe {
        Some(serde_json::Value::Number(n)) => n.to_string(),
        _ => return "0.0%".into(),
    };
    // Ensure exactly one decimal place (truncate if more; add ".0" if none).
    if let Some(dot) = s.find('.') {
        let after = &s[dot + 1..];
        if after.is_empty() {
            format!("{}0%", s)
        } else {
            let end = dot + 2.min(s.len() - dot - 1);
            let mut out = String::with_capacity(dot + 2 + 1);
            out.push_str(&s[..=dot]); // include '.'
            out.push_str(&s[dot + 1..end]);
            out.push('%');
            out
        }
    } else {
        format!("{}.0%", s)
    }
}
