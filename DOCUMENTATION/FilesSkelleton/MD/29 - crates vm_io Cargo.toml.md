
```
Pre-Coding Essentials (Component: crates/vm_io/Cargo.toml, Version/FormulaID: VM-ENGINE v0) — 29/89

1) Goal & Success
Goal: Manifest for vm_io (I/O + canonical JSON + schema validation + hashing + loaders).
Success: Builds as rlib; depends on vm_core; exposes parsing/validation/canonicalization/hashing; no UI/CLI deps; minimal, feature-gated dependencies.

2) Scope
In scope: package metadata, edition/rust-version, [lib], features, deps for JSON parse/write, JSON Schema (2020-12) validation, hashing, UTF-8 paths, error enums.
Out of scope: algorithms/pipeline/report, RNG, CLI.

3) Inputs → Outputs
Inputs: Workspace toolchain; schemas/fixtures at runtime.
Outputs: Library used by vm_pipeline/vm_cli: read/validate/canonicalize/hash artifacts (registry/tally/params/manifest/result/run_record/frontier_map).

4) Entities/Tables (minimal)
(Manifest-only; code defines loaders/validators elsewhere.)

5) Variables (build/features)
Features are additive and off by default where possible:
- `std` (default) — required for fs/I/O.
- `serde` — JSON parse/write (serde + serde_json).
- `schemaval` — JSON Schema (Draft 2020-12) runtime validation.
- `hash` — SHA-256 hashing utilities.
- `path_utf8` — UTF-8 paths via camino.

6) Functions
(Manifest only.)

7) Manifest Outline (structure)
[package] name vm_io, version 0.1.0, edition 2021, rust-version pinned; dual license.
[lib] rlib, path "src/lib.rs".
[features]
default = ["std","serde","schemaval","hash","path_utf8"]
std = []
serde = ["dep:serde","dep:serde_json"]
schemaval = ["dep:jsonschema"]
hash = ["dep:sha2","dep:digest"]
path_utf8 = ["dep:camino"]
[dependencies] — all with `default-features = false` where applicable; no network/UI deps.
[dev-dependencies] — minimal for tests (tempfile, assert_json_diff).
No build.rs.

8) State Flow
vm_pipeline/vm_cli link vm_io to load → validate (schema) → canonicalize (sorted keys, LF) → hash (sha256).

9) Determinism & Numeric Rules
- Canonical JSON writer enforces **sorted keys** and **LF**; arrays are ordered upstream (vm_core determinism helpers).
- Hashing: **sha256** over canonical bytes only.
- No float math beyond parsing JSON numbers; counts remain integers per schemas.

10) Edge Cases & Failure Policy
- Build without validator: `--features "std,serde,hash"` compiles.
- Keep transitive defaults off (audit `cargo tree -e features`).
- No platform-specific deps beyond optional camino.

11) Test Checklist
- `cargo check -p vm_io` (defaults) OK.
- `cargo check -p vm_io --no-default-features --features "std,serde,hash"` OK.
- Link check: vm_pipeline compiles against vm_io loaders/validator/hash.
- No unwanted default features pulled in.
```

**Canonical `Cargo.toml` snippet (drop-in):**

```toml
[package]
name = "vm_io"
version = "0.1.0"
edition = "2021"
rust-version = "1.77"
license = "Apache-2.0 OR MIT"
description = "I/O, canonical JSON, JSON Schema validation (2020-12), and SHA-256 hashing for the VM engine."
# repository = "..."; readme = "README.md"

[lib]
name = "vm_io"
path = "src/lib.rs"
crate-type = ["rlib"]

[features]
default = ["std", "serde", "schemaval", "hash", "path_utf8"]
std = []
serde = ["dep:serde", "dep:serde_json"]
schemaval = ["dep:jsonschema"]
hash = ["dep:sha2", "dep:digest"]
path_utf8 = ["dep:camino"]

[dependencies]
vm_core     = { path = "../vm_core" }

serde       = { version = "1", features = ["derive"], optional = true, default-features = false }
serde_json  = { version = "1", optional = true }                      # std only; used for parse/write

jsonschema  = { version = "0.17", optional = true, default-features = false, features = ["draft2020-12"] }

sha2        = { version = "0.10", optional = true, default-features = false }
digest      = { version = "0.10", optional = true, default-features = false }
hex         = { version = "0.4",  default-features = false }

camino      = { version = "1", optional = true, default-features = false }

thiserror   = { version = "1", default-features = false }

[dev-dependencies]
assert_json_diff = "2"
tempfile         = "3"
```


