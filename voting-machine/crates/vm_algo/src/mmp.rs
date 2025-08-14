// crates/vm_algo/src/allocation/mmp.rs
//
// Part 1/3 — module header, imports, types, core helpers (no_std-ready, deterministic)
//
// Mixed-Member Proportional (MMP) helpers:
// - compute total seats from a top-up share (VM-VAR-013) with half-even rounding,
// - (in Part 2) apportion intended targets using highest-averages (D’Hondt / Sainte-Laguë),
// - (in Part 3) compute top-ups vs local seats and apply overhang policy/model.
//
// Determinism & policy:
// - No RNG anywhere in MMP.
// - All tie-breaks are fully deterministic; later parts will use either registry order
//   (order_index) or OptionId ordering as a stable, documented fallback.
// - Integer-first arithmetic; no float division. Cross-multiply for quotient comparisons.
//
// NOTE: Adjust crate paths if your tree differs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use vm_core::{
    ids::OptionId,
    rounding::round_nearest_even_int,
    variables::{AllocationMethod, OverhangPolicy, TotalSeatsModel},
};

/// Result bundle for an MMP correction step.
/// (Produced in Part 3.)
#[derive(Debug, Clone)]
pub struct MmpOutcome {
    pub targets: BTreeMap<OptionId, u32>,
    pub topups: BTreeMap<OptionId, u32>,
    pub finals: BTreeMap<OptionId, u32>,
    pub effective_total_seats: u32,
    pub overhang_by_option: BTreeMap<OptionId, u32>,
}

/// Errors specific to the MMP helpers.
/// (Used by helpers in this module; public so callers can inspect failures.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MmpError {
    /// `topup_share_pct` must be < 100 (and typically ≤ 60 by upstream policy).
    InvalidTopupSharePct { pct: u8 },
    /// Half-even rounding failed (should not happen with positive denominator).
    RoundingError { num: i128, den: i128 },
    /// Growth model cannot satisfy locals given vote_totals (see Part 3).
    NoViableApportionment { option_id: OptionId },
}

/// Compute intended total seats `T` from local seats `L` and top-up share `p%`
/// using banker's (half-even) rounding on `T = (L*100) / (100 - p)`.
///
/// Domain: upstream typically enforces `p ≤ 60` and strictly `< 100`.
/// Returns a **Result** to avoid lossy sentinels.
pub fn compute_total_from_share_result(local_total: u32, topup_share_pct: u8) -> Result<u32, MmpError> {
    if topup_share_pct >= 100 {
        return Err(MmpError::InvalidTopupSharePct { pct: topup_share_pct });
    }
    let num: i128 = (local_total as i128) * 100;
    let den: i128 = i128::from(100u8.saturating_sub(topup_share_pct)).max(1);
    match round_nearest_even_int(num, den) {
        Ok(v) if v >= 0 && v <= (u32::MAX as i128) => Ok(v as u32),
        Ok(_) => Ok(u32::MAX), // saturate up: extremely large houses clamp to u32::MAX
        Err(_) => Err(MmpError::RoundingError { num, den }),
    }
}

/// Back-compat wrapper that returns `0` on error (deprecated).
/// Prefer `compute_total_from_share_result`.
#[inline]
pub fn compute_total_from_share(local_total: u32, topup_share_pct: u8) -> u32 {
    match compute_total_from_share_result(local_total, topup_share_pct) {
        Ok(v) => v,
        Err(_e) => 0,
    }
}

/// Highest-averages divisor for the chosen method given `s` seats already assigned.
///
/// - D’Hondt: d(s) = s + 1
/// - Sainte-Laguë (plain): d(s) = 2*s + 1
/// (If you support "modified Sainte-Laguë", adapt the s==0 divisor externally.)
#[inline]
pub fn divisor(method: AllocationMethod, s_assigned: u32) -> u128 {
    match method {
        AllocationMethod::DHondt => u128::from(s_assigned) + 1,
        AllocationMethod::SainteLague => (u128::from(s_assigned) << 1) + 1,
        // Add other methods here if vm_core defines them; keep ≥ 1 always.
    }
}

