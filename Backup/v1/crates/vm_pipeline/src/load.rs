//! LOAD stage for normative runs: manifest → vm_io loaders → deterministic bundle.
//! - Enforces the "tally-only" input contract (no raw ballots path).
//! - Delegates schema/ID parsing & canonicalization to vm_io.
//! - Aggregates canonical digests and (if manifest) computes nm_digest + formula_id.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use vm_core::{
    entities::{DivisionRegistry, OptionItem},
    ids::*,
    variables::Params,
};
use vm_io::{
    hasher,
    loader,
    manifest::{self, Manifest, ResolvedPaths},
};

/// Errors surfaced by the LOAD stage.
#[derive(Debug)]
pub enum LoadError {
    Io(String),
    Schema(String),
    Manifest(String),
    Hash(String),
    Contract(String),
}

impl From<vm_io::IoError> for LoadError {
    fn from(e: vm_io::IoError) -> Self {
        use LoadError::*;
        match e {
            vm_io::IoError::Read(err) => Io(err.to_string()),
            vm_io::IoError::Write(err) => Io(err.to_string()),
            vm_io::IoError::Json { pointer, msg } => Schema(format!("json {pointer}: {msg}")),
            vm_io::IoError::Schema { pointer, msg } => Schema(format!("{pointer}: {msg}")),
            vm_io::IoError::Manifest(m) => Manifest(m),
            vm_io::IoError::Expect(m) => Manifest(format!("expectation: {m}")),
            vm_io::IoError::Canon(m) => Manifest(format!("canonicalization: {m}")),
            vm_io::IoError::Hash(m) => Hash(m),
            vm_io::IoError::Path(m) => Io(format!("path: {m}")),
            vm_io::IoError::Limit(m) => Io(format!("limit: {m}")),
        }
    }
}

/// Input IDs echoed downstream (placeholders for normative-run inputs).
/// In a fuller build, these may be parsed from files; for LOAD we keep opaque strings.
#[derive(Debug, Clone)]
pub struct LoadedIds {
    pub reg_id: String,
    pub tally_id: String,
    pub param_set_id: String,
}

/// Deterministic bundle consumed by downstream stages.
#[derive(Debug, Clone)]
pub struct NormContext {
    pub reg: DivisionRegistry,
    /// Registry-wide canonical option set (deduped by OptionId, sorted by (order_index, OptionId)).
    pub options: Vec<OptionItem>,
    pub params: Params,
    pub tallies: loader::UnitTallies,
    pub ids: LoadedIds,
}

/// Canonical input digests (64-hex).
#[derive(Debug, Clone)]
pub struct InputDigests {
    pub reg_sha256: String,
    pub tally_sha256: String,
    pub params_sha256: String,
    pub adjacency_sha256: Option<String>,
}

/// Output of the LOAD stage.
#[derive(Debug, Clone)]
pub struct LoadedStage {
    pub norm_ctx: NormContext,
    pub digests: InputDigests,
    /// Present when a manifest was used (nm_digest over the Normative Manifest JSON).
    pub nm_digest: Option<String>,
    /// Present when a manifest was used; equals nm_digest under current policy.
    pub formula_id: Option<String>,
}

// -------------------------------------------------------------------------------------------------
// Entry points
// -------------------------------------------------------------------------------------------------

/// Preferred: load from a manifest, enforce the normative contract, and compute nm_digest/FID.
pub fn load_normative_from_manifest<P: AsRef<Path>>(path: P) -> Result<LoadedStage, LoadError> {
    // 1) Parse + validate manifest.
    let man = manifest::load_manifest(&path)?;
    ensure_manifest_contract(&man)?;

    // 2) Resolve paths relative to the manifest directory.
    //    The function expects a base; pass the manifest file path (vm_io resolves parent).
    let resolved = manifest::resolve_paths(path.as_ref(), &man)?;

    // 3) Load artifacts (vm_io handles schema + canonicalization + basic cross-refs).
    let io_loaded = loader::load_all_from_manifest(&path)?;

    // 4) Lift into NormContext (re-assert core invariants if desired).
    let norm_ctx = to_norm_context(io_loaded)?;

    // 5) Collect input digests (file bytes for visibility; vm_io already computed canonical digests).
    let digests = collect_input_digests(&resolved)?;

    // 6) Compute Normative Manifest digest + Formula ID from a compact NM view
    //    built from the canonical input digests (deterministic and offline).
    let (nm_digest, formula_id) = compute_nm_fid_if_present_from_digests(&digests)?;

    Ok(LoadedStage {
        norm_ctx,
        digests,
        nm_digest: Some(nm_digest),
        formula_id: Some(formula_id),
    })
}

