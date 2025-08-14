
//! vm_report main library — Part 1/2
//!
//! Aligned to Docs 1–7 and Annexes A–C:
//!   • Renderer reads canonical artifacts only (no recompute)
//!   • Outcome-affecting vars come from RunRecord.vars_effective
//!   • Footer uses *_sha256 digests for inputs
//!   • Tie policy token matches VM-VAR-050 vocabulary

use serde::{Deserialize, Serialize};
use serde_json::Value;
use vm_io::RunRecord;

/// Snapshot for the cover section (Doc 7)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverSnapshot {
    pub fid: String,
    pub engine_version: String,
    pub variant: Option<String>,
    pub created_at: Option<String>, // from Result.created_at
    pub jurisdiction: Option<String>,
    pub election_name: Option<String>,
}

/// Legitimacy gate pass/fail panel (Doc 4B / 5C)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatePanel {
    pub label: String,
    pub pass: bool,
    pub reasons: Vec<String>,
    pub denom_note: Option<String>,
}

/// Frontier appendix (Doc 5C, VM-VAR-034=true)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierCounters {
    pub units_total: u64,
    pub units_passed: u64,
    pub edges_total: u64,
    pub edges_passed: u64,
}

/// Footer IDs/digests (Doc 7 integrity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterIntegrity {
    pub result_id: String,              // "RES:..."
    pub run_id: String,                 // "RUN:..."
    pub frontier_id: Option<String>,    // "FR:..." if present
    pub registry_sha256: String,
    pub tally_sha256: String,
    pub params_sha256: String,
    pub tie_policy: Option<String>,     // "status_quo" | "deterministic_order" | "random"
    pub tie_seed: Option<u64>,          // only if policy == "random"
}

/// Top-level report model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportModel {
    pub cover: CoverSnapshot,
    pub gates: Vec<GatePanel>,
    pub frontier: Option<FrontierCounters>,
    pub footer: FooterIntegrity,
}

/// Helper: fetch string at JSON Pointer
#[inline]
pub fn j_str(v: &Value, ptr: &str) -> Option<String> {
    v.pointer(ptr).and_then(|x| x.as_str()).map(|s| s.to_string())
}

/// Helper: parse integer at JSON Pointer
#[inline]
pub fn j_u64(v: &Value, ptr: &str) -> Option<u64> {
    v.pointer(ptr).and_then(|x| x.as_u64())
}

/// Map cover snapshot from canonical Result + RunRecord
pub fn map_cover_snapshot(result: &Value, run: &RunRecord) -> CoverSnapshot {
    CoverSnapshot {
        fid: run.fid.clone(),
        engine_version: run.engine_version.clone(),
        variant: run.variant.clone(),
        created_at: j_str(result, "/created_at"),
        jurisdiction: j_str(result, "/jurisdiction"),
        election_name: j_str(result, "/election_name"),
    }
}

/// Map gates panel list from canonical Result.gates
pub fn map_gates(result: &Value) -> Vec<GatePanel> {
    let mut panels = Vec::new();
    if let Some(gates_obj) = result.pointer("/gates").and_then(|x| x.as_object()) {
        for (gate_name, gate_val) in gates_obj {
            let pass = gate_val.get("pass").and_then(|x| x.as_bool()).unwrap_or(false);
            let reasons = gate_val
                .get("reasons")
                .and_then(|x| x.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            // No hard-coded denom_note — only if engine provided
            let denom_note = gate_val
                .get("denom_note")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());

            panels.push(GatePanel {
                label: gate_name.clone(),
                pass,
                reasons,
                denom_note,
            });
        }
    }
    panels
}

/// Map frontier counters from canonical FrontierMap (if VM-VAR-034=true)
pub fn map_frontier(frontier: &Value) -> Option<FrontierCounters> {
    let units_total = j_u64(frontier, "/units_total")?;
    let units_passed = j_u64(frontier, "/units_passed")?;
    let edges_total = j_u64(frontier, "/edges_total")?;
    let edges_passed = j_u64(frontier, "/edges_passed")?;
    Some(FrontierCounters {
        units_total,
        units_passed,
        edges_total,
        edges_passed,
    })
}

/// Map footer integrity block from RunRecord (+ optional vars_effective)
pub fn map_footer(_result: &Value, run: &RunRecord) -> FooterIntegrity {
    // Outcome-affecting vars are echoed via RunRecord.vars_effective.
    // We read the canonical strings and normalize policy tokens at the source (engine).
    let tie_policy = run.vars_effective.get("tie_policy").cloned();
    let tie_seed = run
        .vars_effective
        .get("tie_seed")
        .and_then(|s| s.parse::<u64>().ok());

    FooterIntegrity {
        result_id: run.result_id.clone(),
        run_id: run.run_id.clone(),
        frontier_id: run.frontier_id.clone(),
        registry_sha256: run.inputs.registry_sha256.clone(),
        tally_sha256: run.inputs.tally_sha256.clone(),
        params_sha256: run.inputs.params_sha256.clone(),
        tie_policy,
        tie_seed,
    }
}
//! vm_report main library — Part 2/2
//! Completes the report mappers and provides a single entry to build the model
//! from canonical artifacts. Includes safe, spec-aligned percent formatting.

