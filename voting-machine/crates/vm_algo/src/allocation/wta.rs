//! Winner-take-all allocation (deterministic; integers only; RNG used only when TiePolicy::Random).
//!
//! Contract (Doc 1 / Annex A aligned):
//! - Magnitude must be exactly 1.
//! - Pick the highest score; if multiple options tie for top, break the tie by the selected policy:
//!     * StatusQuo  → attempt SQ pick (if flagged upstream), otherwise fall back to DeterministicOrder.
//!     * DeterministicOrder → smallest by canonical option order (order_index, then option_id).
//!     * Random → uniform choice using provided `TieRng`; if no RNG is provided, fall back to DeterministicOrder.
//! - Output encodes WTA as 100 “power” for the winner.
//!
//! Notes:
//! - `OptionItem` in vm_core intentionally has no `is_status_quo` flag; if none is known,
//!   StatusQuo policy deterministically falls back to canonical order.

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    ids::OptionId,
    entities::OptionItem,
    rng::TieRng,
    variables::TiePolicy,
};

use crate::Allocation;
use crate::UnitScores;

/// Errors for WTA allocation.
#[derive(Debug)]
pub enum AllocError {
    /// WTA requires magnitude == 1.
    InvalidMagnitude { got: u32 },
    /// Defensive: a score key did not exist in the provided options list.
    UnknownOption(OptionId),
}

/// Winner-take-all. Expects `magnitude == 1`.
///
/// *Determinism:* when `tie_policy != Random` (or RNG not supplied), ties are broken by canonical
/// option order `(order_index, option_id)`.
pub fn allocate_wta(
    scores: &UnitScores,
    magnitude: u32,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    mut rng: Option<&mut TieRng>,
) -> Result<Allocation, AllocError> {
    if magnitude != 1 {
        return Err(AllocError::InvalidMagnitude { got: magnitude });
    }

    // Defensive: ensure no unknown option IDs appear in the score map.
    // (Upstream cross-ref in vm_io.loader should already guarantee this.)
    let allowed: BTreeSet<&OptionId> = options.iter().map(|o| &o.option_id).collect();
    for k in scores.scores.keys() {
        if !allowed.contains(k) {
            return Err(AllocError::UnknownOption(k.clone()));
        }
    }

    // Find max score and collect all tied-at-max, using canonical option order.
    let (max_score, tied) = top_by_score(scores, options);

    // Select winner (resolve ties per policy).
    let winner = if tied.len() == 1 {
        tied[0].clone()
    } else {
        break_tie_wta(&tied, options, tie_policy, rng.as_deref_mut())
    };

    // Encode WTA: winner gets 100 "power".
    let mut seats_or_power: BTreeMap<OptionId, u32> = BTreeMap::new();
    if max_score > 0 || !tied.is_empty() {
        seats_or_power.insert(winner.clone(), 100);
    } else {
        // All zero scores but we still must choose deterministically; grant winner 100.
        seats_or_power.insert(winner.clone(), 100);
    }

    Ok(Allocation {
        unit_id: scores.unit_id.clone(),
        seats_or_power,
        last_seat_tie: tied.len() > 1,
    })
}

/// Scan scores in canonical option order and return (max_score, tied_ids_at_max).
fn top_by_score(scores: &UnitScores, options: &[OptionItem]) -> (u64, Vec<OptionId>) {
    let mut max_val: u64 = 0;
    let mut tied: Vec<OptionId> = Vec::new();

    for opt in options {
        let v = *scores.scores.get(&opt.option_id).unwrap_or(&0);
        if v > max_val {
            max_val = v;
            tied.clear();
            tied.push(opt.option_id.clone());
        } else if v == max_val {
            tied.push(opt.option_id.clone());
        }
    }
    (max_val, tied)
}

/// Resolve a tie among `tied` option IDs per policy.
/// If StatusQuo cannot be determined from `options`, falls back to DeterministicOrder.
/// If Random is requested but `rng` is None, falls back to DeterministicOrder.
fn break_tie_wta(
    tied: &[OptionId],
    options: &[OptionItem],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> OptionId {
    match tie_policy {
        TiePolicy::StatusQuo => {
            // No `is_status_quo` flag in OptionItem in vm_core; fall back to deterministic order.
            pick_deterministic(tied, options)
        }
        TiePolicy::DeterministicOrder => pick_deterministic(tied, options),
        TiePolicy::Random => {
            if let Some(rng) = rng {
                // Build index lookups into `tied` by canonical order to keep behavior stable.
                // Draw uniformly in [0, tied.len()) via rejection sampling.
                let n = tied.len() as u64;
                if n == 0 {
                    // Should not happen; fall back deterministically.
                    return pick_deterministic(tied, options);
                }
                let idx = rng.gen_range(n).unwrap_or(0) as usize;
                tied[idx].clone()
            } else {
                // No RNG supplied → deterministic fallback.
                pick_deterministic(tied, options)
            }
        }
    }
}

/// Deterministic tie-break: choose the earliest by canonical option order `(order_index, option_id)`.
fn pick_deterministic(tied: &[OptionId], options: &[OptionItem]) -> OptionId {
    // Walk options in canonical order; return the first that is in `tied`.
    let tied_set: BTreeSet<&OptionId> = tied.iter().collect();
    for opt in options {
        if tied_set.contains(&opt.option_id) {
            return opt.option_id.clone();
        }
    }
    // Should never happen (tied must be subset of options). Fall back to lexicographic min.
    tied.iter()
        .min()
        .cloned()
        .unwrap_or_else(|| OptionId::from("UNKNOWN"))
}
