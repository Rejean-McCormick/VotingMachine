//! Sainte-Laguë (highest averages with odd divisors) allocation per unit.
//!
//! Contract (Doc 1 / Annex A aligned):
//! - Apply entry threshold on the natural totals (plurality votes, approvals, score sums).
//! - Allocate `seats` sequentially by picking max of v / (2*s + 1).
//! - Ties resolved by policy: StatusQuo → fallback deterministic (no SQ flag in core),
//!   DeterministicOrder → canonical `(order_index, option_id)`,
//!   Random → seeded `TieRng` (consume exactly k draws for a k-way tie; pick min by (draw, option_id)).
//! - Pure integers; no division in comparisons (cross-multiply in u128).
//!
//! Determinism:
//! - Scans iterate in canonical option order (the `options` slice).
//! - Random ties depend *only* on the provided `TieRng` stream; no draws if no tie.

#![forbid(unsafe_code)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
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
}

/// Allocate seats using Sainte-Laguë (odd divisors 1,3,5,…).
///
/// Notes:
/// - If `seats == 0`, returns an empty map.
/// - Threshold is applied against the sum of provided `scores`.
/// - Keys not present in `scores` are treated as 0; unknown IDs (not in `options`) are ignored.
pub fn allocate_sainte_lague(
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

    // Apply threshold and intersect with registry options.
    let eligible_scores = filter_by_threshold(scores, options, threshold_pct);

    if eligible_scores.is_empty() {
        return Err(AllocError::NoEligibleOptions);
    }

    // Initialize seat vector for eligible options only.
    let mut alloc: BTreeMap<OptionId, u32> =
        eligible_scores.keys().cloned().map(|id| (id, 0)).collect();

    // Award seats sequentially.
    for _round in 0..seats {
        let winner = next_award(
            &alloc,
            &eligible_scores,
            options,
            tie_policy,
            rng.as_deref_mut(),
        );
        *alloc.get_mut(&winner).expect("winner must be in alloc") += 1;
    }

    Ok(alloc)
}

/// Keep options whose natural share meets the threshold: 100*v >= threshold_pct*total (u128 math),
/// and that are present in the registry `options`.
/// If `total == 0`, no one qualifies (prevents allocating seats by tie-break alone).
fn filter_by_threshold(
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    threshold_pct: u8,
) -> BTreeMap<OptionId, u64> {
    // Build membership set of valid (registry) options.
    let allowed: BTreeSet<OptionId> = options.iter().map(|o| o.option_id.clone()).collect();

    let total: u128 = scores
        .iter()
        .filter(|(k, _)| allowed.contains(*k))
        .map(|(_, &v)| v as u128)
        .sum();

    if total == 0 {
        return BTreeMap::new();
    }

    if threshold_pct == 0 {
        // Intersect with registry; unknown IDs are ignored.
        return allowed
            .into_iter()
            .map(|k| {
                let v = *scores.get(&k).unwrap_or(&0);
                (k, v)
            })
            .collect();
    }

    let t = threshold_pct as u128;
    allowed
        .into_iter()
        .filter_map(|k| {
            let v = *scores.get(&k).unwrap_or(&0);
            let v128 = v as u128;
            // 100*v >= t*total  (all u128 to avoid overflow)
            if v128.saturating_mul(100) >= t.saturating_mul(total) {
                Some((k, v))
            } else {
                None
            }
        })
        .collect()
}

/// Argmax of Sainte-Laguë quotients v / (2*s + 1); ties per policy.
fn next_award(
    seats_so_far: &BTreeMap<OptionId, u32>,
    eligible_scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    rng: Option<&mut TieRng>,
) -> OptionId {
    // Track the current best quotient and tied IDs in canonical order.
    let mut best_ids: Vec<OptionId> = Vec::new();
    let mut best_v: u64 = 0;
    let mut best_s: u32 = 0;
    let mut have_best = false;

    for opt in options {
        if let Some(&v) = eligible_scores.get(&opt.option_id) {
            let s = *seats_so_far.get(&opt.option_id).unwrap_or(&0);
            if !have_best {
                have_best = true;
                best_v = v;
                best_s = s;
                best_ids.clear();
                best_ids.push(opt.option_id.clone());
            } else {
                match cmp_quotients(v, s, best_v, best_s) {
                    core::cmp::Ordering::Greater => {
                        best_v = v;
                        best_s = s;
                        best_ids.clear();
                        best_ids.push(opt.option_id.clone());
                    }
                    core::cmp::Ordering::Equal => {
                        best_ids.push(opt.option_id.clone());
                    }
                    core::cmp::Ordering::Less => {} // keep current best
                }
            }
        }
    }

    if best_ids.len() <= 1 {
        return best_ids
            .into_iter()
            .next()
            // Practically unreachable if caller checked eligibility; still pick first canonical option deterministically.
            .unwrap_or_else(|| options.first().expect("options cannot be empty").option_id.clone());
    }

    // Resolve tie per policy.
    match tie_policy {
        TiePolicy::StatusQuo => deterministic_pick(&best_ids, options), // no SQ flag → deterministic
        TiePolicy::DeterministicOrder => deterministic_pick(&best_ids, options),
        TiePolicy::Random => {
            if let Some(mut rng) = rng {
                // Consume exactly k draws for a k-way tie; winner is min by (draw, option_id).
                let mut best: Option<(u64, &OptionId)> = None;
                for oid in &best_ids {
                    let ticket = rng.gen_range(u64::MAX).unwrap_or(0);
                    match best {
                        None => best = Some((ticket, oid)),
                        Some((b_ticket, b_oid)) => {
                            if (ticket, oid) < (b_ticket, b_oid) {
                                best = Some((ticket, oid));
                            }
                        }
                    }
                }
                best.map(|(_, oid)| oid.clone())
                    .unwrap_or_else(|| deterministic_pick(&best_ids, options))
            } else {
                // No RNG supplied → deterministic fallback, but no error.
                deterministic_pick(&best_ids, options)
            }
        }
    }
}

/// Compare quotients q_a = v_a / (2*s_a+1) vs q_b = v_b / (2*s_b+1) using u128 cross-multiplication.
fn cmp_quotients(v_a: u64, s_a: u32, v_b: u64, s_b: u32) -> core::cmp::Ordering {
    // Compare v_a * (2*s_b+1) ? v_b * (2*s_a+1) in u128 to avoid overflow.
    let da = (2u128 * (s_a as u128)) + 1;
    let db = (2u128 * (s_b as u128)) + 1;
    let lhs = (v_a as u128) * db;
    let rhs = (v_b as u128) * da;
    lhs.cmp(&rhs)
}

/// Deterministic fallback: choose the earliest by canonical option order `(order_index, option_id)`.
fn deterministic_pick(tied: &[OptionId], options: &[OptionItem]) -> OptionId {
    let set: BTreeSet<OptionId> = tied.iter().cloned().collect();
    for o in options {
        if set.contains(&o.option_id) {
            return o.option_id.clone();
        }
    }
    // Fallback: pick first canonical option (should not be needed).
    options.first().expect("options cannot be empty").option_id.clone()
}
