//! vm_pipeline — deterministic pipeline surface (load→validate→tabulate→allocate→aggregate→gates→frontier→label→build)
//! This crate stays I/O-free and delegates JSON/Schema/Hashing to `vm_io` and math to `vm_algo`.
//! The code below is a scaffolded pipeline skeleton with stable types and IDs; fill
//! in the stage internals incrementally without changing these public signatures.

use std::path::Path;

use vm_core::{
    ids::*,
    entities::*,
    variables::Params,
};
use vm_io::{
    manifest,
    loader::{self, LoadedContext},
    canonical_json,
    hasher,
};

/// Engine identifiers (baked by the build system in real deployments).
#[derive(Debug, Clone)]
pub struct EngineMeta {
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub build: String,
}

/// Pipeline context: inputs are already loaded and validated by vm_io;
/// `nm_canonical` is the Normative Manifest JSON used to compute the Formula ID.
#[derive(Debug)]
pub struct PipelineCtx {
    pub loaded: LoadedContext,
    pub engine_meta: EngineMeta,
    pub nm_canonical: serde_json::Value,
}

/// Top-level pipeline outputs: Result, RunRecord, optional FrontierMap.
#[derive(Debug)]
pub struct PipelineOutputs {
    pub result: ResultDoc,
    pub run_record: RunRecordDoc,
    pub frontier_map: Option<FrontierMapDoc>,
}

/// Single error surface for the pipeline orchestration.
#[derive(Debug)]
pub enum PipelineError {
    Io(String),
    Schema(String),
    Validate(String),
    Tabulate(String),
    Allocate(String),
    Gates(String),
    Frontier(String),
    Tie(String),
    Build(String),
}

impl From<vm_io::IoError> for PipelineError {
    fn from(e: vm_io::IoError) -> Self {
        use PipelineError::*;
        // Map all vm_io errors to Io/Schema/Hash/Path-ish buckets with stable text.
        match e {
            vm_io::IoError::Schema { pointer, msg } => Schema(format!("{pointer}: {msg}")),
            vm_io::IoError::Json { pointer, msg } => Schema(format!("json {pointer}: {msg}")),
            vm_io::IoError::Read(e) => Io(format!("read: {e}")),
            vm_io::IoError::Write(e) => Io(format!("write: {e}")),
            vm_io::IoError::Manifest(m) => Validate(format!("manifest: {m}")),
            vm_io::IoError::Expect(m) => Validate(format!("expect: {m}")),
            vm_io::IoError::Canon(m) => Build(format!("canon: {m}")),
            vm_io::IoError::Hash(m) => Build(format!("hash: {m}")),
            vm_io::IoError::Path(m) => Io(format!("path: {m}")),
            vm_io::IoError::Limit(m) => Io(format!("limit: {m}")),
        }
    }
}

