//! Plurality tabulation (deterministic, integers-only).
//!
//! Inputs:
//! - `unit_id`: the unit identifier
//! - `votes`:   per-option vote counts (may omit options → treated as 0)
//! - `turnout`: valid/invalid ballots summary
//! - `options`: canonical option list ordered by (order_index, OptionId)
//!
//! Output:
//! - `UnitScores { unit_id, turnout, scores }` where `scores` is a `BTreeMap<OptionId, u64>`.
//!
//! Notes:
//! - Unknown option keys present in `votes` are rejected.
//! - Σ(option votes) must be ≤ `turnout.valid_ballots`.
//! - No RNG, no floats. Downstream should iterate results using the provided
//!   canonical `options` slice to preserve on-wire order.

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    entities::{OptionItem, TallyTotals},
    ids::{OptionId, UnitId},
};

use crate::UnitScores;

/// Tabulation errors for plurality counting.
#[derive(Debug)]
pub enum TabError {
    /// `votes` contained an option ID not present in the canonical `options` list.
    UnknownOption(OptionId),
    /// Sum of per-option votes exceeded the unit's `valid_ballots`.
    TallyExceedsValid { sum_votes: u64, valid_ballots: u64 },
}

/// Deterministic plurality tabulation (integers only; no RNG).
pub fn tabulate_plurality(
    unit_id: UnitId,
    votes: &BTreeMap<OptionId, u64>,
    turnout: TallyTotals,
    options: &[OptionItem],
) -> Result<UnitScores, TabError> {
    let (scores, sum) = canonicalize_scores(votes, options)?;
    check_tally_sanity(sum, &turnout)?;
    Ok(UnitScores {
        unit_id,
        turnout,
        scores,
    })
}

/// Build a canonical score map from provided `votes` and canonical `options`.
/// Iterates `options` in (order_index, OptionId) order; missing keys → 0;
/// rejects unknown keys present in `votes`.
fn canonicalize_scores(
    votes: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> Result<(BTreeMap<OptionId, u64>, u64), TabError> {
    // Fast membership set for unknown-key detection.
    let allowed: BTreeSet<&OptionId> = options.iter().map(|o| &o.option_id).collect();

    // Reject any vote keyed by an unknown option.
    if let Some((bad_id, _)) = votes.iter().find(|(k, _)| !allowed.contains(k)) {
        return Err(TabError::UnknownOption((*bad_id).clone()));
    }

    // Build scores by traversing options in canonical order.
    let mut scores: BTreeMap<OptionId, u64> = BTreeMap::new();
    let mut sum: u64 = 0;

    for opt in options {
        let count = votes.get(&opt.option_id).copied().unwrap_or(0);

        // Detect improbable u64 overflow early and treat as exceeds-valid.
        let (new_sum, overflow) = sum.overflowing_add(count);
        if overflow {
            return Err(TabError::TallyExceedsValid {
                sum_votes: u64::MAX,
                valid_ballots: 0, // will be overwritten by caller-side sanity; set 0 here defensively
            });
        }
        sum = new_sum;

        scores.insert(opt.option_id.clone(), count);
    }

    Ok((scores, sum))
}

/// Sanity: Σ option votes must not exceed `valid_ballots`.
fn check_tally_sanity(sum_votes: u64, turnout: &TallyTotals) -> Result<(), TabError> {
    let valid = turnout.valid_ballots;
    if sum_votes > valid {
        return Err(TabError::TallyExceedsValid {
            sum_votes,
            valid_ballots: valid,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vm_core::entities::OptionItem;

    fn opt(id: &str, idx: u16) -> OptionItem {
        OptionItem::new(
            id.parse().expect("opt id"),
            "name".to_string(),
            idx,
        )
        .expect("option")
    }

    #[test]
    fn happy_path_builds_scores_in_canonical_order() {
        // Options in canonical order by (order_index, option_id)
        let options = vec![opt("O-A", 0), opt("O-B", 1), opt("O-C", 2)];

        // Votes map insertion order is irrelevant.
        let mut votes = BTreeMap::<OptionId, u64>::new();
        votes.insert("O-B".parse().unwrap(), 20);
        votes.insert("O-A".parse().unwrap(), 10);
        votes.insert("O-C".parse().unwrap(), 30);

        let turnout = TallyTotals::new(60, 0);

        let (scores, sum) = canonicalize_scores(&votes, &options).expect("ok");
        assert_eq!(sum, 60);

        // Iteration over BTreeMap is lex by OptionId; downstream will iterate via `options`.
        assert_eq!(scores.get(&"O-A".parse().unwrap()).copied(), Some(10));
        assert_eq!(scores.get(&"O-B".parse().unwrap()).copied(), Some(20));
        assert_eq!(scores.get(&"O-C".parse().unwrap()).copied(), Some(30));

        // Full tabulate
        let unit_id: UnitId = "U-001".parse().unwrap();
        let us = tabulate_plurality(unit_id, &votes, turnout, &options).expect("ok");
        assert_eq!(us.turnout.valid_ballots, 60);
    }

    #[test]
    fn missing_keys_are_zero_unknown_are_error() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut votes = BTreeMap::<OptionId, u64>::new();
        votes.insert("O-A".parse().unwrap(), 5);
        votes.insert("O-X".parse().unwrap(), 1); // unknown

        let err = canonicalize_scores(&votes, &options).unwrap_err();
        match err {
            TabError::UnknownOption(id) => assert_eq!(id.to_string(), "O-X"),
            _ => panic!("expected UnknownOption"),
        }
    }

    #[test]
    fn sanity_sum_must_not_exceed_valid() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut votes = BTreeMap::<OptionId, u64>::new();
        votes.insert("O-A".parse().unwrap(), 50);
        votes.insert("O-B".parse().unwrap(), 51);

        let turnout = TallyTotals::new(100, 0);
        let (scores, sum) = canonicalize_scores(&votes, &options).expect("ok");
        assert_eq!(sum, 101);

        let err = check_tally_sanity(sum, &turnout).unwrap_err();
        match err {
            TabError::TallyExceedsValid { sum_votes, valid_ballots } => {
                assert_eq!(sum_votes, 101);
                assert_eq!(valid_ballots, 100);
            }
            _ => panic!("expected TallyExceedsValid"),
        }
    }
}
