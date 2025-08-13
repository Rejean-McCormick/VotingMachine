//! Approval tabulation (deterministic, integers-only).
//!
//! Inputs:
//! - `unit_id`: the unit identifier
//! - `approvals`: per-option approval counts (may omit options → treated as 0)
//! - `turnout`: per-unit totals { valid_ballots, invalid_ballots } (Doc 1B names)
//! - `options`: canonical option list ordered by (order_index, OptionId)
//!
////! Output:
//! - `UnitScores { unit_id, turnout, scores }` where `scores` is a `BTreeMap<OptionId, u64>`.
//!
//! Notes:
//! - Unknown option keys present in `approvals` are rejected.
//! - Per-option cap: approvals_for_option ≤ valid_ballots.
//! - Σ approvals may exceed valid_ballots (multiple approvals per ballot are allowed).
//! - No RNG, no floats. Downstream should iterate results using the provided
//!   canonical `options` slice to preserve on-wire order.

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    entities::{OptionItem, TallyTotals},
    ids::{OptionId, UnitId},
};

use crate::UnitScores;

/// Tabulation errors for approval counting.
#[derive(Debug)]
pub enum TabError {
    /// `approvals` contained an option ID not present in the canonical `options` list.
    UnknownOption(OptionId),
    /// A single option's approvals exceeded the unit's `valid_ballots`.
    OptionExceedsValid {
        option: OptionId,
        approvals: u64,
        valid_ballots: u64,
    },
}

/// Deterministic approval tabulation (integers only; no RNG).
pub fn tabulate_approval(
    unit_id: UnitId,
    approvals: &BTreeMap<OptionId, u64>,
    turnout: TallyTotals,
    options: &[OptionItem],
) -> Result<UnitScores, TabError> {
    let scores = canonicalize_scores(approvals, options)?;
    check_per_option_caps(&scores, &turnout)?;
    Ok(UnitScores {
        unit_id,
        turnout,
        scores,
    })
}

/// Build a canonical score map from provided `approvals` and canonical `options`.
/// Iterates `options` in (order_index, OptionId) order; missing keys → 0;
/// rejects unknown keys present in `approvals`.
fn canonicalize_scores(
    approvals: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> Result<BTreeMap<OptionId, u64>, TabError> {
    // Fast membership set for unknown-key detection.
    let allowed: BTreeSet<&OptionId> = options.iter().map(|o| &o.option_id).collect();

    // Reject any approval keyed by an unknown option.
    if let Some((bad_id, _)) = approvals.iter().find(|(k, _)| !allowed.contains(k)) {
        return Err(TabError::UnknownOption((*bad_id).clone()));
    }

    // Build scores by traversing options in canonical order.
    let mut scores: BTreeMap<OptionId, u64> = BTreeMap::new();
    for opt in options {
        let count = approvals.get(&opt.option_id).copied().unwrap_or(0);
        scores.insert(opt.option_id.clone(), count);
    }
    Ok(scores)
}

/// Sanity: per-option approvals must not exceed `valid_ballots`.
fn check_per_option_caps(
    scores: &BTreeMap<OptionId, u64>,
    turnout: &TallyTotals,
) -> Result<(), TabError> {
    let valid = turnout.valid_ballots;
    for (opt, &count) in scores {
        if count > valid {
            return Err(TabError::OptionExceedsValid {
                option: opt.clone(),
                approvals: count,
                valid_ballots: valid,
            });
        }
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
        let options = vec![opt("O-A", 0), opt("O-B", 1), opt("O-C", 2)];

        // Insertion order of approvals map is irrelevant.
        let mut approvals = BTreeMap::<OptionId, u64>::new();
        approvals.insert("O-B".parse().unwrap(), 20);
        approvals.insert("O-A".parse().unwrap(), 10);
        approvals.insert("O-C".parse().unwrap(), 40);

        let turnout = TallyTotals::new(100, 0);

        let scores = canonicalize_scores(&approvals, &options).expect("ok");
        assert_eq!(scores.get(&"O-A".parse().unwrap()).copied(), Some(10));
        assert_eq!(scores.get(&"O-B".parse().unwrap()).copied(), Some(20));
        assert_eq!(scores.get(&"O-C".parse().unwrap()).copied(), Some(40));

        // Full tabulate
        let unit_id: UnitId = "U-001".parse().unwrap();
        let us = tabulate_approval(unit_id, &approvals, turnout, &options).expect("ok");
        assert_eq!(us.turnout.valid_ballots, 100);
    }

    #[test]
    fn unknown_option_rejected() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut approvals = BTreeMap::<OptionId, u64>::new();
        approvals.insert("O-A".parse().unwrap(), 5);
        approvals.insert("O-X".parse().unwrap(), 1); // unknown

        let err = canonicalize_scores(&approvals, &options).unwrap_err();
        match err {
            TabError::UnknownOption(id) => assert_eq!(id.to_string(), "O-X"),
            _ => panic!("expected UnknownOption"),
        }
    }

    #[test]
    fn per_option_caps_enforced() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut approvals = BTreeMap::<OptionId, u64>::new();
        approvals.insert("O-A".parse().unwrap(), 51);
        approvals.insert("O-B".parse().unwrap(), 50);

        let turnout = TallyTotals::new(50, 0);

        let scores = canonicalize_scores(&approvals, &options).expect("ok");
        let err = check_per_option_caps(&scores, &turnout).unwrap_err();
        match err {
            TabError::OptionExceedsValid {
                option,
                approvals,
                valid_ballots,
            } => {
                assert_eq!(option.to_string(), "O-A");
                assert_eq!(approvals, 51);
                assert_eq!(valid_ballots, 50);
            }
            _ => panic!("expected OptionExceedsValid"),
        }
    }

    #[test]
    fn zero_valid_ballots_forces_zeros() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut approvals = BTreeMap::<OptionId, u64>::new();
        approvals.insert("O-A".parse().unwrap(), 0);
        approvals.insert("O-B".parse().unwrap(), 0);

        let turnout = TallyTotals::new(0, 0);
        let scores = canonicalize_scores(&approvals, &options).expect("ok");
        check_per_option_caps(&scores, &turnout).expect("ok");
    }
}
