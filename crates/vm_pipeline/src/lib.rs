//! VM Engine — core artifacts & canonicalization (Part 1/2)
//! This file provides:
//! - Canonical JSON utilities (stable key ordering, LF endings)
//! - SHA-256 helpers and ID prefixes (RES:, RUN:, FR:)
//! - Core data types (ResultDoc, RunRecordDoc, FrontierMapDoc)
//! - “NoId → WithId” builders for canonical artifacts
//!
//! Part 2 adds orchestration (run functions, self-verify, file IO).

use sha2::{Digest, Sha256};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{self as json, Value};

/// ----- Error type -----
#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] json::Error),
    #[error("Timestamp must be RFC3339 with Z (UTC): {0}")]
    BadTimestamp(String),
    #[error("Spec violation: {0}")]
    Spec(String),
    #[error("Internal: {0}")]
    Internal(String),
}

/// ----- Tie policy (determinism contract) -----
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TiePolicy {
    /// Deterministic based on the canonical order (e.g., unit_id then order_index).
    #[serde(rename = "deterministic_order")]
    DeterministicOrder,
    /// Uses RNG only when a real tie occurs.
    #[serde(rename = "random")]
    Random,
}

/// Determinism echo in RunRecord (what policy was in effect and whether RNG took place).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Determinism {
    pub tie_policy: TiePolicy,
    /// Present only if the policy is random *and* at least one random tie occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<u64>,
}

/// ----- Canonical JSON helpers -----

/// Recursively sort object keys to guarantee deterministic serialization.
/// Arrays retain order; numbers/strings/booleans are passed through.
fn canonicalize_value(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = json::Map::new();
            for k in keys {
                out.insert(k.clone(), canonicalize_value(&map[k]));
            }
            Value::Object(out)
        }
        Value::Array(a) => {
            let mut out = Vec::with_capacity(a.len());
            for item in a {
                out.push(canonicalize_value(item));
            }
            Value::Array(out)
        }
        _ => v.clone(),
    }
}

/// Convert any Serialize into canonical, LF-terminated UTF-8 bytes.
fn to_canonical_bytes<T: Serialize>(t: &T) -> Result<Vec<u8>, EngineError> {
    let v = json::to_value(t)?;
    let c = canonicalize_value(&v);
    let mut s = json::to_string(&c)?;
    if !s.ends_with('\n') {
        s.push('\n');
    }
    Ok(s.into_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    hex::encode(digest)
}

/// Validate and normalize an RFC3339 UTC timestamp (must end with 'Z').
fn normalize_timestamp_utc(ts: &str) -> Result<String, EngineError> {
    let dt: DateTime<Utc> = ts.parse::<DateTime<Utc>>()
        .map_err(|_| EngineError::BadTimestamp(ts.to_string()))?;
    Ok(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

/// ----- Artifact IDs & wrappers -----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdOnly {
    pub id: String,
    pub sha256: String,
}

/// Result.json (canonical, outcome-carrying summary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultDoc {
    pub id: String,                // "RES:<sha256>"
    pub formula_id: String,        // FID (sha256 of Normative Manifest)
    pub engine_version: String,    // e.g., "VM-ENGINE v0"
    pub created_at: String,        // RFC3339 UTC
    pub summary: ResultSummary,
    pub units: Vec<UnitResult>,    // ordered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,     // presentation-only; computed post-allocation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSummary {
    pub unit_count: u64,
    pub allocation_count: u64,
    pub tie_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitResult {
    pub unit_id: String,
    /// Example outcome payload (keep minimal here; align your model as needed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

/// RunRecord (canonical trace of a run).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecordDoc {
    pub run_id: String,            // "RUN:<timestamp>-<sha256>"
    pub timestamp_utc: String,     // RFC3339 UTC
    pub formula_id: String,        // same FID as in ResultDoc
    pub determinism: Determinism,  // tie policy + rng echo (if any)
    pub inputs: InputsEcho,        // hashes/digests of inputs used
    pub vars_effective: json::Map<String, Value>, // Included vars actually used
    pub outputs: RunOutputs,       // produced artifact IDs + hashes
    #[serde(default)]
    pub ties: Vec<TieEvent>,       // ordered by unit/time as produced
}

/// Digest wrapper for the Normative Manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NmDigest {
    pub nm_sha256: String,
}

/// Inputs echoed in the run record (digests only; no raw content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputsEcho {
    /// Nested digest object (canonical shape).
    pub nm_digest: NmDigest,
    /// Optional additional inputs (registries, tallies, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<json::Map<String, Value>>,
}

