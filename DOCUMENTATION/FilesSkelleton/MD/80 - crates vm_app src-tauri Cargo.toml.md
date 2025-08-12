<!-- Converted from: 80 - crates vm_app src-tauri Cargo.toml.docx on 2025-08-12T18:20:47.725862Z -->

```toml
Lean pre-coding sheet — 80/89
Component: crates/vm_app/src-tauri/Cargo.toml (Tauri backend manifest)
 Version/FormulaID: VM-ENGINE v0
1) Goal & success
Goal: Define the Tauri backend crate manifest for the desktop app; pin deps; respect offline/determinism rules; target Win/macOS/Linux (x86-64/arm64).
Success: cargo build -p vm_app/src-tauri --locked succeeds on all targets; runtime has no network/telemetry; assets (fonts/styles/tiles) are bundled.
2) Scope
In: [package] meta; [dependencies] (tauri + app crates); [features] passthrough (e.g., report-html, frontier); profiles/determinism hints.
Out: UI build (vite/npm), map assets, filesystem policy (lives in tauri.conf.json), app Rust code (src-tauri/src/main.rs). Security posture and CI belong to Doc 3B.
3) Inputs → outputs
Inputs: Workspace toolchain pin; Cargo.lock; local UI bundle under ui/; MapLibre assets (tiles/styles) packaged.
Outputs: Desktop backend binary packaged by Tauri; no external asset fetch at runtime.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
N/A (manifest).
7) Algorithm outline (what the manifest enforces)
Pin Rust toolchain and crate versions; --locked builds only.
Depend on tauri and internal crates (vm_report, vm_core, etc.) with features gated.
Ensure offline runtime: no telemetry, no network; bundle fonts/styles/tiles.
Target Win/macOS/Linux, x86-64/arm64 (CI matrix will enforce).
8) State flow (very short)
Manifest → compile backend → Tauri packages app with local assets. No pipeline semantics here; adheres to platform/offline rules used by reports.
9) Determinism & numeric rules
Canonical serialization and hashing rules apply to artifacts the app emits; manifest itself must not introduce nondeterminism (no build-time net, fixed versions, stable ordering).
10) Edge cases & failure policy
Build scripts or deps that attempt network access under --locked → fail the build (policy).
Filesystem scope and shell commands are restricted by Tauri config (security posture); treat violations as packaging errors, not code paths.
11) Test checklist (must pass)
cargo build -p vm_app/src-tauri --locked OK on Windows/macOS/Ubuntu; x86-64 and arm64.
Runtime checks: no telemetry/network; fonts/styles/tiles are local.
Changing critical math/serialization deps triggers determinism re-cert (6C-020).
```
