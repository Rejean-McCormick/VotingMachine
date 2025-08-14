// crates/vm_pipeline/src/allocate.rs — Part 1/2 (patched)
//
// Spec-aligned allocation stage: local error types, small structs, and helpers.
// Pairs with Part 2/2 (which contains the per-unit and all-units allocation).
//
// Alignment anchors (Docs 1–7 + Annexes A–C):
// • VM-VAR-050  tie_policy: { deterministic_order | random | status_quo }  (Included)
// • VM-VAR-052  tie_seed   (Excluded; must be logged in RunRecord integrity)
// • PR entry thresholds apply *before* apportionment (Doc 4 S4; Doc 6 tests)
// • Do NOT signal structural errors via “tie contexts”; use typed errors.
// • Determinism: stable iteration (UnitId asc), option ordering = registry order
//   when emitting; if a map is stored, re-order at packaging time (Doc 7).
// • Frontier/tie logging: emit concrete tie contexts for last-seat ties; random
//   policy later records RNG crumbs (resolved by the tie resolver).

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    entities::{OptionItem, TallyTotals, UnitMeta, UnitScores},
    ids::{OptionId, UnitId},
};

use vm_algo::enums::{LrQuotaKind, TiePolicy};
use vm_pipeline::ties::{TieContext, TieKind};
use vm_pipeline::PipelineError;

// ---------- Local error taxonomy (no ties used for errors) ----------

#[derive(Debug)]
pub enum AllocateError {
    MissingScores { unit: UnitId },
    OptionSetMismatch {
        unit: UnitId,
        missing_in_scores: BTreeSet<OptionId>,
        extra_in_scores: BTreeSet<OptionId>,
    },
    LrQuotaMissing { unit: UnitId },
    WtaRequiresMagnitude1 { unit: UnitId, got: u32 },
    StatusQuoAnchorMissing { unit: UnitId },
}

impl From<AllocateError> for PipelineError {
    fn from(e: AllocateError) -> Self {
        use AllocateError::*;
        match e {
            MissingScores { unit } => PipelineError::Allocate(format!("missing scores for unit {unit}")),
            OptionSetMismatch { unit, missing_in_scores, extra_in_scores } => {
                PipelineError::Allocate(format!(
                    "option set mismatch for unit {unit}: missing_in_scores={:?}, extra_in_scores={:?}",
                    missing_in_scores, extra_in_scores
                ))
            }
            LrQuotaMissing { unit } => PipelineError::Allocate(format!(
                "LargestRemainder requires a quota kind (Doc 4 §S4); missing for unit {unit}"
            )),
            WtaRequiresMagnitude1 { unit, got } => PipelineError::Allocate(format!(
                "WinnerTakeAll requires magnitude=1 (Doc 4B); unit {unit} has magnitude={got}"
            )),
            StatusQuoAnchorMissing { unit } => PipelineError::Allocate(format!(
                "tie_policy=status_quo requires an anchor (Annex C); missing for unit {unit}"
            )),
        }
    }
}

// ---------- Small data structures used within allocate ----------

#[derive(Debug, Clone)]
pub struct UnitAllocation {
    /// Seats (or power for WTA if your schema requires) by option.
    pub seats_by_option: BTreeMap<OptionId, u32>,
    /// Whether the last seat on this unit was awarded on a tie.
    pub last_seat_tie: bool,
}

#[derive(Debug, Default)]
pub struct AllocateOutputs {
    pub per_unit: BTreeMap<UnitId, UnitAllocation>,
    /// All tie contexts observed during allocation (e.g., last-seat ties).
    pub ties: Vec<TieContext>,
}

// ---------- Helper functions (spec-aligned) ----------

/// Ensure the set of options in `scores` matches the registry `options` for this unit.
/// We *do not* synthesize “empty scores”; misalignment is structural and must be rejected.
pub fn assert_option_set_alignment(
    unit: &UnitId,
    options: &[OptionItem],
    scores: &UnitScores,
) -> Result<(), AllocateError> {
    let reg_set: BTreeSet<OptionId> = options.iter().map(|o| o.id.clone()).collect();
    let scr_set: BTreeSet<OptionId> = scores.by_option.keys().cloned().collect();

    if reg_set == scr_set {
        return Ok(());
    }

    let missing_in_scores = &reg_set - &scr_set;
    let extra_in_scores = &scr_set - &reg_set;
    Err(AllocateError::OptionSetMismatch {
        unit: unit.clone(),
        missing_in_scores,
        extra_in_scores,
    })
}

