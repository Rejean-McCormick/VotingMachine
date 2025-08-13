//! BUILD_RESULT — compose the Result artifact from prior pipeline stages.
//!
//! Deterministic assembly only: no I/O and no hashing here. The caller is
//! responsible for canonical serialization + hashing and assigning the
//! Result ID afterwards.

use std::collections::BTreeMap;

use vm_core::{
    ids::{OptionId, UnitId},
    rounding::Ratio,
};

use crate::apply_rules::LegitimacyReport;
use crate::label::{DecisivenessLabel, Label};

/// Engine-level numeric precision when converting exact ratios to JSON numbers.
/// (This is *not* a display precision; callers may pretty-print later.)
const ENGINE_SHARE_PRECISION: f64 = 1e-9;

// ---------- Public surface ----------

/// Opaque FrontierMap ID (e.g., "FR:<hex64>").
pub type FrontierId = String;

/// Error composing Result (kept tiny and deterministic).
#[derive(Debug)]
pub enum BuildError {
    MissingFormulaId,
}

/// Compose a Result (without IDs/hashes). Caller will assign `id` after hashing.
pub fn build_result(
    formula_id: &str,
    agg: &AggregateResults,
    gates: &LegitimacyReport,
    label: &DecisivenessLabel,
    frontier_id: Option<FrontierId>,
) -> Result<ResultDoc, BuildError> {
    if formula_id.is_empty() {
        return Err(BuildError::MissingFormulaId);
    }

    let units = write_unit_blocks(agg);
    let aggregates = write_aggregates_as_numbers(agg);
    let gates_out = write_gates_as_numbers(gates);

    let label_str = match label.label {
        Label::Decisive => "Decisive",
        Label::Marginal => "Marginal",
        Label::Invalid => "Invalid",
    }
    .to_string();

    let mut out = ResultDoc {
        id: None, // assigned by caller after canonicalization + hashing
        formula_id: formula_id.to_string(),
        label: label_str,
        label_reason: label.reason.clone().into(),
        units,
        aggregates,
        gates: gates_out,
        frontier_map_id: None,
    };

    if let Some(fid) = frontier_id {
        out.frontier_map_id = Some(fid);
    }

    Ok(out)
}

// ---------- Conversion helpers ----------

/// Build per-unit blocks in stable UnitId order.
pub fn write_unit_blocks(agg: &AggregateResults) -> Vec<UnitBlock> {
    let mut out = Vec::with_capacity(agg.units.len());
    for (unit_id, u) in &agg.units {
        // Scores / allocation maps are already BTreeMap for stable key order.
        out.push(UnitBlock {
            unit_id: unit_id.clone(),
            turnout: u.turnout.clone(),
            scores: u.scores.clone(),
            allocation: u.allocation.clone(),
            flags: u.flags.clone(),
        });
    }
    out
}

/// Convert exact ratios in aggregates to JSON numbers at engine precision.
pub fn write_aggregates_as_numbers(agg: &AggregateResults) -> AggregatesOut {
    let shares_num = agg
        .shares
        .iter()
        .map(|(opt, r)| (opt.clone(), ratio_to_number(r)))
        .collect::<BTreeMap<_, _>>();

    AggregatesOut {
        totals: agg.totals.clone(),
        shares: shares_num,
        turnout: agg.turnout.clone(),
        weighting_method: agg.weighting_method.clone(),
    }
}

/// Convert gate outcomes’ observed ratios to JSON numbers.
pub fn write_gates_as_numbers(g: &LegitimacyReport) -> GatesOut {
    let quorum = GatePanel {
        observed: ratio_to_number(&g.quorum.national.observed),
        threshold_pct: g.quorum.national.threshold_pct,
        pass: g.quorum.national.pass,
    };

    let majority = GatePanel {
        observed: ratio_to_number(&g.majority.observed),
        threshold_pct: g.majority.threshold_pct,
        pass: g.majority.pass,
    };

    let double_majority = g.double_majority.as_ref().map(|dm| DoubleMajorityOut {
        national: GatePanel {
            observed: ratio_to_number(&dm.national.observed),
            threshold_pct: dm.national.threshold_pct,
            pass: dm.national.pass,
        },
        regional: GatePanel {
            observed: ratio_to_number(&dm.family.observed),
            threshold_pct: dm.family.threshold_pct,
            pass: dm.family.pass,
        },
        pass: dm.pass,
    });

    let symmetry = g
        .symmetry
        .as_ref()
        .map(|s| SymmetryOut { pass: s.respected });

    GatesOut {
        quorum,
        majority,
        double_majority,
        symmetry,
    }
}

