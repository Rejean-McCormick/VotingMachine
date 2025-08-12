<!-- Converted from: 42 - crates vm_algo src allocation wta.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.614192Z -->

```
Pre-Coding Essentials (Component: crates/vm_algo/src/allocation/wta.rs, Version/FormulaID: VM-ENGINE v0) — 42/89
Goal & Success
Goal: Winner-take-all allocation per Unit: pick the option with the highest Unit score and allocate 100% power to it. Enforce the rule WTA ⇒ Unit.magnitude = 1.
Success: Deterministic winner for any input; m≠1 rejected; ties resolved per VM-VAR-032 tie_policy (status_quo / deterministic / random with VM-VAR-033 tie_seed).
Scope
In scope: Max-by-score selection, WTA coherence checks, tie breaking, return of Allocation { unit_id, seats_or_power } with 100% for the winner.
Out of scope: Tabulation, gates/labels, aggregation, schema/I/O.
Inputs → Outputs
Inputs:
scores: &UnitScores (from TABULATE; integers only)
magnitude: u32 (must be 1)
options: &[OptionItem] (canonical order, includes order_index and is_status_quo)
tie_policy: TiePolicy and optional rng: &mut TieRng if random
Output:
Allocation { unit_id, seats_or_power: { winner → 100 }, last_seat_tie: bool } (WTA uses power=100% as in tests).
Entities/Tables (minimal)
None.
Variables (used here)
VM-VAR-032 tie_policy ∈ {status_quo, deterministic, random} (default: status_quo)
VM-VAR-033 tie_seed ∈ integers (≥ 0) (default: 0) — used only when tie_policy = random
Functions (signatures only)
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
fn top_by_score(scores: &UnitScores) -> (u64, Vec<OptionId>); // max score and all tied at max
fn break_tie_wta(
tied: &[OptionId],
options: &[OptionItem],
tie_policy: TiePolicy,
rng: Option<&mut TieRng>,
) -> OptionId;
Algorithm Outline (implementation plan)
Preconditions
Require magnitude == 1; else AllocError::InvalidMagnitude. (Also validated earlier in VALIDATE.)
Find maximum
Scan scores.scores (integers) to get max and the list of tied options at max. Integer math only.
Tie handling
If tied.len() == 1 → winner = tied[0].
Else apply tie_policy (VM-VAR-032):
status_quo → pick the option with is_status_quo = true; if none or multiple, fall back to deterministic.
deterministic → pick the smallest (order_index, OptionId) among tied (uses Option.order_index).
random → draw uniformly using ChaCha20 seeded RNG constructed from tie_seed (VM-VAR-033); log via pipeline TieLog rules.
Assemble allocation
seats_or_power = { winner: 100 }, last_seat_tie = (tied.len() > 1).
State Flow
Pipeline order: TABULATE → ALLOCATE (this WTA) → AGGREGATE.
Ties (if any) are recorded per pipeline tie-logging rules; RNG only used when tie_policy = random.
Determinism & Numeric Rules
Stable option ordering by (order_index, id); exact integers; no floats.
Random tie breaks use only tie_seed (VM-VAR-033); same inputs + same seed ⇒ identical outcome/logs across OS/arch.
Edge Cases & Failure Policy
magnitude != 1 ⇒ AllocError::InvalidMagnitude.
All scores zero ⇒ still select per tie policy (status_quo → SQ; else deterministic/random).
Unknown options cannot appear (UnitScores comes from validated loader); if encountered, panic in debug / AllocError::UnknownOption in release.
Multiple is_status_quo = true is invalid upstream; if encountered, fall back to deterministic.
Test Checklist (must pass)
VM-TST-002 (WTA wipe-out): plurality A/B/C/D = 10/20/30/40, m=1 ⇒ D gets 100%.
Magnitude guard: set m=2 under WTA ⇒ fail with InvalidMagnitude.
Status-quo tie: top tie between Change and Status Quo with equal scores ⇒ winner = Status Quo under status_quo.
Deterministic tie: same tie with deterministic ⇒ pick lowest (order_index, id).
Random tie (seeded): same tie with random and fixed tie_seed ⇒ reproducible winner and TieLog.
```
