//! crates/vm_io/src/hasher.rs
//!
//! Deterministic hashing and ID builders for canonical artifacts.
//!
//! Aligned with the 7 docs + 3 annexes:
//! - Canonical JSON hashing: UTF-8, LF, **sorted object keys**, array order preserved.
//! - IDs derive from canonical bytes: `RES:` (result), `FR:` (frontier map), and
//!   `RUN:` uses an RFC3339-UTC timestamp plus a hash of canonical run bytes.
//! - Hex digests are **lowercase**; no feature-gated fallbacks (always-on).
//!
//! Important distinctions:
//! - Use `sha256_canonical(..)` for JSON **values/structs** (goes through canonical_json).
//! - Use `sha256_hex(..)`, `sha256_stream(..)`, or `sha256_file(..)` for **raw bytes/files**.
//!
//! Notes on Normative Manifest (FID):
//! - `nm_digest_from_value` assumes the **caller** provides only the *Included* variables
//!   per Annex A (excludes presentation toggles and RNG seed).
//! - If you want a safer helper, use `nm_digest_from_included_keys(value, &keys)`.

#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

use serde::Serialize;
use serde_json::{self as sj, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

// Use the canonicalization utilities provided by this crate.
use crate::canonical_json::canonical_json_bytes;

/* ----------------------------------- Errors ----------------------------------- */

#[derive(Error, Debug)]
pub enum HashError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Serde(#[from] sj::Error),

    #[error("canonicalization error: {0}")]
    Canonical(String),

    #[error("invalid hex (expected lowercase 64-hex): {0}")]
    InvalidHex(String),

    #[error("invalid timestamp (expected RFC3339 UTC like 2025-08-12T10:00:00Z): {0}")]
    InvalidTimestamp(String),
}

/* ---------------------------------- Helpers ---------------------------------- */

/// Encode bytes as **lowercase** hex without external deps.
fn to_lower_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0F) as usize] as char);
    }
    out
}

/// Validate a lowercase 64-hex string and optionally shorten to `n` chars.
fn short_hex(hex64: &str, n: usize) -> Result<String, HashError> {
    if hex64.len() != 64 || !hex64.bytes().all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(HashError::InvalidHex(hex64.to_string()));
    }
    Ok(hex64[..n.min(64)].to_string())
}

/* -------------------------- Canonicalization bridge -------------------------- */

/// Convert any serializable value to **canonical JSON bytes** by first
/// converting to `serde_json::Value` and then delegating to `canonical_json_bytes`.
fn to_canon_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, HashError> {
    let v = sj::to_value(value)?;
    canonical_json_bytes(&v).map_err(|e| HashError::Canonical(e.to_string()))
}

/* ---------------------------- Canonical hashing ---------------------------- */

/// SHA-256 over **canonical JSON bytes** of any serializable value.
pub fn sha256_canonical<T: Serialize>(value: &T) -> Result<String, HashError> {
    let bytes = to_canon_bytes(value)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(to_lower_hex(&hasher.finalize()))
}

/// SHA-256 over **canonical JSON Value** (already parsed).
pub fn sha256_canonical_value(v: &Value) -> Result<String, HashError> {
    let bytes = canonical_json_bytes(v).map_err(|e| HashError::Canonical(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(to_lower_hex(&hasher.finalize()))
}

/* ------------------------------- Raw hashing ------------------------------- */

/// SHA-256 over raw bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    to_lower_hex(&hasher.finalize())
}

/// SHA-256 over a reader stream (raw, not canonicalized).
pub fn sha256_stream<R: Read>(reader: &mut R) -> Result<String, HashError> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024]; // 256 KiB buffer
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(to_lower_hex(&hasher.finalize()))
}

/// SHA-256 over a file’s raw bytes.
pub fn sha256_file(path: &Path) -> Result<String, HashError> {
    let f = File::open(path)?;
    let mut r = BufReader::new(f);
    sha256_stream(&mut r)
}

/* ---------------------------- Artifact ID builders ---------------------------- */

