<!-- Converted from: 28 - crates vm_core src rng.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.242366Z -->

```
Pre-Coding Essentials (Component: crates/vm_core/src/rng.rs, Version/FormulaID: VM-ENGINE v0) — 28/89
Goal & Success
Goal: Deterministic RNG utilities for tie resolution only, using a fixed, seeded stream cipher (ChaCha20).
Success: With the same integer tie_seed and the same inputs, choices/shuffles are byte-identical across OS/arch; no reliance on OS entropy or time; API is minimal and safe.
Scope
In scope: Seed handling from VM-VAR-033 tie_seed (integer), ChaCha20 wrapper, uniform choice without modulo bias, deterministic shuffle (Fisher–Yates), reproducible u64/u128 streams, small log hook.
Out of scope: Any non-tie randomness, parallel RNG (not permitted), OS RNG, time-based seeding.
Inputs → Outputs
Inputs: tie_seed (VM-VAR-033, integer ≥ 0) when tie_policy (VM-VAR-032) = random; candidate sets; optional domain bounds.
Outputs: Indices/permutes/integers; optional compact trace (context label + picks) to be forwarded to TieLog (owned by pipeline).
Entities/Tables (minimal)
None.
Variables
VM-VAR-032 tie_policy ∈ {status_quo, deterministic, random} (default: status_quo) — RNG used only if = random.
VM-VAR-033 tie_seed ∈ integers (≥ 0) (default: 0) — recorded in RunRecord/TieLog when used.
Functions (signatures only)
/// Opaque deterministic RNG for ties.
pub struct TieRng(ChaCha20Rng);

/// Build from integer tie_seed; stable across platforms.
pub fn tie_rng_from_seed(seed: u64) -> TieRng;

impl TieRng {
/// Next unbiased integer in [0, n) using rejection sampling.
pub fn gen_range(&mut self, n: u64) -> u64;

/// Choose index of winner from non-empty slice; error on empty.
pub fn choose_index<T>(&mut self, slice: &[T]) -> Result<usize, RngError>;

/// Deterministic in-place Fisher–Yates shuffle (stable given same seed).
pub fn shuffle<T>(&mut self, xs: &mut [T]);

/// Emit next u64 / u128 for audit or higher-level use.
pub fn next_u64(&mut self) -> u64;
pub fn next_u128(&mut self) -> u128;

/// Return how many 64-bit words have been consumed.
pub fn words_consumed(&self) -> u128;

/// Optional: record a tiny crumb for TieLog (context/candidates/pick).
pub fn log_pick(&self, ctx: &str, pick: usize) -> TieCrumb;
}

/// Small, serializable crumb (pipeline aggregates into TieLog).
pub struct TieCrumb { pub ctx: SmolStr, pub pick: u32, pub word_index: u128 }
Algorithm Outline (implementation plan)
Seed handling
Initialize RNG with ChaCha20Rng::seed_from_u64(tie_seed).
Start counter at 0; bump by 1 per next_u64 (two bumps for next_u128).
Unbiased range generation
Rejection sampling: draw 64-bit x; compute zone = u64::MAX - (u64::MAX % n); if x < zone, return x % n; else redraw.
Handles any n ∈ [1, 2^63]; reject n = 0.
Choice & shuffle
choose_index: error on empty slice; otherwise gen_range(len).
shuffle: Fisher–Yates descending for i in (1..len).rev() with j = gen_range(i as u64 + 1).
Audit/trace
words_consumed returns monotonic count; TieCrumb stores context string, chosen index (u32 ok for list sizes), and the word index when decision was made.
No parallelism
Callers must serialize all tie resolutions in the deterministic order defined by the pipeline.
State Flow
Pipeline enters RESOLVE_TIES only when needed; it constructs TieRng from Params.tie_seed when Params.tie_policy = random; each tie resolution calls choose_index/gen_range in stable context order; crumbs (optional) are collected and written into the final TieLog in RunRecord/Result.
Determinism & Numeric Rules
Identical tie_seed ⇒ identical output sequence and crumbs.
No floats; no OS RNG/time; no global mutable state.
All consumers must keep a fixed call sequence (stable ordering from determinism module).
Edge Cases & Failure Policy
gen_range(0) or choose_index([]) ⇒ RngError::EmptyDomain.
Extremely skewed n values are fine (rejection loops terminate quickly on average).
Do not expose internal state beyond words_consumed (audit only).
Test Checklist (must pass)
Seed determinism: same u64 seed → identical sequences for next_u64, gen_range, shuffle; different seeds differ.
Unbiasedness (sanity): histogram for gen_range(10) over large N is ~uniform (statistical smoke).
Choice: empty slice errors; non-empty returns valid index.
Shuffle: two runs with same seed produce identical permutation; changing seed changes permutation.
Crumbs: log_pick reports correct word index; sequence of crumbs matches call order.
```
