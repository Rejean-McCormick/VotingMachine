//! APPLY_RULES stage: evaluate legitimacy gates in fixed order
//! Quorum → Majority/Supermajority → Double-majority (optional) → Symmetry (optional).
//!
//! Pure integer/rational math; no RNG. Approval majority uses approvals_for_change / valid_ballots.
//! This module intentionally does not perform frontier mapping or tie handling.

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    ids::UnitId,
    rounding::{ge_percent, Ratio},
    variables::{BallotType, Params},
};

/// Minimal aggregate row used by gate math.
#[derive(Clone, Copy, Debug, Default)]
pub struct AggregateRow {
    pub ballots_cast: u64,
    pub invalid_or_blank: u64,
    pub valid_ballots: u64,
    pub eligible_roll: u64,
    pub approvals_for_change: Option<u64>, // present for approval ballots
}

#[derive(Clone, Debug, Default)]
pub struct AggregatesView {
    pub national: AggregateRow,
    pub by_region: BTreeMap<UnitId, AggregateRow>, // empty if not needed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DenomPolicy {
    /// Majority over valid ballots only.
    ValidBallots,
    /// Majority over valid + blank/invalid when VM-VAR-007 toggles this for gates.
    ValidPlusBlank,
    /// Approval support: approvals_for_change / valid_ballots (fixed).
    ApprovalRateValid,
}

#[derive(Clone, Copy, Debug)]
pub struct GateOutcome {
    pub observed: Ratio,   // exact rational; packaging converts to numbers later
    pub threshold_pct: u8, // integer percent
    pub pass: bool,
}

#[derive(Clone, Debug)]
pub struct DoubleOutcome {
    pub national: GateOutcome,
    pub family: GateOutcome,
    pub pass: bool,
    pub members: Vec<UnitId>, // affected family (canonical order)
}

#[derive(Clone, Debug)]
pub struct SymmetryOutcome {
    pub respected: bool,
    pub exceptions: Vec<String>, // codes from Params (VM-VAR-029), if any
}

#[derive(Clone, Debug)]
pub struct QuorumDetail {
    pub national: GateOutcome,                          // turnout Σ ballots_cast / Σ eligible_roll
    pub per_unit_flags: Option<BTreeMap<UnitId, bool>>, // per-unit turnout pass/fail if configured
}

#[derive(Clone, Debug)]
pub struct LegitimacyReport {
    pub pass: bool,
    pub reasons: Vec<String>, // stable machine-readable strings
    pub quorum: QuorumDetail,
    pub majority: GateOutcome,
    pub double_majority: Option<DoubleOutcome>,
    pub symmetry: Option<SymmetryOutcome>,
}

// -------------------------------------------------------------------------------------------------
// Public API
// -------------------------------------------------------------------------------------------------

/// Evaluate decision rules in fixed order. The returned `LegitimacyReport` is deterministic.
///
/// `per_unit_turnout` is optional input to annotate per-unit quorum flags (when configured).
pub fn apply_decision_rules(
    agg: &AggregatesView,
    p: &Params,
    per_unit_turnout: Option<&BTreeMap<UnitId, AggregateRow>>,
) -> LegitimacyReport {
    let mut reasons: Vec<String> = Vec::new();

    // A) Quorum (national + optional per-unit flags)
    let (quorum_detail, quorum_pass) = eval_quorum(agg, p, per_unit_turnout);
    if !quorum_pass {
        reasons.push("Quorum.NationalBelowThreshold".to_string());
    }

    // B) Majority / Supermajority (national)
    let (majority_outcome, maj_pass) = eval_majority(agg, p);
    if !maj_pass {
        reasons.push("Majority.BelowThreshold".to_string());
    }

    // C) Double-majority (optional)
    let (double_outcome_opt, dm_pass) = eval_double_majority(agg, p);
    if !dm_pass {
        // Be specific if we can, otherwise use a generic code.
        let code = if double_outcome_opt.is_none() && p.double_majority_enabled() {
            "DoubleMajority.FamilyUnresolved"
        } else if p.double_majority_enabled() {
            "DoubleMajority.BelowThreshold"
        } else {
            // Not enabled → this gate is not part of pass/fail.
            "DoubleMajority.Disabled"
        };
        if p.double_majority_enabled() {
            reasons.push(code.to_string());
        }
    }

    // D) Symmetry (optional)
    let symmetry_opt = eval_symmetry(p);
    let symmetry_ok = symmetry_opt.as_ref().map(|s| s.respected).unwrap_or(true);
    if !symmetry_ok {
        reasons.push("Symmetry.ExceptionsPresent".to_string());
    }

    // Overall pass: all enabled gates must pass.
    let overall_pass = quorum_pass && maj_pass && dm_pass && symmetry_ok;

    LegitimacyReport {
        pass: overall_pass,
        reasons,
        quorum: quorum_detail,
        majority: majority_outcome,
        double_majority: double_outcome_opt,
        symmetry: symmetry_opt,
    }
}

