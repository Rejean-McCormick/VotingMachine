//! ALLOCATE stage: per-Unit seat/power allocation.
//!
//! Input: `UnitScores` from TABULATE, Unit metadata (magnitude), canonical options,
//! and a `Params` snapshot (allocation method, PR threshold, tie policy/seed).
//! Output: map of `UnitId -> UnitAllocation` and any tie contexts that should be
//! logged/handled downstream. All choices are deterministic given the same inputs
//! and, when `Random` tie policy is used, the same seed.

use std::collections::BTreeMap;

use vm_core::{
    ids::{OptionId, UnitId},
    // `UnitMeta` is expected to expose `magnitude` (u32). If your core uses a different
    // struct name for per-unit metadata, alias it here.
    entities::Unit as UnitMeta,
    rng::{tie_rng_from_seed, TieRng},
    variables::{AllocationMethod, Params, TiePolicy},
};
use vm_algo::{
    allocation::{
        dhondt as algo_dhondt,
        largest_remainder as algo_lr,
        sainte_lague as algo_sl,
        wta as algo_wta,
    },
    tabulation::UnitScores, // unit-level score container from vm_algo
};

// If you keep tie-logging in a dedicated module, import its context type.
// (This file only collects; construction happens where a tie actually occurs.)
use crate::ties::TieContext;

// Re-export the LR quota kind so callers don’t need to import the algo module directly.
pub use vm_algo::allocation::largest_remainder::QuotaKind as LrQuotaKind;

/// Unit-level allocation result. Seats for PR families; `100` "power" for WTA.
#[derive(Debug, Clone)]
pub struct UnitAllocation {
    pub seats_or_power: BTreeMap<OptionId, u32>,
    pub last_seat_tie: bool,
}

/// Public entry: allocate all units by the configured method.
///
/// - Iterates Units in stable `UnitId` order.
/// - Applies the PR entry threshold where relevant (inside the algo call).
/// - Honors tie policy (status_quo / deterministic / seeded random).
pub fn allocate_all(
    unit_scores: &BTreeMap<UnitId, UnitScores>,
    units: &BTreeMap<UnitId, UnitMeta>,
    options_by_unit: &BTreeMap<UnitId, Vec<vm_core::entities::OptionItem>>,
    params: &Params,
) -> (BTreeMap<UnitId, UnitAllocation>, Vec<TieContext>) {
    // Prepare RNG once when tie policy == Random; shared stream across units for deterministic replay.
    let tie_policy = params.tie_policy();
    let mut rng = match tie_policy {
        TiePolicy::Random => {
            let seed = params.tie_seed(); // VM-VAR-033 / 052 per spec (integer ≥ 0)
            Some(tie_rng_from_seed(seed as u64))
        }
        _ => None,
    };

    let mut out = BTreeMap::<UnitId, UnitAllocation>::new();
    let mut tie_contexts: Vec<TieContext> = Vec::new();

    for (unit_id, meta) in units.iter() {
        // Canonical option order comes from loader/validate; fall back to empty vec if missing.
        let options = options_by_unit.get(unit_id).map(Vec::as_slice).unwrap_or(&[]);
        let scores = unit_scores
            .get(unit_id)
            .unwrap_or_else(|| {
                // Graceful empty scores when a unit somehow lacks tabulation (should not happen)
                // (0 votes for each option in canonical order).
                // Build a zero map in canonical order:
                static EMPTY: once_cell::sync::Lazy<UnitScores> = once_cell::sync::Lazy::new(|| UnitScores {
                    unit_id: UnitId::from_static("U:__missing__"),
                    turnout: vm_core::entities::Turnout {
                        ballots_cast: 0,
                        invalid_ballots: 0,
                        valid_ballots: 0,
                    },
                    scores: BTreeMap::new(),
                });
                &*EMPTY
            });

        let (alloc, tie_ctx_opt) = allocate_one_unit(
            unit_id.clone(),
            scores,
            meta,
            options,
            params,
            rng.as_mut().map(|r| r as &mut TieRng),
        );

        if let Some(tc) = tie_ctx_opt {
            tie_contexts.push(tc);
        }
        out.insert(unit_id.clone(), alloc);
    }

    (out, tie_contexts)
}

