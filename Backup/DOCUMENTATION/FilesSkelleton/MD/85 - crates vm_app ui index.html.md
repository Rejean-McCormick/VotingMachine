<!-- Converted from: 85 - crates vm_app ui index.html.docx on 2025-08-12T18:20:47.862133Z -->

```
Lean pre-coding sheet — 85/89
Component: crates/vm_app/ui/index.html (UI entry document)
1) Goal & success
Goal: Minimal, offline HTML shell that mounts the UI bundle and never references remote assets (fonts/styles/JS/map tiles).
Success: App loads locally packaged UI and MapLibre assets; no HTTP/DNS; report views display numbers with one-decimal precision (presentation only).
2) Scope
In: HTML skeleton (doctype, <html lang>, <meta charset>, <meta viewport>), a single script to the bundled entry (Vite output), relative links to local CSS, containers for: report sections (Doc 7A order) and an optional MapLibre div.
Out: Any analytics, CDN fonts, external CSS/JS, or network fetches (forbidden by policy).
3) Inputs → outputs
Inputs: Built UI bundle (from package.json scripts) and packaged map assets (ui/public/map/style.json, …/tiles/world.mbtiles).
Outputs: A static DOM that the UI JS hydrates; no data is pulled directly here—rendered content ultimately mirrors Result, optional FrontierMap, and RunRecord.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
None (index.html is declarative; numeric/display policy enforced by renderer per Doc 7).
6) Functions (signatures only)
N/A (no scripts inline; JS lives in bundled entry).
7) Algorithm outline (page structure)
Minimal head: charset UTF-8, viewport, <meta http-equiv="Content-Security-Policy"> that blocks remote (default-src 'self'). (Enforces offline posture.)
Link local CSS only; no web fonts/CDNs.
Body: <div id="app"> root; child sections in Doc 7A’s order so the renderer can mount them deterministically (Cover/Snapshot → Eligibility → Ballot → Allocation → Aggregation → Gates → Frontier → Ties → Sensitivity → Integrity).
Optional <div id="map">; JS will point MapLibre to local public/map/style.json.
One <script type="module" src="/src/main.ts"> (dev) or built asset path (prod). No inline analytics.
8) State flow (very short)
Static HTML loads → bundled JS bootstraps → UI calls backend commands → UI renders read-only from Result/RunRecord (+ optional FrontierMap) with one-decimal display.
9) Determinism & numeric rules
Page must not compute outcomes; it only presents results with one-decimal percentages; all assets local.
10) Edge cases & failure policy
Any external URL (fonts/styles/scripts/tiles) is a bug—remove and replace with bundled files.
If map assets are missing, hide the map panel; report sections must still render (data comes from artifacts).
11) Test checklist (must pass)
Open with Tauri: no HTTP/DNS observed; all resources load from app:///local package.
Visuals adhere to Doc 7 rules (one-decimal, fixed wording blocks, no external assets).
```
