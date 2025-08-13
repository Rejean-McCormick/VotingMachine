//! Mixed-Member Proportional (MMP) helpers:
//! - compute total seats from a top-up share (VM-VAR-013) with half-even rounding,
//! - apportion targets using highest-averages (D’Hondt or Sainte-Laguë),
//! - compute top-ups against local seats and apply overhang policy/model,
//! - one-shot orchestration for a correction scope.
//!
//! Determinism: stable OptionId order is used for all deterministic tie breaks;
//! no RNG is used anywhere in MMP.

use std::collections::BTreeMap;

use vm_core::{
    ids::OptionId,
    variables::{AllocationMethod, OverhangPolicy, Params, TotalSeatsModel},
    rounding::round_nearest_even_int,
};

/// Result bundle for an MMP correction step.
#[derive(Debug, Clone)]
pub struct MmpOutcome {
    pub targets: BTreeMap<OptionId, u32>,
    pub topups: BTreeMap<OptionId, u32>,
    pub finals: BTreeMap<OptionId, u32>,
    pub effective_total_seats: u32,
    pub overhang_by_option: BTreeMap<OptionId, u32>,
}

/// Compute intended total seats `T` from local seats `L` and top-up share `p%`
/// using banker's (half-even) rounding on `T = (L*100) / (100 - p)`.
///
/// Domain: upstream enforces `p <= 60` (and `< 100` in general).
pub fn compute_total_from_share(local_total: u32, topup_share_pct: u8) -> u32 {
    // Guard against pathological input; upstream should prevent p >= 100.
    if topup_share_pct >= 100 {
        return 0;
    }
    let num = (local_total as i128) * 100;
    let den = (100u8.saturating_sub(topup_share_pct) as i128).max(1);
    // round_nearest_even_int never panics for den > 0.
    match round_nearest_even_int(num, den) {
        Ok(v) if v <= u32::MAX as i128 => v as u32,
        Ok(_) => u32::MAX,
        Err(_) => 0,
    }
}

/// Apportion `total_seats` to options from `vote_totals` via a highest-averages method.
/// Deterministic, integer-only; ties broken by lexicographic `OptionId`.
pub fn apportion_targets(
    total_seats: u32,
    vote_totals: &BTreeMap<OptionId, u64>,
    method: AllocationMethod,
) -> BTreeMap<OptionId, u32> {
    let mut alloc: BTreeMap<OptionId, u32> =
        vote_totals.keys().cloned().map(|k| (k, 0u32)).collect();

    if total_seats == 0 || vote_totals.is_empty() {
        return alloc;
    }

    for _ in 0..total_seats {
        // Choose arg-max of quotient v / d(s); ties by OptionId ascending.
        let mut best_id: OptionId = OptionId::from(String::new()); // temp init; will overwrite
        let mut have_best = false;
        let mut best_v: u64 = 0;
        let mut best_d: u128 = 1;

        for (id, &v) in vote_totals.iter() {
            let s = *alloc.get(id).unwrap_or(&0);
            let d = divisor(method, s);
            // Compare v/d vs best_v/best_d via cross-multiplication: v*best_d ? best_v*d
            let left = (v as u128).saturating_mul(best_d);
            let right = (best_v as u128).saturating_mul(d);
            let better = if !have_best {
                true
            } else if left > right {
                true
            } else if left < right {
                false
            } else {
                // exact tie on quotient → smallest OptionId
                id < &best_id
            };

            if better {
                best_id = id.clone();
                best_v = v;
                best_d = d;
                have_best = true;
            }
        }

        if have_best {
            *alloc.get_mut(&best_id).unwrap() += 1;
        } else {
            // No votes at all: nothing to apportion.
            break;
        }
    }

    alloc
}

