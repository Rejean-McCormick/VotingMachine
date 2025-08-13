Here’s a **reference-aligned skeleton sheet** for **36 – crates/vm\_algo/src/lib.rs.md**, tightened to your ten refs and all prior fixes (array-ordered options by `order_index`, integers only, half-even where allowed, RNG only via injected `TieRng`, approval-gate denominator = valid ballots).

````
Pre-Coding Essentials (Component: crates/vm_algo/src/lib.rs, Version FormulaID VM-ENGINE v0) — 36/89

1) Goal & Success
Goal: Public surface for pure algorithm primitives (tabulation, allocation, gates, frontier support, light MMP). No I/O.
Success: Deterministic results across OS/arch; integer/rational math only; stable ordering (options by (order_index, OptionId)); RNG used only when TiePolicy=Random via injected TieRng.

2) Scope
In scope: pub mod declarations + re-exports; function signatures for:
- Tabulation (plurality, approval, score, IRV, Condorcet)
- Allocation (WTA, D’Hondt, Sainte-Laguë, Largest Remainder)
- MMP helpers
- Gates (quorum/majority/double-majority composition)
- Frontier support helper (ratio only; no topology)
Out of scope: schema/JSON, file/FS, report formatting, pipeline orchestration.

3) Inputs → Outputs (types)
Inputs: vm_core entities (UnitId, OptionId, OptionItem, Turnout), counts from vm_io loader, Params for switches, optional TieRng for ties.
Outputs: plain structs/maps (integers/ratios) + optional audit logs (IRV rounds, Condorcet pairwise), consumed by vm_pipeline.

4) Entities/Types (minimal)
- Uses vm_core::{ids, entities, variables, rounding::Ratio, rng::TieRng, determinism helpers}.

5) Variables (only those used at algo level)
- TiePolicy (VM-VAR-050) governs tie handling; TieRng seeded via VM-VAR-052 outside this crate.
- AllocationMethod / OverhangPolicy / TotalSeatsModel as needed for PR/MMP.

6) Functions (signatures only; no I/O)

