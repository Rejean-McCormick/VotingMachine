
````
Pre-Coding Essentials (Component: crates/vm_core/src/variables.rs, Version/FormulaID: VM-ENGINE v0) — 25/89

1) Goal & Success
Goal: Define typed VM variables (VM-VAR-###) and a Params struct with defaults + domain validation, independent of I/O.
Success: `Params::default()` compiles and reflects per-release defaults; `validate_domains(&Params)` enforces ranges/enums/iff rules that are purely domain-level (no cross-artifact checks); optional serde derives are gated by a feature (no serde_json here).

2) Scope
In scope: enums per family (ballot/allocation/gates/frontier/ties/MMP/etc.), `Params` with typed fields, default constants, domain validation helpers.
Out of scope: schema parsing/JSON (lives in vm_io), pipeline semantics/state, Formula ID hashing or canonicalization (done elsewhere).

3) Inputs → Outputs
Inputs: none at runtime (library types).
Outputs: `Params` (typed snapshot) + helpers like `is_random_ties()`, `frontier_enabled()`, `iter_fid()` (yields only Included vars).

4) Entities/Types (inventory)
- Core enums (snake_case on wire when serde is enabled):
  * `TiePolicy` = { `StatusQuo`, `DeterministicOrder`, `Random` }                  // VM-VAR-050 (Included)
  * `AlgorithmVariant` … per release (non-exhaustive)                               // VM-VAR-073 (Included)
  * `FrontierMode` = { `None`, `Banded`, `Ladder` }                                 // VM-VAR-040 (Included)
  * `FrontierStrategy` = { `ApplyOnEntry`, `ApplyOnExit`, `Sticky` }                // VM-VAR-042 (Included)
  * `ProtectedAreaOverride` = { `Deny`, `Allow` }                                   // VM-VAR-045 (Included)
  * `FrontierBackoffPolicy` = { `None`, `Soften`, `Harden` }                        // VM-VAR-048 (Included)
  * `FrontierStrictness` = { `Strict`, `Lenient` }                                  // VM-VAR-049 (Included)
  * `UnitSortOrder` = { `UnitId`, `LabelPriority`, `Turnout` }                      // VM-VAR-032 (Excluded)
  * `TiesSectionVisibility` = { `Auto`, `Always`, `Never` }                         // VM-VAR-033 (Excluded)
  * `DecisivenessLabelPolicy` = { `Fixed`, `DynamicMargin` }                        // VM-VAR-061 (Excluded)
- Minor newtypes:
  * `Pct(u8)`  // 0..=100, constructor enforces range
  * `RunScope` = `AllUnits` | `Selector(StringToken)`                               // VM-VAR-021 (Included)
  * `StringToken` // `[A-Za-z0-9_.:-]{1,64}` validated token used in selectors/symmetry list
  * `EligibilityOverride { unit_id: UnitId, mode: Include|Exclude }`               // VM-VAR-030 (Included)
- Opaque maps typed but content defined per release:
  * `AutonomyPackageMap` (deterministic key order)                                  // VM-VAR-046 (Included)

5) Params struct (Included vs Excluded sets)
```rust
pub struct Params {
  // Included (FID) — required
  pub v001_algorithm_family: String,          // enum per release
  pub v002_rounding_policy: String,           // enum per release
  pub v003_share_precision: u8,               // 0..=6
  pub v004_denom_rule: String,                // enum per family
  pub v005_aggregation_mode: String,
  pub v006_seat_allocation_rule: String,
  pub v007_tie_scope_model: String,           // e.g., "winner_only" | "rank_all"

  pub v010: Pct, pub v011: Pct, pub v012: Pct, pub v013: Pct,
  pub v014: Pct, pub v015: Pct, pub v016: Pct, pub v017: Pct,

  pub v020: Pct,
  pub v021_run_scope: RunScope,
  pub v022: Pct, pub v023: Pct,
  pub v024_flag_a: bool,                      // if defined as boolean in Annex A
  pub v025_flag_b: bool,                      // if defined as boolean in Annex A
  pub v026: i32,                              // or f32/f64 per release domain
  pub v027: i32,
  pub v028: i32,
  pub v029_symmetry_exceptions: Vec<StringToken>,
  pub v030_eligibility_override_list: Vec<EligibilityOverride>,
  pub v031_ballot_integrity_floor: Pct,

  pub v040_frontier_mode: FrontierMode,
  pub v041_frontier_cut: f32,                 // domain per mode (documented)
  pub v042_frontier_strategy: FrontierStrategy,
  pub v045_protected_area_override: ProtectedAreaOverride,
  pub v046_autonomy_package_map: AutonomyPackageMap,
  pub v047_frontier_band_window: f32,         // 0.0..=1.0
  pub v048_frontier_backoff_policy: FrontierBackoffPolicy,
  pub v049_frontier_strictness: FrontierStrictness,

  pub v050_tie_policy: TiePolicy,
  pub v073_algorithm_variant: AlgorithmVariant,

  // Excluded (non-FID) — optional
  pub v032_unit_sort_order: Option<UnitSortOrder>,
  pub v033_ties_section_visibility: Option<TiesSectionVisibility>,
  pub v034_frontier_map_enabled: Option<bool>,
  pub v035_sensitivity_analysis_enabled: Option<bool>,
  pub v052_tie_seed: Option<u64>,             // integer ≥ 0 (seed recorded in RunRecord iff random ties occurred)
  pub v060_majority_label_threshold: Option<Pct>,
  pub v061_decisiveness_label_policy: Option<DecisivenessLabelPolicy>,
  pub v062_unit_display_language: Option<String>, // "auto" or IETF tag
}
````

6. Functions (signatures only)

```rust
impl Default for Params { fn default() -> Self }           // per-release defaults (document constants DEF_*)
pub fn validate_domains(p: &Params) -> Result<(), VarError>; // ranges/enums/iff (domain only)

