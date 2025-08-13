<!-- Converted from: 60 - crates vm_report Cargo.toml, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.138189Z -->

```toml
Pre-Coding Essentials (Component: crates/vm_report/Cargo.toml, Version/FormulaID: VM-ENGINE v0) — 61/89
1) Goal & Success
Goal: Declare a reporting crate that renders reports offline, with deterministic builds, and outputs matching Doc 7 specs (one-decimal %; structure fixed).
Success: Builds with workspace lock; no network/runtime assets; provides opt-in features for JSON and HTML/PDF render paths required by Doc 7.
2) Scope
In scope: Crate metadata; feature flags (e.g., render_json, render_html); dependencies only for deterministic, offline rendering; test profile hooks.
Out of scope: Report structure/content (lives in code & templates per Doc 7), pipeline objects.
3) Inputs → Outputs (with schemas/IDs)
Inputs: Workspace toolchain & lock; vm_report::structure and render modules consume Result, RunRecord, optional FrontierMap.
Outputs: Build artifacts for report renderers (JSON/HTML), which must show one-decimal percentages and include mandated sections/footers.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
(manifest has no functions)
7) Algorithm Outline (build configuration)
Define crate with edition + resolver v2 (inherits workspace).
Add render_json feature: depends on serialization stack only (no net). Output must bind to Result/RunRecord/FrontierMap fields.
Add render_html feature: includes template engine/assets bundled locally; forbid remote fonts/tiles/scripts.
Ensure one-decimal formatting helpers are part of the crate (exposed for both renderers).
Profiles: deterministic release (inherits workspace settings).
8) State Flow (very short)
Used by vm_report library to render §1–§10 sections fixed by Doc 7; consumes Result/RunRecord/FrontierMap only.
9) Determinism & Numeric Rules
Offline only; no external assets; percent formatting at one decimal; integers for seats.
10) Edge Cases & Failure Policy
If render_html templates missing external assets, do not fetch—fail build; templates must be bundled.
Internationalization: if bilingual is enabled in code, crate must ship mirrored templates; never mix languages in paragraphs.
11) Test Checklist (must pass)
JSON/HTML feature builds succeed under --locked; no network during render.
Renderers show one-decimal percentages; footer IDs sourced from RunRecord/Result.
```
