
````
Pre-Coding Essentials (Component: crates/vm_algo/src/gates_frontier.rs, Version FormulaID VM-ENGINE v0) — 47/89

1) Goal & Success
Goal: Implement decision gates (quorum → national majority → double-majority → symmetry) and, when gates pass, frontier mapping (sliding_scale / autonomy_ladder) with contiguity/protection rules; emit flags affecting the final label.
Success: Integer/rational math only; fixed denominators per spec (approval majority uses valid_ballots); deterministic ordering; correct flags for mediation/enclave/protected_override; outputs feed labeling per Docs 2/5/7 & Annexes.

2) Scope
In scope: Gate computations (national and scoped), affected-family evaluation, symmetry check, frontier status assignment from bands, contiguity using typed edges, and unit flags.
Out of scope: Tabulation/allocation/aggregation and report rendering (Doc 7); any RNG (gates/frontier are deterministic—tie policy affects allocation elsewhere).

3) Inputs → Outputs
Inputs:
- Aggregates needed for gates: national/regional totals: ballots_cast, invalid_ballots, valid_ballots, eligible_roll; per-unit support for “Change”.
- Registry metadata: Units (tree), Adjacency edges {a,b,type} where type ∈ {"land","bridge","water"}.
- Params: gates (VM-VAR-020..029, 030/031 do not redefine “denominator”), frontier (VM-VAR-040..048), ties (VM-VAR-050 tie_policy; VM-VAR-052 tie_seed not used here).
- Option set: includes `is_status_quo` and canonical `(order_index, OptionId)`.

Outputs:
- `LegitimacyReport { quorum, majority, double_majority, symmetry, pass }`
- `FrontierOut` (only if gates pass): per-unit status & flags, summary
- `LabelImpact { decisive | marginal | invalid, reason }` (input to reporting/label stage)

4) Entities/Tables (minimal)
Use vm_core types:
- IDs: `UnitId`, `OptionId`
- Entities: Unit tree, adjacency edge kind
- Rounding helpers: integer ratio compare (no floats)

5) Variables (Annex-A IDs used here)
Gates:
- **020** `quorum_global_pct: u8`
- **021** `quorum_per_unit_pct: u8` and scope control (per spec)
- **022** `national_majority_pct: u8` (e.g., 55)
- **023** `regional_majority_pct: u8` (e.g., 55)
- **024** `double_majority_enabled: bool`
- **025** `symmetry_enabled: bool`
- **026** `affected_region_family_mode: enum { by_list, by_tag, by_proposed_change }`
- **027** `affected_region_family_ref: list/tag` (used with by_list/by_tag)
- **029** `symmetry_exceptions: optional list/tag`
- **007** `include_blank_in_denominator: bool` (gate denominators only; **approval majority ignores this**)
Frontier:
- **040** `frontier_mode: enum { none, sliding_scale, autonomy_ladder }`
- **042** `frontier_bands: ordered, non-overlapping [{min_pct,max_pct,status}]`
- **047** `contiguity_edge_types: set ⊆ {land, bridge, water}`
- **048** `island_exception_rule: enum { none, ferry_allowed, corridor_required }`
Tie policy (not used in gates/frontier math, documented for completeness):
- **050** `tie_policy: enum { status_quo, deterministic, random }`
- **052** `tie_seed: integer ≥ 0` (not used here)

