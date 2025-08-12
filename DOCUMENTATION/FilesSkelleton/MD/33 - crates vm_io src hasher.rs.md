<!-- Converted from: 33 - crates vm_io src hasher.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.413430Z -->

```
Pre-Coding Essentials (Component: crates/vm_io/src/hasher.rs, Version/FormulaID: VM-ENGINE v0) — 33/89
1) Goal & Success
Goal: Provide canonical hashing utilities (SHA-256) used for digests and IDs, fed strictly with canonical JSON bytes.
Success: Same struct → same canonical bytes → same hex digest across OS/arch; supports streaming file hashes; exposes helpers for Result/RunRecord IDs and Formula ID (FID) from the Normative Manifest.
2) Scope
In scope: SHA-256 over bytes/reader, helpers to hash canonical JSON values, FID builder (hash over NM fields only), lowercase hex encoding.
Out of scope: JSON canonicalization itself (lives in canonical_json.rs), file I/O policy, report formatting.
3) Inputs → Outputs
Inputs: &[u8], Read streams, or serde::Serialize values that have been canonicalized.
Outputs: Lowercase hex digests (String), plus tiny wrappers for ID strings (RES:…, RUN:…, FR:… computed elsewhere from digests).
4) Entities/Fields (minimal)
5) Variables (module knobs)
6) Functions (signatures only)
rust
CopyEdit
use std::io::Read;

/// Hash raw bytes → lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String;

/// Hash a reader (stream) → lowercase hex (no mmap).
pub fn sha256_stream<R: Read>(mut r: R) -> Result<String, IoError>;

/// Canonicalize then hash a serializable value.
pub fn sha256_of_canonical<T: serde::Serialize>(value: &T) -> Result<String, IoError>;

/// Build a short hash token (e.g., first 12–16 hex chars) for IDs.
pub fn short_hex(full_hex: &str, len: usize) -> Result<String, IoError>;

/// Compute Formula ID from a Normative Manifest (NM) value:
/// *drop* origin fields; sort keys; hash canonical bytes.
pub fn formula_id_from_nm(nm: &serde_json::Value) -> Result<String, IoError>;

/// Convenience: digest a file at path.
pub fn sha256_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, IoError>;

7) Algorithm Outline (implementation plan)
sha256_hex: sha2::Sha256::digest(bytes) → lowercase hex via hex::encode.
sha256_stream: read in hash.buf_size chunks; update hasher incrementally; return hex.
sha256_of_canonical: call canonical_json::to_canonical_bytes(value) then sha256_hex.
short_hex: validate len>0 && len<=full.len() and all-hex; return Ok(full[..len].to_string()).
formula_id_from_nm:
Accept an NM as serde_json::Value.
Strip non-normative fields (origin, timestamps) recursively as specified; keep only the four normative blocks (schema_version, variables, constants, compat).
Re-serialize with canonical JSON; sha256_hex over those bytes.
sha256_file: open in binary; call sha256_stream.
8) State Flow
Writers/Builders: canonical_json::* → sha256_of_canonical → hex digest.
Manifests/Run: manifest.verify_digests and pipeline ID builders consume sha256_* outputs to compare/compose IDs.
9) Determinism & Numeric Rules
Determinism comes from hashing canonical bytes only; hex always lowercase; no locale/time/path influence.
No floats processed here; any floats should already be rejected by canonicalization layer when hashing artifacts.
10) Edge Cases & Failure Policy
Empty input bytes → valid hash of empty string (document value).
Non-hex passed to short_hex → IoError::Hash("non-hex input").
NM missing required normative blocks in formula_id_from_nm → IoError::Hash("incomplete NM").
Reader errors bubble as IoError::Hash with source detail.
11) Test Checklist (must pass)
Byte equality: same canonical struct twice → identical digest.
Stream vs bytes: sha256_stream(File) equals sha256_hex(read_all_bytes).
Short hex: length guard and hex validation; short_hex(full, 16) stable.
NM FID: adding an origin block does not change FID; changing a variable default does.
Cross-platform: digests equal on Win/macOS/Linux for same canonical input.
```
