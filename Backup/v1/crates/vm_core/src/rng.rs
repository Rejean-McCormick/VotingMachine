//! crates/vm_core/src/rng.rs
//! Deterministic RNG for tie resolution only (VM-VAR-052 seed).
//! ChaCha20-based, single-threaded, no OS entropy/time, no floats.

#![allow(clippy::result_large_err)]

use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use smol_str::SmolStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Construction/usage errors (kept for API symmetry; most fns return Option on empty domains).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RngError {
    EmptyDomain,
}

/// Opaque RNG used strictly for ties. Tracks how many 64-bit words were consumed.
#[derive(Clone)]
pub struct TieRng {
    rng: ChaCha20Rng,
    words_consumed: u128,
}

/// Optional audit crumb (callers may aggregate into RunRecord tie logs).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TieCrumb {
    pub ctx: SmolStr,
    pub pick: u32,
    pub word_index: u128,
}

/// Build from integer tie_seed (VM-VAR-052). Stable across platforms.
#[inline]
pub fn tie_rng_from_seed(seed: u64) -> TieRng {
    TieRng {
        rng: ChaCha20Rng::seed_from_u64(seed),
        words_consumed: 0,
    }
}

impl TieRng {
    /// Next unbiased integer in [0, n) via rejection sampling. Returns None if n == 0.
    #[inline]
    pub fn gen_range(&mut self, n: u64) -> Option<u64> {
        if n == 0 {
            return None;
        }
        // Avoid modulo bias with rejection sampling.
        // zone is the largest multiple of n that fits in u64.
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return Some(x % n);
            }
        }
    }

    /// Choose index of winner from slice; None on empty slice.
    #[inline]
    pub fn choose_index<T>(&mut self, slice: &[T]) -> Option<usize> {
        let n = slice.len() as u64;
        self.gen_range(n).map(|v| v as usize)
    }

    /// Deterministic in-place Fisher–Yates shuffle.
    #[inline]
    pub fn shuffle<T>(&mut self, xs: &mut [T]) {
        let len = xs.len();
        if len <= 1 {
            return;
        }
        // Iterate i = len-1 down to 1, choose j ∈ [0, i]
        for i in (1..len).rev() {
            // unwrap is safe: i >= 1 ⇒ i+1 >= 2 ⇒ domain non-zero
            let j = self.gen_range((i as u64) + 1).unwrap() as usize;
            xs.swap(i, j);
        }
    }

    /// Emit next u64.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let v = self.rng.next_u64();
        self.words_consumed = self.words_consumed.saturating_add(1);
        v
    }

    /// Emit next u128 (concat of two u64 draws).
    #[inline]
    pub fn next_u128(&mut self) -> u128 {
        let hi = self.next_u64() as u128;
        let lo = self.next_u64() as u128;
        (hi << 64) | lo
    }

    /// Return how many 64-bit words have been consumed.
    #[inline]
    pub fn words_consumed(&self) -> u128 {
        self.words_consumed
    }

    /// Optional: build a tiny crumb for audit logs (does not consume RNG).
    #[inline]
    pub fn log_pick(&self, ctx: &str, pick: usize) -> TieCrumb {
        TieCrumb {
            ctx: SmolStr::new(ctx),
            pick: pick as u32,
            word_index: self.words_consumed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determinism_same_seed_same_sequence() {
        let mut a = tie_rng_from_seed(42);
        let mut b = tie_rng_from_seed(42);

        let mut va = [0u64; 8];
        let mut vb = [0u64; 8];
        for i in 0..8 {
            va[i] = a.next_u64();
            vb[i] = b.next_u64();
        }
        assert_eq!(va, vb);

        // gen_range determinism
        let mut a = tie_rng_from_seed(7);
        let mut b = tie_rng_from_seed(7);
        let sa: Vec<u64> = (0..20).map(|_| a.gen_range(10).unwrap()).collect();
        let sb: Vec<u64> = (0..20).map(|_| b.gen_range(10).unwrap()).collect();
        assert_eq!(sa, sb);
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = tie_rng_from_seed(1);
        let mut b = tie_rng_from_seed(2);

        // Very likely to diverge immediately
        let x = a.next_u64();
        let y = b.next_u64();
        assert_ne!(x, y);
    }

    #[test]
    fn choose_and_shuffle() {
        let mut r = tie_rng_from_seed(123);
        // choose_index
        let empty: [i32; 0] = [];
        assert!(r.choose_index(&empty).is_none());

        let v = [10, 20, 30];
        let idx = r.choose_index(&v).unwrap();
        assert!(idx < v.len());

        // shuffle determinism
        let mut a = (0..10).collect::<Vec<_>>();
        let mut b = (0..10).collect::<Vec<_>>();
        let mut r1 = tie_rng_from_seed(9);
        let mut r2 = tie_rng_from_seed(9);
        r1.shuffle(&mut a);
        r2.shuffle(&mut b);
        assert_eq!(a, b);

        // different seed ⇒ almost surely different permutation
        let mut c = (0..10).collect::<Vec<_>>();
        let mut r3 = tie_rng_from_seed(99);
        r3.shuffle(&mut c);
        assert_ne!(a, c);
    }

    #[test]
    fn words_and_u128() {
        let mut r = tie_rng_from_seed(5);
        assert_eq!(r.words_consumed(), 0);
        let _u = r.next_u64();
        assert_eq!(r.words_consumed(), 1);
        let _v = r.next_u128();
        assert_eq!(r.words_consumed(), 3);
    }

    #[test]
    fn crumb_does_not_consume_words() {
        let mut r = tie_rng_from_seed(77);
        let _ = r.next_u64();
        let before = r.words_consumed();
        let c = r.log_pick("tie#1", 2);
        assert_eq!(c.ctx.as_str(), "tie#1");
        assert_eq!(c.pick, 2);
        assert_eq!(c.word_index, before);
        assert_eq!(r.words_consumed(), before);
    }

    #[test]
    fn gen_range_zero_none() {
        let mut r = tie_rng_from_seed(0);
        assert_eq!(r.gen_range(0), None);
    }
}
