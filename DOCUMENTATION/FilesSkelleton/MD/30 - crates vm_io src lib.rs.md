
````
Pre-Coding Essentials (Component: crates/vm_io/src/lib.rs, Version FormulaID VM-ENGINE v0) — 30/89

1) Goal & Success
Goal: Public surface for vm_io — canonical JSON I/O, JSON Schema validation (Draft 2020-12), local path resolution, SHA-256 hashing, and high-level loaders that return typed vm_core structs.
Success: vm_pipeline/vm_cli can load **manifest → registry → params → ballot_tally (optional adjacency)**, validate against schemas, produce canonical bytes + SHA-256, and surface precise, pointered errors. No network, no UI.

2) Scope
In scope: module exports, error types, canonical writer, schema validator wiring, path resolution, digest helpers, high-level loaders/bundles.
Out of scope: algorithms/pipeline logic, RNG, report rendering, FID math (done elsewhere).

3) Inputs → Outputs
Inputs: Local JSON files (manifest.json, division_registry.json, ballot_tally.json, parameter_set.json, optional adjacency.json).
Outputs:
- Typed values from vm_core (DivisionRegistry, Params, …)
- LoadedContext (bundle for the pipeline)
- Canonical JSON bytes (sorted keys, LF) + SHA-256 digests
- Failures as structured errors with JSON Pointers

4) Public API (signatures only)

```rust
// Re-exports (narrow surface)
pub use vm_core::{ /* ids/tokens/entities/variables */ variables::Params, determinism::*, rounding::*, rng::* };

// ---------- Error model ----------
#[derive(thiserror::Error, Debug)]
pub enum IoError {
    #[error("read error: {0}")]                        Read(std::io::Error),
    #[error("write error: {0}")]                       Write(std::io::Error),
    #[error("json parse error at {pointer}: {msg}")]   Json { pointer: String, msg: String },
    #[error("schema validation failed at {pointer}: {msg}")]
                                                      Schema { pointer: String, msg: String },
    #[error("manifest violation: {0}")]                Manifest(String),
    #[error("expectation mismatch: {0}")]              Expect(String),
    #[error("canonicalization: {0}")]                  Canon(String),
    #[error("hashing: {0}")]                           Hash(String),
    #[error("path: {0}")]                              Path(String),
    #[error("limit: {0}")]                             Limit(&'static str),
}

// ---------- Canonical JSON (sorted keys, LF) ----------
pub mod canonical_json {
    use super::IoError;
    pub fn to_canonical_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, IoError>;
    pub fn write_canonical_file<T: serde::Serialize, P: AsRef<std::path::Path>>(value: &T, path: P) -> Result<(), IoError>;
}

// ---------- SHA-256 digests ----------
pub mod hasher {
    use super::IoError;
    pub fn sha256_hex(bytes: &[u8]) -> String;
    pub fn sha256_file<P: AsRef<std::path::Path>>(path: P) -> Result<String, IoError>;
    pub fn sha256_canonical<T: serde::Serialize>(value: &T) -> Result<String, IoError>;
}

// ---------- Manifest & path resolution ----------
pub mod manifest {
    use super::IoError;

    #[derive(Debug, Clone)]
    pub struct Manifest {
        pub id: String,                 // "MAN:…"
        pub reg_path: String,
        pub params_path: String,
        pub ballot_tally_path: String,  // REQUIRED (no ballots_path in normative pipeline)
        pub adjacency_path: Option<String>,
        pub expect: Option<Expect>,     // optional sanity locks
        pub digests: std::collections::BTreeMap<String, String>, // optional: { path -> sha256 hex }
        pub notes: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct Expect {
        pub formula_id: Option<String>,     // expected FID
        pub engine_version: Option<String>, // expected engine version
    }

    #[derive(Debug, Clone)]
    pub struct ResolvedPaths {
        pub base_dir: std::path::PathBuf,
        pub reg:      std::path::PathBuf,
        pub params:   std::path::PathBuf,
        pub tally:    std::path::PathBuf,
        pub adjacency: Option<std::path::PathBuf>,
    }

    pub fn load_manifest<P: AsRef<std::path::Path>>(path: P) -> Result<Manifest, IoError>;
    pub fn resolve_paths(base: &std::path::Path, man: &Manifest) -> Result<ResolvedPaths, IoError>;
    pub fn verify_expectations(man: &Manifest, actual_formula_id: Option<&str>, actual_engine_version: Option<&str>) -> Result<(), IoError>;
}

// ---------- JSON Schema validation ----------
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
    pub fn validate_value(kind: SchemaKind, value: &serde_json::Value) -> Result<(), IoError>;
}

// ---------- High-level loaders (typed) ----------
pub mod loader {
    use super::{IoError};
    use vm_core::{entities::*, variables::Params};

    #[derive(Debug)]
    pub struct LoadedContext {
        pub registry: DivisionRegistry,
        pub params:   Params,
        pub tally:    UnitTallies,                 // aggregated tallies only (normative)
        pub adjacency_inline: Option<Vec<Adjacency>>, // if delivered separately
        pub digests:  InputDigests,                // sha256 hex for inputs
    }

    #[derive(Debug, Default)]
    pub struct InputDigests {
        pub division_registry_sha256: String,
        pub ballot_tally_sha256:      String,
        pub parameter_set_sha256:     String,
        pub adjacency_sha256:         Option<String>,
    }

    // Primary entry
    pub fn load_all_from_manifest<P: AsRef<std::path::Path>>(path: P) -> Result<LoadedContext, IoError>;

    // Targeted loaders
    pub fn load_registry<P: AsRef<std::path::Path>>(path: P) -> Result<DivisionRegistry, IoError>;
    pub fn load_params<P: AsRef<std::path::Path>>(path: P) -> Result<Params, IoError>;
    pub fn load_ballot_tally<P: AsRef<std::path::Path>>(path: P) -> Result<UnitTallies, IoError>;
    pub fn load_adjacency<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Adjacency>, IoError>;

    // Canonical + hash helpers for already-typed artifacts
    pub fn canonical_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, IoError>;
    pub fn sha256_hex_of<T: serde::Serialize>(value: &T) -> Result<String, IoError>;
}
````

5. Module layout & behavior (authoring outline)

* **canonical\_json**

  * Serialize via `serde_json::Serializer` with **stable key order** (convert maps to `BTreeMap` / custom walker).
  * Enforce **LF** line endings, UTF-8, and no trailing spaces.
  * No float coercion; numbers are emitted as provided (schemas keep counts as integers; shares are JSON numbers where applicable).

* **hasher**

  * `sha256_hex(bytes)` on canonical bytes.
  * `sha256_file(path)`: stream in chunks; returns lowercase 64-hex.
  * `sha256_canonical(value)`: canonicalize → hash; used by run-record building.

* **manifest**

  * Parse JSON; validate with `SchemaKind::Manifest`; **reject URLs** (`http://` or `https://`).
  * **Require `ballot_tally_path`** (no ballots\_path in the normative loader).
  * Resolve relative paths against manifest directory; return `ResolvedPaths`.
  * If `expect` provided, check `formula_id`/`engine_version` early via `verify_expectations`.

