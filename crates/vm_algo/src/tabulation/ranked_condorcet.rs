//! Ranked Condorcet — Schulze method (deterministic, integers-only).
//!
//! Scope (per specs):
//! - Build pairwise win counts between options for a unit (no RNG, no floats).
//! - Compute strongest paths using Schulze (Floyd–Warshall style).
//! - Do not depend on map iteration order; always use canonical option order:
//!   (order_index, then OptionId) as provided by the `options` slice.
//!
//! Out of scope (wired by callers/pipeline):
//! - Reading ballots / constructing pairwise from ballots.
//! - Frontier, labels, presentation.
//!
//! Determinism:
//! - No network, no time; all loops iterate by index over the canonical
//!   `options` order; lookups are explicit by (OptionId, OptionId).

#![allow(clippy::needless_pass_by_value)]

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use vm_core::{
    entities::OptionItem,
    ids::OptionId,
};

/// Errors specific to Condorcet/Schulze handling.
#[derive(Debug)]
pub enum CondorcetError {
    /// Attempted to reference an option that is not present in the canonical list.
    UnknownOption(OptionId),
    /// Internal invariant was violated (e.g., self comparison).
    Invariant(&'static str),
}

/// Pairwise win matrix: wins[(A,B)] = number of ballots preferring A over B.
///
/// Notes:
/// - Keys are **owned** `(OptionId, OptionId)` to avoid lifetime pitfalls and
///   enable deterministic canonicalization downstream.
/// - Self pairs (A,A) are present and fixed at 0; callers should treat them as 0.
#[derive(Clone, Default, Debug)]
pub struct Pairwise {
    wins: BTreeMap<(OptionId, OptionId), u64>,
}

impl Pairwise {
    /// Initialize all pairs to 0 for the given canonical option list.
    pub fn new(options: &[OptionItem]) -> Self {
        let mut wins = BTreeMap::new();
        // Canonical order comes from `options`; we materialize owned ids.
        let seq: Vec<OptionId> = seq_ids(options);
        for i in 0..seq.len() {
            for j in 0..seq.len() {
                let a = seq[i].clone();
                let b = seq[j].clone();
                wins.insert((a, b), if i == j { 0 } else { 0 });
            }
        }
        Self { wins }
    }

    /// Increment wins for A over B by `delta` (A != B).
    pub fn increment(&mut self, a: &OptionId, b: &OptionId, delta: u64) -> Result<(), CondorcetError> {
        if a == b {
            return Err(CondorcetError::Invariant("increment on (A,A) is forbidden"));
        }
        let key = (a.clone(), b.clone());
        // Absent keys mean unknown options relative to initialization.
        match self.wins.get_mut(&key) {
            Some(slot) => {
                *slot = slot.saturating_add(delta);
                Ok(())
            }
            None => Err(CondorcetError::UnknownOption(a.clone())),
        }
    }

    /// Read wins for A over B (returns 0 if the pair is absent).
    #[inline]
    pub fn get(&self, a: &OptionId, b: &OptionId) -> u64 {
        self.wins.get(&(a.clone(), b.clone())).copied().unwrap_or(0)
    }