/* --------------------------- Report assembly entrypoint --------------------------- */

/// Build the full `ReportModel` from canonical artifacts.
/// - `result_json`: parsed `result.json`
/// - `run`: parsed `run_record.json`
/// - `frontier_map`: parsed `frontier_map.json` **iff** it was emitted
///
/// Notes:
/// • We only include the frontier appendix if a `frontier_map` was provided.
/// • No recomputation: all fields are echoed from canonical artifacts.
pub fn build_report_model(
    result_json: &serde_json::Value,
    run: &vm_io::RunRecord,
    frontier_map: Option<&serde_json::Value>,
) -> ReportModel {
    let cover: CoverSnapshot = map_cover_snapshot(result_json, run);
    let gates: Vec<GatePanel> = map_gates(result_json);
    let frontier: Option<FrontierCounters> = frontier_map.and_then(map_frontier);
    let footer: FooterIntegrity = map_footer(result_json, run);

    ReportModel { cover, gates, frontier, footer }
}

/* --------------------------- Presentation utilities ------------------------------ */

/// Format a fraction `x` (0.0..=1.0) as a percentage with **one decimal place**,
/// round-half-up, ASCII-only, locale-neutral. Returns `"—"` if `x` is NaN/∞/out of range.
pub fn percent_1dp(x: f64) -> String {
    if !x.is_finite() || x < 0.0 || x > 1.0 {
        return "—".to_string();
    }
    // Round-half-up at one decimal place for percentage (×100).
    // Add a tiny epsilon to emulate half-up rather than bankers rounding.
    let v = x * 100.0;
    let scaled = (v * 10.0 + 0.5_f64).floor() / 10.0;
    format!("{scaled:.1}%")
}

/// Attempt to parse a JSON value into f64 robustly (number or numeric string).
#[inline]
pub fn json_number_to_f64(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

/// Convenience: read a fraction at `ptr` from `obj` and format with `percent_1dp`.
/// Returns `None` if the pointer is missing or not a number.
pub fn percent_at(obj: &serde_json::Value, ptr: &str) -> Option<String> {
    obj.pointer(ptr)
        .and_then(json_number_to_f64)
        .map(percent_1dp)
}

/* ------------------------------------- Tests -------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_formats_round_half_up() {
        assert_eq!(percent_1dp(0.0), "0.0%");
        assert_eq!(percent_1dp(1.0), "100.0%");
        // 12.34% → 12.3%
        assert_eq!(percent_1dp(0.1234), "12.3%");
        // 12.35% → 12.4% (half-up)
        assert_eq!(percent_1dp(0.1235), "12.4%");
        // out-of-range / non-finite
        assert_eq!(percent_1dp(f64::NAN), "—");
        assert_eq!(percent_1dp(-0.01), "—");
        assert_eq!(percent_1dp(1.01), "—");
    }

    #[test]
    fn json_number_parsing() {
        assert_eq!(json_number_to_f64(&serde_json::Value::from(0.25f64)).unwrap(), 0.25);
        assert_eq!(json_number_to_f64(&serde_json::Value::from("0.25")).unwrap(), 0.25);
        assert!(json_number_to_f64(&serde_json::Value::Null).is_none());
    }

    #[test]
    fn assemble_minimal_report() {
        // Minimal result.json with gates {}
        let result = serde_json::json!({
            "created_at": "2025-08-12T10:00:00Z",
            "gates": {}
        });
        // Minimal RunRecord
        let run = vm_io::RunRecord {
            fid: "FID:deadbeef".into(),
            engine_version: "VM-ENGINE v0".into(),
            variant: None,
            result_id: "RES:abc".into(),
            run_id: "RUN:abc".into(),
            frontier_id: None,
            inputs: vm_io::InputDigests {
                registry_sha256: "r".into(),
                tally_sha256: "t".into(),
                params_sha256: "p".into(),
                frontier_inputs_sha256: None,
            },
            vars_effective: std::collections::BTreeMap::from([]),
        };

        let model = build_report_model(&result, &run, None);
        assert_eq!(model.cover.created_at.as_deref(), Some("2025-08-12T10:00:00Z"));
        assert!(model.frontier.is_none());
        assert_eq!(model.footer.result_id, "RES:abc");
    }
}
