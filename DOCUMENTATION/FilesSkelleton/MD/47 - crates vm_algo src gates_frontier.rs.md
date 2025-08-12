<!-- Converted from: 47 - crates vm_algo src gates_frontier.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.763368Z -->

```
Pre-Coding Essentials (Component: crates/vm_algo/src/gates_frontier.rs, Version/FormulaID: VM-ENGINE v0) — 47/89
Goal & Success
Goal: Implement decision gates (quorum → national majority → double-majority → symmetry) and, if gates pass, frontier mapping (sliding_scale / autonomy_ladder) with contiguity/protection rules; emit flags that can affect the final label.
Success: Integer/rational math only; fixed denominators (incl. approval rate uses valid ballots); deterministic ordering; correct flags for mediation/enclave/protected_override; outputs drive labeling per spec.
Scope
In scope: Gate computations (global + per-unit), affected-family evaluation, symmetry check, and frontier mapping (status assignment + contiguity/adjacency policies + protected/mediation flags).
Out of scope: Tabulation/allocation/aggregation and report rendering (Doc 7 consumes our outputs).
Inputs → Outputs
Inputs:
aggregates: national/region totals incl. valid_ballots, ballots_cast, eligible_roll; per-region support for Change.
registry_meta: Units, hierarchy, Adjacency {a,b,type} where type ∈ {land, bridge, water}.
params: VM-VAR 020..029 (gates), 040..048 (frontier), 032..033 (ties).
option_set: includes is_status_quo and deterministic order_index.
Outputs:
LegitimacyReport { quorum, majority, double_majority, symmetry, pass/fail values }.
FrontierMap { per_unit: {status, flags}, summary } (only if gates Pass).
LabelImpact { decisive | marginal | invalid, reason } (inputs to Report).
Entities/Tables (minimal)
(Structures below; full field lists live in Doc 5B/5C.)
Variables (used here)
VM-VAR-020 quorum_global_pct, VM-VAR-021 quorum_per_unit_pct, VM-VAR-021_scope.
VM-VAR-022 national_majority_pct, VM-VAR-023 regional_majority_pct (50–75, default 55).
VM-VAR-024 double_majority_enabled, VM-VAR-025 symmetry_enabled.
VM-VAR-026 affected_region_family_mode ∈ {by_list, by_tag, by_proposed_change}.
VM-VAR-027 affected_region_family_ref (IDs or a tag; used with by_list/by_tag).
VM-VAR-029 symmetry_exceptions (optional list/tag with rationale).
VM-VAR-007 include_blank_in_denominator (on/off).
**VM-VAR-040 frontier_mode ∈ {none, sliding_scale, autonomy_ladder}*.
VM-VAR-042 frontier_bands (ordered, non-overlapping bands → statuses/APs).
VM-VAR-047 contiguity_edge_types ⊆ {land, bridge, water}.
VM-VAR-048 island_exception_rule ∈ {none, ferry_allowed, corridor_required}.
VM-VAR-032 tie_policy ∈ {status_quo, deterministic, random}, VM-VAR-033 tie_seed (int ≥0) (for RNG tie contexts only).
Functions (signatures only)
pub struct GateInputs { /* national & regional tallies, valid_ballots, ballots_cast, eligible_roll, per-unit supports */ }
pub struct GateResult  { /* values + pass/fail booleans */ }

pub struct FrontierInputs { /* per-unit supports for Change, Unit tree, Adjacency edges with types */ }
pub struct FrontierUnit { pub status: FrontierStatus, pub flags: FrontierFlags }
pub struct FrontierOut  { pub units: BTreeMap<UnitId, FrontierUnit>, pub summary: FrontierSummary }

pub fn apply_decision_gates(inp: &GateInputs, p: &Params) -> GateResult;
pub fn map_frontier(inp: &FrontierInputs, p: &Params) -> FrontierOut; // only called if gates pass

// helpers
fn affected_family(units: &[…], mode: FamilyMode, p: &Params) -> BTreeSet<UnitId>;
fn contiguous_blocks(allowed_edges: EdgeSet, adjacency: &[AdjEdge]) -> Vec<BTreeSet<UnitId>>;
fn cutoff_pass(support_pct: u32, cutoff: u32) -> bool; // ≥ rule
Algorithm Outline (implementation plan)
Quorum
National turnout = Σ ballots_cast / Σ eligible_roll (integer %). Pass iff ≥ VM-VAR-020.
Per-unit quorum Pass iff each unit turnout ≥ VM-VAR-021. Scope (VM-VAR-021_scope) controls whether failing units can change status or are excluded from family.
Majority (national)
Default denominator = valid ballots; if VM-VAR-007 = on, include blanks for gate denominators only.
Approval ballots: support uses approval rate = approvals_for_change / valid_ballots (fixed rule).
Pass iff ≥ VM-VAR-022.
Double-majority
If VM-VAR-024 = on, require both national and affected-family ≥ thresholds (VM-VAR-022/023).
Affected family per VM-VAR-026/027.
When frontier_mode = none, VM-VAR-026 ∈ {by_list, by_tag} and VM-VAR-027 must resolve to a non-empty family (Annex B validation).
Symmetry
If VM-VAR-025 = on, ensure thresholds/denominators are identical regardless of direction; exceptions may be recorded via VM-VAR-029.
If any gate fails ⇒ Invalid (skip frontier).
Frontier mapping (when VM-VAR-040 ≠ none)
Bands: Use VM-VAR-042 frontier_bands (ordered, non-overlapping) to assign each unit exactly one status/band. (Binary behavior is represented by a single cutoff band; no separate “binary” mode.)
Contiguity: Build connected components using only allowed edge types (VM-VAR-047). Units meeting band cutoff but isolated by disallowed edges become mediation (no change).
Island/corridor: Apply VM-VAR-048 for island/peninsula handling.
Per-unit quorum interaction: if scope is frontier_only, failing units cannot change status but still count in family; if frontier_and_family, exclude failing units from family sums.
Flags & Label
Set unit flags: mediation, enclave, protected_override as detected.
Labeling later follows Doc 2/Doc 7: if any such flags exist, candidate label = Marginal; otherwise labeling is resolved per decisiveness_label_policy (VM-VAR-045) and default_majority_label_threshold (VM-VAR-044) in the reporting stage.
State Flow
Pipeline: APPLY_DECISION_RULES → (if Pass) MAP_FRONTIER → RESOLVE_TIES (only if blocking) → LABEL. Our outputs feed the Report’s Legitimacy Panel and Frontier sections.
Determinism & Numeric Rules
Stable orders: Units by Unit ID; Options by (order_index, id).
Integer or rational comparisons; round-half-to-even only at defined decision points.
RNG used only if tie_policy = random and only via tie_seed; same inputs + same seed ⇒ identical outputs.
Edge Cases & Failure Policy
Quorum fail (national or scoped per-unit) ⇒ Invalid; skip frontier.
No affected family when required ⇒ Invalid.
Frontier bands must be ordered and non-overlapping; otherwise mapping step errors (validation).
Adjacency must reference known units; unknowns are validation errors.
Test Checklist (must pass)
Quorum pass/fail at exact cutoffs (national & scoped per-unit).
Approval majority uses approval rate / valid ballots (no drift when blanks toggle).
Double-majority with frontier_mode = none fails when family unresolved; passes when by_list/by_tag provided.
Frontier contiguity respects VM-VAR-047; islands behave per VM-VAR-048.
Presence of mediation/enclave/protected_override flips candidate label to Marginal; otherwise label resolved by reporting policy.
```
