//! Overflow-safe integer/rational helpers with banker’s rounding (half-to-even).
//!
//! - Pure integer math; no floats, no I/O.
//! - Deterministic across OS/arch.
//! - Rounding only where explicitly allowed (nearest-even).
//!
//! Public API (as specified):
//!   - `simplify`
//!   - `cmp_ratio`
//!   - `ge_percent`
//!   - `round_nearest_even_int`
//!   - `percent_one_decimal_tenths`
//!   - `ge_percent_half_even`
//!
//! Extra small conveniences (aligned with `lib.rs` outline):
//!   - `Ratio` newtype + `new_ratio_checked`
//!   - `compare_ratio_half_even` (delegates to exact compare)

#![allow(clippy::many_single_char_names)]

use core::cmp::Ordering;

/* --------------------------------- Errors --------------------------------- */

/// Errors for numeric helpers (no I/O, deterministic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericError {
    ZeroDenominator,
}

/* --------------------------------- Ratio ---------------------------------- */

/// Exact rational with invariant `den > 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ratio {
    pub num: i128,
    pub den: i128,
}

impl Ratio {
    /// Construct and normalize sign/reduce by gcd; ensures `den > 0` and gcd(|num|, den) == 1.
    pub fn new_ratio_checked(num: i128, den: i128) -> Result<Self, NumericError> {
        let (n, d) = simplify(num, den)?;
        Ok(Self { num: n, den: d })
    }
}

/* --------------------------------- GCD ------------------------------------ */

#[inline]
fn abs_i128_to_u128(x: i128) -> u128 {
    if x == i128::MIN {
        (1u128) << 127
    } else if x < 0 {
        (-x) as u128
    } else {
        x as u128
    }
}

#[inline]
fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/* --------------------------- Core helper functions ------------------------- */

/// Normalize sign and reduce by gcd; ensures den > 0.
/// Returns (num, den) with gcd(|num|, den) == 1 and den > 0.
pub fn simplify(num: i128, den: i128) -> Result<(i128, i128), NumericError> {
    if den == 0 {
        return Err(NumericError::ZeroDenominator);
    }
    let mut n = num;
    let mut d = den;
    if d < 0 {
        n = -n;
        d = -d;
    }
    let g = gcd_u128(abs_i128_to_u128(n), d as u128) as i128;
    let n2 = n / g;
    let d2 = d / g;
    Ok((n2, d2))
}

/// Overflow-safe compare of a/b vs c/d (total order).
/// Uses cross-cancel; if risk of overflow remains, falls back to Euclid/CF.
pub fn cmp_ratio(a_num: i128, a_den: i128, b_num: i128, b_den: i128) -> Result<Ordering, NumericError> {
    // Normalize
    let (mut an, ad) = simplify(a_num, a_den)?;
    let (mut bn, bd) = simplify(b_num, b_den)?;

    // Fast sign checks
    let a_neg = an < 0;
    let b_neg = bn < 0;
    if a_neg != b_neg {
        return Ok(if a_neg { Ordering::Less } else { Ordering::Greater });
    }
    // Both negative: compare their absolutes reversed
    if a_neg && b_neg {
        an = -an;
        bn = -bn;
        return cmp_ratio_nonneg(an, ad, bn, bd).map(|o| o.reverse());
    }
    // Both non-negative
    cmp_ratio_nonneg(an, ad, bn, bd)
}

/// Compare non-negative rationals (an/ad) vs (bn/bd) with overflow safety.
fn cmp_ratio_nonneg(an: i128, ad: i128, bn: i128, bd: i128) -> Result<Ordering, NumericError> {
    // Quick equality path
    if an == bn && ad == bd {
        return Ok(Ordering::Equal);
    }

    // Cross-cancel to shrink operands
    let g1 = gcd_u128(abs_i128_to_u128(an), abs_i128_to_u128(bn)) as i128;
    let (an, bn) = (an / g1, bn / g1);
    let g2 = gcd_u128(ad as u128, bd as u128) as i128;
    let (ad, bd) = (ad / g2, bd / g2);

    // Try cross-multiplication
    if let (Some(lhs), Some(rhs)) = (an.checked_mul(bd), bn.checked_mul(ad)) {
        return Ok(lhs.cmp(&rhs));
    }

    // Fallback: continued-fraction / Euclid-based compare
    Ok(cmp_by_cf_nonneg(an, ad, bn, bd))
}

/// Continued-fraction style comparison for non-negative rationals, no overflow.
fn cmp_by_cf_nonneg(mut a: i128, mut b: i128, mut c: i128, mut d: i128) -> Ordering {
    debug_assert!(a >= 0 && b > 0 && c >= 0 && d > 0);
    loop {
        let qa = a / b;
        let ra = a % b;
        let qc = c / d;
        let rc = c % d;

        if qa != qc {
            return qa.cmp(&qc);
        }
        // Exactness checks
        if ra == 0 && rc == 0 {
            return Ordering::Equal;
        }
        if ra == 0 {
            // a/b == qa; c/d > qc since rc>0
            return Ordering::Less;
        }
        if rc == 0 {
            return Ordering::Greater;
        }
        // Compare reciprocals: a/b ? c/d  <=>  b/ra ? d/rc, but note the inversion.
        // Swap to next CF step.
        a = b;
        b = ra;
        c = d;
        d = rc;
    }
}

/// Compare a/b against integer percent p (0..=100) without floats.
/// Returns true iff a/b >= p%.
pub fn ge_percent(a_num: i128, a_den: i128, p: u8) -> Result<bool, NumericError> {
    let ord = cmp_ratio(a_num, a_den, p as i128, 100)?;
    Ok(matches!(ord, Ordering::Greater | Ordering::Equal))
}

/// Banker's rounding of a/b to nearest integer (ties to even).
pub
