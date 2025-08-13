
```
Pre-Coding Essentials (Component: crates/vm_core/Cargo.toml, Version/FormulaID: VM-ENGINE v0) — 21/89

1) Goal & Success
Goal: Define the core library crate (no I/O) that holds immutable IDs/types, VM-VAR primitives, rounding helpers, and a seedable, deterministic RNG.
Success: Builds as an rlib on all supported targets; optional `serde` compiles; no JSON/FS/web deps; downstream crates (`vm_io`, `vm_algo`, `vm_pipeline`) link cleanly and deterministically.

2) Scope
In scope: package metadata, Rust edition/rust-version pin, [lib], features, minimal deps, crate-level lint policy (kept lean in Cargo, stricter in code).
Out of scope: binaries/CLI, JSON serialization or file access (lives in `vm_io`), report/presentation concerns.

3) Inputs → Outputs
Inputs: Workspace toolchain + root profiles.
Outputs: `vm_core` rlib exposing:
- Canonical ID types (`ResultId`, `RunId`, `FrontierMapId`), tokens for `unit_id`/`option_id`.
- VM-VAR key newtypes (`VmVarId`) + strongly typed domains (booleans as real bools; percentages as bounded ints; `VM-VAR-052` as `u64` seed ≥0).
- Rounding helpers (integer-first; emit floats only in outer layers).
- RNG facade pinned to `rand_chacha` with stable seeding.

4) Entities/Types (manifest-relevant summary)
- Library name: `vm_core` (crate-type = ["rlib"])
- Feature flags:
  - `std` (default): enable standard library usage.
  - `serde` (optional): derive Serialize/Deserialize on select core types; strictly behind this flag.
- No build script.

5) Variables (only relevant policy toggles)
- None in Cargo; variables live in code as types/enums aligned with Annex A (e.g., tie policy 050 enum; seed 052 = non-negative integer).

6) Functions
(Manifest only; code signatures defined in `src/`.)

7) Manifest Structure (authoring outline)
[package]
- name = "vm_core"
- version = "0.1.0"
- edition = "2021"
- rust-version = "<pin to workspace toolchain>"
- license = "Apache-2.0 OR MIT"
- description = "Core types, VM-VAR domains, rounding, and deterministic RNG for the VM engine."
- repository/homepage/readme optional (workspace policy)
- categories/keywords optional (internal)

[lib]
- name = "vm_core"
- path = "src/lib.rs"
- crate-type = ["rlib"]

[features]
- default = ["std"]
- std = []
- serde = ["dep:serde"]

[dependencies]  // minimal, no I/O
- serde        = { version = "1", features = ["derive"], optional = true, default-features = false }
- rand_core    = { version = "0.6", default-features = false }
- rand_chacha  = { version = "0.3", default-features = false }

[dev-dependencies]  // keep minimal; only what local unit tests need
- (none by default; add tiny crates only if tests require)

[profile.*]  // inherit workspace; do not override LTO/opt here unless reproducibility requires it

(resolver = "2" is assumed at workspace root)

8) State Flow
- `vm_core` is a leaf library: no JSON or FS; other crates depend on it.
- `vm_io` enables `serde` to serialize/deserialize canonical artifacts (Registry/Tally/Params/Result/RunRecord/FrontierMap).
- `vm_algo` / `vm_pipeline` import RNG/rounding + VM-VAR domains from here.

9) Determinism & Numeric Rules
- RNG: expose a seedable API using `rand_chacha::ChaCha20Rng` (pinned via Cargo). Seed is integer-based (aligns with `VM-VAR-052` ≥ 0); tie policy (050) lives as an enum type.
- Integer-first math: rounding helpers live here; JSON numbers (shares) are emitted by outer layers; no floats hidden in dependencies.
- No accidental platform features: all deps set `default-features = false`.

10) Edge Cases & Failure Policy
- Builds with default features and with `--no-default-features --features serde`.
- If `serde` is off, core types compile (derive gates via `#[cfg(feature = "serde")]`).
- Do not introduce `serde_json`, `anyhow`, `thiserror`, or `std::fs` here—keeps layering clean.
- Any new dep must be justified (determinism/scope) and added with `default-features = false`.

11) Test Checklist (must pass)
- `cargo check -p vm_core` (default features) → OK.
- `cargo check -p vm_core --no-default-features --features serde` → OK.
- (If/when no_std supported) `cargo check -p vm_core --no-default-features` → OK.
- Downstream compile proof: `cargo test -p vm_io --features serde` succeeds (feature wiring).
- RNG determinism smoke test in `vm_algo`: same inputs + same `VM-VAR-052` → identical outcomes.
```

**Canonical manifest snippet (for later copy into `Cargo.toml` when you’re ready):**

```toml
[package]
name = "vm_core"
version = "0.1.0"
edition = "2021"
rust-version = "1.77"         # or your workspace pin
license = "Apache-2.0 OR MIT"
description = "Core types, VM-VAR domains, rounding, and deterministic RNG for the VM engine."

[lib]
name = "vm_core"
path = "src/lib.rs"
crate-type = ["rlib"]

[features]
default = ["std"]
std = []
serde = ["dep:serde"]

[dependencies]
serde       = { version = "1", features = ["derive"], optional = true, default-features = false }
rand_core   = { version = "0.6", default-features = false }
rand_chacha = { version = "0.3", default-features = false }

[dev-dependencies]
# (intentionally minimal)
```