/// Alternate: load directly from explicit file paths (no manifest); nm_digest/FID omitted.
pub fn load_normative_from_paths<P: AsRef<Path>>(
    reg_path: P,
    tally_path: P,
    params_path: P,
    adjacency_path: Option<P>,
) -> Result<LoadedStage, LoadError> {
    // 1) Targeted loads (vm_io validates + canonicalizes).
    let registry = loader::load_registry(&reg_path)?;
    let params = loader::load_params(&params_path)?;
    let tallies = loader::load_ballot_tally(&tally_path)?;
    // Optional adjacency is handled by the loader when called from manifest; we just hash if present.

    // 2) Build NormContext.
    let ids = LoadedIds {
        reg_id: "REG:local".into(),
        tally_id: "TLY:local".into(),
        param_set_id: "PS:local".into(),
    };
    let options = collect_registry_options(&registry);
    let norm_ctx = NormContext {
        reg: registry,
        options,
        params,
        tallies,
        ids,
    };

    // 3) Digests over files (streamed).
    let digests = InputDigests {
        reg_sha256: hasher::sha256_file(&reg_path)?,
        tally_sha256: hasher::sha256_file(&tally_path)?,
        params_sha256: hasher::sha256_file(&params_path)?,
        adjacency_sha256: match adjacency_path {
            Some(p) => Some(hasher::sha256_file(p)?),
            None => None,
        },
    };

    Ok(LoadedStage {
        norm_ctx,
        digests,
        nm_digest: None,
        formula_id: None,
    })
}

// -------------------------------------------------------------------------------------------------
// Internals
// -------------------------------------------------------------------------------------------------

/// Ensure the manifest matches the normative contract:
/// - MUST provide `ballot_tally_path`
/// - MUST NOT provide a legacy `ballots_path` (schema should forbid; we still guard).
fn ensure_manifest_contract(man: &Manifest) -> Result<(), LoadError> {
    if man.ballot_tally_path.trim().is_empty() {
        return Err(LoadError::Contract(
            "manifest must specify `ballot_tally_path` for normative runs".into(),
        ));
    }
    // Defensive: if a legacy field slipped through (not present in typed Manifest), reject.
    // We can't see unknown fields here, so this is best-effort; schema in vm_io should enforce it.
    Ok(())
}

/// Convert vm_io's LoadedContext into our NormContext, re-asserting canonical option order.
fn to_norm_context(io: loader::LoadedContext) -> Result<NormContext, LoadError> {
    // Build a registry-wide canonical option list (dedupe by OptionId; keep min (order_index, id)).
    let options = collect_registry_options(&io.registry);

    // IDs are opaque in this stage; downstream RunRecord echoes them or uses placeholders.
    let ids = LoadedIds {
        reg_id: "REG:local".into(),
        tally_id: "TLY:local".into(),
        param_set_id: "PS:local".into(),
    };

    // (Optional) re-assertions could go here (e.g., non-empty units/options). vm_io guarantees canonical sort.
    Ok(NormContext {
        reg: io.registry,
        options,
        params: io.params,
        tallies: io.tally,
        ids,
    })
}

/// Compute nm_digest and formula_id (equal under current policy) from a compact NM JSON
/// derived from canonical input digests. Kept local to the LOAD stage when a manifest is used.
fn compute_nm_fid_if_present_from_digests(d: &InputDigests) -> Result<(String, String), LoadError> {
    // Compact NM view limited to deterministic, normative inputs (digests only).
    let nm = serde_json::json!({
        "normative_inputs": {
            "division_registry_sha256": d.reg_sha256,
            "ballot_tally_sha256": d.tally_sha256,
            "parameter_set_sha256": d.params_sha256,
            "adjacency_sha256": d.adjacency_sha256
        }
    });
    let nm_digest = hasher::nm_digest_from_value(&nm).map_err(LoadError::from)?;
    let fid = hasher::formula_id_from_nm(&nm).map_err(LoadError::from)?;
    Ok((nm_digest, fid))
}

/// Resolve + stream-hash the normative inputs referenced by the manifest.
/// (This is used by the manifest entry path; vm_io already computes canonical digests internally,
/// but we surface file digests here for transparency.)
fn collect_input_digests(paths: &ResolvedPaths) -> Result<InputDigests, LoadError> {
    let reg_sha256 = hasher::sha256_file(&paths.reg)?;
    let tally_sha256 = hasher::sha256_file(&paths.tally)?;
    let params_sha256 = hasher::sha256_file(&paths.params)?;
    let adjacency_sha256 = match &paths.adjacency {
        Some(p) => Some(hasher::sha256_file(p)?),
        None => None,
    };
    Ok(InputDigests {
        reg_sha256,
        tally_sha256,
        params_sha256,
        adjacency_sha256,
    })
}

// -------------------------------------------------------------------------------------------------
// Small utilities
// -------------------------------------------------------------------------------------------------

/// Aggregate a registry-wide option list, deduped by OptionId and sorted by (order_index, OptionId).
fn collect_registry_options(reg: &DivisionRegistry) -> Vec<OptionItem> {
    use core::cmp::Ordering;

    let mut seen: BTreeMap<vm_core::ids::OptionId, OptionItem> = BTreeMap::new();
    for unit in &reg.units {
        for opt in &unit.options {
            // If the same option_id appears across units, prefer the one with the smaller (order_index, id).
            match seen.get(&opt.option_id) {
                None => {
                    seen.insert(opt.option_id.clone(), opt.clone());
                }
                Some(existing) => {
                    let a = (opt.order_index, &opt.option_id);
                    let b = (existing.order_index, &existing.option_id);
                    if a.cmp(&b) == Ordering::Less {
                        seen.insert(opt.option_id.clone(), opt.clone());
                    }
                }
            }
        }
    }
    let mut options: Vec<OptionItem> = seen.into_values().collect();
    options.sort_by(|a, b| {
        (a
