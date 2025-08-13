Here’s a **reference-aligned skeleton sheet** for **35 – crates/vm\_algo/Cargo.toml.md**. It keeps vm\_algo purely algorithmic (no I/O/JSON/UI), depends only on `vm_core`, and gates families via features without changing public types. RNG comes from `vm_core::rng`, rounding from `vm_core::rounding`.

```
Pre-Coding Essentials (Component: crates/vm_algo/Cargo.toml, Version FormulaID VM-ENGINE v0) — 35/89

1) Goal & Success
Goal: Manifest for vm_algo (tabulation, allocation, gates/frontier helpers) depending only on vm_core.
Success: Builds as rlib on all targets; no JSON/FS/UI deps; features toggle algorithm families; unit/property tests compile cleanly.

2) Scope
In scope: package metadata, edition/rust-version, [lib], feature flags for families (ranked, score, PR, MMP, gates, frontier), minimal deps.
Out of scope: CLI, schema/JSON, persistence, report/UI.

3) Inputs → Outputs
Inputs: Workspace toolchain; vm_core API.
Outputs: rlib consumed by vm_pipeline; modules like tabulation/*, allocation/*, mmp/*, gates_frontier/*.

4) Entities/Tables
(Manifest only.)

5) Build variables / features
- `std` (default): allow std usage internally.
- `tab_ranked`: IRV/Condorcet tabulation helpers.
- `tab_score`: score/approval helpers.
- `pr_methods`: divisor/largest-remainder seat allocation.
- `mmp`: mixed-member proportional corrections.
- `gates`: quorum/majority/double-majority checks.
- `frontier`: frontier mapping helpers (status math only; no geometry).

6) Functions
(Manifest only.)

7) Manifest Outline (structure)
[package] name vm_algo, version 0.1.0, edition 2021, rust-version pinned; dual license.
[lib] rlib, path "src/lib.rs".
[features]
default = ["std","tab_ranked","tab_score","pr_methods","mmp","gates","frontier"]
std = []
tab_ranked = []
tab_score = []
pr_methods = []
mmp = []
gates = []
frontier = []
[dependencies]
vm_core = { path = "../vm_core" }   # only dependency; pulls rounding & rng.
[dev-dependencies]
proptest = "1"        # optional, for property tests
rand_chacha = "0.3"   # tests only; runtime RNG comes from vm_core

8) State Flow
vm_pipeline calls vm_algo for tabulate/allocate/gates/frontier using vm_core types; vm_algo has no file I/O.

9) Determinism & Numeric Rules
All numeric ops use vm_core::rounding (integer/rational, half-even). Any randomness is injected via vm_core::rng::TieRng; no external RNG at runtime.

10) Edge Cases & Failure Policy
No transitive std::fs/serde_json pulls. Feature flags must not change public type layouts/signatures—only compilation scope.

11) Test Checklist
- `cargo check -p vm_algo` (defaults) OK.
- `cargo check -p vm_algo --no-default-features --features "std,pr_methods,gates"` OK.
- Unit/property tests compile without I/O deps; allocation edge cases pass when enabled.
```

**Canonical `Cargo.toml` (drop-in):**

```toml
[package]
name = "vm_algo"
version = "0.1.0"
edition = "2021"
rust-version = "1.77"
license = "Apache-2.0 OR MIT"
description = "Algorithm layer: tabulation, seat allocation, gates/frontier; depends only on vm_core."
# repository = "..."; readme = "README.md"

[lib]
name = "vm_algo"
path = "src/lib.rs"
crate-type = ["rlib"]

[features]
default     = ["std", "tab_ranked", "tab_score", "pr_methods", "mmp", "gates", "frontier"]
std         = []
tab_ranked  = []
tab_score   = []
pr_methods  = []
mmp         = []
gates       = []
frontier    = []

[dependencies]
vm_core = { path = "../vm_core" }

[dev-dependencies]
proptest   = "1"
rand_chacha = "0.3"   # tests only; runtime RNG comes from vm_core
```

If you want, I can mirror this with a minimal `src/lib.rs` stub that exposes feature-gated modules and re-exports the small helper traits from `vm_core`.
