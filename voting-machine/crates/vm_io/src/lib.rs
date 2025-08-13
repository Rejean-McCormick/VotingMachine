//! vm_io — canonical JSON I/O, schema validation (Draft 2020-12), hashing, and high-level loaders.
//!
//! This crate is I/O-facing but **offline-only** (no URLs, no network). It converts local JSON
//! artifacts into typed `vm_core` structures, provides canonical JSON bytes (sorted keys, LF),
//! and computes SHA-256 digests used for IDs and verification.
//!
//! Alignment:
//! - Shapes and contracts come from Docs 1–7 and Annex A/B/C (see repo root).
//! - Canonical artifacts: UTF-8, LF, **sorted keys**; arrays are pre-ordered by the engine.
//! - Validation schemas: JSON Schema Draft 2020-12 (feature `schemaval`).
//!
//! Notes:
//! - Feature matrix (see Cargo.toml): `std` (default), `serde`, `schemaval`, `hash`, `path_utf8`.
//! - We intentionally avoid any network/file-download behavior here.

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use thiserror::Error;

// Re-exports (narrow surface; callers usually import from vm_core directly)
pub use vm_core::{
    determinism::*,
    rounding::*,
    rng::*,
    // Types commonly consumed by loaders/pipeline:
    entities::*,
    variables::Params,
};

/// I/O errors surfaced by this crate. Messages are short/stable for test-pack diffs.
#[derive(Error, Debug)]
pub enum IoError {
    #[error("read error: {0}")]
    Read(std::io::Error),
    #[error("write error: {0}")]
    Write(std::io::Error),
    #[error("json parse error at {pointer}: {msg}")]
    Json { pointer: String, msg: String },
    #[error("schema validation failed at {pointer}: {msg}")]
    Schema { pointer: String, msg: String },
    #[error("manifest violation: {0}")]
    Manifest(String),
    #[error("expectation mismatch: {0}")]
    Expect(String),
    #[error("canonicalization: {0}")]
    Canon(String),
    #[error("hashing: {0}")]
    Hash(String),
    #[error("path: {0}")]
    Path(String),
    #[error("limit: {0}")]
    Limit(&'static str),
}

/// Canonical JSON (sorted keys, LF). Serialization requires the `serde` feature.
pub mod canonical_json {
    use super::IoError;

    #[cfg(feature = "serde")]
    use serde::Serialize;
    #[cfg(feature = "serde")]
    use serde_json::{Map, Value};

    /// Convert any `Serialize` value to canonical JSON bytes (sorted object keys, LF newlines).
    #[cfg(feature = "serde")]
    pub fn to_canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, IoError> {
        let val = serde_json::to_value(value)
            .map_err(|e| IoError::Json { pointer: "/".into(), msg: e.to_string() })?;
        let canon = canonicalize_value(val);
        let mut bytes = serde_json::to_vec(&canon)
            .map_err(|e| IoError::Canon(e.to_string()))?;
        // Enforce LF: replace CRLF with LF in-place (JSON writer already uses '\n', but be strict)
        if bytes.windows(2).any(|w| w == b"\r\n") {
            let s = String::from_utf8(bytes).map_err(|e| IoError::Canon(e.to_string()))?;
            bytes = s.replace("\r\n", "\n").into_bytes();
        }
        Ok(bytes)
    }

    /// Write canonical JSON to a file at `path` (overwrites). Parent dirs must exist.
    #[cfg(feature = "serde")]
    pub fn write_canonical_file<T: Serialize, P: AsRef<std::path::Path>>(value: &T, path: P) -> Result<(), IoError> {
        use std::io::Write;
        let bytes = to_canonical_bytes(value)?;
        let mut f = std::fs::File::create(path.as_ref()).map_err(IoError::Write)?;
        f.write_all(&bytes).map_err(IoError::Write)
    }

    /// Recursively sort object keys; preserve array order; leave numbers/strings as-is.
    #[cfg(feature = "serde")]
    fn canonicalize_value(v: Value) -> Value {
        match v {
            Value::Object(map) => {
                // Collect and sort keys, re-inserting in sorted order to fix iteration order.
                let mut keys: Vec<_> = map.keys().cloned().collect();
                keys.sort();
                let mut out = Map::with_capacity(map.len());
                for k in keys {
                    out.insert(k.clone(), canonicalize_value(map.get(&k).cloned().unwrap_or(Value::Null)));
                }
                Value::Object(out)
            }
            Value::Array(xs) => Value::Array(xs.into_iter().map(canonicalize_value).collect()),
            other => other,
        }
    }

