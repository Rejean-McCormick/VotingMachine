````md
Pre-Coding Essentials (Component: crates/vm_app/Cargo.toml, Version/FormulaID: VM-ENGINE v0) — 79/89

1) Goal & Success
Goal: Minimal “meta-manifest” for the desktop app wrapper. It declares package metadata and mirrors feature flags, but adds **no** runtime/network deps. The actual Tauri backend is a separate crate at `vm_app/src-tauri/`.  
Success: `cargo metadata -p vm_app` works; building the backend explicitly (`-p vm_app/src-tauri`) succeeds under `--locked`; offline/deterministic constraints are preserved.

2) Scope
In scope: Package stanza, edition/license, `publish=false`, resolver v2, deterministic release profile, feature names documented (mirrored by backend).  
Out of scope: Any backend/UI code (lives in `src-tauri/` and `ui/`); no dependencies here to avoid pulling Tauri unless directly targeted.

3) Inputs → Outputs
Inputs: Workspace toolchain; backend crate at `src-tauri/`; UI bundle under `ui/` (consumed by Tauri only).  
Outputs: None directly. This package exists to group app metadata and surface features consistently.

4) Entities/Tables (minimal)
N/A (manifest only).

5) Variables (only ones used here)
Feature flags (names only, **no wiring from this crate**):
- `frontier` — enables frontier map support in the backend.
- `report-html` — enables HTML renderer in the backend.

6) Functions
(Manifest only.)

7) Suggested Cargo.toml shape (deterministic & offline)
```toml
[package]
name = "vm_app"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0 OR MIT"
publish = false
description = "Desktop app wrapper (meta); backend is in src-tauri/"
# This crate should not pull the backend automatically.
# Build the app explicitly with: cargo build -p vm_app/src-tauri --locked

# Keep resolver v2 for correct feature unification across workspace.
resolver = "2"

[features]
# Intentionally empty arrays here: the real behavior is implemented in src-tauri.
# These names are mirrored there; callers should enable features on the backend crate.
default = []
frontier = []
report-html = []

# No [dependencies] here on purpose — prevents tauri/network deps from being pulled
# unless the backend crate is explicitly built.
[dependencies]

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
````

8. State Flow
   Acts as a container package only. Building the actual app targets `vm_app/src-tauri`, which bundles offline UI/assets and links downstream crates (pipeline/report) deterministically.

9. Determinism & Numeric Rules

* No networked build scripts or deps at this level.
* Release profile enforces deterministic codegen (`lto`, `codegen-units=1`, `panic="abort"`).
* The app layer must not introduce telemetry or online fonts; assets are bundled by the backend.

10. Edge Cases & Failure Policy

* If Node/Vite or UI assets are missing, **this** meta package still resolves; only `src-tauri` builds should fail (by design).
* Enabling `frontier`/`report-html` on this crate alone has no effect; they must be enabled on `vm_app/src-tauri`. This is documented to avoid confusion.

11. Test Checklist (must pass)

* `cargo metadata -p vm_app` succeeds.
* `cargo build -p vm_app/src-tauri --locked` succeeds on supported OS/arch (when backend & UI are present).
* Building `vm_app` alone does **not** attempt to fetch networked deps or compile Tauri.
* With `--features frontier,report-html` targeting the backend crate, downstream features link and runtime remains offline.

```
```
