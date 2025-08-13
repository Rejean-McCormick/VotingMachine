<!-- Converted from: 82 - crates vm_app src-tauri tauri.conf.json.docx on 2025-08-12T18:20:47.789944Z -->

```
Lean pre-coding sheet — 82/89
Component: crates/vm_app/src-tauri/tauri.conf.json (Tauri app config)
1) Goal & success
Goal: Lock an offline, sandboxed desktop config: no updater/telemetry/network; bundle local assets (fonts/styles/map tiles); restrict filesystem scope; no shell execution.
Success: App runs on Win/macOS/Linux using only packaged files (UI + MapLibre assets). No HTTP/DNS calls; reports render with no external assets.
2) Scope
In: tauri.conf.json keys for: tauri.security, tauri.allowlist (disable network, shell), tauri.updater (off), tauri.fs.scope (allowlisted dirs), build.distDir/bundle (package UI/assets).
Out: App code (main.rs), runtime pipeline, UI build steps.
3) Inputs → outputs (with schemas/IDs)
Inputs: Local UI bundle (vite output), MapLibre tiles/styles/fonts packaged with the app.
Outputs: A packaged desktop app; pipeline artifacts (Result/RunRecord) remain canonical (UTF-8, sorted keys, LF, UTC).
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
N/A (pure configuration).
7) Algorithm outline (what the config enforces)
Disable network/telemetry/updater globally.
Disallow shell; limit FS to explicit allowlist (open/save dirs only).
Bundle assets (UI, fonts, styles, map tiles) and load them locally.
Ensure produced artifacts keep canonical JSON rules (reinforced by core but referenced in app docs).
8) State flow (very short)
App starts → loads packaged index.html → backend commands operate on local files only (FS within scope). No network paths used.
9) Determinism & numeric rules
Config must not enable any source of non-determinism (no live fetches, no remote fonts). Reporting still uses one-decimal precision; assets are local.
10) Edge cases & failure policy
Any attempt to read outside fs.scope or use net APIs must fail closed.
If the UI references remote assets, build should fail (assets must be bundled).
11) Test checklist (must pass)
Launch app offline; confirm no HTTP/DNS and working UI/maps from packaged assets.
Open/save dialogs restricted to allowed dirs; shell execution unavailable.
```