// -------------------------------------------------------------------------------------------------
// Helpers (pure, deterministic)
// -------------------------------------------------------------------------------------------------

/// ballots_cast / eligible_roll (den>0; if eligible_roll==0, treat as 0/1).
fn turnout_ratio(row: &AggregateRow) -> Ratio {
    let num = row.ballots_cast as i128;
    let den = if row.eligible_roll == 0 { 1 } else { row.eligible_roll } as i128;
    Ratio { num, den } // invariant den > 0
}

/// Determine national support ratio and the denominator policy.
/// - Approval ballots: approvals_for_change / valid_ballots (ApprovalRateValid).
/// - Otherwise: valid (or valid+blank if VM-VAR-007 on).
fn support_ratio_national(agg: &AggregatesView, p: &Params) -> (Ratio, DenomPolicy) {
    match p.ballot_type() {
        BallotType::Approval => {
            let num_u64 = agg.national.approvals_for_change.unwrap_or(0);
            let den_u64 = agg.national.valid_ballots;
            let den = if den_u64 == 0 { 1 } else { den_u64 };
            (
                Ratio {
                    num: num_u64 as i128,
                    den: den as i128,
                },
                DenomPolicy::ApprovalRateValid,
            )
        }
        _ => {
            let include_blank = p.include_blank_in_denominator();
            let valid = agg.national.valid_ballots;
            let den_u64 = if include_blank {
                agg.national.valid_ballots
                    .saturating_add(agg.national.invalid_or_blank)
            } else {
                valid
            };
            // For non-approval ballots, the numerator (support for change) must come from upstream
            // aggregation. If not provided at this stage, we conservatively treat it as 0.
            // Validation should ensure meaningful configuration before reaching here.
            let observed_num = agg.national.approvals_for_change.unwrap_or(0);
            let den = if den_u64 == 0 { 1 } else { den_u64 };
            let policy = if include_blank {
                DenomPolicy::ValidPlusBlank
            } else {
                DenomPolicy::ValidBallots
            };
            (
                Ratio {
                    num: observed_num as i128,
                    den: den as i128,
                },
                policy,
            )
        }
    }
}

/// Resolve affected family members per Params. In absence of a richer resolver,
/// default to all keys in `by_region` (canonical `UnitId` order) when double-majority is enabled.
fn family_units(agg: &AggregatesView, p: &Params) -> Vec<UnitId> {
    if !p.double_majority_enabled() {
        return Vec::new();
    }
    agg.by_region.keys().cloned().collect()
}

/// Family support ratio computed consistently with national policy.
/// When approval: sum approvals_for_change / sum valid.
/// Else: sum (numerators supplied upstream via approvals_for_change if applicable) / sum denom per policy.
fn support_ratio_family(agg: &AggregatesView, members: &[UnitId], p: &Params) -> (Ratio, DenomPolicy) {
    match p.ballot_type() {
        BallotType::Approval => {
            let mut num_sum: u128 = 0;
            let mut den_sum: u128 = 0;
            for u in members {
                if let Some(row) = agg.by_region.get(u) {
                    num_sum = num_sum.saturating_add(row.approvals_for_change.unwrap_or(0) as u128);
                    den_sum = den_sum.saturating_add(row.valid_ballots as u128);
                }
            }
            let den = if den_sum == 0 { 1 } else { den_sum } as i128;
            (
                Ratio {
                    num: num_sum as i128,
                    den,
                },
                DenomPolicy::ApprovalRateValid,
            )
        }
        _ => {
            let include_blank = p.include_blank_in_denominator();
            let mut num_sum: u128 = 0;
            let mut den_sum: u128 = 0;
            for u in members {
                if let Some(row) = agg.by_region.get(u) {
                    // As above: if upstream didn’t supply a numerator, treat as 0.
                    num_sum = num_sum.saturating_add(row.approvals_for_change.unwrap_or(0) as u128);
                    let unit_den = if include_blank {
                        row.valid_ballots.saturating_add(row.invalid_or_blank)
                    } else {
                        row.valid_ballots
                    } as u128;
                    den_sum = den_sum.saturating_add(unit_den);
                }
            }
            let den = if den_sum == 0 { 1 } else { den_sum } as i128;
            let policy = if include_blank {
                DenomPolicy::ValidPlusBlank
            } else {
                DenomPolicy::ValidBallots
            };
            (
                Ratio {
                    num: num_sum as i128,
                    den,
                },
                policy,
            )
        }
    }
}

