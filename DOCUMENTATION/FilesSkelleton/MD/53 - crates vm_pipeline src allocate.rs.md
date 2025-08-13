Here’s the clean, no-code skeleton for **File 53 — `crates/vm_pipeline/src/allocate.rs`**, aligned with refs **42–45 (allocation)** and the pipeline flow.

# Pre-Coding Essentials — 53/89

**Component:** `crates/vm_pipeline/src/allocate.rs`
**Version/FormulaID:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** For each Unit, take `UnitScores` and the Unit’s magnitude, then allocate seats/power using the configured method (WTA, D’Hondt, Sainte-Laguë, Largest Remainder). Collect `UnitAllocation` for downstream aggregation.
* **Success:** Integer/rational math only; PR threshold honored; deterministic ordering; ties handled per policy (status\_quo / deterministic / seeded random). Totals equal Unit.magnitude (or 100% power for WTA).

## 2) Scope

* **In scope:** Per-Unit dispatch to the correct allocation method; PR entry threshold filtering; stable tie handling; audit hooks for last-seat ties.
* **Out of scope:** Tabulation, gates/frontier, MMP correction (handled in `mmp` module), report rendering, I/O.

## 3) Inputs → Outputs

**Inputs**

* `unit_scores: BTreeMap<UnitId, UnitScores>` (from TABULATE)
* `units: BTreeMap<UnitId, UnitMeta>` (provides magnitude, metadata)
* `options: Vec<OptionItem>` per Unit (canonical `(order_index, id)`)
* `Params` snapshot (VM-VARs)

**Outputs**

* `BTreeMap<UnitId, UnitAllocation>` (seats or 100% power)
* Optional `TieContext` items for last-seat/WTA-winner ties (for RESOLVE\_TIES)

## 4) Entities (minimal)

* `UnitId`, `OptionId`, `UnitMeta`, `OptionItem`
* `UnitScores { scores: BTreeMap<OptionId,u64>, turnout }`
* `UnitAllocation { seats_or_power: BTreeMap<OptionId,u32|u8>, last_seat_tie: bool, notes: … }`
* `TiePolicy`, `TieRng` (seeded), `AllocError`

## 5) Variables (used here)

* **VM-VAR-010** `allocation_method ∈ {winner_take_all, proportional_favor_big, proportional_favor_small, largest_remainder, mixed_local_correction}`
* **VM-VAR-011** `use_unit_magnitudes` (v1: on)
* **VM-VAR-012** `pr_entry_threshold_pct ∈ % 0..10`
* **VM-VAR-032** `tie_policy ∈ {status_quo, deterministic, random}`
* **VM-VAR-033** `tie_seed ∈ integer ≥ 0` (used only if `tie_policy = random`)
* (LR quota selection lives inside vm\_algo LR module)

## 6) Functions (signatures only; no code)

```rust
use std::collections::BTreeMap;
use vm_core::{
  ids::{UnitId, OptionId},
  entities::{UnitMeta, OptionItem},
  variables::{Params, TiePolicy},
  rng::TieRng,
};
use crate::tabulate::UnitScores;

// Public entry
pub fn allocate_all(
  unit_scores: &BTreeMap<UnitId, UnitScores>,
  units: &BTreeMap<UnitId, UnitMeta>,
  options_by_unit: &BTreeMap<UnitId, Vec<OptionItem>>,
  params: &Params,
) -> (BTreeMap<UnitId, UnitAllocation>, Vec<TieContext>);

// Internal helpers (orchestration)
fn allocate_one_unit(
  unit_id: UnitId,
  scores: &UnitScores,
  meta: &UnitMeta,
  options: &[OptionItem],
  p: &Params,
  rng: Option<&mut TieRng>,
) -> (UnitAllocation, Option<TieContext>);

fn apply_pr_threshold(
  scores: &BTreeMap<OptionId, u64>,
  valid_ballots: u64,
  threshold_pct: u8,
  options: &[OptionItem],
) -> BTreeMap<OptionId, u64>;

fn ensure_wta_magnitude(meta: &UnitMeta) -> Result<(), AllocError>;

// Tie utility (policy routing)
fn break_tie(
  context: &'static str,
  contenders: &[OptionId],
  options: &[OptionItem],
  tie_policy: TiePolicy,
  rng: Option<&mut TieRng>,
) -> OptionId;

// Thin wrappers to vm_algo methods (selection only; math lives in vm_algo)
fn run_wta(
  scores: &BTreeMap<OptionId, u64>,
  m: u32,
  options: &[OptionItem],
  tie: TiePolicy,
  rng: Option<&mut TieRng>,
) -> Result<UnitAllocation, AllocError>;

fn run_dhondt(
  seats: u32,
  scores: &BTreeMap<OptionId, u64>,
  options: &[OptionItem],
  threshold_pct: u8,
  tie: TiePolicy,
  rng: Option<&mut TieRng>,
) -> Result<UnitAllocation, AllocError>;

fn run_sainte_lague(
  seats: u32,
  scores: &BTreeMap<OptionId, u64>,
  options: &[OptionItem],
  threshold_pct: u8,
  tie: TiePolicy,
  rng: Option<&mut TieRng>,
) -> Result<UnitAllocation, AllocError>;

fn run_largest_remainder(
  seats: u32,
  scores: &BTreeMap<OptionId, u64>,
  options: &[OptionItem],
  threshold_pct: u8,
  quota: LrQuotaKind,   // Hare, Droop, Imperiali
  tie: TiePolicy,
  rng: Option<&mut TieRng>,
) -> Result<UnitAllocation, AllocError>;
```

