
````
Pre-Coding Essentials (Component: crates/vm_io/src/hasher.rs, Version FormulaID VM-ENGINE v0) — 33/89

1) Goal & Success
Goal: SHA-256 hashing utilities over canonical bytes for digests and IDs; helpers to compute the **Normative Manifest** digest (nm_digest) and **Formula ID (FID)**, plus prefixed ID builders (RES:/RUN:/FR:).
Success: Same struct → same canonical bytes → same lowercase 64-hex across OS/arch; streaming file hashing supported; ID helpers emit **full 64-hex** IDs consistent with vm_core `ids`.

2) Scope
In scope: SHA-256 over bytes/reader; hash of canonicalized values; nm_digest & FID builders; lowercase hex; helpers to format RES/RUN/FR IDs.
Out of scope: canonicalization mechanics (live in `canonical_json`); any network I/O; report formatting; RNG.

3) Inputs → Outputs
Inputs: `&[u8]`, `Read` streams, or `serde::Serialize` values (canonicalized in this module via `canonical_json`).
Outputs: lowercase 64-hex digests (`String`) and ID strings:
- `RES:<hex64>` (Result)
- `RUN:<RFC3339Z>-<hex64>` (RunRecord)
- `FR:<hex64>` (FrontierMap)
- `nm_digest` (hex64) and `formula_id` (hex64)

4) Entities/Fields (minimal)
- Depends on `vm_io::canonical_json` for canonical bytes.
- Uses `IoError::{Hash, Canon, Read/Write}` for error mapping.

5) Variables (module knobs)
- `SHORT_ID_LEN` (if you later want a preview token; **IDs here use full 64-hex**).
- Chunk size for streaming (e.g., 64 KiB).

6) Functions (signatures only)
```rust
use std::io::Read;

// Core digests
pub fn sha256_hex(bytes: &[u8]) -> String;
pub fn sha256_stream<R: Read>(reader: &mut R) -> Result<String, IoError>;
pub fn sha256_of_canonical<T: serde::Serialize>(value: &T) -> Result<String, IoError>;

// Prefixed ID builders (full 64-hex)
pub fn res_id_from_canonical<T: serde::Serialize>(value: &T) -> Result<String, IoError>; // "RES:<hex64>"
pub fn fr_id_from_canonical<T: serde::Serialize>(value: &T) -> Result<String, IoError>;  // "FR:<hex64>"
pub fn run_id_from_bytes(timestamp_utc: &str, bytes: &[u8]) -> Result<String, IoError>;   // "RUN:<ts>-<hex64>"

// Hex helpers
pub fn is_hex64(s: &str) -> bool;
pub fn short_hex(full_hex: &str, len: usize) -> Result<String, IoError>; // utility (not used for IDs)

// Normative Manifest / Formula ID
/// Compute the SHA-256 over the **Normative Manifest** canonical bytes (nm_digest).
/// Caller must pass an NM value that already contains only normative fields per Annex A.
pub fn nm_digest_from_value(nm: &serde_json::Value) -> Result<String, IoError>;

/// Compute Formula ID (FID) from the **Normative Manifest**.
/// By convention FID == nm_digest (hex64). If a future version derives FID from a subset,
/// this function remains the single place to change that policy.
pub fn formula_id_from_nm(nm: &serde_json::Value) -> Result<String, IoError>;

// File convenience
pub fn sha256_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, IoError>;
````

7. Algorithm Outline (implementation plan)

* **sha256\_hex**: `sha2::Sha256::digest(bytes)` → `hex::encode` (lowercase).
* **sha256\_stream**: read in fixed chunks; update hasher; return lowercase hex; map IO errors to `IoError::Hash`.
* **sha256\_of\_canonical**: `canonical_json::to_canonical_bytes(value)` → `sha256_hex(&bytes)`.
* **ID builders**:

  * `res_id_from_canonical`: canonicalize `Result` struct → digest → format `"RES:{hex64}"`.
  * `fr_id_from_canonical`: canonicalize FrontierMap → digest → `"FR:{hex64}"`.
  * `run_id_from_bytes`: validate `timestamp_utc` as RFC3339Z (string check; leave strict parse to caller if needed), digest `bytes` → `"RUN:{ts}-{hex64}"`.
* **hex helpers**:

  * `is_hex64`: fast length & char class check (0–9, a–f).
  * `short_hex`: guard `1..=64`, ensure all-hex, slice prefix.
* **NM / FID**:

  * `nm_digest_from_value`: canonicalize the **already-filtered** NM `Value` (normative only: Included VM-VARs per Annex A; exclude 032–035, 052, 060–062; and any non-normative blocks), then hash.
  * `formula_id_from_nm`: currently `Ok(nm_digest_from_value(nm)?)`. Keep as a wrapper to centralize policy.
* **sha256\_file**: open in binary; stream via `sha256_stream`.

8. State Flow

* Writers/builders: `canonical_json` → `sha256_of_canonical` → hex.
* Pipeline:

  * compute `nm_digest` from NM → store in RunRecord; also compute `formula_id` (same as nm\_digest).
  * compute `RES:`/`FR:` from canonical artifacts after they’re built.
  * compute `RUN:` from timestamp + canonical bytes of “run input basis” (inputs + engine meta as defined by pipeline).

9. Determinism & Numeric Rules

* Canonical bytes ensure platform-independent hashing.
* Lowercase hex; no locale/time influence (timestamps supplied by caller).
* No floats handled here; any floats in artifacts are already in canonical JSON per spec (shares allowed as JSON numbers).

10. Edge Cases & Failure Policy

* Empty bytes: valid (document SHA-256 of empty input).
* `run_id_from_bytes`: reject non-RFC3339Z timestamps with `IoError::Hash("bad timestamp")` (string check).
* `short_hex`: non-hex input or len out of range → `IoError::Hash`.
* `nm_digest_from_value`: if caller supplies non-normative fields, digest still computed; policy is that **caller must pre-filter** NM. If strictness desired, add an optional validator elsewhere.

11. Test Checklist (must pass)

* Byte equality: same canonical value twice → identical digest.
* Stream vs bytes: `sha256_stream(File)` equals `sha256_file(path)` and equals `sha256_hex(all_bytes)`.
* Hex helpers: `is_hex64` true only for lowercase 64-hex; `short_hex` guards length & hex.
* IDs: `res_id_from_canonical`/`fr_id_from_canonical` produce **full 64-hex** IDs; `run_id_from_bytes` format matches `"RUN:<ts>-<hex64>"`.
* NM/FID: `formula_id_from_nm(nm)` equals `nm_digest_from_value(nm)`; changing any Included VM-VAR affects FID; changing any Excluded VM-VAR does **not** (because NM should not include them).
* Cross-platform: identical inputs yield identical hex on Windows/macOS/Linux.

```

```
