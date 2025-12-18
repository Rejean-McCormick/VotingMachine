//! vm_core — Core types, domains, ordering helpers, and deterministic RNG.
//!
//! This crate is **I/O-free**. It defines stable types/APIs used across the
//! engine (`vm_io`, `vm_algo`, `vm_pipeline`, `vm_report`, `vm_cli`).
//!
//! - Output IDs: `RES:`, `RUN:`, `FR:`
//! - Registry tokens: `UnitId`, `OptionId`
//! - VM-VAR domains: `TiePolicy` (050), `AlgorithmVariant` (073), `Params`
//! - Deterministic ordering helpers
//! - Integer-first numerics & ratio helpers
//! - Seedable RNG (ChaCha20) for **ties only**
//!
//! Serialization derives are gated behind `serde` feature.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod errors {
    use core::fmt;

    /// Minimal error set for core-domain validation & parsing.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum CoreError {
        InvalidId,
        InvalidToken,
        InvalidHex,
        InvalidTimestamp,
        InvalidRatio,
        DomainOutOfRange(&'static str),
        EmptyChoiceSet,
    }

    impl fmt::Display for CoreError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                CoreError::InvalidId => write!(f, "invalid id"),
                CoreError::InvalidToken => write!(f, "invalid token"),
                CoreError::InvalidHex => write!(f, "invalid hex"),
                CoreError::InvalidTimestamp => write!(f, "invalid timestamp"),
                CoreError::InvalidRatio => write!(f, "invalid ratio"),
                CoreError::DomainOutOfRange(k) => write!(f, "domain out of range: {k}"),
                CoreError::EmptyChoiceSet => write!(f, "empty choice set"),
            }
        }
    }
}

pub mod ids {
    //! Newtypes and parsers for output/digest identifiers.

    use crate::errors::CoreError;
    use alloc::string::{String, ToString};
    use core::fmt;
    use core::str::FromStr;

    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    fn is_lower_hex(s: &str) -> bool {
        s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    }

    fn is_lower_hex_len(s: &str, n: usize) -> bool {
        s.len() == n && is_lower_hex(s)
    }

    fn is_ts_utc_z(s: &str) -> bool {
        // Very strict RFC3339-like check: "YYYY-MM-DDTHH:MM:SSZ" (length 20)
        let b = s.as_bytes();
        if b.len() != 20 { return false; }
        matches!(b[4], b'-')
            && matches!(b[7], b'-')
            && matches!(b[10], b'T')
            && matches!(b[13], b':')
            && matches!(b[16], b':')
            && matches!(b[19], b'Z')
            && b.iter().enumerate().all(|(i, c)| match i {
                0..=3 | 5..=6 | 8..=9 | 11..=12 | 14..=15 | 17..=18 => matches!(c, b'0'..=b'9'),
                4 | 7 | 10 | 13 | 16 | 19 => true,
                _ => false,
            })
    }

    /// 64-hex lowercase (digest/fingerprint).
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct Sha256(String);

    impl Sha256 {
        pub fn as_str(&self) -> &str { &self.0 }
    }

    impl fmt::Display for Sha256 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl FromStr for Sha256 {
        type Err = CoreError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if is_lower_hex_len(s, 64) { Ok(Self(s.to_string())) } else { Err(CoreError::InvalidHex) }
        }
    }

    /// FormulaId = 64-hex fingerprint of normative manifest/rules.
    pub type FormulaId = Sha256;

    /// "RES:" + 64-hex (lowercase)
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct ResultId(String);

    impl ResultId {
        pub fn as_str(&self) -> &str { &self.0 }
    }

    impl fmt::Display for ResultId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl FromStr for ResultId {
        type Err = CoreError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let rest = s.strip_prefix("RES:").ok_or(CoreError::InvalidId)?;
            if is_lower_hex_len(rest, 64) { Ok(Self(s.to_string())) } else { Err(CoreError::InvalidId) }
        }
    }

    /// "FR:" + 64-hex (lowercase)
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct FrontierMapId(String);

    impl FrontierMapId {
        pub fn as_str(&self) -> &str { &self.0 }
    }

    impl fmt::Display for FrontierMapId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl FromStr for FrontierMapId {
        type Err = CoreError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let rest = s.strip_prefix("FR:").ok_or(CoreError::InvalidId)?;
            if is_lower_hex_len(rest, 64) { Ok(Self(s.to_string())) } else { Err(CoreError::InvalidId) }
        }
    }

    /// "RUN:" + "<YYYY-MM-DDTHH:MM:SSZ>" + "-" + "<8..64-hex lowercase>"
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct RunId(String);

    impl RunId {
        pub fn as_str(&self) -> &str { &self.0 }
    }

    impl fmt::Display for RunId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0)
        }
    }

    impl FromStr for RunId {
        type Err = CoreError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let rest = s.strip_prefix("RUN:").ok_or(CoreError::InvalidId)?;
            // Split at the dash between timestamp and hash
            let (ts, hash) = rest.split_once('-').ok_or(CoreError::InvalidId)?;
            if !is_ts_utc_z(ts) { return Err(CoreError::InvalidTimestamp); }
            if !(8..=64).contains(&hash.len()) || !is_lower_hex(hash) {
                return Err(CoreError::InvalidId);
            }
            Ok(Self(s.to_string()))
        }
    }
}

pub mod tokens {
    //! Registry token types (`UnitId`, `OptionId`) with strict charset.

    use crate::errors::CoreError;
    use alloc::string::{String, ToString};
    use core::fmt;
    use core::str::FromStr;

    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    fn is_token(s: &str) -> bool {
        let len = s.len();
        if !(1..=64).contains(&len) { return false; }
        s.bytes().all(|b| matches!(b,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' |
            b'_' | b'-' | b':' | b'.'
        ))
    }