// ---------- Local numeric helper ----------

#[inline]
fn ratio_to_number(r: &Ratio) -> f64 {
    // Exact conversion to f64 (engine precision governs consumer expectations).
    // Guard den==0 defensively: treat as 0.0 (validate should prevent this).
    if r.den == 0 {
        return 0.0;
    }
    let v = (r.num as f64) / (r.den as f64);
    // Optionally snap tiny negatives/overflows to 0..1 range if desired:
    // keep as-is for transparency; callers can clamp for presentation.
    (v / ENGINE_SHARE_PRECISION).round() * ENGINE_SHARE_PRECISION
}

// ============================================================================
// Minimal, schema-shaped types for this builder
// (Replace with your canonical crate types if you already have them.)
// ============================================================================

/// Caller attaches canonicalized + hashed id (e.g., "RES:<hex64>") later.
#[derive(Clone, Debug)]
pub struct ResultDoc {
    pub id: Option<String>, // assigned by caller
    pub formula_id: String, // REQUIRED
    pub label: String,      // "Decisive" | "Marginal" | "Invalid"
    pub label_reason: String,
    pub units: Vec<UnitBlock>,
    pub aggregates: AggregatesOut,
    pub gates: GatesOut,
    pub frontier_map_id: Option<FrontierId>,
}

/// One unit’s block in the Result.
#[derive(Clone, Debug)]
pub struct UnitBlock {
    pub unit_id: UnitId,
    pub turnout: UnitTurnoutOut,
    pub scores: BTreeMap<OptionId, u64>,
    pub allocation: AllocationOut,
    pub flags: UnitFlagsOut,
}

/// Allocation representation.
/// - PR: `seats` present, sum equals unit magnitude.
/// - WTA: `power_pct` == 100 for the winner (winner OptionId included for clarity).
#[derive(Clone, Debug)]
pub struct AllocationOut {
    pub seats: Option<BTreeMap<OptionId, u32>>,
    pub wta_winner: Option<OptionId>,
    pub power_pct: Option<u32>, // typically 100 when WTA is used
}

#[derive(Clone, Debug, Default)]
pub struct UnitFlagsOut {
    pub unit_data_ok: bool,
    pub unit_quorum_met: bool,
    pub unit_pr_threshold_met: bool,
    pub protected_override_used: bool,
    pub mediation_flagged: bool,
}

#[derive(Clone, Debug, Default)]
pub struct UnitTurnoutOut {
    pub ballots_cast: u64,
    pub invalid_or_blank: u64,
    pub valid_ballots: u64,
}

/// Aggregates: totals + shares as *numbers* and turnout.
#[derive(Clone, Debug)]
pub struct AggregatesOut {
    pub totals: BTreeMap<OptionId, u64>,
    pub shares: BTreeMap<OptionId, f64>, // numbers, not {num,den}
    pub turnout: AggregatedTurnoutOut,
    pub weighting_method: String,
}

#[derive(Clone, Debug, Default)]
pub struct AggregatedTurnoutOut {
    pub ballots_cast: u64,
    pub invalid_or_blank: u64,
    pub valid_ballots: u64,
    pub eligible_roll: u64,
}

/// Gates panel — observed values as *numbers*.
#[derive(Clone, Debug)]
pub struct GatesOut {
    pub quorum: GatePanel,
    pub majority: GatePanel,
    pub double_majority: Option<DoubleMajorityOut>,
    pub symmetry: Option<SymmetryOut>,
}