* **schema**

  * Bundle schemas at build time (e.g., `include_str!`) or ship alongside; validate value against **Draft 2020-12** schemas.
  * Map first violation to `IoError::Schema { pointer, msg }` (JSON Pointer to failing path).

* **loader**

  * `load_all_from_manifest`:

    1. read/validate manifest → resolve paths;
    2. load+validate registry, params, **ballot\_tally** (and optional adjacency);
    3. compute digests for inputs (sha256 of canonical bytes);
    4. **normalize ordering** (units by `unit_id`; options by `(order_index, option_id)`).
    5. return `LoadedContext`.
  * Targeted `load_*` mirror the same (parse → schema-validate → return typed).
  * DOS guards: enforce max file size & max JSON depth; emit `IoError::Limit("io.max_bytes")` / `"io.max_depth"`.

6. Determinism & Numeric Rules

* Canonical JSON: **sorted keys + LF**; arrays come pre-sorted by upstream determinism helpers.
* SHA-256 over canonical bytes only.
* No floats introduced by vm\_io; where the spec stores shares as numbers, they are carried as JSON numbers unmodified.

7. Path & safety policy

* **Local filesystem only**; **reject URLs** outright.
* Relative paths are allowed; resolve against manifest directory; optional policy to reject escaping a workspace root.
* No network, no environment-dependent behavior.

8. Edge Cases & Failure Policy

* **Manifest** missing `ballot_tally_path` → `IoError::Manifest`.
* Both ballots and tally paths present (legacy) → `IoError::Manifest` (explicitly disallowed).
* Schema disabled (feature-gated build): parsing succeeds; caller is responsible for downstream validation (documented).
* Invalid hex in `digests{path→sha256}` or mismatch vs computed → `IoError::Hash`.
* Adjacent files present but empty/oversized → `IoError::Limit`.

9. Test Checklist

* Canonical writer: same structure → identical bytes; LF enforced; keys sorted.
* Hashing: `sha256_file` of file equals `sha256_canonical` of parsed value.
* Manifest: URL paths rejected; **requires ballot\_tally\_path**; expectations mismatch → `Expect`.
* Schema: invalid registry/params/tally/manifest produce JSON-Pointered `Schema` errors.
* Loader (happy path): registry + **tally** + params load; adjacency optional; ordering normalized deterministically.
* DOS guards: files > limit and depth > limit fail fast with `Limit`.
* Feature matrix: build with/without `schemaval` and `hash` features per Cargo manifest.

Notes

* Keep this crate **std-bound**; `--no-default-features` without `std` is not supported.
* Do not expose any network or platform-specific behavior.
* Keep error messages short and stable for the Annex-B test pack comparisons.

```

```
