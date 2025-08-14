//! LOAD stage for normative runs: manifest → vm_io loaders → deterministic bundle.
//! - Enforces the "tally-only" input contract (no raw ballots path).
//! - Delegates schema/ID parsing & canonicalization to vm_io.
//! - Aggregates canonical digests and (if manifest) computes nm_digest + formula_id.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use vm_core::{
    entities::{DivisionRegistry, OptionItem},
    variables::Params,
};
use vm_io::{
    hasher,
    loader,
    manifest::{self, Manifest},
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
    /// Present when a manifest was used (nm_digest over the Normative Manifest JSON built from Included variables).
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

    // 2) Load artifacts (vm_io handles schema + canonicalization + cross-refs & reordering).
    let io_loaded = loader::load_all_from_manifest(path.as_ref())?;

    // 3) Lift into NormContext.
    let options = collect_registry_options(&io_loaded.registry);
    let norm_ctx = NormContext {
        reg: io_loaded.registry,
        options,
        params: io_loaded.params,
        tallies: io_loaded.tally,
        ids: LoadedIds {
            reg_id: "REG:local".into(),
            tally_id: "TLY:local".into(),
            param_set_id: "PS:local".into(),
        },
    };

    // 4) Map canonical digests out of vm_io (these are digests of canonical bytes).
    let digests = InputDigests {
        reg_sha256: io_loaded.digests.division_registry_sha256,
        tally_sha256: io_loaded.digests.ballot_tally_sha256,
        params_sha256: io_loaded.digests.parameter_set_sha256,
        adjacency_sha256: io_loaded.digests.adjacency_sha256,
    };

    // 5) Compute Normative Manifest (NM) from **Included VM-VARs** and derive FID.
    let (nm_digest, fid) = compute_nm_and_fid_from_params(&norm_ctx.params)?;

    Ok(LoadedStage {
        norm_ctx,
        digests,
        nm_digest: Some(nm_digest),
        formula_id: Some(fid),
    })
}

/// Alternate entry: direct file paths (no manifest). Does not compute FID/NM.
pub fn load_normative_from_paths<P: AsRef<Path>>(
    registry_path: P,
    tally_path: P,
    params_path: P,
    adjacency_path: Option<P>,
) -> Result<LoadedStage, LoadError> {
    // Load individually via vm_io targeted loaders.
    let reg = loader::load_registry(registry_path.as_ref())?;
    let params = loader::load_params(params_path.as_ref())?;
    let mut tallies = loader::load_ballot_tally(tally_path.as_ref())?;

    // Reuse registry-deduced options; consumers sort per-registry anyway.
    // (vm_io::load_all_from_manifest performs per-unit reorder; path mode keeps tallies as loaded.)
    let options = collect_registry_options(&reg);

    let norm_ctx = NormContext {
        reg,
        options,
        params,
        tallies: {
            // Keep as loaded; callers relying on manifest-mode canonicalization should prefer it.
            // We still ensure stable unit ordering if present.
            tallies.units.sort_by(|a, b| a.unit_id.cmp(&b.unit_id));
            tallies
        },
        ids: LoadedIds {
            reg_id: "REG:local".into(),
            tally_id: "TLY:local".into(),
            param_set_id: "PS:local".into(),
        },
    };

    // Stream-hash file bytes for transparency in path mode.
    let digests = InputDigests {
        reg_sha256: hasher::sha256_file(registry_path.as_ref())?,
        tally_sha256: hasher::sha256_file(tally_path.as_ref())?,
        params_sha256: hasher::sha256_file(params_path.as_ref())?,
        adjacency_sha256: match adjacency_path {
            Some(p) => Some(hasher::sha256_file(p.as_ref())?),
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
    // Defensive note: if a legacy field “ballots_path” existed, schema should reject it.
    // We can only sanity-check the typed struct here.
    Ok(())
}

/// Compute NM and FID from the **Included VM-VARs** only (Annex A / Doc 2..5).
fn compute_nm_and_fid_from_params(params: &Params) -> Result<(String, String), LoadError> {
    use serde_json::{Map, Value};

    // Convert Params → Value, then filter keys by Included set.
    let full = serde_json::to_value(params)
        .map_err(|e| LoadError::Schema(format!("serialize Params: {e}")))?;

    let obj = full.as_object().ok_or_else(|| {
        LoadError::Schema("Params serialization did not yield an object".into())
    })?;

    let mut included: Map<String, Value> = Map::new();
    for (k, v) in obj {
        if is_included_var_key(k) {
            included.insert(k.clone(), v.clone());
        }
    }

    // Shape the NM with a stable top-level key to keep room for future fields.
    let nm = Value::Object({
        let mut m = Map::new();
        m.insert("vars_included".to_string(), Value::Object(included));
        m
    });

    let nm_digest = hasher::nm_digest_from_value(&nm).map_err(LoadError::from)?;
    let fid = hasher::formula_id_from_nm(&nm).map_err(LoadError::from)?;
    Ok((nm_digest, fid))
}

/// Inclusion predicate per Annex A:
/// IN: 001–007, 010–017, 020–031, 040–049, 050, 073
/// OUT: 032–035, 052, 060–062 (and any other non-listed).
fn is_included_var_key(key: &str) -> bool {
    // Expect keys like "v001_algorithm_family", "v014", "v050_tie_policy", etc.
    if !key.starts_with('v') { return false; }
    let digits = key.as_bytes().get(1..4);
    let Ok(n) = digits
        .and_then(|s| std::str::from_utf8(s).ok())
        .and_then(|s| s.parse::<u16>().ok())
    else { return false; };

    match n {
        1..=7 => true,        // 001..007
        10..=17 => true,      // 010..017
        20..=31 => true,      // 020..031
        40..=49 => true,      // 040..049
        50 => true,           // 050
        73 => true,           // 073
        // Explicit exclusions (for clarity; already excluded by ranges above except 052/060..062)
        32..=35 => false,
        52 => false,
        60..=62 => false,
        _ => false,
    }
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
        let ka = (a.order_index, &a.option_id);
        let kb = (b.order_index, &b.option_id);
        ka.cmp(&kb)
    });
    options
}
