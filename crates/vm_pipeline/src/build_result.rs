//! build_result.rs — Part 1/2
//! Types, inputs, and validators to build a canonical Result artifact.
//! Part 2 will assemble the idless payload, compute sha256 → RES:<sha>,
//! and return (ResultDoc, result_sha256).

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{self as json};

use crate::{ResultDoc, ResultSummary, UnitResult};

/// ---------- Errors specific to building a Result ----------
#[derive(Debug)]
pub enum BuildResultError {
    BadUtc(&'static str, String),
    BadHex64(&'static str, String),
    UnitsOrder(String),
    Spec(String),
}

/// ---------- Inputs for building a Result (caller provides units + tie count) ----------
#[derive(Debug, Clone)]
pub struct ResultInputs {
    /// FID (sha256 of the Normative Manifest) — hex64 lowercase.
    pub formula_id: String,
    /// Engine version token, e.g., "VM-ENGINE v0".
    pub engine_version: String,
    /// RFC3339 UTC; will be normalized and echoed as `created_at`.
    pub created_at_utc: String,
    /// Ordered per-unit outcomes (must already be in canonical order).
    pub units: Vec<UnitResult>,
    /// Presentation-only label (already computed by caller per Doc 7 rules).
    pub label: Option<String>,
    /// Total number of tie events encountered during allocation (for summary).
    pub tie_count: u64,
}

/// ---------- Internal: idless Result shape used for canonical hashing ----------
#[derive(Debug, Clone, Serialize)]
struct ResultNoId {
    pub formula_id: String,
    pub engine_version: String,
    pub created_at: String,        // normalized RFC3339 Z
    pub summary: ResultSummary,
    pub units: Vec<UnitResult>,    // ordered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,     // presentation-only
}

/// ---------- Validators & helpers (pure, deterministic) ----------

/// Parse+normalize an RFC3339 UTC (must end with 'Z'); returns normalized string.
pub fn normalize_rfc3339_utc(name: &'static str, ts: &str) -> Result<String, BuildResultError> {
    let dt: DateTime<Utc> = ts
        .parse::<DateTime<Utc>>()
        .map_err(|_| BuildResultError::BadUtc(name, ts.to_string()))?;
    Ok(dt.to_rfc3339_opts(SecondsFormat::Secs, true))
}

/// Lowercase hex64 check.
pub fn is_hex64(s: &str) -> bool {
    if s.len() != 64 {
        return false;
    }
    s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

pub fn validate_hex64(name: &'static str, s: &str) -> Result<(), BuildResultError> {
    if is_hex64(s) {
        Ok(())
    } else {
        Err(BuildResultError::BadHex64(name, s.to_string()))
    }
}

/// Ensure units are in canonical order (lexicographic by unit_id).
/// We do not reorder here; we fail fast to keep the pipeline explicit.
pub fn check_units_canonical_order(units: &[UnitResult]) -> Result<(), BuildResultError> {
    for i in 1..units.len() {
        if units[i - 1].unit_id > units[i].unit_id {
            return Err(BuildResultError::UnitsOrder(format!(
                "units not sorted: {} > {} at index {}",
                units[i - 1].unit_id, units[i].unit_id, i
            )));
        }
    }
    Ok(())
}

/// Compute the summary block deterministically.
pub fn compute_summary(units: &[UnitResult], tie_count: u64) -> ResultSummary {
    let unit_count = units.len() as u64;
    let allocation_count = units
        .iter()
        .filter(|u| u.assigned_to.is_some())
        .count() as u64;
    ResultSummary {
        unit_count,
        allocation_count,
        tie_count,
    }
}

/// Validate high-level inputs and return normalized fields ready for assembly.
pub fn validate_result_inputs(inp: &ResultInputs) -> Result<(String, String), BuildResultError> {
    // FID sanity
    validate_hex64("formula_id (sha256)", &inp.formula_id)?;
    // Engine version must not be empty
    if inp.engine_version.trim().is_empty() {
        return Err(BuildResultError::Spec("engine_version is empty".into()));
    }
    // Normalize timestamp
    let created_at_norm = normalize_rfc3339_utc("created_at_utc", &inp.created_at_utc)?;
    // Units canonical order
    check_units_canonical_order(&inp.units)?;
    Ok((inp.formula_id.clone(), created_at_norm))
}
//! build_result.rs — Part 2/2
//! Assemble idless payload → canonical bytes → sha256,
//! form "RES:<sha256>", self-verify, and return (ResultDoc, result_sha256).

use std::path::Path;
use std::fs::File;
use std::io::Write;

use vm_io::{
    canonical_json::to_canonical_bytes,
    hasher::sha256_hex,
};

use crate::{ResultDoc, ResultSummary, UnitResult};

use super::{
    BuildResultError,
    ResultInputs,
    ResultNoId,
    compute_summary,
    validate_result_inputs,
};

/// Build the canonical Result artifact and return `(ResultDoc, result_sha256)`.
pub fn build_result(inp: ResultInputs) -> Result<(ResultDoc, String), BuildResultError> {
    // ---- Validate & normalize inputs ----
    let (_fid_ok, created_at_norm) = validate_result_inputs(&inp)?;
    let summary: ResultSummary = compute_summary(&inp.units, inp.tie_count);

    // ---- Idless payload (used for canonical hashing) ----
    let noid = ResultNoId {
        formula_id: inp.formula_id.clone(),
        engine_version: inp.engine_version.clone(),
        created_at: created_at_norm.clone(),
        summary,
        units: inp.units.clone(),
        label: inp.label.clone(),
    };

    // ---- Canonical bytes → sha256 ----
    let canon_bytes = to_canonical_bytes(&noid)
        .map_err(|_| BuildResultError::Spec("canonicalization failed".into()))?;
    let res_sha = sha256_hex(&canon_bytes);
    let res_id = format!("RES:{res_sha}");

    // ---- Assemble final document ----
    let doc = ResultDoc {
        id: res_id.clone(),
        formula_id: noid.formula_id,
        engine_version: noid.engine_version,
        created_at: created_at_norm,
        summary: noid.summary,
        units: noid.units,
        label: noid.label,
    };

    // ---- Self-verify (light) ----
    if !doc.id.starts_with("RES:") || &doc.id[4..] != res_sha {
        return Err(BuildResultError::Spec("Result ID does not match canonical bytes".into()));
    }

    Ok((doc, res_sha))
}

/// Convenience: get canonical bytes of a finalized Result (useful for writing/tests).
pub fn result_canonical_bytes(doc: &ResultDoc) -> Result<Vec<u8>, BuildResultError> {
    to_canonical_bytes(doc)
        .map_err(|_| BuildResultError::Spec("canonicalization failed".into()))
}

/// Optional helper to write a finalized Result to disk as canonical JSON (LF).
pub fn write_result(path: &Path, doc: &ResultDoc) -> Result<(), BuildResultError> {
    let bytes = result_canonical_bytes(doc)?;
    let mut f = File::create(path).map_err(|e| BuildResultError::Spec(format!("io: {e}")))?;
    f.write_all(&bytes).map_err(|e| BuildResultError::Spec(format!("io: {e}")))?;
    Ok(())
}
