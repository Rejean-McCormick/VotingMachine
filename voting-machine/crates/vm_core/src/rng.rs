// crates/vm_core/src/rng.rs — Part 1/2 (patched)
//
// Deterministic, integer-only RNG utilities for tie-breaking (VM-VAR-050/052).
// Focus: unbiased range generation, stable seeding, word-index crumbs.
//
// Spec anchors (Docs 1–7 + Annexes A–C):
// • VM-VAR-050 tie_policy governs how ties are resolved (deterministic_order | random | status_quo).
// • VM-VAR-052 tie_seed is the only source of randomness for ties; it is EXCLUDED from FID
//   but must be logged in RunRecord integrity. We track the index of the RNG word used.
// • Integer-only RNG: no floating point. Unbiased ranges via rejection sampling.
// • Cross-platform determinism: explicit seeding and word-index accounting.
//
// This file is split in two halves. Part 1 provides types, constructors, and core
// integer RNG operations (next_u64, gen_range, pick+crumb). Part 2 adds helpers
// like shuffle/choose and (optionally) serde/tests.

use smol_str::SmolStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

/// A single logged decision for a tie, including context and the RNG word index.
///
/// `word_index` is **1-based**: the first 64-bit RNG word consumed by this
/// `TieRng` has index 1; the second has index 2; etc. For range generation
/// using rejection sampling, `word_index` refers to the **accepted** RNG word
/// that decided the pick (rejected draws are counted but not logged here).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TieCrumb {
    /// Stable, human-readable context (e.g., "unit:U42/last-seat").
    pub ctx: SmolStr,
    /// Chosen index in the contender set (0-based). `usize` avoids truncation.
    pub pick: usize,
    /// 1-based index of the deciding RNG 64-bit word (saturates at u128::MAX).
    pub word_index: u128,
}

/// Deterministic RNG for ties, seeded only from VM-VAR-052.
///
/// Internally uses ChaCha20 with an explicit 32-byte seed derived from the
/// 64-bit tie seed (little-endian bytes in the first 8 positions; the rest 0).
/// This avoids endianness ambiguity and keeps mapping stable across platforms.
/// (Pinning crate versions at Cargo level ensures stream stability across builds.)
#[derive(Debug, Clone)]
pub struct TieRng {
    rng: ChaCha20Rng,
    words_consumed: u128,
}

impl TieRng {
    /// Construct from a 64-bit VM-VAR-052 seed. The mapping from `u64` to the
    /// ChaCha20 32-byte seed is explicit: `seed.to_le_bytes()` into the first
    /// 8 bytes; the remaining 24 bytes are zero.
    #[inline]
    pub fn from_seed_u64(seed: u64) -> Self {
        let mut seed32 = [0u8; 32];
        seed32[..8].copy_from_slice(&seed.to_le_bytes());
        let rng = ChaCha20Rng::from_seed(seed32);
        Self {
            rng,
            words_consumed: 0,
        }
    }

    /// Total number of 64-bit words consumed so far (saturating at `u128::MAX`).
    /// This is a **draw counter**, not a byte counter.
    #[inline]
    pub fn words_consumed(&self) -> u128 {
        self.words_consumed
    }

    /// Draw the next u64 from the stream and increment the word counter.
    /// This is the only place where the counter is advanced.
    #[inline]
    fn next_u64(&mut self) -> u64 {
        // Saturating add so extremely long runs don't panic.
        self.words_consumed = self.words_consumed.saturating_add(1);
        self.rng.next_u64()
    }

    /// Unbiased integer in [0, n) using rejection sampling with the standard
    /// PCG "threshold" trick. Returns `None` if `n == 0`.
    ///
    /// Let `threshold = 2^64 mod n` (computed via `wrapping_neg() % n`).
    /// Accept `x` if `x >= threshold`; then `x % n` is uniformly distributed.
    #[inline]
    pub fn gen_range(&mut self, n: u64) -> Option<u64> {
        self.gen_range_with_index(n).map(|(v, _idx)| v)
    }

    /// Same as `gen_range`, but also returns the **1-based** index of the
    /// deciding RNG word. Useful for logging deterministic crumbs.
    #[inline]
    pub fn gen_range_with_index(&mut self, n: u64) -> Option<(u64, u128)> {
        if n == 0 {
            return None;
        }
        let threshold = n.wrapping_neg() % n; // == (2^64 % n)
        loop {
            let x = self.next_u64();          // increments words_consumed
            if x >= threshold {
                // words_consumed now points at the accepted word → 1-based
                return Some((x % n, self.words_consumed));
            }
        }
    }

    /// Atomically pick an index in `[0, n)` and return an attached `TieCrumb`
    /// whose `word_index` refers to the deciding RNG word for this pick.
    /// Returns `None` if `n == 0`.
    #[inline]
    pub fn pick_index_with_crumb(&mut self, ctx: &str, n: u64) -> Option<(usize, TieCrumb)> {
        let (v, word_index) = self.gen_range_with_index(n)?;
        let idx = v as usize;
        let crumb = TieCrumb {
            ctx: SmolStr::new(ctx),
            pick: idx,
            word_index,
        };
        Some((idx, crumb))
    }
}

