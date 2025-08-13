Pre-Coding Essentials (Component: crates/vm_core/src/entities.rs, Version/FormulaID: VM-ENGINE v0) — 24/89

1) Goal & Success
Goal: Define domain types used across the engine (registry, units, options, shared blocks) with stable semantics and no I/O.
Success: Types compile on all targets; invariants are encoded (e.g., options non-empty; order_index bounds); deterministic sorting helpers are provided (units by UnitId; options by order_index then OptionId); optional serde derives are gated.

2) Scope
In scope: structs/enums for DivisionRegistry, Unit, OptionItem, TallyTotals, DecisivenessLabel; thin constructors/validators; deterministic sort helpers.
Out of scope: parameter variables (live in variables.rs), ID parsing (ids/tokens modules), serialization (vm_io), pipeline state, report rendering.

3) Inputs → Outputs
Inputs: none at runtime (library types only).
Outputs: strongly typed values for vm_io (codec), vm_algo (compute), vm_pipeline (orchestration), vm_report (read-only).

4) Entities/Types (inventory)
- `DivisionRegistry`
  - `schema_version: String`
  - `units: Vec<Unit>`  // ≥1; canonical order: ↑ unit_id
- `Unit`
  - `unit_id: UnitId`
  - `name: String`         // 1..=200 chars
  - `protected_area: bool`
  - `options: Vec<OptionItem>`  // ≥1; canonical order: ↑ (order_index, option_id)
- `OptionItem`
  - `option_id: OptionId`
  - `name: String`          // 1..=200 chars
  - `order_index: u16`      // ≥0 (fits spec bounds comfortably)
- `TallyTotals` (utility used by algo/pipeline; mirrors BallotTally per-unit totals)
  - `valid_ballots: u64`    // ≥0
  - `invalid_ballots: u64`  // ≥0
  - `fn ballots_cast(&self) -> u64 { valid + invalid }`
- `DecisivenessLabel`
  - enum { `Decisive`, `Marginal`, `Invalid` }  // used by Result labeling

Derives (where meaningful): `Clone`, `Debug`, `Eq`, `PartialEq`, `Hash`. Avoid blanket `Ord` derives that would bake in non-canonical field order.

5) Variables (only ones used here)
None (parameters are modeled in variables.rs). This file remains parameter-agnostic.

6) Functions (signatures only)
Constructors / validators
- `impl DivisionRegistry {
     pub fn new(schema_version: String, units: Vec<Unit>) -> Result<Self, EntityError>;
     pub fn units(&self) -> &[Unit];
     pub fn unit(&self, id: &UnitId) -> Option<&Unit>;
   }`

- `impl Unit {
     pub fn new(unit_id: UnitId, name: String, protected_area: bool, options: Vec<OptionItem>) -> Result<Self, EntityError>;
     pub fn is_root(&self) -> bool { /* root-ness is a pipeline concept; return false here or omit */ }
   }`

- `impl OptionItem {
     pub fn new(option_id: OptionId, name: String, order_index: u16) -> Result<Self, EntityError>;
   }`

Deterministic ordering helpers
- `pub fn sort_units_by_id(units: &mut [Unit]);`                     // ↑ unit_id
- `pub fn sort_options_canonical(opts: &mut [OptionItem]);`          // ↑ (order_index, option_id)
- `pub fn cmp_options(a: &OptionItem, b: &OptionItem) -> std::cmp::Ordering;`

TallyTotals utility
- `impl TallyTotals { pub fn new(valid_ballots: u64, invalid_ballots: u64) -> Self; pub fn ballots_cast(&self) -> u64; }`

7) Implementation plan (invariants & helpers)
- Enforce `units.len() ≥ 1`, each `Unit.options.len() ≥ 1`.
- Enforce `name` length 1..=200 for Unit/OptionItem.
- Enforce `order_index` within u16; comparator is `(order_index, option_id)`.
- Sorting functions are total and stable; rely only on token IDs and `order_index`.
- No adjacency/magnitude/baseline fields here (out of scope for Registry).
- Optional serde derives behind `#[cfg(feature = "serde")]`:
  - Prefer explicit field renames only if external JSON differs (vm_io handles wire naming; keep core neutral).

8) State Flow (very short)
vm_io constructs these from validated JSON; vm_algo consumes them for counts/allocation; vm_pipeline enforces cross-artifact checks and ordering; vm_report reads Result artifacts (not defined here).

9) Determinism & Numeric Rules
- Canonical orders: Units ↑ unit_id; Options ↑ (order_index, option_id).
- No floating-point here; counts are integers; presentation rounding happens elsewhere.

10) Edge Cases & Failure Policy
- `DivisionRegistry::new` fails on empty `units`.
- `Unit::new` fails on empty `options` or out-of-bounds names.
- `OptionItem::new` fails on invalid `name` length (or any future domain guard).
- Sorting helpers must produce identical order across OS/arch.

11) Test Checklist
- Construct minimal valid registry: one Unit, one OptionItem → OK.
- Reject empty `units` or empty `options` → `EntityError::EmptyCollection`.
- Name bounds: length 0 or >200 → `EntityError::InvalidName`.
- Option ordering: given scrambled `(order_index, option_id)`, sorting is stable & deterministic.
- `TallyTotals::ballots_cast()` equals sum; large values don’t overflow u64.

Notes for coding
- Keep this file domain-only (no path logic, no JSON). Document each invariant with a unit test.
- Keep public API minimal; prefer construction through `new(..)` to preserve invariants.
