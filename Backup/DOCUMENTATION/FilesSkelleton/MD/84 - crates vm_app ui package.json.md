<!-- Converted from: 84 - crates vm_app ui package.json.docx on 2025-08-12T18:20:47.854482Z -->

```
Lean pre-coding sheet — 84/89
Component: crates/vm_app/ui/package.json (UI workspace manifest)
1) Goal & success
Goal: Define UI package metadata and scripts so the desktop app ships with bundled, offline assets (no external fonts/styles/tiles), and produces artifacts that align with report rules (presentation-only, one decimal).
Success: npm run build creates a local bundle consumed by Tauri with no network fetches at run-time; versions are pinned/locked for reproducible builds.
2) Scope
In: name/version/private, scripts (build, dev, lint, typecheck), dependency pins for the UI (vite, types), and explicit note that all runtime assets are local (MapLibre tiles/styles, fonts).
Out: Back-end commands (Tauri main.rs) and security posture (lives in tauri.conf.json).
3) Inputs → outputs
Inputs: UI source under ui/ plus local map assets (style.json/mbtiles) to be referenced relatively.
Outputs: Static assets folder consumed by Tauri packaging; no external assets in reports/UI.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (scripts only)
"build" — run vite build to produce a static bundle; it must reference only local assets.
"dev" — local preview; should still avoid remote CDNs.
"lint" / "typecheck" — consistency; no effect on determinism.
7) Algorithm outline (practical steps)
Pin UI deps; commit lockfile for reproducible builds.
Ensure imports for fonts/styles/MapLibre point to packaged files (no URLs).
Build static bundle; Tauri packages it without any updater/telemetry.
8) State flow (very short)
npm run build → emits static UI → Tauri bundles UI and local assets → backend commands read/write canonical artifacts.
9) Determinism & numeric rules
UI must not alter engine outputs; percentages shown with one decimal; offline-only assets.
10) Edge cases & failure policy
If any dependency pulls remote assets at build or runtime, fail build and replace with bundled copies.
If a page requests remote fonts/styles/tiles, treat as a config error (must be local).
11) Test checklist (must pass)
Build succeeds with no network; output bundle references only local paths.
Reports render with one-decimal precision; footers & IDs come solely from Result/RunRecord.
```
