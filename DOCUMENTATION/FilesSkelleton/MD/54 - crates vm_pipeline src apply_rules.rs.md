```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/apply_rules.rs, Version/FormulaID: VM-ENGINE v0) — 54/89

1) Goal & Success
Goal: Evaluate the legitimacy gates in fixed order — Quorum → Majority/Supermajority → (optional) Double-majority → (optional) Symmetry — and return a deterministic LegitimacyReport. Outcomes use exact integer/rational comparisons; approval support uses approval rate / valid_ballots as per Doc 1B.
Success: If any gate fails, the pipeline marks the run Invalid and skips MAP_FRONTIER. Values, thresholds, and booleans are captured for packaging; no RNG here.

2) Scope
In scope: Gate math only (no frontier topology), exact denominators (valid vs. include blanks), affected-family resolution, and symmetry checks; build a self-contained report structure consumed by LABEL and packaging.
Out of scope: Frontier mapping, tie resolution, result/run_record packaging (other stages).

3) Inputs → Outputs
Inputs:
• AggregatesView (country + optional regional views with: ballots_cast, invalid_or_blank, valid_ballots, eligible_roll, approvals_for_change if approval).
• Params (VM-VARs used: 007, 020–023, 024–027, 029).
• Optional per-unit turnout map to annotate per-unit quorum flags.
Outputs:
• LegitimacyReport { pass, reasons[], quorum, majority, double_majority?, symmetry? } — stable, reproducible content for subsequent stages.

4) Data Types (minimal)
use std::collections::{BTreeMap, BTreeSet};
use vm_core::{
  ids::UnitId,
  rounding::{Ratio, cmp_ratio_half_even, ge_percent, ge_percent_half_even},
  variables::{Params},
};

/// Minimal aggregate view consumed by gate math.
pub struct AggregateRow {
  pub ballots_cast: u64,
  pub invalid_or_blank: u64,
  pub valid_ballots: u64,
  pub eligible_roll: u64,
  pub approvals_for_change: Option<u64>, // present for approval ballots
}
pub struct AggregatesView {
  pub national: AggregateRow,
  pub by_region: BTreeMap<UnitId, AggregateRow>, // empty if not needed
}

#[derive(Clone, Copy, Debug)]
pub enum DenomPolicy {
  ValidBallots,          // default for majority/supermajority
  ValidPlusBlank,        // when VM-VAR-007 = on (gates only)
  ApprovalRateValid,     // approval support = approvals_for_change / valid_ballots
}

pub struct GateOutcome {
  pub observed: Ratio,     // internal exact value; presentation happens later
  pub threshold_pct: u8,   // integer percent
  pub pass: bool,
}
pub struct DoubleOutcome {
  pub national: GateOutcome,
  pub family: GateOutcome,
  pub pass: bool,
  pub members: Vec<UnitId>, // affected family (canonical order)
}
pub struct SymmetryOutcome {
  pub respected: bool,
  pub exceptions: Vec<String>, // codes taken from VM-VAR-029 if any
}
pub struct QuorumDetail {
  pub national: GateOutcome,                    // turnout Σ ballots_cast / Σ eligible_roll
  pub per_unit_flags: Option<BTreeMap<UnitId, bool>>, // per-unit turnout pass/fail if configured
}
pub struct LegitimacyReport {
  pub pass: bool,
  pub reasons: Vec<String>, // stable machine-readable strings
  pub quorum: QuorumDetail,
  pub majority: GateOutcome,
  pub double_majority: Option<DoubleOutcome>,
  pub symmetry: Option<SymmetryOutcome>,
}

5) Public API (signatures only)
pub fn apply_decision_rules(
  agg: &AggregatesView,
  p: &Params,
  per_unit_turnout: Option<&BTreeMap<UnitId, AggregateRow>>, // when per-unit quorum annotated separately
) -> LegitimacyReport;

// helpers (pure, deterministic)
fn turnout_ratio(row: &AggregateRow) -> Ratio;                       // ballots_cast / eligible_roll (den>0 expected)
fn support_ratio_national(agg: &AggregatesView, p: &Params) -> (Ratio, DenomPolicy);
fn family_units(agg: &AggregatesView, p: &Params) -> Vec<UnitId>;    // resolves VM-VAR-026/027; canonical order
fn support_ratio_family(
  agg: &AggregatesView,
  members: &[UnitId],
  p: &Params
) -> (Ratio, DenomPolicy);
fn eval_quorum(
  agg: &AggregatesView,
  p: &Params,
  per_unit_turnout: Option<&BTreeMap<UnitId, AggregateRow>>
) -> (QuorumDetail, bool /*pass*/);
fn eval_majority(agg: &AggregatesView, p: &Params) -> (GateOutcome, bool /*pass*/);
fn eval_double_majority(
  agg: &AggregatesView,
  p: &Params
) -> (Option<DoubleOutcome>, bool /*pass or true if not enabled*/);
fn eval_symmetry(p: &Params) -> Option<SymmetryOutcome>;

6) Algorithm Outline
Order is fixed and short-circuits on failure for the frontier step (but still records all computed parts available):
A) Quorum
• national_ratio = ballots_cast / eligible_roll (Σ national). pass_national = ge_percent(national_ratio.num, national_ratio.den, p.quorum_global_pct()).
• If per-unit quorum configured (>0): for each unit compute ballots_cast/eligible_roll and store pass/fail. The scope/impact on family is handled in double-majority helper (see §C).
B) Majority / Supermajority (national)
• Determine DenomPolicy:
  – Approval ballots: ApprovalRateValid, numerator = approvals_for_change, denominator = valid_ballots.
  – Else: ValidBallots by default; if VM-VAR-007 = on then ValidPlusBlank (valid + invalid_or_blank).
• Compute observed ratio accordingly. pass_majority = ge_percent(observed.num, observed.den, p.national_majority_pct()).
C) Double-majority (optional)
• If VM-VAR-024 = on:
  – Resolve affected family via VM-VAR-026/027 to a canonical Vec<UnitId>. If unresolved/empty → pass=false with reason.
  – If per-unit quorum scope excludes failing units from family, drop those members here (policy from VM-VAR-021 scope).
  – Compute family support ratio using the same DenomPolicy as national (including blanks only if VM-VAR-007 = on).
  – Pass iff national_pass && ge_percent(family_ratio, p.regional_majority_pct()).
D) Symmetry (optional)
• If VM-VAR-025 = on: check that the chosen denominators/thresholds are direction-neutral (as specified by the spec). If VM-VAR-029 exceptions present, respected=false and exceptions echoed; otherwise respected=true.

Assemble LegitimacyReport:
• pass = quorum.pass && majority.pass && (double_majority.pass if enabled) && (symmetry.respected if enabled).
• reasons: stable codes like "Quorum.NationalBelowThreshold", "Majority.BelowThreshold", "DoubleMajority.FamilyUnresolved", "Symmetry.ExceptionsPresent".

7) State Flow
AGGREGATE → **APPLY_DECISION_RULES (this file)** → if pass { MAP_FRONTIER } else { skip frontier } → RESOLVE_TIES (only if blocking later) → LABEL → BUILD_RESULT / BUILD_RUN_RECORD.

8) Determinism & Numeric Rules
• Pure integer/rational math; no floats.
• Threshold comparisons use exact ratios (ge_percent / ge_percent_half_even where the spec requires banker’s rule — currently not required here; majority/quorum use ≥ exact percent).
• No RNG.
• Family member list is sorted deterministically (UnitId lexicographic) for stable hashing later.

9) Edge Cases & Failure Policy
• eligible_roll = 0 anywhere → that unit’s turnout ratio is 0/1 for gating; national eligible_roll=0 ⇒ quorum national fails.
• approvals_for_change missing for approval ballots ⇒ treat as 0 (loader/validate should ensure presence; here remain defensive).
• Empty/invalid family when VM-VAR-024=on ⇒ DoubleMajority fail with reason.
• Per-unit quorum scope:
  – If configured to exclude failing units from family, drop them when computing family support; otherwise include but mark flags.
• Do not panic on missing data; prefer pass=false with precise reason(s). Validation stage should already flag structural issues.

10) Test Checklist (must pass)
• Exact boundaries: observed == threshold passes for quorum and majority.
• Approval mode: observed = approvals_for_change / valid_ballots; toggling VM-VAR-007 has **no** effect on approval denominator.
• Per-unit quorum flags computed and, when scope excludes, family support changes accordingly.
• Double-majority off: report has None for double_majority and overall pass reflects quorum & majority only.
• Symmetry on with exceptions list populated ⇒ respected=false and reason recorded; off ⇒ symmetry=None.
• Determinism: permuting by_region order or family input order yields identical LegitimacyReport bytes after canonical serialization.

11) Notes for Packaging
• Result serialization (later stage) will emit gate observed values as JSON numbers (engine precision), not {num,den}; keep Ratios internal and convert at packaging time.
• RunRecord stores formula_id and engine identifiers; gate outcomes themselves live in Result; tie events (if any) are recorded in RunRecord, but there are none in this stage.
```