/// `RES:<hex>` — ID for `result.json` derived from **canonical** bytes.
pub fn res_id_from_canonical<T: Serialize>(value: &T) -> Result<String, HashError> {
    let hex = sha256_canonical(value)?;
    Ok(format!("RES:{hex}"))
}

/// `FR:<hex>` — ID for `frontier_map.json` derived from **canonical** bytes.
pub fn fr_id_from_canonical<T: Serialize>(value: &T) -> Result<String, HashError> {
    let hex = sha256_canonical(value)?;
    Ok(format!("FR:{hex}"))
}

/* --------------------------------- RUN IDs --------------------------------- */

/// Normalize a timestamp to canonical RFC3339 UTC seconds with trailing `Z`.
/// Accepts:
///   • `YYYY-MM-DDTHH:MM:SSZ`
///   • `YYYY-MM-DDTHH:MM:SS.ssssssZ` (fractional seconds)
///   • `YYYY-MM-DDTHH:MM:SS[.sss]±00:00` (normalized to `Z`)
fn normalize_rfc3339_utc_seconds(ts: &str) -> Result<String, HashError> {
    // Minimal strict parser: YYYY-MM-DDTHH:MM:SS[.frac](Z|+00:00|-00:00)
    if ts.len() < 20 { return Err(HashError::InvalidTimestamp(ts.to_string())); }

    // Core "YYYY-MM-DDTHH:MM:SS"
    let (y, m, d, hh, mm, ss) = {
        let y = ts.get(0..4).ok_or_else(|| HashError::InvalidTimestamp(ts.into()))?
            .parse::<i32>().map_err(|_| HashError::InvalidTimestamp(ts.into()))?;
        if &ts[4..5] != "-" { return Err(HashError::InvalidTimestamp(ts.into())); }
        let m = ts.get(5..7).ok_or_else(|| HashError::InvalidTimestamp(ts.into()))?
            .parse::<u32>().map_err(|_| HashError::InvalidTimestamp(ts.into()))?;
        if &ts[7..8] != "-" { return Err(HashError::InvalidTimestamp(ts.into())); }
        let d = ts.get(8..10).ok_or_else(|| HashError::InvalidTimestamp(ts.into()))?
            .parse::<u32>().map_err(|_| HashError::InvalidTimestamp(ts.into()))?;
        if &ts[10..11] != "T" { return Err(HashError::InvalidTimestamp(ts.into())); }
        let hh = ts.get(11..13).ok_or_else(|| HashError::InvalidTimestamp(ts.into()))?
            .parse::<u32>().map_err(|_| HashError::InvalidTimestamp(ts.into()))?;
        if &ts[13..14] != ":" { return Err(HashError::InvalidTimestamp(ts.into())); }
        let mm = ts.get(14..16).ok_or_else(|| HashError::InvalidTimestamp(ts.into()))?
            .parse::<u32>().map_err(|_| HashError::InvalidTimestamp(ts.into()))?;
        if &ts[16..17] != ":" { return Err(HashError::InvalidTimestamp(ts.into())); }
        let ss = ts.get(17..19).ok_or_else(|| HashError::InvalidTimestamp(ts.into()))?
            .parse::<u32>().map_err(|_| HashError::InvalidTimestamp(ts.into()))?;
        (y, m, d, hh, mm, ss)
    };

    // Basic range checks (not full calendar validation)
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || hh > 23 || mm > 59 || ss > 59 {
        return Err(HashError::InvalidTimestamp(ts.into()));
    }

    // Optional fractional seconds
    let mut idx = 19;
    if ts.get(idx..idx+1) == Some(".") {
        // consume 1..9 fractional digits
        idx += 1;
        let start = idx;
        while idx < ts.len() && ts.as_bytes()[idx].is_ascii_digit() {
            idx += 1;
            if idx - start > 9 { break; }
        }
        if idx == start {
            return Err(HashError::InvalidTimestamp(ts.into()));
        }
    }

    // Suffix: Z or ±00:00
    let suffix = ts.get(idx..).ok_or_else(|| HashError::InvalidTimestamp(ts.into()))?;
    let ok = matches!(suffix, "Z" | "+00:00" | "-00:00");
    if !ok {
        return Err(HashError::InvalidTimestamp(ts.into()));
    }

    Ok(format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z"))
}

/// `RUN:<timestamp>Z:<hex>` — ID for `run_record.json`.
/// `timestamp_utc` must be RFC3339 UTC; it is normalized to seconds + `Z`.
/// `run_bytes_canonical` must be canonical bytes of the run record payload you include in the ID.
pub fn run_id_from_bytes(timestamp_utc: &str, run_bytes_canonical: &[u8]) -> Result<String, HashError> {
    let ts = normalize_rfc3339_utc_seconds(timestamp_utc)?;
    let mut hasher = Sha256::new();
    hasher.update(run_bytes_canonical);
    let hex = to_lower_hex(&hasher.finalize());
    Ok(format!("RUN:{ts}:{hex}"))
}

/// Convenience: build a RUN id from a serializable run payload (canonicalized internally).
pub fn run_id_from_canonical<T: Serialize>(timestamp_utc: &str, run_value: &T) -> Result<String, HashError> {
    let bytes = to_canon_bytes(run_value)?;
    run_id_from_bytes(timestamp_utc, &bytes)
}

/* ------------------------- Normative Manifest (FID) ------------------------- */

/// Compute the digest of the **Normative Manifest** (caller must supply only *Included* vars).
/// If you need filtering, see `nm_digest_from_included_keys`.
pub fn nm_digest_from_value(nm_value_included_only: &Value) -> Result<String, HashError> {
    sha256_canonical_value(nm_value_included_only)
}

/// Safer helper: filter a flat JSON object to the provided `included_keys` before hashing.
/// - Keys not in `included_keys` are dropped.
/// - Values are copied as-is (assumed deterministic/primitive).
pub fn nm_digest_from_included_keys(value: &Value, included_keys: &[&str]) -> Result<String, HashError> {
    let obj = value.as_object().ok_or_else(|| {
        HashError::Serde(sj::Error::custom("normative manifest must be a JSON object"))
    })?;
    let set: std::collections::BTreeSet<&str> = included_keys.iter().copied().collect();
    let mut filtered = sj::Map::new();
    for (k, v) in obj {
        if set.contains(k.as_str()) {
            filtered.insert(k.clone(), v.clone());
        }
    }
    sha256_canonical_value(&Value::Object(filtered))
}

/* ------------------------------------ Tests ------------------------------------ */

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::json;

    #[test]
    fn hex_encoding_is_lowercase() {
        let h = sha256_hex(b"abc");
        assert_eq!(h, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert!(short_hex(&h, 12).unwrap().chars().all(|c| c.is_ascii_hexdigit() && c.is_lowercase()));
    }

    #[test]
    fn canonical_hashing_is_stable() {
        #[derive(Serialize)]
        struct T { b: u32, a: u32 }
        let t = T { b: 2, a: 1 };
        let h1 = sha256_canonical(&t).unwrap();
        // same fields different order through Value, should match
        let v = json!({"b":2,"a":1});
        let h2 = sha256_canonical_value(&v).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn run_id_normalizes_timestamp_and_fractional() {
        let id1 = run_id_from_bytes("2025-08-12T10:00:00Z", b"payload").unwrap();
        let id2 = run_id_from_bytes("2025-08-12T10:00:00.123Z", b"payload").unwrap();
        let id3 = run_id_from_bytes("2025-08-12T10:00:00+00:00", b"payload").unwrap();
        let id4 = run_id_from_bytes("2025-08-12T10:00:00-00:00", b"payload").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
        assert_eq!(id1, id4);
        assert!(id1.starts_with("RUN:2025-08-12T10:00:00Z:"));
    }

    #[test]
    fn nm_digest_filtering() {
        let v = json!({"050":"deterministic_order","052":52,"032":true,"x":"ignore"});
        let h_all = nm_digest_from_value(&v).unwrap();
        let h_inc = nm_digest_from_included_keys(&v, &["050"]).unwrap();
        assert_ne!(h_all, h_inc); // filtering changed the digest
    }
}
