<!-- Converted from: 55 - crates vm_pipeline src map_frontier.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.000094Z -->

```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/map_frontier.rs, Version/FormulaID: VM-ENGINE v0) — 55/89
Goal & Success
Goal: Translate per-Unit support into FrontierMap statuses using the selected frontier mode, contiguity policy, island/corridor rule, and scoped per-Unit quorum effects.
Success: Exactly one status per Unit; components computed only from allowed edge types; per-Unit quorum respected; protected areas never change (blocked with a flag); output matches FrontierMap fields; deterministic given same inputs.
Scope
In scope: frontier_mode ∈ {sliding_scale, autonomy_ladder} (where “binary” is just a single cutoff band), component/contiguity computation, island/corridor handling, per-Unit quorum interaction, autonomy package tagging (via bands), summary counters.
Out of scope: Gate evaluation (must have passed already), tie resolution, report rendering.
Inputs → Outputs (with schemas/IDs)
Inputs
LoadedContext (Units with protected_area?, Adjacency {a,b,type}, ParameterSet).
Per-Unit support % for Change (approval: approval rate = approvals_for_change / valid_ballots).
Optional map of per-Unit quorum pass/fail (if VM-VAR-021 > 0 with scope).
Variables (used here)
VM-VAR-040 frontier_mode ∈ {none, sliding_scale, autonomy_ladder} (we are called only if ≠ none).
VM-VAR-042 frontier_bands — ordered, non-overlapping bands; may carry action/AP ids for autonomy ladder.
VM-VAR-047 contiguity_edge_types ⊆ {land, bridge, water}.
VM-VAR-048 island_exception_rule ∈ {none, ferry_allowed, corridor_required}.
VM-VAR-021 quorum_per_unit_pct (+ VM-VAR-021_scope) if provided by gates.
Output
FrontierMap (FR:…) with per-Unit fields: {status, band_id?, component_id, flags{mediation,enclave,protected_blocked,quorum_blocked}} and summary counters.
Entities/Tables (minimal)
(Types align with Doc 1/5 sketches: FrontierUnit/FrontierOut; IDs are Unit IDs; band ids come from frontier_bands.)
Variables (used here)
(See list under Inputs; no 041/045/046 variables are used.)
Functions (signatures only)
pub fn map_frontier(
units: &UnitsView,
unit_support_pct: &BTreeMap<UnitId, Ratio>,   // approval: approval rate; others per Doc 4
adjacency: &AdjacencyView,
p: &Params,
per_unit_quorum: Option<&BTreeMap<UnitId, bool>>
) -> FrontierMap;

fn build_components(adjacency: &AdjacencyView, allowed: &ContiguityModes) -> Components;
fn apply_island_exception(components: &Components, rule: IslandRule) -> MediationFlags;
fn status_by_band(s: Ratio, bands: &[Band]) -> (Status, BandId);  // single-cutoff = one band
Algorithm Outline
Preconditions
Caller ensures frontier_mode != none.
frontier_bands are validated earlier: ordered, non-overlapping, cover intended ranges; autonomy ladder bands carry AP ids if needed.
Components & adjacency
Build connected components using only VM-VAR-047 edge types (stable order).
Apply VM-VAR-048:
none → water isolation does not connect; such isolated eligible units may become mediation.
ferry_allowed → allow water to connect islands to mainland.
corridor_required → bridges alone insufficient; require an explicit “corridor” classification to connect across water.
Per-Unit quorum interaction
If VM-VAR-021 > 0: a failing unit cannot change; set quorum_blocked=true and force status=no_change. Family inclusion/exclusion was already handled in gates by VM-VAR-021_scope.
Protected areas
If unit.protected_area == true and the band implies a change, block change; set protected_blocked=true; final status=no_change.
Assign status by band
Compute (status, band_id) via status_by_band(support, frontier_bands).
“Binary” behavior is modeled by providing exactly one cutoff band (e.g., ≥60% ⇒ change); otherwise use multiple bands (sliding or ladder).
For autonomy_ladder, bands include the AP id to tag into the FrontierMap.
Mediation & enclaves
If a unit meets a band but is not connected (per allowed edges) to a qualifying cluster, mark mediation (no change).
Units fully surrounded by non-change areas after mapping can be flagged as enclave (informational).
Emit FrontierMap
One record per Unit: status, band_id?, component_id, flags.
Maintain summary counters by status/flags for reporting.
State Flow
APPLY_DECISION_RULES (Pass) → MAP_FRONTIER → RESOLVE_TIES (only if blocking) → LABEL. If gates Fail, caller skips frontier.
Determinism & Numeric Rules
Integer/rational comparisons; ≥ at cutoffs; no presentation rounding.
Stable iteration orders (Units by ID). Same inputs ⇒ identical FrontierMap bytes.
Edge Cases & Failure Policy
Missing adjacency when frontier_mode active ⇒ ReferenceError.
Bands unordered/overlapping should have been blocked in VALIDATE.
Exact cutoff (support == threshold) counts as meeting the band.
AP id missing on an autonomy ladder band ⇒ ReferenceError.
Test Checklist (must pass)
Cutoff band case (“binary”): single band ≥60%; allowed={land}; island separated by water ⇒ mainland units change; island units mediation under none, connect under ferry_allowed.
Multi-band: ordered bands map to exactly one status per Unit; ladder bands tag AP ids; no flags ⇒ Decisive.
Protected area: protected Unit mapped to change by bands remains no_change and sets protected_blocked=true.
Quorum scope: failing per-Unit quorum forces no_change and quorum_blocked=true.
```
