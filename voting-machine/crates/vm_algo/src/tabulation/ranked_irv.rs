// crates/vm_algo/src/tabulation/ranked_irv.rs
//
// Part 1/2 — module header, imports, core helpers (no_std-ready, deterministic)
//
// Fixes applied relative to prior issues:
// - no_std compatibility: use `alloc` + BTreeMap/BTreeSet (no HashMap).
// - Deterministic tie-break: lowest tally, then registry order_index, then OptionId.
// - Robust order index map with duplicate detection (owned keys, not &refs).
// - No invalid `OptionId::from("...")` construction anywhere.
//
// Part 2 will provide the main IRV tabulation function and elimination loop.
//
// NOTE: Adjust these import paths (UnitId, OptionId, Turnout, TabError) to your tree.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use core::cmp::Ordering;

// ---- Adjust these imports to your actual crate layout -----------------------
use crate::errors::TabError; // e.g., crate::errors::TabError or crate::error::TabError
use vm_core::ids::{OptionId, UnitId}; // e.g., vm_core::ids::{OptionId, UnitId}
use vm_core::models::Turnout;         // e.g., vm_core::models::Turnout { valid_ballots: u64 }
// -----------------------------------------------------------------------------

/// Borrowed ranked ballot: ordered preferences, most-preferred first.
/// Upstream validation guarantees all `OptionId`s are members of `options`.
#[derive(Clone, Copy)]
pub struct RankedBallot<'a> {
    pub prefs: &'a [OptionId],
}

/// Build registry order index: `OptionId -> order_index`.
/// Detects duplicates deterministically.
pub fn build_order_index(
    unit_id: UnitId,
    options: &[OptionId],
) -> Result<BTreeMap<OptionId, usize>, TabError> {
    let mut seen = BTreeSet::new();
    let mut idx = BTreeMap::new();
    for (i, &opt) in options.iter().enumerate() {
        if !seen.insert(opt) {
            return Err(TabError::DuplicateOptionInRegistry {
                unit_id,
                option_id: opt,
            });
        }
        idx.insert(opt, i);
    }
    Ok(idx)
}

/// Return the first still-active preference for a ballot, or None if exhausted.
#[inline]
pub fn next_active_pref(
    ballot: &RankedBallot<'_>,
    eliminated: &BTreeSet<OptionId>,
) -> Option<OptionId> {
    for &opt in ballot.prefs.iter() {
        if !eliminated.contains(&opt) {
            return Some(opt);
        }
    }
    None
}

/// Tally first-choice (current) preferences across ballots, skipping exhausted ones.
/// Only counts options that are **not** eliminated.
/// Returns `(counts, continuing_ballots)` where `continuing_ballots` excludes exhausted ballots.
pub fn tally_current_first_choices(
    ballots: &[RankedBallot<'_>],
    eliminated: &BTreeSet<OptionId>,
) -> (BTreeMap<OptionId, u64>, u64) {
    let mut counts: BTreeMap<OptionId, u64> = BTreeMap::new();
    let mut continuing: u64 = 0;

    for b in ballots {
        if let Some(opt) = next_active_pref(b, eliminated) {
            continuing = continuing.saturating_add(1);
            // deterministic map key order via BTreeMap
            let entry = counts.entry(opt).or_insert(0);
            *entry = entry.saturating_add(1);
        }
    }
    (counts, continuing)
}

/// Majority winner among `continuing_options` given counts and continuing ballot total.
/// Returns `Some(winner)` if any option has strict majority; otherwise `None`.
pub fn check_majority(
    counts: &BTreeMap<OptionId, u64>,
    continuing_options: &BTreeSet<OptionId>,
    continuing_ballots: u64,
) -> Option<OptionId> {
    if continuing_ballots == 0 {
        return None;
    }
    let threshold = (continuing_ballots / 2) + 1;
    let mut best: Option<(u64, OptionId)> = None;

    for &opt in continuing_options.iter() {
        let c = *counts.get(&opt).unwrap_or(&0);
        if c >= threshold {
            // majority found; if multiple meet, pick deterministically by OptionId
            match best {
                None => best = Some((c, opt)),
                Some((prev_c, prev_opt)) => {
                    if c > prev_c || (c == prev_c && opt < prev_opt) {
                        best = Some((c, opt));
                    }
                }
            }
        }
    }
    best.map(|(_, opt)| opt)
}

/// Deterministically pick the **lowest** option to eliminate:
/// 1) smallest tally
/// 2) then smallest registry `order_index`
/// 3) then smallest `OptionId`
///
/// `eligible` are the not-eliminated options; others are ignored.
pub fn pick_lowest_to_eliminate(
    counts: &BTreeMap<OptionId, u64>,
    order_index: &BTreeMap<OptionId, usize>,
    eligible: &BTreeSet<OptionId>,
) -> Option<OptionId> {
    let mut best: Option<(u64, usize, OptionId)> = None;

    for &opt in eligible.iter() {
        let tally = *counts.get(&opt).unwrap_or(&0);
        let ord = *order_index
            .get(&opt)
            .expect("order_index must contain every eligible option");

        match best {
            None => best = Some((tally, ord, opt)),
            Some((bt, bo, boi)) => {
                // Lower tally is worse (preferred for elimination)
                // Then lower order index (earlier in registry)
                // Then lower OptionId
                let candidate = (tally, ord, opt);
                let cmp = match tally.cmp(&bt) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => match ord.cmp(&bo) {
                        Ordering::Less => Ordering::Less,
                        Ordering::Greater => Ordering::Greater,
                        Ordering::Equal => opt.cmp(&boi),
                    },
                };
                if cmp == Ordering::Less {
                    best = Some(candidate);
                }
            }
        }
    }

    best.map(|t| t.2) // NOTE: correct tuple index (.2), not .3
}
// crates/vm_algo/src/tabulation/ranked_irv.rs
//
// Part 2/2 — main IRV tabulation loop (deterministic, no_std-ready)
//
// Requires Part 1 (helpers & types) to be present in this module.