// Part 2/2 will add:
// - shuffle_in_place<T>(…)
// - choose_* helpers (by index / by slice)
// - optional serde helpers/tests
// crates/vm_core/src/rng.rs — Part 2/2 (patched)
//
// Utilities built on top of the core RNG from Part 1:
// - Deterministic Fisher–Yates shuffle
// - Choice helpers over counts and slices
// - Convenience crumb-producing variants
//
// All functions are integer-only and preserve determinism across platforms.

impl TieRng {
    /// Deterministic in-place Fisher–Yates shuffle.
    ///
    /// Uses the unbiased scheme:
    /// for i in (1..len).rev() { j ~ U{0..i}; swap(i, j) }
    #[inline]
    pub fn shuffle_in_place<T>(&mut self, slice: &mut [T]) {
        let len = slice.len();
        if len <= 1 {
            return;
        }
        // Walk i = len-1 down to 1
        let mut i = len - 1;
        loop {
            // gen_range(i+1) is guaranteed non-empty here
            let j = match self.gen_range((i as u64) + 1) {
                Some(v) => v as usize,
                None => unreachable!("gen_range(>0) must return Some"),
            };
            slice.swap(i, j);
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    /// Choose a single index in `[0, n)`; returns `None` if `n == 0`.
    #[inline]
    pub fn choose_index(&mut self, n: usize) -> Option<usize> {
        self.gen_range(n as u64).map(|v| v as usize)
    }

    /// Choose a single index in `[0, n)`, returning a `TieCrumb` bound
    /// to the deciding RNG word. Returns `None` if `n == 0`.
    #[inline]
    pub fn choose_index_with_crumb(&mut self, ctx: &str, n: usize) -> Option<(usize, TieCrumb)> {
        self.pick_index_with_crumb(ctx, n as u64)
    }

    /// Choose one element from a non-empty slice, returning its index.
    /// Returns `None` if the slice is empty.
    #[inline]
    pub fn choose_one_index<T>(&mut self, slice: &[T]) -> Option<usize> {
        self.choose_index(slice.len())
    }

    /// Choose one element from a non-empty slice and return `(index, crumb)`.
    /// Returns `None` if the slice is empty.
    #[inline]
    pub fn choose_one_index_with_crumb<T>(
        &mut self,
        ctx: &str,
        slice: &[T],
    ) -> Option<(usize, TieCrumb)> {
        self.choose_index_with_crumb(ctx, slice.len())
    }
}

// ------------------------------
// Tests (determinism & basics)
// ------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_range_zero_none() {
        let mut rng = TieRng::from_seed_u64(0xDEADBEEFCAFEBABE);
        assert_eq!(rng.gen_range(0), None);
        assert_eq!(rng.words_consumed(), 0);
    }

    #[test]
    fn gen_range_threshold_deterministic() {
        // Determinism check across calls; we don't assert distribution here.
        let mut a = TieRng::from_seed_u64(123456789);
        let mut b = TieRng::from_seed_u64(123456789);
        let mut seq_a = [0u64; 16];
        let mut seq_b = [0u64; 16];
        for i in 0..16 {
            seq_a[i] = a.gen_range(10).unwrap();
            seq_b[i] = b.gen_range(10).unwrap();
        }
        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn pick_with_crumb_monotonic_index() {
        let mut rng = TieRng::from_seed_u64(0x0123_4567_89AB_CDEF);
        let (_, c1) = rng.pick_index_with_crumb("ctx/first", 5).unwrap();
        let (_, c2) = rng.pick_index_with_crumb("ctx/second", 5).unwrap();
        assert!(c1.word_index >= 1);
        assert!(c2.word_index > c1.word_index);
        // Counter reflects total accepted draws
        assert!(rng.words_consumed() >= c2.word_index);
    }

    #[test]
    fn shuffle_is_deterministic() {
        let seed = 42u64;
        let mut a = TieRng::from_seed_u64(seed);
        let mut b = TieRng::from_seed_u64(seed);
        let mut xs = (0..16).collect::<Vec<_>>();
        let mut ys = (0..16).collect::<Vec<_>>();

        a.shuffle_in_place(&mut xs);
        b.shuffle_in_place(&mut ys);
        assert_eq!(xs, ys);
    }

    #[test]
    fn choose_one_index_matches_len() {
        let mut rng = TieRng::from_seed_u64(7);
        let data: [u8; 0] = [];
        assert!(rng.choose_one_index(&data).is_none());

        let data = [10, 20, 30];
        for _ in 0..10 {
            let ix = rng.choose_one_index(&data).unwrap();
            assert!(ix < data.len());
        }
    }
}
