//! crates/vm_io/src/loader.rs — Part 1/2 (fixed)
//! High-level, **offline** JSON loaders with schema checks, size limits, and raw-byte digests.
//!
//! Alignment with Docs 1–7 + Annex A/B/C:
//! - Resolve manifest-relative paths from the **manifest file’s directory**
//! - Reject any URL-like path (any `<scheme>://`, plus `file:`/`data:` forms) for strict offline posture
//! - Apply JSON Schema (Draft 2020-12) when the `schemaval` feature is enabled
//! - Enforce bounded reads on untrusted files
//! - Compute SHA-256 over **raw file bytes** for input digests (no JSON reformatting)
//!
//! Part 2 adds composition (`load_all`), expectation checks, and the bundle types.

#![forbid(unsafe_code)]

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::{IoError, IoResult};
use crate::manifest::{Manifest, ResolvedPaths};

use vm_core::{DivisionRegistry, Params};

#[cfg(feature = "serde")]
use serde_json::Value;

/* ---------- constants ---------- */

/// Maximum bytes allowed for any single JSON input (registry/params/tally).
const MAX_JSON_BYTES: u64 = 8 * 1024 * 1024; // 8 MiB

/* ---------- helpers: strict offline, size-bounded reads, schema ---------- */

/// Return true if `s` looks like a URL in a way we must reject for strict offline posture.
/// - Matches `<scheme>://...` where scheme = `[A-Za-z][A-Za-z0-9+.-]*`
/// - Also rejects `file:` and `data:` even without `//`
/// - Intentionally does **not** match Windows drive letters like `C:\...`
#[inline]
fn looks_like_url_strict(s: &str) -> bool {
    let t = s.trim();
    if t.starts_with("file:") || t.starts_with("data:") {
        return true;
    }
    if let Some(pos) = t.find("://") {
        let scheme = &t[..pos];
        if !scheme.is_empty()
            && scheme.chars().next().unwrap().is_ascii_alphabetic()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'))
        {
            return true;
        }
    }
    false
}

#[inline]
fn ensure_offline_path(path: &Path) -> IoResult<()> {
    let s = path.to_string_lossy();
    if looks_like_url_strict(&s) {
        return Err(IoError::Invalid(format!("URL-like path is not allowed: {s}")));
    }
    Ok(())
}

/// Read a JSON file to `serde_json::Value` with a size cap and basic UTF-8 enforcement.
#[cfg(feature = "serde")]
fn read_json_value_bounded(path: &Path) -> IoResult<Value> {
    let meta = fs::metadata(path)?;
    if meta.len() > MAX_JSON_BYTES {
        return Err(IoError::Invalid("file too large".into()));
    }
    let mut f = fs::File::open(path)?;
    let mut s = String::with_capacity(meta.len().min(1_000_000) as usize);
    f.read_to_string(&mut s)?;
    let v: Value = serde_json::from_str(&s)?;
    Ok(v)
}

#[cfg(not(feature = "serde"))]
#[allow(unused_variables)]
fn read_json_value_bounded(_path: &Path) -> IoResult<()> {
    Err(IoError::Canon("serde feature disabled".into()))
}

#[cfg(feature = "schemaval")]
#[inline]
fn validate_schema(kind: crate::schema::SchemaKind, v: &serde_json::Value) -> IoResult<()> {
    crate::schema::validate_value(kind, v)
}

#[cfg(not(feature = "schemaval"))]
#[inline]
fn validate_schema(_: crate::schema::SchemaKind, _: &serde_json::Value) -> IoResult<()> {
    Ok(())
}

/* ---------- digests (raw bytes) ---------- */

/// Compute SHA-256 hex of a file’s **raw bytes** (preferred for input digests).
fn sha256_of_file(path: &Path) -> IoResult<String> {
    #[cfg(feature = "hash")]
    {
        crate::hasher::sha256_file(path)
    }
    #[cfg(not(feature = "hash"))]
    {
        Err(IoError::Hash("hash feature disabled".into()))
    }
}

/* ---------- manifest: load + resolve (from manifest file path) ---------- */

/// Load a manifest JSON and resolve all relative paths from the **manifest file’s directory**.
pub fn load_manifest_and_resolve(manifest_path: &Path) -> IoResult<(Manifest, ResolvedPaths)> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    let base = manifest_path
        .parent()
        .ok_or_else(|| IoError::Path(format!("manifest has no parent directory: {}", manifest_path.display())))?;

    let resolved = crate::manifest::resolve_paths(base, &manifest)?;
    // Strict offline posture (defense-in-depth; manifest loader already rejects common schemes)
    ensure_offline_path(&resolved.reg)?;
    ensure_offline_path(&resolved.params)?;
    ensure_offline_path(&resolved.tally)?;
    if let Some(adj) = &resolved.adjacency {
        ensure_offline_path(adj)?;
    }
    Ok((manifest, resolved))
}

/* ---------- typed loaders (individual) ---------- */

/// Load a DivisionRegistry from disk (schema-checked when enabled).
#[cfg(feature = "serde")]
pub fn load_registry(path: &Path) -> IoResult<DivisionRegistry> {
    let v = read_json_value_bounded(path)?;
    validate_schema(crate::schema::SchemaKind::DivisionRegistry, &v)?;
    let reg: DivisionRegistry = serde_json::from_value(v)?;
    Ok(reg)
}

#[cfg(not(feature = "serde"))]
#[allow(unused_variables)]
pub fn load_registry(_path: &Path) -> IoResult<DivisionRegistry> {
    Err(IoError::Canon("serde feature disabled".into()))
}