    // Placeholders when `serde` feature is off (crate won't be very useful without it).
    #[cfg(not(feature = "serde"))]
    #[allow(unused_variables)]
    pub fn to_canonical_bytes<T>(_value: &T) -> Result<Vec<u8>, IoError> {
        Err(IoError::Canon("serde feature disabled".into()))
    }
    #[cfg(not(feature = "serde"))]
    #[allow(unused_variables)]
    pub fn write_canonical_file<T, P>(_value: &T, _path: P) -> Result<(), IoError> {
        Err(IoError::Canon("serde feature disabled".into()))
    }
}

/// SHA-256 digests over canonical bytes/files. Requires `hash` feature for real hashing.
pub mod hasher {
    use super::IoError;

    #[cfg(feature = "hash")]
    use sha2::{Digest, Sha256};

    /// Return lowercase hex SHA-256 of `bytes`.
    #[cfg(feature = "hash")]
    pub fn sha256_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let out = hasher.finalize();
        hex::encode(out)
    }

    /// Stream a file and return its SHA-256 hex.
    #[cfg(feature = "hash")]
    pub fn sha256_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, IoError> {
        use std::io::Read;
        let mut f = std::fs::File::open(path.as_ref()).map_err(IoError::Read)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 32 * 1024];
        loop {
            let n = f.read(&mut buf).map_err(IoError::Read)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(hex::encode(hasher.finalize()))
    }

    /// Canonicalize a serializable value and return its SHA-256 hex.
    #[cfg(all(feature = "hash", feature = "serde"))]
    pub fn sha256_canonical<T: serde::Serialize>(value: &T) -> Result<String, IoError> {
        let bytes = crate::canonical_json::to_canonical_bytes(value)?;
        Ok(sha256_hex(&bytes))
    }

    // Placeholders when features are off.
    #[cfg(not(feature = "hash"))]
    pub fn sha256_hex(_bytes: &[u8]) -> String { String::new() }
    #[cfg(not(feature = "hash"))]
    pub fn sha256_file<P: AsRef<std::path::Path>>(_path: P) -> Result<String, IoError> { Err(IoError::Hash("hash feature disabled".into())) }
    #[cfg(any(not(feature = "hash"), not(feature = "serde")))]
    pub fn sha256_canonical<T>(_value: &T) -> Result<String, IoError> { Err(IoError::Hash("hash/serde feature disabled".into())) }
}

/// Manifest loading and path resolution. Local filesystem only; URLs are rejected.
pub mod manifest {
    use super::IoError;

