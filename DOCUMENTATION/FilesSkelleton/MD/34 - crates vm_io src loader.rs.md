
````
Pre-Coding Essentials (Component: crates/vm_io/src/loader.rs, Version FormulaID VM-ENGINE v0) — 34/89

1) Goal & Success
Goal: Load local JSON artifacts (manifest → registry → params → ballot_tally, optional adjacency), validate them against schemas, normalize ordering, and return a typed LoadedContext for the pipeline.
Success: Given a valid manifest, returns a fully-typed context with: units/options in canonical order, tally in Doc-1B shape (per-unit totals + options[] array), and precise early cross-refs (unit/option IDs exist; tally.reg_id matches registry.id).

2) Scope
In scope: file read, JSON parse, JSON-Schema validation, typed decode (vm_core types + vm_io tally types), canonical ordering, light referential checks, input digests.
Out of scope: heavy semantics (tree/cycles/gates/threshold math), allocation/tabulation, report writing.

3) Inputs → Outputs
Inputs: `manifest::ResolvedPaths` (reg, params, ballot_tally, optional adjacency).
Outputs: `LoadedContext` { registry, params, tally, adjacency_inline?, digests }, ready for vm_pipeline.

4) Entities/Types (minimal)
- Uses vm_core: `DivisionRegistry`, `Unit`, `OptionItem`, `Adjacency`, `Params`.
- vm_io (this crate) provides tally wire types matching **ballot_tally.schema.json** (per-unit, **options as an array** ordered by registry `order_index`).

5) Loader knobs
- `io.max_bytes`, `io.max_depth` (DoS guards).
- `cross_refs.strict` (default true): fail fast on unknown Unit/Option IDs in tally/adjacency.

6) Functions (signatures only)
```rust
use crate::IoError;
use vm_core::{entities::*, variables::Params};

/// Tally wire model (vm_io-local typed view, array-based options)
#[derive(Debug, Clone)]
pub struct UnitTotals {
    pub unit_id: UnitId,            // U:…
    pub totals: Totals,             // { valid_ballots, invalid_ballots }
    pub options: Vec<OptionCount>,  // ordered by registry order_index
}
#[derive(Debug, Clone)]
pub struct Totals { pub valid_ballots: u64, pub invalid_ballots: u64 }
#[derive(Debug, Clone)]
pub struct OptionCount { pub option_id: OptionId, pub count: u64 }
#[derive(Debug, Clone)]
pub struct UnitTallies { pub ballot_type: BallotType, pub units: Vec<UnitTotals> }

#[derive(Debug, Clone, Copy)]
pub enum BallotType { Plurality, Approval, Score, RankedIrv, RankedCondorcet }

#[derive(Debug)]
pub struct InputDigests {
    pub division_registry_sha256: String,
    pub ballot_tally_sha256:      String,
    pub parameter_set_sha256:     String,
    pub adjacency_sha256:         Option<String>,
}

#[derive(Debug)]
pub struct LoadedContext {
    pub registry: DivisionRegistry,
    pub params:   Params,
    pub tally:    UnitTallies,
    pub adjacency_inline: Option<Vec<Adjacency>>,
    pub digests:  InputDigests,
}

// -------- Top-level orchestration --------
pub fn load_all_from_manifest(path: &std::path::Path) -> Result<LoadedContext, IoError>;

// -------- Targeted loaders --------
pub fn load_registry(path: &std::path::Path) -> Result<DivisionRegistry, IoError>;
pub fn load_params(path: &std::path::Path) -> Result<Params, IoError>;
pub fn load_ballot_tally(path: &std::path::Path) -> Result<UnitTallies, IoError>;
pub fn load_adjacency(path: &std::path::Path) -> Result<Vec<Adjacency>, IoError>;

// -------- Canonicalization & checks --------
pub fn normalize_units(mut units: Vec<Unit>) -> Vec<Unit>;                 // sort ↑ UnitId
pub fn normalize_options(mut opts: Vec<OptionItem>) -> Vec<OptionItem>;   // sort ↑ (order_index, OptionId)
pub fn normalize_tally_options(t: &mut UnitTallies, order: &[OptionItem]); // enforce array order per registry

/// Cross-file referential checks used by Annex-B cases.
pub fn check_cross_refs(
    reg: &DivisionRegistry,
    opts: &[OptionItem],
    tally: &UnitTallies,
    adjacency: Option<&[Adjacency]>,
) -> Result<(), IoError>;
````

7. Algorithm Outline (implementation plan)

* **Orchestrate**

  1. Read + parse manifest (`manifest::load_manifest`), resolve paths, (optionally) enforce expectations/digests.
  2. Load **registry** → schema validate → build `DivisionRegistry` → extract & **normalize**:

     * `units` sorted by `UnitId` ascending.
     * `options` sorted by `(order_index, OptionId)`; check order\_index uniqueness.
  3. Load **parameter\_set** → schema validate → build typed `Params` → `validate_domains` (domain only).
  4. Load **ballot\_tally** (normative) → schema validate → build `UnitTallies` with **options\[] arrays**.
  5. Optional **adjacency**: load/validate if separate file is provided.
  6. **Cross-refs (light)**:

     * `tally.reg_id` (if present on wire) must equal `registry.id`.
     * Every `UnitTotals.unit_id` exists in registry.
     * Every `OptionCount.option_id` exists in registry options.
     * If `adjacency` present: each edge references known units and `a != b`.
  7. **Normalize tally option order** to match registry `(order_index, OptionId)`; units order by `UnitId`.
  8. Compute input **SHA-256 digests** (canonical bytes) for registry, tally, params (+ adjacency if present).
  9. Return `LoadedContext`.

* **Parsing & validation**

  * Use Draft 2020-12 schemas (vm\_io::schema) before deserializing to typed forms.
  * Map first failure to `IoError::Schema { pointer, msg }`.

* **Deterministic ordering**

  * Units by `UnitId`; Options by `(order_index, OptionId)`.
  * Tally `options[]` arrays are re-ordered to match registry order; **no maps**.

* **Safety & limits**

  * Enforce `io.max_bytes` and `io.max_depth` per file.
  * Local files only; URLs already rejected at manifest stage.

8. State Flow
   `vm_cli/vm_pipeline` → `load_all_from_manifest` → receives `LoadedContext` → pipeline runs VALIDATE → TABULATE → … using normalized, typed data and known digests.

9. Determinism & Numeric Rules

* Stable sorts and array ordering before any hashing/serialization.
* Counts are integers; shares (if later computed) are JSON numbers in Result (not here).
* No RNG.

10. Edge Cases & Failure Policy

* Duplicate or missing `order_index` in registry options → error.
* Unknown Unit/Option in tally → error.
* `tally.reg_id` (if present) ≠ `registry.id` → error.
* Empty options array in any `UnitTotals` (shouldn’t happen with proper tally) → error.
* Adjacency self-loops or unknown units → error.
* Files exceeding size/depth limits → `IoError::Limit`.

11. Test Checklist

* **Happy (tally)**: valid registry + params + tally → `LoadedContext` with sorted units/options matching registry order; digests computed.
* **Cross-ref failures**: unknown unit/option rejected; reg mismatch rejected; adjacency bad refs rejected.
* **Determinism**: same inputs with permuted key orders → identical normalized memory layout; canonical JSON hashes equal.
* **DoS guards**: >max\_bytes or >max\_depth fail fast with clear `IoError::Limit`.
* **Schema pointering**: invalid JSON produces `IoError::Schema` with accurate JSON Pointer.

```

```
