<!-- Converted from: 89 - crates vm_app ui public map tiles world.mbtiles.docx on 2025-08-12T18:20:47.960056Z -->

```
Lean pre-coding sheet — 89/89
Component: crates/vm_app/ui/public/map/tiles/world.mbtiles (offline vector-tiles DB; binary)
 Version/FormulaID: VM-ENGINE v0
1) Goal & success
Goal: Ship a local MBTiles database so MapLibre renders maps entirely offline in the desktop app.
Success: App renders base map without any HTTP/DNS; tiles/fonts/styles are bundled; reports remain self-contained.
2) Scope
In: One binary MBTiles file at ui/public/map/tiles/world.mbtiles, consumed by the UI’s MapLibre style. (Asset only; not read by core/pipeline.)
Out: Frontier logic or contiguity rules (computed in core/pipeline; map is presentation only).
3) Inputs → outputs
Inputs: This MBTiles file; the style.json next to it points to local sources.
Outputs: None. Visuals only; no effect on Result / RunRecord.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
None.
6) Functions (signatures only)
N/A (static asset).
7) Algorithm outline (practical steps)
Place world.mbtiles at crates/vm_app/ui/public/map/tiles/. (Path matches the repo skeleton.)
Reference it locally from style.json (local sources; no remote glyphs/sprites).
UI loads the style; backend exposes only local files; pipeline output (e.g., FrontierMap) is overlaid visually—map does not change computation.
8) State flow (very short)
Start app → style loads → tiles read locally → optional overlays styled by FrontierMap statuses; no network used.
9) Determinism & numeric rules
Purely presentational; cannot alter allocation/gates/labels. Core determinism and rounding rules apply only to computational artifacts.
10) Edge cases & failure policy
Missing MBTiles: map panel hides; reports still render (frontier text/table remain).
Any remote URL in style/sources/glyphs/sprite is a config error; replace with packaged files.
Adjacency edge types, if shown, must use the fixed set {land, bridge, water} from the data—styling only.
11) Test checklist (must pass)
Launch app offline; base map renders; zero HTTP/DNS calls observed.
Frontier overlay colors/legend match statuses from FrontierMap tests; absence of tiles does not change Result/RunRecord.
All 89/89 sheets completed.
Sources
You said:
.
2/2
ChatGPT said:
Thought for 30s
Noted.
All 89 lean pre-coding sheets are complete and aligned with Docs 1–7 + Annex A/B. No cross-doc conflicts surfaced in the sheets.
If you want a single consolidated INDEX.md of all sheets (or for me to start generating code stubs), say which you prefer.
```
