

# Pre-Coding Essentials — 40/89

**Component:** `crates/vm_algo/src/tabulation/ranked_irv.rs`
**Version/FormulaID:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Deterministically tabulate **IRV** per unit from compressed ranked ballot groups, with a fixed exhaustion policy and a round-by-round audit log.
* **Success:** Stop when a candidate reaches **majority of continuing ballots** or only one continues. Emit **IrvLog** (eliminations, transfers, exhausted). Integer math only; **no RNG** in tallying.

## 2) Scope

* **In scope:**

  * Per-unit IRV from `(ranking_vec, count)` ballot groups.
  * Canonical option order & stable data structures.
  * Exhaustion policy **reduce\_continuing\_denominator** (VM-VAR-006).
  * Round audit.
* **Out of scope:** Allocation/WTA, gates math, frontier, I/O/schema parsing.

## 3) Inputs → Outputs

* **Inputs:**

  * `unit_id : UnitId`
  * `ballots : &[(Vec<OptionId>, u64)]` (validated groups)
  * `options : &[OptionItem]` (defines canonical `(order_index, id)` order)
  * `turnout : Turnout` (`ballots_cast`, `invalid_or_blank`, `valid_ballots`)
  * `params : &Params` (reads VM-VAR-001=ranked\_irv, VM-VAR-006)
* **Outputs:**

  * `(UnitScores, IrvLog)`

    * `UnitScores.scores` = **final-round tallies per option** (canonical order; eliminated options end at 0 unless they remain in the final round).
    * `IrvLog` = ordered round records with eliminated id, transfers map, exhausted count; plus winner id.

## 4) Entities/Tables (minimal)

* Core types: `UnitId`, `OptionId`, `Turnout`, `OptionItem`, `Params`, `UnitScores`.
* IRV audit types (data-only):

  * `IrvRound { eliminated: OptionId, transfers: BTreeMap<OptionId,u64>, exhausted: u64 }`
  * `IrvLog { rounds: Vec<IrvRound>, winner: OptionId }`
* Error type used elsewhere if needed: `TabError` (not expected in normal, pre-validated datasets).

## 5) Variables (used here)

* `V = turnout.valid_ballots` (initial **continuing\_total**).
* `exhaustion_policy = VM-VAR-006` (**reduce\_continuing\_denominator** fixed).
* Ballot groups are trusted unique with counts ≥ 0 (validation stage).

## 6) Functions (signatures only; **no code here**)

* `tabulate_ranked_irv(unit_id, ballots, options, turnout, params) -> (UnitScores, IrvLog)`
* Internal helpers (pure, deterministic):

  * `first_preferences(ballots, continuing_set) -> BTreeMap<OptionId,u64>`
  * `pick_lowest(tallies, continuing_order) -> OptionId` (ties broken by `(order_index, id)`)
  * `transfer_from_eliminated(ballots, eliminated, continuing_set) -> (BTreeMap<OptionId,u64>, exhausted: u64)`
  * `apply_exhaustion_policy(continuing_total, exhausted, policy) -> u64`
  * `finalize_scores(last_round_tallies, options) -> BTreeMap<OptionId,u64>`

## 7) Algorithm Outline (implementation plan)

1. **Initialize**

   * Build **continuing** set from `options` in canonical `(order_index, id)` order.
   * Tally **first preferences** over continuing; set `continuing_total = V`.
2. **Loop (rounds):**
   a) **Majority check:** if some `x` has `tally[x] > continuing_total / 2`, declare **winner = x**; stop.
   b) **Single remaining:** if `continuing.len() == 1`, that option is **winner**; stop.
   c) **Find lowest:** select the **lowest tally**; break ties **deterministically** by `(order_index, id)`.
   d) **Transfer:** for each ballot group currently at the **eliminated** option, scan forward to the next **continuing** preference; if none, **exhaust**.
   e) **Exhaustion policy:** with **reduce\_continuing\_denominator**, subtract this round’s exhausted count from `continuing_total`.
   f) **Update tallies & continuing set;** log `IrvRound { eliminated, transfers, exhausted }`.
   g) **Repeat**.
3. **Assemble outputs**

   * `UnitScores.scores` = **final-round tallies** for all `options` in canonical order (winners/continuers keep their final counts; earlier-eliminated → `0`).
   * `IrvLog` with `rounds` and `winner`.

## 8) State Flow

* Pipeline: **TABULATE (IRV)** → result feeds executive contexts or summary; still passes through **ALLOCATE** stage in pipeline order, but IRV itself furnishes the single-winner outcome for magnitude=1 scenarios.

## 9) Determinism & Numeric Rules

* **Stable ordering** everywhere (BTree\*, canonical option order).
* **Integer math only**; majority test uses exact integer division semantics (no floats).
* **No RNG**: elimination ties resolved by `(order_index, id)`.

## 10) Edge Cases & Failure Policy

* **V = 0** (no valid ballots): produce deterministic **winner = smallest `(order_index, id)`**; zero rounds in log; all scores `0` except (optionally) winner `0` as well—final tallies all `0`.
* **All equal at zero:** eliminate deterministically until one remains (logs reflect eliminations with zero transfers).
* **Unknown IDs / duplicates in a ranking:** assumed **pre-validated**; when scanning next preference, **skip** unknown/eliminated/repeats.
* **Fully truncated ballots:** they exhaust on first use; denominator shrinks accordingly.

## 11) Test Checklist (must pass)

* **IRV exhaustion case (Annex B / VM-TST-010):**

  * Round 1: `A=35, B=40, C=25` → eliminate `C`; transfer `15` to `B`, **exhaust** `10`; continuing becomes `90`; final `B=55, A=35`; **winner B**.
  * `IrvLog` records eliminated=`C`, transfers, exhausted=`10`.
* **Deterministic elimination tie:** reorder options or groups of equal tallies → same **winner/log** via canonical order.
* **Exhaustion policy honored:** `continuing_total` shrinks exactly by the exhausted count each round.
* **Zero-ballot unit:** logs zero rounds, deterministic fallback winner, all final tallies `0`.

---

### Notes & Alignments

* **Exhaustion policy** is fixed to **reduce\_continuing\_denominator (VM-VAR-006)**, as in your refs.
* **Tie handling** is deterministic within IRV tabulation (no policy/seed here); RNG is reserved for allocation ties elsewhere.
* `UnitScores` carries **final-round** tallies and `Turnout` unmodified; percentages (if needed) are computed downstream.