// ---------------------------- Result / RunRecord / Frontier docs ----------------------------
// These are minimal, typed mirrors of the external schemas with only the stable fields needed
// for end-to-end ID computation. Extend in-place without changing field names or types.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResultDoc {
    pub id: String,             // "RES:<hex64>"
    pub formula_id: String,     // 64-hex
    pub label: LabelBlock,      // {value, reason?}
    // In a fuller implementation you’d add: gates panel, per-unit blocks, aggregates, shares, etc.
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LabelBlock {
    pub value: String,             // "Decisive" | "Marginal" | "Invalid"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunRecordDoc {
    pub id: String,                                // "RUN:<ts>-<hex64>"
    pub timestamp_utc: String,                     // RFC3339 Z
    pub engine: EngineMeta,                        // vendor/name/version/build
    pub formula_id: String,                        // 64-hex
    pub normative_manifest_sha256: String,         // nm_digest
    pub inputs: RunInputs,                         // input IDs + digests
    pub policy: TiePolicyEcho,                     // tie policy (+seed if random)
    pub outputs: RunOutputs,                       // produced artifacts (+hashes)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunInputs {
    pub division_registry_id: String,
    pub division_registry_sha256: String,
    pub ballot_tally_id: String,
    pub ballot_tally_sha256: String,
    pub parameter_set_id: String,
    pub parameter_set_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjacency_sha256: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TiePolicyEcho {
    pub tie_policy: String,                 // "status_quo" | "deterministic" | "random"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng_seed: Option<u64>,              // present iff policy == random
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunOutputs {
    pub result_id: String,                  // "RES:<hex64>"
    pub result_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontier_map_id: Option<String>,    // "FR:<hex64>"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontier_map_sha256: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrontierMapDoc {
    pub id: String,                         // "FR:<hex64>"
    pub summary: FrontierMapSummary,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct FrontierMapSummary {
    pub band_counts: std::collections::BTreeMap<String, u32>,
    pub mediation_units: u32,
    pub enclave_units: u32,
    pub any_protected_override: bool,
}

// -------------------------------------- Public API --------------------------------------

/// Orchestrate the pipeline with a preloaded context.
/// NOTE: This scaffold emits a minimal but canonical Result/RunRecord (and optional FrontierMap),
/// computing IDs via vm_io::hasher. Fill in each stage (TABULATE/ALLOCATE/…) behind these fences.
pub fn run_with_ctx(ctx: PipelineCtx) -> Result<PipelineOutputs, PipelineError> {
    // --- Compute Formula ID from Normative Manifest (NM) ---
    let fid = compute_formula_id(&ctx.nm_canonical);

    // --- LABEL (placeholder) ---
    // For the scaffold, consider any non-empty FID as "Decisive".
    let label = LabelBlock { value: "Decisive".to_string(), reason: None };

    // --- BUILD_RESULT (without id) ---
    #[derive(serde::Serialize)]
    struct ResultNoId<'a> {
        formula_id: &'a str,
        label: &'a LabelBlock,
    }
    let res_no_id = ResultNoId { formula_id: &fid, label: &label };
    let res_id = hasher::res_id_from_canonical(&res_no_id)
        .map_err(PipelineError::from)?;
    let res_bytes = canonical_json::to_canonical_bytes(&res_no_id).map_err(PipelineError::from)?;
    let res_sha = hasher::sha256_hex(&res_bytes);

    let result = ResultDoc {
        id: res_id.clone(),
        formula_id: fid.clone(),
        label,
    };

    // --- (Optional) FRONTIER MAP (skipped unless enabled and gates pass) ---
    // This scaffold emits None; fill in by calling vm_algo::gates_frontier (apply_decision_gates → map_frontier)
    // and then hashing a stable, typed map.
    let frontier_map: Option<FrontierMapDoc> = None;

    // --- BUILD_RUN_RECORD (without id) ---
    let nm_digest = vm_io::hasher::nm_digest_from_value(&ctx.nm_canonical)
        .map_err(PipelineError::from)?;

    // Tie policy echo (policy string must align with wire naming in vm_io).
    let policy_str = match ctx.loaded.params.v050_tie_policy {
        vm_core::variables::TiePolicy::StatusQuo => "status_quo",
        vm_core::variables::TiePolicy::DeterministicOrder => "deterministic",
        vm_core::variables::TiePolicy::Random => "random",
    }.to_string();

    let policy = TiePolicyEcho {
        tie_policy: policy_str,
        rng_seed: ctx.loaded.params.v052_tie_seed,
    };

    // Inputs: NOTE these IDs are placeholders; in a full engine you would parse or assign canonical
    // input IDs in vm_io, then echo them here. For the scaffold we place the sha256 hex only.
    let inputs = RunInputs {
        division_registry_id: "REG:local".to_string(),
        division_registry_sha256: ctx.loaded.digests.division_registry_sha256.clone(),
        ballot_tally_id: "TLY:local".to_string(),
        ballot_tally_sha256: ctx.loaded.digests.ballot_tally_sha256.clone(),
        parameter_set_id: "PS:local".to_string(),
        parameter_set_sha256: ctx.loaded.digests.parameter_set_sha256.clone(),
        adjacency_sha256: ctx.loaded.digests.adjacency_sha256.clone(),
    };

    let outputs = RunOutputs {
        result_id: res_id.clone(),
        result_sha256: res_sha.clone(),
        frontier_map_id: frontier_map.as_ref().map(|fm| fm.id.clone()),
        frontier_map_sha256: None, // filled if/when frontier_map is produced
    };

    // RFC3339 UTC timestamp; deterministic placeholder here (fill with real clock in CLI/app).
    let timestamp = "1970-01-01T00:00:00Z".to_string();

    #[derive(serde::Serialize)]
    struct RunNoId<'a> {
        timestamp_utc: &'a str,
        engine: &'a EngineMeta,
        formula_id: &'a str,
        normative_manifest_sha256: &'a str,
        inputs: &'a RunInputs,
        policy: &'a TiePolicyEcho,
        outputs: &'a RunOutputs,
    }
    let run_no_id = RunNoId {
        timestamp_utc: &timestamp,
        engine: &ctx.engine_meta,
        formula_id: &fid,
        normative_manifest_sha256: &nm_digest,
        inputs: &inputs,
        policy: &policy,
        outputs: &outputs,
    };
    let run_bytes = canonical_json::to_canonical_bytes(&run_no_id).map_err(PipelineError::from)?;
    let run_id = hasher::run_id_from_bytes(&timestamp, &run_bytes).map_err(PipelineError::from)?;

    let run_record = RunRecordDoc {
        id: run_id,
        timestamp_utc: timestamp,
        engine: ctx.engine_meta,
        formula_id: fid,
        normative_manifest_sha256: nm_digest,
        inputs,
        policy,
        outputs,
    };

    Ok(PipelineOutputs { result, run_record, frontier_map })
}

/// Convenience entry: parse/validate a manifest, load artifacts via vm_io, then run the pipeline.
/// This helper constructs a trivial NM (empty object) for the Formula ID placeholder; callers
/// integrating the full Annex-A “Normative Manifest” should pass a richer `nm_canonical` via `run_with_ctx`.
pub fn run_from_manifest_path<P: AsRef<Path>>(path: P) -> Result<PipelineOutputs, PipelineError> {
    let loaded = loader::load_all_from_manifest(path).map_err(PipelineError::from)?;

    let ctx = PipelineCtx {
        loaded,
        engine_meta: engine_identifiers(),
        nm_canonical: serde_json::json!({}), // placeholder NM (include only normative fields in a full build)
    };

    run_with_ctx(ctx)
}

/// Engine identifiers for use in RunRecord and manifest “expect” checks.
pub fn engine_identifiers() -> EngineMeta {
    EngineMeta {
        vendor: "vm".to_string(),
        name: "vm_engine".to_string(),
        version: "0.1.0".to_string(),
        build: "dev".to_string(),
    }
}

/// Compute Formula ID (FID) from the Normative Manifest (NM) JSON.
/// Current policy: FID == SHA-256 hex of NM’s canonical bytes.
pub fn compute_formula_id(nm: &serde_json::Value) -> String {
    hasher::formula_id_from_nm(nm).unwrap_or_else(|_| "0".repeat(64))
}

// -------------------------------------- (internal helpers) --------------------------------------
// As you flesh out the pipeline, add private stage functions here that take typed inputs and
// return typed outputs. Keep all I/O, JSON shape, and hashing calls routed through vm_io.