/// Quorum evaluation: national + optional per-unit flags (when threshold > 0 and input provided).
fn eval_quorum(
    agg: &AggregatesView,
    p: &Params,
    per_unit_turnout: Option<&BTreeMap<UnitId, AggregateRow>>,
) -> (QuorumDetail, bool) {
    let q_nat_pct = p.quorum_global_pct();
    // National turnout
    let nat_ratio = turnout_ratio(&agg.national);
    let nat_pass = ge_percent(
        nat_ratio.num,
        nat_ratio.den,
        q_nat_pct,
    ).unwrap_or(false);

    // Per-unit flags only if configured and data is provided.
    let per_unit_pct = p.quorum_per_unit_pct();
    let per_unit_flags = if per_unit_pct > 0 {
        per_unit_turnout.map(|m| {
            let mut flags = BTreeMap::<UnitId, bool>::new();
            for (uid, row) in m.iter() {
                let r = turnout_ratio(row);
                let ok = ge_percent(r.num, r.den, per_unit_pct).unwrap_or(false);
                flags.insert(uid.clone(), ok);
            }
            flags
        })
    } else {
        None
    };

    let detail = QuorumDetail {
        national: GateOutcome {
            observed: nat_ratio,
            threshold_pct: q_nat_pct,
            pass: nat_pass,
        },
        per_unit_flags,
    };

    (detail, nat_pass)
}

/// National majority/supermajority gate.
fn eval_majority(agg: &AggregatesView, p: &Params) -> (GateOutcome, bool) {
    let (ratio, _policy) = support_ratio_national(agg, p);
    let pct = p.national_majority_pct();
    let pass = ge_percent(ratio.num, ratio.den, pct).unwrap_or(false);
    (
        GateOutcome {
            observed: ratio,
            threshold_pct: pct,
            pass,
        },
        pass,
    )
}

/// Optional double-majority gate. Returns (None, true) when not enabled.
fn eval_double_majority(agg: &AggregatesView, p: &Params) -> (Option<DoubleOutcome>, bool) {
    if !p.double_majority_enabled() {
        return (None, true);
    }

    // National outcome (reuse majority calculation for consistency).
    let (nat_ratio, _) = support_ratio_national(agg, p);
    let nat_pct = p.national_majority_pct();
    let nat_pass = ge_percent(nat_ratio.num, nat_ratio.den, nat_pct).unwrap_or(false);

    // Resolve family members; empty -> failure.
    let members = family_units(agg, p);
    if members.is_empty() {
        return (None, false);
    }

    let (fam_ratio, _) = support_ratio_family(agg, &members, p);
    let fam_pct = p.regional_majority_pct();
    let fam_pass = ge_percent(fam_ratio.num, fam_ratio.den, fam_pct).unwrap_or(false);

    let overall = nat_pass && fam_pass;
    let outcome = DoubleOutcome {
        national: GateOutcome {
            observed: nat_ratio,
            threshold_pct: nat_pct,
            pass: nat_pass,
        },
        family: GateOutcome {
            observed: fam_ratio,
            threshold_pct: fam_pct,
            pass: fam_pass,
        },
        pass: overall,
        members,
    };
    (Some(outcome), overall)
}

/// Optional symmetry check. If disabled, returns None (treated as respected).
fn eval_symmetry(p: &Params) -> Option<SymmetryOutcome> {
    if !p.symmetry_enabled() {
        return None;
    }
    // If exceptions are declared, we report respected = false.
    let exceptions: Vec<String> = p
        .symmetry_exceptions()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let respected = exceptions.is_empty();
    Some(SymmetryOutcome { respected, exceptions })
}
