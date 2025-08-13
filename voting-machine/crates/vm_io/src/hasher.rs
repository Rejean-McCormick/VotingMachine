//! SHA-256 hashing utilities over **canonical JSON** bytes,
//! plus helpers to build prefixed IDs (RES:/RUN:/FR:) and
//! to compute the Normative Manifest digest (nm_digest) and Formula ID (FID).
//!
//! Deterministic: same canonical structure ⇒ same lowercase 64-hex across OS/arch.

#![forbid(unsafe_code)]

use crate::IoError;

#[cfg(feature = "hash")]
use digest::Digest;
#[cfg(feature = "hash")]
use sha2::Sha256;

#[cfg(all(feature = "hash", feature = "serde"))]
use crate::canonical_json::to_canonical_bytes;
#[cfg(all(feature = "hash", feature = "serde"))]
use serde::Serialize;

/// Compute lowercase 64-hex SHA-256 of raw bytes.
#[cfg(feature = "hash")]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    hex::encode(out) // lowercase
}

#[cfg(not(feature = "hash"))]
pub fn sha256_hex(_bytes: &[u8]) -> String {
    // Keep signature available even if feature is off (used rarely directly).
    // Caller should not rely on this in no-hash builds.
    String::new()
}

/// Streaming SHA-256 for any reader; returns lowercase 64-hex.
#[cfg(feature = "hash")]
pub fn sha256_stream<R: std::io::Read>(reader: &mut R) -> Result<String, IoError> {
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024]; // 64 KiB
    loop {
        let n = reader.read(&mut buf).map_err(IoError::Read)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(not(feature = "hash"))]
pub fn sha256_stream<R: std::io::Read>(_reader: &mut R) -> Result<String, IoError> {
    Err(IoError::Hash("hash feature disabled".into()))
}

/// SHA-256 of canonical JSON representation (sorted keys, LF); returns lowercase 64-hex.
#[cfg(all(feature = "hash", feature = "serde"))]
pub fn sha256_canonical<T: Serialize>(value: &T) -> Result<String, IoError> {
    let bytes = to_canonical_bytes(value)?;
    Ok(sha256_hex(&bytes))
}

#[cfg(not(all(feature = "hash", feature = "serde")))]
pub fn sha256_canonical<T>(_value: &T) -> Result<String, IoError> {
    Err(IoError::Hash("hash+serde features required".into()))
}

/// Convenience: hash a file from disk; returns lowercase 64-hex.
#[cfg(feature = "hash")]
pub fn sha256_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, IoError> {
    let mut f = std::fs::File::open(path).map_err(IoError::Read)?;
    sha256_stream(&mut f)
}

#[cfg(not(feature = "hash"))]
pub fn sha256_file<P: AsRef<std::path::Path>>(_path: P) -> Result<String, IoError> {
    Err(IoError::Hash("hash feature disabled".into()))
}

// ---------- Prefixed ID builders (full 64-hex) ----------

/// "RES:<hex64>" — Result ID from canonical JSON of the Result struct.
#[cfg(all(feature = "hash", feature = "serde"))]
pub fn res_id_from_canonical<T: Serialize>(value: &T) -> Result<String, IoError> {
    let h = sha256_canonical(value)?;
    Ok(format!("RES:{h}"))
}

#[cfg(not(all(feature = "hash", feature = "serde")))]
pub fn res_id_from_canonical<T>(_value: &T) -> Result<String, IoError> {
    Err(IoError::Hash("hash+serde features required".into()))
}

/// "FR:<hex64>" — FrontierMap ID from canonical JSON of the FrontierMap struct.
#[cfg(all(feature = "hash", feature = "serde"))]
pub fn fr_id_from_canonical<T: Serialize>(value: &T) -> Result<String, IoError> {
    let h = sha256_canonical(value)?;
    Ok(format!("FR:{h}"))
}

#[cfg(not(all(feature = "hash", feature = "serde")))]
pub fn fr_id_from_canonical<T>(_value: &T) -> Result<String, IoError> {
    Err(IoError::Hash("hash+serde features required".into()))
}

/// "RUN:<RFC3339Z>-<hex64>" — RunRecord ID built from caller-supplied UTC timestamp
/// (exact shape "YYYY-MM-DDTHH:MM:SSZ") and arbitrary canonical-basis bytes.
#[cfg(feature = "hash")]
pub fn run_id_from_bytes(timestamp_utc: &str, bytes: &[u8]) -> Result<String, IoError> {
    if !is_rfc3339_utc_seconds(timestamp_utc) {
        return Err(IoError::Hash("bad timestamp (expect RFC3339 UTC 'YYYY-MM-DDTHH:MM:SSZ')".into()));
    }
    let h = sha256_hex(bytes);
    Ok(format!("RUN:{timestamp_utc}-{h}"))
}

#[cfg(not(feature = "hash"))]
pub fn run_id_from_bytes(_timestamp_utc: &str, _bytes: &[u8]) -> Result<String, IoError> {
    Err(IoError::Hash("hash feature disabled".into()))
}

// ---------- Normative Manifest / Formula ID ----------

/// Compute SHA-256 over the **Normative Manifest** canonical bytes.
/// Caller must pre-filter to *Included* variables only per Annex A.
#[cfg(all(feature = "hash", feature = "serde"))]
pub fn nm_digest_from_value(nm: &serde_json::Value) -> Result<String, IoError> {
    sha256_canonical(nm)
}

#[cfg(not(all(feature = "hash", feature = "serde")))]
pub fn nm_digest_from_value(_nm: &serde_json::Value) -> Result<String, IoError> {
    Err(IoError::Hash("hash+serde features required".into()))
}

/// Compute Formula ID (FID) from the **Normative Manifest**.
/// Currently FID == nm_digest (64-hex). Centralized here for future policy changes.
#[cfg(all(feature = "hash", feature = "serde"))]
pub fn formula_id_from_nm(nm: &serde_json::Value) -> Result<String, IoError> {
    nm_digest_from_value(nm)
}

#[cfg(not(all(feature = "hash", feature = "serde")))]
pub fn formula_id_from_nm(_nm: &serde_json::Value) -> Result<String, IoError> {
    Err(IoError::Hash("hash+serde features required".into()))
}

// ---------- Hex helpers ----------

/// True iff string is **lowercase** 64-hex.
pub fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.as_bytes().iter().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Return a short prefix of a 64-hex string (1..=64). Errors if non-hex or out of range.
pub fn short_hex(full_hex: &str, len: usize) -> Result<String, IoError> {
    if !(1..=64).contains(&len) {
        return Err(IoError::Hash("short_hex length out of range".into()));
    }
    if !is_hex64(full_hex) {
        return Err(IoError::Hash("short_hex expects lowercase 64-hex".into()));
    }
    Ok(full_hex[..len].to_string())
}

// ---------- local helpers ----------

/// Strict check for "YYYY-MM-DDTHH:MM:SSZ".
fn is_rfc3339_utc_seconds(s: &str) -> bool {
    if s.len() != 20 { return false; }
    let b = s.as_bytes();
    // YYYY-MM-DDTHH:MM:SSZ
    // 0123 5 78  11  14  17 19
    //     -   -   T   :   :   Z
    fn is_digit(x: u8) -> bool { (b'0'..=b'9').contains(&x) }
    for &i in &[0,1,2,3,5,6,8,9,11,12,14,15,17,18] {
        if !is_digit(b[i]) { return false; }
    }
    b[4] == b'-' && b[7] == b'-' && b[10] == b'T' &&
    b[13] == b':' && b[16] == b':' && b[19] == b'Z'
}