/// Produced artifacts (+hashes) of the run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunOutputs {
    pub result_id: String,
    pub result_sha256: String,
    pub run_record_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontier_map_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontier_map_sha256: Option<String>,
}

/// Canonical record of a random tie (only when RNG actually used).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieEvent {
    pub unit_id: String,
    /// Optional extra detail (band, competing candidates, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Optional frontier map (presentation-supporting; separate canonical artifact).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierEntry {
    pub unit_id: String,
    pub band_met: String, // token per spec glossary; keep minimal here
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierMapDoc {
    pub id: String,        // "FR:<sha256>"
    pub created_at: String,
    pub entries: Vec<FrontierEntry>, // ordered
}

/// Internal “NoId” shapes used to compute canonical bytes and derive IDs.

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResultNoId {
    pub formula_id: String,
    pub engine_version: String,
    pub created_at: String,
    pub summary: ResultSummary,
    pub units: Vec<UnitResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunRecordNoId {
    pub timestamp_utc: String,
    pub formula_id: String,
    pub determinism: Determinism,
    pub inputs: InputsEcho,
    pub vars_effective: json::Map<String, Value>,
    pub outputs: RunOutputsNoRunHash, // run_hash not yet known
    #[serde(default)]
    pub ties: Vec<TieEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunOutputsNoRunHash {
    pub result_id: String,
    pub result_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontier_map_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontier_map_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrontierMapNoId {
    pub created_at: String,
    pub entries: Vec<FrontierEntry>,
}

/// Compute Formula ID (FID) from the **Normative Manifest** canonical bytes.
pub fn compute_formula_id(normative_manifest: &Value) -> Result<String, EngineError> {
    let nm_canon = to_canonical_bytes(normative_manifest)?;
    Ok(sha256_hex(&nm_canon))
}

/// Build a canonical Result artifact from a fully-specified `ResultNoId`.
fn finalize_result(noid: &ResultNoId) -> Result<(ResultDoc, String), EngineError> {
    let bytes = to_canonical_bytes(noid)?;
    let hash = sha256_hex(&bytes);
    let id = format!("RES:{hash}");
    let with_id = ResultDoc {
        id: id.clone(),
        formula_id: noid.formula_id.clone(),
        engine_version: noid.engine_version.clone(),
        created_at: noid.created_at.clone(),
        summary: noid.summary.clone(),
        units: noid.units.clone(),
        label: noid.label.clone(),
    };
    Ok((with_id, hash))
}

/// Build a canonical FrontierMap artifact (if present).
fn finalize_frontier(noid: &FrontierMapNoId) -> Result<(FrontierMapDoc, String), EngineError> {
    let bytes = to_canonical_bytes(noid)?;
    let hash = sha256_hex(&bytes);
    let id = format!("FR:{hash}");
    let with_id = FrontierMapDoc {
        id,
        created_at: noid.created_at.clone(),
        entries: noid.entries.clone(),
    };
    Ok((with_id, hash))
}

/// Build a canonical RunRecord artifact.
/// Note: `run_id` includes the timestamp and the hash of the canonical run record bytes.
fn finalize_run_record(
    timestamp_utc: &str,
    formula_id: &str,
    determinism: Determinism,
    inputs: InputsEcho,
    vars_effective: json::Map<String, Value>,
    result_id: &str,
    result_sha256: &str,
    ties: Vec<TieEvent>,
    frontier_map: Option<IdOnly>,
) -> Result<(RunRecordDoc, String), EngineError> {
    let ts = normalize_timestamp_utc(timestamp_utc)?;
    let outputs_no_run = RunOutputsNoRunHash {
        result_id: result_id.to_string(),
        result_sha256: result_sha256.to_string(),
        frontier_map_id: frontier_map.as_ref().map(|x| x.id.clone()),
        frontier_map_sha256: frontier_map.as_ref().map(|x| x.sha256.clone()),
    };

    let noid = RunRecordNoId {
        timestamp_utc: ts.clone(),
        formula_id: formula_id.to_string(),
        determinism,
        inputs,
        vars_effective,
        outputs: outputs_no_run,
        ties, // now carried through
    };

    // Hash the canonical RunRecord (without run_id) to derive run_id.
    let run_bytes = to_canonical_bytes(&noid)?;
    let run_hash = sha256_hex(&run_bytes);
    let run_id = format!("RUN:{ts}-{run_hash}");

    let outputs = RunOutputs {
        result_id: result_id.to_string(),
        result_sha256: result_sha256.to_string(),
        run_record_sha256: run_hash.clone(),
        frontier_map_id: frontier_map.as_ref().map(|x| x.id.clone()),
        frontier_map_sha256: frontier_map.as_ref().map(|x| x.sha256.clone()),
    };

    let with_id = RunRecordDoc {
        run_id,
        timestamp_utc: ts,
        formula_id: formula_id.to_string(),
        determinism: noid.determinism,
        inputs: noid.inputs,
        vars_effective: noid.vars_effective,
        outputs,
        ties: noid.ties,
    };

    Ok((with_id, run_hash))
}

/// Convenience helper to wrap an already-finalized artifact into IdOnly.
fn id_only(id: &str, sha256: &str) -> IdOnly {
    IdOnly { id: id.to_string(), sha256: sha256.to_string() }
}

/// ----- Public bundle for orchestration results (returned by part 2 functions) -----

#[derive(Debug, Clone)]
pub struct BuildOutputs {
    pub result: ResultDoc,
    pub result_sha256: String,
    pub run_record: RunRecordDoc,
    pub run_record_sha256: String,
    pub frontier_map: Option<FrontierMapDoc>,
    pub frontier_map_sha256: Option<String>,
}
// Part 2/2 — Orchestration, self-verify, and file I/O.
// Depends on types & helpers from Part 1.

use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use serde_json::{self as json, Value};

/// Inputs to build all canonical artifacts for a run.
#[derive(Debug, Clone)]
pub struct OrchestrationInputs {
    /// Full Normative Manifest (Included vars only), as JSON.
    pub normative_manifest: Value,
    /// Engine version token, e.g., "VM-ENGINE v0".
    pub engine_version: String,
    /// RFC3339 UTC. Also used as `Result.created_at`.
    pub timestamp_utc: String,
    /// Effective Included variables actually used by the run (echoed in RunRecord).
    pub vars_effective: json::Map<String, Value>,
    /// Already-computed per-unit results, in canonical order.
    pub units: Vec<UnitResult>,
    /// Presentation-only label (computed by the caller after allocations).
    pub label: Option<String>,
    /// Determinism echo (policy + optional rng seed; we’ll enforce rules below).
    pub determinism: Determinism,
    /// Random tie events (empty when none; required to echo rng_seed for random policy).
    pub ties: Vec<TieEvent>,
    /// Optional frontier entries (emitted only when Some and non-empty).
    pub frontier_entries: Option<Vec<FrontierEntry>>,
}

/// Build artifacts (`result.json`, `run_record.json`, optional `frontier_map.json`)
/// and return them with their sha256 digests. Performs spec self-verify checks.
pub fn build_artifacts(inp: OrchestrationInputs) -> Result<BuildOutputs, EngineError> {
    // --- Normalize and compute FID from the Normative Manifest ---
    let fid = compute_formula_id(&inp.normative_manifest)?;
    let nm_sha = {
        let nm_bytes = to_canonical_bytes(&inp.normative_manifest)?;
        sha256_hex(&nm_bytes)
    };

    // --- Determinism & rng_seed echo rules ---
    let tie_policy = inp.determinism.tie_policy;
    let rng_seed = match tie_policy {
        TiePolicy::DeterministicOrder => None, // never echo a seed in deterministic policy
        TiePolicy::Random => {
            if inp.ties.is_empty() { None } else { inp.determinism.rng_seed }
        }
    };
    let determinism = Determinism { tie_policy, rng_seed };

    // --- Build Result (NoId → WithId) ---
    let created_at = normalize_timestamp_utc(&inp.timestamp_utc)?; // also used in RUN prefix
    let summary = ResultSummary {
        unit_count: inp.units.len() as u64,
        allocation_count: inp.units.iter().filter(|u| u.assigned_to.is_some()).count() as u64,
        tie_count: inp.ties.len() as u64,
    };
    let result_noid = ResultNoId {
        formula_id: fid.clone(),
        engine_version: inp.engine_version.clone(),
        created_at: created_at.clone(),
        summary,
        units: inp.units.clone(),
        label: inp.label.clone(),
    };
    let (result_doc, result_sha256) = finalize_result(&result_noid)?;

    // --- Optional Frontier Map ---
    let (frontier_doc_opt, frontier_sha_opt, frontier_idonly_opt) = if let Some(entries) = inp.frontier_entries {
        if entries.is_empty() {
            (None, None, None)
        } else {
            let fm_noid = FrontierMapNoId {
                created_at: created_at.clone(),
                entries,
            };
            let (fm_doc, fm_sha) = finalize_frontier(&fm_noid)?;
            let idonly = id_only(&fm_doc.id, &fm_sha);
            (Some(fm_doc), Some(fm_sha), Some(idonly))
        }
    } else {
        (None, None, None)
    };

    // --- Inputs echo for RunRecord (nested digest shape) ---
    let inputs_echo = InputsEcho {
        nm_digest: NmDigest { nm_sha256: nm_sha },
        extra: None,
    };

    // --- Build RunRecord (NoId → WithId) ---
    let vars_effective = inp.vars_effective;
    let (run_record_doc, run_record_sha256) = finalize_run_record(
        &created_at,
        &fid,
        determinism,
        inputs_echo,
        vars_effective,
        &result_doc.id,
        &result_sha256,
        inp.ties.clone(),           // pass ties through
        frontier_idonly_opt,
    )?;

    // --- Self-verify (spec S6): cross-check IDs and FID consistency ---
    // 1) FID must match between Result and RunRecord.
    if result_doc.formula_id != fid || run_record_doc.formula_id != fid {
        return Err(EngineError::Spec("formula_id mismatch across artifacts".into()));
    }
    // 2) RES: id must be "RES:<sha256_of_result_noid>"
    if !result_doc.id.starts_with("RES:") || &result_doc.id[4..] != result_sha256 {
        return Err(EngineError::Internal("Result ID does not match canonical bytes".into()));
    }
    // 3) RUN: id must be "RUN:<timestamp>-<sha256_of_runrecord_noid>"
    if !run_record_doc.run_id.starts_with("RUN:") {
        return Err(EngineError::Internal("RunRecord ID prefix missing".into()));
    }
    let expect_prefix = format!("RUN:{}-", created_at);
    if !run_record_doc.run_id.starts_with(&expect_prefix) {
        return Err(EngineError::Internal("RunRecord timestamp prefix mismatch".into()));
    }
    if run_record_doc.outputs.run_record_sha256 != run_record_sha256 {
        return Err(EngineError::Internal("RunRecord SHA mismatch".into()));
    }
    // 4) RNG echo rule: if random policy and ties non-empty → rng_seed must be Some.
    if tie_policy == TiePolicy::Random {
        let has_rng = !inp.ties.is_empty();
        match run_record_doc.determinism.rng_seed {
            Some(_) if has_rng => {},                  // ok
            None if !has_rng => {},                    // ok
            Some(_) if !has_rng => {
                return Err(EngineError::Spec("rng_seed echoed but no random tie occurred".into()))
            }
            None if has_rng => {
                return Err(EngineError::Spec("random tie occurred but rng_seed was not echoed".into()))
            }
        }
    }
    // 5) Ties echo consistency
    if run_record_doc.ties.len() != inp.ties.len() {
        return Err(EngineError::Spec("RunRecord ties[] length mismatch".into()));
    }

    Ok(BuildOutputs {
        result: result_doc,
        result_sha256,
        run_record: run_record_doc,
        run_record_sha256,
        frontier_map: frontier_doc_opt,
        frontier_map_sha256: frontier_sha_opt,
    })
}

/// Write artifacts to `out_dir` with canonical JSON (LF) encoding.
/// Filenames: result.json, run_record.json, frontier_map.json (if present).
pub fn write_artifacts(out_dir: &Path, outs: &BuildOutputs) -> Result<(), EngineError> {
    create_dir_all(out_dir)?;

    // result.json
    {
        let bytes = to_canonical_bytes(&outs.result)?;
        write_file(out_dir.join("result.json"), &bytes)?;
    }
    // run_record.json
    {
        let bytes = to_canonical_bytes(&outs.run_record)?;
        write_file(out_dir.join("run_record.json"), &bytes)?;
    }
    // frontier_map.json (optional)
    if let Some(ref fm) = outs.frontier_map {
        let bytes = to_canonical_bytes(fm)?;
        write_file(out_dir.join("frontier_map.json"), &bytes)?;
    }
    Ok(())
}

fn write_file<P: AsRef<Path>>(path: P, bytes: &[u8]) -> Result<(), EngineError> {
    let mut f = File::create(path)?;
    f.write_all(bytes)?;
    Ok(())
}
