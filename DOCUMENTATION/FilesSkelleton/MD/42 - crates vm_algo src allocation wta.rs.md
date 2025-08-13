Here’s the clean, no-code skeleton for **File 42 — `crates/vm_algo/src/allocation/wta.rs`**, aligned with your refs.

# Pre-Coding Essentials — 42/89

**Component:** `crates/vm_algo/src/allocation/wta.rs`
**Version/FormulaID:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Winner-take-all (WTA) per Unit: pick the highest-scoring option and allocate **100% power** to it. Enforce **magnitude = 1**.
* **Success:** Deterministic winner; rejects `m ≠ 1`; ties resolved via **VM-VAR-032** (status\_quo / deterministic / random with **VM-VAR-033** seed). Integer-only.

## 2) Scope

* **In scope:** Max-by-score selection, WTA coherence checks, tie breaking, `Allocation { unit_id, seats_or_power }` with 100% to the winner.
* **Out of scope:** Tabulation, gates/frontier, aggregation, schema/I/O.

## 3) Inputs → Outputs

**Inputs**

* `scores : &UnitScores` (from TABULATE; integer tallies)
* `magnitude : u32` (must be **1**)
* `options : &[OptionItem]` (canonical order & `is_status_quo`)
* `tie_policy : TiePolicy`
* `rng : Option<&mut TieRng>` (used **only** when `tie_policy = random`)

**Output**

* `Allocation { unit_id, seats_or_power: { winner → 100 }, last_seat_tie: bool }`

## 4) Entities/Tables (minimal)

* `UnitScores`, `Allocation`, `AllocError`
* `OptionItem { order_index, id, is_status_quo }`
* `TiePolicy`, `TieRng` (ChaCha20 seeded)

## 5) Variables (used here)

* **VM-VAR-032** `tie_policy ∈ {status_quo, deterministic, random}`
* **VM-VAR-033** `tie_seed ∈ u64` (only when `random`)

## 6) Functions (signatures only; no code)

```rust
use std::collections::BTreeMap;
use vm_core::{
  ids::{UnitId, OptionId},
  entities::OptionItem,
  rng::TieRng,
  variables::TiePolicy,
};
use crate::tabulation::UnitScores;

pub fn allocate_wta(
  scores: &UnitScores,
  magnitude: u32,
  options: &[OptionItem],
  tie_policy: TiePolicy,
  rng: Option<&mut TieRng>,
) -> Result<Allocation, AllocError>;

// helpers
fn top_by_score(scores: &UnitScores) -> (u64, Vec<OptionId>); // max score + all tied at max
fn break_tie_wta(
  tied: &[OptionId],
  options: &[OptionItem],
  tie_policy: TiePolicy,
  rng: Option<&mut TieRng>,
) -> OptionId;
```

## 7) Algorithm Outline (implementation plan)

1. **Preconditions**

   * Assert `magnitude == 1`; else `AllocError::InvalidMagnitude`.
2. **Find maximum**

   * Scan `scores.scores` for `max` and collect all option IDs with that value.
3. **Tie handling**

   * If `tied.len() == 1` → winner is that ID.
   * Else apply **tie\_policy**:

     * **status\_quo:** choose candidate with `is_status_quo = true`; if none or multiple, fall back to deterministic.
     * **deterministic:** choose smallest `(order_index, OptionId)` among tied.
     * **random:** draw uniformly with seeded `TieRng` (ChaCha20 from **VM-VAR-033**); log via tie pipeline rules.
4. **Assemble allocation**

   * `seats_or_power = { winner: 100 }` (percent).
   * `last_seat_tie = (tied.len() > 1)`. Return `Allocation`.

## 8) State Flow

* Pipeline: **TABULATE → ALLOCATE (this WTA) → AGGREGATE**.
* Ties, if any, are recorded per tie-logging rules (policy/seed).

## 9) Determinism & Numeric Rules

* Stable option ordering `(order_index, id)`; integer comparisons only.
* RNG path uses **only** the provided seed; same inputs + same seed ⇒ identical outcome/log.

## 10) Edge Cases & Failure Policy

* `magnitude != 1` ⇒ `AllocError::InvalidMagnitude`.
* All scores **zero** ⇒ still select via tie policy (SQ → SQ; else deterministic/random).
* Unknown options should not occur (validated upstream); if encountered: debug assert / `AllocError::UnknownOption` in release.
* Multiple `is_status_quo = true` upstream invalid; fall back to deterministic locally.

## 11) Test Checklist (must pass)

* **VM-TST-002 WTA:** plurality A/B/C/D = 10/20/30/40, `m=1` ⇒ D gets **100%**.
* **Magnitude guard:** `m=2` under WTA ⇒ `InvalidMagnitude`.
* **Status-quo tie:** tie between Change & Status Quo ⇒ Status Quo wins under `status_quo`.
* **Deterministic tie:** same tie with `deterministic` ⇒ lowest `(order_index, id)` wins.
* **Random tie (seeded):** fixed seed ⇒ reproducible winner and TieLog entry.