/// Apply PR entry threshold before apportionment, returning a filtered
/// (option_id → score) map. Threshold semantics are **inclusive**:
/// option is eligible if (score / valid_ballots) * 100 >= threshold_pct.
/// (If your spec requires strict “>”, change the comparison accordingly.)
pub fn apply_pr_threshold(
    scores: &UnitScores,
    turnout: &TallyTotals,
    threshold_pct: f64,
) -> BTreeMap<OptionId, u64> {
    if threshold_pct <= 0.0 {
        return scores.by_option.clone();
    }

    let valid = turnout.valid_ballots as f64;
    if valid <= 0.0 {
        // Degenerate case: no valid ballots → nobody passes (algo should yield 0 seats)
        return BTreeMap::new();
    }

    let mut filtered = BTreeMap::new();
    for (opt, &sc) in &scores.by_option {
        let pct = (sc as f64) * 100.0 / valid;
        if pct + f64::EPSILON >= threshold_pct {
            filtered.insert(opt.clone(), sc);
        }
    }
    filtered
}

/// Build a **last-seat** tie context for reporting/resolution downstream.
/// The resolver will use VM-VAR-050/052 to pick the winner if needed.
pub fn make_last_seat_tie_context(
    unit: &UnitId,
    contenders: &[OptionId],
    policy: TiePolicy, // VM-VAR-050
) -> TieContext {
    let mut c = contenders.to_vec();
    c.sort(); // stable canonical payload
    TieContext {
        unit_id: unit.clone(),
        kind: TieKind::LastSeat,
        contenders: c,
        policy,
    }
}

/// Validate WTA magnitude at the allocation stage boundary. In a strictly
/// validated pipeline this should be unreachable; we keep the guard to defend
/// against upstream drift and map it to an AllocateError.
pub fn ensure_wta_magnitude(unit: &UnitId, magnitude: u32) -> Result<(), AllocateError> {
    if magnitude == 1 {
        Ok(())
    } else {
        Err(AllocateError::WtaRequiresMagnitude1 {
            unit: unit.clone(),
            got: magnitude,
        })
    }
}

/// Ensure LR quota presence when method == LargestRemainder.
/// In a strictly validated pipeline this should be unreachable.
pub fn ensure_lr_quota(unit: &UnitId, quota: Option<LrQuotaKind>) -> Result<LrQuotaKind, AllocateError> {
    match quota {
        Some(q) => Ok(q),
        None => Err(AllocateError::LrQuotaMissing { unit: unit.clone() }),
    }
}
// crates/vm_pipeline/src/allocate.rs — Part 2/2 (patched)
//
// Full allocation routine with spec-aligned tie handling and guards.
// Pairs with Part 1/2 in this same module file. No blocks are split mid-way.

use std::collections::BTreeMap;

use vm_core::{
    entities::{OptionItem, TallyTotals, UnitMeta, UnitScores},
    ids::{OptionId, UnitId},
};
use vm_algo::enums::{AllocationMethod, LrQuotaKind, TiePolicy};
use vm_pipeline::ties::{TieContext, TieKind};
use vm_pipeline::PipelineError;

use crate::vm_pipeline::allocate::{
    // From Part 1/2 (same module file)
    apply_pr_threshold, assert_option_set_alignment, ensure_lr_quota, ensure_wta_magnitude,
    make_last_seat_tie_context, AllocateError, AllocateOutputs, UnitAllocation,
};

// ------------------------- public entrypoints -------------------------

