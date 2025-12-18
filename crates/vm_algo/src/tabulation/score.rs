// --------------------------------------------------------------------------------
// FILE: crates/vm_algo/src/tabulation/score.rs
// --------------------------------------------------------------------------------
//! Score tabulation (deterministic, integers-only).
//!
//! Inputs:
//! - `unit_id`: the unit identifier
//! - `score_sums`: per-option **summed scores** (already aggregated upstream)
//! - `turnout`: per-unit totals { valid_ballots, invalid_ballots }
//! - `params`: typed parameter set (used for scale/normalization if defined per release)
//! - `options`: canonical option list ordered by (order_index, OptionId)
//!
//! Output:
//! - `UnitScores { unit_id, turnout, scores }` where `scores` is a `BTreeMap<OptionId, u64>`.
//!
//! Rules in this layer:
//! - Reject unknown option IDs in `score_sums` (must match `options` exactly).
//! - If `valid_ballots == 0`, all option sums must be 0.
//! - If a per-ballot max score exists in Params, enforce
//!   `sum_for_option <= valid_ballots * max_scale` (widened arithmetic).
//!
//! No RNG, no floats. Downstream should iterate results using the provided
//! canonical `options` slice to preserve on-wire order.

#![forbid(unsafe_code)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};

use vm_core::{
    entities::{OptionItem, TallyTotals},
    ids::{OptionId, UnitId},
    variables::Params,
};

use crate::UnitScores;

/// Tabulation errors for score counting.
#[derive(Debug)]
pub enum TabError {
    /// `score_sums` contained an option ID not present in `options`.
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
    // Membership set for unknown-key detection (own the keys to avoid lifetime pitfalls).
    let allowed: BTreeSet<OptionId> = options.iter().map(|o| o.option_id.clone()).collect();

    // Reject any score keyed by an unknown option.
    for (k, _) in score_sums.iter() {
        if !allowed.contains(k) {
            return Err(TabError::UnknownOption(k.clone()));
        }
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
        // All sums must be zero in a unit with no valid ballots.
        // Use a saturating accumulation to avoid overflow traps while
        // still conveying a witness total in the error payload.
        let non_zero_total = scores
            .values()
            .fold(0u64, |acc, &x| acc.saturating_add(x));
        if non_zero_total != 0 {
            return Err(TabError::InconsistentTurnout { non_zero_total });
        }
        return Ok(());
    }

    // Try to extract a per-ballot max scale from Params (per release).
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
/// Returns `None` if the current release does not expose such a variable.
fn extract_max_scale(_params: &Params) -> Option<u64> {
    // Wire this to a real field/VM-VAR if/when defined by your release.
    None
}