/// Run IRV tabulation and return:
/// - final round tallies in **registry order** (Vec of `(OptionId, u64)`), and
/// - the selected winner `OptionId`.
///
/// Notes:
/// - Deterministic elimination order: lowest tally → lowest registry order_index → lowest OptionId.
/// - Majority is computed on **continuing ballots** (non-exhausted in the current round).
/// - Eliminated options appear with `0` in the final tallies vector (final-round view).
pub fn tabulate_ranked_irv_in_registry_order_vec(
    unit_id: UnitId,
    ballots: &[RankedBallot<'_>],
    options: &[OptionId],
    _turnout: &Turnout, // kept for API parity; IRV uses ballots directly
) -> Result<(Vec<(OptionId, u64)>, OptionId), TabError> {
    // Build canonical order index, detecting duplicates.
    let order_index = build_order_index(unit_id, options)?;

    // Eligible options = all registry options at start.
    let mut eliminated: BTreeSet<OptionId> = BTreeSet::new();
    let mut eligible: BTreeSet<OptionId> = BTreeSet::new();
    for &opt in options.iter() {
        eligible.insert(opt);
    }

    // Guard: must have at least one option.
    if eligible.is_empty() {
        return Err(TabError::NoOptions { unit_id });
    }

    // Main IRV loop.
    loop {
        // Build continuing set (eligible \ eliminated) each round.
        let mut continuing: BTreeSet<OptionId> = BTreeSet::new();
        for &opt in options.iter() {
            if !eliminated.contains(&opt) {
                continuing.insert(opt);
            }
        }

        // If only one option remains, declare it as winner (exhaustive elimination).
        if continuing.len() == 1 {
            let winner = *continuing.iter().next().expect("len==1");
            let final_counts = finalize_counts_in_registry_order(&continuing, &BTreeMap::new(), options);
            return Ok((final_counts, winner));
        }

        // Tally first-choice preferences among continuing ballots.
        let (counts, continuing_ballots) = tally_current_first_choices(ballots, &eliminated);

        // Majority check (strict > 50% of continuing ballots).
        if let Some(winner) = check_majority(&counts, &continuing, continuing_ballots) {
            let final_counts = finalize_counts_in_registry_order(&continuing, &counts, options);
            return Ok((final_counts, winner));
        }

        // No majority yet — eliminate one option deterministically.
        match pick_lowest_to_eliminate(&counts, &order_index, &continuing) {
            Some(to_eliminate) => {
                // Insert and continue to next round.
                let inserted = eliminated.insert(to_eliminate);
                debug_assert!(inserted, "candidate must not already be eliminated");
                // Loop continues.
            }
            None => {
                // Should not happen: if `continuing` is non-empty, we must be able to pick one.
                // Return a structured error to surface invariant violation.
                return Err(TabError::NoEliminationCandidate { unit_id });
            }
        }
    }
}

/// Convenience variant returning a map keyed by OptionId (sorted by key)
/// plus the winner. If you need **registry order**, use the Vec variant.
pub fn tabulate_ranked_irv(
    unit_id: UnitId,
    ballots: &[RankedBallot<'_>],
    options: &[OptionId],
    turnout: &Turnout,
) -> Result<(BTreeMap<OptionId, u64>, OptionId), TabError> {
    let (vec_final, winner) =
        tabulate_ranked_irv_in_registry_order_vec(unit_id, ballots, options, turnout)?;
    let mut out: BTreeMap<OptionId, u64> = BTreeMap::new();
    for (opt, c) in vec_final {
        out.insert(opt, c);
    }
    Ok((out, winner))
}

/// Build final-round counts in **registry order** for reporting/consumers.
/// - continuing options keep their round counts;
/// - eliminated options are reported as 0 in the final view.
fn finalize_counts_in_registry_order(
    continuing: &BTreeSet<OptionId>,
    round_counts: &BTreeMap<OptionId, u64>,
    options: &[OptionId],
) -> Vec<(OptionId, u64)> {
    let mut out: Vec<(OptionId, u64)> = Vec::with_capacity(options.len());
    for &opt in options.iter() {
        let c = if continuing.contains(&opt) {
            *round_counts.get(&opt).unwrap_or(&0)
        } else {
            0
        };
        out.push((opt, c));
    }
    out
}

// ---- Error type expectation --------------------------------------------------
//
// This part assumes the following additional `TabError` variants exist:
// - NoOptions { unit_id: UnitId }
// - NoEliminationCandidate { unit_id: UnitId }
//
// If names/fields differ in your repo, adjust the error returns above.
// -----------------------------------------------------------------------------
