//! Largest Remainder (LR) allocation per unit with selectable quota
//! (Hare, Droop, Imperiali).
//!
//! Contract (Doc 4 / Doc 5 aligned):
//! - Thresholding happens upstream; this function assumes `scores` already filtered.
//! - Quota kinds:
//!     * Hare:      floor(V / m)
//!     * Droop:     floor(V / (m + 1)) + 1
//!     * Imperiali: floor(V / (m + 2))
//! - Floors are v_i / q (integer div); remainders are v_i % q.
//! - If q == 0 (tiny totals), floors are 0 and remainders are raw scores.
//! - If sum_floors < seats → distribute leftovers by largest remainder
//!   (tie keys: remainder ↓, raw score ↓, then canonical order).
//! - If sum_floors > seats (Imperiali edge) → trim from smallest remainder
//!   (remainder ↑, raw score ↑, then canonical order).
//!
//! Determinism:
//! - No RNG or policy here (S4); deterministic tie-breaking only.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use vm_core::ids::OptionId;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum QuotaKind {
    Hare,
    Droop,
    Imperiali,
}

#[derive(Debug)]
pub enum AllocError {
    /// No eligible options (empty `scores`) while seats > 0.
    NoEligibleOptions,
}

/// Public API expected by the pipeline (kept stable).
/// Breaks LR ties by: remainder ↓, raw score ↓, then `OptionId` ascending.
/// If canonical order differs from `OptionId`, prefer the `*_with_order` variant below.
pub fn allocate_largest_remainder(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    quota: QuotaKind,
) -> Result<BTreeMap<OptionId, u32>, AllocError> {
    allocate_largest_remainder_with_order(seats, scores, quota, None)
}

/// Variant that accepts a canonical order (slice of OptionIds). When provided,
/// ties use that order as the final key (ascending index in the slice).
pub fn allocate_largest_remainder_with_order(
    seats: u32,
    scores: &BTreeMap<OptionId, u64>,
    quota: QuotaKind,
    canonical_order: Option<&[OptionId]>,
) -> Result<BTreeMap<OptionId, u32>, AllocError> {
    // Trivial cases
    if seats == 0 {
        return Ok(BTreeMap::new());
    }
    if scores.is_empty() {
        return Err(AllocError::NoEligibleOptions);
    }

    let total: u128 = scores.values().map(|&v| v as u128).sum();
    let q = compute_quota(total, seats as u128, quota);

    let (mut alloc, remainders) = floors_and_remainders(scores, q);

    // Sum floors and compare to target seats.
    let sum_floors: u128 = alloc.values().map(|&s| s as u128).sum();

    if sum_floors < seats as u128 {
        // Assign remaining seats by static LR ranking (deterministic).
        let needed = (seats as u128 - sum_floors) as u32;
        distribute_leftovers(needed, &mut alloc, &remainders, scores, canonical_order);
    } else if sum_floors > seats as u128 {
        // Imperiali edge (or degenerate quota): remove seats based on inverse LR ranking.
        trim_over_allocation(seats, &mut alloc, &remainders, scores, canonical_order);
    }

    // Always recompute final sum to avoid stale assertions.
    let sum: u128 = alloc.values().map(|&s| s as u128).sum();
    debug_assert_eq!(sum, seats as u128);
    Ok(alloc)
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
            // floor(V/(m+1)) + 1 ; m>0 guaranteed by caller
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
    scores: &BTreeMap<OptionId, u64>,
    q: u128,
) -> (BTreeMap<OptionId, u32>, BTreeMap<OptionId, u128>) {
    let mut floors: BTreeMap<OptionId, u32> = BTreeMap::new();
    let mut rems: BTreeMap<OptionId, u128> = BTreeMap::new();

    for (id, &v) in scores.iter() {
        let v128 = v as u128;
        if q == 0 {
            floors.insert(id.clone(), 0);
            rems.insert(id.clone(), v128);
        } else {
            let f128 = v128 / q;
            // Saturate to u32::MAX; in practice seats bound this far below.
            let f = if f128 > (u32::MAX as u128) { u32::MAX } else { f128 as u32 };
            let r = v128 % q;
            floors.insert(id.clone(), f);
            rems.insert(id.clone(), r);
        }
    }

    (floors, rems)
}