    macro_rules! def_token {
        ($name:ident) => {
            #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            pub struct $name(String);

            impl $name {
                pub fn as_str(&self) -> &str { &self.0 }
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
            }

            impl FromStr for $name {
                type Err = CoreError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    if is_token(s) { Ok(Self(s.to_string())) } else { Err(CoreError::InvalidToken) }
                }
            }
        }
    }

    def_token!(UnitId);
    def_token!(OptionId);
}

pub mod determinism {
    //! Stable ordering helpers.

    use core::cmp::Ordering;
    use crate::tokens::OptionId;

    /// Types participating in stable selections can expose a total order key.
    pub trait StableOrd {
        type Key: Ord;
        fn stable_key(&self) -> Self::Key;
    }

    /// Minimal registry option metadata used for deterministic ordering.
    #[derive(Clone, Debug)]
    pub struct RegOptionMeta<'a> {
        pub order_index: u32,
        pub option_id: &'a OptionId,
    }

    /// Compare by `order_index`, then by `option_id` lexicographically.
    pub fn cmp_options_by_order(a: &RegOptionMeta<'_>, b: &RegOptionMeta<'_>) -> Ordering {
        match a.order_index.cmp(&b.order_index) {
            Ordering::Equal => a.option_id.as_str().cmp(b.option_id.as_str()),
            o => o,
        }
    }

    /// Sort unit ids ascending (lexicographic).
    pub fn sort_units_by_id(ids: &mut [crate::tokens::UnitId]) {
        ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    }
}

pub mod rounding {
    //! Integer-first ratio type and helpers.

    use crate::errors::CoreError;
    use core::cmp::Ordering;

    /// Exact ratio with normalized sign and positive denominator.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct Ratio {
        pub num: i128,
        pub den: i128,
    }

    #[inline]
    fn abs_i128(x: i128) -> i128 { if x < 0 { -x } else { x } }

    fn gcd_i128(mut a: i128, mut b: i128) -> i128 {
        a = abs_i128(a);
        b = abs_i128(b);
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        if a == 0 { 1 } else { a }
    }

    /// Construct a ratio, ensuring `den > 0` and reducing by GCD.
    pub fn new_ratio_checked(num: i128, den: i128) -> Result<Ratio, CoreError> {
        if den == 0 { return Err(CoreError::InvalidRatio); }
        let (mut n, mut d) = (num, den);
        if d < 0 {
            n = -n;
            d = -d;
        }
        let g = gcd_i128(n, d);
        Ok(Ratio { num: n / g, den: d / g })
    }

    /// Compare two ratios exactly (cross-multiply) with a tie returning `Equal`.
    ///
    /// NOTE: Uses checked multiplication; in the unlikely event of overflow,
    /// falls back to `f64` comparison (deterministic but lossy).
    pub fn compare_ratio_half_even(a: &Ratio, b: &Ratio) -> Ordering {
        // Reduce before cross-multiply to reduce overflow chance.
        let g1 = gcd_i128(a.num, b.num);
        let g2 = gcd_i128(a.den, b.den);
        let an = a.num / g1;
        let bn = b.num / g1;
        let ad = a.den / g2;
        let bd = b.den / g2;

        if let (Some(l), Some(r)) = (an.checked_mul(bd), bn.checked_mul(ad)) {
            l.cmp(&r)
        } else {
            // Fallback: compare as f64 (deterministic IEEE-754). Only for extreme values.
            let af = (a.num as f64) / (a.den as f64);
            let bf = (b.num as f64) / (b.den as f64);
            af.partial_cmp(&bf).unwrap_or(Ordering::Equal)
        }
    }
}

pub mod rng {
    //! Seeded RNG for **ties only** (no OS entropy).

    use rand_chacha::ChaCha20Rng;
    use rand_core::{RngCore, SeedableRng};

    use crate::determinism::StableOrd;
    use crate::errors::CoreError;

    /// Newtype over ChaCha20Rng for tie-breaking.
    pub struct TieRng(ChaCha20Rng);

    /// Create a tie RNG from an integer seed (VM-VAR-052).
    pub fn tie_rng_from_seed(seed: u64) -> TieRng {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&seed.to_le_bytes());
        TieRng(ChaCha20Rng::from_seed(bytes))
    }

    impl TieRng {
        /// Choose an index from a non-empty slice uniformly using rejection sampling.
        /// Returns `None` on empty slice.
        pub fn choose<T: StableOrd>(&mut self, slice: &[T]) -> Option<usize> {
            let n = slice.len();
            if n == 0 { return None; }
            let n_u64 = n as u64;
            // Rejection sampling to avoid modulo bias.
            let zone = u64::MAX - (u64::MAX % n_u64);
            loop {
                let x = self.0.next_u64();
                if x < zone {
                    return Some((x % n_u64) as usize);
                }
            }
        }

        /// Expose underlying RNG for controlled uses (e.g., shuffle within tie set).
        pub fn rng_mut(&mut self) -> &mut ChaCha20Rng { &mut self.0 }
    }

    impl Default for TieRng {
        fn default() -> Self { tie_rng_from_seed(0) }
    }
}

pub mod variables {
    //! VM-VAR domains and a minimal `Params` struct (non-exhaustive).
    //!
    //! Notes:
    //! - `tie_policy` (050) is **Included** in FID.
    //! - `tie_seed`   (052) is **Excluded** from FID and only relevant if
    //!   `tie_policy == Random`. Recording of the seed happens in RunRecord
    //!   **only if** a random tie actually occurred.
    //! - `
