````md
Pre-Coding Essentials (Component: fixtures/annex_b/part_0/parameter_set.json, Version/FormulaID: VM-ENGINE v0) — 69/89

1) Goal & Success
Goal: Provide the canonical Part-0 ParameterSet fixture (a frozen VM-VAR map) that drives the engine’s behavior in tests.
Success: Loads under the Part-0 schema, produces byte-identical pipeline outputs across OS/arch when paired with the matching Registry/Tallies; JSON is canonicalizable (UTF-8, LF, sorted keys).

2) Scope
In scope: One JSON file defining engine variables (VM-VAR-###) and fixed defaults for Part-0 runs.
Out of scope: Any run-specific inputs or external data; algorithm math; report rendering.

3) Inputs → Outputs
Input (path): `fixtures/annex_b/part_0/parameter_set.json`
Output (loader): typed `Params` (with a stable `ParamSetId`) consumed by TABULATE/ALLOCATE/AGGREGATE/GATES.

4) Entities/Tables (shape)
JSON object (no extraneous fields):
```json
{
  "id": "PS:part0",                 // stable, referenced in artifacts
  "schema_version": "1",            // per Annex B Part 0
  "variables": {
    "VM-VAR-001": "approval",       // ballot_type
    "VM-VAR-007": "off",            // include_blank_in_denominator
    "VM-VAR-010": "proportional_favor_small",
    "VM-VAR-011": "on",
    "VM-VAR-012": 0,
    "VM-VAR-020": 50,
    "VM-VAR-021": 0,                // or integer % with scope sibling
    "VM-VAR-021_scope": "frontier_only",
    "VM-VAR-022": 55,
    "VM-VAR-023": 55,
    "VM-VAR-024": "off",
    "VM-VAR-025": "off",
    "VM-VAR-028": "residents_only",
    "VM-VAR-030": "equal_unit",
    "VM-VAR-031": "country",
    "VM-VAR-032": "status_quo",
    "VM-VAR-033": 0,
    "VM-VAR-040": "none"
  }
}
````

(Values above are illustrative Part-0-style defaults; the fixture must choose a coherent set.)

5. Variables (domains to enforce)

* Ballot/Tabulation: **001** ∈ {plurality, approval, score, ranked\_irv, ranked\_condorcet}; **007** ∈ {on, off}.
* Allocation: **010** ∈ {winner\_take\_all, proportional\_favor\_big, proportional\_favor\_small, largest\_remainder, mixed\_local\_correction}; **011** ∈ {on, off}; **012** ∈ integer % \[0..10].
* Gates: **020** (quorum\_global\_pct) ∈ \[0..100]; **021** (per-unit quorum) ∈ \[0..100], with **021\_scope** ∈ {frontier\_only, frontier\_and\_family} if 021>0; **022**/**023** ∈ \[50..75]; **024**, **025** ∈ {on, off}.
* Rolls/Weighting/Aggregation: **028** ∈ {residents\_only, residents\_plus\_displaced, custom\:list}; **030** ∈ {equal\_unit, population\_baseline}; **031** = country (v1).
* Frontier: **040** ∈ {none, sliding\_scale, autonomy\_ladder} (Part-0 typically “none”).
* Ties/RNG: **032** ∈ {status\_quo, deterministic, random}; **033** ∈ integer ≥ 0 (used only if 032=random).

6. Functions
   N/A (fixture). Engine loads via vm\_io → `Params`.

7. Algorithm Outline (how the engine consumes values)

* Ballot & denominators: **001** selects tabulation; approval gate always uses approval rate = approvals\_for\_change / valid\_ballots; **007** can widen **gate** denominators only.
* Allocation: **010** picks family (WTA/PR/LR/MMP); **012** filters below-threshold options; **011** must be “on” in v1 (use unit magnitudes).
* Aggregation: **030** chooses weighting; **031** fixes aggregate level to country.
* Gates: quorum **020/021**, majority **022**, regional majority **023**, double-majority **024**, symmetry **025**; scope **021\_scope** affects family inclusion/exclusion (not tabulation).

8. State Flow
   LOAD (parse & validate) → VALIDATE (domain & coherence) → consumed in TABULATE → ALLOCATE → AGGREGATE → APPLY\_DECISION\_RULES → (optional) MAP\_FRONTIER → …

9. Determinism & Numeric Rules

* Integers/rationals only; half-even rounding used only where the spec allows (gates/reporting).
* Canonicalization: JSON must serialize deterministically (sorted keys, UTF-8, LF) for hashing.

10. Edge Cases & Failure Policy

* Unknown VM-VAR key or out-of-domain value ⇒ validation issue.
* **010=winner\_take\_all** with any Unit.magnitude≠1 ⇒ configuration error.
* **030=population\_baseline** without baselines in Registry ⇒ validation error.
* **024=on** with unresolved/empty family (when required) ⇒ validation error.
* **032=random** without a numeric **033** seed ⇒ validation error (no OS RNG fallback).

11. Test Checklist (must pass)

* Schema validation passes; keys/values in range; no extra fields.
* Loading this PS with Part-0 Registry/Tallies yields identical Result/RunRecord hashes across OS (canonical JSON).
* Approval ballot renders the mandatory “approval-rate denominator” sentence in reports.
* Threshold & gate cutoffs behave with ≥ semantics (e.g., 55.0% vs 55 passes).

```
```
