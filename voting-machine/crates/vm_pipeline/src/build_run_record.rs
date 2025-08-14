// crates/vm_pipeline/src/build_run_record.rs — Half 1/2
//
// Foundations for building the RunRecord and Integrity sections.
// This half is self-contained: types, errors, helpers for canonical IDs,
// and a thin public API surface (the full population happens in Half 2/2).
//
// Spec anchors (Docs 1–7 + Annexes A–C):
// - Result ID (RID) = SHA-256 of the canonical **result.json** payload.
// - Run ID          = SHA-256 of the canonical **run_record.json** payload
//                     (computed after full assembly; see Half 2/2).
// - Formula ID (FID)= SHA-256 of the canonical **Included-only VM-VARs**
//                     normative manifest (Annex A). VM-VAR-052 (tie seed) is
//                     explicitly **Excluded** from FID.
// - Frontier ID     = SHA-256 of the canonical **frontier_map.json** if present.
// - Engine meta is recorded verbatim (vendor/name/version/build).
// - Tie seed (VM-VAR-052) must be **logged** in Integrity, but **excluded** from FID.
//
// Design notes:
// - We canonically hash **JSON values** by recursively sorting object keys,
//   then serializing with `serde_json::to_string` (stable given that order).
//   This avoids pulling vm_io::canonical_json here and keeps this module
//   independent of I/O. Keep the same algorithm everywhere hashing IDs.

use std::fmt;

use crate::EngineMeta;
use serde_json::{Map as JsonMap, Value as Json};

/// Errors while building canonical IDs or assembling the run record.
#[derive(Debug)]
pub enum BuildRecordError {
    Canonicalize(&'static str),
    Hash(&'static str),
    Json(String),
    Unimplemented(&'static str),
}

impl fmt::Display for BuildRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BuildRecordError::*;
        match self {
            Canonicalize(m) => write!(f, "canonicalization error: {m}"),
            Hash(m) => write!(f, "hash error: {m}"),
            Json(m) => write!(f, "json error: {m}"),
            Unimplemented(m) => write!(f, "not yet implemented: {m}"),
        }
    }
}
impl std::error::Error for BuildRecordError {}

/// Convenience result alias.
pub type BuildResult<T> = Result<T, BuildRecordError>;

/// Integrity block as required by Doc 7.
/// Note: `run_id_hex` is filled in Half 2/2 after the full run_record is assembled.
#[derive(Debug, Clone)]
pub struct IntegrityFields {
    pub result_id_hex: String,
    pub run_id_hex: Option<String>,
    pub formula_id_hex: String,
    pub frontier_id_hex: Option<String>,
    pub engine_vendor: String,
    pub engine_name: String,
    pub engine_version: String,
    pub engine_build: String,
    /// VM-VAR-052 (excluded from FID); logged for determinism provenance.
    pub tie_seed_hex: Option<String>,
}

/// Build IntegrityFields from component documents.
/// - `result_json`     : full result payload (already canonical or canonicalizable)
/// - `nm_included_json`: **Included-only** normative manifest (Annex A)
/// - `frontier_json`   : frontier map payload (if any)
/// - `engine_meta`     : engine identity/version/build
/// - `tie_seed`        : optional seed (VM-VAR-052) to log (not part of FID)
pub fn compute_integrity_fields(
    result_json: &Json,
    nm_included_json: &Json,
    frontier_json: Option<&Json>,
    engine_meta: &EngineMeta,
    tie_seed: Option<u64>,
) -> BuildResult<IntegrityFields> {
    let result_id_hex = sha256_hex_of_canonical_json(result_json)?;
    let formula_id_hex = sha256_hex_of_canonical_json(nm_included_json)?;
    let frontier_id_hex = match frontier_json {
        Some(v) => Some(sha256_hex_of_canonical_json(v)?),
        None => None,
    };
    let tie_seed_hex = tie_seed.map(seed_hex64);

    Ok(IntegrityFields {
        result_id_hex,
        run_id_hex: None, // filled after full run_record is assembled (Half 2/2)
        formula_id_hex,
        frontier_id_hex,
        engine_vendor: engine_meta.vendor.clone(),
        engine_name: engine_meta.name.clone(),
        engine_version: engine_meta.version.clone(),
        engine_build: engine_meta.build.clone(),
        tie_seed_hex,
    })
}

/// Compute the **Run ID** from a fully-assembled run record JSON (Half 2/2 calls this).
pub fn compute_run_id_hex(run_record_json: &Json) -> BuildResult<String> {
    sha256_hex_of_canonical_json(run_record_json)
}

// --------------------------------------------------------------------------------------
// Canonical JSON → SHA-256 helpers (no I/O; stable across platforms)
// --------------------------------------------------------------------------------------

/// Produce a canonicalized clone of `value` with all object keys sorted
/// recursively (arrays preserve order; scalars unchanged).
fn canonicalize_json(value: &Json) -> Json {
    match value {
        Json::Null | Json::Bool(_) | Json::Number(_) | Json::String(_) => value.clone(),
        Json::Array(items) => {
            let canon_items = items.iter().map(canonicalize_json).collect::<Vec<_>>();
            Json::Array(canon_items)
        }
        Json::Object(map) => {
            // Rebuild as a new Object with sorted keys
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = JsonMap::new();
            for k in keys {
                let v = map.get(k).expect("key from keys() must exist");
                out.insert(k.clone(), canonicalize_json(v));
            }
            Json::Object(out)
        }
    }
}

/// Compute lower-case 64-hex SHA-256 of the canonical JSON value.
fn sha256_hex_of_canonical_json(value: &Json) -> BuildResult<String> {
    let canon = canonicalize_json(value);
    let s = serde_json::to_string(&canon).map_err(|_| BuildRecordError::Canonicalize("serialize"))?;
    Ok(sha256_hex(s.as_bytes()))
}

/// Compute lower-case 64-hex SHA-256 of `bytes`.
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    // hex encode
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push(hex_digit((b >> 4) & 0x0F));
        out.push(hex_digit(b & 0x0F));
    }
    out
}

