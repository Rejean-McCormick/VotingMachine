//! Largest Remainder (LR) allocation per unit with selectable quota
//! (Hare, Droop, Imperiali), after applying an entry threshold.
//!
//! Contract (Doc 1 / Annex A aligned):
//! - Threshold is applied to natural totals (plurality votes, approvals, score sums).
//! - Quota kinds:
//!     * Hare:      floor(V / m)
//!     * Droop:     floor(V / (m + 1)) + 1
//!     * Imperiali: floor(V / (m + 2))
//! - Floors are v_i / q (integer div); remainders are v_i % q.
//! - If q == 0 (tiny totals), floors are 0 and remainders are raw scores.
//! - If sum_floors < seats → distribute leftovers by largest remainder
//!   (tie keys: remainder ↓, raw score ↓, then canonical (order_index, id);
//!   StatusQuo policy falls back to deterministic due to no SQ flag here).
//! - If sum_floors > seats (Imperiali edge) → trim from smallest remainder
//!   (inverse ordering; same tie-policy rules).
//!
//! Determinism:
//! - All scans follow canonical option order provided by `options`.
//! - Random ties depend *only* on the injected `TieRng` stream.

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    ids::OptionId,
    entities::OptionItem,
    rng::TieRng,
    variables::TiePolicy,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum QuotaKind {
    Hare,
    Droop,
    Imperiali,
}

#[derive(Debug)]
pub enum AllocError {
    /// After threshold filtering, no options remain eligible while seats > 0.
    NoEligibleOptions,
    /// Policy was Random but no RNG was supplied (and seats > 0).
    MissingRngForRandomPolicy,
}

/// Allocate seats using Largest Remainder with a selected quota.
///
/// Notes:
/// - If `seats == 0`, returns an empty map.
/// - Keys missing from `scores` are treated as 0 (they rarely pass a positive threshold).
pub fn allocate_largest_remainder(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem], // canonical (order_index, id)
    threshold_pct: u8,
    quota: QuotaKind,
    tie_policy: TiePolicy,
    mut rng: Option<&mut TieRng>,
) -> Result<BTreeMap<OptionId, u32>, AllocError> {
    if seats == 0 {
        return Ok(BTreeMap::new());
    }
    if matches!(tie_policy, TiePolicy::Random) && rng.is_none() {
        return Err(AllocError::MissingRngForRandomPolicy);
    }

    let eligible = filter_by_threshold(scores, threshold_pct);
    if eligible.is_empty() {
        return Err(AllocError::NoEligibleOptions);
    }

    let total: u128 = eligible.values().map(|&v| v as u128).sum();
    let q = compute_quota(total, seats as u128, quota);

    let (mut alloc, remainders) = floors_and_remainders(&eligible, q);

    // Sum floors (u128 to avoid intermediate growth) and compare with seats.
    let sum_floors: u128 = alloc.values().map(|&s| s as u128).sum();

    if sum_floors < seats as u128 {
        let needed = (seats as u128 - sum_floors) as u32;
        distribute_leftovers(
            needed,
            &mut alloc,
            &remainders,
            &eligible,
            options,
            tie_policy,
            rng.as_deref_mut(),
        );
    } else if sum_floors > seats as u128 {
        // Imperiali (or degenerate q) may over-allocate in floors.
        trim_over_allocation_if_needed(
            seats,
            &mut alloc,
            &remainders,
            &eligible,
            options,
            tie_policy,
            rng.as_deref_mut(),
        );
    }

    debug_assert_eq!(
        alloc.values().map(|&s| s as u128).sum::<u128>(),
        seats as u128
    );
    Ok(alloc)
}

/// Keep options whose natural share meets threshold: 100*v >= threshold_pct*total (u128 math).
fn filter_by_threshold(
    scores: &BTreeMap<OptionId, u64>,
    threshold_pct: u8,
) -> BTreeMap<OptionId, u64> {
    let total: u128 = scores.values().map(|&v| v as u128).sum();
    if threshold_pct == 0 {
        return scores.clone();
    }
    let t = threshold_pct as u128;
    scores
        .iter()
        .filter_map(|(k, &v)| {
            let v128 = v as u128;
            if v128.saturating_mul(100) >= t.saturating_mul(total) {
                Some((k.clone(), v))
            } else {
                None
            }
        })
        .collect()
}