    /// Expose immutable map (e.g., to feed Schulze).
    pub fn as_map(&self) -> &BTreeMap<(OptionId, OptionId), u64> {
        &self.wins
    }
}

/// Produce an owned, canonical sequence of OptionIds from `options`.
/// Order is the on-wire canonical `(order_index, OptionId)` provided upstream.
#[inline]
pub fn seq_ids(options: &[OptionItem]) -> Vec<OptionId> {
    options.iter().map(|o| o.option_id.clone()).collect()
}

/// Compute the Schulze strongest paths matrix `P` from pairwise wins.
///
/// Definitions (standard Schulze):
/// - Let `d[A,B] = wins(A,B)` be pairwise preferences.
/// - Initialize:
///     P[A,B] = d[A,B] if d[A,B] > d[B,A], else 0 (for A != B); P[A,A] = 0.
/// - For each intermediate `K`, update:
///     P[A,B] = max(P[A,B], min(P[A,K], P[K,B])) for all A != B and A != K and B != K.
///
/// Determinism:
/// - Loops are strictly `for k in 0..n { for i in 0..n { for j in 0..n { ... }}}`,
///   which is the canonical order for Floyd–Warshall style updates.
/// - Indices are **not** transposed; we always update P[i][j] from (i,k) & (k,j).
pub fn schulze_strongest_paths(
    options: &[OptionItem],
    wins: &BTreeMap<(OptionId, OptionId), u64>,
) -> BTreeMap<(OptionId, OptionId), u64> {
    let seq: Vec<OptionId> = seq_ids(options);
    let n = seq.len();

    // Helper closures to access d(A,B) and P(A,B).
    let d = |a: &OptionId, b: &OptionId| -> u64 {
        wins.get(&(a.clone(), b.clone())).copied().unwrap_or(0)
    };

    // Initialize P.
    let mut p: BTreeMap<(OptionId, OptionId), u64> = BTreeMap::new();
    for i in 0..n {
        for j in 0..n {
            let a = &seq[i];
            let b = &seq[j];
            let val = if i == j {
                0
            } else {
                let dab = d(a, b);
                let dba = d(b, a);
                if dab > dba { dab } else { 0 }
            };
            p.insert((a.clone(), b.clone()), val);
        }
    }

    // Floyd–Warshall style update: k → i → j
    for k in 0..n {
        for i in 0..n {
            if i == k { continue; }
            for j in 0..n {
                if j == i || j == k { continue; }

                let a = &seq[i];
                let b = &seq[j];
                let c = &seq[k];

                // p[a,b] = max(p[a,b], min(p[a,c], p[c,b]))
                let pab = *p.get(&(a.clone(), b.clone())).unwrap_or(&0);
                let pac = *p.get(&(a.clone(), c.clone())).unwrap_or(&0);
                let pcb = *p.get(&(c.clone(), b.clone())).unwrap_or(&0);
                let candidate = core::cmp::min(pac, pcb);
                if candidate > pab {
                    p.insert((a.clone(), b.clone()), candidate);
                }
            }
        }
    }

    p
}
/// Result of Condorcet/Schulze tabulation for a single unit.
#[derive(Clone, Debug)]
pub struct CondorcetResult {
    /// Schulze strongest paths P[(A,B)].
    pub strongest_paths: BTreeMap<(OptionId, OptionId), u64>,
    /// Deterministic total order of options (best → worst).
    /// Ties (P[A,B] == P[B,A]) are resolved by canonical option order.
    pub order: Vec<OptionId>,
    /// All Condorcet winners (may be 0, 1, or >1 in case of cycles).
    /// Preserves canonical option order among winners.
    pub winners: Vec<OptionId>,
}

/// Compute deterministic Schulze ranking from `strongest_paths`.
/// Sort key: for A vs B, prefer larger P[A,B]; ties fall back to canonical order.
pub fn schulze_order(
    options: &[OptionItem],
    strongest_paths: &BTreeMap<(OptionId, OptionId), u64>,
) -> Vec<OptionId> {
    let seq = seq_ids(options);

    // Precompute canonical index for stable, deterministic tiebreaks.
    let mut idx = BTreeMap::<OptionId, usize>::new();
    for (i, id) in seq.iter().cloned().enumerate() {
        idx.insert(id, i);
    }

    let mut out = seq.clone();
    out.sort_by(|a, b| {
        use core::cmp::Ordering;

        let ab = *strongest_paths
            .get(&(a.clone(), b.clone()))
            .unwrap_or(&0);
        let ba = *strongest_paths
            .get(&(b.clone(), a.clone()))
            .unwrap_or(&0);

        match ab.cmp(&ba).reverse() {
            // Reverse so that larger ab ranks "earlier" (descending).
            Ordering::Equal => {
                // Canonical order fallback (by original options order).
                let ia = *idx.get(a).expect("idx");
                let ib = *idx.get(b).expect("idx");
                ia.cmp(&ib)
            }
            non_eq => non_eq,
        }
    });

    out
}

/// Return the (possibly multiple) Condorcet winners under Schulze:
/// A is a winner if for every B != A, P[A,B] >= P[B,A].
pub fn condorcet_winners(
    options: &[OptionItem],
    strongest_paths: &BTreeMap<(OptionId, OptionId), u64>,
) -> Vec<OptionId> {
    let seq = seq_ids(options);
    let n = seq.len();
    let mut winners = alloc::vec::Vec::new();

    'outer: for i in 0..n {
        let a = &seq[i];
        for j in 0..n {
            if i == j {
                continue;
            }
            let b = &seq[j];
            let pab = *strongest_paths
                .get(&(a.clone(), b.clone()))
                .unwrap_or(&0);
            let pba = *strongest_paths
                .get(&(b.clone(), a.clone()))
                .unwrap_or(&0);
            if pab < pba {
                // A does not beat/tie B.
                continue 'outer;
            }
        }
        winners.push(a.clone());
    }

