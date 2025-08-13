//! Score tabulation (deterministic, integers-only).
//!
//! Inputs:
//! - `unit_id`: the unit identifier
//! - `score_sums`: per-option **summed scores** (already aggregated upstream)
//! - `turnout`: per-unit totals { valid_ballots, invalid_ballots } (Doc 1B names)
//! - `params`: typed parameter set (used for scale/normalization if defined per release)
//! - `options`: canonical option list ordered by (order_index, OptionId)
//!
//! Output:
//! - `UnitScores { unit_id, turnout, scores }` where `scores` is a `BTreeMap<OptionId, u64>`.
//!
//! Rules/enforcement in this layer (domain-only):
//! - Unknown option keys present in `score_sums` are rejected.
//! - If `valid_ballots == 0`: all option sums must be 0 → otherwise error.
//! - If a max per-ballot score is available from Params, enforce per-option cap
//!   `sum_for_option <= valid_ballots * max_scale` (widened arithmetic).
//!
//! No RNG, no floats. Downstream should iterate results using the provided
//! canonical `options` slice to preserve on-wire order.

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    entities::{OptionItem, TallyTotals},
    ids::{OptionId, UnitId},
    variables::Params,
};

use crate::UnitScores;

/// Tabulation errors for score counting.
#[derive(Debug)]
pub enum TabError {
    /// `score_sums` contained an option ID not present in the canonical `options` list.
    UnknownOption(OptionId),
    /// Turnout says there are zero valid ballots but some option has a non-zero sum.
    InconsistentTurnout { non_zero_total: u64 },
    /// A single option's summed score exceeded the plausible cap: `valid_ballots * max_scale`.
    OptionExceedsCap {
        option: OptionId,
        sum: u64,
        cap: u128,
    },
    /// (Reserved) Invalid scale bounds in parameters, if enforced per release.
    InvalidScaleBounds,
}

/// Deterministic score tabulation (integers only; no RNG).
pub fn tabulate_score(
    unit_id: UnitId,
    score_sums: &BTreeMap<OptionId, u64>,
    turnout: TallyTotals,
    params: &Params,
    options: &[OptionItem],
) -> Result<UnitScores, TabError> {
    let scores = canonicalize_scores(score_sums, options)?;
    check_scale_and_caps(&scores, &turnout, params)?;
    Ok(UnitScores {
        unit_id,
        turnout,
        scores,
    })
}

