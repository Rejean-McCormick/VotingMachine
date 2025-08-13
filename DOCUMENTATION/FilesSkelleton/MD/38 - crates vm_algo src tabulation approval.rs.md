Here’s a **reference-aligned skeleton sheet** for **38 – crates/vm\_algo/src/tabulation/approval.rs.md**, matching the 10 refs and prior fixes (Doc 1B names **valid\_ballots/invalid\_ballots**, canonical option order by `(order_index, OptionId)`, integers only, no RNG; approval gate later uses **valid ballots** as the denominator).

```
Pre-Coding Essentials (Component: crates/vm_algo/src/tabulation/approval.rs, Version FormulaID VM-ENGINE v0) — 38/89

1) Goal & Success
Goal: Deterministically compute UnitScores for approval ballots from per-option approval counts and turnout.
Success: Returns exact integer scores per OptionId; preserves canonical option order via provided options[]; carries turnout (valid_ballots, invalid_ballots). No floats, no RNG.

2) Scope
In scope: Per-unit approval tabulation; non-negativity; per-option cap ≤ valid_ballots; canonical ordering.
Out of scope: Allocation, gates/thresholds, aggregation, tie resolution, I/O/schema.

3) Inputs → Outputs
Inputs:
- unit_id: UnitId
- approvals: &BTreeMap<OptionId, u64> (per-option approval counts)
- turnout: Turnout (ballots_cast, invalid_ballots, valid_ballots)
- options: &[OptionItem] (enforce canonical (order_index, id) ordering)
Output:
- UnitScores { unit_id, turnout, scores: BTreeMap<OptionId, u64> } (iterate using options for canonical order)

4) Entities/Tables (minimal)
(Uses vm_core: UnitId, OptionId, OptionItem, Turnout; vm_algo: UnitScores.)

5) Variables (only ones used here)
None (all integers).

6) Functions (signatures only)
rust
CopyEdit
use std::collections::BTreeMap;
use vm_core::{
    ids::{UnitId, OptionId},
    entities::{Turnout, OptionItem},
};
use crate::UnitScores;

#[derive(Debug)]
pub enum TabError {
    UnknownOption(OptionId),
    OptionExceedsValid { option: OptionId, approvals: u64, valid_ballots: u64 },
}

/// Deterministic approval tabulation (integers only; no RNG).
pub fn tabulate_approval(
    unit_id: UnitId,
    approvals: &BTreeMap<OptionId, u64>,
    turnout: Turnout,
    options: &[OptionItem],
) -> Result<UnitScores, TabError>;

/// Internal: build canonical score map from provided approvals and option list.
/// Iterates options in (order_index, OptionId) order; missing keys → 0; rejects unknown approval keys.
fn canonicalize_scores(
    approvals: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> Result<(BTreeMap<OptionId, u64>, /*sum not needed; keep for symmetry?*/ ()), TabError>;

/// Internal sanity: per-option cap approvals_for_option ≤ valid_ballots.
fn check_per_option_caps(
    scores: &BTreeMap<OptionId, u64>,
    turnout: &Turnout,
) -> Result<(), TabError>;

7) Algorithm Outline (implementation plan)
Canonical order
- Iterate `options` in canonical order ((order_index, OptionId)).
- For each `opt.id`, read `approvals.get(&opt.id).copied().unwrap_or(0)` and insert into a fresh `BTreeMap<OptionId,u64>`.
- After building, scan input `approvals` for any OptionId not in `options` → `TabError::UnknownOption`.

Sanity checks
- Non-negativity guaranteed by `u64`.
- Per-option cap: for every `(opt, count)` ensure `count ≤ turnout.valid_ballots`; else `OptionExceedsValid`.
- Do **not** enforce Σ approvals ≤ valid_ballots (multiple approvals per ballot are allowed).

Assemble result
- Return `UnitScores { unit_id, turnout, scores }`.
- No shares/percentages here; approval rate for gates uses `approvals_for_change / valid_ballots` elsewhere.

8) State Flow
TABULATE (approval) → UnitScores → ALLOCATE (PR/WTA) → AGGREGATE → GATES (approval rate uses valid ballots).

9) Determinism & Numeric Rules
- Determinism via canonical option traversal and BTreeMap storage.
- Integers only; no rounding; no RNG.

10) Edge Cases & Failure Policy
- Missing option in input map ⇒ treated as 0.
- Unknown/extra option in input map ⇒ `TabError::UnknownOption`.
- `valid_ballots == 0` ⇒ all scores must be 0 (per-option caps enforce this).
- Any per-option approvals exceeding valid_ballots ⇒ `TabError::OptionExceedsValid`.

11) Test Checklist (must pass)
- Happy path: A/B/C/D approvals (10/20/30/40), turnout valid=100, invalid=0 → scores equal input; iteration order matches `options`.
- Per-option cap: valid_ballots=50; any option >50 ⇒ fail with OptionExceedsValid.
- Unknown key in approvals ⇒ fail with UnknownOption.
- valid_ballots=0 with non-zero approvals ⇒ fail (per-option caps).
- Determinism: shuffle approvals insertion order → identical UnitScores content and canonical bytes when serialized.
```
