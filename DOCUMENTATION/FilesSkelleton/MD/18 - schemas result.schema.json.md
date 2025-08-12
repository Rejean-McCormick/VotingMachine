<!-- Converted from: 18 - schemas result.schema.json, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.005062Z -->

```
Pre-Coding Essentials (Component: schemas/result.schema.json, Version/FormulaID: VM-ENGINE v0) — 18/89
1) Goal & Success
Goal: JSON Schema for Result—the computed outcome bundle for a run.
Success: Validates RES: ID; carries input IDs (REG, TLY, PS); includes per-unit blocks, aggregates, legitimacy gates, and the final label (Decisive|Marginal|Invalid). Shapes/fields align with Docs 1/4/5/7; integers/ratios only (percentages are presentation-only).
2) Scope
In scope: Top-level identifiers, per-unit summaries (scores/turnout/allocation/flags), aggregates by level, gate outcomes as exact ratios, final label (+reason), optional frontier_map_id, optional tie_log.
Out of scope: Frontier geometry/content (that’s FrontierMap), provenance timestamps (that’s RunRecord), rendering/rounding (Doc 7 handles presentation).
3) Inputs → Outputs
Inputs (by reference): reg_id (REG:...), ballot_tally_id (TLY:...), parameter_set_id (PS:...).
Output: A single strict Result JSON object; report consumes it (plus optional FrontierMap and RunRecord).
4) Entities/Fields (schema shape to encode)
Root
id (required, string) — RES:<short-hash>
reg_id (required, string) — REG:<...>
ballot_tally_id (required, string) — TLY:<...>
parameter_set_id (required, string) — PS:<...>
label (required, enum) — Decisive | Marginal | Invalid
label_reason (optional, string) — short rationale used in report
aggregates (required, object) — by level
units (required, array) — list of UnitBlock
gates (required, object) — quorum / majority / double-majority / symmetry outcomes
tie_log (optional, array) — entries from tie resolution (if any)
frontier_map_id (optional, string) — FR:<...> when mapping run produced one.
UnitBlock (array items)
unit_id (required, string) — U:REG:...
turnout (required, object) — { ballots_cast:int≥0, invalid_or_blank:int≥0, valid_ballots:int≥0 }
scores (required for non-ranked inputs) — map OPT:... → int≥0 (plurality=votes, approval=approvals, score=score_sum)
allocation (required, object) — map OPT:... → int (seats) or power_pct:int (WTA 100)
flags (required, object) — { unit_data_ok:bool, unit_quorum_met:bool, unit_pr_threshold_met:bool, protected_override_used:bool, mediation_flagged:bool }.
Aggregates (by level)
Object keyed by level (country, region, district used), each with:
totals — map OPT:... → int (seats or votes as applicable)
shares — map OPT:... → ratio{num:int, den:int}
turnout — { ballots_cast:int, invalid_or_blank:int, valid_ballots:int, eligible_roll:int }
weighting_method — echo of VM-VAR-030 for clarity.
Gates (legitimacy outcomes)
quorum — { observed:ratio, threshold_pct:int, pass:bool }
majority — { observed:ratio, threshold_pct:int, pass:bool }
double_majority — { national: {observed:ratio, threshold_pct:int, pass:bool}, regional: {observed:ratio, threshold_pct:int, pass:bool}, pass:bool }
symmetry — { pass:bool }
Ratios are integers only; reporting does the 1-decimal rendering. Approval gate’s observed value is the approval rate (approvals_for_change / valid_ballots).
5) Variables (validators & enums to embed in the schema)
6) Functions
(Schema only.)
7) Algorithm Outline (schema authoring steps)
$schema = JSON Schema 2020-12; set $id.
$defs: ResId, RegId, TlyId, PsId, UnitId, OptId, Ratio, UnitBlock, GateOutcome.
Root object: required = ["id","reg_id","ballot_tally_id","parameter_set_id","label","aggregates","units","gates"], additionalProperties:false.
UnitBlock: strict object; integers ≥0; allocation either seats map or WTA power (choose one via oneOf).
Aggregates: require turnout and either totals or shares (allow both); ratios encoded as {num,den} ints.
Gates: encode shapes above; require integers for thresholds; ratios only.
Optional tie_log array item schema: { context:string, candidates:array<OPT>, policy:enum, order_or_seed:string, winner:OPT }. (Produced only when ties block decisions.)
Non-normative $comment: arrays should be sorted (Units by unit_id; Options by order_index then id)—enforced in code for determinism.
8) State Flow
Populated by BUILD_RESULT after LABEL step; then RunRecord is built pointing to it. Reports read Result (+ optional FrontierMap, RunRecord).
9) Determinism & Numeric Rules
Integers & ratios only; no floats inside Result.
Percentages are derived at report time; round half to even only at defined comparison points; report shows one decimal.
Ordering is stable: Units by Unit ID, Options by order_index then ID; canonical JSON (UTF-8, LF, sorted keys).
10) Edge Cases & Failure Policy
Validation failed earlier ⇒ label="Invalid", gates panel contains N/A/Fail as per report rules; frontier omitted.
Gates failed ⇒ label="Invalid"; frontier omitted.
IRV/Condorcet: carry round logs/pairwise only via audit/TieLog if used; continuing-denominator policy is fixed.
WTA: allocation uses 100% power for the winner; schema must allow either seat map or power%.
11) Test Checklist (must pass)
Minimal Decisive result with one unit, Sainte-Laguë seats in canonical order; label present.
aggregates.turnout.valid_ballots = ballots_cast - invalid_or_blank
```