/// Given intended `targets` and `local_seats`, compute top-ups and apply the
/// chosen overhang policy/model. When policy requires re-apportionment or
/// expansion, uses `method_for_targets` on `vote_totals`.
pub fn compute_topups_and_apply_overhang(
    targets: &BTreeMap<OptionId, u32>,
    local_seats: &BTreeMap<OptionId, u32>,
    overhang_policy: OverhangPolicy,
    _total_seats_model: TotalSeatsModel,
    method_for_targets: AllocationMethod,
    vote_totals: &BTreeMap<OptionId, u64>,
) -> MmpOutcome {
    // Union of all option IDs present.
    let mut all_ids: BTreeMap<OptionId, ()> = BTreeMap::new();
    for k in targets.keys() {
        all_ids.insert(k.clone(), ());
    }
    for k in local_seats.keys() {
        all_ids.insert(k.clone(), ());
    }
    for k in vote_totals.keys() {
        all_ids.insert(k.clone(), ());
    }

    // Helpers
    let mut target_sum: u128 = 0;
    let mut local_sum: u128 = 0;

    for k in all_ids.keys() {
        target_sum += (*targets.get(k).unwrap_or(&0)) as u128;
        local_sum += (*local_seats.get(k).unwrap_or(&0)) as u128;
    }

    // Compute per-option deficits and overhangs on the current targets.
    let mut deficits: BTreeMap<OptionId, u32> = BTreeMap::new();
    let mut overhangs: BTreeMap<OptionId, u32> = BTreeMap::new();
    for k in all_ids.keys() {
        let t = *targets.get(k).unwrap_or(&0);
        let l = *local_seats.get(k).unwrap_or(&0);
        if t >= l {
            deficits.insert(k.clone(), t - l);
            overhangs.insert(k.clone(), 0);
        } else {
            deficits.insert(k.clone(), 0);
            overhangs.insert(k.clone(), l - t);
        }
    }

    match overhang_policy {
        OverhangPolicy::AllowOverhang => {
            // Top-ups exactly cover deficits; final total = T + Σ overhang.
            let mut topups = BTreeMap::new();
            let mut finals = BTreeMap::new();
            let mut eff_total: u128 = target_sum;

            for k in all_ids.keys() {
                let up = *deficits.get(k).unwrap_or(&0);
                let l = *local_seats.get(k).unwrap_or(&0);
                let oh = *overhangs.get(k).unwrap_or(&0);
                topups.insert(k.clone(), up);
                finals.insert(k.clone(), l.saturating_add(up));
                eff_total = eff_total.saturating_add(oh as u128);
            }

            MmpOutcome {
                targets: targets.clone(),
                topups,
                finals,
                effective_total_seats: (eff_total.min(u32::MAX as u128)) as u32,
                overhang_by_option: overhangs,
            }
        }

        OverhangPolicy::CompensateOthers => {
            // Fixed house size = T. Pool seats = T - Σ local.
            let pool = if target_sum > local_sum {
                (target_sum - local_sum) as u32
            } else {
                0
            };

            // Eligible for top-ups: options without overhang (deficit > 0).
            // We iteratively award up to `deficit_i` seats using highest-averages on vote_totals.
            let mut assigned: BTreeMap<OptionId, u32> =
                all_ids.keys().cloned().map(|k| (k, 0u32)).collect();

            if pool > 0 {
                // Work on a filtered vote vector (only deficit > 0 contribute);
                // if all zeros, the loop will just assign nothing (finals = locals).
                let eligible: BTreeMap<OptionId, u64> = all_ids
                    .keys()
                    .filter_map(|k| {
                        let cap = *deficits.get(k).unwrap_or(&0);
                        if cap > 0 {
                            Some((
                                k.clone(),
                                *vote_totals.get(k).unwrap_or(&0), // may be 0
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();

                if !eligible.is_empty() {
                    // Seat-by-seat highest-averages (deterministic), capped by remaining deficit.
                    for _ in 0..pool {
                        let mut pick: OptionId = OptionId::from(String::new());
                        let mut have_pick = false;
                        let mut best_v: u64 = 0;
                        let mut best_d: u128 = 1;

                        for (id, &v) in eligible.iter() {
                            let already = *assigned.get(id).unwrap_or(&0);
                            let cap = *deficits.get(id).unwrap_or(&0);
                            if already >= cap {
                                continue; // capacity reached
                            }
                            let d = divisor(method_for_targets, already);
                            let left = (v as u128).saturating_mul(best_d);
                            let right = (best_v as u128).saturating_mul(d);
                            let better = if !have_pick {
                                true
                            } else if left > right {
                                true
                            } else if left < right {
                                false
                            } else {
                                id < &pick
                            };
                            if better {
                                pick = id.clone();
                                best_v = v;
                                best_d = d;
                                have_pick = true;
                            }
                        }

                        if have_pick {
                            *assigned.get_mut(&pick).unwrap() += 1;
                        } else {
                            // No eligible seat can be assigned (all caps reached or no votes).
                            break;
                        }
                    }
                }
            }

            let mut topups = BTreeMap::new();
            let mut finals = BTreeMap::new();
            for k in all_ids.keys() {
                let add = *assigned.get(k).unwrap_or(&0);
                let l = *local_seats.get(k).unwrap_or(&0);
                // Overhang options get 0 top-ups in this policy.
                topups.insert(k.clone(), add);
                finals.insert(k.clone(), l.saturating_add(add));
            }

            MmpOutcome {
                targets: targets.clone(),
                topups,
                finals,
                effective_total_seats: (target_sum.min(u32::MAX as u128)) as u32,
                overhang_by_option: overhangs,
            }
        }

        OverhangPolicy::AddTotalSeats => {
            // Expand total seats minimally until targets >= locals component-wise.
            let mut tk = (target_sum.min(u32::MAX as u128)) as u32;
            // Safety: ensure at least current locals.
            let local_total_u32 = (local_sum.min(u32::MAX as u128)) as u32;
            if tk < local_total_u32 {
                tk = local_total_u32;
            }

            // Minimal iterative growth (deterministic).
            loop {
                let t_k = apportion_targets(tk, vote_totals, method_for_targets);
                if all_ids.iter().all(|(k, _)| {
                    let l = *local_seats.get(k).unwrap_or(&0);
                    let t = *t_k.get(k).unwrap_or(&0);
                    t >= l
                }) {
                    // Compute topups/finals using t_k.
                    let mut topups = BTreeMap::new();
                    let mut finals = BTreeMap::
