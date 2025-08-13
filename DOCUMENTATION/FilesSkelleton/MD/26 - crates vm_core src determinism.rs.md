
````
Pre-Coding Essentials (Component: crates/vm_core/src/determinism.rs, Version/FormulaID: VM-ENGINE v0) — 26/89

1) Goal & Success
Goal: Provide core utilities that enforce stable total ordering and deterministic reduction across the engine.
Success: All merges/sorts use canonical orders (Units by UnitId; Options by (order_index, option_id)); reductions are order-independent under parallelization; byte output doesn’t vary by OS/arch or thread layout.

2) Scope
In scope: ordering traits/helpers, canonical sorting, deterministic reducers, map canonicalization glue.
Out of scope: RNG (rng.rs), numeric comparisons/rounding (rounding.rs), any hashing/I/O.

3) Inputs → Outputs
Inputs: collections of tokens/entities and partial results from parallel stages.
Outputs: stably ordered slices/maps and order-independent reduction results, ready for canonical serialization & hashing (done elsewhere).

4) Entities/Types (minimal)
- Depends on `UnitId`, `OptionId` (tokens) and `OptionItem` (has `order_index`, `option_id`).

5) Variables
None (module is pure/functional).

6) Functions (signatures only)
```rust
// 1) Ordering primitives
pub trait StableOrd {
    fn stable_cmp(&self, other: &Self) -> core::cmp::Ordering;
}
impl StableOrd for UnitId { /* lexicographic by token */ }
impl StableOrd for OptionId { /* lexicographic by token */ }
// Requires OptionItem { order_index: u16, option_id: OptionId }
impl StableOrd for OptionItem { /* (order_index, option_id) */ }

// 2) Canonical sort helpers (in-place; stable across platforms)
pub fn sort_units_by_id<T: AsRef<UnitId>>(xs: &mut [T]);
pub fn sort_options_canonical(xs: &mut [OptionItem]);        // (order_index, option_id)
pub fn cmp_options_by_order(a: &OptionItem, b: &OptionItem) -> core::cmp::Ordering;

// 3) Deterministic reduction (order-independent if combine is associative)
pub trait StableReduce: Sized {
    fn identity() -> Self;
}
pub fn reduce_deterministic<T, F>(mut items: Vec<T>, combine: F) -> Option<T>
where
    T: StableReduce + StableOrd,
    F: Fn(T, T) -> T;

// 4) Map canonicalization (always key-ordered)
pub fn btreemap_from_iter_kv<K: Ord, V, I: IntoIterator<Item = (K, V)>>(
    it: I
) -> alloc::collections::BTreeMap<K, V>;

// 5) Canonical bytes (interface only; implemented where serialization lives)
pub trait HashCanon { fn canonical_bytes(&self) -> Vec<u8>; }
````

7. Implementation Outline

* **Stable orders**

  * `UnitId`: `Ord` on the canonical token string → total order; `StableOrd` delegates.
  * `OptionItem`: compare `order_index` first; if equal, compare `option_id`.
  * No knobs: deterministic option order is **always** `(order_index, option_id)`.

* **Canonical sort**

  * Provide thin, zero-alloc wrappers (`sort_*`) around in-place sorts using the above rules.
  * Keep `cmp_options_by_order` public for callers that need custom sorting contexts.

* **Deterministic reduction**

  * Strategy: sort the inputs by `StableOrd`, then fold with `combine`.
  * Parallelization pattern: reduce chunks locally, collect chunk results, then call `reduce_deterministic` once—final value independent of chunking order (caller must supply associative `combine`).

* **Map canonicalization**

  * Prefer `BTreeMap` over `HashMap` wherever iteration order is serialized/hashed.
  * Provide `btreemap_from_iter_kv` as a one-stop way to materialize a sorted map.

* **No globals**

  * No TLS, no OS entropy, no time calls. All functions are pure w\.r.t. input values.

8. State Flow
   Parallel stages → local partials → `sort_*_canonical` + `reduce_deterministic` → downstream serialization/hashing (outside this module).

9. Determinism & Numeric Rules

* Determinism from total orders + order-independent reduction.
* No numeric comparisons/rounding here (lives in rounding.rs).

10. Edge Cases & Failure Policy

* `reduce_deterministic(Vec::new())` → `None`.
* If caller’s `combine` isn’t associative, results may differ by chunking—caller responsibility (engine paths must use associative reducers).
* Sorting helpers must be in-place and avoid unnecessary allocations.

11. Test Checklist

* Sorting:

  * Units: sort by `UnitId` lexicographically; stable across OS/arch.
  * Options: sort by `(order_index, option_id)`; ties broken by `option_id`.
* Reduction:

  * Partition input randomly; reduce each partition; merge with `reduce_deterministic` ⇒ equals single-pass fold over the fully sorted list.
  * Identity law holds: `combine(x, T::identity()) == x`.
* Map canonicalization:

  * Iteration over `btreemap_from_iter_kv` keys is strictly sorted; byte serialization order is identical across runs.

```


```
