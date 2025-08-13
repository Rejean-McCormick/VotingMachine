````
Pre-Coding Essentials (Component: BUILD_RESULT, Version/FormulaID: VM-ENGINE v0) — 58/89

1) Goal & Success
Goal: Compose the **Result** artifact from prior pipeline outputs (allocations, aggregates, gates, label, optional frontier pointer), ready for canonical serialization and hashing.
Success: Fully aligns with the adjusted Result schema:
• Result **includes** `formula_id` (from the Normative Manifest / FID).
• Result **does not** carry input references (`reg_id`, `ballot_tally_id`, `parameter_set_id`) — those live in RunRecord.
• Shares/ratios in Result are JSON **numbers** at engine precision (not `{num,den}` objects).
• No tie log in Result (tie events live in RunRecord).

2) Scope
In scope: Assemble fields for Result; convert internal rational values to JSON numbers at engine precision; ensure canonical ordering of units/options; attach gates panel and label; optionally include `frontier_map_id`.
Out of scope: File I/O & hashing (vm_io handles canonical bytes + digests); RunRecord construction (next stage); any recomputation of aggregates/gates.

3) Inputs → Outputs (with schemas/IDs)
Inputs:
• `formula_id: String` (computed earlier from Normative Manifest / FID).
• `AggregateResults` (unit allocations + national/regional totals; integer tallies; exact ratios internally).
• `LegitimacyReport` (gate values & pass/fail; exact ratios internally).
• `DecisivenessLabel { label, reason }`.
• `Optionally FrontierId` (if a FrontierMap was produced).
Outputs:
• `Result` object:
  - `id: ResultId` (assigned after canonicalization/hash by caller).
  - `formula_id: String`  ✅
  - `label: "Decisive" | "Marginal" | "Invalid"`, `label_reason: String`
  - `units: [UnitBlock...]` (scores/turnout/allocation/flags; canonical order)
  - `aggregates: { ... }` (totals & **shares as numbers**; turnout; weighting method echo)
  - `gates: { quorum, majority, double_majority?, symmetry }` (**observed values as numbers**; thresholds as ints; pass booleans)
  - `frontier_map_id?: FrontierId`
• (No input IDs, no tie log here.)

4) Entities/Tables (minimal)
• `UnitBlock`:
  - `unit_id`
  - `turnout { ballots_cast, invalid_or_blank, valid_ballots }`
  - `scores { OPT → u64 }`
  - `allocation { OPT → u32 }` or `{ power_pct: u32 }` for WTA=100
  - `flags { unit_data_ok, unit_quorum_met, unit_pr_threshold_met, protected_override_used, mediation_flagged }`
• `Aggregates`:
  - `totals { OPT → u64 | u32 }`
  - `shares { OPT → number }`  ✅ (engine precision)
  - `turnout { ballots_cast, invalid_or_blank, valid_ballots, eligible_roll }`
  - `weighting_method: String` (echo of VM-VAR-030)
• `Gates`:
  - `quorum { observed:number, threshold_pct:int, pass:bool }`
  - `majority { observed:number, threshold_pct:int, pass:bool }`
  - `double_majority? { national:{observed:number,threshold_pct:int,pass:bool}, regional:{observed:number,threshold_pct:int,pass:bool}, pass:bool }`
  - `symmetry { pass:bool }`

5) Variables (only ones used here)
None computational; the engine has already produced aggregates/gates. A local constant `ENGINE_SHARE_PRECISION` (e.g., 1e-6 or better) governs number emission.

6) Functions (signatures only)
```rust
/// Compose a Result (without IDs/hashes). Caller will assign `id` after hashing.
pub fn build_result(
  formula_id: &str,
  agg: &AggregateResults,
  gates: &LegitimacyReport,
  label: &DecisivenessLabel,
  frontier_id: Option<FrontierId>,
) -> Result<ResultDoc, BuildError>;

fn write_unit_blocks(agg: &AggregateResults) -> Vec<UnitBlock>;
fn write_aggregates_as_numbers(agg: &AggregateResults) -> AggregatesOut; // shares as JSON numbers
fn write_gates_as_numbers(g: &LegitimacyReport) -> GatesOut;            // observed as numbers
````

7. Algorithm Outline (deterministic assembly)

8. Initialize ResultDoc:

   * `formula_id = formula_id.to_owned()`.
   * `label`, `label_reason` from `DecisivenessLabel`.

9. Units:

   * Build `UnitBlock` list from per-unit tabulation + allocation.
   * Sort **by `unit_id`**; inside maps, sort **Options by (order\_index, OptionId)**.

10. Aggregates:

    * Copy integer `totals` and `turnout`.
    * Convert internal exact ratios to JSON **numbers** using `ENGINE_SHARE_PRECISION`; do **not** round to presentation decimals.
    * Echo `weighting_method`.

11. Gates:

    * From `LegitimacyReport`, convert each `observed` ratio to a JSON number at engine precision; thresholds remain integers; set `pass` flags.

12. Frontier:

    * If present, set `frontier_map_id`.

13. Return `ResultDoc` (caller will canonicalize, hash, and assign `id`).

14. State Flow
    LABEL\_DECISIVENESS → **BUILD\_RESULT** → BUILD\_RUN\_RECORD (which embeds inputs/digests, engine vendor/name/version/build, NM digest, formula\_id, and **ties\[]**).

15. Determinism & Numeric Rules
    • Ordering: Units by `UnitId`; Options by `(order_index, OptionId)`; deterministic BTree-backed maps before serialization.
    • Integer math throughout; **only at this step do we emit numeric shares** as JSON numbers with fixed engine precision.
    • No RNG; tie resolution was completed earlier and is recorded in RunRecord only.

16. Edge Cases & Failure Policy
    • Missing per-unit data → set `unit_data_ok=false`; keep deterministic ordering; fill zeros where appropriate.
    • WTA units must emit `{ power_pct: 100 }` for the winner (never seats); PR units must sum seats to `magnitude`.
    • `frontier_map_id` omitted if frontier not produced (e.g., gates failed).
    • Never attach input IDs or tie logs here.

17. Test Checklist (must pass)
    • Result contains `formula_id`; **does not** contain `reg_id`/`ballot_tally_id`/`parameter_set_id`.
    • `shares` and gate `observed` are JSON numbers (stable to engine precision), not `{num,den}`.
    • Units/options are canonically ordered; WTA emits power=100.
    • No tie log present in Result; RunRecord carries `ties[]`, inputs’ 64-hex digests, engine vendor/name/version/build, and NM digest.
    • Canonical serialization → identical bytes/hash across OS given identical inputs.

```
```
