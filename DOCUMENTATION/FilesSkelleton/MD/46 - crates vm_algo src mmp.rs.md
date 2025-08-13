
````
Pre-Coding Essentials (Component: crates/vm_algo/src/mmp.rs, Version FormulaID VM-ENGINE v0) — 46/89

1) Goal & Success
Goal: Mixed-Member Proportional (MMP) helpers to (a) compute seat targets from vote totals, (b) derive top-ups against local seats, and (c) apply overhang policy and total-seats model.
Success: Pure integer/rational math with round-half-to-even only where allowed; deterministic results; respects Annex-A variables (VM-VAR-013..017). Outputs sum correctly under the chosen policy.

2) Scope
In scope: total seats from top-up share; proportional target apportionment; top-up computation; overhang handling (allow/compensate/add-seats).
Out of scope: reading ballots/locals (caller passes counts), PR inside units, reporting/formatting.

3) Inputs → Outputs
Inputs:
- `vote_totals: BTreeMap<OptionId, u64>` — list/national votes (scope depends on correction level)
- `local_seats: BTreeMap<OptionId, u32>` — already awarded “local” seats
- `base_total_local: u32` — Σ local seats within the correction scope
- `params: &Params` — reads **VM-VAR-013..017** only (013 topup_share_pct, 014 overhang_policy, 015 target_share_basis=natural_vote_share (fixed), 016 correction_level, 017 total_seats_model)
- `method_for_targets: AllocationMethod` — method used to apportion targets (typically Sainte-Laguë)
Outputs:
- `targets: BTreeMap<OptionId, u32>` — intended total seats by option at the correction scope
- `topups: BTreeMap<OptionId, u32>` — max(0, target − local)
- `finals: BTreeMap<OptionId, u32>` — local + topup (after policy/model)
- `effective_total_seats: u32` — final seat count after policy/model
- `overhang_by_option: BTreeMap<OptionId, u32>` — diagnostic max(0, local − target)

4) Entities/Tables (minimal)
Uses `vm_core::{ids::OptionId, variables::{Params, AllocationMethod, OverhangPolicy, TotalSeatsModel}, rounding::round_nearest_even_int}`.

5) Variables (Annex-A; used here)
- **VM-VAR-013** `mlc_topup_share_pct: u8 0..=60` (cap enforced upstream)
- **VM-VAR-014** `overhang_policy: OverhangPolicy` ∈ { allow_overhang, compensate_others, add_total_seats }
- **VM-VAR-015** `target_share_basis: enum` — **fixed** to `natural_vote_share` (no alternative basis here)
- **VM-VAR-016** `mlc_correction_level: enum` ∈ { country, region } (affects caller’s scoping)
- **VM-VAR-017** `total_seats_model: TotalSeatsModel` (e.g., fixed_house, expandable)

