<!-- Converted from: 46 - crates vm_algo src mmp.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.730959Z -->

```
Pre-Coding Essentials (Component: crates/vm_algo/src/mmp.rs, Version/FormulaID: VM-ENGINE v0) — 46/89
1) Goal & Success
Goal: Mixed-Member Proportional (MMP) helpers: compute target seats from vote totals, derive deficits/top-ups against local seats, and handle overhang per policy.
Success: Pure integer/rational math; deterministic results; respects params (mlc_topup_share_pct, target_share_basis, mlc_correction_level, overhang_policy, total_seats_model). Outputs sum to the intended total under the chosen policy.
2) Scope
In scope: seat-target apportionment from vote shares; top-up computation; minimal iterative expansion when total seats must grow to satisfy overhang policy.
Out of scope: reading ballots/locals (caller passes counts), PR within units (other modules), reporting.
3) Inputs → Outputs
Inputs:
vote_totals: BTreeMap<OptionId, u64> (party/national list votes)
local_seats: BTreeMap<OptionId, u32> (already awarded “local” seats)
base_total_local: u32 (sum of local seats across correction scope)
params: &Params (reads VM-VAR-013..017, 015 fixed to natural_vote_share)
method_for_targets: AllocationMethod (e.g., Sainte-Laguë or D’Hondt) for target seat apportionment
correction_level: country or region (affects which totals you pass per call)
Outputs:
TargetSeats: BTreeMap<OptionId,u32> (apportioned total seats per option at scope)
TopUps: BTreeMap<OptionId,u32> where topup = max(0, target - local)
FinalSeatTotals: BTreeMap<OptionId,u32> (local + topups)
effective_total_seats: u32 (after any expansion)
4) Entities/Tables (minimal)
5) Variables (used here)
6) Functions (signatures only)
rust
CopyEdit
use std::collections::BTreeMap;
use vm_core::{
ids::OptionId, variables::{Params, AllocationMethod, OverhangPolicy, TotalSeatsModel},
rounding::{ge_percent_half_even},
};

pub struct MmpOutcome {
pub targets: BTreeMap<OptionId, u32>,
pub topups: BTreeMap<OptionId, u32>,
pub finals: BTreeMap<OptionId, u32>,
pub effective_total_seats: u32,
pub overhang_by_option: BTreeMap<OptionId, u32>,
}

/// Compute the intended total seat count given local seats and a top-up share %.
/// If share = s%, total ≈ local / (1 - s). Uses half-even when rounding.
pub fn compute_total_from_share(local_total: u32, topup_share_pct: u8) -> u32;

/// Apportion total seats to options from vote totals using the chosen method.
/// (Typically Sainte-Laguë for proportional targets.)
pub fn apportion_targets(
total_seats: u32,
vote_totals: &BTreeMap<OptionId, u64>,
method: AllocationMethod,
) -> BTreeMap<OptionId, u32>;

/// Given targets and local seats, compute top-ups and apply overhang policy.
/// May expand total seats if policy demands (see params.VM-VAR-014/017).
pub fn compute_topups_and_apply_overhang(
targets: &BTreeMap<OptionId, u32>,
local_seats: &BTreeMap<OptionId, u32>,
overhang_policy: OverhangPolicy,
total_seats_model: TotalSeatsModel,
method_for_targets: AllocationMethod,
vote_totals: &BTreeMap<OptionId, u64>,
) -> MmpOutcome;

/// One-shot convenience orchestrator for a correction scope (country or region).
pub fn mmp_correct(
vote_totals: &BTreeMap<OptionId, u64>,
local_seats: &BTreeMap<OptionId, u32>,
params: &Params,
method_for_targets: AllocationMethod,
) -> MmpOutcome;

7) Algorithm Outline (implementation plan)
Total from top-up share
Let L = Σ local_seats, s = VM-VAR-013 / 100. Intended total T = round_half_even( L / (1 - s) ).
Guard: if s = 0, T = L; if s = 100, invalid (reject in params validation).
Target apportionment
Apportion T seats to options from vote_totals using method_for_targets (default recommended: Sainte-Laguë).
Deterministic: options ordered by (order_index, id); all math integer; quotient comparisons via cross-multiplication.
Top-up deficits
For each option i: deficit_i = max(0, target_i - local_i).
overhang_i = max(0, local_i - target_i) (diagnostic).
Overhang policy + total model
allow_overhang: keep T for targets; set topup_i = deficit_i. Final totals = local + topup. Effective total may exceed T by Σ overhang; report that delta.
compensate_others: keep overall seats fixed at T. Set topup_i = deficit_i for non-overhang options; if Σtopups > T - L, scale by discrete apportionment: re-apportion the available top-up seat pool across non-overhang options by vote share (or by deficit_i weights), using the same method_for_targets. Overhang options get zero top-ups; others may not fully reach target.
add_total_seats: expand total seats minimally so that after apportionment target_i >= local_i for all i. Algorithm:
Start with T0 = T. While ∃ i with target_i(Tk) < local_i, set Tk+1 = Tk + 1, recompute targets; stop when all target_i >= local_i.
Then topup_i = target_i - local_i; effective_total = Tk. (This is the standard “expanding house size to clear overhang”.)
Assemble outcome
finals_i = local_i + topup_i. Store overhang_by_option. Return MmpOutcome.
Correction level
If VM-VAR-016 = region, callers run this per region and later aggregate; if country, run once nationally. (Library is agnostic—just operate on the passed maps.)
8) State Flow
Pipeline: after ALLOCATE (locals) and AGGREGATE to correction scope, call mmp_correct to compute top-ups; then continue to gates/frontier and packaging.
9) Determinism & Numeric Rules
Integer/rational math only; half-even rounding only where specified (total-from-share step).
All apportionment uses stable option ordering; no RNG is used in MMP.
10) Edge Cases & Failure Policy
L = 0 with s > 0: T = 0 (no seats to apportion) → all zeros.
vote_totals sum to 0: apportion returns zeros; all top-ups zero; overhang may exist only if locals > 0 (then allow_overhang yields finals=locals; compensate_others gives no top-ups; add_total_seats expands until target>=local which may require large growth—guard with sane cap in params or fail if exceeding limit).
Options appearing in local_seats but not in vote_totals: treat votes=0. Options with votes but no locals get pure top-ups.
Ensure Σ finals equals L + Σ topups and matches intended effective total as per policy.
Protect against overflow: use u128 for intermediate products (e.g., L * 100).
11) Test Checklist (must pass)
Baseline: L=100, s=30% → T≈143 (half-even); Sainte-Laguë apportioning of T with simple votes; deficits compute; totals consistent.
Overhang allow: party X local=60, target=50 ⇒ overhang=10; finals→X=60; effective total > T by 10.
Compensate others: same inputs with compensate_others keep total at T; verify non-overhang parties’ top-ups are re-apportioned and Σ finals = T.
Add total seats: iterative growth yields first Tk where all target>=local; verify minimality (dropping Tk-1 violates some target>=local).
Zero votes: all targets/top-ups zero; finals=locals under allow_overhang; others per policy.
Determinism: permuting input map orders yields identical outcomes.
```
