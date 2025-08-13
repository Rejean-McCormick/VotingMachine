

````
Pre-Coding Essentials (Component: crates/vm_core/src/rng.rs, Version FormulaID VM-ENGINE v0) — 28/89

1) Goal & Success
Goal: Deterministic RNG utilities for tie resolution only, using a fixed, seeded stream cipher (ChaCha20).
Success: With the same integer tie_seed (VM-VAR-052) and the same call sequence, choices/shuffles are byte-identical across OS/arch; no reliance on OS entropy or time; API is minimal and safe.

2) Scope
In scope: Seed handling from VM-VAR-052 (integer ≥ 0), ChaCha20 wrapper, uniform choice without modulo bias, deterministic shuffle (Fisher–Yates), reproducible u64/u128 streams, optional crumb for audit.
Out of scope: Any non-tie randomness, parallel RNG, OS RNG, time-based seeding.

3) Inputs → Outputs
Inputs: `tie_seed` (u64) when `tie_policy` (VM-VAR-050) = Random; candidate slices; optional bounds.
Outputs: Indices/permutes/integers; optional compact crumb (context + pick + word index) for the pipeline’s TieLog.

4) Types (minimal)
- `pub struct TieRng(ChaCha20Rng);`              // opaque
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum RngError { EmptyDomain }`
- `pub struct TieCrumb { pub ctx: SmolStr, pub pick: u32, pub word_index: u128 }`  // optional; pipeline aggregates

5) Functions (signatures only)
```rust
/// Build from integer tie_seed (VM-VAR-052). Stable across platforms.
pub fn tie_rng_from_seed(seed: u64) -> TieRng;

impl TieRng {
    /// Next unbiased integer in [0, n) via rejection sampling. Returns None if n == 0.
    pub fn gen_range(&mut self, n: u64) -> Option<u64>;

    /// Choose index of winner from slice; None on empty slice.
    pub fn choose_index<T>(&mut self, slice: &[T]) -> Option<usize>;

    /// Deterministic in-place Fisher–Yates shuffle.
    pub fn shuffle<T>(&mut self, xs: &mut [T]);

    /// Emit next u64 / u128 (u128 = concat of two u64 draws).
    pub fn next_u64(&mut self) -> u64;
    pub fn next_u128(&mut self) -> u128;

    /// Return how many 64-bit words have been consumed.
    pub fn words_consumed(&self) -> u128;

    /// Optional: build a tiny crumb for audit logs.
    pub fn log_pick(&self, ctx: &str, pick: usize) -> TieCrumb;
}
````

6. Algorithm Outline (implementation plan)

* Seed handling

  * Initialize with `ChaCha20Rng::seed_from_u64(seed)`.
  * Maintain an internal `words_consumed: u128` counter; +1 per `next_u64`, +2 per `next_u128`.

* Unbiased range generation

  * Rejection sampling to avoid modulo bias:

    * Draw `x = next_u64()`.
    * `let zone = u64::MAX - (u64::MAX % n);`
    * If `n == 0` → `None`; else loop until `x < zone`, return `x % n`.

* Choice & shuffle

  * `choose_index`: `slice.is_empty() ? None : Some(gen_range(len as u64)? as usize)`.
  * `shuffle`: standard Fisher–Yates (descending `i`, choose `j ∈ [0, i]` via `gen_range`).

* Audit crumb (optional)

  * `TieCrumb { ctx: SmolStr::new(ctx), pick: pick as u32, word_index: self.words_consumed() }`.

* No parallelism

  * Callers must resolve ties in the deterministic order defined elsewhere (options ordered by `(order_index, option_id)`).

7. State Flow
   Pipeline enters RESOLVE\_TIES only when needed; constructs `TieRng` from `Params.v052_tie_seed` when `v050_tie_policy == Random`; every tie uses `choose_index`/`gen_range` in a fixed call sequence; crumbs (optional) collected and later written into **RunRecord.ties\[]**.

8. Determinism & Numeric Rules

* Identical `tie_seed` + identical call sequence ⇒ identical outputs and crumbs.
* No floats; no OS RNG/time; no globals. Single stream, single thread.

9. Edge Cases & Failure Policy

* `gen_range(0)` or `choose_index([])` ⇒ `None` (callers handle and never call with empty domain in production).
* Highly skewed `n` values are fine; rejection loop terminates quickly in expectation.
* Internal state is opaque; only `words_consumed()` is exposed for audit.

10. Test Checklist (must pass)

* Seed determinism: same `u64` seed → identical sequences for `next_u64`, `gen_range`, `shuffle`; different seeds diverge.
* Unbiasedness (smoke): histogram for `gen_range(10)` over large N is \~uniform.
* Choice: empty slice ⇒ `None`; non-empty returns valid index in range.
* Shuffle: two runs with same seed produce identical permutation; different seed changes permutation.
* Crumbs: `log_pick` reports the correct `word_index`; crumb sequence matches call order.

11. Notes for coding

* Keep this module self-contained and I/O-free.
* Do not use `rand::thread_rng()` or any OS entropy source.
* Keep signatures returning `Option` for empty domains to match the core API style used elsewhere (e.g., `choose(..) -> Option<usize>`).
* Any additional helpers must not expose internal RNG state beyond what’s specified.

```


```
