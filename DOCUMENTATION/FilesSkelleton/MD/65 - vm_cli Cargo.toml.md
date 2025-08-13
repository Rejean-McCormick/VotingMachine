```toml
Pre-Coding Essentials (Component: vm_cli/Cargo.toml, Version/FormulaID: VM-ENGINE v0) — 65/89

1) Goal & Success
Goal: Declare the offline, deterministic CLI binary crate `vm` that wires the pipeline and reporting without introducing networked deps or nondeterminism.
Success: `cargo build -p vm_cli --locked` succeeds on Win/macOS/Linux; `vm --help` runs; features pass through cleanly to report/frontier; no runtime network; outputs follow Docs 4–5 stage order.

2) Scope
In scope: Crate metadata, [[bin]] target, dependency pins, feature passthrough (report-json/html, frontier), deterministic build profile.
Out of scope: CLI argument parsing and main flow (`src/args.rs`, `src/main.rs` live in components 67/68).

3) Inputs → Outputs
Inputs: Workspace toolchain + internal crates (`vm_pipeline`, `vm_io`, `vm_report`), `clap` for parsing, optional `serde_json` for `--print-json`.
Outputs: Binary `vm` that orchestrates LOAD → VALIDATE → TABULATE → ALLOCATE → AGGREGATE → APPLY_DECISION_RULES → MAP_FRONTIER → RESOLVE_TIES → LABEL → BUILD_RESULT → BUILD_RUN_RECORD.

4) Entities/Tables (minimal)
Manifest only.

5) Variables (build/features)
- Features exposed here map 1:1 to downstream crate features (no surprise toggles).
- No build.rs; no networked/build-time scripts.

6) Functions
(Manifest only.)

7) Algorithm Outline (manifest structure blueprint)
# Package & library surface
[package]
name = "vm_cli"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0 OR MIT"
resolver = "2"
# no build.rs

# Binary target
[[bin]]
name = "vm"
path = "src/main.rs"

# Features: pass-through, deterministic defaults
[features]
default = ["std", "report-json"]         # JSON rendering available by default
std = []
# Reporting frontends (map to vm_report)
report-json = ["vm_report/render_json"]
report-html = ["vm_report/render_html"]
# Frontier support (map to algo/pipeline/report if those crates gate it)
frontier = ["vm_algo/frontier", "vm_pipeline/frontier", "vm_report/frontier"]

# Dependencies (pin majors; avoid default-features where sensible)
[dependencies]
vm_pipeline = { path = "../vm_pipeline" }
vm_io       = { path = "../vm_io" }
vm_report   = { path = "../vm_report", default-features = false }  # enable via features above

clap = { version = "4", features = ["derive"], default-features = false }  # deterministic, no color/env features
# Optional pretty/JSON printing in CLI
serde        = { version = "1", features = ["derive"], optional = true, default-features = false }
serde_json   = { version = "1", optional = true }  # used only behind --print-json
indicatif    = { version = "0.17", optional = true, default-features = false } # if a spinner/progress is desired; keep optional/off by default

# Dev-only helpers
[dev-dependencies]
assert_cmd = "2"
predicates = "3"

# Profiles: deterministic release
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = "symbols"

8) State Flow
The CLI depends on the three internal crates and invokes the fixed pipeline state machine. Report generation is delegated to `vm_report` (JSON by default, HTML when feature-enabled).

9) Determinism & Numeric Rules
- No time- or OS-derived randomness in the CLI; RNG seed comes only from ParameterSet and is surfaced by pipeline.
- CLI does not modify canonical bytes; hashing/canonicalization live in `vm_io`.
- Disable unnecessary default features on deps (e.g., `clap`) to avoid nondeterministic coloring/terminal probing.

10) Edge Cases & Failure Policy
- Feature combos compile cleanly: `--features frontier`, `--features report-html`, or both.
- If inputs lack adjacency and `frontier` is enabled, downstream skips frontier without failing the run.
- No network access at build/run; templates/assets are bundled by downstream report crate.

11) Test Checklist (must pass)
- `cargo build -p vm_cli --locked` (debug & release) on Win/macOS/Linux.
- `cargo run -p vm_cli -- --help` exits 0.
- `cargo run -p vm_cli --features report-html -- --print-html` succeeds when model present.
- `cargo run -p vm_cli --features frontier` links; pipeline emits FrontierMap only if data present.
- End-to-end smoke over Annex B fixtures yields deterministic Result & RunRecord; `--print-json` uses `serde_json` path when enabled.
```