#[derive(Clone, Debug)]
pub struct GatePanel {
    pub observed: f64,     // number at engine precision
    pub threshold_pct: u8, // integer percent
    pub pass: bool,
}

#[derive(Clone, Debug)]
pub struct DoubleMajorityOut {
    pub national: GatePanel,
    pub regional: GatePanel,
    pub pass: bool,
}

#[derive(Clone, Debug)]
pub struct SymmetryOut {
    pub pass: bool,
}

// ============================================================================
// Minimal AggregateResults input model (produced upstream by AGGREGATE).
// Replace with your canonical type if already defined.
// ============================================================================

#[derive(Clone, Debug)]
pub struct AggregateResults {
    /// Per-unit view in stable UnitId order (BTreeMap guarantees determinism).
    pub units: BTreeMap<UnitId, UnitAggregate>,
    /// National totals by option (integers).
    pub totals: BTreeMap<OptionId, u64>,
    /// National shares by option (exact ratios).
    pub shares: BTreeMap<OptionId, Ratio>,
    /// National turnout row.
    pub turnout: AggregatedTurnoutOut,
    /// Echo of weighting method (e.g., VM-VAR-030).
    pub weighting_method: String,
}

#[derive(Clone, Debug)]
pub struct UnitAggregate {
    pub turnout: UnitTurnoutOut,
    pub scores: BTreeMap<OptionId, u64>,
    pub allocation: AllocationOut,
    pub flags: UnitFlagsOut,
}

// ============================================================================
// Tests (narrow, deterministic)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn ratio(n: i64, d: i64) -> Ratio {
        Ratio { num: n, den: d }
    }

    #[test]
    fn ratios_convert_to_numbers() {
        assert!((ratio_to_number(&ratio(1, 2)) - 0.5).abs() < 1e-12);
        assert!((ratio_to_number(&ratio(2, 3)) - (2.0 / 3.0)).abs() < 1e-9);
        assert_eq!(ratio_to_number(&ratio(0, 0)), 0.0); // defensive
    }

    #[test]
    fn builds_minimal_result() {
        let unit = UnitAggregate {
            turnout: UnitTurnoutOut {
                ballots_cast: 100,
                invalid_or_blank: 0,
                valid_ballots: 100,
            },
            scores: BTreeMap::new(),
            allocation: AllocationOut {
                seats: None,
                wta_winner: None,
                power_pct: None,
            },
            flags: UnitFlagsOut {
                unit_data_ok: true,
                ..Default::default()
            },
        };

        let mut units = BTreeMap::new();
        let uid = UnitId::from("U:001");
        units.insert(uid.clone(), unit);

        let agg = AggregateResults {
            units,
            totals: BTreeMap::new(),
            shares: BTreeMap::new(),
            turnout: AggregatedTurnoutOut {
                ballots_cast: 100,
                invalid_or_blank: 0,
                valid_ballots: 100,
                eligible_roll: 120,
            },
            weighting_method: "natural".into(),
        };

        // Fake-up a tiny legitimacy report with passing gates
        // (shapes mirror crate::apply_rules types).
        let gates = crate::apply_rules::LegitimacyReport {
            pass: true,
            reasons: vec![],
            quorum: crate::apply_rules::QuorumDetail {
                national: crate::apply_rules::GateOutcome {
                    observed: ratio(100, 120),
                    threshold_pct: 50,
                    pass: true,
                },
                per_unit_flags: None,
            },
            majority: crate::apply_rules::GateOutcome {
                observed: ratio(55, 100),
                threshold_pct: 50,
                pass: true,
            },
            double_majority: None,
            symmetry: None,
        };

        let label = DecisivenessLabel {
            label: Label::Decisive,
            reason: "margin_meets_threshold".into(),
        };

        let res = build_result("abcdef", &agg, &gates, &label, None).expect("ok");
        assert_eq!(res.formula_id, "abcdef");
        assert_eq!(res.label, "Decisive");
        assert!(res.frontier_map_id.is_none());
        assert_eq!(res.units.len(), 1);
    }
}
