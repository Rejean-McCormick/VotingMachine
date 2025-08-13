//! crates/vm_io/src/canonical_json.rs
//! Canonical JSON bytes (UTF-8; objects with lexicographically sorted keys; arrays untouched).
//! Compact (default) and pretty (2-space, LF) variants; atomic file write.
//!
//! Notes:
//! - Keys are sorted by UTF-8 byte order using a recursive transformer.
//! - Compact: minimal whitespace, no trailing newline.
//! - Pretty: 2-space indent, LF newlines (no trailing newline).
//! - Errors bubble from serde_json/std::io (assumes crate::IoError has From conversions).

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{self as sj, Map, Value};

use crate::IoError;

/// Return canonical JSON bytes (UTF-8, sorted keys, compact; no trailing newline).
pub fn to_canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, IoError> {
    // 1) Serialize to Value (serde_json already rejects NaN/Inf)
    let v: Value = sj::to_value(value).map_err(IoError::from)?;
    // 2) Recursively sort object keys; arrays untouched
    let v = canonicalize_value(v);
    // 3) Emit compact bytes (no trailing newline)
    let mut out = Vec::with_capacity(4096);
    let mut ser = sj::Serializer::new(&mut out);
    v.serialize(&mut ser).map_err(IoError::from)?;
    Ok(out)
}

/// Pretty variant (2-space indent) that still sorts keys and enforces LF.
pub fn to_canonical_pretty_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, IoError> {
    let v: Value = sj::to_value(value).map_err(IoError::from)?;
    let v = canonicalize_value(v);

    let mut out = Vec::with_capacity(4096);
    let fmt = sj::ser::PrettyFormatter::with_indent(b"  "); // PrettyFormatter emits '\n' (LF)
    let mut ser = sj::Serializer::with_formatter(&mut out, fmt);
    v.serialize(&mut ser).map_err(IoError::from)?;
    Ok(out)
}

/// Write canonical JSON file (creates parent dirs; atomic replace via temp+rename).
pub fn write_canonical_file<T: Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<(), IoError> {
    let path = path.as_ref();

    // Build bytes
    let bytes = to_canonical_bytes(value)?;

    // Ensure parent exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(IoError::from)?;
    }

    // Temp path: "<file>.<ext>.tmp" or "<file>.tmp"
    let tmp_path = make_tmp_path(path);

    // Write to temp, flush, fsync
    {
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(IoError::from)?;
        let mut w = BufWriter::new(f);
        w.write_all(&bytes).map_err(IoError::from)?;
        w.flush().map_err(IoError::from)?;
        // Ensure durability before rename
        w.into_inner().map_err(IoError::from)?.sync_all().map_err(IoError::from)?;
    }

    // Atomic-ish replace: try rename; if the target exists on platforms where rename doesn't replace,
    // remove and retry (best-effort cross-platform).
    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Attempt best-effort replace
            let _ = fs::remove_file(path);
            fs::rename(&tmp_path, path).map_err(IoError::from).or_else(|_| {
                // Final fallback: copy+remove (not strictly atomic but prevents dangling temp files)
                let mut f = File::create(path).map_err(IoError::from)?;
                f.write_all(&bytes).map_err(IoError::from)?;
                let _ = fs::remove_file(&tmp_path);
                Err(IoError::from(e))
            })?;
            Ok(())
        }
    }
}

/// Recursively sort all JSON object keys using BTreeMap; arrays/scalars untouched.
fn canonicalize_value(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            // First canonicalize children
            let mut bt: BTreeMap<String, Value> = BTreeMap::new();
            for (k, v) in map {
                bt.insert(k, canonicalize_value(v));
            }
            // Rebuild in sorted order
            let mut out: Map<String, Value> = Map::with_capacity(bt.len());
            for (k, v) in bt {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        Value::Array(xs) => {
            // Preserve array order (caller is responsible for deterministic ordering upstream)
            Value::Array(xs.into_iter().map(canonicalize_value).collect())
        }
        other => other,
    }
}

fn make_tmp_path(path: &Path) -> PathBuf {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if !ext.is_empty() => {
            let mut s = ext.to_string();
            s.push_str(".tmp");
            let mut p = path.to_path_buf();
            p.set_extension(s);
            p
        }
        _ => {
            let mut p = path.to_path_buf();
            p.set_extension("tmp");
            p
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::str;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Demo {
        a: u32,
        m: std::collections::HashMap<String, u32>,
        v: Vec<u8>,
    }

    #[test]
    fn ordering_and_idempotence_compact() {
        let mut m1 = std::collections::HashMap::new();
        m1.insert("z".into(), 1);
        m1.insert("a".into(), 2);

        let d1 = Demo { a: 1, m: m1, v: vec![3, 2, 1] };
        let b1 = to_canonical_bytes(&d1).unwrap();

        // Reparse & reserialize → identical
        let parsed: Value = sj::from_slice(&b1).unwrap();
        let b2 = to_canonical_bytes(&parsed).unwrap();
        assert_eq!(b1, b2);

        // Keys sorted
        let s = str::from_utf8(&b1).unwrap();
        assert!(s.find("\"a\":1").is_some());
        assert!(s.find("\"m\":{").is_some());
        assert!(s.find("\"a\":2").is_some()); // inside "m", "a" comes before "z"
    }

    #[test]
    fn pretty_has_only_whitespace_differences() {
        let mut m1 = std::collections::BTreeMap::new();
        m1.insert("b".to_string(), 1u32);
        m1.insert("a".to_string(), 2u32);
        let v = serde_json::json!({ "k": m1, "arr": [3,2,1] });

        let c = to_canonical_bytes(&v).unwrap();
        let p = to_canonical_pretty_bytes(&v).unwrap();

        // Parse both; semantic equality
        let vc: Value = sj::from_slice(&c).unwrap();
        let vp: Value = sj::from_slice(&p).unwrap();
        assert_eq!(vc, vp);

        // Pretty contains LF
        assert!(str::from_utf8(&p).unwrap().contains('\n'));
    }

    #[test]
    fn arrays_untouched() {
        let v = serde_json::json!({ "x": [ {"b":1,"a":2}, {"d":4,"c":3} ]});
        let out = to_canonical_bytes(&v).unwrap();
        let s = str::from_utf8(&out).unwrap();
        // Inner objects have sorted keys, array order preserved
        assert!(s.contains(r#"{"a":2,"b":1}"#));
        assert!(s.contains(r#"{"c":3,"d":4}"#));
        assert!(s.find(r#"{"a":2,"b":1}"#) < s.find(r#"{"c":3,"d":4}"#));
    }
}
