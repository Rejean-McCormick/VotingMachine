<!-- Converted from: 69 - fixtures annex_b part_0 parameter_set.json.docx on 2025-08-12T18:20:47.425061Z -->

```
Lean pre-coding sheet — 69/89
Component: fixtures/annex_b/part_0/parameter_set.json (Part 0 fixtures)
 Version/FormulaID: per Annex A; does not encode run-time values (FID covers rule primitives, not specific runs).
1) Goal & success
Goal: Provide the canonical ParameterSet fixture for Part 0 tests: a frozen map of VM-VAR-### → value used by the engine; acts as the single source of truth for thresholds, allocation family, weighting, and operational defaults.
Success: Loads under the Part 0 schema and yields deterministic runs when combined with the matching Registry/Tallies (JSON canonicalization, sorted keys, LF, UTC).
2) Scope
In scope: Structure of one Part 0 ParameterSet fixture; allowed variables and default domains.
Out of scope: Engine FID contents (covered by Annex A); algorithm details (Doc 4); report rendering (Doc 7).
3) Inputs → outputs
Inputs (loader): JSON file at fixtures/annex_b/part_0/parameter_set.json.
Outputs: In LoadedContext, a frozen ParameterSet (PS:… id, vars map); values feed TABULATE/ALLOCATE/AGGREGATE/GATES per Doc 4.
4) Entities/Tables (minimal)
5) Variables (used here)
Use only variables defined in Doc 2; percentages are integer %. Baseline set for Part 0 small tests typically includes:
 001, 007, 010–012, 020–025, 030–031, 040 (if frontier later), 050–052 (ties/RNG when needed). Defaults and domains per tables.
6) Functions (signatures only)
N/A (fixture only).
7) Algorithm outline (how values are consumed)
Ballot & denominators: VM-VAR-001 selects tabulation family; approval gate uses approval rate = approvals_for_change / valid_ballots (fixed). VM-VAR-007 may widen gate denominator only.
Allocation: VM-VAR-010 selects WTA/PR/LR/MMP; VM-VAR-012 PR entry threshold; constraint: if WTA then Unit.magnitude=1.
Aggregation: VM-VAR-030 weighting (population_baseline vs equal_unit); VM-VAR-031 aggregate level = country.
Gates: quorum/majority/double-majority/symmetry: VM-VAR-020..025.
8) State flow (very short)
Use in pipeline: Loaded at LOAD, validated at VALIDATE, applied through TABULATE → … → LABEL exactly per step order.
9) Determinism & numeric rules
Integers/rationals; no float equality. Presentation rounding only in reports; internal comparisons use round half to even.
Canonicalization: UTF-8, sorted JSON keys, LF, UTC timestamps (affects later hashing).
10) Edge cases & failure policy
Reject if any VM-VAR is unknown, out of domain, or violates dependencies (e.g., WTA with m>1; population weighting without baselines; DM on with bad family mode).
Approvals/score/ranked: ensure downstream rules read the correct natural denominators; gates use the fixed approval rule.
11) Test checklist (must pass)
Schema-validate as a ParameterSet; all ids/values in range.
Engine run using this PS with Part 0 tallies/registry yields identical Result/RunRecord across OS/arch (determinism).
Approval gate sentence enforced when ballot_type=approval.
```
