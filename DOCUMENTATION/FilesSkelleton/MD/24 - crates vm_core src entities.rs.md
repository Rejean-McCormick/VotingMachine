<!-- Converted from: 24 - crates vm_core src entities.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.158060Z -->

```
Pre-Coding Essentials (Component: crates/vm_core/src/entities.rs, Version/FormulaID: VM-ENGINE v0) — 24/89
1) Goal & Success
Goal: Define the domain types used across the engine (registry, units, options, adjacency, common blocks like turnout/labels), with no I/O and stable semantics aligned to Docs 1–7 & Annex.
Success: Types compile on all targets; invariants are encoded (e.g., magnitude ≥ 1); sorting helpers exist (units by UnitId, options by order_index then OptionId); no JSON/FS dependencies.
2) Scope
In scope: Structs/enums for DivisionRegistry, Unit, OptionItem, Adjacency, Provenance, Turnout, DecisivenessLabel; thin constructors/validators.
Out of scope: Parameter variables (live in variables.rs), ID parsing (in ids.rs), serialization (in vm_io), pipeline ephemera (lives in vm_pipeline), report rendering.
3) Inputs → Outputs
Inputs: none at runtime (library definitions).
Outputs: Strongly-typed values used by vm_io (decode/encode), vm_algo (compute), vm_pipeline (state machine), vm_report (mapping).
4) Entities/Tables (minimal)
(IDs come from ids.rs; variables from variables.rs.)
5) Variables (only ones used here)
6) Functions (signatures only)
DivisionRegistry
pub fn new(id:RegId, name:String, version:String, provenance:Provenance, units:Vec<Unit>, adjacency:Vec<Adjacency>) -> Result<Self, EntityError>
pub fn root_units(&self) -> impl Iterator<Item=&Unit>
pub fn unit(&self, id:&UnitId) -> Option<&Unit>
Unit
pub fn new(...) -> Result<Self, EntityError> (checks magnitude≥1, non-negative rolls/baselines, parent≠self)
pub fn is_root(&self) -> bool
OptionItem
pub fn new(id:OptionId, display_name:String, order_index:u16, is_status_quo:bool) -> Result<Self, EntityError>
Adjacency
pub fn new(a:UnitId, b:UnitId, edge:EdgeType) -> Result<Self, EntityError> (reject a==b)
Sorting helpers (deterministic)
pub fn sort_units_by_id(units:&mut [Unit])
pub fn sort_options_canonical(opts:&mut [OptionItem]) // by order_index then id
7) Algorithm Outline (implementation plan)
Define data enums:
EdgeType = { Land, Bridge, Water }
DecisivenessLabel = { Decisive, Marginal, Invalid }
YyyyOrIsoDate as a tiny tagged enum or validated String newtype.
Define structs as above; derive Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash where meaningful.
Implement constructors that enforce local invariants:
magnitude≥1; eligible_roll≥0.
If population_baseline.is_some() then population_baseline_year.is_some() (pairing rule).
Adjacency: a != b.
Provide deterministic sort helpers:
Units by UnitId (lexicographic).
Options by order_index then OptionId.
Keep no serialization code here. serde derives gated behind feature="serde" with #[serde(transparent)] only for simple newtypes.
8) State Flow (very short)
vm_io constructs these from validated JSON; vm_algo consumes them; vm_pipeline aggregates and labels results; vm_report reads finalized results (not defined here).
9) Determinism & Numeric Rules
Stable total orders exposed via helpers (Units by ID; Options by order_index then ID).
No floats; counts and baselines are integers.
Presentation rounding happens in report layer; nothing here rounds.
10) Edge Cases & Failure Policy
Multiple roots or zero roots are not validated here; leave to pipeline VALIDATE step.
Missing baseline fields are allowed here (optional) but become pipeline errors when population weighting is enabled.
valid_ballots should equal ballots_cast - invalid_or_blank; constructor for Turnout enforces that or computes it.
Adjacency duplicates or cross-registry edges are checked later (pipeline).
11) Test Checklist (must pass)
Unit::new rejects magnitude=0, accepts magnitude≥1.
Turnout::new(100, 7) yields valid_ballots=93; negative-like underflows (u64) impossible by API.
Option sort: (order_index,id) total and stable; equal order_index breaks ties by OptionId.
Adjacency::new(a,b,…) rejects a==b.
Sorting helpers produce the same order on all OS/arch.
Optional baseline pair invariant enforced (value ↔ year).
Notes for coding
Keep this file purely domain (no path logic, no JSON).
Document each invariant with a doc comment and a unit test.
Public API should be minimal; most mutation via constructors to keep invariants true.
```
