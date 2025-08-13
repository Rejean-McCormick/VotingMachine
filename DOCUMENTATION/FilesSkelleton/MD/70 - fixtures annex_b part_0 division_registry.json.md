````md
Pre-Coding Essentials (Component: fixtures/annex_b/part_0/division_registry.json, Version/FormulaID: VM-ENGINE v0) — 70/89

1) Goal & Success
Goal: Ship the canonical DivisionRegistry fixture for Part-0 tests: a versioned, single-root unit tree (plus optional adjacency) with required provenance and baseline fields.
Success: Parses & schema-validates; exactly one root, no cycles; magnitudes/rolls in range; adjacency references known units only; bytes are canonicalizable (UTF-8, LF, sorted keys) so downstream hashes are stable across OS/arch.

2) Scope
In scope: Registry identity & provenance; Units[] (tree, magnitudes, rolls, optional flags/tags); optional Adjacency[] for frontier/contiguity tests.
Out of scope: Options and tallies (separate fixtures), reporting content, algorithm math.

3) Inputs → Outputs
Input: `fixtures/annex_b/part_0/division_registry.json` (local file).
Output (LOAD stage): `DivisionRegistry` + `Units` + optional `Adjacency` inside `LoadedContext`. Used by VALIDATE (tree, magnitudes, data presence) and by later gates/frontier steps.

4) Entities/Tables (shape)
Canonical JSON object (no extra fields):
```json
{
  "id": "REG:part0",
  "schema_version": "1",
  "provenance": {
    "source": "string",              // publisher or dataset name
    "published_date": "YYYY-MM-DD"   // ISO date
  },
  "units": [
    {
      "unit_id": "U:root",           // unique, canonical string
      "parent_id": null,             // null for the single root
      "name": "Country",
      "magnitude": 1,                // u32 ≥ 1
      "eligible_roll": 12345,        // u64 ≥ 0
      "protected_area": false,       // optional, default false
      "tags": []                     // optional array<string>
    }
    // children: parent_id = some "U:*"
  ],
  "adjacency": [
    {
      "a": "U:child1",
      "b": "U:child2",
      "type": "land"                 // enum: land | bridge | water
      // optional corridor flag if needed by tests:
      // "corridor": false
    }
  ]
}
````

5. Variables (fields & domains enforced by schema/validate)

* `id`: `RegId` string; stable across fixtures.
* `schema_version`: "1".
* `provenance.source`: non-empty string; `provenance.published_date`: ISO date.
* `units[*].unit_id`: unique; exactly one `parent_id=null` (root); others reference existing unit IDs.
* `units[*].magnitude` ≥ 1 (WTA configs elsewhere also require m=1).
* `units[*].eligible_roll` ≥ 0.
* `units[*].protected_area`: optional bool (default false).
* `adjacency[*].a`/`b`: existing unit IDs; `type` ∈ {"land","bridge","water"}; optional `corridor` bool (default false).

6. Functions
   N/A (fixture). Engine loads this via vm\_io → `loader::load_registry()` and validates via pipeline VALIDATE.

7. Algorithm Outline (how the engine consumes it)

* LOAD: parse JSON → typed structs; normalize Units by `UnitId` (stable order).
* VALIDATE:

  * Tree checks: exactly one root, no cycles.
  * Magnitudes: all ≥ 1.
  * Rolls: `eligible_roll` present and ≥ ballots\_cast when quorum enabled (cross-checked later with tallies).
  * Adjacency: all endpoints exist; `type` in domain.
  * If weighting method = `population_baseline`, baselines must be present (not typical for Part-0 unless tests require).
* Later:

  * Gates use `eligible_roll` to compute turnout.
  * Frontier (if enabled in other tests) uses `adjacency` and `protected_area`.

8. State Flow
   `division_registry.json` → LOAD → VALIDATE → (if pass) TABULATE … GATES/FRONTIER. On validation fail, run is marked **Invalid**; pipeline still packages Result/RunRecord with reasons.

9. Determinism & Numeric Rules

* Engine will re-iterate Units by `UnitId` and use BTreeMaps for stable ordering.
* JSON must be canonicalizable (UTF-8, LF, sorted keys); hashing is over canonical bytes.
* All counts are integers; no floating-point inside the registry.

10. Edge Cases & Failure Policy

* Multiple roots / no root / cycle ⇒ validation error.
* `magnitude < 1` ⇒ error.
* Unknown `adjacency.type` or edges referencing unknown units ⇒ error.
* Quorum enabled but missing/zero `eligible_roll` where ballots exist ⇒ validation error.
* If frontier tests are run: missing adjacency while frontier mode demands contiguity ⇒ validation error in that scenario.

11. Test Checklist (must pass)

* Schema validation succeeds; no additionalProperties.
* Tree invariants: single root, acyclic; all `parent_id` valid.
* Magnitudes ≥ 1 everywhere.
* Adjacency endpoints exist; `type` ∈ {land, bridge, water}.
* Canonicalization: serializing this JSON (after key shuffling) yields identical canonical bytes & SHA-256 across OS/arch.

```
```