/// Build a canonical score map from provided `score_sums` and canonical `options`.
/// Iterates `options` in (order_index, OptionId) order; missing keys → 0;
/// rejects unknown keys present in `score_sums`.
fn canonicalize_scores(
    score_sums: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> Result<BTreeMap<OptionId, u64>, TabError> {
    // Fast membership set for unknown-key detection.
    let allowed: BTreeSet<&OptionId> = options.iter().map(|o| &o.option_id).collect();

    // Reject any score keyed by an unknown option.
    if let Some((bad_id, _)) = score_sums.iter().find(|(k, _)| !allowed.contains(k)) {
        return Err(TabError::UnknownOption((*bad_id).clone()));
    }

    // Build scores by traversing options in canonical order.
    let mut scores: BTreeMap<OptionId, u64> = BTreeMap::new();
    for opt in options {
        let sum = score_sums.get(&opt.option_id).copied().unwrap_or(0);
        scores.insert(opt.option_id.clone(), sum);
    }
    Ok(scores)
}

/// Domain checks:
/// * If valid_ballots == 0 ⇒ all sums must be 0.
/// * If a max per-ballot score is available in Params (per release), enforce
///   sum_for_option <= valid_ballots * max_scale using widened arithmetic.
fn check_scale_and_caps(
    scores: &BTreeMap<OptionId, u64>,
    turnout: &TallyTotals,
    params: &Params,
) -> Result<(), TabError> {
    let v = turnout.valid_ballots;

    if v == 0 {
        // All sums must be zero in a zero-valid-ballots unit.
        let non_zero_total: u64 = scores.values().copied().sum();
        if non_zero_total != 0 {
            return Err(TabError::InconsistentTurnout { non_zero_total });
        }
        return Ok(()); // nothing else to check
    }

    // Try to extract a per-ballot max scale from Params (per release).
    // NOTE: The core ParameterSet in this repository does not define a generic
    // score scale. If your release does, wire it into this extractor.
    if let Some(max_scale) = extract_max_scale(params) {
        let cap_per_option: u128 = (v as u128) * (max_scale as u128);
        for (opt, &sum) in scores {
            if (sum as u128) > cap_per_option {
                return Err(TabError::OptionExceedsCap {
                    option: opt.clone(),
                    sum,
                    cap: cap_per_option,
                });
            }
        }
    }

    Ok(())
}

/// Attempt to extract a per-ballot max score from Params.
/// Returns `None` if the current release does not expose such a variable
/// in the ParameterSet (common in canonical inputs where score scales are
/// part of ingestion rather than runtime params).
fn extract_max_scale(_params: &Params) -> Option<u64> {
    // Placeholder: wire to a real field if/when defined per release.
    None
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

        // Insertion order of map is irrelevant.
        let mut sums = BTreeMap::<OptionId, u64>::new();
        sums.insert("O-B".parse().unwrap(), 200);
        sums.insert("O-A".parse().unwrap(), 100);
        sums.insert("O-C".parse().unwrap(), 400);

        let turnout = TallyTotals::new(100, 0);
        let params = Params::default();

        let scores = canonicalize_scores(&sums, &options).expect("ok");
        assert_eq!(scores.get(&"O-A".parse().unwrap()).copied(), Some(100));
        assert_eq!(scores.get(&"O-B".parse().unwrap()).copied(), Some(200));
        assert_eq!(scores.get(&"O-C".parse().unwrap()).copied(), Some(400));

        // Full tabulate
        let unit_id: UnitId = "U-001".parse().unwrap();
        let us = tabulate_score(unit_id, &sums, turnout, &params, &options).expect("ok");
        assert_eq!(us.turnout.valid_ballots, 100);
    }

    #[test]
    fn unknown_option_rejected() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut sums = BTreeMap::<OptionId, u64>::new();
        sums.insert("O-A".parse().unwrap(), 5);
        sums.insert("O-X".parse().unwrap(), 1); // unknown

        let err = canonicalize_scores(&sums, &options).unwrap_err();
        match err {
            TabError::UnknownOption(id) => assert_eq!(id.to_string(), "O-X"),
            _ => panic!("expected UnknownOption"),
        }
    }

    #[test]
    fn zero_valid_ballots_requires_all_zero_sums() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut sums = BTreeMap::<OptionId, u64>::new();
        sums.insert("O-A".parse().unwrap(), 0);
        sums.insert("O-B".parse().unwrap(), 0);

        let turnout = TallyTotals::new(0, 0);
        let params = Params::default();

        tabulate_score("U-1".parse().unwrap(), &sums, turnout, &params, &options).expect("ok");
    }

    #[test]
    fn zero_valid_ballots_with_non_zero_fails() {
        let options = vec![opt("O-A", 0), opt("O-B", 1)];
        let mut sums = BTreeMap::<OptionId, u64>::new();
        sums.insert("O-A".parse().unwrap(), 1);
        sums.insert("O-B".parse().unwrap(), 0);

        let turnout = TallyTotals::new(0, 0);
        let params = Params::default();

        let err = tabulate_score("U-1".parse().unwrap(), &sums, turnout, &params, &options)
            .unwrap_err();
        match err {
            TabError::InconsistentTurnout { non_zero_total } => {
                assert_eq!(non_zero_total, 1);
            }
            _ => panic!("expected InconsistentTurnout"),
        }
    }
}
