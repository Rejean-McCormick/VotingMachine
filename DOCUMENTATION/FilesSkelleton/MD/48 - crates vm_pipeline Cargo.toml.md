Here’s a **reference-aligned skeleton sheet** for the pipeline crate manifest. It keeps deps minimal (no JSON/FS/RNG here beyond what vm\_io/vm\_core provide), pins layering, and avoids stray defaults.

```toml
Pre-Coding Essentials (Component: crates/vm_pipeline/Cargo.toml, Version FormulaID VM-ENGINE v0) — 48/89

1) Goal & Success
Goal: Declare the pipeline crate that orchestrates the fixed state machine (load → validate → tabulate → allocate → aggregate → gates → frontier → ties → label → build result → build run record).
Success: Builds offline and deterministically; links only to vm_core (types/math/RNG), vm_io (I/O, canonical JSON, schemas, hashing), and vm_algo (tab/alloc/gates/frontier helpers). No UI/network deps.

2) Scope
In scope: package metadata; [lib] target; dependency wiring; optional feature flags to compile out frontier/MMP surfaces.
Out of scope: algorithm implementations (vm_algo), schema/json (vm_io), reporting/UI.

3) Inputs → Outputs
Inputs: Workspace toolchain and the three internal crates.
Outputs: rlib exposing pipeline entry points used by CLI/app to produce Result (RES:…) and RunRecord (RUN:…) artifacts (and optional FrontierMap (FR:…)).

4) Entities/Tables (minimal)
(N/A for manifest.)

5) Variables (only ones used here)
None at manifest level.

6) Functions
(Manifest only; pipeline functions live in src/ and mirror Doc-5 steps.)

7) Algorithm Outline (manifest structure)
[package]
name        = "vm_pipeline"
version     = "0.1.0"
edition     = "2021"
rust-version = "1.74"        # keep in sync with workspace toolchain
license     = "Apache-2.0 OR MIT"
description = "Deterministic orchestration of the VM engine pipeline (no I/O/UI)."
repository  = ""              # optional
categories  = ["algorithms", "no-std"]  # optional (still uses std by default)

[lib]
name       = "vm_pipeline"
path       = "src/lib.rs"
crate-type = ["rlib"]

[features]
default  = ["std", "frontier", "mmp"]
std      = []                 # allow building without std later if code supports it
frontier = []                 # compiles frontier mapping step
mmp      = []                 # compiles MMP helpers

[dependencies]
vm_core = { path = "../vm_core" }        # types, IDs, rounding, RNG wrapper (ChaCha), variables
vm_io   = { path = "../vm_io" }          # canonical JSON, schema validation, loaders, hashing
vm_algo = { path = "../vm_algo" }        # tabulation/allocation/gates/frontier compute

# No serde/serde_json/sha2/rand here; keep layering clean.

[dev-dependencies]
# Keep light; integration tests can use tempfile/assert_json_diff via vm_io when needed.
tempfile = "3"                          # optional for fixture temp dirs

# No build.rs; no networked or platform-specific deps.

8) State Flow (very short)
This crate exposes the pipeline API; runtime follows the fixed order in Doc-5 and emits Result/RunRecord (FrontierMap optional). Inputs are exactly: registry + ballot_tally + parameter_set; raw ballots are non-normative.

9) Determinism & Numeric Rules
Determinism inherited from vm_core (ordering/rounding/RNG) and vm_io (canonical JSON + SHA-256). No float deps; no OS entropy; RNG used only via vm_core::rng when tie policy demands it.

10) Edge Cases & Failure Policy
Avoid accidental JSON/FS deps here (vm_io owns them). Do not pull network features. Feature flags must not change public types—only compile scope for frontier/MMP.

11) Test Checklist (must pass)
- cargo build --locked -p vm_pipeline (defaults) succeeds on supported OS/arch.
- cargo check --no-default-features --features "std" builds (frontier/mmp off).
- Integration tests (in this crate) orchestrate the full state machine and produce Result + RunRecord with fields aligned to schemas/result.schema.json & run_record.schema.json; FrontierMap present only when frontier feature on.
```
