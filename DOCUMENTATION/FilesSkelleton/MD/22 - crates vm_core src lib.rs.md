<!-- Converted from: 22 - crates vm_core src lib.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.096590Z -->

```
Pre-Coding Essentials (Component: crates/vm_core/src/lib.rs, Version/FormulaID: VM-ENGINE v0) — 22/89
1) Goal & Success
Goal: Public surface of vm_core (IDs, entities, variables, numeric policy, RNG), re-exporting submodules with minimal, stable API.
Success: Other crates (vm_io, vm_algo, vm_pipeline, vm_report, vm_cli) depend only on vm_core types/traits—no I/O here; builds on all targets with/without the optional serde feature.
2) Scope
In scope: module declarations, pub use re-exports, core result types, deterministic ordering helpers, numeric/rounding traits, seeded RNG handle, small error enums (core-only).
Out of scope: file/JSON I/O, CLI, state machine orchestration, report formatting.
3) Inputs → Outputs (with schemas/IDs)
Inputs: none at runtime (library crate).
Outputs: public API:
 ids::{RegId, UnitId, OptionId, TallyId, ParamSetId, ResultId, RunId, FrontierId},
 entities::{DivisionRegistry, Unit, Option, …},
 variables::{VmVar, Params},
 determinism::{StableOrd, HashCanon},
 rounding::{Ratio, compare_ratio_half_even},
 rng::{TieRng}.
4) Entities/Tables (minimal)
(Core provides types; vm_io owns serialization.)
5) Variables (only ones used here)
6) Functions (signatures only)
IDs & parsing:
pub fn parse_reg_id(s:&str) -> Option<RegId>
pub fn parse_unit_id(s:&str) -> Option<UnitId>
pub fn parse_option_id(s:&str) -> Option<OptionId>
Deterministic ordering helpers:
pub fn cmp_options(a:&Option, b:&Option) -> Ordering // by order_index then id
pub fn sort_units_stable(ids:&mut [UnitId])
Numeric policy:
pub struct Ratio { pub num:i128, pub den:i128 }
pub fn compare_ratio_half_even(a:&Ratio, b:&Ratio) -> Ordering
RNG (ties only):
pub struct TieRng(ChaCha20Rng);
pub fn tie_rng_from_seed(hex64:&str) -> Result<TieRng, CoreError>
impl TieRng { pub fn choose<T:StableOrd>(&mut self, slice:&[T]) -> usize }
Variables:
pub struct Params { /* VM-VAR map materialized into typed fields */ }
pub fn params_default() -> Params
pub fn validate_params(p:&Params) -> Result<(), CoreError> // domain checks only
Hash canon (interface only; implementation in downstream if needed):
pub trait HashCanon { fn canonical_bytes(&self) -> Vec<u8>; }
7) Algorithm Outline (module layout)
pub mod ids; — newtypes + parsers + regex guards for REG:/U:/OPT:/TLY:/PS:/RES:/RUN:/FR:.
pub mod entities; — structs for core entities; Option includes order_index:int.
pub mod variables; — Params + VmVar enums/constants; domain-level validators (no cross-artifact checks).
pub mod determinism; — StableOrd trait; comparators for units/options; canonical sorting utilities.
pub mod rounding; — Ratio, compare_ratio_half_even, integer/rational comparison helpers.
pub mod rng; — TieRng wrapper over ChaCha20 seeded from hex64; no OS entropy.
pub use re-exports from these modules for downstream crates.
8) State Flow (very short)
Downstream crates import vm_core::* for types/traits → vm_io handles I/O & schema validation → vm_algo uses rounding, rng, StableOrd → vm_pipeline orchestrates.
9) Determinism & Numeric Rules
Stable total orders: Units by UnitId; Options by order_index then OptionId.
No float comparisons: expose ratio/int APIs; round half-to-even only at defined comparison points.
RNG: only via TieRng with explicit 64-hex seed; no time/OS RNG.
10) Edge Cases & Failure Policy
ID parsers strictly validate prefix and shape; return None/Err on mismatch.
Ratio constructors must reject den ≤ 0 and normalize sign (store positive den).
tie_rng_from_seed rejects non-hex/invalid length.
cmp_options must be total and stable even with equal order_index (break ties by OptionId).
11) Test Checklist (must pass)
ID parsing round-trips: valid shapes parse; malformed shapes fail.
Option ordering: (order_index, id) sorting stable; deterministic across platforms.
Ratio comparisons: property tests confirm transitivity; tie cases follow half-even rule.
RNG: same seed → identical choice sequences; different seeds → different sequences; no panics on empty slices (return error).
Params default values match Doc 2 defaults; domain checks reject out-of-range percentages or inconsistent combos (random tie without seed not checked here—pipeline enforces when chosen).
```