#[inline]
fn hex_digit(n: u8) -> char {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    HEX[n as usize] as char
}

/// Render VM-VAR-052 as fixed-width 16-hex uppercase (human-facing).
/// Note: We keep IDs (hashes) lower-case; seeds are often shown uppercase.
fn seed_hex64(seed: u64) -> String {
    format!("{seed:016X}")
}

// --------------------------------------------------------------------------------------
// Public API surface (population comes in Half 2/2)
// --------------------------------------------------------------------------------------

/// Build the full run_record JSON (Doc 7) from normalized inputs.
/// This is a **thin entrypoint**; Half 2/2 will implement the population.
/// Returning an error here keeps this file compilable without partial structs.
///
/// Inputs expected (normalized by earlier pipeline stages):
/// - `result_json`      : canonical result payload
/// - `run_inputs_json`  : minimal provenance bundle (paths/digests/timestamps) if you track it
/// - `nm_included_json` : Included-only VM-VARs manifest (Annex A; already normalized)
/// - `frontier_json`    : optional frontier map
/// - `engine_meta`      : engine identity
/// - `tie_seed`         : VM-VAR-052 (logged only; excluded from FID)
/// - `gates_summary`    : optional precomputed gates summary block
#[allow(unused)]
pub fn build_run_record(
    result_json: &Json,
    run_inputs_json: &Json,
    nm_included_json: &Json,
    frontier_json: Option<&Json>,
    engine_meta: &EngineMeta,
    tie_seed: Option<u64>,
    gates_summary: Option<&Json>,
) -> BuildResult<Json> {
    // Half 2/2 will assemble the full structure and then call `compute_run_id_hex`.
    let _ = (result_json, run_inputs_json, nm_included_json, frontier_json, engine_meta, tie_seed, gates_summary);
    Err(BuildRecordError::Unimplemented("Half 2/2 will populate run_record"))
}
// crates/vm_pipeline/src/build_run_record.rs — Half 2/2
//
// Full population of the run_record JSON (Doc 7) using the helpers from Half 1/2.
// - Builds Integrity (result_id/formula_id/frontier_id/engine/tie_seed)
// - Assembles the full run_record body (inputs, included-only manifest, frontier, gates)
// - Computes run_id over the **run_record without the run_id field**, then injects run_id
//   (so the hash is stable and well-defined).

use serde_json::{json, Map as JsonMap, Value as Json};