## 7) Algorithm Outline (implementation plan)

* **Unit loop:** Iterate Units in stable `UnitId` order.
* **Select method (VM-VAR-010):**

  * **WTA:** Assert `magnitude == 1`; pick max score; resolve ties per policy; return `{winner: 100%}`.
  * **Proportional (favor\_big = D’Hondt / favor\_small = Sainte-Laguë):**

    1. Apply **PR threshold** to `scores` using the Unit’s natural total (sum of option scores for the ballot family).
    2. Run the respective vm\_algo allocator; pass `options` for deterministic tie order.
  * **Largest Remainder:** Same threshold step; compute via vm\_algo LR with configured quota.
  * **Mixed Local Correction (MMP):** Not allocated here—delegate to `mmp` stage later (locals are taken from earlier SMD allocations; top-ups happen in MMP module).
* **Tie handling:**

  * `status_quo` → prefer SQ among contenders; else fall back to deterministic.
  * `deterministic` → `(order_index, OptionId)` min.
  * `random` → use seeded `TieRng` (VM-VAR-033) and record a `TieContext` if this is a **blocking** tie (e.g., last seat, WTA winner).
* **Assemble:** Produce `UnitAllocation` with totals equal to Unit magnitude (or 100% power). Collect any `TieContext`.

## 8) State Flow

`TABULATE → ALLOCATE (this) → AGGREGATE → APPLY_DECISION_RULES → MAP_FRONTIER → RESOLVE_TIES → LABEL → BUILD_RESULT`.

## 9) Determinism & Numeric Rules

* Canonical data structures (`BTreeMap`); options iterated by `(order_index, id)`.
* Integer/rational comparisons; quotient comparisons via cross-multiplication (inside vm\_algo).
* RNG only when `tie_policy = random`, seeded by **VM-VAR-033**; identical inputs + seed ⇒ identical outcome.

## 10) Edge Cases & Failure Policy

* **WTA with m≠1** ⇒ `AllocError::InvalidMagnitude`.
* **Threshold excludes all** ⇒ empty allocation (all zeros); downstream label/report handle.
* **All scores zero** ⇒ allocate entirely via tie policy (deterministic order unless random).
* **Overflow guards** handled inside vm\_algo (use `u128` for products).
* **Unknown options** should be prevented upstream; if encountered, surface a typed error.

## 11) Test Checklist (must pass)

* **VM-TST-001:** Sainte-Laguë with m=10, approvals {10,20,30,40} ⇒ seats **1/2/3/4**.
* **VM-TST-002:** WTA with m=1, plurality {10,20,30,40} ⇒ **D gets 100%**.
* **VM-TST-003:** Convergence A/B/C shares 34/33/33, m=7 ⇒ **3/2/2** for LR, Sainte-Laguë, D’Hondt.
* **Threshold behavior:** Raising **VM-VAR-012** drops sub-threshold options from consideration.
* **Determinism:** Shuffled input order yields identical allocations due to canonical ordering.
* **Tie policy:** Deterministic vs random (seeded) produce expected, reproducible winners; last-seat ties record `TieContext`.
