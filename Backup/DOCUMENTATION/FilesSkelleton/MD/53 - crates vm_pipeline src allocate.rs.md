<!-- Converted from: 53 - crates vm_pipeline src allocate.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.937831Z -->

```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/allocate.rs, Version/FormulaID: VM-ENGINE v0) — 53/89
Goal & Success
Goal: Given each Unit’s UnitScores and magnitude, assign seats/power using the chosen method (WTA, D’Hondt, Sainte-Laguë, Largest Remainder) and emit UnitAllocation for downstream aggregation.
Success: Integer/rational math only; honors PR entry threshold; stable ordering; tie handling per policy; totals equal the Unit’s magnitude (or 100% for WTA).
Scope
In scope: Per-Unit allocation; PR threshold filtering; deterministic tie handling (and RNG path if configured).
Out of scope: Tabulation, gates/frontier, reporting.
Inputs → Outputs (with schemas/IDs)
Inputs: UnitScores (natural tallies); Unit.magnitude; Params (VM-VAR-010..012, 032..033); option order (by order_index).
Output: UnitAllocation { seats_or_power{Option→int/%}, tie_notes } (sums to m or 100%). Consumed by AGGREGATE.
Entities/Tables (minimal)
(Dev note: skeleton may track AllocationOutcome{allocations,tie_logs} for audit.)
Variables (used here)
VM-VAR-010 allocation_method ∈ {winner_take_all, proportional_favor_big, proportional_favor_small, largest_remainder, mixed_local_correction} (MMP delegated elsewhere in v1)
VM-VAR-011 use_unit_magnitudes ∈ {on, off} (v1: on)
VM-VAR-012 pr_entry_threshold_pct ∈ integer % 0..10 (default 0)
VM-VAR-032 tie_policy ∈ {status_quo, deterministic, random} (default status_quo)
VM-VAR-033 tie_seed ∈ integer ≥ 0 (default 0; used only if tie_policy=random)
Functions (signatures only)
use std::collections::BTreeMap;
use vm_core::{
ids::{UnitId, OptionId},
entities::{UnitMeta, OptionItem},
rng::TieRng,
variables::TiePolicy,
};
use crate::tabulation::UnitScores;

pub fn allocate_all(
unit_scores: &BTreeMap<UnitId, UnitScores>,
units: &BTreeMap<UnitId, UnitMeta>, // includes magnitude
p: &Params,
) -> BTreeMap<UnitId, UnitAllocation>;

fn apply_threshold(
scores: &BTreeMap<OptionId, u64>,
p: &Params
) -> BTreeMap<OptionId, u64>;

fn allocate_wta(
scores: &BTreeMap<OptionId, u64>,
m: u32,
tie: TiePolicy,
rng: Option<&mut TieRng>
) -> UnitAllocation;

fn allocate_dhondt(
scores: &BTreeMap<OptionId, u64>,
m: u32,
tie: TiePolicy,
rng: Option<&mut TieRng>
) -> UnitAllocation;

fn allocate_sainte_lague(
scores: &BTreeMap<OptionId, u64>,
m: u32,
tie: TiePolicy,
rng: Option<&mut TieRng>
) -> UnitAllocation;

fn allocate_largest_remainder(
scores: &BTreeMap<OptionId, u64>,
m: u32,
tie: TiePolicy,
rng: Option<&mut TieRng>
) -> UnitAllocation;

// helpers
fn break_tie(
context: &'static str,
contenders: &[OptionId],
options: &[OptionItem],
tie: TiePolicy,
rng: Option<&mut TieRng>
) -> OptionId;
Algorithm Outline (per method)
Precheck (WTA): if allocation_method = winner_take_all then assert m = 1 (also validated upstream). Winner is max Unit score; grant 100% power. Ties per policy.
PR threshold: for proportional/LR, drop options with share < VM-VAR-012. Keep deterministic option order for survivors.
D’Hondt (highest averages 1,2,3…): iterate seat slots, choose max quotient each step; record any last-seat tie context.
Sainte-Laguë (odd divisors 1,3,5…): same loop with odd sequence.
Largest Remainder: compute exact ideal = m * score / sum_scores (rational); assign floors; distribute remaining seats by largest fractional remainder.
Tie handling (common):
status_quo → if SQ present among contenders, pick it; else fall back to deterministic.
deterministic → smallest (order_index, OptionId).
random → uniform with ChaCha20 seeded by VM-VAR-033 tie_seed; log per pipeline TieLog rules.
Postconditions: Σ seats == m (or 100% WTA). Emit tie_notes when policy applied. Allocation trail available for audit.
State Flow
Pipeline: TABULATE → ALLOCATE → AGGREGATE (fixed). UnitAllocation feeds hierarchy aggregation.
Determinism & Numeric Rules
Stable ordering: Options by (order_index, id); Units by ID.
Integer/rational comparisons only; no presentation rounding here.
RNG only if tie_policy = random, seeded by VM-VAR-033; same inputs + same seed ⇒ identical outcomes/logs.
Edge Cases & Failure Policy
All scores zero ⇒ tie among all options (resolve per policy).
Threshold excludes all options ⇒ allocate zeros; downstream label/report handle.
If a blocking last-seat tie must be logged, return typed TieContext/TieError or pass to tie stage per design.
Test Checklist (must pass)
VM-TST-001: Sainte-Laguë with m=10 and approvals {10,20,30,40} ⇒ 1/2/3/4.
VM-TST-002: WTA with m=1, plurality {10,20,30,40} ⇒ D gets 100%.
VM-TST-003: LR vs D’Hondt vs Sainte-Laguë with m=7, shares 34/33/33 ⇒ 3/2/2 for all three.
Deterministic order respected (A > B > C > D); totals equal m.
```