/// Allocate seats for all units, returning per-unit allocations and any tie
/// contexts that must be resolved/logged downstream.
pub fn allocate_all_units(
    options_by_unit: &BTreeMap<UnitId, Vec<OptionItem>>,
    scores_by_unit: &BTreeMap<UnitId, UnitScores>,
    meta_by_unit: &BTreeMap<UnitId, UnitMeta>,
    tie_policy: TiePolicy, // VM-VAR-050 (Included)
) -> Result<AllocateOutputs, PipelineError> {
    let mut out = AllocateOutputs::default();

    // Iterate in stable UnitId order (BTreeMap guarantees).
    for (unit_id, options) in options_by_unit {
        // Fetch scores/meta
        let scores = scores_by_unit
            .get(unit_id)
            .ok_or_else(|| AllocateError::MissingScores { unit: unit_id.clone() })?;
        let meta = meta_by_unit
            .get(unit_id)
            .ok_or_else(|| PipelineError::Allocate(format!("missing meta for unit {unit_id}")))?;

        // Guard: option set alignment (structural)
        assert_option_set_alignment(unit_id, options, scores)?;

        let (alloc, mut ties) = allocate_one_unit(unit_id, options, scores, meta, tie_policy)?;
        out.per_unit.insert(unit_id.clone(), alloc);
        out.ties.append(&mut ties);
    }

    Ok(out)
}

/// Allocate a single unit according to its allocation method and parameters.
/// Emits a `UnitAllocation` and zero or more `TieContext`s (e.g., last-seat ties).
pub fn allocate_one_unit(
    unit: &UnitId,
    options: &[OptionItem],
    scores: &UnitScores,
    meta: &UnitMeta,
    tie_policy: TiePolicy, // VM-VAR-050
) -> Result<(UnitAllocation, Vec<TieContext>), PipelineError> {
    let mut ties: Vec<TieContext> = Vec::new();

    // Pull per-unit method/magnitude/threshold/quota from metadata.
    let method = get_method(meta);
    let magnitude = get_magnitude(meta);
    let pr_threshold = get_pr_threshold_pct(meta);
    let lr_quota = get_lr_quota_kind(meta);

    match method {
        AllocationMethod::WinnerTakeAll => {
            // Winner-take-all: magnitude MUST be 1 (validated upstream; checked here defensively)
            ensure_wta_magnitude(unit, magnitude)?;

            // Detect top score and candidates; WTA doesn’t need PR thresholding.
            let mut max_votes: u64 = 0;
            for &v in scores.by_option.values() {
                if v > max_votes {
                    max_votes = v;
                }
            }

            // Collect all contenders tied at max
            let mut contenders: Vec<OptionId> = scores
                .by_option
                .iter()
                .filter_map(|(opt, &v)| if v == max_votes { Some(opt.clone()) } else { None })
                .collect();

            let mut seats: BTreeMap<OptionId, u32> = options.iter().map(|o| (o.id.clone(), 0u32)).collect();
            let mut last_seat_tie = false;

            if contenders.is_empty() {
                // Degenerate: no options present (normally blocked upstream). Nothing to allocate.
                // Leave all zeros; no tie.
            } else if contenders.len() == 1 {
                // Single winner
                seats.insert(contenders[0].clone(), 1);
            } else {
                // Tie for the seat → log for downstream resolver
                last_seat_tie = true;

                // Emit a concrete tie context for the resolver/logs
                let mut contenders_ctx = contenders.clone();
                contenders_ctx.sort(); // stable order for context payload
                ties.push(make_last_seat_tie_context(unit, &contenders_ctx, tie_policy));

                // Status-quo requires an anchor we don't have here → refuse deterministically
                if tie_policy == TiePolicy::StatusQuo {
                    return Err(AllocateError::StatusQuoAnchorMissing { unit: unit.clone() }.into());
                }

                // Deterministic placeholder to keep structure consistent until resolver runs:
                // - deterministic_order → registry order (order_index, then OptionId)
                // - random              → lexicographically smallest OptionId
                let winner = if tie_policy == TiePolicy::DeterministicOrder {
                    // sort by registry order
                    let mut items: Vec<OptionId> = contenders.into_iter().collect();
                    items.sort_by_key(|id| {
                        options
                            .iter()
                            .find(|o| o.id == *id)
                            .map(|o| (o.order_index, id.clone()))
                            .unwrap_or((u32::MAX, id.clone()))
                    });
                    items[0].clone()
                } else {
                    // random policy: resolver will later record RNG crumbs (VM-VAR-052)
                    contenders.sort();
                    contenders[0].clone()
                };

                seats.insert(winner, 1);
            }

            let alloc = UnitAllocation {
                seats_by_option: seats,
                last_seat_tie,
            };
            Ok((alloc, ties))
        }

        AllocationMethod::Dhondt => {
            // Apply PR entry threshold first
            let turnout = get_turnout(scores);
            let filtered = apply_pr_threshold(scores, &turnout, pr_threshold);

            // Run the allocator in vm_algo. If your vm_algo exposes last-seat tie
            // details, propagate them here; else we conservatively set false.
            let seats = vm_algo::allocation::dhondt_allocate(magnitude, &filtered)
                .map_err(|m| PipelineError::Allocate(format!("dhondt: {m}")))?;

            let alloc = UnitAllocation {
                seats_by_option: reorder_to_registry(options, seats),
                last_seat_tie: false, // TODO: set true and push context if vm_algo exposes contenders
            };
            Ok((alloc, ties))
        }

        AllocationMethod::SainteLague => {
            let turnout = get_turnout(scores);
            let filtered = apply_pr_threshold(scores, &turnout, pr_threshold);

            let seats = vm_algo::allocation::sainte_lague_allocate(magnitude, &filtered)
                .map_err(|m| PipelineError::Allocate(format!("sainte_lague: {m}")))?;

            let alloc = UnitAllocation {
                seats_by_option: reorder_to_registry(options, seats),
                last_seat_tie: false, // TODO: propagate tie details if available
            };
            Ok((alloc, ties))
        }

        AllocationMethod::LargestRemainder => {
            let turnout = get_turnout(scores);
            let filtered = apply_pr_threshold(scores, &turnout, pr_threshold);

            let quota = ensure_lr_quota(unit, lr_quota)?;

            let seats = vm_algo::allocation::largest_remainder_allocate(magnitude, &filtered, quota)
                .map_err(|m| PipelineError::Allocate(format!("largest_remainder: {m}")))?;

            let alloc = UnitAllocation {
                seats_by_option: reorder_to_registry(options, seats),
                last_seat_tie: false, // TODO: propagate tie details if available
            };
            Ok((alloc, ties))
        }

        // If you have additional methods (e.g., MixedLocalCorrection), either dispatch
        // here to the appropriate vm_algo entrypoint or mark unreachable if validated upstream.
        other => Err(PipelineError::Allocate(format!(
            "unsupported allocation method for unit {unit}: {other:?}"
        ))),
    }
}

