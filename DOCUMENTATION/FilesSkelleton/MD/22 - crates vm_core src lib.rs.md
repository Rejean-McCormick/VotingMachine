Here’s the corrected, reference-aligned skeleton sheet for **22 – crates/vm\_core/src/lib.rs.md**. I fixed the misalignments: dropped non-normative input IDs (no `REG:`/`TLY:`/`PS:` types), made tie seed an **integer** (`VM-VAR-052`), kept deterministic order by `order_index` (no extra “key”), modeled `algorithm_variant` (073), and kept this crate **I/O-free**.

````
Pre-Coding Essentials (Component: crates/vm_core/src/lib.rs, Version/FormulaID: VM-ENGINE v0) — 22/89

1) Goal & Success
Goal: Expose the stable, minimal public API for core engine types: IDs (outputs only), registry tokens, VM-VAR domains, deterministic ordering, integer-first numerics, and a seedable RNG for ties.
Success: Other crates (`vm_io`, `vm_algo`, `vm_pipeline`, `vm_report`, `vm_cli`) depend on these types/traits only. Builds on all targets with/without the optional `serde` feature. No file/JSON I/O here.

2) Scope
In scope: module declarations; `pub use` re-exports; output IDs (`RES:`, `RUN:`, `FR:`), `unit_id`/`option_id` token types; VM-VAR enums/typed fields (050, 073, etc.); deterministic ordering helpers; integer/ratio utilities; seeded RNG adapter; small core error enums.
Out of scope: file/JSON I/O (lives in `vm_io`), pipeline orchestration, report formatting, CLI, hashing/canonicalization bytes (interfaces only; implemented in `vm_io`).

3) Inputs → Outputs (artifacts/IDs)
Inputs: none at runtime (library crate).
Outputs (types/API only):
- ids::{ResultId, RunId, FrontierMapId, FormulaId, Sha256}   // outputs & digests
- tokens::{UnitId, OptionId}                                  // registry tokens (no prefixed input IDs)
- variables::{Params, TiePolicy(050), AlgorithmVariant(073), /* other Included VM-VARs as typed fields */}
- determinism::{StableOrd, cmp_options_by_order, sort_units_by_id}
- rounding::{Ratio, new_ratio_checked, compare_ratio_half_even}
- rng::{TieRng, tie_rng_from_seed}
- errors::{CoreError}

4) Entities/Types (module inventory)
- `ids`: newtypes + `FromStr/Display` for:
  - `ResultId`  ("RES:" + 64-hex)
  - `RunId`     ("RUN:" + UTC-compact + "-" + 64-hex)
  - `FrontierMapId` ("FR:" + 64-hex)
  - `FormulaId` (64-hex FID)
  - `Sha256`    (64-hex)
- `tokens`: newtypes for `UnitId`, `OptionId` (pattern: `[A-Za-z0-9_.:-]{1,64}`)
- `variables`:
  - `TiePolicy` enum = { `StatusQuo`, `DeterministicOrder`, `Random` }   // VM-VAR-050
  - `AlgorithmVariant` enum (VM-VAR-073)  // exact variants enumerated per release
  - `Params` struct with **typed fields** for Included VM-VARs (001–007, 010–017, 020–031 incl. 021 & 029–031, 040–049, 050, 073) and optional Excluded (032–035, 052, 060–062)
  - `tie_seed` (VM-VAR-052) as `u64` (≥0) — **Excluded** from FID
  - `validate_domains(&Params) -> Result<(), CoreError>` (domain checks only; no cross-artifact checks)
  - `defaults() -> Params`
- `determinism`:
  - `trait StableOrd { fn stable_key(&self) -> impl Ord; }`
  - `fn cmp_options_by_order(a:&RegOptionMeta, b:&RegOptionMeta) -> Ordering` // order_index then option_id
  - `fn sort_units_by_id(ids:&mut [UnitId])`
- `rounding`:
  - `struct Ratio { pub num: i128, pub den: i128 }`  // den > 0; sign normalized
  - `fn new_ratio_checked(num:i128, den:i128) -> Result<Ratio, CoreError>`
  - `fn compare_ratio_half_even(a:&Ratio, b:&Ratio) -> Ordering`
