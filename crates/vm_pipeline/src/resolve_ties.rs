//! Final decisiveness labeling (Decisive | Marginal | Invalid).
//!
//! Inputs are gate outcomes, national margin vs threshold (pp), and optional
//! frontier-risk flags. No I/O, no RNG. Deterministic across platforms.

#[cfg(feature = "use_smol_str")]
use smol_str::SmolStr;
#[cfg(not(feature = "use_smol_str"))]
type SmolStr = String;

//
// Public types (minimal mirrors of upstream stages)
//

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Label {
    Decisive,
    Marginal,
    Invalid,
}

#[derive(Clone, Debug)]
pub struct DecisivenessLabel {
    pub label: Label,
    /// Short, machine-readable reason (snake_case).
    pub reason: SmolStr,
}

/// Minimal mirror of the gate report we need here.
pub struct LegitimacyReport {
    pub pass: bool,
    pub reasons: Vec<String>, // if pass=false, first item should explain failure
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct FrontierFlags {
    pub mediation_flagged: bool,
    pub enclave: bool,
    pub protected_override_used: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct LabelConfig {
    /// VM-VAR-062: minimum national margin (pp) required for "Decisive".
    pub decisive_margin_pp: i32,
}

//
// Public API
//

/// Main entry (explicit threshold provided via LabelConfig).
pub fn label_decisiveness_cfg(
    legit: &LegitimacyReport,
    national_margin_pp: i32,
    frontier_flags: Option<&FrontierFlags>,
    cfg: LabelConfig,
) -> DecisivenessLabel {
    // 1) If gates failed → Invalid
    if !legit.pass {
        return DecisivenessLabel {
            label: Label::Invalid,
            reason: first_failure_reason(legit),
        };
    }

    // 2) Margin & frontier risk logic
    if national_margin_pp < cfg.decisive_margin_pp {
        return DecisivenessLabel {
            label: Label::Marginal,
            reason: SmolStr::from("margin_below_threshold"),
        };
    }

    if has_frontier_risk(frontier_flags) {
        return DecisivenessLabel {
            label: Label::Marginal,
            reason: SmolStr::from("frontier_risk_flags_present"),
        };
    }

    DecisivenessLabel {
        label: Label::Decisive,
        reason: SmolStr::from("margin_meets_threshold"),
    }
}

/// Convenience entry that reads VM-VAR-062 from Params.
pub fn label_decisiveness(
    legit: &LegitimacyReport,
    national_margin_pp: i32,
    frontier_flags: Option<&FrontierFlags>,
    params: &vm_core::variables::Params,
) -> DecisivenessLabel {
    let cfg = LabelConfig {
        decisive_margin_pp: params.decisive_margin_pp(),
    };
    label_decisiveness_cfg(legit, national_margin_pp, frontier_flags, cfg)
}

//
// Internal helpers (pure)
//

fn first_failure_reason(legit: &LegitimacyReport) -> SmolStr {
    if let Some(first) = legit.reasons.first() {
        return SmolStr::from(first.as_str());
    }
    SmolStr::from("gates_failed")
}

fn has_frontier_risk(ff: Option<&FrontierFlags>) -> bool {
    if let Some(f) = ff {
        f.mediation_flagged || f.enclave || f.protected_override_used
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lr(pass: bool, reasons: &[&str]) -> LegitimacyReport {
        LegitimacyReport {
            pass,
            reasons: reasons.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn gates_fail_yields_invalid() {
        let legit = lr(false, &["Quorum.NationalBelowThreshold"]);
        let out = label_decisiveness_cfg(&legit, 10, None, LabelConfig { decisive_margin_pp: 5 });
        assert_eq!(out.label, Label::Invalid);
        assert_eq!(out.reason, "Quorum.NationalBelowThreshold");
    }

    #[test]
    fn margin_below_threshold_is_marginal() {
        let legit = lr(true, &[]);
        let out = label_decisiveness_cfg(&legit, 4, None, LabelConfig { decisive_margin_pp: 5 });
        assert_eq!(out.label, Label::Marginal);
        assert_eq!(out.reason, "margin_below_threshold");
    }

    #[test]
    fn equal_to_threshold_is_decisive_when_no_risk() {
        let legit = lr(true, &[]);
        let out = label_decisiveness_cfg(&legit, 5, None, LabelConfig { decisive_margin_pp: 5 });
        assert_eq!(out.label, Label::Decisive);
        assert_eq!(out.reason, "margin_meets_threshold");
    }

    #[test]
    fn frontier_risk_forces_marginal() {
        let legit = lr(true, &[]);
        let ff = FrontierFlags { mediation_flagged: true, enclave: false, protected_override_used: false };
        let out = label_decisiveness_cfg(&legit, 12, Some(&ff), LabelConfig { decisive_margin_pp: 5 });
        assert_eq!(out.label, Label::Marginal);
        assert_eq!(out.reason, "frontier_risk_flags_present");
    }

    #[test]
    fn fallback_reason_when_missing() {
        let legit = lr(false, &[]);
        let out = label_decisiveness_cfg(&legit, 0, None, LabelConfig { decisive_margin_pp: 0 });
        assert_eq!(out.label, Label::Invalid);
        assert_eq!(out.reason, "gates_failed");
    }
}