/// Assign `target_extra` seats by largest remainder (deterministic ranking):
/// remainder ↓, raw score ↓, then canonical order (if provided) else `OptionId` ↑.
fn distribute_leftovers(
    target_extra: u32,
    alloc: &mut BTreeMap<OptionId, u32>,
    remainders: &BTreeMap<OptionId, u128>,
    scores: &BTreeMap<OptionId, u64>,
    canonical_order: Option<&[OptionId]>,
) {
    if target_extra == 0 || remainders.is_empty() {
        return;
    }

    // Build canonical index lookup if provided.
    let order_ix: BTreeMap<OptionId, usize> = match canonical_order {
        Some(slice) => slice
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect(),
        None => BTreeMap::new(),
    };

    // Build a stable ranking once; reuse cyclically if target_extra > candidates.
    let mut ranking: Vec<(OptionId, u128, u64, usize)> = remainders
        .iter()
        .map(|(id, &r)| {
            let sc = *scores.get(id).unwrap_or(&0);
            let ix = order_ix.get(id).cloned().unwrap_or(usize::MAX);
            (id.clone(), r, sc, ix)
        })
        .collect();

    ranking.sort_by(|a, b| {
        // r desc, score desc, canonical asc (or OptionId asc if no canonical index)
        b.1.cmp(&a.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| {
                match (a.3, b.3) {
                    (usize::MAX, usize::MAX) => a.0.cmp(&b.0), // fall back to OptionId
                    (ia, ib) => ia.cmp(&ib),
                }
            })
    });

    if ranking.is_empty() {
        return;
    }

    let n = ranking.len();
    let mut given = 0u32;
    let mut idx = 0usize;

    while given < target_extra {
        let (ref id, _, _, _) = ranking[idx];
        *alloc.entry(id.clone()).or_insert(0) += 1;
        given += 1;
        idx += 1;
        if idx == n {
            idx = 0; // cycle if more seats than candidates (degenerate quotas)
        }
    }
}

/// Trim seats when floors over-allocate (Imperiali edge) using inverse LR ranking:
/// remainder ↑, raw score ↑, then canonical order (if provided) else `OptionId` ↑.
fn trim_over_allocation(
    target_seats: u32,
    alloc: &mut BTreeMap<OptionId, u32>,
    remainders: &BTreeMap<OptionId, u128>,
    scores: &BTreeMap<OptionId, u64>,
    canonical_order: Option<&[OptionId]>,
) {
    let mut total: u128 = alloc.values().map(|&s| s as u128).sum();
    if total <= target_seats as u128 {
        return;
    }

    let order_ix: BTreeMap<OptionId, usize> = match canonical_order {
        Some(slice) => slice
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect(),
        None => BTreeMap::new(),
    };

    // Consider only options with at least one seat.
    let mut ranking: Vec<(OptionId, u128, u64, usize)> = alloc
        .iter()
        .filter_map(|(id, &s)| (s > 0).then(|| {
            let r = *remainders.get(id).unwrap_or(&0);
            let sc = *scores.get(id).unwrap_or(&0);
            let ix = order_ix.get(id).cloned().unwrap_or(usize::MAX);
            (id.clone(), r, sc, ix)
        }))
        .collect();

    ranking.sort_by(|a, b| {
        // r asc, score asc, canonical asc (or OptionId asc if no canonical index)
        a.1.cmp(&b.1)
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| {
                match (a.3, b.3) {
                    (usize::MAX, usize::MAX) => a.0.cmp(&b.0), // fall back to OptionId
                    (ia, ib) => ia.cmp(&ib),
                }
            })
    });

    if ranking.is_empty() {
        return;
    }

    let mut idx = 0usize;
    while total > target_seats as u128 {
        let (ref id, _, _, _) = ranking[idx];
        if let Some(s) = alloc.get_mut(id) {
            if *s > 0 {
                *s -= 1;
                total -= 1;
            }
        }
        idx += 1;
        if idx == ranking.len() {
            idx = 0; // cycle defensively; should rarely happen
        }
    }
}
