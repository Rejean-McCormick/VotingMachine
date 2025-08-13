```
Pre-Coding Essentials (Component: crates/vm_app/ui/public/map/style.json, Version/FormulaID: VM-ENGINE v0) — 88/89

1) Goal & Success
Goal: Provide an offline MapLibre GL style (spec v8) that renders registry units, frontier results, and adjacency edges using only packaged/local assets.
Success: Loads with zero network calls; layers/filters match engine fields (status + flags); looks identical across OS; paths are relative; no http(s) URLs.

2) Scope
In scope: Style JSON skeleton, local sources (raster optional, GeoJSON required), glyph/sprite references, layer IDs/filters/paints, expected feature properties.
Out of scope: Generating GeoJSON, computing statuses, downloading tiles, runtime map logic (UI handles setData etc.).

3) Inputs → Outputs
Inputs (local files, relative to style.json):
- ./fonts/{fontstack}/{range}.pbf            (glyphs)
- ./sprites/sprite                           (sprite sheet, with .json/.png resolved by engine)
- ./tiles/raster/{z}/{x}/{y}.png             (optional, packaged raster tiles)
- ./data/units.geojson                       (unit polygons with id/name, optional status mirror)
- ./data/frontier.geojson                    (per-unit status + flags)
- ./data/adjacency.geojson                   (edge lines with edge_type)
Output: A MapLibre GL style object conforming to v8, with predictable layer IDs and filters.

4) Entities/Tables (minimal expectations for feature properties)
units.geojson (Polygon/MultiPolygon):
  - id: string
  - name: string (optional)
  - status?: "no_change" | "autonomy" | "phased" | "immediate" (optional display hint)
frontier.geojson (Polygon/MultiPolygon; same geometry keys as units):
  - status: same domain as above (required to color by band)
  - mediation: bool
  - enclave: bool
  - protected_blocked: bool
  - quorum_blocked: bool
adjacency.geojson (LineString/MultiLineString):
  - edge_type: "land" | "bridge" | "water"

5) Variables (used by style)
None (static style). All semantics come from feature properties provided by the UI/engine.

6) “Functions” (structure only — layer inventory & IDs)
Style header:
  - version: 8
  - name: "VM Offline Frontier"
  - sprite: "./sprites/sprite"
  - glyphs: "./fonts/{fontstack}/{range}.pbf"
Sources:
  - basemap (optional raster): "./tiles/raster/{z}/{x}/{y}.png" (tileSize=256, min/maxzoom e.g., 0–6)
  - units (geojson): "./data/units.geojson"
  - frontier (geojson): "./data/frontier.geojson"
  - adjacency (geojson): "./data/adjacency.geojson"
Layers (ordered top→bottom render logic):
  1) background                — solid background
  2) basemap                   — raster (optional)
  3) units-fill                — fill by status (fallback gray)
  4) units-outline             — thin border
  5) frontier-mediation        — semi-transparent overlay for mediation=true
  6) frontier-enclave-outline  — dashed outline for enclave=true
  7) frontier-override-protected — dashed/purple line for protected_blocked=true
  8) adjacency-land            — gray solid
  9) adjacency-bridge          — dashed darker line
 10) adjacency-water           — light blue line (low opacity)
 11) unit-labels               — symbol layer (name/id)

7) Algorithm Outline (implementation plan)
Header
- Set version=8; point sprite/glyphs to relative paths (no absolute/remote URLs).
Sources
- Define 3 GeoJSON sources (units/frontier/adjacency); 1 optional raster source (basemap).
- Ensure all “data” and “tiles” paths are relative (“./…”).
Layers (with canonical IDs + paints)
- background: "#eef2f5".
- basemap: show at low zooms (0–6); remove entirely if raster tiles aren’t shipped.
- units-fill: `fill-color` by ["match", ["get","status"], ...] with fallback neutral; `fill-opacity` ~0.6.
- units-outline: thin line "#607D8B".
- frontier flags:
  • mediation: fill overlay (e.g., soft orange) with opacity ~0.5 where mediation=true.
  • enclave: line with short dash (visual cue).
  • protected_blocked: dashed purple line (2px) to indicate blocked change.
- adjacency-* by edge_type: land (solid gray), bridge (dashed dark), water (light blue, semi-opaque).
- unit-labels: `text-field` ["coalesce", name, id]; fonts ["Inter Regular","Noto Sans Regular"]; halo to improve contrast.
Color/IDs
- Keep IDs exactly as listed in §6 to simplify UI lookups.
- Colors can be tuned later; keep distinct hues for statuses and flags.

8) State Flow
UI loads style (offline) → sets/updates source data (setData for frontier as runs change) → layers reflect status/flags without network. No map logic lives in style.

9) Determinism & Numeric Rules
- No http(s) URLs; only relative paths.
- No conditional style expressions that fetch remote data.
- Same bytes in style + same GeoJSON ⇒ identical rendering across OS.
- Avoid environment-influenced template values (timestamps, locales).

10) Edge Cases & Failure Policy
- Missing basemap tiles: remove basemap source/layer; units/frontier still render.
- Missing glyphs/sprite: labels/symbols may fail; keep fallback fonts list short; ship local glyphs.
- Properties absent: `match` must include a default color; boolean-flag layers should simply not render where keys missing.
- Very small polygons: labels may overlap; leave to UI zoom constraints (out of scope).
- Large datasets: performance tuned by keeping paints simple (no heavy data-driven stops).

11) Test Checklist (must pass)
- Lints/loads with MapLibre GL (v8 spec).
- All paths are relative; no network requests observed.
- With sample frontier.geojson: 
  • status domains map to distinct fills,
  • mediation/enclave/protected flags render as overlays/lines,
  • adjacency types draw with correct style.
- Removing basemap still produces a clean thematic map.
- Works identically on Win/macOS/Linux when bundled by Tauri.
```