```rust
use std::collections::BTreeMap;
use vm_core::{
    ids::{OptionId, UnitId},
    entities::{OptionItem, Turnout},
    variables::{Params, TiePolicy, AllocationMethod, OverhangPolicy, TotalSeatsModel},
    rounding::Ratio,
    rng::TieRng,
};

// ---------- Common structs returned by algorithms ----------

/// Raw scores per unit, ready for allocation/aggregation.
pub struct UnitScores {
    pub unit_id: UnitId,
    pub turnout: Turnout,                      // ballots_cast, invalid_ballots, valid_ballots
    pub scores: BTreeMap<OptionId, u64>,       // plurality=votes; approval=approvals; score=score_sums
}

/// Per-unit allocation result; deterministic ordering by (order_index, OptionId).
pub struct Allocation {
    pub unit_id: UnitId,
    pub seats_or_power: BTreeMap<OptionId, u32>, // WTA encoded as winner=100 (power), else seats
    pub last_seat_tie: bool,                     // true iff a tie policy decided a last seat
}

// ---------- Tabulation (deterministic) ----------

pub fn tabulate_plurality(
    unit_id: UnitId,
    votes: &BTreeMap<OptionId, u64>,
    turnout: Turnout,
) -> UnitScores;

pub fn tabulate_approval(
    unit_id: UnitId,
    approvals: &BTreeMap<OptionId, u64>,
    turnout: Turnout,
) -> UnitScores;

pub fn tabulate_score(
    unit_id: UnitId,
    score_sums: &BTreeMap<OptionId, u64>,
    turnout: Turnout,
    params: &Params,            // scale_min/scale_max domain already validated upstream
) -> UnitScores;

// Ranked IRV (compressed ballots: ranking with count), fixed exhaustion policy per spec
pub struct IrvRound {
    pub eliminated: OptionId,
    pub transfers: BTreeMap<OptionId, u64>,
    pub exhausted: u64,
}
pub struct IrvLog {
    pub rounds: Vec<IrvRound>,
    pub winner: OptionId,
}
pub fn tabulate_ranked_irv(
    ballots: &[(Vec<OptionId>, u64)],   // each: ranking (unique OPTs), multiplicity
    options: &[OptionItem],             // ordered by (order_index, id)
    params: &Params,
) -> (UnitScores, IrvLog);

// Condorcet (pairwise tallies + completion)
pub struct Pairwise {
    pub wins: BTreeMap<(OptionId, OptionId), u64>,  // (A,B) = votes preferring A over B
}
pub fn tabulate_ranked_condorcet(
    ballots: &[(Vec<OptionId>, u64)],
    options: &[OptionItem],
    params: &Params,                  // completion rule per Params
) -> (UnitScores, Pairwise);

// ---------- Allocation inside a Unit ----------

/// Winner-take-all. Assumes magnitude == 1 (validated by pipeline). TiePolicy applied on top scores.
pub fn allocate_wta(
    scores: &UnitScores,
    magnitude: u32,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,         // only used when tie_policy == Random
) -> Allocation;

/// Divisor methods (pure integer arithmetic; stable option order).
pub fn allocate_dhondt(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> BTreeMap<OptionId, u32>;

pub fn allocate_sainte_lague(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> BTreeMap<OptionId, u32>;

/// Largest Remainder with threshold (exclude strictly below threshold_pct of valid ballots before quota).
pub fn allocate_largest_remainder(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    threshold_pct: u8,
    options: &[OptionItem],
) -> BTreeMap<OptionId, u32>;

// ---------- MMP helpers (top-ups after local seats) ----------

pub fn mmp_target_shares(
    total_seats: u32,
    vote_totals: &BTreeMap<OptionId, u64>,
    method: AllocationMethod,          // e.g., Sainte-Laguë baseline
) -> BTreeMap<OptionId, u32>;

pub fn mmp_topups(
    local_seats: &BTreeMap<OptionId, u32>,
    targets: &BTreeMap<OptionId, u32>,
    overhang_policy: OverhangPolicy,
    total_seats_model: TotalSeatsModel,
) -> BTreeMap<OptionId, u32>;

// ---------- Gates (ratios; integers only) ----------

pub struct GateOutcome { pub pass: bool, pub observed: Ratio, pub threshold_pct: u8 }

/// Quorum: observed = valid_ballots / eligible_roll; compare to threshold with rational compare.
pub fn gate_quorum(valid_ballots: u64, eligible_roll: u64, threshold_pct: u8) -> GateOutcome;

/// Majority: observed = approvals_for_change / valid_ballots; half-even only where spec allows.
pub fn gate_majority(valid_ballots: u64, approvals_for_change: u64, threshold_pct: u8) -> GateOutcome;

/// Double-majority composition (national + regional).
pub struct DoubleMajority {
    pub national: GateOutcome,
    pub regional: GateOutcome,
    pub pass: bool,
}
pub fn gate_double_majority(national: GateOutcome, regional: GateOutcome) -> DoubleMajority;

// ---------- Frontier support helper (no topology) ----------
pub fn frontier_support_ratio(approvals_for_change: u64, valid_ballots: u64) -> Ratio;
````

7. Algorithm Outline (module layout)

* `pub mod tabulation;`  → plurality/approval/score/IRV/Condorcet; export logs (IrvLog, Pairwise).
* `pub mod allocation;`  → WTA, divisor methods, largest remainder; deterministic option order.
* `pub mod mmp;`         → seat targets & top-ups; integer math only.
* `pub mod gates_frontier;` → quorum/majority/double-majority; frontier support ratio helper.
* Re-export: `UnitScores`, `Allocation`, `IrvLog`, `Pairwise`, `GateOutcome`, `DoubleMajority`.

8. State Flow
   Pipeline:

9. Tabulate per unit → `UnitScores`.

10. Allocate seats/power per unit → `Allocation`.

11. Aggregate & compute gates → `GateOutcome`/`DoubleMajority`.

12. Frontier map uses `frontier_support_ratio` (approval rate) + contiguity elsewhere.

13. Determinism & Numeric Rules

* Stable orders: options by `(order_index, OptionId)`; never rely on map iteration.
* Use vm\_core::rounding for rational compare & half-even at permitted points; no floats.
* RNG only when `TiePolicy::Random` and `TieRng` is provided; otherwise deterministic (StatusQuo/DeterministicOrder).

10. Edge Cases & Failure Policy

* `valid_ballots == 0` ⇒ gates observed = 0/1, pass=false.
* WTA expects `magnitude == 1` (validated upstream); function applies tie policy among top scores.
* PR threshold: options strictly below `threshold_pct` of valid ballots are excluded before quotas/divisors.
* Ranked inputs must have unique rankings per ballot (loader/schema ensure); IRV exhaustion per fixed policy; Condorcet completion per `Params`.

11. Test Checklist (must pass; align with Annex-B canonical cases)

* Sainte-Laguë seats (m=10; 10/20/30/40) → 1/2/3/4.
* WTA (m=1; plurality 10/20/30/40) → top option wins 100% power.
* LR vs D’Hondt vs Sainte-Laguë convergence toy case → identical vector on symmetric totals.
* IRV toy example: rounds/elimination log matches fixture; winner consistent.
* Condorcet simple cycle resolved per selected completion rule in `Params`.
* Gates: quorum/majority outcomes match integer rational comparisons; approval gate denominator = valid ballots (not ballots\_cast).
* Determinism: permuting input order yields identical outputs; Random ties reproduce with same seed.

```

If you want, I can also draft the **module stubs** (`tabulation.rs`, `allocation.rs`, `mmp.rs`, `gates_frontier.rs`) with the exact function signatures and doc comments from this sheet.
```
