```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/validate.rs, Version/FormulaID: VM-ENGINE v0) — 51/89

1) Goal & Success
Goal: Perform structural and semantic validation of loaded, canonical inputs prior to any computation; emit a deterministic ValidationReport.
Success: If any blocking issue is found, the pipeline marks the run Invalid and skips TABULATE→…→FRONTIER. Non-blocking notes are captured as Warnings. All issue codes/paths are precise and reproducible.

2) Scope
In scope: Registry tree checks; magnitude rules; tally sanity per ballot_type; params ↔ inputs coherence; WTA constraint; baseline pairing invariant; quorum data presence/bounds; double-majority family preconditions; frontier prerequisites (shape-level); option ordering conformance (order_index).
Out of scope: Allocation math, gates math, contiguity computation, cycle detection via topology beyond registry tree (graph features validated later where needed).

3) Inputs → Outputs
Input: NormContext (from LOAD) {
  reg: DivisionRegistry,
  options: Vec<OptionItem>,            // canonical (order_index, OptionId)
  params: Params,                      // typed VM-VARs
  tallies: UnitTallies,                // per-type payload (plurality/approval/score/ranked_*)
  ids: { reg_id, tally_id, param_set_id }
}
Output: ValidationReport { pass: bool, issues: Vec<ValidationIssue> }.

4) Entities/Tables (minimal)
pub enum Severity { Error, Warning }
pub enum EntityRef { Root, Unit(UnitId), Option(OptionId), Param(&'static str), TallyUnit(UnitId), Adjacency(UnitId, UnitId) }
pub struct ValidationIssue { pub severity: Severity, pub code: &'static str, pub message: String, pub where_: EntityRef }
pub struct ValidationReport { pub pass: bool, pub issues: Vec<ValidationIssue> }

5) Variables (validated here — domains already checked in vm_core::variables)
• VM-VAR-001 ballot_type ∈ {plurality, approval, score, ranked_irv, ranked_condorcet} (must match tally.ballot_type).
• VM-VAR-010 allocation_method (checked for WTA constraint).
• VM-VAR-012 pr_entry_threshold_pct (range domain is core-validated; used for warnings only here if incompatible with ballot_type).
• VM-VAR-020 quorum_global_pct, VM-VAR-021 quorum_per_unit_pct (+ scope), VM-VAR-022 national_majority_pct, VM-VAR-023 regional_majority_pct.
• VM-VAR-024 double_majority_enabled, VM-VAR-026 affected_region_family_mode, VM-VAR-027 affected_region_family_ref.
• VM-VAR-025 symmetry_enabled, VM-VAR-029 symmetry_exceptions (shape only).
• VM-VAR-040 frontier_mode, VM-VAR-042 frontier_bands, VM-VAR-047 contiguity_edge_types, VM-VAR-048 island_exception_rule.
• VM-VAR-032 tie_policy; VM-VAR-052 tie_seed (int ≥ 0) only if tie_policy=random (re-assert optional).
• Note: Do not introduce ad-hoc “weighting_method”. Only enforce baseline pairing invariant on registry fields.

6) Functions (signatures only)
pub fn validate(ctx: &NormContext) -> ValidationReport;

fn check_registry_tree(reg: &DivisionRegistry) -> Vec<ValidationIssue>;
fn check_unit_magnitudes(units: &[Unit]) -> Vec<ValidationIssue>;
fn check_options_order(options: &[OptionItem]) -> Vec<ValidationIssue>;

fn check_params_vs_tally(params: &Params, tallies: &UnitTallies) -> Vec<ValidationIssue>;
fn check_tally_sanity_plurality(tallies: &UnitTallies, options: &[OptionItem]) -> Vec<ValidationIssue>;
fn check_tally_sanity_approval(tallies: &UnitTallies, options: &[OptionItem]) -> Vec<ValidationIssue>;
fn check_tally_sanity_score(tallies: &UnitTallies, options: &[OptionItem], params: &Params) -> Vec<ValidationIssue>;
fn check_tally_sanity_ranked_irv(tallies: &UnitTallies, options: &[OptionItem]) -> Vec<ValidationIssue>;
fn check_tally_sanity_ranked_condorcet(tallies: &UnitTallies, options: &[OptionItem]) -> Vec<ValidationIssue>;

fn check_wta_constraint(units: &[Unit], params: &Params) -> Vec<ValidationIssue>;
fn check_baseline_pairing(units: &[Unit]) -> Vec<ValidationIssue>;

fn check_quorum_data(units: &[Unit], tallies: &UnitTallies, params: &Params) -> Vec<ValidationIssue>;
fn check_double_majority_family(params: &Params, reg: &DivisionRegistry) -> Vec<ValidationIssue>;
fn check_frontier_prereqs(params: &Params, reg: &DivisionRegistry) -> Vec<ValidationIssue>;

7) Algorithm Outline (checks to implement exactly)
A) Registry
• Tree: exactly one root (parent == None), no cycles, every non-root parent exists.  → Error codes: "Hierarchy.MultipleRoots", "Hierarchy.Orphan", "Hierarchy.Cycle".
• UnitId/RegId coherence (embedded REG: matches ctx.ids.reg_id). → Error "Ids.RegistryMismatch".
• Magnitude: each Unit.magnitude ≥ 1. → Error "Unit.MagnitudeLtOne".
• Baseline pairing: if population_baseline is Some then population_baseline_year must be Some (and vice versa). → Error "Unit.BaselinePairMissing".

B) Options
• Canonical order: options sorted by (order_index, OptionId) and order_index unique/non-negative. (LOAD canonically sorts; re-assert) → Error "Option.OrderIndexDuplicate" / Warning "Option.OutOfOrder" (if only cosmetic).

C) Params ↔ Tally shape
• params.ballot_type == tallies.ballot_type. → Error "Params.BallotTypeMismatch".
• IDs: tallies.reg_id == reg.id (re-assert) → Error "Tally.RegistryMismatch".

D) Tally sanity (per unit)
Let valid = totals.valid_ballots, invalid = totals.invalid_ballots, and ballots_cast := valid + invalid (derived if not explicitly present).
Plurality:
  – sum(options[].votes) ≤ valid. → Error "Tally.Plurality.SumGtValid".
Approval:
  – For each option o: approvals_o ≤ valid. → Error "Tally.Approval.OptionGtValid".
  – Do not enforce Σ approvals ≤ valid (multiple approvals allowed).
Score:
  – Read scale_min/scale_max (per file payload); ensure scale_min < scale_max. → Error "Tally.Score.BadScale".
  – ballots_counted ≤ valid. → Error "Tally.Score.BallotsCountedGtValid".
  – For each option o: score_sum_o ≤ ballots_counted * scale_max (use u128 intermediate). → Error "Tally.Score.OptionExceedsCap".
Ranked IRV:
  – Each ranking array has unique items; Σ(group.count) ≤ valid per unit. → Error "Tally.IRV.BadRanking" / "Tally.IRV.SumGtValid".
Ranked Condorcet:
  – Same group checks as IRV; pairwise abstentions permitted; Σ(count) ≤ valid. → Error "Tally.Condorcet.SumGtValid".
Common:
  – valid ≥ 0, invalid ≥ 0; if any negative encountered (shouldn’t per schema) → Error "Tally.NegativeCount".
  – Unknown OptionId in any options[] list (must match registry) → Error "Tally.UnknownOption".
  – Options in tallies must be an **array ordered by order_index** (not a map). If order deviates from registry order_index → Error "Tally.OptionsOrderMismatch".

E) WTA constraint
• If allocation_method = winner_take_all then every Unit.magnitude == 1. → Error "Method.WTA.RequiresMagnitude1".

F) Quorum data presence/bounds (when set)
• If quorum_global_pct set (>0): require eligible_roll on all Units that contribute to global sums and Σ eligible_roll ≥ Σ ballots_cast. → Error "Quorum.MissingEligibleRoll" / "Quorum.BallotsGtEligible".
• If quorum_per_unit_pct set (>0): per unit, require eligible_roll and ballots_cast ≤ eligible_roll. → Error "Quorum.Unit.BallotsGtEligible".
(Quorum pass/fail is computed later; here we only ensure data sufficiency.)

G) Double-majority preconditions
• If double_majority_enabled = on and frontier_mode = none:
  – affected_region_family_mode ∈ {by_list, by_tag}. → Error "DoubleMajority.Mode".
  – Resolve to a non-empty family (all referenced Units exist or the tag resolves). → Error "DoubleMajority.EmptyFamily".

H) Frontier prerequisites (shape-level only)
• If frontier_mode ≠ none:
  – bands non-empty; each band 0..=100 with min_pct ≤ max_pct; bands are strictly ordered and non-overlapping (the overlap condition can be Error here or deferred; choose Error "Frontier.BandsOverlap").  
  – contiguity_edge_types ⊆ {land, bridge, water}. → Error "Frontier.EdgeTypeUnknown".  
  – All adjacency edges reference known Units and allowed edge types. → Error "Frontier.AdjacencyBadRef".

I) RNG tie knobs (re-assert only)
• If tie_policy = random then VM-VAR-052 (integer seed) must be present (≥0). → Error "Tie.RandomSeedMissing" (vm_core already enforces; duplicate as safety).

8) State Flow
validate(ctx) aggregates all issues from the helpers above; pass = (no Error).  
Pipeline: LOAD → **VALIDATE** (pass=false ⇒ label Invalid; skip TABULATE..FRONTIER) → otherwise continue.

9) Determinism & Numeric Rules
Pure integer checks; no floats.  
Option/Unit scans follow canonical order (Units by UnitId; Options by (order_index, OptionId)).  
No RNG here. Results (issues, ordering, messages) are byte-identical across OS/arch.

10) Edge Cases & Failure Policy
• Empty registry or no root ⇒ Errors (run becomes Invalid).  
• Tallies with units not in registry ⇒ Error "Tally.UnknownUnit".  
• Score tallies with ballots_counted > valid or scale_min ≥ scale_max ⇒ Errors.  
• Ranked groups with duplicate options in ranking ⇒ Error.  
• Bands present while frontier_mode = none ⇒ Error "Frontier.BandsWithoutMode".  
• Symmetry_enabled with missing complementary thresholds is handled later (gates); here only warn "Symmetry.ConfigWeird" if clearly inconsistent.

11) Test Checklist (must pass)
• Tree: single root passes; two roots or cycle detected ⇒ fails with precise codes.  
• Options: duplicate order_index ⇒ Error; mismatched order between registry and tally options[] ⇒ Error.  
• Plurality: Σ votes > valid ⇒ Error; equality passes.  
• Approval: any option approvals > valid ⇒ Error; Σ approvals > valid still passes.  
• Score: scale_min=0, scale_max=5; ballots_counted=80, valid=100; per-option cap = 80*5 enforced.  
• IRV/Condorcet: duplicate ranking member or Σ group counts > valid ⇒ Error.  
• WTA: allocation_method = winner_take_all with any Unit.magnitude ≠ 1 ⇒ Error.  
• Quorum data: ballots_cast > eligible_roll in any unit when quorum_per_unit_pct>0 ⇒ Error.  
• Double-majority: mode by_list with empty list ⇒ Error.  
• Frontier: overlapping bands or unknown edge type ⇒ Error.  
• IDs: tly.reg_id ≠ reg.id ⇒ Error.
```
