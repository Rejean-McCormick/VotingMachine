<!-- Converted from: 86 - crates vm_app ui vite.config.ts.docx on 2025-08-12T18:20:47.881008Z -->

```
Lean pre-coding sheet — 86/89
Component: crates/vm_app/ui/vite.config.ts (UI build config)
1) Goal & success
Goal: Produce a fully offline, reproducible static bundle for the desktop app (Tauri). No external/CDN assets; output paths work from a packaged file URL.
Success: npm run build emits deterministic files referenced relatively (e.g., ./assets/...), consumed by Tauri with no network at runtime.
2) Scope
In: Vite base path, outDir, asset handling, Rollup output naming, dev server hardening (dev only), and explicit prohibition of remote assets.
Out: App security policy (lives in tauri.conf.json) and backend commands (main.rs).
3) Inputs → outputs
Inputs: index.html, src/main.ts, local styles/fonts, local MapLibre assets under ui/public/map/.
Outputs: Static bundle in dist/ with content-hash filenames and relative URLs so Tauri can load from package.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
N/A (configuration file).
7) Algorithm outline (what the config enforces)
Set base='./' and outDir='dist'.
Disable inlining & sourcemaps; use explicit Rollup name templates (assets/[name]-[hash][extname], etc.).
For dev, no proxies/CDNs; prefer local-only dev to mirror offline runtime.
Ensure map/style/font paths are relative into public/ so they’re bundled locally.
8) State flow (very short)
vite build → dist/ → Tauri packages dist/ with tauri.conf.json → app loads all assets locally; backend writes canonical artifacts.
9) Determinism & numeric rules
Build must be reproducible; no environment-dependent URLs or timestamps; runtime remains offline. Reporting precision rules live in Doc 7 (UI just displays).
10) Edge cases & failure policy
Any absolute http(s):// import or CSS @import url(...) → fail build and replace with local copy.
Sourcemaps with absolute file paths can break reproducibility; keep disabled for releases.
11) Test checklist (must pass)
npm run build works offline; dist/ contains only relative references; Tauri runs with no HTTP/DNS.
```
