<!-- Converted from: 34 - crates vm_io src loader.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.433020Z -->

```
Pre-Coding Essentials (Component: crates/vm_io/src/loader.rs, Version/FormulaID: VM-ENGINE v0) — 34/89
1) Goal & Success
Goal: Load local JSON artifacts (manifest → registry → params → ballots or tally, optional adjacency), validate them against schemas, normalize ordering, and return a typed LoadedContext for the pipeline.
Success: Given a valid manifest, returns a fully-typed context with: IDs parsed, units/options sorted canonically, tallies/ballots shaped for tabulation, and early referential checks (unit/option IDs exist, reg_id matches).
2) Scope
In scope: File read, JSON parse, schema validation calls, ID parsing, canonical ordering, light referential checks, and construction of ephemeral types (UnitTallies/BallotsRaw, LoadedContext).
Out of scope: Heavy semantic validation (tree/root/magnitude rules, gates math), allocation/tabulation, report writing.
3) Inputs → Outputs
Inputs: Paths from manifest::ResolvedPaths.
Outputs:
LoadedContext { reg, options, params, tally_or_ballots, adjacency_inline?, ids }
Detected BallotSource (raw ballots vs tally).
4) Entities/Tables (minimal)
Options are expected to be explicit (with order_index) in the registry artifact; loader requires them for deterministic ordering.
5) Variables (loader knobs)
6) Functions (signatures only)
rust
CopyEdit
use crate::{IoError};
use vm_core::{ids::*, entities::*, variables::Params};

pub enum TallyOrBallots { Tally(UnitTallies), Ballots(BallotsRaw) }

pub struct LoadedIds {
pub reg_id: RegId,
pub param_set_id: ParamSetId,
pub tally_id: Option<TallyId>, // None when raw ballots
}

pub struct LoadedContext {
pub reg: DivisionRegistry,
pub options: Vec<OptionItem>,
pub params: Params,
pub tally_or_ballots: TallyOrBallots,
pub adjacency_inline: Option<Vec<Adjacency>>,
pub ids: LoadedIds,
}

// Top-level orchestration
pub fn load_all_from_manifest(path: &std::path::Path) -> Result<LoadedContext, IoError>;

// Individual loaders (used by the above and by tests)
pub fn load_registry(path: &std::path::Path) -> Result<(DivisionRegistry, Vec<OptionItem>), IoError>;
pub fn load_params(path: &std::path::Path) -> Result<Params, IoError>;
pub fn load_tally(path: &std::path::Path) -> Result<(TallyId, UnitTallies), IoError>;
pub fn load_ballots(path: &std::path::Path) -> Result<BallotsRaw, IoError>;

// Cross-checks & normalization
pub fn normalize_options(mut opts: Vec<OptionItem>) -> Vec<OptionItem>; // sort by (order_index, id) + uniqueness checks
pub fn normalize_units(mut units: Vec<Unit>) -> Vec<Unit>;              // sort by UnitId
pub fn check_cross_refs(reg: &DivisionRegistry, opts: &[OptionItem], tally: &UnitTallies) -> Result<(), IoError>;

7) Algorithm Outline (implementation plan)
Orchestrate
Read + parse manifest (manifest::load_manifest), resolve paths, enforce expectations/digests.
Load registry → schema validate → parse IDs → extract options[] and units[] → normalize_options/normalize_units.
Load parameter set → schema validate → build Params (typed) → validate_params (domain only).
Choose source
If tally path present: load tally → schema validate → parse TlyId.
Else: load raw ballots → schema validate → keep in BallotsRaw.
Cross-checks (when tally)
tally.reg_id == reg.id (strict).
Every tally.units[i].unit_id exists in registry.
Every option key referenced in tallies exists in options (by OptionId).
Option order_index uniqueness and monotonicity (no duplicates, ≥1).
Canonical ordering
Sort units by UnitId ascending; options by (order_index, OptionId); for each unit’s option maps, re-materialize as key-sorted structures (BTreeMap) to make downstream hashing independent of input order.
Return
Build LoadedIds { reg_id, param_set_id, tally_id? }; place adjacency list if registry contains it; return LoadedContext.
8) State Flow
vm_cli/vm_pipeline calls load_all_from_manifest → receives LoadedContext → pipeline runs VALIDATE → TABULATE → … using normalized, typed data.
9) Determinism & Numeric Rules
Determinism via stable sorts and key-sorted maps before any hashing/serialization.
No floats parsed; counts remain integers.
No RNG.
10) Edge Cases & Failure Policy
Missing/duplicate order_index → IoError::Manifest("option order_index duplicate/missing").
Unknown unit_id/OPT: in tallies → IoError::Manifest("unknown unit/option id").
reg_id mismatch between tally and registry → IoError::Manifest("tally.reg_id != registry.id").
Oversized file / parse depth exceeded → IoError::Read/Json with explicit limit names.
Raw ballots supplied: skip cross-checks that need tallies; pipeline will validate semantics after tabulation.
Loader never mutates counts; only sorts/normalizes structures.
11) Test Checklist (must pass)
Happy (tally): Proper registry + params + tally → returns LoadedContext with sorted units/options and matching IDs.
Happy (raw ballots): Proper registry + params + ballots → returns context with TallyOrBallots::Ballots.
Cross-ref failures: unknown unit/option rejected; reg_id mismatch rejected.
Option ordering: duplicates or out-of-domain order_index rejected; equal index values break ties by OptionId only after uniqueness check.
Determinism: same inputs in permuted key orders → identical normalized memory layout (verified by serializing to canonical JSON and comparing hashes).
```
