```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/map_frontier.rs, Version/FormulaID: VM-ENGINE v0) — 55/89

1) Goal & Success
Goal: Compute a deterministic FrontierMap from per-Unit support and topology after the gates have PASSED.
Success: Every Unit receives exactly one status from configured bands; contiguity obeys allowed edge types and corridor policy; protected or quorum-failing Units are blocked and flagged; output is byte-stable across OS/arch and matches the FrontierMap schema (Doc 1B/20).

2) Scope
In scope: frontier_mode ∈ {sliding_scale, autonomy_ladder}, band selection (single or multi-band), component building from adjacency, corridor/island handling, per-Unit quorum effect, protected-area blocking, enclave/mediation flags, summary counters.
Out of scope: Gate evaluation (done earlier), RNG/ties (none here), report rendering, JSON I/O.

3) Inputs → Outputs (types)
Inputs:
• UnitsView (IDs, protected_area bool, magnitude not used here)
• unit_support_pct: BTreeMap<UnitId, Ratio>  // approval: approvals_for_change/valid_ballots; others per spec
• AdjacencyView: edges (UnitId↔UnitId, EdgeType ∈ {Land, Bridge, Water})
• Params (VM-VARs used: 021 (+scope), 040, 042, 047, 048)
• per_unit_quorum: Option<BTreeMap<UnitId, bool>>  // true if unit met per-Unit quorum

Output:
• FrontierMap (FR:…)  // in-memory struct mirrored to schemas/frontier_map.schema.json
  - per unit: status, optional band_id/AP tag, component_id, flags {mediation,enclave,protected_blocked,quorum_blocked}
  - summary counters by status/flags

4) Data Structures (minimal)
use std::collections::{BTreeMap, BTreeSet};
use vm_core::{
  ids::UnitId,
  entities::{EdgeType},      // Land | Bridge | Water
  rounding::Ratio,
  variables::Params,
};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum FrontierMode { None, SlidingScale, AutonomyLadder }

#[derive(Clone, Debug)]
pub struct Band {
  pub min_pct: u8,        // 0..=100 inclusive boundary
  pub max_pct: u8,        // 0..=100 inclusive boundary (min ≤ max)
  pub status: String,     // machine-readable band/status label
  pub ap_id: Option<String>, // autonomy package tag (ladder only)
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ComponentId(pub u32);

#[derive(Default, Clone, Debug)]
pub struct UnitFlags {
  pub mediation: bool,
  pub enclave: bool,
  pub protected_blocked: bool,
  pub quorum_blocked: bool,
}

#[derive(Clone, Debug)]
pub struct UnitFrontier {
  pub status: String,                // one of bands[].status or "none" when mode=None
  pub band_index: Option<usize>,     // index into configured bands
  pub component: ComponentId,
  pub flags: UnitFlags,
}

#[derive(Default, Clone, Debug)]
pub struct FrontierMap {
  pub units: BTreeMap<UnitId, UnitFrontier>,
  pub summary_by_status: BTreeMap<String, u32>,
  pub summary_flags: BTreeMap<&'static str, u32>, // "mediation","enclave","protected_blocked","quorum_blocked"
}

5) Public API (signatures only)
pub fn map_frontier(
  units: &UnitsView,
  unit_support_pct: &BTreeMap<UnitId, Ratio>,
  adjacency: &AdjacencyView,
  p: &Params,
  per_unit_quorum: Option<&BTreeMap<UnitId, bool>>,
) -> FrontierMap;

// Internal helpers (pure, deterministic)
fn resolve_mode_and_bands(p: &Params) -> (FrontierMode, Vec<Band>);
fn build_components(
  adjacency: &AdjacencyView,
  allowed: &AllowedEdges,                 // derived from VM-VAR-047
  corridor: IslandCorridorRule,           // VM-VAR-048
) -> (BTreeMap<UnitId, ComponentId>, Vec<BTreeSet<UnitId>>);
fn pick_band_status(support: Ratio, bands: &[Band]) -> (String, Option<usize>);
fn apply_protection_and_quorum(
  unit_id: &UnitId,
  intended_status: &mut String,
  band_idx: &mut Option<usize>,
  flags: &mut UnitFlags,
  units: &UnitsView,
  per_unit_quorum: Option<&BTreeMap<UnitId, bool>>,
);
fn tag_mediation_and_enclaves(
  unit_id: &UnitId,
  unit_map: &mut BTreeMap<UnitId, UnitFrontier>,
  components: &Vec<BTreeSet<UnitId>>,
  allowed: &AllowedEdges,
);
fn update_summaries(out: &mut FrontierMap);

6) Algorithm Outline
A) Resolve configuration
• Read frontier_mode (VM-VAR-040); if None, return map with status "none" and empty bands for all Units.
• Load and sanity-check bands (VM-VAR-042): ordered, non-overlapping (validated earlier), preserve order given.
• Derive AllowedEdges from VM-VAR-047 ⊆ {land, bridge, water}.
• Read IslandCorridorRule from VM-VAR-048 ∈ {none, ferry_allowed, corridor_required}.

B) Build components
• Construct undirected graph using only allowed edges; adjust connectivity per corridor rule:
  – none: water edges included only if VM-VAR-047 contains water; no special bridging.
  – ferry_allowed: treat water edges as connectable across short gaps (model as included when configured).
  – corridor_required: require a special “corridor” classification; otherwise treat water spans as disconnected.
• Compute connected components; assign ComponentId in stable ascending order of smallest UnitId in each component.

C) Assign band/status per Unit
• For each UnitId in sorted order:
  – support = unit_support_pct[unit] (default 0/1 if missing).
  – (status, band_idx) = pick_band_status(support, bands)  // ≥ cutoffs; single cutoff is a one-band case.
  – Initialize flags = UnitFlags::default(); comp = components[unit].
  – apply_protection_and_quorum:
     · If unit.protected_area && status != "none": set protected_blocked=true; force status="none"; band_idx=None.
     · If per-Unit quorum present and false for this unit: quorum_blocked=true; force status="none"; band_idx=None.

D) Mediation & enclaves
• Mediation: if a unit meets a change band (before protection/quorum) but is isolated from any same-status cluster under allowed edges/corridor → set flags.mediation=true and force status="none".
• Enclave (informational): after final statuses, if a unit with status ≠ "none" is fully surrounded by "none" within its component, set flags.enclave=true (no status flip).

E) Populate FrontierMap
• out.units[unit] = UnitFrontier { status, band_index, component, flags }.
• update_summaries(out): count by status; sum each flag across units.

7) State Flow
APPLY_DECISION_RULES (Pass) → **MAP_FRONTIER (this file)** → RESOLVE_TIES (not used here; reserved for downstream blocking contexts) → LABEL → BUILD_RESULT/BUILD_RUN_RECORD (FR referenced from RUN).

8) Determinism & Numeric Rules
• Units processed in lexicographic UnitId order; components numbered deterministically.
• Integer/rational math only for support and comparisons (≥ at cutoffs); no floats or rounding.
• No RNG; tie/seed variables are irrelevant in this stage.

9) Edge Cases & Failure Policy
• Missing unit_support entry → treat as 0/1 (maps to lowest band/"none").
• Bands present with min>max or overlap should have been rejected in VALIDATE; here assume well-formed.
• Adjacency referencing unknown Units → should be blocked earlier; treat as fatal validation error upstream.
• corridor_required without any corridor-typed edge → islands remain disconnected (likely mediation).
• All status "none" is valid output.

10) Test Checklist (must pass)
• Single-cutoff (“binary”): band ≥60%; mainland connected by Land edges changes; island separated by Water:
  – with corridor=none → island units mediation=true, status "none".
  – with ferry_allowed and water allowed → island linked, changes.
• Multi-band ladder: each Unit maps to exactly one band; AP tags carried through.
• Protected area: protected unit that meets change band ends with status "none" and protected_blocked=true.
• Per-Unit quorum: failing units forced to "none" and quorum_blocked=true; passing neighbors unaffected.
• Components deterministic: renumbering inputs or edge order does not change ComponentId assignment for the same graph.
• Summaries: counts by status and flags match per-Unit listings.

11) Notes for Packaging & Schemas
• Aligns with frontier_map.schema.json (Doc 20): store per-unit status, flags, and (if desired at write time) the observed support as {num,den}. Result artifacts keep shares as JSON numbers elsewhere per Doc 18.
• Booleans are real JSON booleans (no "on"/"off" strings). VM-VAR IDs: 021 (per-Unit quorum), 040 (mode), 042 (bands), 047 (edge types), 048 (corridor policy). Seed/tie variables are not used here.
```
