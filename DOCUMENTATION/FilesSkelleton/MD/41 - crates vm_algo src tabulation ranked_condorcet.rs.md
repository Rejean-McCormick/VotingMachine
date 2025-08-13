Got it — here’s the clean, no-code skeleton for **File 41 — `crates/vm_algo/src/tabulation/ranked_condorcet.rs`**, aligned with your references.

# Pre-Coding Essentials — 41/89

**Component:** `crates/vm_algo/src/tabulation/ranked_condorcet.rs`
**Version/FormulaID:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Deterministically tabulate **Condorcet** per unit: build full pairwise matrix from ranked ballots; if a Condorcet winner exists, select it; otherwise resolve using the configured completion rule (**Schulze** or **Minimax**).
* **Success:** Integer-only counts, canonical option order, reproducible **Pairwise** audit matrix and **CondorcetLog**; no RNG (cycles resolved purely by the completion rule).

## 2) Scope

* **In scope:** Pairwise tallying; Condorcet winner detection; completion executor (Schulze/Minimax); deterministic tie-breaking inside the method; emit audit structures.
* **Out of scope:** Allocation, gates/frontier, I/O/schema parsing, reporting.

## 3) Inputs → Outputs

* **Inputs:**

  * `unit_id : UnitId`
  * `ballots : &[(Vec<OptionId>, u64)]` (validated ranked groups)
  * `options : &[OptionItem]` (canonical `(order_index, id)` order)
  * `turnout : Turnout`
  * `params : &Params` (reads VM-VAR-001=ranked\_condorcet, **VM-VAR-005** completion)
* **Outputs:**

  * `(UnitScores, Pairwise, CondorcetLog)`

    * `UnitScores.scores` represents the **winner outcome** (winner-only tally or rule-consistent final tallies; see §7).
    * `Pairwise` = audit matrix of wins.
    * `CondorcetLog` = rule used, winner id, pairwise summary.

## 4) Entities/Tables (minimal)

* Core: `UnitId`, `OptionId`, `Turnout`, `OptionItem`, `Params`, `UnitScores`.
* Pairwise audit:

  * `Pairwise { wins: BTreeMap<(OptionId, OptionId), u64> }`
* Log:

  * `CondorcetLog { completion_rule: CompletionRule, winner: OptionId, pairwise_summary: Pairwise }`
* Config enum:

  * `CompletionRule { Schulze, Minimax }`

## 5) Variables (used here)

* Canonical **option order** = `(order_index, id)`.
* `V = turnout.valid_ballots` (for optional winner-only score representation).
* `rule = params.condorcet_completion (VM-VAR-005)`.

## 6) Functions (signatures only; no code)

* **Entry:**

  * `tabulate_ranked_condorcet(unit_id, ballots, options, turnout, params) -> (UnitScores, Pairwise, CondorcetLog)`
* **Internals (pure, deterministic):**

  * `build_pairwise(ballots, options) -> Pairwise`
  * `condorcet_winner(pw, options) -> Option<OptionId>`
  * `schulze_winner(pw, options) -> OptionId`
  * `minimax_winner(pw, options) -> OptionId`
  * `winner_scores(winner, turnout, options, mode) -> BTreeMap<OptionId, u64>`
    *(Mode governs whether to emit winner-only = `{winner: V, others: 0}` or a rule-consistent final scores map; default = winner-only.)*

## 7) Algorithm Outline (implementation plan)

1. **Canonicalize inputs**

   * Work in option order `(order_index, id)`; all maps use BTree\* for stable iteration.
2. **Build pairwise matrix**

   * For each ballot group (ranking list, count), for every ordered pair `(A, B)` where **A is ranked above B**, add count to `wins[(A,B)]`.
   * If neither ranked (truncation) → **abstain** (no increment). Equal ranks are assumed absent or pre-resolved by loader.
3. **Detect Condorcet winner**

   * If some `X` has `wins[(X,Y)] > wins[(Y,X)]` for **all Y ≠ X**, pick `X`.
4. **No Condorcet winner → completion rule**

   * **Schulze:** compute strongest paths; select maximal per Schulze relation.
   * **Minimax:** choose option minimizing its **maximum pairwise defeat**.
   * Where internal ties arise, break deterministically by `(order_index, id)` (no RNG).
5. **Assemble outputs**

   * `winner = found_or_completed`.
   * `UnitScores.scores` via `winner_scores` policy (default: winner-only `{winner: V, others: 0}` in canonical key order).
   * Return `(UnitScores, Pairwise, CondorcetLog{rule, winner, pairwise_summary})`.

## 8) State Flow

* Pipeline: **TABULATE (Condorcet)** → **ALLOCATE/AGGREGATE** as usual. For executive/single-winner contexts, this determines the unit winner.

## 9) Determinism & Numeric Rules

* Integer counts only; **no floats**.
* Stable ordering via canonical option order and BTree\* structures.
* Completion rules are **algorithmic**; **no tie-policy RNG** is used here.

## 10) Edge Cases & Failure Policy

* **Zero valid ballots / empty rankings:** no meaningful pairwise counts; select deterministic fallback = smallest `(order_index, id)`; scores winner-only = `{winner: 0, others: 0}`.
* **Truncation/unknown IDs:** assumed pre-validated; when comparing, unranked pairs abstain.
* **All pairwise ties:** completion reduces to deterministic order fallback.
* Negative/overflow: impossible with `u64` increments; matrix size bounded by `|options|^2`.

## 11) Test Checklist (must pass)

* **Annex B Condorcet (Schulze)** profile → expected winner **B**; matrix matches pairwise counts.
* **Rule switch:** same profile with **Minimax** → winner per minimax mechanics; asserts may differ accordingly.
* **Determinism:** permute ballot order / option IDs → identical `Pairwise` and **winner**.
* **Degenerate:** all abstentions / ties → deterministic fallback; stable `UnitScores` map in canonical order.

---

If you want the winner-only vs. final-tally representation toggled explicitly, I’ll include the mode knob in the params notes for the next pass.