    winners
}

/// End-to-end Condorcet/Schulze tabulation from a prepared pairwise matrix.
/// Deterministic; no RNG; ties resolved by canonical option order.
///
/// Callers are responsible for constructing `pairwise` from ballots in a way
/// that respects canonical option order and validation rules upstream.
pub fn tabulate_ranked_condorcet(
    options: &[OptionItem],
    pairwise: &Pairwise,
) -> CondorcetResult {
    let p = schulze_strongest_paths(options, pairwise.as_map());
    let order = schulze_order(options, &p);
    let winners = condorcet_winners(options, &p);

    CondorcetResult {
        strongest_paths: p,
        order,
        winners,
    }
}

#[cfg(test)]
mod tests_schulze {
    use super::*;
    use alloc::vec;

    fn opt(id: &str, idx: u16) -> OptionItem {
        OptionItem::new(
            id.parse().expect("opt id"),
            "name".to_string(),
            idx,
        )
        .expect("option")
    }

    #[test]
    fn schulze_trivial_singleton() {
        let options = vec![opt("O-A", 0)];
        let p = BTreeMap::new();
        let order = schulze_order(&options, &p);
        assert_eq!(order.len(), 1);
        assert_eq!(order[0].to_string(), "O-A");

        let winners = condorcet_winners(&options, &p);
        assert_eq!(winners.len(), 1);
        assert_eq!(winners[0].to_string(), "O-A");
    }

    #[test]
    fn schulze_deterministic_tie_uses_canonical_order() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut p: BTreeMap<(OptionId, OptionId), u64> = BTreeMap::new();
        // Tie: P[A,B] == P[B,A]
        p.insert(("O-A".parse().unwrap(), "O-B".parse().unwrap()), 3);
        p.insert(("O-B".parse().unwrap(), "O-A".parse().unwrap()), 3);

        let order = schulze_order(&options, &p);
        assert_eq!(
            order.iter().map(ToString::to_string).collect::<Vec<_>>(),
            alloc::vec!["O-A".to_string(), "O-B".to_string()]
        );

        let winners = condorcet_winners(&options, &p);
        // Both are winners under ≥ rule.
        assert_eq!(winners.len(), 2);
        assert_eq!(
            winners.iter().map(ToString::to_string).collect::<Vec<_>>(),
            alloc::vec!["O-A".to_string(), "O-B".to_string()]
        );
    }

    #[test]
    fn end_to_end_tabulate_ranked_condorcet() {
        let options = vec![opt("O-A", 0), opt("O-B", 1), opt("O-C", 2)];
        let mut pw = Pairwise::new(&options);
        // Simple majority cycle A>B, B>C, C>A (classic rock-paper-scissors).
        pw.increment(&"O-A".parse().unwrap(), &"O-B".parse().unwrap(), 6).unwrap();
        pw.increment(&"O-B".parse().unwrap(), &"O-A".parse().unwrap(), 4).unwrap();

        pw.increment(&"O-B".parse().unwrap(), &"O-C".parse().unwrap(), 6).unwrap();
        pw.increment(&"O-C".parse().unwrap(), &"O-B".parse().unwrap(), 4).unwrap();

        pw.increment(&"O-C".parse().unwrap(), &"O-A".parse().unwrap(), 6).unwrap();
        pw.increment(&"O-A".parse().unwrap(), &"O-C".parse().unwrap(), 4).unwrap();

        let result = tabulate_ranked_condorcet(&options, &pw);

        // There is a cycle, so winners will be all three (each ties/beats the others via P).
        assert_eq!(result.winners.len(), 3);
        // Order is deterministic by canonical order when pairwise strengths are symmetric.
        assert_eq!(
            result.order.iter().map(ToString::to_string).collect::<Vec<_>>(),
            alloc::vec!["O-A".to_string(), "O-B".to_string(), "O-C".to_string()]
        );

        // Strongest paths matrix is populated.
        assert!(!result.strongest_paths.is_empty());
    }
}
/// Validate that a `Pairwise` matrix is complete for the given `options`.
/// Requirements:
/// - Every (A,B) pair exists (owned OptionIds), including the diagonal.
/// - Diagonal entries (A,A) are exactly 0.
/// - No extraneous keys exist (i.e., all keys reference only `options`).
pub fn validate_pairwise_complete(
    options: &[OptionItem],
    pairwise: &Pairwise,
) -> Result<(), CondorcetError> {
    let seq = seq_ids(options);
    let set: alloc::collections::BTreeSet<OptionId> = seq.iter().cloned().collect();

    // Check presence and diagonal zeros.
    for a in &seq {
        for b in &seq {
            let key = (a.clone(), b.clone());
            let v = pairwise.as_map().get(&key).copied().unwrap_or(u64::MAX);
            if a == b {
                if v != 0 {
                    return Err(CondorcetError::Invariant("pairwise diagonal must be 0"));
                }
            } else if v == u64::MAX {
                return Err(CondorcetError::Invariant("pairwise missing (A,B) entry"));
            }
        }
    }

    // Check there are no extraneous keys.
    for (k, _) in pairwise.as_map().iter() {
        if !set.contains(&k.0) || !set.contains(&k.1) {
            return Err(CondorcetError::UnknownOption(
                if !set.contains(&k.0) { k.0.clone() } else { k.1.clone() },
            ));
        }
    }

    Ok(())
}