/// Assemble the complete run_record (Doc 7) with a correct run_id.
///
/// Inputs:
/// - `result_json`      : canonical result payload
/// - `run_inputs_json`  : provenance bundle (paths/digests/timestamps) if tracked
/// - `nm_included_json` : Included-only VM-VAR manifest (Annex A; already normalized)
/// - `frontier_json`    : optional frontier map
/// - `engine_meta`      : engine identity
/// - `tie_seed`         : VM-VAR-052 (logged only; excluded from FID)
/// - `gates_summary`    : optional summary block
///
/// Output:
/// - Fully assembled JSON with:
///     { integrity: {...}, inputs: {...}, nm_included: {...}, frontier_map?: {...}, gates_summary?: {...} }
pub fn build_run_record(
    result_json: &Json,
    run_inputs_json: &Json,
    nm_included_json: &Json,
    frontier_json: Option<&Json>,
    engine_meta: &crate::EngineMeta,
    tie_seed: Option<u64>,
    gates_summary: Option<&Json>,
) -> crate::build_run_record::BuildResult<Json> {
    // 1) Integrity (without run_id for now)
    let integ = super::compute_integrity_fields(
        result_json,
        nm_included_json,
        frontier_json,
        engine_meta,
        tie_seed,
    )?;

    // 2) Build run_record **without** run_id (this is what we hash to get run_id)
    let run_record_wo_run_id = assemble_run_record_json(&integ, false, run_inputs_json, nm_included_json, frontier_json, gates_summary)?;

    // 3) Compute run_id over the canonicalized run_record without run_id
    let run_id_hex = super::compute_run_id_hex(&run_record_wo_run_id)?;

    // 4) Rebuild integrity **with** run_id, then assemble the final run_record
    let run_record = {
        // clone the integrity with run_id set
        let integ_with_run = IntegrityJson {
            result_id_hex: integ.result_id_hex.clone(),
            run_id_hex: Some(run_id_hex),
            formula_id_hex: integ.formula_id_hex.clone(),
            frontier_id_hex: integ.frontier_id_hex.clone(),
            engine_vendor: integ.engine_vendor.clone(),
            engine_name: integ.engine_name.clone(),
            engine_version: integ.engine_version.clone(),
            engine_build: integ.engine_build.clone(),
            tie_seed_hex: integ.tie_seed_hex.clone(),
        };
        assemble_run_record_json_raw(&integ_with_run, run_inputs_json, nm_included_json, frontier_json, gates_summary)
    };

    Ok(run_record)
}

// ------------------- internal helpers -------------------

/// Mirror of IntegrityFields for JSON emission with control over including run_id.
#[derive(Debug, Clone)]
struct IntegrityJson {
    result_id_hex: String,
    run_id_hex: Option<String>,
    formula_id_hex: String,
    frontier_id_hex: Option<String>,
    engine_vendor: String,
    engine_name: String,
    engine_version: String,
    engine_build: String,
    tie_seed_hex: Option<String>,
}

/// Assemble run_record object with a choice to include or omit run_id in integrity.
fn assemble_run_record_json(
    integ: &super::IntegrityFields,
    include_run_id: bool,
    run_inputs_json: &Json,
    nm_included_json: &Json,
    frontier_json: Option<&Json>,
    gates_summary: Option<&Json>,
) -> crate::build_run_record::BuildResult<Json> {
    let tmp = IntegrityJson {
        result_id_hex: integ.result_id_hex.clone(),
        run_id_hex: if include_run_id { integ.run_id_hex.clone() } else { None },
        formula_id_hex: integ.formula_id_hex.clone(),
        frontier_id_hex: integ.frontier_id_hex.clone(),
        engine_vendor: integ.engine_vendor.clone(),
        engine_name: integ.engine_name.clone(),
        engine_version: integ.engine_version.clone(),
        engine_build: integ.engine_build.clone(),
        tie_seed_hex: integ.tie_seed_hex.clone(),
    };
    Ok(assemble_run_record_json_raw(
        &tmp,
        run_inputs_json,
        nm_included_json,
        frontier_json,
        gates_summary,
    ))
}

/// Build the final JSON object from parts (caller decides whether `run_id` is present).
fn assemble_run_record_json_raw(
    integ: &IntegrityJson,
    run_inputs_json: &Json,
    nm_included_json: &Json,
    frontier_json: Option<&Json>,
    gates_summary: Option<&Json>,
) -> Json {
    // integrity
    let mut integrity = JsonMap::new();
    integrity.insert("result_id".to_string(), Json::String(integ.result_id_hex.clone()));
    if let Some(run) = &integ.run_id_hex {
        integrity.insert("run_id".to_string(), Json::String(run.clone()));
    }
    integrity.insert("formula_id_hex".to_string(), Json::String(integ.formula_id_hex.clone()));
    if let Some(fr) = &integ.frontier_id_hex {
        integrity.insert("frontier_id".to_string(), Json::String(fr.clone()));
    }
    integrity.insert("engine_vendor".to_string(), Json::String(integ.engine_vendor.clone()));
    integrity.insert("engine_name".to_string(), Json::String(integ.engine_name.clone()));
    integrity.insert("engine_version".to_string(), Json::String(integ.engine_version.clone()));
    integrity.insert("engine_build".to_string(), Json::String(integ.engine_build.clone()));
    if let Some(seed) = &integ.tie_seed_hex {
        integrity.insert("tie_seed".to_string(), Json::String(seed.clone()));
    }

    // root object
    let mut root = JsonMap::new();
    root.insert("integrity".to_string(), Json::Object(integrity));
    root.insert("inputs".to_string(), run_inputs_json.clone());
    // embed Included-only manifest for audit (Annex A)
    root.insert("nm_included".to_string(), nm_included_json.clone());
    if let Some(fr) = frontier_json {
        root.insert("frontier_map".to_string(), fr.clone());
    }
    if let Some(gs) = gates_summary {
        root.insert("gates_summary".to_string(), gs.clone());
    }

    Json::Object(root)
}
