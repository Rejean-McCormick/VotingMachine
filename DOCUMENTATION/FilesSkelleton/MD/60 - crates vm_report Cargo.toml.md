```toml
Pre-Coding Essentials (Component: crates/vm_report/Cargo.toml, Version/FormulaID: VM-ENGINE v0) — 60/89

1) Goal & Success
Goal: Declare the reporting crate that renders Result/RunRecord (and optional FrontierMap) offline with deterministic, reproducible output per Doc 7.
Success: Builds as an rlib under a locked toolchain; strictly offline (no network deps); percent formatting is one-decimal; JSON and HTML renderers are feature-gated and consume only engine artifacts.

2) Scope
In scope: Manifest metadata, edition/rust-version, features for renderers, minimal deps with `default-features = false`, and deterministic profiles.
Out of scope: Pipeline/algorithms/I/O; report structure and code live in `src/` and templates bundled locally.

3) Inputs → Outputs
Inputs: Workspace toolchain; vm_core (percent helpers), vm_io (types if needed for serde), local templates.
Outputs: Library usable by CLI/app to render JSON/HTML reports; no binaries here.

4) Entities/Tables (minimal)
N/A (manifest only).

5) Variables (build/features)
Feature toggles:
- `render_json` — JSON renderer (serde only).
- `render_html` — HTML renderer (pure offline templating; templates embedded).
- `std` — default on.

6) Functions
(Manifest only.)

7) Manifest skeleton (deterministic, offline)
[package]
name = "vm_report"
version = "0.1.0"
edition = "2021"
rust-version = "1.76"            # pin ≥ workspace
license = "Apache-2.0 OR MIT"
description = "Deterministic offline report renderers for VM-ENGINE results"
repository = "<workspace>"

[lib]
name = "vm_report"
path = "src/lib.rs"
crate-type = ["rlib"]

[features]
default      = ["std", "render_json"]
std          = []
render_json  = ["dep:serde", "dep:serde_json"]
render_html  = ["dep:minijinja", "dep:include_dir"]   # offline, embedded templates

[dependencies]
vm_core     = { path = "../vm_core", default-features = false }          # percent_one_decimal helpers
vm_io       = { path = "../vm_io",   default-features = false, optional = true } # types if reused in renderer API
serde       = { version = "1", features = ["derive"], default-features = false, optional = true }
serde_json  = { version = "1", default-features = false, optional = true }
minijinja   = { version = "1", default-features = false, optional = true }       # pure rust templates, no net
include_dir = { version = "0.7", default-features = false, optional = true }     # bundle templates at compile time
itoa        = { version = "1", default-features = false }                        # fast int→string for % formatting

[dev-dependencies]
insta              = { version = "1", default-features = false }        # snapshot tests for deterministic output
similar-asserts    = "1"
serde_json         = { version = "1", default-features = false }
minijinja          = { version = "1", default-features = false }        # used in tests when render_html on
include_dir        = { version = "0.7", default-features = false }

[package.metadata.vm]
offline = true
deterministic = true

# No build.rs. All templates embedded via include_dir. No network/font/map tiles at runtime.

8) State Flow (very short)
`vm_report` is linked by the app/CLI after pipeline completion. It consumes Result/RunRecord/FrontierMap and renders JSON/HTML strictly offline.

9) Determinism & Numeric Rules
- One-decimal percentage formatting implemented via vm_core integer helpers (no floats).
- No time/locale dependencies; UTF-8 only.
- Template assets embedded; no remote fetches.

10) Edge Cases & Failure Policy
- If `render_html` is enabled but templates missing, builds/tests fail (no fallback to network).
- No optional transitive defaults that could introduce networking or floats.

11) Test Checklist (must pass)
- `cargo check -p vm_report` with defaults (std+render_json).
- `cargo check -p vm_report --no-default-features --features "std,render_html"` (HTML only) builds.
- Snapshot tests confirm identical HTML/JSON bytes across OS/arch.
- `cargo tree -e features` shows no net/reqwest and no float-formatting deps pulled in by default.
```
