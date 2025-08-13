
````
Pre-Coding Essentials (Component: crates/vm_algo/src/allocation/sainte_lague.rs, Version FormulaID VM-ENGINE v0) — 44/89

1) Goal & Success
Goal: Implement Sainte-Laguë (highest averages favoring smaller parties): sequential awards using odd divisors 1,3,5,… after applying the PR entry threshold.
Success: For Unit magnitude m, returns a seat vector summing to m; below-threshold options excluded; ties resolved per policy; pure integer math.

2) Scope
In scope: Per-Unit Sainte-Laguë allocation; threshold filter; quotient loop with odd divisors; deterministic/reproducible tie handling via canonical order or seeded RNG.
Out of scope: Tabulation, aggregation, gates/frontier, any I/O/schema.

3) Inputs → Outputs
Inputs:
- `seats: u32` (Unit.magnitude; ≥ 0)
- `scores: &BTreeMap<OptionId, u64>` (natural tallies from tabulation)
- `options: &[OptionItem]` (provides `(order_index, id)` and `is_status_quo`)
- `threshold_pct: u8` (entry threshold, % in 0..=100; engine cap for spec is enforced elsewhere)
- `tie_policy: TiePolicy` (VM-VAR-050)
- `rng: Option<&mut TieRng>` (used iff `tie_policy = Random`, seeded upstream from VM-VAR-052 integer)
Output:
- `BTreeMap<OptionId, u32>` (seats per option; sum == `seats`)

4) Entities/Tables (minimal)
Uses `vm_core::{ids::OptionId, entities::OptionItem, variables::TiePolicy, rng::TieRng}`.

5) Variables (used here)
- VM-VAR-050 `tie_policy` ∈ { `status_quo`, `deterministic`, `random` }
- VM-VAR-052 `tie_seed` ∈ integers (≥ 0), used upstream to create `TieRng`.

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

pub fn allocate_sainte_lague(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],       // canonical (order_index, id)
    threshold_pct: u8,
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> Result<BTreeMap<OptionId, u32>, AllocError>;

// ---- helpers ----

/// Keep options whose natural share meets the threshold: 100*v >= threshold_pct*total (u128 math).
fn filter_by_threshold(
    scores: &BTreeMap<OptionId, u64>,
    threshold_pct: u8,
) -> BTreeMap<OptionId, u64>;

/// Argmax of Sainte-Laguë quotients v / (2*s + 1); ties per policy.
fn next_award(
    seats_so_far: &BTreeMap<OptionId, u32>,
    eligible_scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> OptionId;

/// Compare quotients q_a = v_a / (2*s_a+1) vs q_b = v_b / (2*s_b+1) using u128 cross-multiplication.
fn cmp_quotients(
    v_a: u64, s_a: u32,
    v_b: u64, s_b: u32,
) -> core::cmp::Ordering;

/// Deterministic fallback: pick smallest by (order_index, then OptionId).
fn deterministic_pick(tied: &[OptionId], options: &[OptionItem]) -> OptionId;

/// Status-quo resolver: if exactly one `is_status_quo` in `tied`, return it; else fall back to deterministic.
fn status_quo_pick(tied: &[OptionId], options: &[OptionItem]) -> OptionId;
````

7. Algorithm Outline (implementation plan)

* **Threshold filter**

  * Compute `total = Σ scores.values()`.
  * Keep `(opt, v)` where `100 * v ≥ threshold_pct * total` (use u128 for products).
  * If `seats > 0` and none eligible ⇒ `AllocError::NoEligibleOptions`.

* **Initialize**

  * `alloc[opt] = 0` for each eligible option (BTreeMap).
  * Build `canon_order` = options sorted by `(order_index, OptionId)`; keep map for `is_status_quo`.

* **Seat loop** (repeat `seats` times)

  * For each eligible `opt`, compute Sainte-Laguë divisor `d = 2*alloc[opt] + 1`.
  * Select argmax of `v/d` by comparing `v_a * d_b` vs `v_b * d_a` using u128.
  * If one best ⇒ award it; else resolve tie:

    * `TiePolicy::StatusQuo` → `status_quo_pick`; if not decisive, fall back to deterministic.
    * `TiePolicy::Deterministic` → `deterministic_pick`.
    * `TiePolicy::Random` → require `rng` and pick uniformly; else `MissingRngForRandomPolicy`.
  * Increment: `alloc[winner] += 1`.

* **Finish**

  * Return `alloc` (sum equals `seats`).

8. State Flow
   Called by AllocateUnit after Tabulate; precedes aggregation. Threshold and tie handling conform to Doc 4; tie events are logged by the pipeline (not here).

9. Determinism & Numeric Rules

* Integer-only comparisons; u128 cross-multiplication prevents overflow.
* Stable order via `(order_index, OptionId)` for deterministic selections.
* Random ties depend solely on injected `TieRng` (seeded from VM-VAR-052); identical inputs + seed ⇒ identical outcomes.

10. Edge Cases & Failure Policy

* `seats == 0` ⇒ return empty map.
* No eligible options ⇒ `NoEligibleOptions`.
* All zero scores with `seats > 0` ⇒ every round is a tie; resolve per policy (SQ → status-quo if unique; else deterministic/random).
* Products use u128 to avoid overflow on extreme inputs.

11. Test Checklist (must pass)

* **VM-TST-001**: A/B/C/D = 10/20/30/40, `m=10` ⇒ seats `1/2/3/4`.
* **Convergence**: A/B/C = 34/33/33, `m=7` ⇒ seats `3/2/2`.
* **Threshold**: raising `threshold_pct` excludes sub-threshold options from any award.
* **Determinism**: permuting input map insertion order yields the same allocation due to canonical ordering.
* **Tie behavior**: craft equal-quotient round; verify status-quo, deterministic, and random (with fixed seed) each behave as specified and are reproducible.

```
```
