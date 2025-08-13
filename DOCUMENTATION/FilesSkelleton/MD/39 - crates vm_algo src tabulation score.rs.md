
# Pre-Coding Essentials — 39/89

**Component:** `crates/vm_algo/src/tabulation/score.rs`
**Version/FormulaID:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Deterministically tabulate **score ballots** per unit from **already-summed per-option score totals** and **turnout**, validating scale/normalization constraints.
* **Success:** Produce `UnitScores` with exact integer sums keyed by `OptionId`, **canonical option order**, and **turnout**. Enforce **scale/domain sanity** and **caps**. No floats, no RNG.

## 2) Scope

* **In scope:**

  * Consume pre-aggregated `score_sums` (per-option totals).
  * Validate against `Params` (VM-VAR-002 min, VM-VAR-003 max, VM-VAR-004 normalization policy).
  * Canonicalize to the declared option list.
  * Plausibility caps vs `valid_ballots`.
* **Out of scope:**

  * Per-ballot normalization from raw ballots (lives higher up if raw ballots exist).
  * Allocation, gates/threshold math, aggregation, tie logic, I/O/schema.

## 3) Inputs → Outputs

* **Inputs:**

  * `unit_id : UnitId`
  * `score_sums : &BTreeMap<OptionId, u64>` (per-option totals for this unit)
  * `turnout : Turnout` (`ballots_cast`, `invalid_or_blank`, `valid_ballots`)
  * `params : &Params` (reads VM-VAR-002..004)
  * `options : &[OptionItem]` (defines canonical `(order_index, id)` order)
* **Output:**

  * `UnitScores { unit_id, turnout, scores: BTreeMap<OptionId, u64> }` (iteration matches canonical option order)

## 4) Entities/Tables (minimal)

* Uses core types: `UnitId`, `OptionId`, `Turnout`, `OptionItem`, `Params`, `UnitScores`.
* Error type: `TabError` (variants used below).

## 5) Variables (used here)

* `min_scale = VM-VAR-002` (inclusive per-ballot minimum).
* `max_scale = VM-VAR-003` (inclusive per-ballot maximum).
* `norm_policy = VM-VAR-004` (e.g., `off` | `linear` per platform spec).
* `V = turnout.valid_ballots`.

## 6) Functions (signatures only; **no code here**)

* `tabulate_score(...) -> UnitScores`
* `canonicalize_scores(score_sums, options) -> Result<BTreeMap<OptionId,u64>, TabError>`
* `check_scale_and_caps(scores, turnout, params) -> Result<(), TabError>`

## 7) Algorithm Outline (implementation plan)

1. **Canonical order**

   * Iterate `options` in `(order_index, id)` order.
   * For each option: take `score_sums.get(id)` or `0` if missing.
   * **Reject** any key present in `score_sums` that is **not** in `options` (referential integrity).
2. **Scale sanity (params)**

   * Read `min_scale` and `max_scale`; require `min_scale < max_scale` (inclusive per-ballot bounds).
   * This function **does not** reconstruct per-ballot vectors; it validates **aggregate plausibility** only.
3. **Caps / plausibility checks**

   * Let `V = turnout.valid_ballots`.
   * If `V == 0`: **all option sums must be `0`** → else `TabError::InconsistentTurnout`.
   * Compute cap per option as `V * max_scale` (use widened arithmetic to avoid overflow in the check).
   * For every option: `sum_i ≤ V * max_scale` → else `TabError::OptionExceedsCap`.
   * Negative values are impossible (`u64`).
   * **Normalization policy note:**

     * If `norm_policy = off`: caps apply as above.
     * If `norm_policy = linear` (per-ballot normalization to span), aggregates are assumed already normalized; the **same cap** `V * max_scale` still applies.
4. **Assemble**

   * Return `UnitScores { unit_id, turnout, scores }`.
   * **No ratios or percentages here.** Gates/labels later compute any `%` they need.
5. **Interoperability (downstream)**

   * For binary “Change vs SQ” checks on score ballots, later gates compute:
     `support% = score_sum_for_change / (max_scale * V)` using integer/rational math outside this function.

## 8) State Flow

* Pipeline position: **TABULATE (score)** → **ALLOCATE** (PR/WTA/etc.) → **AGGREGATE** → **GATES**.
* `UnitScores` is the sole artifact from this file.

## 9) Determinism & Numeric Rules

* Determinism via canonical option iteration and `BTreeMap` storage.
* **Integer‐only** comparisons; no floats; no RNG.
* Overflow safety: compute `V * max_scale` using widened integer (e.g., `u128`) for the comparison, then compare safely to `u64` sums.

## 10) Edge Cases & Failure Policy

* **Unknown OptionId** present in `score_sums` ⇒ `TabError::UnknownOption`.
* **V = 0** but any non-zero per-option sum ⇒ `TabError::InconsistentTurnout`.
* Any option sum **> V \* max\_scale** ⇒ `TabError::OptionExceedsCap`.
* Missing option keys in `score_sums` are treated as `0` (not an error).

## 11) Test Checklist (must pass)

* **Happy path:** `min=0, max=5`, `V=100`, sums within caps → returns identical sums in canonical order.
* **Caps:** `V=50`, `max=5`: any option sum `> 250` ⇒ fail with `OptionExceedsCap`.
* **Zero valid ballots:** `V=0` and any non-zero sum ⇒ `InconsistentTurnout`; otherwise all zeros accepted.
* **Unknown option key:** present in `score_sums` but not `options` ⇒ `UnknownOption`.
* **Determinism:** shuffle input map insertion order and `options` vector → identical `UnitScores.scores` iteration order and bytes when serialized canonically.

---

### Notes on Corrections vs prior attempt

* Made the **V=0** rule explicit: **all** per-option sums must be `0`, otherwise error.
* Clarified **normalization**: this function assumes **pre-summed** inputs; regardless of policy, caps use `V * max_scale`.
* Strengthened **overflow** guidance (widened arithmetic for `V * max_scale`).
* Tightened **UnknownOption** handling: reject extraneous keys; missing keys default to `0`.
* Emphasized **no percentages here** and pointed to the precise downstream support% formula.