/// Integer-only quota.
/// Hare: floor(V / m)
/// Droop: floor(V / (m + 1)) + 1
/// Imperiali: floor(V / (m + 2))
fn compute_quota(total: u128, seats: u128, quota: QuotaKind) -> u128 {
    match quota {
        QuotaKind::Hare => {
            if seats == 0 { 0 } else { total / seats }
        }
        QuotaKind::Droop => {
            // floor(V/(m+1)) + 1 ; seats > 0 in caller
            (total / (seats + 1)) + 1
        }
        QuotaKind::Imperiali => {
            // floor(V/(m+2))
            total / (seats + 2)
        }
    }
}

/// Compute floors and remainders given quota q (u128 math; q==0 handled).
fn floors_and_remainders(
    eligible: &BTreeMap<OptionId, u64>,
    q: u128,
) -> (BTreeMap<OptionId, u32>, BTreeMap<OptionId, u128>) {
    let mut floors: BTreeMap<OptionId, u32> = BTreeMap::new();
    let mut rems: BTreeMap<OptionId, u128> = BTreeMap::new();

    for (id, &v) in eligible {
        let v128 = v as u128;
        if q == 0 {
            floors.insert(id.clone(), 0);
            rems.insert(id.clone(), v128);
        } else {
            let f128 = v128 / q;
            // Saturate to u32::MAX; in practice seats bound this far below.
            let f = if f128 > (u32::MAX as u128) {
                u32::MAX
            } else {
                f128 as u32
            };
            let r = v128 % q;
            floors.insert(id.clone(), f);
            rems.insert(id.clone(), r);
        }
    }

    (floors, rems)
}

/// Assign remaining seats by largest remainder; ties per policy
/// (remainder ↓, raw score ↓, canonical (order_index, id); StatusQuo→deterministic; Random uses rng).
fn distribute_leftovers(
    target_extra: u32,
    alloc: &mut BTreeMap<OptionId, u32>,
    remainders: &BTreeMap<OptionId, u128>,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    mut rng: Option<&mut TieRng>,
) {
    for _ in 0..target_extra {
        // Find max remainder among eligible keys.
        let mut max_r: Option<u128> = None;
        let mut max_ids: Vec<OptionId> = Vec::new();

        for opt in options {
            if let Some(&r) = remainders.get(&opt.option_id) {
                match max_r {
                    None => {
                        max_r = Some(r);
                        max_ids.clear();
                        max_ids.push(opt.option_id.clone());
                    }
                    Some(mr) => {
                        if r > mr {
                            max_r = Some(r);
                            max_ids.clear();
                            max_ids.push(opt.option_id.clone());
                        } else if r == mr {
                            max_ids.push(opt.option_id.clone());
                        }
                    }
                }
            }
        }

        let winner = if max_ids.len() <= 1 {
            max_ids[0].clone()
        } else {
            // Narrow by raw score desc
            let mut best_score: Option<u64> = None;
            let mut narrowed: Vec<OptionId> = Vec::new();
            for id in &max_ids {
                let sc = *scores.get(id).unwrap_or(&0);
                match best_score {
                    None => {
                        best_score = Some(sc);
                        narrowed.clear();
                        narrowed.push(id.clone());
                    }
                    Some(bs) => {
                        if sc > bs {
                            best_score = Some(sc);
                            narrowed.clear();
                            narrowed.push(id.clone());
                        } else if sc == bs {
                            narrowed.push(id.clone());
                        }
                    }
                }
            }
            if narrowed.len() <= 1 {
                narrowed[0].clone()
            } else {
                match tie_policy {
                    TiePolicy::StatusQuo => status_quo_pick(&narrowed, options),
                    TiePolicy::DeterministicOrder => deterministic_pick(&narrowed, options),
                    TiePolicy::Random => {
                        let n = narrowed.len() as u64;
                        let idx = rng
                            .as_deref_mut()
                            .expect("rng must be provided for Random policy")
                            .gen_range(n)
                            .unwrap_or(0) as usize;
                        narrowed[idx].clone()
                    }
                }
            }
        };

        *alloc.entry(winner).or_insert(0) += 1;
    }
}

