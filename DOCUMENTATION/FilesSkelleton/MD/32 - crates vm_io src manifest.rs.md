
````
Pre-Coding Essentials (Component: crates/vm_io/src/manifest.rs, Version FormulaID VM-ENGINE v0) — 32/89

1) Goal & Success
Goal: Parse, validate, and resolve the **run manifest** into concrete local file paths and expectations for a deterministic offline run.
Success: Given `manifest.json`, return a typed `Manifest` and `ResolvedPaths` with **ballot_tally_path required**, **no URLs**, paths resolved against the manifest directory, and optional `expect`/`digests` verified with precise, pointered errors.

2) Scope
In scope: JSON parse → schema check → typed struct; relative-path resolution; **no ballots path**; optional `expect{formula_id, engine_version}`; optional `digests` verification; precise error mapping.
Out of scope: Reading target files (loader.rs), hashing bytes (hasher.rs), canonical JSON writing (canonical_json.rs).

3) Inputs → Outputs
Inputs: Path to `manifest.json`.
Outputs:
- `Manifest` (typed view).
- `ResolvedPaths` (normalized absolute/base-relative paths for `registry`, `params`, **`ballot_tally`**, optional `adjacency`).
- Optional checks: `expect` and `digests` verification.

4) Entities/Types
- Uses `IoError` from `vm_io`.

5) Module knobs
- `reject_urls: bool` (default true).
- `allow_parent_traversal: bool` (default true; set false to confine within base dir).
- `max_bytes`, `max_depth` (DoS guards).

6) Functions (signatures only)
```rust
// ---------- Public API ----------
#[derive(Debug, Clone)]
pub struct Manifest {
    pub id: String,                 // "MAN:…"
    pub reg_path: String,
    pub params_path: String,
    pub ballot_tally_path: String,  // REQUIRED (normative pipeline)
    pub adjacency_path: Option<String>,
    pub expect: Option<Expect>,
    pub digests: Option<std::collections::BTreeMap<String, DigestEntry>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Expect {
    pub formula_id: Option<String>,     // expected FormulaID (hex)
    pub engine_version: Option<String>, // expected engine version string
}

#[derive(Debug, Clone)]
pub struct DigestEntry { pub sha256: String } // 64-hex, lowercase preferred

#[derive(Debug, Clone)]
pub struct ResolvedPaths {
    pub base_dir: camino::Utf8PathBuf,
    pub reg:      camino::Utf8PathBuf,
    pub params:   camino::Utf8PathBuf,
    pub tally:    camino::Utf8PathBuf,
    pub adjacency: Option<camino::Utf8PathBuf>,
}

// ---------- Top-level ----------
pub fn load_manifest<P: AsRef<std::path::Path>>(path: P) -> Result<Manifest, IoError>;
pub fn validate_manifest(man: &Manifest) -> Result<(), IoError>;
pub fn resolve_paths<P: AsRef<std::path::Path>>(manifest_file: P, man: &Manifest)
    -> Result<ResolvedPaths, IoError>;
pub fn enforce_expectations(man: &Manifest, engine_version: &str, formula_id_hex: &str)
    -> Result<(), IoError>;
pub fn verify_digests(paths: &ResolvedPaths, digests: &std::collections::BTreeMap<String, DigestEntry>)
    -> Result<(), IoError>;
````

7. Algorithm Outline (implementation plan)

* **Read & parse**

  * Read file with `max_bytes` cap.
  * Parse to `serde_json::Value`; map parse errors to `IoError::Json { pointer: "/", msg }`.

* **Schema validation**

  * Validate against `schemas/manifest.schema.json` (Draft 2020-12).
  * On first violation: `IoError::Schema { pointer, msg }`.

* **To typed `Manifest`**

  * Deserialize to `Manifest`.
  * **Reject any legacy `ballots_path` if present in raw JSON** (defensive check; schema should already forbid).
  * Quick checks:

    * `ballot_tally_path` is **non-empty**.
    * All path strings must **not** start with `http://` or `https://` when `reject_urls`.

* **Resolve paths**

  * `base_dir = manifest_file.parent()` (UTF-8 via camino).
  * Join & normalize each path.
  * If `allow_parent_traversal == false`, reject any resolved path that escapes `base_dir` post-normalization.
  * Existence check is optional here (loader does the actual reads).

* **Expectations (optional)**

  * If `expect.formula_id` present, compare case-insensitively to `formula_id_hex` (must be 64-hex).
  * If `expect.engine_version` present, compare exact string.
  * Mismatch → `IoError::Expect("formula_id mismatch")` / `"engine_version mismatch"`.

* **Digests (optional)**

  * Validate each `sha256` as **64-hex** (lowercase recommended).
  * Only allow keys that are one of: `reg_path`, `params_path`, `ballot_tally_path`, `adjacency_path` (if present). Unknown keys → `IoError::Manifest("unknown digest path: …")`.
  * Compute SHA-256 (canonical bytes if loading typed later; here use file bytes) and compare (case-insensitive).
  * Mismatch → `IoError::Manifest("digest mismatch for …")`.

* **Return**

  * On success, return `Manifest` and/or `ResolvedPaths`.

8. State Flow
   `vm_cli` / `vm_pipeline`:

9. `load_manifest` → `validate_manifest`.

10. `resolve_paths` (optionally `enforce_expectations` after computing actual engine/formula).

11. Pass `ResolvedPaths` to loader to read/validate Registry, Params, **BallotTally**, optional Adjacency.

12. Determinism & Numeric Rules

* Determinism via **local files**, **no URLs**, and optional **digest checks**.
* No numeric processing here.

10. Edge Cases & Failure Policy

* Missing `ballot_tally_path` → error.
* Any `ballots_path` present (legacy) → error.
* Any path begins with `http(s)://` → error.
* Normalized path escapes base and traversal not allowed → error.
* `digests` contains non-hex or path not in manifest → error.
* `expect` provided but caller doesn’t pass actual data to `enforce_expectations` → caller contract (documented).

11. Test Checklist

* **Happy (tally)**: `reg_path`, `params_path`, `ballot_tally_path` → parse/validate/resolve succeeds.
* **Legacy ballots present**: `validate_manifest` fails clearly.
* **URL rejection**: any `http(s)://…` in paths → fail.
* **Traversal**: with traversal off, `../outside.json` rejects; with it on, resolve succeeds.
* **Expect**: mismatched formula\_id/engine\_version → `IoError::Expect`.
* **Digests**: correct hex & content → pass; non-hex or mismatch → fail; unknown digest key → fail.
* **Cross-platform**: same manifest resolves to identical normalized UTF-8 paths on Windows/macOS/Linux (relative to base).

```

```
