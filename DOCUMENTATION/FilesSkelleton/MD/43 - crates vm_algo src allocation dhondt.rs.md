
````
Pre-Coding Essentials (Component: crates/vm_algo/src/allocation/dhondt.rs, Version FormulaID VM-ENGINE v0) — 43/89

1) Goal & Success
Goal: Implement D’Hondt (highest averages) per Unit: sequentially award seats using divisors 1,2,3,… after applying the PR entry threshold.
Success: For magnitude m, returns a seat vector summing to m; below-threshold options excluded; last-seat ties resolved per policy; pure integer math.

2) Scope
In scope: Per-Unit D’Hondt allocation; threshold filter; quotient selection loop; deterministic/reproducible tie handling; stable ordering by (order_index, OptionId).
Out of scope: Tabulation, aggregation, gates/frontier, I/O/schema.

3) Inputs → Outputs
Inputs:
- `seats: u32` (Unit.magnitude; ≥1)
- `scores: &BTreeMap<OptionId, u64>` (natural tallies from tabulation)
- `options: &[OptionItem]` (canonical order; includes `order_index`, `is_status_quo`)
- `threshold_pct: u8` (PR entry threshold; schema caps per Annex A; integers 0..=100; engine cap ≤10 per spec)
- `tie_policy: TiePolicy` (VM-VAR-050)
- `rng: Option<&mut TieRng>` (used only when `tie_policy = Random`; seed from VM-VAR-052 upstream)
Output:
- `BTreeMap<OptionId, u32>` seats per option (sum = seats)

4) Entities/Tables (minimal)
(Uses `vm_core::{ids::OptionId, entities::OptionItem, variables::TiePolicy, rng::TieRng}`.)

5) Variables (used here)
- **VM-VAR-050** `tie_policy` ∈ { `status_quo`, `deterministic`, `random` }
- **VM-VAR-052** `tie_seed` ∈ integers (≥ 0) — used upstream to build `TieRng`

6) Functions (signatures only)
```rust
use std::collections::BTreeMap;
use vm_core::{
    ids::OptionId,
    entities::OptionItem,
    rng::TieRng,
    variables::TiePolicy,
};

#[derive(Debug)]
pub enum AllocError {
    NoEligibleOptions,
    MissingRngForRandomPolicy,
}

pub fn allocate_dhondt(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],      // canonical (order_index, id)
    threshold_pct: u8,
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> Result<BTreeMap<OptionId, u32>, AllocError>;

// ---- helpers ----

/// Filter by PR threshold using the ballot’s natural totals (plurality: vote share; approval: approvals share; score: score-sum share).
fn filter_by_threshold(
    scores: &BTreeMap<OptionId, u64>,
    threshold_pct: u8,
) -> BTreeMap<OptionId, u64>;

/// Choose argmax of v/(s+1) across eligible; ties resolved per policy (SQ → SQ; else deterministic order; random uses rng).
fn next_award(
    seats_so_far: &BTreeMap<OptionId, u32>,
    eligible_scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> OptionId;

/// Compare D’Hondt quotients q_a = v_a/(s_a+1) vs q_b = v_b/(s_b+1) without floats (u128 cross-multiply).
fn cmp_quotients(
    v_a: u64, s_a: u32,
    v_b: u64, s_b: u32,
) -> core::cmp::Ordering;

/// Deterministic tie-break among candidates (order_index, then OptionId).
fn deterministic_pick(
    tied: &[OptionId],
    options: &[OptionItem],
) -> OptionId;

/// Status-quo resolver: pick unique `is_status_quo==true` within `tied`; otherwise fall back to deterministic.
fn status_quo_pick(
    tied: &[OptionId],
    options: &[OptionItem],
) -> OptionId;
````

7. Algorithm Outline (implementation plan)

* **Threshold filter**

  * Let `total = Σ scores.values()` (natural totals for the ballot family already in tallies).
  * Keep `(opt, v)` where `100 * v ≥ threshold_pct * total` (use u128 for products).
  * If none remain and `seats > 0` ⇒ `AllocError::NoEligibleOptions`.

* **Initialize**

  * `alloc[opt] = 0` for every eligible option (BTreeMap).
  * Build `canon_order` = `options` sorted by `(order_index, OptionId)`; keep quick lookup for `is_status_quo`.

* **Seat loop** (repeat `seats` times)

  * For each eligible `opt`, compute quotient via comparison only (no division): compare candidates using `cmp_quotients(v, s+1, v’, s’+1)`.
  * Track the current best; collect all with equal max.
  * **Tie handling**:

    * If exactly one best ⇒ award it.
    * Else resolve:

      * `TiePolicy::StatusQuo` → `status_quo_pick`, else
      * `TiePolicy::Deterministic` → `deterministic_pick`, else
      * `TiePolicy::Random` → require `rng` and draw uniformly; else `MissingRngForRandomPolicy`.
  * Increment seat for chosen option: `alloc[winner] += 1`.

* **Finish**

  * Return `alloc` (sum must equal `seats`).

8. State Flow
   Called by AllocateUnit after Tabulate; precedes aggregation. Threshold and tie policy adhere to Annex A / Doc 4. Tie events are logged at the pipeline level.

9. Determinism & Numeric Rules

* Integer-only comparisons; `cmp_quotients` uses u128 cross-multiplication to avoid overflow.
* Deterministic tie handling via canonical order; random ties depend solely on injected `TieRng`.

10. Edge Cases & Failure Policy

* `seats == 0` ⇒ return empty map.
* After threshold no options ⇒ `NoEligibleOptions`.
* All zero scores with seats > 0 ⇒ all rounds are ties; resolve each per policy (SQ → SQ if unique; else deterministic/random).
* Products for comparisons use u128 to avoid overflow on large inputs.

11. Test Checklist (must pass)

* Convergence: A/B/C = 34/33/33, m=7 ⇒ 3/2/2.
* Baseline: A/B/C/D = 10/20/30/40, m=10 → matches D’Hondt (distinct from Sainte-Laguë fixture).
* Threshold: with `threshold_pct > 0`, below-threshold options receive 0 seats and are never considered.
* Determinism: permuting input map order yields identical result due to canonical ordering.
* Tie behavior: craft equal-quotient round; check `status_quo`, `deterministic`, and `random` (with fixed seed) each yield the expected, reproducible pick.

```

```