/// Imperiali edge: if floors sum > target, trim from smallest remainder until sum == target.
/// Ties resolved using inverse of distribute ranking (remainder ↑, raw score ↑, then canonical),
/// with StatusQuo→deterministic; Random uses rng.
fn trim_over_allocation_if_needed(
    target_seats: u32,
    alloc: &mut BTreeMap<OptionId, u32>,
    remainders: &BTreeMap<OptionId, u128>,
    scores: &BTreeMap<OptionId, u64>,
    options: &[OptionItem],
    tie_policy: TiePolicy,
    mut rng: Option<&mut TieRng>,
) -> bool {
    let mut changed = false;

    let mut total: u128 = alloc.values().map(|&s| s as u128).sum();
    while total > target_seats as u128 {
        // Consider only options with at least one seat to trim.
        let mut min_r: Option<u128> = None;
        let mut cand_ids: Vec<OptionId> = Vec::new();

        for opt in options {
            if let Some(&s) = alloc.get(&opt.option_id) {
                if s == 0 {
                    continue;
                }
                let r = *remainders.get(&opt.option_id).unwrap_or(&0);
                match min_r {
                    None => {
                        min_r = Some(r);
                        cand_ids.clear();
                        cand_ids.push(opt.option_id.clone());
                    }
                    Some(mr) => {
                        if r < mr {
                            min_r = Some(r);
                            cand_ids.clear();
                            cand_ids.push(opt.option_id.clone());
                        } else if r == mr {
                            cand_ids.push(opt.option_id.clone());
                        }
                    }
                }
            }
        }

        // On tie, prefer smaller raw score (inverse of leftovers), then canonical order.
        let loser = if cand_ids.len() <= 1 {
            cand_ids[0].clone()
        } else {
            let mut best_score: Option<u64> = None; // but now "best" means *smallest*
            let mut narrowed: Vec<OptionId> = Vec::new();
            for id in &cand_ids {
                let sc = *scores.get(id).unwrap_or(&0);
                match best_score {
                    None => {
                        best_score = Some(sc);
                        narrowed.clear();
                        narrowed.push(id.clone());
                    }
                    Some(bs) => {
                        if sc < bs {
                            best_score = Some(sc);
                            narrowed.clear();
                            narrowed.push(id.clone());
                        } else if sc == bs {
                            narrowed.push(id.clone());
                        }
                    }
                }
            }
            if narrowed.len() <= 1 {
                narrowed[0].clone()
            } else {
                match tie_policy {
                    TiePolicy::StatusQuo => status_quo_pick(&narrowed, options),
                    TiePolicy::DeterministicOrder => deterministic_pick(&narrowed, options),
                    TiePolicy::Random => {
                        let n = narrowed.len() as u64;
                        let idx = rng
                            .as_deref_mut()
                            .expect("rng must be provided for Random policy")
                            .gen_range(n)
                            .unwrap_or(0) as usize;
                        narrowed[idx].clone()
                    }
                }
            }
        };

        if let Some(s) = alloc.get_mut(&loser) {
            *s -= 1;
            changed = true;
            total -= 1;
        } else {
            // Should not happen; defensive break to avoid infinite loop.
            break;
        }
    }

    changed
}

// ----------------- tie helpers -----------------

/// Deterministic pick: first in canonical `(order_index, option_id)` among `tied`.
fn deterministic_pick(tied: &[OptionId], options: &[OptionItem]) -> OptionId {
    let set: BTreeSet<&OptionId> = tied.iter().collect();
    for o in options {
        if set.contains(&o.option_id) {
            return o.option_id.clone();
        }
    }
    // Defensive fallback (should not occur given inputs).
    tied.iter().min().cloned().expect("non-empty tied")
}

/// Status-quo resolver: core `OptionItem` has no SQ flag, so fall back to deterministic.
fn status_quo_pick(tied: &[OptionId], options: &[OptionItem]) -> OptionId {
    deterministic_pick(tied, options)
}