6) Functions (signatures only)
```rust
use std::collections::BTreeMap;
use vm_core::{
    ids::OptionId,
    variables::{Params, AllocationMethod, OverhangPolicy, TotalSeatsModel},
    rounding::round_nearest_even_int,
};

pub struct MmpOutcome {
    pub targets: BTreeMap<OptionId, u32>,
    pub topups: BTreeMap<OptionId, u32>,
    pub finals: BTreeMap<OptionId, u32>,
    pub effective_total_seats: u32,
    pub overhang_by_option: BTreeMap<OptionId, u32>,
}

/// T ≈ L / (1 - s), where s = VM-VAR-013 / 100. Uses banker’s rounding (half-even).
pub fn compute_total_from_share(local_total: u32, topup_share_pct: u8) -> u32;

/// Apportion `total_seats` to options from `vote_totals` via `method` (Sainte-Laguë/D’Hondt).
pub fn apportion_targets(
    total_seats: u32,
    vote_totals: &BTreeMap<OptionId, u64>,
    method: AllocationMethod,
) -> BTreeMap<OptionId, u32>;

/// Given `targets` and `local_seats`, compute top-ups and apply overhang policy/model.
/// May expand total seats (add_total_seats) via minimal iterative growth.
pub fn compute_topups_and_apply_overhang(
    targets: &BTreeMap<OptionId, u32>,
    local_seats: &BTreeMap<OptionId, u32>,
    overhang_policy: OverhangPolicy,
    total_seats_model: TotalSeatsModel,
    method_for_targets: AllocationMethod,
    vote_totals: &BTreeMap<OptionId, u64>,
) -> MmpOutcome;

/// One-shot orchestration for a correction scope (country or region).
pub fn mmp_correct(
    vote_totals: &BTreeMap<OptionId, u64>,
    local_seats: &BTreeMap<OptionId, u32>,
    params: &Params,
    method_for_targets: AllocationMethod,
) -> MmpOutcome;

// ---- Wrappers to align with vm_algo::lib.rs public API ----

/// Wrapper: targets from `total_seats` using `method`.
pub fn mmp_target_shares(
    total_seats: u32,
    vote_totals: &BTreeMap<OptionId, u64>,
    method: AllocationMethod,
) -> BTreeMap<OptionId, u32>;

/// Wrapper: top-ups only (compute from targets+locals with policy/model).
pub fn mmp_topups(
    local_seats: &BTreeMap<OptionId, u32>,
    targets: &BTreeMap<OptionId, u32>,
    overhang_policy: OverhangPolicy,
    total_seats_model: TotalSeatsModel,
    method_for_targets: AllocationMethod,
    vote_totals: &BTreeMap<OptionId, u64>,
) -> BTreeMap<OptionId, u32>;
````

7. Algorithm Outline (implementation plan)

* **Total from top-up share**

  * Let `L = local_total`, `p = topup_share_pct (0..=60)`.
  * Compute `T = round_nearest_even_int( (L * 100) as i128, (100 - p) as i128 )` cast to `u32`.
  * Guards (domain enforced upstream in `Params`): `p < 100`; if `L=0` then `T=0`.

* **Target apportionment**

  * Deterministically apportion `T` seats to options by `method_for_targets`:

    * Use canonical option order `(order_index → OptionId)`.
    * Highest-averages methods compare quotients via u128 cross-multiplication; no floats.
  * Result is `targets: BTreeMap<OptionId, u32>` with sum = `T`.

* **Top-up deficits & overhang (diagnostics)**

  * For each option `i`: `deficit_i = max(0, target_i - local_i)`.
  * `overhang_i = max(0, local_i - target_i)`.

* **Overhang policy + total seats model**

  * `allow_overhang`:

    * Keep `T` as intended; set `topup_i = deficit_i`.
    * `finals_i = local_i + topup_i`; `effective_total = T + Σ overhang_i`.
  * `compensate_others` (fixed house size):

    * Keep `effective_total = T`.
    * Seat pool for top-ups is `T - Σ local_i` (may be < Σ deficits).
    * Re-apportion this pool across **non-overhang** options using `method_for_targets` with weights (prefer `deficit_i`, or equivalently vote shares among eligible non-overhang options). Overhang options receive 0 top-ups.
  * `add_total_seats` (expand to clear overhang):

    * Initialize `Tk = T`.
    * While ∃ `i` with `apportion_targets(Tk, votes)[i] < local_i`:

      * `Tk += 1`; recompute targets.
    * Set `topup_i = target_i(Tk) - local_i`; `effective_total = Tk`; `finals = local + topups`.

* **Assemble**

  * Return `MmpOutcome { targets, topups, finals, effective_total_seats, overhang_by_option }`.

* **Correction level (VM-VAR-016)**

  * If `region`: caller runs `mmp_correct` per region; results later aggregate.
  * If `country`: run once nationally.

8. State Flow
   After local allocations and aggregation to the chosen correction scope, call `mmp_correct` (or manual sequence) to compute top-ups/finals. Pipeline proceeds to gates/frontier and packaging. Tie logs are irrelevant (no RNG in MMP).

9. Determinism & Numeric Rules

* Integer/rational math only; `round_nearest_even_int` solely at total-from-share step.
* Stable ordering ensures identical outputs across OS/arch.
* No randomization; apportionment uses canonical comparisons.

10. Edge Cases & Failure Policy

* `L=0` with `p>0` → `T=0`; all outputs zero.
* `Σ votes = 0`:

  * `apportion_targets(T, votes)` returns zeros.
  * `allow_overhang`: finals = locals; effective\_total = T + Σ overhang.
  * `compensate_others`: no top-ups available; finals = locals (house size stays T).
  * `add_total_seats`: iteratively grows until each `target_i ≥ local_i` (guard with sane cap in Params; fail/abort if exceeding cap—enforced upstream).
* Options present in locals but not in votes: treat `votes=0`.
* Sum invariants: `Σ finals = effective_total_seats` (except `allow_overhang`, where `effective_total = T + Σ overhang` by definition).

11. Test Checklist (must pass)

* Share → total: `L=100, p=30` ⇒ `T = round_half_even(100/0.7) = 143`.
* Targets apportionment (Sainte-Laguë) deterministic and sums to `T`.
* Overhang allow: `local_X=60, target_X=50` ⇒ `overhang_X=10`; `effective_total = T + 10`.
* Compensate others: total stays `T`; non-overhang top-ups re-apportioned; `Σ finals = T`.
* Add total seats: minimal `Tk` where all `target_i(Tk) ≥ local_i`; `Tk−1` violates; `Σ finals = Tk`.
* Zero votes: targets all zero; outcomes per policy above; deterministic across runs.
* Map order insensitivity: permuting input BTreeMap insertions yields identical outputs.

```


```