    /// Parsed manifest (normative: must point to registry, params, **ballot_tally**).
    #[derive(Debug, Clone)]
    pub struct Manifest {
        pub reg_path: String,
        pub params_path: String,
        pub ballot_tally_path: String, // required; no raw ballots path in canonical pipeline
        pub adjacency_path: Option<String>,
        pub expect: Option<Expect>, // optional sanity locks
        pub digests: std::collections::BTreeMap<String, String>, // { path -> sha256 hex }
        pub notes: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct Expect {
        pub formula_id: Option<String>,
        pub engine_version: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct ResolvedPaths {
        pub base_dir: std::path::PathBuf,
        pub reg: std::path::PathBuf,
        pub params: std::path::PathBuf,
        pub tally: std::path::PathBuf,
        pub adjacency: Option<std::path::PathBuf>,
    }

    /// Load a manifest JSON file and do basic shape/URL checks. (Schema validation is feature-gated.)
    #[cfg(feature = "serde")]
    pub fn load_manifest<P: AsRef<std::path::Path>>(path: P) -> Result<Manifest, IoError> {
        use std::io::Read;
        let p = path.as_ref();
        let mut f = std::fs::File::open(p).map_err(IoError::Read)?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(IoError::Read)?;
        let val: serde_json::Value =
            serde_json::from_str(&s).map_err(|e| IoError::Json { pointer: "/".into(), msg: e.to_string() })?;

        // Optional: schema validation
        #[cfg(feature = "schemaval")]
        crate::schema::validate_value(crate::schema::SchemaKind::Manifest, &val)?;

        let reg_path = val.get("reg_path").and_then(|v| v.as_str()).ok_or_else(|| IoError::Manifest("missing reg_path".into()))?;
        let params_path = val.get("params_path").and_then(|v| v.as_str()).ok_or_else(|| IoError::Manifest("missing params_path".into()))?;
        let ballot_tally_path = val
            .get("ballot_tally_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IoError::Manifest("missing ballot_tally_path".into()))?;

        // Reject URLs — offline-only contract
        for (label, s) in [("reg_path", reg_path), ("params_path", params_path), ("ballot_tally_path", ballot_tally_path)] {
            if s.starts_with("http://") || s.starts_with("https://") {
                return Err(IoError::Manifest(format!("{label} must be a local path (no URLs)")));
            }
        }

        let adjacency_path = val.get("adjacency_path").and_then(|v| v.as_str()).map(str::to_string);
        let expect = val.get("expect").and_then(|e| {
            let mut ex = Expect::default();
            if let Some(fid) = e.get("formula_id").and_then(|v| v.as_str()) {
                ex.formula_id = Some(fid.to_string());
            }
            if let Some(ev) = e.get("engine_version").and_then(|v| v.as_str()) {
                ex.engine_version = Some(ev.to_string());
            }
            Some(ex)
        });

        // Optional digests map
        let mut digests = std::collections::BTreeMap::new();
        if let Some(obj) = val.get("digests").and_then(|d| d.as_object()) {
            for (k, v) in obj {
                if let Some(hex) = v.get("sha256").and_then(|h| h.as_str()) {
                    if !is_hex64(hex) {
                        return Err(IoError::Manifest(format!("invalid sha256 for {k}")));
                    }
                    digests.insert(k.clone(), hex.to_string());
                }
            }
        }

        Ok(Manifest {
            reg_path: reg_path.to_string(),
            params_path: params_path.to_string(),
            ballot_tally_path: ballot_tally_path.to_string(),
            adjacency_path,
            expect,
            digests,
            notes: val.get("notes").and_then(|v| v.as_str()).map(str::to_string),
        })
    }

    #[cfg(not(feature = "serde"))]
    #[allow(unused_variables)]
    pub fn load_manifest<P: AsRef<std::path::Path>>(_path: P) -> Result<Manifest, IoError> {
        Err(IoError::Manifest("serde feature disabled".into()))
    }

    /// Resolve relative paths against the manifest's directory.
    pub fn resolve_paths(base: &std::path::Path, man: &Manifest) -> Result<ResolvedPaths, IoError> {
        let base_dir = base.to_path_buf();
        let rp = base_dir.join(&man.reg_path);
        let pp = base_dir.join(&man.params_path);
        let tp = base_dir.join(&man.ballot_tally_path);
        let ap = man.adjacency_path.as_ref().map(|s| base_dir.join(s));
        Ok(ResolvedPaths { base_dir, reg: rp, params: pp, tally: tp, adjacency: ap })
    }

    /// Quick expectation check (called by pipeline once actual values are known).
    pub fn verify_expectations(man: &Manifest, actual_formula_id: Option<&str>, actual_engine_version: Option<&str>) -> Result<(), IoError> {
        if let Some(exp) = &man.expect {
            if let (Some(want), Some(have)) = (&exp.formula_id, actual_formula_id) {
                if want != have {
                    return Err(IoError::Expect(format!("formula_id mismatch (want {want}, have {have})")));
                }
            }
            if let (Some(want), Some(have)) = (&exp.engine_version, actual_engine_version) {
                if want != have {
                    return Err(IoError::Expect(format!("engine_version mismatch (want {want}, have {have})")));
                }
            }
        }
        Ok(())
    }

    fn is_hex64(s: &str) -> bool {
        s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
    }
}

/// JSON Schema validation (Draft 2020-12). If the `schemaval` feature is disabled,
/// `validate_value` becomes a no-op that returns `Ok(())`.
pub mod schema {
    use super::IoError;

    #[derive(Debug, Clone, Copy)]
    pub enum SchemaKind {
        DivisionRegistry,
        ParameterSet,
        BallotTally,
        Manifest,
        Result,
        RunRecord,
        FrontierMap,
    }

    #[cfg(feature = "schemaval")]
    pub fn validate_value(kind: SchemaKind, value: &serde_json::Value) -> Result<(), IoError> {
        use jsonschema::{Draft, JSONSchema};

        // NOTE: Schemas are expected to live in the repository; wire-up via include_str! or
        // external file access as your build prefers. We keep literal placeholders here to
        // avoid compile-time path coupling. Replace these with real `include_str!` in your repo.
        fn schema_text(kind: SchemaKind) -> &'static str {
            match kind {
                SchemaKind::DivisionRegistry => include_str!("../../