- `rng`:
  - `struct TieRng(ChaCha20Rng);`
  - `fn tie_rng_from_seed(seed: u64) -> TieRng`  // integer seed (VM-VAR-052)
  - `impl TieRng { pub fn choose<T: StableOrd>(&mut self, slice:&[T]) -> Option<usize> }`
- `errors`:
  - `enum CoreError { InvalidId, InvalidToken, InvalidRatio, DomainOutOfRange(&'static str), EmptyChoiceSet }`

(De/)serialization derives are behind `#[cfg(feature = "serde")]` only.

5) Variables (only ones used here)
- VM-VARs represented as **typed fields** in `Params`:
  - 050 `tie_policy: TiePolicy` (**Included in FID**)
  - 052 `tie_seed: u64` (**Excluded**; recorded in RunRecord only if random ties happened)
  - 073 `algorithm_variant: AlgorithmVariant` (**Included**)
  - Booleans are real `bool` (no "on"/"off" strings). Percentages are bounded integers (0..=100) where applicable.

6) Functions (signatures only — stable API surface)
IDs & tokens:
- `impl FromStr for ResultId/RunId/FrontierMapId/FormulaId/Sha256`
- `impl FromStr for UnitId/OptionId`
Helpers:
- `pub fn cmp_options_by_order(a:&RegOptionMeta, b:&RegOptionMeta) -> Ordering`
- `pub fn sort_units_by_id(ids:&mut [UnitId])`
Numeric policy:
- `pub fn new_ratio_checked(num:i128, den:i128) -> Result<Ratio, CoreError>`
- `pub fn compare_ratio_half_even(a:&Ratio, b:&Ratio) -> Ordering`
RNG (ties only):
- `pub fn tie_rng_from_seed(seed:u64) -> TieRng`
- `impl TieRng { pub fn choose<T:StableOrd>(&mut self, slice:&[T]) -> Option<usize> }`
Variables:
- `pub fn defaults() -> Params`
- `pub fn validate_domains(p:&Params) -> Result<(), CoreError>`

7) Module layout (authoring outline)
```rust
pub mod ids;         // RES/RUN/FR/FormulaId/Sha256 newtypes + parsing
pub mod tokens;      // UnitId, OptionId
pub mod variables;   // Params + VM-VAR enums (050, 073, etc.), domain validators
pub mod determinism; // StableOrd, cmp_options_by_order, sort_units_by_id
pub mod rounding;    // Ratio + comparison/rounding helpers
pub mod rng;         // TieRng seeded from integer; no OS entropy
pub mod errors;      // CoreError

pub use ids::*;
pub use tokens::*;
pub use variables::*;
pub use determinism::*;
pub use rounding::*;
pub use rng::*;
pub use errors::*;
````

8. State Flow
   Downstream: `vm_io` provides (de)serialization + canonicalization; `vm_algo` consumes rounding/RNG/helpers; `vm_pipeline` orchestrates and populates Result/RunRecord using these types.

9. Determinism & Numeric Rules

* Option ordering: strictly `order_index` then `option_id`. No “deterministic\_order\_key”: the key is fixed.
* Integer-first math; `Ratio` used for exact comparisons; rounding (half-even) only where specified.
* RNG used **only** for ties when `TiePolicy::Random`; seed is a **u64** (VM-VAR-052). Same seed → identical sequences across OS/arch.

10. Edge Cases & Failure Policy

* ID/token parsing: strict prefixes/patterns; `FromStr` returns `Err(CoreError::InvalidId)` / `InvalidToken`.
* `Ratio`: `den <= 0` → error; sign normalized to keep `den > 0`.
* RNG: `choose(&[])` → `None` (no panic).
* Params domain: out-of-range percentages, bad enums → `DomainOutOfRange`; cross-artifact checks are out of scope here.

11. Test Checklist (must pass)

* IDs: valid parse/round-trip; malformed rejected.
* Ordering: options sorted by (order\_index, option\_id) deterministically.
* Ratios: constructor rejects zero/negative den; comparisons are transitive; ties honor half-even.
* RNG: same `u64` seed → identical choice indices across platforms; different seeds → diverging sequences.
* Params: defaults compile; booleans are real `bool`; 052 is `u64`; 050/073 typed correctly.

```

If you want, I can now turn this into a **real `src/lib.rs` scaffold** (with module stubs, newtype definitions, and `cfg(feature="serde")` derives) for direct copy-paste into your repo.
```
