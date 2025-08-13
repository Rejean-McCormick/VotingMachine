<!-- Converted from: 54 - crates vm_pipeline src apply_rules.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.969059Z -->

```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/apply_rules.rs, Version/FormulaID: VM-ENGINE v0) — 55/89
1) Goal & Success
Goal: Evaluate legitimacy gates in fixed order—Quorum → Majority/Supermajority → Double-majority → Symmetry—and produce a LegitimacyReport with Pass/Fail and reasons.
Success: If any gate fails, mark run Invalid and instruct pipeline to skip MAP_FRONTIER. Denominators follow Doc 4 rules (approval → approval rate).
2) Scope
In scope: Compute national turnout; apply national/optional per-unit quorum; compute national support %; compute affected-region family support; evaluate symmetry & exceptions; assemble LegitimacyReport.
Out of scope: Frontier mapping, tie resolution, labeling (later stages).
3) Inputs → Outputs (with schemas/IDs)
Inputs: AggregateResults (country + region-level if needed), ParameterSet (VM-VARs), optional per-Unit turnout flags.
Output: LegitimacyReport with Quorum/Majority/Double-majority/Symmetry sections, raw values & thresholds, and overall Pass/Fail.
4) Entities/Tables (minimal)
5) Variables (used here)
6) Functions (signatures only)
rust
CopyEdit
pub struct LegitimacyReport {
pub pass: bool,
pub reasons: Vec<String>,
pub quorum: GateOutcome,          // {turnout_pct, threshold, pass, per_unit_flags?}
pub majority: GateOutcome,        // {support_pct, threshold, denom_policy, pass}
pub double_majority: Option<GateOutcome2>, // {national_pct, family_pct, thresholds, pass, family_members[]}
pub symmetry: Option<SymmetryOutcome>,     // {respected: bool, exceptions?: Vec<Exception>}
}

pub fn apply_decision_rules(agg: &AggregatesView, p: &Params, per_unit_turnout: Option<&PerUnitTurnout>)
-> LegitimacyReport;

// helpers (pure):
fn compute_turnout_pct(agg: &AggregatesView) -> Ratio;
fn compute_support_pct(agg: &AggregatesView, p: &Params) -> (Ratio, DenomPolicy);
fn compute_family_support(agg: &AggregatesView, p: &Params, scope: QuorumScope) -> (Ratio, Vec<UnitId>);
fn evaluate_symmetry(p: &Params) -> SymmetryOutcome;

(Fields follow Doc 5A §3.5 outline.)
7) Algorithm Outline
Quorum: turnout Σ ballots_cast / Σ eligible_roll (integer/rational math). Pass iff ≥ VM-VAR-020. If VM-VAR-021 > 0, compute per-Unit flags and note 021_scope effect.
Majority: compute national support %. Default denominator = valid ballots; if VM-VAR-007=on include blanks; approval ballots use approval rate. Pass iff ≥ VM-VAR-022.
Double-majority: if VM-VAR-024=on, require national ≥ VM-VAR-022 and family ≥ VM-VAR-023. Determine family via VM-VAR-026/027; if frontier_mode=none, enforce by_list/by_tag. Respect 021_scope when excluding failing Units from family if configured.
Symmetry: if VM-VAR-025=on, verify thresholds/denominators are neutral in both directions; if VM-VAR-029 non-empty, mark “Not respected” and record rationale list.
Outcome: Build LegitimacyReport with raw numbers, thresholds, denominators, family members, and Pass/Fail. If any Fail, signal Invalid and skip MAP_FRONTIER.
8) State Flow
AGGREGATE → APPLY_DECISION_RULES → (if Pass) MAP_FRONTIER; else skip → RESOLVE_TIES (only if blocking) → LABEL.
9) Determinism & Numeric Rules
Exact integer/rational comparisons; ≥ rules for thresholds; half-even rounding only where comparisons require (none for simple ratio compare). Offline; stable orders unaffected here.
10) Edge Cases & Failure Policy
Exact threshold (e.g., 55.000% vs 55) → Pass.
Include blanks affects gates only; tabulation/allocation remain on valid ballots.
Missing eligible_roll when quorum required → should have failed VALIDATE; here, just produce Fail/Invalid per policy.
Do not throw on gate failures; return pass=false with reasons.
11) Test Checklist (must pass)
VM-TST-004: approval, support = 55.0% vs threshold 55 ⇒ Pass (≥); label later Decisive.
VM-TST-005: national turnout 48% vs 50% ⇒ Fail Quorum → Invalid; frontier omitted.
Weighting flip scenarios still compute support using the correct denominators (approval rate) regardless of aggregation method.
```