/// Compare two highest-averages quotients v1/d1 vs v2/d2 via cross multiplication.
/// Returns `core::cmp::Ordering`.
#[inline]
pub fn cmp_quotients(v1: u64, d1: u128, v2: u64, d2: u128) -> core::cmp::Ordering {
    use core::cmp::Ordering;
    // d1, d2 must be ≥ 1 by construction.
    let left = (v1 as u128).saturating_mul(d2);
    let right = (v2 as u128).saturating_mul(d1);
    if left < right {
        Ordering::Less
    } else if left > right {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// Build the union set of option IDs present across inputs (stable set).
#[inline]
pub fn union_option_ids(
    a: &BTreeMap<OptionId, impl Copy>,
    b: &BTreeMap<OptionId, impl Copy>,
    c: &BTreeMap<OptionId, impl Copy>,
) -> BTreeSet<OptionId> {
    let mut set = BTreeSet::new();
    for k in a.keys() {
        set.insert(*k);
    }
    for k in b.keys() {
        set.insert(*k);
    }
    for k in c.keys() {
        set.insert(*k);
    }
    set
}

/// Seed a map for all options in `ids` with an initial value.
#[inline]
pub fn seed_map_u32(ids: &BTreeSet<OptionId>, init: u32) -> BTreeMap<OptionId, u32> {
    ids.iter().copied().map(|k| (k, init)).collect()
}

/// Sum values in a map (u32 → u128 accumulator).
#[inline]
pub fn sum_map_u32(map: &BTreeMap<OptionId, u32>) => u128 {
    map.values().fold(0u128, |acc, &v| acc + (v as u128))
}
// crates/vm_algo/src/allocation/mmp.rs
//
// Part 2/3 — apportionment (highest-averages), registry-order aware, no_std-ready.
//
// Implements:
// - build_order_index for deterministic tie-breaks (registry order, then OptionId)
// - apportion_targets: seats → parties from votes using D’Hondt / Sainte-Laguë
//   * seeds all registry options (zero-vote parties kept in map)
//   * seat-by-seat assignment with cross-multiply comparison (integer-only)
//   * exact ties: lower registry order_index first, then lower OptionId
//   * zero-vote corner case: even round-robin by registry order (deterministic)
//
// NOTE: Adjust types/paths in Part 1 if your repo differs.

use core::cmp::Ordering;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::allocation::mmp::{
    divisor, cmp_quotients, MmpError
};
use vm_core::ids::OptionId;
use vm_core::variables::AllocationMethod;

/// Build registry order index: `OptionId -> order_index`.
/// Duplicate options are treated as a logic error (debug-asserted); the first
/// occurrence wins deterministically in release builds.
#[inline]
pub fn build_order_index(options: &[OptionId]) -> BTreeMap<OptionId, usize> {
    let mut seen = BTreeSet::new();
    let mut idx = BTreeMap::new();
    for (i, &opt) in options.iter().enumerate() {
        let first = seen.insert(opt);
        debug_assert!(first, "Duplicate OptionId in registry slice: {:?}", opt);
        // First occurrence wins deterministically; repeated inserts overwrite with same i anyway.
        idx.insert(opt, i);
    }
    idx
}

/// Evenly distribute `total_seats` in a deterministic round-robin by registry order.
/// Used only when the entire vote vector sums to zero.
fn round_robin_even_distribution(
    total_seats: u32,
    options: &[OptionId],
) -> BTreeMap<OptionId, u32> {
    let n = options.len() as u32;
    let mut out: BTreeMap<OptionId, u32> = options.iter().copied().map(|k| (k, 0)).collect();
    if n == 0 || total_seats == 0 {
        return out;
    }
    // Base quota and remainder
    let base = total_seats / n;
    let rem  = total_seats % n;

    // Everyone gets the base
    for &opt in options.iter() {
        *out.get_mut(&opt).unwrap() = base;
    }
    // First `rem` options in registry order get one extra, deterministically
    for opt in options.iter().take(rem as usize) {
        if let Some(v) = out.get_mut(opt) {
            *v = v.saturating_add(1);
        }
    }
    out
}

/// Apportion `total_seats` to `options` from `vote_totals` via highest-averages.
/// Deterministic, integer-only; tie-breaks: higher quotient → lower registry order → lower OptionId.
///
/// - All `options` are seeded in the output (zero-vote parties are retained).
/// - If the sum of votes is zero, seats are distributed evenly by registry order.
/// - For method divisors, see `divisor` (Part 1).
pub fn apportion_targets(
    total_seats: u32,
    options: &[OptionId],
    vote_totals: &BTreeMap<OptionId, u64>,
    method: AllocationMethod,
) -> BTreeMap<OptionId, u32> {
    // Seed output with all registry options
    let mut alloc: BTreeMap<OptionId, u32> = options.iter().copied().map(|k| (k, 0u32)).collect();
    if total_seats == 0 || options.is_empty() {
        return alloc;
    }

    // Sum votes (u128 accumulator) to detect all-zero corner case
    let mut sum_votes: u128 = 0;
    for &opt in options.iter() {
        sum_votes = sum_votes.saturating_add(u128::from(*vote_totals.get(&opt).unwrap_or(&0)));
    }
    if sum_votes == 0 {
        return round_robin_even_distribution(total_seats, options);
    }

    // Build order index for tie-breaks
    let order_index = build_order_index(options);

    // Seat-by-seat assignment
    for _seat in 0..total_seats {
        let mut best: Option<(OptionId, u64, u128, usize)> = None;
        for &opt in options.iter() {
            let v = *vote_totals.get(&opt).unwrap_or(&0);
            let s = *alloc.get(&opt).unwrap_or(&0);
            let d = divisor(method, s);

            match best {
                None => best = Some((opt, v, d, *order_index.get(&opt).unwrap())),
                Some((b_opt, b_v, b_d, b_ord)) => {
                    // Compare quotients v/d vs b_v/b_d
                    match cmp_quotients(v, d, b_v, b_d) {
                        Ordering::Greater => best = Some((opt, v, d, *order_index.get(&opt).unwrap())),
                        Ordering::Less => { /* keep best */ }
                        Ordering::Equal => {
                            // Tie: lower registry order wins; then lower OptionId
                            let ord = *order_index.get(&opt).unwrap();
                            if ord < b_ord || (ord == b_ord && opt < b_opt) {
                                best = Some((opt, v, d, ord));
                            }
                        }
                    }
                }
            }
        }

        if let Some((winner, _, _, _)) = best {
            if let Some(x) = alloc.get_mut(&winner) {
                *x = x.saturating_add(1);
            }
        } else {
            // No candidate (should be unreachable since options is non-empty)
            break;
        }
    }

    alloc
}
// crates/vm_algo/src/allocation/mmp.rs
//
// Part 3/3 — top-ups & overhang policies (deterministic, no_std-ready)
//
// Implements:
// - compute_topups_and_apply_overhang (three policies):
//   * AllowOverhang: accept overhang; finals = locals + deficits; T_eff = T + Σ overhang
//   * CompensateOthers: fixed house (T); distribute pool = T - Σ locals to non-overhang parties
//   * AddTotalSeats: expand house size minimally so that apportionment satisfies seats ≥ locals
//
// Notes:
// - Uses registry `options` slice for tie-breaks and for seeding zero-vote parties.
// - Integer-first logic; deterministic tie-breaks via registry order then OptionId.
// - Returns Result to surface impossible growth cases (e.g., locals with zero votes).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::cmp::max;

use crate::allocation::mmp::{
    apportion_targets, build_order_index, cmp_quotients, divisor, union_option_ids, MmpError,
    MmpOutcome,
};
use vm_core::ids::OptionId;
use vm_core::variables::{AllocationMethod, OverhangPolicy, TotalSeatsModel};

/// Given intended `targets` and `local_seats`, compute top-ups and apply the
/// chosen overhang policy/model. When policy requires re-apportionment or
/// house-size expansion, uses `method_for_targets` on `vote_totals`.
///
/// Returns an `MmpOutcome` describing per-option targets/topups/finals and the
/// effective total seat count.
pub fn compute_topups_and_apply_overhang(
    options: &[OptionId],
    targets: &BTreeMap<OptionId, u32>,
    local_seats: &BTreeMap<OptionId, u32>,
    overhang_policy: OverhangPolicy,
    _total_seats_model: TotalSeatsModel, // placeholder for future variants (e.g., modified Sainte-Laguë first divisor)
    method_for_targets: AllocationMethod,
    vote_totals: &BTreeMap<OptionId, u64>,
) -> Result<MmpOutcome, MmpError> {
    // Build a stable set of all option IDs present across inputs + registry.
    let opt_map: BTreeMap<OptionId, u8> = options.iter().copied().map(|k| (k, 0)).collect();
    let mut all_ids: BTreeSet<OptionId> = union_option_ids(targets, local_seats, &opt_map);
    for k in vote_totals.keys() {
        all_ids.insert(*k);
    }

    // Sums (128-bit to avoid overflow)
    let mut target_sum: u128 = 0;
    let mut local_sum: u128 = 0;
    for k in all_ids.iter() {
        target_sum = target_sum.saturating_add(u128::from(*targets.get(k).unwrap_or(&0)));
        local_sum = local_sum.saturating_add(u128::from(*local_seats.get(k).unwrap_or(&0)));
    }

    // Compute per-option deficits and overhangs on the given targets.
    let mut deficits: BTreeMap<OptionId, u32> = BTreeMap::new();
    let mut overhangs: BTreeMap<OptionId, u32> = BTreeMap::new();
    for k in all_ids.iter() {
        let t = *targets.get(k).unwrap_or(&0);
        let l = *local_seats.get(k).unwrap_or(&0);
        if t >= l {
            deficits.insert(*k, t - l);
            overhangs.insert(*k, 0);
        } else {
            deficits.insert(*k, 0);
            overhangs.insert(*k, l - t);
        }
    }

    // Registry order index for deterministic tie-breaks in capped distributions.
    let order_index = build_order_index(options);

    match overhang_policy {
        OverhangPolicy::AllowOverhang => {
            // Top-ups exactly cover deficits; finals = locals + deficits;
            // effective total seats increases by Σ overhang.
            let mut topups: BTreeMap<OptionId, u32> = BTreeMap::new();
            let mut finals: BTreeMap<OptionId, u32> = BTreeMap::new();
            let mut t_eff: u128 = target_sum;

            for k in all_ids.iter() {
                let up = *deficits.get(k).unwrap_or(&0);
                let l = *local_seats.get(k).unwrap_or(&0);
                let oh = *overhangs.get(k).unwrap_or(&0);
                topups.insert(*k, up);
                finals.insert(*k, l.saturating_add(up));
                t_eff = t_eff.saturating_add(u128::from(oh));
            }

            let outcome = MmpOutcome {
                targets: targets.clone(),
                topups,
                finals,
                effective_total_seats: (t_eff.min(u32::MAX as u128)) as u32,
                overhang_by_option: overhangs,
            };
            debug_assert!(sum_map_equals(&outcome.finals) == outcome.effective_total_seats as u128);
            Ok(outcome)
        }

        OverhangPolicy::CompensateOthers => {
            // Fixed house size = T (the given targets' total). Pool = max(T - Σ locals, 0).
            let T: u32 = (target_sum.min(u32::MAX as u128)) as u32;
            let local_total_u32: u32 = (local_sum.min(u32::MAX as u128)) as u32;
            let pool: u32 = T.saturating_sub(local_total_u32);

            // Assign pool seats to **non-overhang** options (deficit > 0) via highest-averages,
            // capped to each option's deficit.
            let mut assigned: BTreeMap<OptionId, u32> =
                all_ids.iter().copied().map(|k| (k, 0u32)).collect();

            if pool > 0 {
                for _ in 0..pool {
                    // Pick best eligible option deterministically.
                    let mut best: Option<(OptionId, u64, u128, usize)> = None;
                    for id in all_ids.iter() {
                        let cap = *deficits.get(id).unwrap_or(&0);
                        let already = *assigned.get(id).unwrap_or(&0);
                        if cap == 0 || already >= cap {
                            continue; // overhang or cap reached
                        }
                        let v = *vote_totals.get(id).unwrap_or(&0);
                        let d = divisor(method_for_targets, already);

                        match best {
                            None => {
                                let ord = *order_index.get(id).unwrap_or(&usize::MAX);
                                best = Some((*id, v, d, ord));
                            }
                            Some((b_id, b_v, b_d, b_ord)) => {
                                use core::cmp::Ordering::*;
                                match cmp_quotients(v, d, b_v, b_d) {
                                    Greater => {
                                        let ord = *order_index.get(id).unwrap_or(&usize::MAX);
                                        best = Some((*id, v, d, ord));
                                    }
                                    Less => {}
                                    Equal => {
                                        let ord = *order_index.get(id).unwrap_or(&usize::MAX);
                                        if ord < b_ord || (ord == b_ord && *id < b_id) {
                                            best = Some((*id, v, d, ord));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some((pick, _, _, _)) = best {
                        if let Some(x) = assigned.get_mut(&pick) {
                            *x = x.saturating_add(1);
                        }
                    } else {
                        // No assignable seat (all caps reached) — stop early.
                        break;
                    }
                }
            }

            // Build topups/finals; overhangs are kept (no compensation to overhang options themselves).
            let mut topups: BTreeMap<OptionId, u32> = BTreeMap::new();
            let mut finals: BTreeMap<OptionId, u32> = BTreeMap::new();
            for k in all_ids.iter() {
                let add = *assigned.get(k).unwrap_or(&0);
                let l = *local_seats.get(k).unwrap_or(&0);
                topups.insert(*k, add);
                finals.insert(*k, l.saturating_add(add));
            }

            let outcome = MmpOutcome {
                targets: targets.clone(),
                topups,
                finals,
                effective_total_seats: T,
                overhang_by_option: overhangs,
            };
            // In a perfect fill, Σ finals == T + Σ overhang? Here finals excludes explicit overhang bump;
            // with fixed house, Σ finals should equal max(T, Σ locals) but we set pool = T - Σ locals,
            // so Σ finals == T (or less if deficits < pool and we broke early).
            debug_assert!(sum_map_equals(&outcome.finals) as u32 <= outcome.effective_total_seats);
            Ok(outcome)
        }

        OverhangPolicy::AddTotalSeats => {
            // Expand house size minimally until an **unconstrained apportionment**
            // assigns seats ≥ locals for every option. This may be impossible if
            // some option has locals > 0 but zero votes; detect that early.
            for id in all_ids.iter() {
                let l = *local_seats.get(id).unwrap_or(&0);
                let v = *vote_totals.get(id).unwrap_or(&0);
                if l > 0 && v == 0 {
                    return Err(MmpError::NoViableApportionment { option_id: *id });
                }
            }

            let mut T: u32 = max(
                (target_sum.min(u32::MAX as u128)) as u32,
                (local_sum.min(u32::MAX as u128)) as u32,
            );

            // Monotone search upwards; guaranteed to terminate if every party with locals has votes.
            // Add a conservative iteration cap to avoid pathological loops.
            let mut iters: u32 = 0;
            let cap: u32 = u32::MAX; // natural hard cap

            loop {
                let t_k = apportion_targets(T, options, vote_totals, method_for_targets);

                // Check feasibility: apportionment meets or exceeds locals component-wise.
                let mut ok = true;
                for id in all_ids.iter() {
                    let l = *local_seats.get(id).unwrap_or(&0);
                    let t = *t_k.get(id).unwrap_or(&0);
                    if t < l {
                        ok = false;
                        break;
                    }
                }

                if ok {
                    // No overhang remains under this expanded house.
                    let mut topups: BTreeMap<OptionId, u32> = BTreeMap::new();
                    let mut finals: BTreeMap<OptionId, u32> = BTreeMap::new();
                    let mut new_targets: BTreeMap<OptionId, u32> = BTreeMap::new();
                    let mut zeros: BTreeMap<OptionId, u32> = BTreeMap::new();

                    for id in all_ids.iter() {
                        let l = *local_seats.get(id).unwrap_or(&0);
                        let t = *t_k.get(id).unwrap_or(&0);
                        let up = t.saturating_sub(l);
                        topups.insert(*id, up);
                        finals.insert(*id, t);
                        new_targets.insert(*id, t);
                        zeros.insert(*id, 0);
                    }

                    let outcome = MmpOutcome {
                        targets: new_targets,
                        topups,
                        finals,
                        effective_total_seats: T,
                        overhang_by_option: zeros, // by construction, zero overhang
                    };
                    debug_assert!(sum_map_equals(&outcome.finals) as u32 == outcome.effective_total_seats);
                    return Ok(outcome);
                }

                // Grow T and continue.
                if T == cap {
                    // Should be unreachable given the early zero-vote guard.
                    return Err(MmpError::NoViableApportionment {
                        option_id: *all_ids.iter().next().unwrap(), // placeholder
                    });
                }
                T = T.saturating_add(1);
                iters = iters.saturating_add(1);
                // Optional: break after very large search to avoid CPU blowups in adversarial inputs.
                // if iters > 10_000_000 { return Err(MmpError::NoViableApportionment { option_id: *all_ids.iter().next().unwrap() }); }
            }
        }
    }
}

/// Sum helper (u32 values) → u128 total.
#[inline]
fn sum_map_equals(map: &BTreeMap<OptionId, u32>) -> u128 {
    map.values().fold(0u128, |acc, &v| acc + (v as u128))
}