/// Orchestrate allocation for a single unit based on VM-VAR-010.
fn allocate_one_unit(
    unit_id: UnitId,
    scores: &UnitScores,
    meta: &UnitMeta,
    options: &[vm_core::entities::OptionItem],
    p: &Params,
    rng: Option<&mut TieRng>,
) -> (UnitAllocation, Option<TieContext>) {
    let method = p.allocation_method();
    let pr_threshold = p.pr_entry_threshold_pct();
    let tie = p.tie_policy();

    match method {
        AllocationMethod::WinnerTakeAll => {
            // Enforce magnitude == 1 for WTA.
            if meta.magnitude != 1 {
                // Surface a deterministic zero-allocation with a tie note; upstream VALIDATE should block this earlier.
                return (
                    UnitAllocation {
                        seats_or_power: BTreeMap::new(),
                        last_seat_tie: false,
                    },
                    Some(TieContext::error(
                        unit_id,
                        "Method.WTA.RequiresMagnitude1",
                        "winner_take_all requires unit magnitude == 1",
                    )),
                );
            }
            // WTA lives inside vm_algo; it accepts UnitScores to use the same integer turnouts if needed.
            match algo_wta::allocate_wta(scores, meta.magnitude, options, tie, rng) {
                Ok(winner_alloc) => {
                    let mut map = winner_alloc.seats_or_power;
                    // Ensure exactly one entry has 100, others 0 (algo_wta ensures this).
                    (UnitAllocation { seats_or_power: map, last_seat_tie: winner_alloc.last_seat_tie }, None)
                }
                Err(e) => (
                    UnitAllocation { seats_or_power: BTreeMap::new(), last_seat_tie: false },
                    Some(TieContext::error(unit_id, "Allocate.WTA", &format!("allocation failed: {:?}", e))),
                ),
            }
        }

        AllocationMethod::ProportionalFavorBig => {
            // D’Hondt
            let result = algo_dhondt::allocate_dhondt(
                meta.magnitude,
                &scores.scores,
                options,
                pr_threshold,
                tie,
                rng,
            );
            match result {
                Ok(seats) => (UnitAllocation { seats_or_power: seats, last_seat_tie: false }, None),
                Err(e) => (
                    UnitAllocation { seats_or_power: BTreeMap::new(), last_seat_tie: false },
                    Some(TieContext::error(unit_id, "Allocate.DHondt", &format!("allocation failed: {:?}", e))),
                ),
            }
        }

        AllocationMethod::ProportionalFavorSmall => {
            // Sainte-Laguë
            let result = algo_sl::allocate_sainte_lague(
                meta.magnitude,
                &scores.scores,
                options,
                pr_threshold,
                tie,
                rng,
            );
            match result {
                Ok(seats) => (UnitAllocation { seats_or_power: seats, last_seat_tie: false }, None),
                Err(e) => (
                    UnitAllocation { seats_or_power: BTreeMap::new(), last_seat_tie: false },
                    Some(TieContext::error(unit_id, "Allocate.SainteLague", &format!("allocation failed: {:?}", e))),
                ),
            }
        }

        AllocationMethod::LargestRemainder => {
            // LR with quota configured in Params (Hare/Droop/Imperiali)
            let quota = lr_quota_from_params(p);
            let result = algo_lr::allocate_largest_remainder(
                meta.magnitude,
                &scores.scores,
                options,
                pr_threshold,
                quota,
                tie,
                rng,
            );
            match result {
                Ok(seats) => (UnitAllocation { seats_or_power: seats, last_seat_tie: false }, None),
                Err(e) => (
                    UnitAllocation { seats_or_power: BTreeMap::new(), last_seat_tie: false },
                    Some(TieContext::error(unit_id, "Allocate.LargestRemainder", &format!("allocation failed: {:?}", e))),
                ),
            }
        }

        // Mixed-member correction is handled at aggregate/MMP level; locals come from SMD/WTA above.
        AllocationMethod::MixedLocalCorrection => {
            (UnitAllocation { seats_or_power: BTreeMap::new(), last_seat_tie: false }, None)
        }
    }
}

// ----- helpers -------------------------------------------------------------------------------------------------------

#[inline]
fn lr_quota_from_params(p: &Params) -> LrQuotaKind {
    // Wire this to your params variable (e.g., VM-VAR for LR quota). Default to Hare if unset.
    if let Some(kind) = p.lr_quota_kind() {
        match kind {
            // Keep exact mapping as your `Params` exposes it.
            vm_core::variables::LrQuotaKind::Hare => LrQuotaKind::Hare,
            vm_core::variables::LrQuotaKind::Droop => LrQuotaKind::Droop,
            vm_core::variables::LrQuotaKind::Imperiali => LrQuotaKind::Imperiali,
        }
    } else {
        LrQuotaKind::Hare
    }
}

// -------------------------------------------------------------------------------------------------
// TieContext convenience (error helper)
// -------------------------------------------------------------------------------------------------

mod tie_context_helpers {
    use super::*;
    pub trait TieContextExt {
        fn error(unit_id: UnitId, code: &'static str, msg: &str) -> Self;
    }
    impl TieContextExt for TieContext {
        fn error(unit_id: UnitId, code: &'static str, msg: &str) -> Self {
            TieContext {
                unit_id,
                code: code.into(),
                message: msg.into(),
                contenders: Vec::new(),
                note: None,
            }
        }
    }
}
use tie_context_helpers::TieContextExt;
