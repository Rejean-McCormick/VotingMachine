//! Canonical JSON utilities (vm_io)
//! - Objects: keys sorted lexicographically (UTF-8 codepoint order)
//! - Arrays: order preserved (caller is responsible for stable ordering)
//! - Output: compact (no extra spaces, no trailing newline)
//! - Atomic write: temp file in same dir + fsync(temp) + rename; fsync(dir) on Unix
//! - Fallback: if rename fails (e.g., cross-device), write directly to target,
//!   fsync(target), then remove temp, fsync(dir).

#![allow(clippy::needless_borrow)]

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde_json::Value;

#[cfg(feature = "serde")]
/// Convert a serde_json `Value` to canonical JSON bytes (compact, no trailing newline).
pub fn to_canonical_json_bytes(v: &Value) -> Vec<u8> {
    let mut out = Vec::with_capacity(1024);
    write_canonical_value(v, &mut out);
    out
}

#[cfg(feature = "serde")]
/// Write canonical JSON to `path` atomically (with safe cross-device fallback).
pub fn write_canonical_file(path: &Path, v: &Value) -> io::Result<()> {
    let bytes = to_canonical_json_bytes(v);

    // Ensure parent directory exists.
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
    fs::create_dir_all(parent)?;

    // Create a unique temp next to the destination (same directory).
    let tmp = make_unique_tmp_path(path);
    let mut tf = OpenOptions::new()
        .write(true)
        .create_new(true) // avoid clobbering another writer's temp
        .open(&tmp)?;

    // Write and fsync the temp file.
    tf.write_all(&bytes)?;
    tf.sync_all()?;
    drop(tf);

    // Try atomic rename first.
    match fs::rename(&tmp, path) {
        Ok(()) => {
            // On Unix, also fsync the directory to persist the rename.
            let _ = fsync_dir(parent);
            Ok(())
        }
        Err(_e) => {
            // Fallback: write directly to the target (handles cross-device cases).
            let res: io::Result<()> = (|| {
                let mut f = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)?;
                f.write_all(&bytes)?;
                f.sync_all()?;
                Ok(())
            })();

            if let Err(err) = res {
                let _ = fs::remove_file(&tmp); // best-effort cleanup on error
                return Err(err);
            }

            // Best-effort cleanup of the temp file on success.
            let _ = fs::remove_file(&tmp);

            // On Unix, fsync the directory as well.
            let _ = fsync_dir(parent);
            Ok(())
        }
    }
}

#[cfg(not(feature = "serde"))]
#[inline]
pub fn write_canonical_file(_path: &Path, _v: &()) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "serde feature disabled for vm_io:canonical_json",
    ))
}

#[cfg(feature = "serde")]
fn write_canonical_value(v: &Value, out: &mut Vec<u8>) {
    match v {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(b) => {
            if *b {
                out.extend_from_slice(b"true");
            } else {
                out.extend_from_slice(b"false");
            }
        }
        Value::Number(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Value::String(s) => {
            // Use serde_json to produce a correctly escaped JSON string literal.
            let quoted = serde_json::to_string(s).expect("string serialization cannot fail");
            out.extend_from_slice(quoted.as_bytes());
        }
        Value::Array(arr) => {
            out.push(b'[');
            let mut first = true;
            for elem in arr {
                if !first {
                    out.push(b',');
                }
                first = false;
                write_canonical_value(elem, out);
            }
            out.push(b']');
        }
        Value::Object(map) => {
            out.push(b'{');
            // Collect & sort keys lexicographically.
            let mut keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
            keys.sort_unstable();
            let mut first = true;
            for k in keys {
                if !first {
                    out.push(b',');
                }
                first = false;
                // Key
                let quoted_key = serde_json::to_string(k).expect("key serialization cannot fail");
                out.extend_from_slice(quoted_key.as_bytes());
                out.push(b':');
                // Value
                write_canonical_value(&map[k], out);
            }
            out.push(b'}');
        }
    }
}

/// Create a unique temp path next to `target`: "<filename>.<pid>.<counter>.tmp"
fn make_unique_tmp_path(target: &Path) -> PathBuf {
    use std::ffi::OsString;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let pid = std::process::id();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);

    let fname = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    let tmp_name: OsString = OsString::from(format!("{fname}.{pid}.{n}.tmp"));

    match target.parent() {
        Some(dir) => dir.join(tmp_name),
        None => PathBuf::from(tmp_name),
    }
}

/// Fsync the directory containing the file (Unix only). No-op on other platforms.
#[cfg(unix)]
fn fsync_dir(dir: &Path) -> io::Result<()> {
    // Portable approach: open the directory for reading and sync it.
    let df = OpenOptions::new().read(true).open(dir)?;
    df.sync_all()
}

#[cfg(not(unix))]
#[inline]
fn fsync_dir(_dir: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn objects_are_sorted_arrays_preserved() {
        let v = json!({
            "b": 1,
            "a": { "y": 1, "x": 2 },
            "arr": [ {"k":2,"j":1}, 3, "z" ]
        });
        let s = String::from_utf8(to_canonical_json_bytes(&v)).unwrap();
        assert_eq!(
            s,
            r#"{"a":{"x":2,"y":1},"arr":[{"j":1,"k":2},3,"z"],"b":1}"#
        );
    }

    #[test]
    fn no_trailing_newline() {
        let v = json!({"a":1});
        let bytes = to_canonical_json_bytes(&v);
        assert!(!bytes.ends_with(b"\n"), "must not end with newline");
    }
}
