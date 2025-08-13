````md
Perfect Skeleton Sheet — crates/vm_app/src-tauri/Cargo.toml — 80/89
(Aligned with VM-ENGINE v0 offline/determinism rules)

1) Goal & Success
Goal: Manifest for the Tauri desktop backend. Pinned, offline, deterministic; bundles local assets; targets Win/macOS/Linux (x86-64/arm64).
Success: `cargo build -p vm_app/src-tauri --locked` succeeds on supported targets; runtime has no telemetry/network; fonts/styles/tiles are local.

2) Scope
In: [package], [dependencies], [build-dependencies], [features], deterministic [profile]s; optional target-gated deps.  
Out: UI build (vite/npm), map assets, security policy (lives in `tauri.conf.json`), app Rust code.

3) Inputs → Outputs
Inputs: Workspace toolchain + lockfile; UI bundle under `ui/`; MapLibre tiles/styles (local).  
Outputs: Desktop backend binary packaged by Tauri; emits canonical JSON artifacts only (via downstream crates).

4) Entities/Tables (minimal)
N/A (manifest only).

5) Variables (only ones used here)
Feature flags (pass-through):
- `frontier` → enables frontier map support (maps to downstream crates).
- `report-html` → enables HTML renderer in reporting.

6) Functions
(Manifest only.)

7) Recommended Cargo.toml (template)
```toml
[package]
name = "vm_app_tauri"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0 OR MIT"
publish = false
description = "Tauri backend for the VM Engine desktop app (offline, deterministic)."

# Use resolver v2 for correct feature unification
resolver = "2"

# --- Binaries ---
[[bin]]
name = "vm-app"
path = "src/main.rs"

# --- Features (pass-through toggles; backend remains offline) ---
[features]
default = []
frontier = ["vm_pipeline?/frontier", "vm_report?/frontier"]
report-html = ["vm_report?/render-html"]

# --- Dependencies ---
[dependencies]
# Prefer workspace-pinned versions for determinism.
tauri = { workspace = true, features = ["fs-all", "dialog-all", "shell-open"] }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
# Internal crates (paths or workspace deps)
vm_core = { workspace = true }
vm_io = { workspace = true }
vm_algo = { workspace = true }
vm_pipeline = { workspace = true, optional = true }
vm_report = { workspace = true, optional = true }

# If the workspace does not define these deps, replace `workspace = true`
# with pinned versions or local paths, e.g.:
# tauri = { version = "=1.5.12", features = ["fs-all","dialog-all","shell-open"] }
# vm_pipeline = { path = "../../vm_pipeline", optional = true }

# --- Build dependencies ---
[build-dependencies]
tauri-build = { workspace = true }

# --- Target-specific hints (optional, purely local tooling) ---
[target."cfg(windows)".dependencies]
# winresource crate can be used if embedding icons locally (no network).
# winresource = { workspace = true, optional = true }

# --- Deterministic profiles ---
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
panic = "abort"
````

8. State Flow
   This manifest compiles the backend; Tauri packages the app with **local** assets. All pipeline/report logic stays in internal crates; no network calls.

9. Determinism & Numeric Rules

* Builds with `--locked`; all external versions pinned via workspace.
* No build-time network access (fail fast if any dep tries).
* Artifacts produced via downstream crates use canonical JSON (UTF-8, sorted keys, LF, UTC).

10. Edge Cases & Failure Policy

* Missing UI bundle or tiles: backend still compiles; packaging step will fail clearly (by design).
* Feature combos are pass-through; if `frontier` is enabled without adjacency data, downstream simply omits mapping (no net).
* Any dependency that attempts network access under `--locked` must fail the build.

11. Test Checklist (must pass)

* `cargo build -p vm_app/src-tauri --locked` on Windows/macOS/Linux, x86-64/arm64.
* Runtime: no telemetry/network; fonts/styles/tiles loaded from the app bundle.
* Changing serialization/math deps triggers re-certification per determinism policy.

```
```
