// crates/vm_algo/src/tabulation/plurality.rs
//
// Fixed implementation (deterministic, no_std-ready, registry-order aware).
//
// Key fixes vs previous version:
// - Preserves the Registry / `options` slice order via a Vec-returning API.
// - Keeps the BTreeMap-returning API for back-compat, but clarifies it is
//   sorted by OptionId (not registry order) and adds a helper that returns
//   registry order when needed.
// - Detects duplicate entries in `options` (dedicated error).
// - Adds per-option sanity check: count ≤ turnout.valid_ballots.
// - Keeps integer-first math: u128 accumulation for total to avoid overflow.
//
// Contract (aligned to Docs 1–7 + Annexes A–C):
// - Inputs are expected to be validated upstream; this function enforces the
//   invariants defensively and returns structured errors (no silent fixes).
// - No RNG, no I/O; stable ordering and pure integer arithmetic.
//
// NOTE: Adjust import paths (UnitId, OptionId, Turnout, TabError) to your tree.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

// ---- Adjust these imports to your actual crate layout -----------------------
use crate::errors::TabError; // e.g., crate::errors::TabError or crate::error::TabError

// Example paths (rename to your real modules):
use vm_core::ids::{OptionId, UnitId}; // e.g., vm_core::ids::{OptionId, UnitId}
use vm_core::models::Turnout;         // e.g., vm_core::models::Turnout { valid_ballots: u64 }
// -----------------------------------------------------------------------------

/// Plurality tabulation that **preserves Registry order** (the order of `options`).
///
/// Returns a Vec of `(OptionId, votes)` in the exact order provided by `options`.
/// Use this when canonical, registry-driven ordering is required downstream.
pub fn tabulate_plurality_in_registry_order_vec(
    unit_id: UnitId,
    votes: &BTreeMap<OptionId, u64>,
    turnout: &Turnout,
    options: &[OptionId],
) -> Result<Vec<(OptionId, u64)>, TabError> {
    // Build a set of `options` and detect duplicates deterministically.
    let mut seen: BTreeSet<OptionId> = BTreeSet::new();
    for &opt in options.iter() {
        if !seen.insert(opt) {
            // Duplicate discovered in registry slice.
            return Err(TabError::DuplicateOptionInRegistry {
                unit_id,
                option_id: opt,
            });
        }
    }

    // Defensive: unknown options present in `votes`?
    // (Should be unreachable after upstream validation.)
    for (&opt, _) in votes.iter() {
        if !seen.contains(&opt) {
            return Err(TabError::UnknownOption {
                unit_id,
                option_id: opt,
            });
        }
    }

    // Defensive: missing options relative to registry order?
    // Spec prefers strict presence rather than "missing => 0".
    for &opt in options.iter() {
        if !votes.contains_key(&opt) {
            return Err(TabError::MissingOption {
                unit_id,
                option_id: opt,
            });
        }
    }

    // Per-option sanity: no single option may exceed valid ballots.
    for (&opt, &count) in votes.iter() {
        if count > turnout.valid_ballots {
            return Err(TabError::OptionVotesExceedValid {
                unit_id,
                option_id: opt,
                count,
                valid_ballots: turnout.valid_ballots,
            });
        }
    }

    // Integer-first total accumulation to avoid overflow.
    let mut sum_u128: u128 = 0;
    for &count in votes.values() {
        sum_u128 = sum_u128
            .checked_add(count as u128)
            .ok_or_else(|| TabError::ArithmeticOverflow {
                unit_id,
                context: "plurality: sum(votes) overflowed u128",
            })?;
    }

    if sum_u128 > (turnout.valid_ballots as u128) {
        return Err(TabError::TallyExceedsValid {
            unit_id,
            sum_votes: (if sum_u128 <= u128::from(u64::MAX) {
                sum_u128 as u64
            } else {
                u64::MAX
            }),
            valid_ballots: turnout.valid_ballots,
        });
    }

    // Build the result in **registry order**.
    let mut out: Vec<(OptionId, u64)> = Vec::with_capacity(options.len());
    for &opt in options.iter() {
        // safe due to missing-option check above
        let count = *votes.get(&opt).expect("checked: option exists");
        out.push((opt, count));
    }

    Ok(out)
}

/// Plurality tabulation that returns a map keyed by OptionId (sorted by key).
///
/// This preserves determinism but **not** the original registry order of `options`.
/// If you need registry order downstream, call
/// [`tabulate_plurality_in_registry_order_vec`] instead.
pub fn tabulate_plurality(
    unit_id: UnitId,
    votes: &BTreeMap<OptionId, u64>,
    turnout: &Turnout,
    options: &[OptionId],
) -> Result<BTreeMap<OptionId, u64>, TabError> {
    let vec_in_order =
        tabulate_plurality_in_registry_order_vec(unit_id, votes, turnout, options)?;

    // Collect into BTreeMap (sorted by OptionId). This is OK for callers that
    // do not rely on registry order from this function. Canonical ordering needs
    // the Vec API above or a dedicated reorder at the call site.
    let mut out: BTreeMap<OptionId, u64> = BTreeMap::new();
    for (opt, count) in vec_in_order {
        out.insert(opt, count);
    }
    Ok(out)
}

// ---- Error type expectation --------------------------------------------------
//
// This implementation expects `TabError` to provide the following variants.
// If your actual error enum differs, adjust the return sites accordingly.
//
// - UnknownOption { unit_id: UnitId, option_id: OptionId }
// - MissingOption { unit_id: UnitId, option_id: OptionId }
// - DuplicateOptionInRegistry { unit_id: UnitId, option_id: OptionId }
// - OptionVotesExceedValid { unit_id: UnitId, option_id: OptionId, count: u64, valid_ballots: u64 }
// - TallyExceedsValid { unit_id: UnitId, sum_votes: u64, valid_ballots: u64 }
// - ArithmeticOverflow { unit_id: UnitId, context: &'static str }
//
// -----------------------------------------------------------------------------
