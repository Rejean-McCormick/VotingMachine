//! D’Hondt (highest averages) allocation per unit.
//!
//! Contract (Doc 1 / Annex A aligned):
//! - Apply entry threshold on the *natural* totals (plurality votes, approvals, score sums).
//! - Allocate `seats` sequentially by picking max of v/(s+1).
//! - Ties resolved by policy: StatusQuo → fallback deterministic (no SQ flag in core),
//!   DeterministicOrder → canonical `(order_index, option_id)`, Random → seeded `TieRng`.
//! - Pure integers; no division in comparisons (cross-multiply in u128).
//!
//! Determinism:
//! - Iteration/scans run in canonical option order (the `options` slice).
//! - Random ties depend *only* on the provided `TieRng` stream.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use vm_core::{
    entities::OptionItem,
    ids::OptionId,
    rng::TieRng,
    variables::TiePolicy,
};

#[derive(Debug)]
pub enum AllocError {
    /// After threshold filtering, no options remain eligible while seats > 0.
    NoEligibleOptions,
    /// Policy was Random but no RNG was supplied (and seats > 0).
    MissingRngForRandomPolicy,
}

/// Allocate seats using D’Hondt (highest averages) method.
///
/// *Notes*:
/// - If `seats == 0`, returns an empty map.
/// - Threshold is applied against the sum of provided `scores`.
/// - Keys not present in `scores` are treated as 0 (they rarely pass a positive threshold).
pub fn allocate_dhondt(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem], // canonical (order_index, id)
    threshold_pct: u8,
    tie_policy: TiePolicy,
    mut rng: Option<&mut TieRng>,
) -> Result<BTreeMap<OptionId, u32>, AllocError> {
    if seats == 0 {
        return Ok(BTreeMap::new());
    }
    if matches!(tie_policy, TiePolicy::Random) && rng.is_none() {
        return Err(AllocError::MissingRngForRandomPolicy);
    }

    // 1) Threshold on natural totals.
    let eligible_scores = filter_by_threshold(scores, threshold_pct);

    // 2) Establish eligible IDs in *canonical option order*.
    let mut eligible_order: Vec<OptionId> = Vec::new();
    for o in options {
        if eligible_scores.contains_key(&o.option_id) {
            eligible_order.push(o.option_id.clone());
        }
    }

    if eligible_order.is_empty() {
        // No eligible options appear in canonical list → cannot allocate.
        return Err(AllocError::NoEligibleOptions);
    }

    // 3) Initialize seat vector for eligible options only (preserve determinism).
    let mut alloc: BTreeMap<OptionId, u32> =
        eligible_order.iter().cloned().map(|id| (id, 0)).collect();

    // 4) Sequentially award seats using D’Hondt quotients.
    for _round in 0..seats {
        let winner =
            next_award(&alloc, &eligible_scores, &eligible_order, tie_policy, rng.as_deref_mut());
        *alloc.get_mut(&winner).expect("winner must be in alloc") += 1;
    }

    Ok(alloc)
}

/// Filter by PR threshold using natural totals.
/// Keeps (opt, v) where 100 * v >= threshold_pct * total.
fn filter_by_threshold(
    scores: &BTreeMap<OptionId, u64>,
    threshold_pct: u8,
) -> BTreeMap<OptionId, u64> {
    let total: u128 = scores.values().map(|&v| v as u128).sum();
    // Fast path: threshold 0 keeps all known keys (missing keys treated as 0 and excluded).
    if threshold_pct == 0 {
        return scores.clone();
    }
    let t = threshold_pct as u128;
    scores
        .iter()
        .filter_map(|(k, &v)| {
            let v128 = v as u128;
            // Keep if share >= threshold (cross-multiplied; integer math only).
            if v128.saturating_mul(100) >= t.saturating_mul(total) {
                Some((k.clone(), v))
            } else {
                None
            }
        })
        .collect()
}

/// Choose argmax of v/(s+1) across eligible; ties resolved per policy.
/// `eligible_order` MUST be in canonical option order.
fn next_award(
    seats_so_far: &BTreeMap<OptionId, u32>,
    eligible_scores: &BTreeMap<OptionId, u64>,
    eligible_order: &[OptionId],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> OptionId {
    // Track the current best quotient and IDs (encounter order == canonical order).
    let mut best_ids: Vec<OptionId> = Vec::new();
    let mut best_v: u64 = 0;
    let mut best_s: u32 = 0;
    let mut have_best = false;

    for id in eligible_order {
        let v = *eligible_scores
            .get(id)
            .expect("eligible_order implies presence in eligible_scores");
        let s = *seats_so_far.get(id).unwrap_or(&0);

        if !have_best {
            have_best = true;
            best_v = v;
            best_s = s;
            best_ids.clear();
            best_ids.push(id.clone());
        } else {
            match cmp_quotients(v, s, best_v, best_s) {
                core::cmp::Ordering::Greater => {
                    best_v = v;
                    best_s = s;
                    best_ids.clear();
                    best_ids.push(id.clone());
                }
                core::cmp::Ordering::Equal => {
                    best_ids.push(id.clone());
                }
                core::cmp::Ordering::Less => {
                    // keep current best
                }
            }
        }
    }

    debug_assert!(
        !best_ids.is_empty(),
        "eligible_order must contain at least one candidate"
    );

    if best_ids.len() == 1 {
        return best_ids[0].clone();
    }

    // Resolve tie per policy.
    match tie_policy {
        TiePolicy::StatusQuo | TiePolicy::DeterministicOrder => {
            // First in canonical order (best_ids is built in canonical order).
            best_ids[0].clone()
        }
        TiePolicy::Random => {
            let n = best_ids.len() as u64; // n >= 2 here
            let idx = rng
                .expect("rng must be provided for Random policy")
                .gen_range(n) as usize; // uniform in [0, n)
            best_ids[idx].clone()
        }
    }
}

/// Compare D’Hondt quotients v_a/(s_a+1) vs v_b/(s_b+1) without floats.
/// Returns Ordering::Greater if a's quotient is larger.
fn cmp_quotients(v_a: u64, s_a: u32, v_b: u64, s_b: u32) -> core::cmp::Ordering {
    // Compare v_a * (s_b+1) ? v_b * (s_a+1) in u128 to avoid overflow.
    let da = (s_a as u128) + 1;
    let db = (s_b as u128) + 1;
    let lhs = (v_a as u128) * db;
    let rhs = (v_b as u128) * da;
    lhs.cmp(&rhs)
}