6) Functions (signatures only)
```rust
use std::collections::{BTreeMap, BTreeSet};
use vm_core::{
    ids::UnitId,
    entities::{/* Unit/Adjacency types */},
    variables::Params,
    rounding::{ge_percent}, // integer compare a/b ≥ p%
};

pub struct GateInputs {
    pub nat_ballots_cast: u64,
    pub nat_invalid_ballots: u64,
    pub nat_valid_ballots: u64,
    pub nat_eligible_roll: u64,
    pub region_valid_ballots: BTreeMap<String, u64>,        // keyed by region id/label
    pub region_support_for_change: BTreeMap<String, u64>,   // approvals_for_change or equivalent
    pub unit_valid_ballots: BTreeMap<UnitId, u64>,
    pub unit_ballots_cast: BTreeMap<UnitId, u64>,
    pub unit_eligible_roll: BTreeMap<UnitId, u64>,
    pub unit_support_for_change: BTreeMap<UnitId, u64>,     // approval rate numerator per unit
}

pub struct GateResult {
    pub quorum_national: bool,
    pub quorum_per_unit_passset: BTreeSet<UnitId>, // units meeting per-unit quorum
    pub majority_national: bool,
    pub majority_regional: bool, // when double_majority applies
    pub double_majority: bool,
    pub symmetry: bool,
    pub pass: bool,
}

pub struct FrontierInputs {
    pub unit_support_for_change: BTreeMap<UnitId, (u64 /*num*/, u64 /*den*/)> , // observed support ratios
    pub units_all: BTreeSet<UnitId>,
    pub adjacency: Vec<(UnitId, UnitId, FrontierEdge)>,
    pub protected_units: BTreeSet<UnitId>, // if registry/provenance marks protection
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FrontierEdge { Land, Bridge, Water }

#[derive(Clone, Debug)]
pub struct FrontierFlags {
    pub contiguity_ok: bool,
    pub mediation_flagged: bool,
    pub protected_override_used: bool,
    pub enclave: bool,
}

#[derive(Clone, Debug)]
pub struct FrontierUnit {
    pub status: String,   // one of bands[].status (or "none" when mode = none)
    pub flags: FrontierFlags,
}

pub struct FrontierSummary {
    pub band_counts: BTreeMap<String, u32>,
    pub mediation_units: u32,
    pub enclave_units: u32,
    pub any_protected_override: bool,
}

pub struct FrontierOut {
    pub units: BTreeMap<UnitId, FrontierUnit>, // stable by UnitId
    pub summary: FrontierSummary,
}

pub fn apply_decision_gates(inp: &GateInputs, p: &Params) -> GateResult;

pub fn map_frontier(inp: &FrontierInputs, p: &Params) -> FrontierOut; // call only if gates.pass

// helpers (signatures)
fn compute_quorum_national(ballots_cast: u64, eligible_roll: u64, cutoff_pct: u8) -> bool;
fn compute_quorum_per_unit(
    unit_ballots_cast: &BTreeMap<UnitId,u64>,
    unit_eligible_roll: &BTreeMap<UnitId,u64>,
    cutoff_pct: u8
) -> BTreeSet<UnitId>;
fn national_approval_majority(valid_ballots: u64, approvals_for_change: u64, cutoff_pct: u8) -> bool;
fn affected_family(
    mode: /* from Params */, refval: /* list/tag resolver */,
    units_all: &BTreeSet<UnitId>
) -> BTreeSet<UnitId>;
fn assign_band_status(support_pct_tenths: u16, bands: &[(u8,u8,String)]) -> String; // min..max inclusive
fn contiguous_components(
    allowed: &BTreeSet<FrontierEdge>,
    adjacency: &[(UnitId,UnitId,FrontierEdge)]
) -> Vec<BTreeSet<UnitId>>;
````

7. Algorithm Outline (implementation plan)
   Gates

* **Quorum (national):** turnout = Σ ballots\_cast / Σ eligible\_roll. Pass iff `ge_percent(Σ ballots_cast, Σ eligible_roll, VM-VAR-020)`.
* **Quorum (per-unit):** for each unit, turnout ≥ **021**; collect Pass set. Policy scope for failing units is applied later (frontier/family interaction per spec).
* **Majority (national):**

  * **Approval ballots:** support = `approvals_for_change / valid_ballots` (**fixed denominator**). Pass iff `ge_percent(approvals_for_change, valid_ballots, 022)`.
  * If **007 include\_blank\_in\_denominator = true**, this toggle affects only gates where the spec allows it; **it does not affect approval majority**, which remains `/ valid_ballots`.
* **Double-majority:** if **024 on**, require national majority **AND** affected-family majority ≥ **023** using the same support definition per unit/family. Resolve affected family by **026/027** (by\_list/by\_tag/by\_proposed\_change). Family must be non-empty when required.
* **Symmetry:** if **025 on**, ensure the rule/denominator are direction-invariant subject to **029** exceptions; synthesize a boolean `symmetry = true/false`.
* **Pass:** all required gate booleans must be true; otherwise `pass=false` (skip frontier).

Frontier mapping (only if gates pass)

* **Mode:** If **040 = none**, statuses are `"none"`, no bands.
* **Bands:** **042** provides ordered, non-overlapping `{min_pct,max_pct,status}`. For each unit:

  * Compute support % as integer tenths for reporting compatibility; choose the first band where `min ≤ pct ≤ max`. (No floats; comparisons use integers.)
* **Contiguity:** Build components using only edges in **047**. Units that meet a change-band but are disconnected (when connectivity is required for that band’s semantics) get `mediation_flagged = true`.
* **Island/corridor:** Apply **048**:

  * `none`: no special handling;
  * `ferry_allowed`: treat `bridge`/`water` edges as admissible where needed;
  * `corridor_required`: ensure status clusters are connected through admissible corridors or flag mediation.
* **Protected overrides:** If unit is protected and its assigned status would imply a change in violation of protection, set `protected_override_used = true` and adjust status per spec (usually “hold”/“none”).
* **Enclave:** A unit whose status cluster is fully surrounded by non-matching status (under admissible edges) sets `enclave = true`.
* **Summary:** counts per status; `mediation_units`, `enclave_units`, `any_protected_override`.

Label impact (signals only)

* If any of: mediation present, enclaves present, protected overrides used ⇒ suggest `Marginal` to the reporting labeler. Otherwise labeling follows Docs 2/7 (e.g., **044/045** thresholds/policy in report layer).

8. State Flow
   Pipeline: TABULATE/AGGREGATE → **apply\_decision\_gates** → (if pass) **map\_frontier** → LABEL (reporting consumes LegitimacyReport + FrontierOut; final label is produced in report layer).

9. Determinism & Numeric Rules

* Pure integer/rational comparisons via vm\_core rounding helpers (`ge_percent` etc.).
* Stable ordering: Units by `UnitId` (lexicographic); maps are `BTree*`.
* **No RNG here** (VM-VAR-050/052 influence allocation tie breaks, not gates/frontier).

10. Edge Cases & Failure Policy

* Quorum fail (national or required per-unit) ⇒ `pass=false`, frontier skipped.
* Affected family unresolved/empty when required ⇒ `pass=false`.
* Bands missing/overlapping/out-of-order ⇒ error from validation; mapping aborts.
* Adjacency referencing unknown units ⇒ validation error.
* Units missing in `FrontierInputs.units_all` ⇒ mapping error (pipeline ensures 1:1 with registry).

11. Test Checklist (must pass)

* Quorum edges: exact cutoff passes; just-below fails (national & per-unit).
* Approval majority uses `approvals_for_change / valid_ballots` regardless of **007**.
* Double-majority requires both national and family pass; unresolved family ⇒ fail.
* Symmetry toggled on with/without exceptions yields expected boolean.
* Frontier:

  * Mode `none`: all statuses `"none"`, flags false.
  * Valid bands assign statuses deterministically; per-unit support mapped to the correct band at boundaries.
  * Contiguity respects **047**; island/corridor handling per **048**; mediation/enclave flagged correctly.
* Determinism: identical inputs on any OS/arch produce identical outputs/ordering.

```


```
