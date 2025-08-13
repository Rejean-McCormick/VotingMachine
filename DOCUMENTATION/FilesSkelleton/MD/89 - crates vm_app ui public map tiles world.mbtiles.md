

```
Lean pre-coding sheet — 89/89
Component: crates/vm_app/ui/public/map/tiles/world.mbtiles (offline vector-tiles DB; binary)
Version/FormulaID: VM-ENGINE v0

1) Goal & success
Goal: Ship a *local* MBTiles database so MapLibre renders a base map entirely offline in the desktop app.
Success: App shows a basemap with zero HTTP/DNS; all map sources (tiles, glyphs, sprites) are bundled locally; computational artifacts (Result/RunRecord) remain unaffected.

2) Scope
In scope: A single MBTiles file at ui/public/map/tiles/world.mbtiles packaged with the app and referenced by the style.json/local protocol.
Out of scope: Frontier logic, contiguity rules, or any pipeline math. The map is presentational only.

3) Inputs → outputs
Inputs: The binary MBTiles file; style.json points to it via a local path/protocol.
Outputs: None to the engine. Visual layer only; does not alter Result/RunRecord or hashes.

4) Entities/Tables (minimal)
Asset only (no code). Consumed by the UI renderer through the style’s source URL.

5) Variables (only ones used here)
None (no VM-VARs). Any styling knobs live in style.json.

6) Functions (signatures only)
N/A — static asset.

7) Implementation Outline (practical wiring)
• Location: keep at crates/vm_app/ui/public/map/tiles/world.mbtiles.
• Reference from style.json with a **local** URL/protocol understood by the app (e.g., app:///map/tiles/world.mbtiles or an app-local mbtiles:// handler). Never http(s)://.
• Ensure the style’s source is compatible with offline MBTiles access in your stack (custom protocol/loader or pre-extracted ./tiles/{z}/{x}/{y}.pbf directory if you don’t ship a reader).
• Bundle glyphs/sprites locally; style.json must not reference remote sprite/glyph URLs.
• Tauri FS scope must allow reads of this packaged path; no symlink escapes.

8) State flow (very short)
App start → style.json loads → tiles read locally from world.mbtiles → optional Frontier overlay draws atop. No network; no effect on pipeline outputs.

9) Determinism & numeric rules
Purely presentational. Cannot change tabulation/allocation/gates/labels. Core determinism (ordering/rounding/RNG) applies only to computational artifacts; this asset does not enter hashing of Result/RunRecord.

10) Edge cases & failure policy
• Missing MBTiles: hide/disable the map panel; reports still render fully from artifacts.
• Any remote URL in style (tiles/glyphs/sprite): configuration error — replace with bundled assets.
• Licensing: ensure your tiles’ data license (e.g., OSM-derived) permits offline bundling; include NOTICE if required.

11) Test checklist (must pass)
• Launch offline → base map renders; zero HTTP/DNS observed.
• Frontier overlays (if enabled) appear correctly; removing the MBTiles hides only the map, not report content.
• Packaging works on Win/macOS/Linux (x64/arm64) with no runtime downloads.
```

