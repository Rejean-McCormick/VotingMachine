<!-- Converted from: 10 - LICENSE, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:45.829765Z -->

```
Pre-Coding Essentials (Component: LICENSE, Version/FormulaID: VM-ENGINE v0) — 10/89
1) Goal & Success
Goal: Choose and apply clear, compatible licensing for code, docs/specs, schemas, fixtures, and assets so downstream use is unambiguous and CI can enforce it.
Success: Single LICENSE (and NOTICE if needed) covers code; separate notices clarify docs/data/assets; all files have SPDX headers; cargo-deny (or equivalent) passes.
2) Scope
In scope: Top-level license for code, license statements for Docs 1–7 & Annex A/B, schemas, fixtures, UI assets (tiles/icons), third-party attributions.
Out of scope: Dependency licenses (validated via tooling), contributor agreements.
3) Inputs → Outputs
Inputs: Repository contents (crates/*, schemas/*, fixtures/*, docs/*, UI assets).
Outputs: LICENSE (primary), optional NOTICE, short per-folder LICENSE or COPYING files where license differs (schemas/fixtures/assets).
4) Entities/Tables (minimal)
(Short, keywords only.)
5) Variables
6) Functions
(None.)
7) Algorithm Outline
Pick code license: Dual license Apache-2.0 OR MIT. Write combined text in top-level LICENSE (both), or LICENSE-APACHE + LICENSE-MIT with a short LICENSE pointer.
Docs/specs: Add docs/LICENSE with CC BY 4.0 text; add header note to each Doc 1–7 / Annex A/B.
Schemas: Add schemas/LICENSE (prefer CC0-1.0 for maximal reuse). Include SPDX headers in schema files via comment fields (if allowed) or README note.
Fixtures: Add fixtures/LICENSE (CC0-1.0). Mention that hashes/expected results are non-copyrightable facts.
Assets: Add crates/vm_app/ui/public/LICENSES.md listing each third-party style/font/icon with required attribution and URLs; include any provider NOTICE (e.g., MapLibre style/tiles).
SPDX headers: Add SPDX-License-Identifier to all source files (.rs, .toml, .ts, etc.).
NOTICE (optional): If Apache-2.0 used, create NOTICE summarizing copyrights/trademarks.
CI/license check: Configure cargo-deny (or equivalent) to fail on incompatible deps and verify SPDX headers present.
8) State Flow
Reader sees clear top-level license; subfolders with different terms have explicit LICENSE/LICENSES.md. Build/test includes a license audit step.
9) Determinism & Numeric Rules
N/A. (This file supports policy, not computation.)
10) Edge Cases & Failure Policy
Vendored deps: keep upstream LICENSE files in vendor/.
Map tiles / fonts: ensure redistribution rights; if not redistributable, exclude from repo and document fetch procedure.
Generated outputs: artifacts/ remain unlicensed build products; not committed.
Mixed content: if any doc embeds code, clarify that code snippets are Apache-2.0/MIT while prose is CC BY 4.0.
11) Test Checklist (must pass)
grep finds SPDX headers in all source files.
License audit passes (no copyleft-incompatible crates unless intentionally allowed).
Docs/schemas/fixtures folders contain their LICENSE files.
UI assets have LICENSES.md with attributions; app “About” screen shows required credits.
Packaging (dist/) includes relevant LICENSE/NOTICE files.
```
