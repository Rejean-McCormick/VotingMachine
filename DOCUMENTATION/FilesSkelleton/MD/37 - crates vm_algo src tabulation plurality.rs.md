Here’s a **reference-aligned skeleton sheet** for **37 – crates/vm\_algo/src/tabulation/plurality.rs.md**, tightened to your ten refs and the earlier adjustments (Doc 1B naming: **valid\_ballots/invalid\_ballots**; canonical option order = **order\_index then OptionId**; integers only; no RNG).

````
Pre-Coding Essentials (Component: crates/vm_algo/src/tabulation/plurality.rs, Version FormulaID VM-ENGINE v0) — 37/89

1) Goal & Success
Goal: Deterministically compute UnitScores for plurality ballots from per-option vote counts and turnout.
Success: Returns exact integer scores per OptionId; iteration respects canonical option order (order_index, then OptionId) via the provided options[]; carries turnout (valid_ballots, invalid_balllots, and implied ballots_cast = valid+invalid). No floats, no RNG.

2) Scope
In scope: Per-unit plurality tabulation; non-negativity; sanity that Σ(option votes) ≤ valid_ballots; canonical option ordering.
Out of scope: Allocation, gates/thresholds, aggregation, tie resolution, I/O/schema.

3) Inputs → Outputs
Inputs:
- `unit_id: UnitId`
- `votes: &BTreeMap<OptionId, u64>` (raw per-option counts)
- `turnout: Turnout` (expects fields for `valid_ballots` and `invalid_ballots`; `ballots_cast` is derivable)
- `options: &[OptionItem]` (ordered canonically by `order_index` then `OptionId`)
Output:
- `UnitScores { unit_id, turnout, scores: BTreeMap<OptionId, u64> }`
  (map keys are OptionId; downstream code should iterate using `options` to preserve canonical order)

4) Entities/Tables (minimal)
- Uses vm_core: `UnitId`, `OptionId`, `OptionItem`, `Turnout`, `UnitScores` (from vm_algo public API), and a local `TabError`.

5) Variables (only ones used here)
- None beyond inputs; integers only.

6) Functions (signatures only)
```rust
use std::collections::BTreeMap;
use vm_core::{
    ids::{UnitId, OptionId},
    entities::{Turnout, OptionItem},
};
use crate::UnitScores;

#[derive(Debug)]
pub enum TabError {
    UnknownOption(OptionId),
    TallyExceedsValid { sum_votes: u64, valid_ballots: u64 },
}

/// Deterministic plurality tabulation (integers only; no RNG).
pub fn tabulate_plurality(
    unit_id: UnitId,
    votes: &BTreeMap<OptionId, u64>,
    turnout: Turnout,
    options: &[OptionItem],
) -> Result<UnitScores, TabError>;

/// Internal: build a canonical score map from provided votes and option list.
/// Iterates `options` in (order_index, OptionId) order; missing keys → 0; rejects unknown vote keys.
fn canonicalize_scores(
    votes: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> Result<(BTreeMap<OptionId, u64>, u64 /*sum*/), TabError>;

/// Internal sanity checks (Σ option votes ≤ valid_ballots).
fn check_tally_sanity(sum_votes: u64, turnout: &Turnout) -> Result<(), TabError>;
````

7. Algorithm Outline (implementation plan)

* **Canonical order**

  * Iterate `options` in their canonical order (already `(order_index, OptionId)`).
  * For each `opt.id`, read `votes.get(&opt.id).copied().unwrap_or(0)`; accumulate `sum_votes`.
  * Insert `(opt.id, count)` into a fresh `BTreeMap<OptionId, u64>` (key order is lexicographic by OptionId; **iteration order for downstream must use `options`**, not the map).

* **Unknown keys guard**

  * If `votes` contains any `OptionId` not present in `options`, return `TabError::UnknownOption(...)`.
    (Loader should already prevent this; this remains a defensive check.)

* **Sanity**

  * `valid_ballots = turnout.valid_ballots()`.
  * Ensure `sum_votes ≤ valid_ballots`; else `TabError::TallyExceedsValid { … }`.
  * Non-negativity is guaranteed by `u64`.

* **Assemble**

  * Return `UnitScores { unit_id, turnout, scores }`.

* **No normalization**

  * Do not compute shares/percentages; downstream gates/allocation use integers or `Ratio`.

8. State Flow
   Pipeline: TABULATE (this) → ALLOCATE (WTA/PR) → AGGREGATE → GATES. `UnitScores` feeds allocation and later gates/labels.

9. Determinism & Numeric Rules

* Determinism via canonical option traversal and stable `BTreeMap` storage.
* Integer math only; no rounding; no RNG.

10. Edge Cases & Failure Policy

* Missing option in `votes` ⇒ treated as 0.
* Extra option present in `votes` but not in `options` ⇒ `TabError::UnknownOption`.
* `sum_votes > valid_ballots` ⇒ `TabError::TallyExceedsValid`.
* `turnout.valid_ballots == 0` ⇒ all zeros; gates handle legitimacy.

11. Test Checklist (must pass)

* **Happy path**: A/B/C/D votes (e.g., 10/20/30/40), `valid_ballots = 100`, `invalid_ballots = 0` → scores equal input; iteration order follows `options`.
* **Missing key**: absent option in `votes` yields 0 without panic.
* **Unknown key**: `votes` contains OPT\:X not in `options` → `UnknownOption`.
* **Sanity**: Σ(votes) == valid\_ballots passes; Σ(votes) > valid\_ballots fails with `TallyExceedsValid`.
* **Determinism**: Permute insertion order of `votes` → identical `UnitScores` content; iterating via `options` yields a stable sequence and canonical bytes after serialization.

```

If you want, I can mirror this with a minimal `plurality.rs` stub (enum, fns, and TODOs) that compiles against your `vm_core`/`vm_algo` scaffolds.
```
