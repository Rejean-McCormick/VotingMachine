<!-- Converted from: 16 - schemas parameter_set.schema.json, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:45.974469Z -->

```
Pre-Coding Essentials (Component: schemas/parameter_set.schema.json, Version/FormulaID: VM-ENGINE v0) — 16/89
1) Goal & Success
Goal: JSON Schema for ParameterSet capturing the full, immutable snapshot of VM variables (VM-VAR-###) used for a run.
Success: Validates PS: ID, enforces domains/default shapes for all outcome-affecting variables (Docs 2A…2C + Annex A), and is strict (additionalProperties: false). Loader can build a typed map with zero ambiguity.
2) Scope
In scope: Top-level metadata, variables{VM-VAR-###: value}, enums/ranges for ballot/allocation/gates/weighting/frontier/ties/MMP, optional notes.
Out of scope: Derivations (labels, results), cross-entity checks (done in pipeline), Formula ID computation (Annex A handles that).
3) Inputs → Outputs
Inputs: parameter_set.json.
Outputs: Pass/fail against schema; on pass, a frozen ParameterSet object used by all pipeline stages and echoed into RunRecord.
4) Entities/Fields (schema shape to encode)
Root object
id (required, string) — PS:<name>:v<semver>
variables (required, object) — keys VM-VAR-### with constrained values (see §5)
notes (optional, string)
SemVer pattern: ^v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:[-+][A-Za-z0-9.-]+)?$
5) Variables (domains to encode in schema)
All percentages are integers 0..100. Booleans are modeled as enums on|off to keep serialization explicit.
A) Ballot (001–007)
B) Allocation & MMP (010–017)
C) Gates / Thresholds / Families (020–029)
D) Aggregation / Weighting (030–031)
E) Frontier & Contiguity (040–048)
F) Ties & RNG (050–052)
G) Labels (060–062) (if used)
Optional integers % 0..100 for any label margin thresholds the report might show. If absent, engine uses built-in defaults.
H) Executive toggle (073) (if used)
073 executive_enabled — enum: on off.
Conditional rules to encode with if/then/else:
If ballot_type="score" ⇒ require 002/003; allow 004.
If ballot_type="ranked_condorcet" ⇒ require 005.
If ballot_type="ranked_irv" ⇒ 006 must equal reduce_continuing_denominator.
If allocation_method="mixed_local_correction" ⇒ require 013–017.
If double_majority_enabled="on" ⇒ require 022, 023, 026; if 026=by_list ⇒ require 027 non-empty.
If weighting_method="population_baseline" ⇒ pipeline will require Unit baseline fields (schema note only).
If tie_policy="random" ⇒ require 052 (64-hex).
6) Functions
(Schema only.)
7) Algorithm Outline (schema authoring steps)
$schema: JSON Schema 2020-12; set $id.
$defs for: Percent, BoolEnum, Hex64, SemVer, enums for each family (ballot, allocation, etc.), and the VariableMap where each VM-VAR-### has its own subschema.
Root: type: object, required: ["id","variables"], additionalProperties: false.
id: pattern for PS:<name>:v<semver>.
variables: type: object, additionalProperties: false, with explicit properties for each VM-VAR-### listed above.
Encode cross-field conditionals with allOf blocks (e.g., seed required when ties are random).
Include non-normative $comment about canonical JSON (UTF-8, LF, sorted keys) and that unspecified variables use doc defaults (but recommend explicit inclusion for reproducibility).
8) State Flow
Loader validates → builds typed Params → echoed into RunRecord and referenced by Report. Pipeline reads these to drive Tabulate / Allocate / Aggregate / Gates / Frontier / Ties in fixed order.
9) Determinism & Numeric Rules
All thresholds are integers; no floats.
Approval gate denominator is fixed by spec (valid ballots); schema pins 029 accordingly.
Requiring rng_seed when tie_policy=random ensures reproducible tie outcomes.
10) Edge Cases & Failure Policy
Missing mandatory variables for chosen modes (e.g., MMP without 013–017) ⇒ schema fail.
Invalid hex for rng_seed or wrong length ⇒ schema fail.
Inconsistent frontier bands (overlap, out-of-order) ⇒ pipeline fail (schema checks only bounds/shape).
pr_entry_threshold_pct > 10 ⇒ schema fail (per spec cap).
If double_majority_enabled="on" with empty family when by_list ⇒ schema fail.
11) Test Checklist (must pass)
Happy path: approval + Sainte-Laguë with defaults (001=approval, 010=proportional_favor_small, 012=0, 020=50, 022=55, 023=55, 024=on, 025=on, 030=population_baseline, 031=country, 050=status_quo) → pass.
Score mode: set 001=score, 002=0, 003=5, 004=off → pass; 003 ≤ 002 ⇒ fail.
IRV: 001=ranked_irv, 006 must equal reduce_continuing_denominator; any other value ⇒ fail.
Condorcet: 001=ranked_condorcet, 005 present; missing 005 ⇒ fail.
MMP: 010=mixed_local_correction with 013..017 supplied → pass; omit any ⇒ fail.
Random ties: 050=random with valid 052 (64-hex) → pass; missing or malformed seed ⇒ fail.
Frontier bands: bands with min_pct ≤ max_pct pass; overlapping bands accepted by schema but later fail in pipeline validation.
```