// ------------------------- small getters / adapters -------------------------

/// Get allocation method from unit meta (adjust if your API differs).
#[inline]
fn get_method(meta: &UnitMeta) -> AllocationMethod {
    meta.allocation_method
}

/// Get magnitude from unit meta (adjust if your API differs).
#[inline]
fn get_magnitude(meta: &UnitMeta) -> u32 {
    meta.magnitude
}

/// Get PR entry threshold pct from unit meta or params; default to 0.0 if unspecified.
/// Adjust if your API differs (e.g., Option<f64> in meta or computed from ParameterSet).
#[inline]
fn get_pr_threshold_pct(meta: &UnitMeta) -> f64 {
    meta.pr_threshold_pct.unwrap_or(0.0)
}

/// Get LR quota kind for Largest Remainder (adjust if your API differs).
#[inline]
fn get_lr_quota_kind(meta: &UnitMeta) -> Option<LrQuotaKind> {
    meta.lr_quota_kind
}

/// Extract turnout totals from UnitScores (adjust field/method name if needed).
#[inline]
fn get_turnout(scores: &UnitScores) -> TallyTotals {
    // If your struct exposes `totals` or `turnout`, adjust accordingly.
    scores.totals.clone()
}

/// Reorder seats map to registry order (order_index, then OptionId) for stable emission.
/// NOTE: This returns a BTreeMap keyed by OptionId; the *iteration* order here will be
/// OptionId-lexicographic. The packaging/reporting stage must use `options` to emit in
/// registry order when order matters (Doc 7).
#[inline]
fn reorder_to_registry(
    options: &[OptionItem],
    seats_by_option: BTreeMap<OptionId, u32>,
) -> BTreeMap<OptionId, u32> {
    let mut out = BTreeMap::new();
    // We keep a map here for compatibility; order-sensitive emission happens later.
    // Still, ensure all known options are present (fill zeros) to avoid sparse maps.
    let known: BTreeMap<OptionId, u32> = options.iter().map(|o| (o.id.clone(), 0u32)).collect();
    for (k, v) in known {
        let val = *seats_by_option.get(&k).unwrap_or(&0);
        out.insert(k, val);
    }
    out
}
