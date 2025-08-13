//! crates/vm_io/src/manifest.rs
//! Parse, validate, and resolve the run **manifest.json** into concrete local paths,
//! with optional expectations and digest verification. No network I/O.
//!
//! Contracts (concise):
//! - Required paths: registry, params, **ballot_tally**. Optional: adjacency. (Doc 5A S0) 
//! - Offline only: reject URLs; resolve relative paths against the manifest's directory. (Doc 3A §3) 
//! - Optional `expect.{formula_id,engine_version}` and `digests{}` checks.

use std::collections::BTreeMap;
use std::fs;
use std::io::{Error as IoStdError, ErrorKind};
use std::path::{Path, PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{self as sj, Value};

#[cfg(feature = "schemaval")]
use jsonschema::{Draft, JSONSchema};

use sha2::{Digest, Sha256};

use crate::IoError;

// ---------- Module knobs (can be surfaced as Config later) ----------
const REJECT_URLS: bool = true;
const ALLOW_PARENT_TRAVERSAL: bool = true; // set false to confine inside base_dir
const MAX_BYTES: usize = 2 * 1024 * 1024;

// Schema embedded from crate root: <crate>/schemas/manifest.schema.json
#[cfg(feature = "schemaval")]
const MANIFEST_SCHEMA_JSON: &str = include_str!("../schemas/manifest.schema.json");

// ---------- Public API types ----------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String, // "MAN:…"
    pub reg_path: String,
    pub params_path: String,
    pub ballot_tally_path: String, // REQUIRED (normative pipeline)
    pub adjacency_path: Option<String>,
    pub expect: Option<Expect>,
    pub digests: Option<BTreeMap<String, DigestEntry>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expect {
    pub formula_id: Option<String>,     // expected 64-hex
    pub engine_version: Option<String>, // semver-ish string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestEntry {
    pub sha256: String, // 64-hex (lowercase preferred)
}

#[derive(Debug, Clone)]
pub struct ResolvedPaths {
    pub base_dir: Utf8PathBuf,
    pub reg: Utf8PathBuf,
    pub params: Utf8PathBuf,
    pub tally: Utf8PathBuf,
    pub adjacency: Option<Utf8PathBuf>,
}

// ---------- Top-level functions ----------
pub fn load_manifest<P: AsRef<Path>>(path: P) -> Result<Manifest, IoError> {
    let path = path.as_ref();
    // Size guard + read
    let meta = fs::metadata(path).map_err(IoError::from)?;
    if meta.len() as usize > MAX_BYTES {
        return Err(invalid_data_err(format!(
            "manifest exceeds MAX_BYTES ({} > {})",
            meta.len(),
            MAX_BYTES
        )));
    }
    let bytes = fs::read(path).map_err(IoError::from)?;

    // Parse JSON
    let raw: Value = sj::from_slice(&bytes).map_err(|e| json_err("/", e))?;

    // Defensive: reject legacy `ballots_path` if present
    if let Some(obj) = raw.as_object() {
        if obj.contains_key("ballots_path") {
            return Err(invalid_data_err("legacy field `ballots_path` is not allowed".into()));
        }
    }

    // Optional: schema validation
    #[cfg(feature = "schemaval")]
    {
        let schema_v: Value = sj::from_str(MANIFEST_SCHEMA_JSON)
            .map_err(|e| invalid_data_err(format!("invalid embedded manifest.schema.json: {e}")))?;
        let compiled = JSONSchema::options()
            .with_draft(Draft::Draft2020_12)
            .compile(&schema_v)
            .map_err(|e| invalid_data_err(format!("schema compile error: {e}")))?;
        if let Err(iter) = compiled.validate(&raw) {
            // Report first violation with a JSON Pointer-like path
            if let Some(err) = iter.into_iter().next() {
                let ptr = err.instance_path.to_string();
                let msg = err.to_string();
                return Err(schema_err(
                    if ptr.is_empty() { "/" } else { ptr.as_str() },
                    msg,
                ));
            }
        }
    }

    // Into typed Manifest
    let man: Manifest = sj::from_value(raw).map_err(|e| json_err("/", e))?;
    Ok(man)
}

pub fn validate_manifest(man: &Manifest) -> Result<(), IoError> {
    // Required: ballot_tally_path non-empty
    if man.ballot_tally_path.trim().is_empty() {
        return Err(invalid_data_err("`ballot_tally_path` is required and must be non-empty".into()));
    }

    // URL rejection (all path-like fields)
    if REJECT_URLS {
        for (label, s) in [
            ("reg_path", &man.reg_path),
            ("params_path", &man.params_path),
            ("ballot_tally_path", &man.ballot_tally_path),
        ] {
            if looks_like_url(s) {
                return Err(invalid_data_err(format!("{} must be a local file path (no URLs)", label)));
            }
        }
        if let Some(adj) = &man.adjacency_path {
            if looks_like_url(adj) {
                return Err(invalid_data_err("adjacency_path must be a local file path (no URLs)".into()));
            }
        }
    }

    // Optional digests: quick shape validation (hex form only; file existence checked in verify_digests)
    if let Some(dmap) = &man.digests {
        for (k, v) in dmap {
            if !["reg_path", "params_path", "ballot_tally_path", "adjacency_path"].contains(&k.as_str()) {
                return Err(invalid_data_err(format!("unknown digest key `{}`", k)));
            }
            if !is_hex64(&v.sha256) {
                return Err(invalid_data_err(format!("digest for `{}` must be 64 hex chars", k)));
            }
        }
    }

    Ok(())
}

pub fn resolve_paths<P: AsRef<Path>>(manifest_file: P, man: &Manifest) -> Result<ResolvedPaths, IoError> {
    let base_dir_fs: &Path = manifest_file
        .as_ref()
        .parent()
        .ok_or_else(|| invalid_data_err("manifest_file has no parent directory".into()))?;

    let base_dir_utf8 = to_utf8(base_dir_fs)?;

    let reg = normalize_join(&base_dir_utf8, &man.reg_path)?;
    let params = normalize_join(&base_dir_utf8, &man.params_path)?;
    let tally = normalize_join(&base_dir_utf8, &man.ballot_tally_path)?;
    let adjacency = match &man.adjacency_path {
        Some(p) => Some(normalize_join(&base_dir_utf8, p)?),
        None => None,
    };

    // Optional confinement: disallow escaping base_dir
    if !ALLOW_PARENT_TRAVERSAL {
        for (label, p) in [
            ("reg_path", &reg),
            ("params_path", &params),
            ("ballot_tally_path", &tally),
        ] {
            if !p.starts_with(&base_dir_utf8) {
                return Err(invalid_data_err(format!(
                    "{} escapes base_dir after normalization",
                    label
                )));
            }
        }
        if let Some(adj) = &adjacency {
            if !adj.starts_with(&base_dir_utf8) {
                return Err(invalid_data_err("adjacency_path escapes base_dir after normalization".into()));
            }
        }
    }

    Ok(ResolvedPaths {
        base_dir: base_dir_utf8,
        reg,
        params,
        tally,
        adjacency,
    })
}

pub fn enforce_expectations(
    man: &Manifest,
    engine_version: &str,
    formula_id_hex: &str,
) -> Result<(), IoError> {
    if let Some(exp) = &man.expect {
        if let Some(exp_fid) = &exp.formula_id {
            if !is_hex64(exp_fid) {
                return Err(expect_err("expect.formula_id is not a 64-hex string"));
            }
            if exp_fid.to_ascii_lowercase() != formula_id_hex.to_ascii_lowercase() {
                return Err(expect_err("formula_id mismatch"));
            }
        }
        if let Some(exp_eng) = &exp.engine_version {
            if exp_eng != engine_version {
                return Err(expect_err("engine_version mismatch"));
            }
        }
    }
    Ok(())
}

pub fn verify_digests(
    paths: &ResolvedPaths,
    digests: &BTreeMap<String, DigestEntry>,
) -> Result<(), IoError> {
    for (k, v) in digests {
        let (label, p) = match k.as_str() {
            "reg_path" => ("registry", &paths.reg),
            "params_path" => ("params", &paths.params),
            "ballot_tally_path" => ("ballot_tally", &paths.tally),
            "adjacency_path" => {
                let adj = paths
                    .adjacency
                    .as_ref()
                    .ok_or_else(|| invalid_data_err("digest provided for adjacency_path, but manifest has no adjacency_path".into()))?;
                ("adjacency", adj)
            }
            _ => return Err(invalid_data_err(format!("unknown digest path: {}", k))),
        };

        // Read bytes and compute sha256
        let bytes = fs::read(p).map_err(IoError::from)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let got = hex::encode(hasher.finalize());
        if got.to_ascii_lowercase() != v.sha256.to_ascii_lowercase() {
            return Err(invalid_data_err(format!(
                "digest mismatch for {}: expected {}, got {}",
                label, v.sha256, got
            )));
        }
    }
    Ok(())
}

// ---------- Helpers ----------
fn json_err(pointer: &str, e: sj::Error) -> IoError {
    invalid_data_err(format!("JSON parse error at {}: {}", pointer, e))
}

fn schema_err(pointer: &str, msg: String) -> IoError {
    invalid_data_err(format!("Schema violation at {}: {}", pointer, msg))
}

fn invalid_data_err(msg: impl Into<String>) -> IoError {
    IoError::from(IoStdError::new(ErrorKind::InvalidData, msg.into()))
}

fn expect_err(msg: impl Into<String>) -> IoError {
    IoError::from(IoStdError::new(ErrorKind::InvalidData, format!("Expect: {}", msg.into())))
}

fn is_hex64(s: &str) -> bool {
    if s.len() != 64 {
        return false;
    }
    s.bytes().all(|b| (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b))
}

fn looks_like_url(s: &str) -> bool {
    let sl = s.trim().to_ascii_lowercase();
    sl.starts_with("http://") || sl.starts_with("https://")
}

fn to_utf8(p: &Path) -> Result<Utf8PathBuf, IoError> {
    Utf8Path::from_path(p)
        .map(|u| u.to_owned())
        .ok_or_else(|| invalid_data_err(format!("non-UTF8 path: {}", p.display())))
}

// Lexical normalization (no filesystem access): resolves "." and ".." components.
// This does not follow symlinks and keeps absolute vs relative-ness.
fn lexical_normalize(mut p: Utf8PathBuf) -> Utf8PathBuf {
    let is_abs = p.is_absolute();
    let mut out = Utf8PathBuf::new();
    for comp in p.components() {
        match comp.as_str() {
            "." => {}
            ".." => {
                let _ = out.pop();
            }
            other => out.push(other),
        }
    }
    if is_abs {
        // Ensure we keep absolute if original was absolute
        if !out.is_absolute() {
            out = Utf8Path::new("/").join(out);
        }
    }
    out
}

fn normalize_join(base: &Utf8Path, rel: &str) -> Result<Utf8PathBuf, IoError> {
    // If `rel` is absolute, take as-is; else join to base.
    let relp = Utf8Path::new(rel);
    let joined = if relp.is_absolute() {
        relp.to_path_buf()
    } else {
        base.join(relp)
    };
    Ok(lexical_normalize(joined))
}

// ---------- Tests ----------
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn expect_checks() {
        let man = Manifest {
            id: "MAN:demo".into(),
            reg_path: "reg.json".into(),
            params_path: "params.json".into(),
            ballot_tally_path: "tally.json".into(),
            adjacency_path: None,
            expect: Some(Expect {
                formula_id: Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd".into()),
                engine_version: Some("v1.2.3".into()),
            }),
            digests: None,
            notes: None,
        };
        // OK
        enforce_expectations(
            &man,
            "v1.2.3",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd",
        )
        .unwrap();

        // Bad hex
        let mut man_bad = man.clone();
        man_bad.expect.as_mut().unwrap().formula_id = Some("xyz".into());
        assert!(enforce_expectations(&man_bad, "v1.2.3", "00").is_err());

        // Mismatch
        let mut man_bad2 = man.clone();
        man_bad2.expect.as_mut().unwrap().engine_version = Some("v9.9.9".into());
        assert!(enforce_expectations(
            &man_bad2,
            "v1.2.3",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd"
        )
        .is_err());
    }

    #[test]
    fn resolve_and_verify_digests() {
        let dir = tempdir().unwrap();
        let base = dir.path().join("cases");
        fs::create_dir_all(&base).unwrap();

        // Write three files
        let reg_p = base.join("reg.json");
        let params_p = base.join("params.json");
        let tally_p = base.join("tally.json");
        for (p, content) in [
            (&reg_p, br#"{"a":1}"#.as_slice()),
            (&params_p, br#"{"b":2}"#.as_slice()),
            (&tally_p, br#"{"c":3}"#.as_slice()),
        ] {
            let mut f = File::create(p).unwrap();
            f.write_all(content).unwrap();
        }

        let man = Manifest {
            id: "MAN:demo".into(),
            reg_path: "reg.json".into(),
            params_path: "params.json".into(),
            ballot_tally_path: "tally.json".into(),
            adjacency_path: None,
            expect: None,
            digests: None,
            notes: None,
        };

        let man_path = base.join("manifest.json");
        fs::write(&man_path, br#"{"id":"x","reg_path":"reg.json","params_path":"params.json","ballot_tally_path":"tally.json"}"#).unwrap();

        let res = resolve_paths(&man_path, &man).unwrap();

        // Compute digests and verify
        let mut d = BTreeMap::new();
        for (k, p) in [
            ("reg_path", res.reg.clone()),
            ("params_path", res.params.clone()),
            ("ballot_tally_path", res.tally.clone()),
        ] {
            let bytes = fs::read(p).unwrap();
            let mut h = Sha256::new();
            h.update(&bytes);
            d.insert(k.to_string(), DigestEntry { sha256: hex::encode(h.finalize()) });
        }
        verify_digests(&res, &d).unwrap();
    }

    #[test]
    fn url_rejection_and_legacy() {
        // URL rejection via validate_manifest
        let man = Manifest {
            id: "MAN:demo".into(),
            reg_path: "https://example.com/reg.json".into(),
            params_path: "params.json".into(),
            ballot_tally_path: "tally.json".into(),
            adjacency_path: None,
            expect: None,
            digests: None,
            notes: None,
        };
        assert!(validate_manifest(&man).is_err());
    }
}
