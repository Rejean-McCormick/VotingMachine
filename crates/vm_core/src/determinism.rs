//! Determinism utilities: stable ordering & order-independent reduction.
//!
//! This module is **I/O-free**. It provides:
//! - Stable total orders for core tokens/entities
//! - Canonical in-place sort helpers
//! - A deterministic reduce helper (independent of chunking order if the
//!   combiner is associative)
//! - A small map helper to materialize canonical (key-ordered) maps
//! - A trait for “canonical bytes” (interface only; implemented elsewhere)

extern crate alloc;

use core::cmp::Ordering;

use crate::entities::OptionItem;
use crate::tokens::{OptionId, UnitId};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/* -------------------------------------------------------------------------- */
/*                               Stable Ordering                              */
/* -------------------------------------------------------------------------- */

/// Provide a **total**, stable order for values that must sort canonically.
pub trait StableOrd {
    fn stable_cmp(&self, other: &Self) -> Ordering;
}

impl StableOrd for UnitId {
    #[inline]
    fn stable_cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl StableOrd for OptionId {
    #[inline]
    fn stable_cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl StableOrd for OptionItem {
    /// Canonical option order is **always** `(order_index, option_id)`.
    #[inline]
    fn stable_cmp(&self, other: &Self) -> Ordering {
        match self.order_index.cmp(&other.order_index) {
            Ordering::Equal => self.option_id.as_str().cmp(other.option_id.as_str()),
            o => o,
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                            Canonical sort helpers                           */
/* -------------------------------------------------------------------------- */

/// Compare two options by `(order_index, option_id)`.
#[inline]
pub fn cmp_options_by_order(a: &OptionItem, b: &OptionItem) -> Ordering {
    a.stable_cmp(b)
}

/// Sort options **in place** into canonical order.
#[inline]
pub fn sort_options_canonical(xs: &mut [OptionItem]) {
    xs.sort_by(|a, b| a.stable_cmp(b));
}

/// Sort units **in place** by ascending `UnitId` (lexicographic).
#[inline]
pub fn sort_units_by_id<T: AsRef<UnitId>>(xs: &mut [T]) {
    xs.sort_by(|a, b| a.as_ref().as_str().cmp(b.as_ref().as_str()));
}

/* -------------------------------------------------------------------------- */
/*                         Deterministic (order-free) reduce                   */
/* -------------------------------------------------------------------------- */

/// Marker trait providing an identity element for reductions.
///
/// The identity is not used for the non-empty case (we fold from the first
/// element). It exists to document the algebra and for potential callers that
/// wish to validate identities in tests.
pub trait StableReduce: Sized {
    fn identity() -> Self;
}

/// Deterministically reduce a vector by first sorting it canonically and then
/// folding with the provided associative `combine`.
///
/// If `items` is empty, returns `None`.
#[inline]
pub fn reduce_deterministic<T, F>(mut items: Vec<T>, combine: F) -> Option<T>
where
    T: StableReduce + StableOrd,
    F: Fn(T, T) -> T,
{
    if items.is_empty() {
        return None;
    }
    items.sort_by(|a, b| a.stable_cmp(b));
    let mut it = items.into_iter();
    let first = it.next().unwrap();
    Some(it.fold(first, |acc, x| combine(acc, x)))
}

/* -------------------------------------------------------------------------- */
/*                          Canonical map materialization                      */
/* -------------------------------------------------------------------------- */

/// Build a key-ordered `BTreeMap` from an iterator of `(K, V)`.
#[inline]
pub fn btreemap_from_iter_kv<K: Ord, V, I: IntoIterator<Item = (K, V)>>(it: I) -> BTreeMap<K, V> {
    it.into_iter().collect()
}

/* -------------------------------------------------------------------------- */
/*                         Canonical bytes (interface)                         */
/* -------------------------------------------------------------------------- */

/// Types that can emit **canonical bytes** suitable for hashing.
/// (Implementation lives in the codec layer; this is the interface.)
pub trait HashCanon {
    fn canonical_bytes(&self) -> Vec<u8>;
}

/* ---------------------------------- Tests --------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{OptionId, UnitId};

    fn oid(s: &str) -> OptionId { s.parse().unwrap() }
    fn uid(s: &str) -> UnitId { s.parse().unwrap() }

    #[test]
    fn unit_sort_is_lex() {
        let mut v = vec![uid("U-10"), uid("U-2"), uid("A-1")];
        sort_units_by_id(&mut v);
        let got: Vec<&str> = v.iter().map(|u| u.as_str()).collect();
        assert_eq!(got, vec!["A-1", "U-10", "U-2"]);
    }

    #[test]
    fn option_sort_by_index_then_id() {
        let mut xs = vec![
            OptionItem { option_id: oid("O-B"), name: "b".into(), order_index: 1 },
            OptionItem { option_id: oid("O-A"), name: "a".into(), order_index: 1 },
            OptionItem { option_id: oid("O-C"), name: "c".into(), order_index: 0 },
        ];
        sort_options_canonical(&mut xs);
        let got: Vec<(&str, u16)> = xs.iter().map(|o| (o.option_id.as_str(), o.order_index)).collect();
        assert_eq!(got, vec![("O-C", 0), ("O-A", 1), ("O-B", 1)]);
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Sum(u32);
    impl StableReduce for Sum {
        fn identity() -> Self { Sum(0) }
    }
    impl StableOrd for Sum {
        fn stable_cmp(&self, other: &Self) -> Ordering { self.0.cmp(&other.0) }
    }

    #[test]
    fn reduce_is_deterministic_sorted_fold() {
        let v = vec![Sum(5), Sum(1), Sum(3)];
        let r = reduce_deterministic(v, |a, b| Sum(a.0 + b.0)).unwrap();
        assert_eq!(r, Sum(9));
    }

    #[test]
    fn reduce_empty_none() {
        let v: Vec<Sum> = vec![];
        assert!(reduce_deterministic(v, |a, b| Sum(a.0 + b.0)).is_none());
    }

    #[test]
    fn btreemap_canonical_keys() {
        let m = btreemap_from_iter_kv([("b", 2), ("a", 1), ("c", 3)]);
        let keys: Vec<&str> = m.keys().copied().collect();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }
}
