<!-- Converted from: 88 - crates vm_app ui public map style.json.docx on 2025-08-12T18:20:47.931648Z -->

```
Lean pre-coding sheet — 88/89
Component: crates/vm_app/ui/public/map/style.json (MapLibre style; packaged)
 Version/FormulaID: VM-ENGINE v0
1) Goal & success
Goal: Provide a MapLibre style.json that references only local sources (tiles/sprites/fonts), for offline frontier rendering.
Success: App renders maps with no network, using bundled tiles/styles/fonts; reporting remains self-contained.
2) Scope
In scope: Style metadata (name, version), sources → local MBTiles/vector sources, glyphs/sprite → local paths, layers for units/adjacency/frontier statuses.
Out of scope: Algorithmic frontier logic (done in pipeline), any remote URLs. Frontier statuses come from FrontierMap produced by pipeline.
3) Inputs → outputs
Inputs: Local tiles DB (public/map/tiles/world.mbtiles), local sprite/glyph files, FrontierMap data (IDs/status) from pipeline outputs.
Outputs: On-screen map reflecting FrontierMap statuses; no artifacts written.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
N/A (static style document).
7) Algorithm outline (style logic)
Declare local sources for tiles; set glyphs/sprite to relative app paths.
Define layers for units/labels; style according to FrontierMap statuses: no change / autonomy / phased / immediate; show mediation/protected flags.
Optional line styles for adjacency by type (land/bridge/water).
Ensure color/legend mapping matches report’s Frontier section.
8) State flow (very short)
UI loads style.json → MapLibre reads local tiles/assets → UI overlays statuses from FrontierMap produced after MAP_FRONTIER.
9) Determinism & numeric rules
Map is presentation-only; no computations affect Result/RunRecord. Offline-only assets; report precision rules unchanged.
10) Edge cases & failure policy
Any absolute http(s):// in sources/glyphs/sprite is a bug; replace with packaged files.
Missing FrontierMap → hide frontier layer/legend; report still renders (map optional).
Missing adjacency layer → disable contiguity line styling; statuses still display.
11) Test checklist (must pass)
App runs offline; map loads with zero HTTP/DNS; all style URLs resolve locally.
Frontier legend and colors match the statuses produced by FrontierMap tests (Annex B Part 5).
Toggling features does not change Result/RunRecord bytes (presentation only).
```
