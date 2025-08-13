
````
Pre-Coding Essentials (Component: crates/vm_core/src/rounding.rs, Version/FormulaID: VM-ENGINE v0) — 27/89

1) Goal & Success
Goal: Overflow-safe integer/rational helpers for comparisons and rounding, with round-half-to-even where permitted.
Success: No floats; comparisons are overflow-safe; exact halves resolve with banker’s rounding; helpers cover gate checks and 1-decimal percentage rendering for reports.

2) Scope
In scope: ratio normalization, overflow-safe compare, half-even rounding to integer and to one-decimal percent, threshold comparisons.
Out of scope: seat allocation math (in vm_algo), serialization (in vm_io).

3) Inputs → Outputs
Inputs: integer pairs (num, den), with den > 0.
Outputs: orderings, booleans (threshold checks), rounded integers/decimals (for report layer).

4) Entities/Types (minimal)
- `NumericError` — error type for zero denominators / impossible states.
- `Ratio` (defined in vm_core::rounding or reused from core): `{ num: i128, den: i128 }` with invariant `den > 0`.

5) Variables
- None; pure functions only. Constants for `100`, `1000` to avoid magic numbers.

6) Functions (signatures only; public API surface)
```rust
/// Errors for numeric helpers (no I/O, deterministic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericError { ZeroDenominator }

/// Normalize sign and reduce by gcd; ensures den > 0.
/// Returns (num, den) with gcd(|num|, den) == 1 and den > 0.
pub fn simplify(num: i128, den: i128) -> Result<(i128, i128), NumericError>;

/// Overflow-safe compare of a/b vs c/d (total order).
/// Uses cross-cancel; if risk of overflow remains, falls back to Euclid/CF.
pub fn cmp_ratio(a_num: i128, a_den: i128, b_num: i128, b_den: i128)
    -> Result<core::cmp::Ordering, NumericError>;

/// Compare a/b against integer percent p (0..=100) without floats.
/// Returns true iff a/b >= p%.
pub fn ge_percent(a_num: i128, a_den: i128, p: u8) -> Result<bool, NumericError>;

/// Banker's rounding of a/b to nearest integer (ties to even).
pub fn round_nearest_even_int(num: i128, den: i128) -> Result<i128, NumericError>;

/// Banker's rounding of (a/b)*100 to **one decimal place**.
/// Returns tenths of a percent as an integer in 0..=1000 (e.g., 33.3% → 333).
pub fn percent_one_decimal_tenths(num: i128, den: i128) -> Result<i32, NumericError>;

/// Threshold with half-even at the boundary:
/// returns true iff a/b >= p% using integer rounding with banker's rule at the exact half.
pub fn ge_percent_half_even(a_num: i128, a_den: i128, p: u8) -> Result<bool, NumericError>;
````

7. Algorithm Outline (implementation plan)

* **simplify(num, den)**

  * If `den == 0` ⇒ `Err(ZeroDenominator)`.
  * Move sign to numerator: if `den < 0` then `num = -num; den = -den`.
  * Compute `g = gcd(|num|, den)` (binary GCD); return `(num/g, den/g)`.

* **cmp\_ratio(a/b ? c/d)** (overflow-safe)

  * Early zero/sign handling; normalize both via `simplify()`.
  * Cross-cancel: `a' = a/g1`, `c' = c/g1` with `g1 = gcd(|a|, |c|)`; `b' = b/g2`, `d' = d/g2` with `g2 = gcd(b, d)`.
  * Try `checked_mul(a', d')` vs `checked_mul(c', b')`; if any multiply would overflow, switch to Euclid/continued-fraction compare (division + remainder loop) until decision.

* **ge\_percent(a/b, p%)**

  * Normalize `(a, b)` with `simplify`.
  * Compare `a/b ? p/100` via cross-cancel:

    * `g1 = gcd(|a|, 100)`, `g2 = gcd(b, p as i128)`.
    * Compare `(a / g1) * (100 / g1') ? (p / g2) * (b / g2)`, using `checked_mul` with small factors; fall back to Euclid method if needed.
  * Short-circuit if `a == 0` or `p == 0`.

* **round\_nearest\_even\_int(num/den)**

  * Normalize with `simplify`.
  * `q = num / den`, `r = num % den` (with `den > 0`).
  * If `2*|r| < den` ⇒ `q`.
  * If `2*|r| > den` ⇒ `q + sign(num)`.
  * Else (exact half): return even of `{q, q + sign(num)}` (i.e., if `q` is odd, step toward `sign(num)`).

* **percent\_one\_decimal\_tenths(num/den)**

  * Compute round-half-even of `(num * 1000) / den` as an integer 0..=1000.
  * Avoid overflow: reduce `(num, den)`; prefer splitting by small constants (e.g., multiply by `125` and left-shift by `3` equals `*1000`) with `checked_mul`; if still risky, do long division with remainders and apply half-even on the final step.

* **ge\_percent\_half\_even(a/b, p%)**

  * Compare to integer percent boundary using the same half-even rule as `round_nearest_even_int`.
  * Algorithm: compute the **nearest-even integer percent** `x = round_nearest_even_int((a*100)/b)` and return `x >= p`.

8. State Flow

* Algorithms/gates call `cmp_ratio`, `ge_percent`, or `ge_percent_half_even` per spec.
* Report layer uses `percent_one_decimal_tenths` to render one-decimal percentages (no extra rounding elsewhere).

9. Determinism & Numeric Rules

* Pure integer math; outcomes identical across platforms.
* Half-even only at sanctioned comparison points; otherwise rely on exact rational ordering.
* Denominators normalized to `> 0`.

10. Edge Cases & Failure Policy

* `den == 0` ⇒ `Err(NumericError::ZeroDenominator)` in all APIs.
* Extremely large operands that risk overflow during multiply ⇒ deterministic fallback to Euclid/CF path.
* Negative numerators are handled (sign normalized); counts in practice are non-negative.

11. Test Checklist (must pass)

* **Compare without overflow**

  * `cmp_ratio(1,3, 333333333333333333, 999999999999999999) = Equal`.
  * Property tests vs a big-int reference on moderate domains (dev-only).
* **Half-even integer rounding**

  * `round_nearest_even_int(5,2) == 2`  (2.5 → 2)
  * `round_nearest_even_int(3,2) == 2`  (1.5 → 2)
  * `round_nearest_even_int(7,2) == 4`  (3.5 → 4)
* **Percent threshold**

  * `ge_percent(55,100,55) == true`
  * `ge_percent(549,1000,55) == false`
  * Half boundary via `ge_percent_half_even` follows banker’s rule.
* **One-decimal percent**

  * `(1,3) → 333`; `(2,3) → 667`; `(1,8) → 125` (12.5% stays 12.5 with half-even handling).
* Determinism: repeated runs produce identical outputs for all helpers.

```

```
