
````
Pre-Coding Essentials (Component: crates/vm_algo/src/allocation/largest_remainder.rs, Version FormulaID VM-ENGINE v0) — 45/89

1) Goal & Success
Goal: Implement Largest Remainder (LR) with selectable quota (Hare, Droop, Imperiali) after applying the PR entry threshold.
Success: Floors + remainder distribution sum exactly to m; below-threshold options excluded; Imperiali over-allocation trimmed deterministically; ties resolved per VM-VAR-050; integer-only math.

2) Scope
In scope: Threshold filter; quota computation (Hare/Droop/Imperiali); floors; remainder ranking and assignment; deterministic tie-breaking; Imperiali trim path.
Out of scope: Tabulation, aggregation, gates/frontier, any I/O/schema.

3) Inputs → Outputs
Inputs:
- `seats: u32` (m ≥ 0)
- `scores: &BTreeMap<OptionId, u64>` (natural tallies from tabulation)
- `options: &[OptionItem]` (canonical order `(order_index, id)`, includes `is_status_quo`)
- `threshold_pct: u8` (entry threshold % in 0..=100; spec cap ≤10 enforced upstream)
- `quota: QuotaKind` (Hare | Droop | Imperiali)
- `tie_policy: TiePolicy` (VM-VAR-050)
- `rng: Option<&mut TieRng>` (used only when `tie_policy = Random`; seed from VM-VAR-052 integer upstream)
Output:
- `BTreeMap<OptionId, u32>` seats per option (sum = `seats`)

4) Entities/Tables (minimal)
Uses `vm_core::{ids::OptionId, entities::OptionItem, variables::TiePolicy, rng::TieRng}`.

5) Variables (used here)
- VM-VAR-050 `tie_policy` ∈ { `status_quo`, `deterministic`, `random` }
- VM-VAR-052 `tie_seed` ∈ integers (≥ 0), used upstream to construct `TieRng`.

6) Functions (signatures only)
```rust
use std::collections::BTreeMap;
use vm_core::{
    ids::OptionId,
    entities::OptionItem,
    rng::TieRng,
    variables::TiePolicy,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum QuotaKind { Hare, Droop, Imperiali }

#[derive(Debug)]
pub enum AllocError {
    NoEligibleOptions,
    MissingRngForRandomPolicy,
}

pub fn allocate_largest_remainder(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],       // canonical (order_index, id)
    threshold_pct: u8,
    quota: QuotaKind,
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> Result<BTreeMap<OptionId, u32>, AllocError>;

// ---- helpers ----

/// Keep options whose natural share meets threshold: 100*v >= threshold_pct*total (u128 math).
fn filter_by_threshold(
    scores: &BTreeMap<OptionId, u64>,
    threshold_pct: u8,
) -> BTreeMap<OptionId, u64>;

/// Integer-only quota:
/// Hare: floor(V / m)
/// Droop: floor(V / (m + 1)) + 1
/// Imperiali: floor(V / (m + 2))
fn compute_quota(total: u128, seats: u128, quota: QuotaKind) -> u128;

/// Compute floors and remainders given quota q (u128 math; q==0 handled).
fn floors_and_remainders(
    eligible: &BTreeMap<OptionId, u64>,
    q: u128,
) -> (BTreeMap<OptionId, u32>, BTreeMap<OptionId, u128>);

/// Assign remaining seats by largest remainder; ties per policy (SQ → status_quo; else deterministic; random uses rng).
fn distribute_leftovers(
    target_seats: u32,
    alloc: &mut BTreeMap<OptionId, u32>,
    remainders: &BTreeMap<OptionId, u128>,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
);

/// Imperiali edge: if floors sum > target, trim from smallest remainder until sum == target (ties per policy).
fn trim_over_allocation_if_needed(
    target_seats: u32,
    alloc: &mut BTreeMap<OptionId, u32>,
    remainders: &BTreeMap<OptionId, u128>,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> bool;

// Tie helpers
fn deterministic_pick(tied: &[OptionId], options: &[OptionItem]) -> OptionId;
fn status_quo_pick(tied: &[OptionId], options: &[OptionItem]) -> OptionId;
````

7. Algorithm Outline (implementation plan)

* **Threshold filter**

  * `total = Σ scores.values()`.
  * Keep `(opt, v)` with `100 * v ≥ threshold_pct * total` (use u128 products).
  * If none eligible and `seats > 0` ⇒ `AllocError::NoEligibleOptions`.

* **Quota (`q`)**

  * Let `V = total as u128`, `m = seats as u128`.
  * Compute by `compute_quota(V, m, quota)`.
  * **q == 0 handling**: treat all floors as 0 and remainder as the full score (so distribution proceeds by remainders).

* **Floors & remainders**

  * For each eligible option:

    * If `q > 0`: `floor_i = (v_i as u128 / q) as u32`, `rem_i = (v_i as u128 % q)`.
    * If `q == 0`: `floor_i = 0`, `rem_i = v_i as u128`.
  * `sum_floors = Σ floor_i`.

* **Distribution / Trim**

  * If `sum_floors < seats`: assign `seats - sum_floors` leftovers one-by-one to **largest remainder**.

    * Ranking key: `(remainder desc, raw_score desc, canonical (order_index, id))`.
    * If still tied and `tie_policy = Random`, draw uniformly using `rng`; if `rng` is None ⇒ `MissingRngForRandomPolicy`.
    * If `tie_policy = StatusQuo` and exactly one of the tied has `is_status_quo`, pick it; else fall back to deterministic ranking.
  * If `sum_floors > seats` (possible under Imperiali with tiny totals):

    * Trim seats starting from **smallest remainder** until total equals `seats`.
    * Ties on smallest remainder resolved with inverse of the above ranking (asc remainder, asc raw score, then canonical) or via policy (SQ/deterministic/random) consistently.

* **Return**

  * Deterministic `BTreeMap<OptionId, u32>` with `Σ alloc == seats`.

8. State Flow
   Called by AllocateUnit after Tabulate; before aggregation. Threshold behavior and tie handling match Doc 4; pipeline performs logging of tie events; this function returns only the seat vector.

9. Determinism & Numeric Rules

* Integer-only math; u128 for multiplications/divisions; no floats.
* Canonical option order `(order_index, OptionId)` governs deterministic choices.
* Random ties depend solely on injected `TieRng` (seeded from VM-VAR-052); identical inputs + seed ⇒ identical outcomes.

10. Edge Cases & Failure Policy

* `seats == 0` ⇒ return empty map.
* After threshold: no eligible options ⇒ `NoEligibleOptions`.
* `total == 0` with `seats > 0` ⇒ all remainders 0; allocate entirely by tie policy/ranking.
* Imperiali over-allocation ⇒ trimming path reduces seats deterministically (or via seeded RNG when requested).

11. Test Checklist (must pass)

* **Convergence (VM-TST-003)**: A/B/C = 34/33/33, `m=7` ⇒ `3/2/2`.
* **Droop boundary**: `V=90, m=4` → `q=19`; with votes `{A:50, B:28, C:12}` floors+remainders yield total 4 deterministically.
* **Imperiali trim**: `V=3, m=1` → `q=1`; floors `1,1,1` (sum 3) ⇒ trim to 1 seat by smallest remainder (all equal → canonical order unless policy=random).
* **Threshold**: raising `threshold_pct` excludes sub-threshold options from any seat award.
* **Determinism**: permuting input map insertion order yields identical allocation due to canonical order; with `Random` + fixed seed, selection is reproducible.

```

```