// Convenience predicates
impl Params {
  pub fn is_random_ties(&self) -> bool { matches!(self.v050_tie_policy, TiePolicy::Random) }
  pub fn frontier_enabled(&self) -> bool { !matches!(self.v040_frontier_mode, FrontierMode::None) }
  pub fn pr_threshold(&self) -> Option<u8> { Some(self.v022.0) } // example normalization
  pub fn iter_fid<'a>(&'a self) -> FidIter<'a>;                  // yields (vm_var_id, value_view) for Included set only
}

// (Serde helpers are *not* provided here; vm_io handles JSON. Optional derives only.)
```

7. Validation rules (domain-level — no cross-artifact checks)

* Percentages: all `Pct` in **0..=100**.
* `v003_share_precision`: **0..=6**.
* Frontier:

  * `v047_frontier_band_window`: **0.0..=1.0**.
  * `v041_frontier_cut`: domain depends on `v040_frontier_mode` (document per release; enforce numeric bounds if defined).
* Ties:

  * `v050_tie_policy` is enum.
  * `v052_tie_seed` if present must be `>= 0` (u64). **Do not require** it when policy is `Random`; runtime records it only if a random tie actually occurred.
* Basic iff examples (keep domain-only here; deeper coupling is pipeline/schema territory):

  * If the chosen algorithm family requires 013–017 (MMP style), ensure they are within range (presence is guaranteed by struct).
  * Booleans are real booleans (no `"on"|"off"` strings anywhere).
  * 029/030/031 use the official names and shapes (no invented “weighting\_method” etc.).

8. State Flow
   vm\_io builds `Params` from JSON and calls `validate_domains`; vm\_pipeline uses it to drive steps (tabulate/allocate/gates/frontier/ties); RunRecord echoes `vars_effective` and tie policy/seed per runtime rules.

9. Determinism & Numeric Rules

* All numeric fields are integers except the documented frontier floats; no hidden floats elsewhere.
* RNG is seeded from **`v052_tie_seed: Option<u64>`** and only used when `v050_tie_policy == Random`.

10. Edge Cases & Failure Policy

* Out-of-range percentages/precision ⇒ `VarError::OutOfRange { var: "VM-VAR-###" }`.
* Invalid selector/token shapes in 021/029 ⇒ `VarError::BadToken`.
* `v047_frontier_band_window` outside 0..=1 ⇒ `VarError::OutOfRange`.
* Any boolean serialized as text is rejected by vm\_io before it reaches here (this module is typed).

11. Test Checklist

* `Params::default()` compiles; each default inside documented bounds.
* `validate_domains`:

  * `v003_share_precision = 7` ⇒ Err.
  * `Pct(-1 or >100)` anywhere ⇒ Err.
  * `v047_frontier_band_window = 1.1` ⇒ Err.
  * `v052_tie_seed = Some(0)` ⇒ Ok; absent while `v050=Random` ⇒ Ok (seed recorded at runtime if used).
* Predicates: `is_random_ties()` and `frontier_enabled()` return expected values.
* `iter_fid()` yields exactly the Included set: `001–007, 010–017, 020–031 (incl. 021 & 029–031), 040–049, 050, 073`.

Notes for coding

* Keep this file I/O-free. Optional `#[cfg(feature="serde")]` derives for `Params` and enums are fine, but JSON map conversions live in `vm_io`.
* Document each field with the VM-VAR number and a one-line domain note; add unit tests for each guard.

````

**Error & iterator sketches (for implementers):**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarError {
  OutOfRange { var: &'static str },
  BadToken { var: &'static str },
  Unsupported { var: &'static str },
}

pub struct FidIter<'a> { /* yields (&'static str /*"VM-VAR-###"*/, ValueView<'a>) */ }
````


