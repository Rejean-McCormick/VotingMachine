// crates/vm_io/src/manifest.rs — Part 1/2 (final patched)
//
// Scope of this half:
// - Types (external Manifest, ResolvedManifest, optional digests/expectations)
// - Error enum
// - Helpers (scheme/hex/path utils)
// - Validation of external manifest (shape & offline policy)
// - Path resolution + existence/type checks (S0 preconditions)
//
// Alignment with the 10 refs (Docs 1–7 + Annexes A–C):
// • S0 inputs are paths only: registry, tally, params (Doc 5B).
// • Optional inputs: adjacency, autonomy (if supported).
// • Offline-only: reject any path with a scheme ("://", "http:", "https:") (Doc 3).
// • No legacy `ballots_path`: the canonical input is *tally*. We do NOT accept aliases.
// • `id` is NOT required by spec; optional and ignored by canonical artifacts.
// • Digests (if provided) must be 64-lower-hex and only for present inputs.
// • Required inputs must exist and be files (not dirs).

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// External manifest accepted by the loader.
///
/// `id` is optional and non-normative (not used in any canonical artifact).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct Manifest {
    /// Optional user-provided manifest identifier (non-normative).
    pub id: Option<String>,

    /// Required input paths (commonly relative to the manifest’s directory).
    pub reg_path: String,
    pub params_path: String,
    pub ballot_tally_path: String,

    /// Optional inputs.
    #[cfg_attr(feature = "serde", serde(default))]
    pub adjacency_path: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub autonomy_path: Option<String>,

    /// Optional sha256 digests for inputs (lowercase 64-hex). If present,
    /// verification happens over canonical JSON bytes (Part 2).
    #[cfg_attr(feature = "serde", serde(default))]
    pub inputs_sha256: Option<InputDigests>,

    /// Optional expectations (engine/formula) — enforced in Part 2.
    #[cfg_attr(feature = "serde", serde(default))]
    pub expect: Option<Expectations>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct InputDigests {
    #[cfg_attr(feature = "serde", serde(default))]
    pub reg_path: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub params_path: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub ballot_tally_path: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub adjacency_path: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub autonomy_path: Option<String>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Expectations {
    /// Expected engine version string (exact match).
    #[cfg_attr(feature = "serde", serde(default))]
    pub engine_version: Option<String>,
    /// Expected formula ID (lowercase 64-hex).
    #[cfg_attr(feature = "serde", serde(default))]
    pub formula_id_hex: Option<String>,
}

/// Paths resolved against a base directory (usually the manifest’s dir).
#[derive(Debug, Clone)]
pub struct ResolvedManifest {
    pub reg_path: PathBuf,
    pub params_path: PathBuf,
    pub ballot_tally_path: PathBuf,
    pub adjacency_path: Option<PathBuf>,
    pub autonomy_path: Option<PathBuf>,
    pub digests: Option<InputDigests>,
    pub expect: Option<Expectations>,
}

/// Loader/validation errors.
#[derive(Debug)]
pub enum ManifestError {
    Missing(&'static str),
    Empty(&'static str),
    UrlPath(&'static str, String),
    Io(&'static str, String),
    NotAFile(&'static str, String),
    /// Bad hex format / shape (used for 64-hex checks, not mismatches).
    DigestShape(&'static str, String),
    /// Provided digest doesn’t match computed canonical sha256.
    DigestMismatch(&'static str, String),
    /// Expectation (engine/formula) mismatch (used in Part 2).
    ExpectationMismatch(&'static str, String),
    /// Digest provided for an input that is not present in the manifest.
    DigestForMissing(&'static str),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ManifestError::*;
        match self {
            Missing(k) => write!(f, "missing required field: {}", k),
            Empty(k) => write!(f, "field must not be empty: {}", k),
            UrlPath(k, v) => write!(f, "path must be offline (no scheme) for {}: {}", k, v),
            Io(k, v) => write!(f, "cannot access {}: {}", k, v),
            NotAFile(k, v) => write!(f, "path is not a file for {}: {}", k, v),
            DigestShape(k, v) => write!(f, "invalid sha256 format for {}: {}", k, v),
            DigestMismatch(k, v) => write!(f, "sha256 mismatch for {}: {}", k, v),
            ExpectationMismatch(k, v) => write!(f, "expectation mismatch for {}: {}", k, v),
            DigestForMissing(k) => write!(f, "digest supplied for missing input: {}", k),
        }
    }
}
impl std::error::Error for ManifestError {}

// ---------- helpers (pure) ----------

#[inline]
fn has_any_scheme(s: &str) -> bool {
    // Generic scheme detection plus explicit http(s).
    s.contains("://") || s.starts_with("http:") || s.starts_with("https:")
}

#[inline]
fn is_lower_hex_64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b))
}

#[inline]
fn join_under(base: &Path, rel: &str) -> PathBuf {
    let p = Path::new(rel);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base.join(p)
    }
}

// ---------- validation (shape, offline, digests present only for present paths) ----------

/// Validate manifest *shape* and offline path policy. Does not perform I/O.
/// Call `resolve_paths` afterwards to resolve and existence-check.
pub fn validate_manifest(man: &Manifest) -> Result<(), ManifestError> {
    // Required strings present and non-empty
    if man.reg_path.is_empty() {
        return Err(ManifestError::Empty("reg_path"));
    }
    if man.params_path.is_empty() {
        return Err(ManifestError::Empty("params_path"));
    }
    if man.ballot_tally_path.is_empty() {
        return Err(ManifestError::Empty("ballot_tally_path"));
    }

    // Offline-only for all present path fields
    offline_check("reg_path", &man.reg_path)?;
    offline_check("params_path", &man.params_path)?;
    offline_check("ballot_tally_path", &man.ballot_tally_path)?;
    if let Some(s) = &man.adjacency_path {
        if s.is_empty() {
            return Err(ManifestError::Empty("adjacency_path"));
        }
        offline_check("adjacency_path", s)?;
    }
    if let Some(s) = &man.autonomy_path {
        if s.is_empty() {
            return Err(ManifestError::Empty("autonomy_path"));
        }
        offline_check("autonomy_path", s)?;
    }

    // Digests (if any): lowercase 64-hex and only for present inputs
    if let Some(d) = &man.inputs_sha256 {
        if let Some(h) = &d.reg_path {
            if !is_lower_hex_64(h) {
                return Err(ManifestError::DigestShape("reg_path", h.clone()));
            }
        }
        if let Some(h) = &d.params_path {
            if !is_lower_hex_64(h) {
                return Err(ManifestError::DigestShape("params_path", h.clone()));
            }
        }
        if let Some(h) = &d.ballot_tally_path {
            if !is_lower_hex_64(h) {
                return Err(ManifestError::DigestShape("ballot_tally_path", h.clone()));
            }
        }
        if let Some(h) = &d.adjacency_path {
            if man.adjacency_path.is_none() {
                return Err(ManifestError::DigestForMissing("adjacency_path"));
            }
            if !is_lower_hex_64(h) {
                return Err(ManifestError::DigestShape("adjacency_path", h.clone()));
            }
        }
        if let Some(h) = &d.autonomy_path {
            if man.autonomy_path.is_none() {
                return Err(ManifestError::DigestForMissing("autonomy_path"));
            }
            if !is_lower_hex_64(h) {
                return Err(ManifestError::DigestShape("autonomy_path", h.clone()));
            }
        }
    }

    // Expectations (if present): basic format checks here; strict checks in Part 2
    if let Some(exp) = &man.expect {
        if let Some(fid) = &exp.formula_id_hex {
            if !is_lower_hex_64(fid) {
                return Err(ManifestError::DigestShape("formula_id_hex", fid.clone()));
            }
        }
        // engine_version is free-form (exact match enforced in Part 2 if provided)
        let _ = &exp.engine_version;
    }

    Ok(())
}

fn offline_check(label: &'static str, path: &str) -> Result<(), ManifestError> {
    if has_any_scheme(path) {
        return Err(ManifestError::UrlPath(label, path.to_string()));
    }
    Ok(())
}

// ---------- resolution (join base + existence/type checks) ----------

/// Resolve manifest paths under `base_dir` (usually the directory of the manifest file),
/// ensure required inputs exist and are files, and return a `ResolvedManifest`.
///
/// This performs *no* hashing or schema validation; Part 2 handles those steps.
pub fn resolve_paths(base_dir: &Path, man: &Manifest) -> Result<ResolvedManifest, ManifestError> {
    let reg = join_under(base_dir, &man.reg_path);
    let params = join_under(base_dir, &man.params_path);
    let tally = join_under(base_dir, &man.ballot_tally_path);
    let adj = man.adjacency_path.as_ref().map(|s| join_under(base_dir, s));
    let aut = man.autonomy_path.as_ref().map(|s| join_under(base_dir, s));

    // Existence/type checks for required files
    must_exist_file("reg_path", &reg)?;
    must_exist_file("params_path", &params)?;
    must_exist_file("ballot_tally_path", &tally)?;

    // Optional files (if provided)
    if let Some(p) = &adj {
        must_exist_file("adjacency_path", p)?;
    }
    if let Some(p) = &aut {
        must_exist_file("autonomy_path", p)?;
    }

    Ok(ResolvedManifest {
        reg_path: reg,
        params_path: params,
        ballot_tally_path: tally,
        adjacency_path: adj,
        autonomy_path: aut,
        digests: man.inputs_sha256.clone(),
        expect: man.expect.clone(),
    })
}

fn must_exist_file(label: &'static str, p: &Path) -> Result<(), ManifestError> {
    let md = fs::metadata(p).map_err(|e| ManifestError::Io(label, format!("{} ({e})", p.display())))?;
    if !md.is_file() {
        return Err(ManifestError::NotAFile(label, p.display().to_string()));
    }
    Ok(())
}

// crates/vm_io/src/manifest.rs — Part 2/2 (final patched)
//
// Scope of this half:
// - Canonical-JSON hashing (feature-gated: `hash`)
// - Digest verification over *canonical* bytes
// - Expectations enforcement (engine/formula)
// - Public entrypoints to load → validate → resolve → verify
//
// Notes:
// • Hashes are computed over canonical JSON (sorted keys, UTF-8) per Docs 1/Annex B.
// • If `hash` feature is disabled, digest verification returns an error.
// • Expectation mismatches return `ExpectationMismatch`.

#[cfg(feature = "serde")]
use serde_json::{self as json, Value as Json};

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

// ---------------------------- canonical JSON ----------------------------

#[cfg(feature = "serde")]
fn canonicalize_json(v: &Json) -> Json {
    use json::map::Map;
    match v {
        Json::Null | Json::Bool(_) | Json::Number(_) | Json::String(_) => v.clone(),
        Json::Array(a) => Json::Array(a.iter().map(canonicalize_json).collect()),
        Json::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                out.insert(k.clone(), canonicalize_json(&m[k]));
            }
            Json::Object(out)
        }
    }
}

#[cfg(feature = "serde")]
fn canonical_json_bytes_from_file(p: &Path) -> Result<Vec<u8>, super::ManifestError> {
    let mut buf = Vec::new();
    let mut f = fs::File::open(p)
        .map_err(|e| super::ManifestError::Io("read", format!("{} ({e})", p.display())))?;
    f.read_to_end(&mut buf)
        .map_err(|e| super::ManifestError::Io("read", format!("{} ({e})", p.display())))?;
    let v: Json = json::from_slice(&buf)
        .map_err(|e| super::ManifestError::Io("parse", format!("{} ({e})", p.display())))?;
    let canon = canonicalize_json(&v);
    let s =
        json::to_string(&canon).map_err(|e| super::ManifestError::Io("canonicalize", format!("{e}")))?;
    Ok(s.into_bytes())
}

#[cfg(feature = "hash")]
fn sha256_hex(bytes: &[u8]) -> Result<String, super::ManifestError> {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    let mut out = String::with_capacity(64);
    for b in d {
        out.push("0123456789abcdef".as_bytes()[(b >> 4) as usize] as char);
        out.push("0123456789abcdef".as_bytes()[(b & 0x0F) as usize] as char);
    }
    Ok(out)
}

#[cfg(not(feature = "hash"))]
fn sha256_hex(_bytes: &[u8]) -> Result<String, super::ManifestError> {
    Err(super::ManifestError::Io(
        "hash",
        "sha256 requested but `hash` feature is disabled".to_string(),
    ))
}

// ---------------------------- digests & expectations ----------------------------

/// Verify provided digests over **canonical JSON** bytes.
/// Only verifies keys that are present both in `digests` and as actual paths.
/// Returns `Ok(())` when no digests were provided.
pub fn verify_digests(resolved: &super::ResolvedManifest) -> Result<(), super::ManifestError> {
    let Some(d) = &resolved.digests else { return Ok(()); };

    #[cfg(feature = "serde")]
    fn check_one(path: &Path, expect_hex: &str, label: &'static str) -> Result<(), super::ManifestError> {
        let canon = canonical_json_bytes_from_file(path)?;
        let got = sha256_hex(&canon)?;
        if got != expect_hex {
            return Err(super::ManifestError::DigestMismatch(
                label,
                format!("expected={} got={}", expect_hex, got),
            ));
        }
        Ok(())
    }

    #[cfg(not(feature = "serde"))]
    fn check_one(_path: &Path, _expect_hex: &str, _label: &'static str) -> Result<(), super::ManifestError> {
        Err(super::ManifestError::Io(
            "serde",
            "digest verification requires `serde` feature".to_string(),
        ))
    }

    if let Some(ref hex) = d.reg_path {
        check_one(&resolved.reg_path, hex, "reg_path")?;
    }
    if let Some(ref hex) = d.params_path {
        check_one(&resolved.params_path, hex, "params_path")?;
    }
    if let Some(ref hex) = d.ballot_tally_path {
        check_one(&resolved.ballot_tally_path, hex, "ballot_tally_path")?;
    }
    if let (Some(ref p), Some(ref hex)) = (&resolved.adjacency_path, &d.adjacency_path) {
        check_one(p, hex, "adjacency_path")?;
    } else if d.adjacency_path.is_some() && resolved.adjacency_path.is_none() {
        return Err(super::ManifestError::DigestForMissing("adjacency_path"));
    }
    if let (Some(ref p), Some(ref hex)) = (&resolved.autonomy_path, &d.autonomy_path) {
        check_one(p, hex, "autonomy_path")?;
    } else if d.autonomy_path.is_some() && resolved.autonomy_path.is_none() {
        return Err(super::ManifestError::DigestForMissing("autonomy_path"));
    }

    Ok(())
}

/// Enforce expectations (engine version / formula id) if provided.
pub fn enforce_expectations(
    resolved: &super::ResolvedManifest,
    actual_engine_version: &str,
    actual_formula_id_hex: &str,
) -> Result<(), super::ManifestError> {
    let Some(exp) = &resolved.expect else { return Ok(()); };

    if let Some(want) = &exp.engine_version {
        if want != actual_engine_version {
            return Err(super::ManifestError::ExpectationMismatch(
                "engine_version",
                format!("expected={} got={}", want, actual_engine_version),
            ));
        }
    }

    if let Some(want_fid) = &exp.formula_id_hex {
        if want_fid != actual_formula_id_hex {
            return Err(super::ManifestError::ExpectationMismatch(
                "formula_id_hex",
                format!("expected={} got={}", want_fid, actual_formula_id_hex),
            ));
        }
    }

    Ok(())
}

// ---------------------------- top-level load/verify ----------------------------

const MAX_MANIFEST_BYTES: usize = 4 * 1024 * 1024; // align with CLI quick-check (4 MiB)

/// Load a manifest JSON from `manifest_path`, validate, resolve under its directory,
/// and return a `ResolvedManifest`. This does **not** verify digests or expectations.
#[cfg(feature = "serde")]
pub fn load_and_resolve_manifest(manifest_path: &Path) -> Result<super::ResolvedManifest, super::ManifestError> {
    let mut f = fs::File::open(manifest_path)
        .map_err(|e| super::ManifestError::Io("read", format!("{} ({e})", manifest_path.display())))?;
    let mut buf = Vec::new();
    f.by_ref()
        .take(MAX_MANIFEST_BYTES as u64)
        .read_to_end(&mut buf)
        .map_err(|e| super::ManifestError::Io("read", format!("{} ({e})", manifest_path.display())))?;

    let man: super::Manifest = serde_json::from_slice(&buf)
        .map_err(|e| super::ManifestError::Io("parse", format!("{} ({e})", manifest_path.display())))?;

    super::validate_manifest(&man)?;

    let base = manifest_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    super::resolve_paths(&base, &man)
}

/// Convenience: full load+verify pipeline.
/// - Loads/validates/resolves the manifest.
/// - Verifies input digests if present (over canonical JSON).
/// - Enforces expectations if present.
#[cfg(feature = "serde")]
pub fn load_verify_manifest(
    manifest_path: &Path,
    actual_engine_version: &str,
    actual_formula_id_hex: &str,
) -> Result<super::ResolvedManifest, super::ManifestError> {
    let resolved = load_and_resolve_manifest(manifest_path)?;
    verify_digests(&resolved)?;
    enforce_expectations(&resolved, actual_engine_version, actual_formula_id_hex)?;
    Ok(resolved)
}

#[cfg(not(feature = "serde"))]
pub fn load_and_resolve_manifest(_manifest_path: &Path) -> Result<super::ResolvedManifest, super::ManifestError> {
    Err(super::ManifestError::Io(
        "serde",
        "manifest loading requires `serde` feature".to_string(),
    ))
}

#[cfg(not(feature = "serde"))]
pub fn load_verify_manifest(
    _manifest_path: &Path,
    _actual_engine_version: &str,
    _actual_formula_id_hex: &str,
) -> Result<super::ResolvedManifest, super::ManifestError> {
    Err(super::ManifestError::Io(
        "serde",
        "manifest loading requires `serde` feature".to_string(),
    ))
}
