
# Pre-Coding Essentials — 52/89

**Component:** `crates/vm_pipeline/src/tabulate.rs`
**Version/FormulaID:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Implement the **TABULATE** stage: compute per-Unit `UnitScores` according to **VM-VAR-001** (plurality, approval, score, ranked\_irv, ranked\_condorcet) and collect audit artifacts (IRV rounds, Condorcet pairwise/logs).
* **Success:** Deterministic `UnitScores` for every Unit, integer-only math, correct turnout propagation, canonical option order, and audit sidecars; no RNG here.

## 2) Scope

* **In scope:** Per-Unit dispatch to vm\_algo tabulators; construction of outputs; capture of IRV/Condorcet audit data; collection of any **pending tie contexts** that must be resolved later.
* **Out of scope:** Allocation/thresholds/aggregation, gates/frontier, I/O/schema parsing, report rendering.

## 3) Inputs → Outputs

**Inputs**

* `LoadedContext` (Units, Options with `order_index`, BallotTally, ParameterSet snapshot).
* `Params` (reads **VM-VAR-001**, plus score/ ranked knobs).

**Outputs**

* `BTreeMap<UnitId, UnitScores>` (natural tallies + turnout).
* `TabulateAudit` (IRV round logs, Condorcet pairwise/logs, pending tie contexts for later **RESOLVE\_TIES**).

## 4) Entities/Tables (minimal)

* `UnitId`, `OptionId`, `Turnout`, `OptionItem`
* `UnitScores` (type imported from tabulation layer)
* `IrvLog`, `Pairwise`, `CondorcetLog`, `TieContext`

## 5) Variables (used here)

* **VM-VAR-001** `ballot_type ∈ {plurality, approval, score, ranked_irv, ranked_condorcet}`
* **VM-VAR-002/003/004** score scale & normalization (forwarded to score tabulator)
* **VM-VAR-005** condorcet completion rule
* **VM-VAR-006** IRV exhaustion policy (reduce continuing denominator)
* **VM-VAR-007** include\_blank\_in\_denominator (affects *gates later*, not tabulation)

## 6) Functions (signatures only; no code)

```rust
use std::collections::BTreeMap;
use vm_core::{
  ids::{UnitId, OptionId},
  entities::{Turnout, OptionItem},
  variables::Params,
};
use vm_algo::tabulation::{UnitScores};
use vm_algo::tabulation::ranked_irv::IrvLog;
use vm_algo::tabulation::ranked_condorcet::{Pairwise, CondorcetLog};
use crate::ties::TieContext; // downstream stage will consume this

/// Audit sidecar for TABULATE.
pub struct TabulateAudit {
  pub irv_logs: BTreeMap<UnitId, IrvLog>,
  pub condorcet_pairwise: BTreeMap<UnitId, Pairwise>,
  pub condorcet_logs: BTreeMap<UnitId, CondorcetLog>,
  pub pending_ties: Vec<TieContext>,
}

/// High-level: tabulate all units according to VM-VAR-001.
pub fn tabulate_all(
  ctx: &LoadedContext,
  p: &Params
) -> (BTreeMap<UnitId, UnitScores>, TabulateAudit);

// Per-type unit dispatchers (light wrappers around vm_algo).
fn tabulate_unit_plurality(unit: &UnitInput) -> UnitScores;
fn tabulate_unit_approval(unit: &UnitInput) -> UnitScores;
fn tabulate_unit_score(unit: &UnitInput, p: &Params) -> UnitScores;
fn tabulate_unit_ranked_irv(
  unit: &UnitInput,
  p: &Params
) -> (UnitScores, Option<IrvLog>, Option<TieContext>);
fn tabulate_unit_ranked_condorcet(
  unit: &UnitInput,
  p: &Params
) -> (UnitScores, Option<Pairwise>, Option<CondorcetLog>);
```

## 7) Algorithm Outline (implementation plan)

* **Canonical selection:** For each Unit (stable order by `UnitId`), dispatch based on **VM-VAR-001**.
* **Plurality:** call vm\_algo `tabulate_plurality` → return integer votes per option + turnout.
* **Approval:** call vm\_algo `tabulate_approval` → per-option approvals + turnout. *(Approval rate for gates computed later; not here.)*
* **Score:** call vm\_algo `tabulate_score` with scale/normalization from **VM-VAR-002..004** → per-option score sums + turnout. Enforce caps in algo; no ratios here.
* **Ranked IRV:** call vm\_algo `tabulate_ranked_irv` → collect `UnitScores`, `IrvLog`. If an elimination tie blocks progress, push `TieContext` to `pending_ties`.
* **Ranked Condorcet:** call vm\_algo `tabulate_ranked_condorcet` → collect `UnitScores`, `Pairwise`, `CondorcetLog` (winner determined by completion rule).
* **Aggregation of outputs:** Build `unit_scores` map and `TabulateAudit` maps; do **not** compute shares/percentages.

## 8) State Flow

`LOAD → VALIDATE → TABULATE (this) → ALLOCATE → AGGREGATE → APPLY_DECISION_RULES → …`
Audit payloads from this stage feed reporting and diagnostics; `pending_ties` feeds **RESOLVE\_TIES** if needed.

## 9) Determinism & Numeric Rules

* Stable orders: Units by `UnitId`; Options by `(order_index, OptionId)`.
* Integer math only; **no RNG** in this stage.
* Turnout is carried verbatim; blanks/invalid are excluded from valid tallies (gates may include blanks per **VM-VAR-007**, later).

## 10) Edge Cases & Failure Policy

* Unknown `ballot_type` ⇒ typed error for pipeline; stop following Doc 5 rules.
* Zero `valid_ballots` ⇒ produce zero `scores`; turnout still carried.
* Any tally sanity issues should have been caught in **VALIDATE**; keep debug guards.
* IRV all-exhausted / all-zero ties: vm\_algo returns deterministic outcome or a `TieContext`; pipeline defers resolution.

## 11) Test Checklist (must pass)

* **PR baseline:** approval tallies feed Sainte-Laguë to 1–2–3–4 downstream (VM-TST-001).
* **WTA flow:** plurality tallies feed WTA winner (VM-TST-002).
* **Convergence:** score/approval inputs route cleanly; no shares computed here.
* **Ranked IRV:** RoundLog shows shrinking continuing denominator; pending ties captured when applicable.
* **Condorcet:** Pairwise matrix + completion log recorded; winner stable.
* **Determinism:** Shuffling unit/option input order yields identical `UnitScores` after canonicalization.
