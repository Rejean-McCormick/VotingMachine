<!-- Converted from: 37 - crates vm_algo src tabulation plurality.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.502336Z -->

```
Pre-Coding Essentials (Component: crates/vm_algo/src/tabulation/plurality.rs, Version/FormulaID: VM-ENGINE v0) — 37/89
1) Goal & Success
Goal: Deterministically compute UnitScores for plurality ballots from per-option vote counts and turnout.
Success: Returns exact integer scores per OptionId, preserves canonical option order, and carries turnout (ballots_cast, invalid_or_blank, valid_ballots). No floats, no RNG.
2) Scope
In scope: Per-unit plurality tabulation; validation of non-negative counts; optional invariants (sum of option votes ≤ valid_ballots).
Out of scope: Allocation, gates/thresholds, aggregation, tie resolution, I/O/schema.
3) Inputs → Outputs
Inputs:
unit_id: UnitId
votes: &BTreeMap<OptionId, u64> (raw counts per option)
turnout: Turnout (ballots_cast, invalid_or_blank, valid_ballots)
options: &[OptionItem] (to enforce canonical (order_index, id) ordering)
Output:
UnitScores { unit_id, turnout, scores: BTreeMap<OptionId, u64> } (scores sorted by canonical option order)
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
rust
CopyEdit
use std::collections::BTreeMap;
use vm_core::{
ids::{UnitId, OptionId},
entities::{Turnout, OptionItem},
};

/// Deterministic plurality tabulation.
pub fn tabulate_plurality(
unit_id: UnitId,
votes: &BTreeMap<OptionId, u64>,
turnout: Turnout,
options: &[OptionItem],
) -> UnitScores;

/// Internal: build canonical score map from provided votes and option list.
fn canonicalize_scores(
votes: &BTreeMap<OptionId, u64>,
options: &[OptionItem],
) -> BTreeMap<OptionId, u64>;

/// Internal checks (enabled in debug; return Result in release if preferred).
fn check_tally_sanity(
votes_sum: u64,
turnout: &Turnout,
) -> Result<(), TabError>;

7) Algorithm Outline (implementation plan)
Canonical order
Iterate options in (order_index, OptionId) order and pull votes.get(&opt.id).copied().unwrap_or(0).
Insert into a new BTreeMap<OptionId,u64> to have deterministic iteration for downstream.
Sanity checks
All counts are non-negative (u64 already).
valid_ballots = ballots_cast - invalid_or_blank (trust Turnout constructor).
sum(scores.values()) ≤ valid_ballots (not enforced for approval/score; required for plurality). If violated, return TabError::TallyExceedsValid.
Assemble result
Return UnitScores{ unit_id, turnout, scores }.
No normalization
Do not divide or compute shares here; seats/gates use integers or ratios later.
8) State Flow
Pipeline: TABULATE (this function) → ALLOCATE (WTA/PR) → AGGREGATE → GATES. UnitScores feed allocation and later gates/labels.
9) Determinism & Numeric Rules
Determinism via canonical option iteration and BTreeMap storage.
Integer math only; no rounding; no RNG.
10) Edge Cases & Failure Policy
Missing option in votes ⇒ treated as 0.
Extra option present in votes but not in options ⇒ ignore or error? Choose error (TabError::UnknownOption) to keep referential integrity; loader should prevent this earlier.
votes_sum > valid_ballots ⇒ error (TabError::TallyExceedsValid).
turnout.valid_ballots == 0 ⇒ still return zeros; downstream gates handle legitimacy.
11) Test Checklist (must pass)
Happy path: A/B/C/D votes (e.g., 10/20/30/40), turnout 100/0/100 → scores equal input; canonical order matches options.
Missing option key in votes yields 0, not panic.
Unknown option key in votes triggers TabError::UnknownOption.
Tally sanity: sum(votes) == valid_ballots passes; sum(votes) > valid_ballots fails.
Determinism: shuffle insertion order of votes → identical UnitScores.scores order and bytes when serialized canonically.
```
