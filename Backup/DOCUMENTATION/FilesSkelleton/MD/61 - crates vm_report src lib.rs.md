<!-- Converted from: 61 - crates vm_report src lib.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.167302Z -->

```
Pre-Coding Essentials (Component: crates/vm_report/src/lib.rs, Version/FormulaID: VM-ENGINE v0) — 61/89
Goal & Success
Goal: Provide the high-level reporting API that consumes only Result, optional FrontierMap, and RunRecord, then renders reports that follow Doc 7’s fixed sections, precision, and data sources.
Success: Output respects one-decimal percentages and exact section order; includes the approval-denominator sentence for approval ballots; pulls all fields from the mandated artifacts; echoes tie policy/seed when applicable.
Scope
In scope: Build a ReportModel view from Result/RunRecord/FrontierMap; enforce Doc 7 section order, snapshot contents, and data mapping; handle invalid/gates-fail fallbacks; optional Sensitivity if CompareScenarios exists; formatting helpers (one-decimal).
Out of scope: I/O, file writing, and template assets (handled by renderers). No external data fetch (offline).
Inputs → Outputs (with schemas/IDs)
Inputs: Result (RES:…), optional FrontierMap (FR:…), RunRecord (RUN:…). All content must be derived from these.
Outputs:
ReportModel struct (sections §1–§10) consumed by renderers; footer fields bound to RunRecord/Result.
Public fns return serialized JSON/HTML strings; renderers add no data.
Entities/Tables (minimal)
(N/A – the report model mirrors Doc 7A sections/fields.)
Variables (rendered here; not computed)
Values are displayed from the ParameterSet snapshot embedded via Result/RunRecord. Domains shown for clarity:
VM-VAR-001 ballot_type ∈ {plurality, approval, score, ranked_irv, ranked_condorcet}
VM-VAR-010 allocation_method ∈ {winner_take_all, proportional_favor_big, proportional_favor_small, largest_remainder, mixed_local_correction}
VM-VAR-012 pr_entry_threshold_pct ∈ % 0..10
VM-VAR-020 quorum_global_pct ∈ % 0..100
VM-VAR-021 quorum_per_unit_pct ∈ % 0..100 (+ VM-VAR-021_scope ∈ {frontier_only, frontier_and_family} if set)
VM-VAR-022 national_majority_pct, VM-VAR-023 regional_majority_pct ∈ % 50..75 (default 55)
VM-VAR-024 double_majority_enabled, VM-VAR-025 symmetry_enabled ∈ {on, off}
VM-VAR-028 roll_inclusion_policy ∈ {residents_only, residents_plus_displaced, custom:list}
VM-VAR-030 weighting_method ∈ {equal_unit, population_baseline}
VM-VAR-031 aggregate_level = country (v1 fixed)
VM-VAR-040 frontier_mode ∈ {none, sliding_scale, autonomy_ladder}
VM-VAR-042 frontier_bands (ordered, non-overlapping; structure only)
VM-VAR-047 contiguity_edge_types ⊆ {land, bridge, water}
VM-VAR-048 island_exception_rule ∈ {none, ferry_allowed, corridor_required}
VM-VAR-032 tie_policy ∈ {status_quo, deterministic, random} (echoed in report provenance)
VM-VAR-033 tie_seed ∈ integer ≥0 (echoed only when tie_policy = random)
Functions (signatures only)
pub struct ReportModel { /* sections per Doc 7A; all fields sourced from Result/RunRecord/FrontierMap */ }

pub fn build_model(
result: &ResultDb,
run: &RunRecordDb,
frontier: Option<&FrontierMapDb>,
compare: Option<&CompareScenarios>
) -> ReportModel;

pub fn render_json(model: &ReportModel) -> String;   // one-decimal formatting applied
pub fn render_html(model: &ReportModel) -> String;   // templates consume only fields from model
Algorithm Outline
Cover & Snapshot
Fill label (“Decisive/Marginal/Invalid”), and snapshot items: Ballot (VM-VAR-001), Allocation (VM-VAR-010), Weighting (VM-VAR-030), Thresholds (VM-VAR-020/022/023), Double-majority (VM-VAR-024), Symmetry (VM-VAR-025), Frontier mode (VM-VAR-040). Values come from the ParameterSet snapshot (via Result/RunRecord).
Eligibility & Rolls
Print roll inclusion policy from VM-VAR-028 using fixed labels:
residents_only → “Residents only”
residents_plus_displaced → “Residents + displaced”
custom:list → “Custom (see list)” and render the provided list.
Show DivisionRegistry provenance {source, published_date} and totals (Σ eligible_roll, Σ ballots_cast).
If VM-VAR-021 > 0, add the per-unit quorum note, mentioning VM-VAR-021_scope.
Ballot (method paragraph)
Plain-English description per ballot type.
Mandatory sentence for approval ballots: “For legitimacy gates, the support % is the approval rate = approvals for the Change option divided by valid ballots.”
Legitimacy Panel
Show quorum/majority/double-majority/symmetry with Pass/Fail and values pulled from Result.
Outcome/Label
“Decisive / Marginal / Invalid (reason)”.
Frontier (if present)
Render statuses and counters from FrontierMap; note counts for quorum_blocked, protected_blocked, mediation, enclave.
Sensitivity
If CompareScenarios exists, render 2×3 table; else “N/A”.
Integrity / Provenance
Echo IDs and engine identifiers from RunRecord.
If tie_policy = random, display tie_seed (VM-VAR-033).
State Flow (very short)
Called after pipeline packaging; strictly offline; renderers consume only ReportModel.
Determinism & Numeric Rules
One-decimal at presentation; no double rounding; numbers come verbatim from artifacts; stable section order.
Edge Cases & Failure Policy
Validation failed: render Cover/Eligibility/Ballot + “Why invalid”; mark ❌ Invalid; omit Frontier.
Gates failed: up to the panel with ❌; outcome “Invalid (gate failed: …)”; omit Frontier.
If an unknown roll_inclusion_policy value is encountered, render the raw value verbatim (no crash) and flag in a non-blocking footnote.
Test Checklist (must pass)
Section order and one-decimal formatting.
Approval-denominator sentence appears for approval ballots.
Roll policy renders correctly for each allowed value (and gracefully for custom:list).
Frontier shown only when FrontierMap exists; diagnostics mirror flags.
Integrity identifiers match RunRecord + Result.id; tie_policy echoed; tie_seed shown only when random.
```