/// Load a Params set from disk (schema-checked when enabled).
#[cfg(feature = "serde")]
pub fn load_params(path: &Path) -> IoResult<Params> {
    let v = read_json_value_bounded(path)?;
    validate_schema(crate::schema::SchemaKind::ParameterSet, &v)?;
    let params: Params = serde_json::from_value(v)?;
    Ok(params)
}

#[cfg(not(feature = "serde"))]
#[allow(unused_variables)]
pub fn load_params(_path: &Path) -> IoResult<Params> {
    Err(IoError::Canon("serde feature disabled".into()))
}

/// Load a ballot tally from disk as raw JSON (schema-checked when enabled).
/// NOTE: `vm_core` does not currently expose a `BallotTally` struct; callers may
///       adapt the value or map it into their domain type later in the pipeline.
#[cfg(feature = "serde")]
pub fn load_tally_raw(path: &Path) -> IoResult<Value> {
    let v = read_json_value_bounded(path)?;
    validate_schema(crate::schema::SchemaKind::BallotTally, &v)?;
    Ok(v)
}

#[cfg(not(feature = "serde"))]
#[allow(unused_variables)]
pub fn load_tally_raw(_path: &Path) -> IoResult<()> {
    Err(IoError::Canon("serde feature disabled".into()))
}

/* ---------- digests for inputs (registry/params/tally[/adjacency]) ---------- */

#[derive(Debug, Clone)]
pub struct InputDigests {
    pub reg_sha256: String,
    pub params_sha256: String,
    pub tally_sha256: String,
    pub adjacency_sha256: Option<String>,
}

pub fn compute_input_digests(paths: &ResolvedPaths) -> IoResult<InputDigests> {
    Ok(InputDigests {
        reg_sha256: sha256_of_file(&paths.reg)?,
        params_sha256: sha256_of_file(&paths.params)?,
        tally_sha256: sha256_of_file(&paths.tally)?,
        adjacency_sha256: match &paths.adjacency {
            Some(p) => Some(sha256_of_file(p)?),
            None => None,
        },
    })
}

//! crates/vm_io/src/loader.rs — Part 2/2 (fixed)
//! Composition helpers (load_all), expectation checks, and bundle types.
//!
//! NOTE: Functions and types that carry raw JSON values are gated on `serde`.

#![forbid(unsafe_code)]

use std::path::Path;

use crate::{IoError, IoResult};
use crate::manifest::{Manifest, ResolvedPaths, verify_expectations};
use super::{load_manifest_and_resolve, load_registry, load_params, load_tally_raw, compute_input_digests, InputDigests};

use vm_core::{DivisionRegistry, Params};

#[cfg(feature = "serde")]
use serde_json::Value;

/* ---------- loaded bundle types ---------- */

#[cfg(feature = "serde")]
#[derive(Debug)]
pub struct LoadedInputs {
    pub registry: DivisionRegistry,
    pub params: Params,
    /// Raw ballot tally JSON (typed mapping is pipeline-dependent)
    pub tally: Value,
}

#[cfg(feature = "serde")]
#[derive(Debug)]
pub struct LoadedBundle {
    pub manifest: Manifest,
    pub paths: ResolvedPaths,
    pub inputs: LoadedInputs,
    pub digests: InputDigests,
}

/* ---------- composition ---------- */

/// Load everything referenced by a manifest file:
/// - parse + basic validation of the manifest
/// - resolve relative paths from the manifest file's directory
/// - strict offline checks on all resolved paths
/// - schema-checked loads of registry/params/tally (when `schemaval` is on)
/// - raw-byte SHA-256 digests for each input (and adjacency when present)
/// - normalization of `Params` to stabilize FID (dedup/sort of order-sensitive lists)
#[cfg(feature = "serde")]
pub fn load_all(manifest_path: &Path) -> IoResult<LoadedBundle> {
    let (manifest, paths) = load_manifest_and_resolve(manifest_path)?;

    // Load each artifact with schema checks (feature-gated inside)
    let registry = load_registry(&paths.reg)?;
    let mut params = load_params(&paths.params)?;
    let tally = load_tally_raw(&paths.tally)?;

    // Normalize Params for FID stability (no-ops if already canonical)
    params.normalize_for_fid();

    // Raw-byte digests (match test-pack & Annex B)
    let digests: InputDigests = compute_input_digests(&paths)?;

    Ok(LoadedBundle {
        manifest,
        paths,
        inputs: LoadedInputs { registry, params, tally },
        digests,
    })
}

#[cfg(not(feature = "serde"))]
#[allow(unused_variables)]
pub fn load_all(_manifest_path: &Path) -> IoResult<()> {
    Err(IoError::Canon("serde feature disabled".into()))
}

/* ---------- expectations ---------- */

/// Verify the optional `expect { formula_id, engine_version }` block from the manifest
/// against the **actual** values computed/known by the caller (e.g., after FID calc).
///
/// Callers that don't have one or both values yet may pass `None` for that field.
/// This function returns `Ok(()))` when there is no `expect` block.
pub fn check_manifest_expectations(
    manifest: &Manifest,
    actual_formula_id: Option<&str>,
    actual_engine_version: Option<&str>,
) -> IoResult<()> {
    verify_expectations(manifest, actual_formula_id, actual_engine_version)
}

/* ---------- convenience on the bundle ---------- */

#[cfg(feature = "serde")]
impl LoadedBundle {
    /// Convenience to re-run expectation checks using the bundle's manifest.
    pub fn assert_expectations(
        &self,
        actual_formula_id: Option<&str>,
        actual_engine_version: Option<&str>,
    ) -> IoResult<()> {
        check_manifest_expectations(&self.manifest, actual_formula_id, actual_engine_version)
    }
}
