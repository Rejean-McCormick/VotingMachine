<!-- Converted from: 70 - fixtures annex_b part_0 division_registry.json.docx on 2025-08-12T18:20:47.459474Z -->

```
Lean pre-coding sheet — 70/89
Component: fixtures/annex_b/part_0/division_registry.json (Part 0 fixtures)
 Version/FormulaID: Registry content is data (not in FID); FID covers rule primitives only.
1) Goal & success
Goal: Provide the canonical DivisionRegistry for Part 0: a versioned unit tree (plus optional adjacency) with required provenance and baseline fields.
Success: Loads and validates; exactly one root, no cycles; fields present and in-range; determinism preserved via canonical JSON (UTF-8, sorted keys, LF).
2) Scope
In scope: id, provenance{source,published_date}, Units[] with locked fields, optional **Adjacency[]`.
Out of scope: Options and tallies (separate fixtures), report rendering.
3) Inputs → outputs
Input artifact: fixtures/annex_b/part_0/division_registry.json.
Pipeline output usage: Appears in LoadedContext (Registry, Units, Adjacency) at LOAD; then checked at VALIDATE before any math.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
N/A (fixture only).
7) Algorithm outline (how it’s consumed)
LOAD reads Registry (units+adjacency) into LoadedContext.
VALIDATE enforces: tree with one root; magnitude≥1; roll/baseline requirements; adjacency type domain; WTA ⇒ magnitude=1.
8) State flow (very short)
Used at LOAD → VALIDATE; on success the pipeline proceeds; on failure, run is marked Invalid and later stages are skipped per rules.
9) Determinism & numeric rules
Ordering: stable orders (Units by Unit ID) before any hashing/serialization; JSON canonicalization (UTF-8, sorted keys, LF).
Counts are integers; no float equality; presentation rounding occurs in reports, not here.
10) Edge cases & failure policy
Missing or multiple roots; cycles; magnitude<1; negative rolls; population weighting selected but missing/zero baselines; adjacency referencing unknown units or unknown type. Reject at VALIDATE with clear errors.
11) Test checklist (must pass)
Schema validates; hierarchy and magnitude constraints pass; if VM-VAR-030=population_baseline, all aggregated Units have positive baselines and a year. Determinism is indirect (same input ⇒
```