/// Enumerate all ordered pairs (A,B) in **canonical** option order,
/// including the diagonal (A,A). Useful for deterministic iteration.
pub fn canonical_pairs(options: &[OptionItem]) -> alloc::vec::Vec<(OptionId, OptionId)> {
    let seq = seq_ids(options);
    let mut out = alloc::vec::Vec::with_capacity(seq.len() * seq.len());
    for a in &seq {
        for b in &seq {
            out.push((a.clone(), b.clone()));
        }
    }
    out
}

#[cfg(test)]
mod tests_pairwise {
    use super::*;
    use alloc::vec;

    fn opt(id: &str, idx: u16) -> OptionItem {
        OptionItem::new(
            id.parse().expect("opt id"),
            "name".to_string(),
            idx,
        )
        .expect("option")
    }

    #[test]
    fn pairwise_new_is_complete_and_zero_diag() {
        let options = vec![opt("O-A", 0), opt("O-B", 1), opt("O-C", 2)];
        let pw = Pairwise::new(&options);
        validate_pairwise_complete(&options, &pw).expect("complete");
    }

    #[test]
    fn increment_rejects_diagonal_and_unknown() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut pw = Pairwise::new(&options);

        // Diagonal increment forbidden.
        let err = pw.increment(&"O-A".parse().unwrap(), &"O-A".parse().unwrap(), 1).unwrap_err();
        match err {
            CondorcetError::Invariant(msg) => assert!(msg.contains("forbidden")),
            _ => panic!("expected Invariant error"),
        }

        // Unknown option rejected.
        let err = pw.increment(&"O-X".parse().unwrap(), &"O-B".parse().unwrap(), 1).unwrap_err();
        match err {
            CondorcetError::UnknownOption(id) => assert_eq!(id.to_string(), "O-X"),
            _ => panic!("expected UnknownOption"),
        }
    }

    #[test]
    fn canonical_pairs_enumerates_in_canonical_order() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let pairs = canonical_pairs(&options);
        let as_strings: alloc::vec::Vec<(String, String)> = pairs
            .into_iter()
            .map(|(a,b)| (a.to_string(), b.to_string()))
            .collect();
        assert_eq!(
            as_strings,
            alloc::vec![
                ("O-A".to_string(), "O-A".to_string()),
                ("O-A".to_string(), "O-B".to_string()),
                ("O-B".to_string(), "O-A".to_string()),
                ("O-B".to_string(), "O-B".to_string()),
            ]
        );
    }

    #[test]
    fn validate_detects_missing_and_extraneous_keys() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut pw = Pairwise::new(&options);

        // Corrupt: remove one key.
        let key = ("O-A".parse().unwrap(), "O-B".parse().unwrap());
        assert!(pw.as_map().contains_key(&key));
        // Unsafe: we need a mutable access; reconstruct a broken map for the test.
        let mut broken = Pairwise { wins: pw.as_map().clone() };
        broken.wins.remove(&key);

        let err = validate_pairwise_complete(&options, &broken).unwrap_err();
        match err {
            CondorcetError::Invariant(msg) => assert!(msg.contains("missing")),
            _ => panic!("expected Invariant(missing)"),
        }

        // Corrupt: introduce extraneous key.
        let mut broken2 = Pairwise::new(&options);
        broken2.wins.insert(
            ("O-X".parse().unwrap(), "O-A".parse().unwrap()),
            1,
        );
        let err = validate_pairwise_complete(&options, &broken2).unwrap_err();
        match err {
            CondorcetError::UnknownOption(id) => assert_eq!(id.to_string(), "O-X"),
            _ => panic!("expected UnknownOption"),
        }
    }
}
