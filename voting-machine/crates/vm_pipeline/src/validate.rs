//! crates/vm_pipeline/src/validate.rs
//! Structural & semantic validation before any computation.
//! Deterministic outputs; no RNG; pure integer reasoning.
//!
//! NOTE: This file implements the full reporting model and the
//! option-ordering checks now (enforceable with current vm_core types).
//! Other checks are scaffolded and will be filled as the corresponding
//! fields/types (tree parents, magnitudes, tallies, params) are wired.

#![allow(clippy::result_large_err)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use vm_core::{
    entities::{DivisionRegistry, OptionItem, Unit},
    ids::{OptionId, UnitId},
};

/// Context normalized by LOAD (paths → typed, canonical ordering, ids echoed).
/// This is declared elsewhere in the pipeline; we only borrow it here.
#[allow(dead_code)]
pub struct NormContext<'a> {
    pub reg: &'a DivisionRegistry,
    pub options: &'a [OptionItem],
    // pub params: &'a vm_core::variables::Params,
    // pub tallies: &'a vm_core::tallies::UnitTallies,
    // pub ids: NormIds,
}

/// Issue severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Where the issue occurred (kept small & deterministic).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntityRef {
    Root,
    Unit(UnitId),
    Option(OptionId),
    Param(&'static str),
    TallyUnit(UnitId),
    Adjacency(UnitId, UnitId),
}

/// One validation finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub where_: EntityRef,
}

/// Deterministic report: pass = (no Error); ordering of issues is stable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub pass: bool,
    pub issues: Vec<ValidationIssue>,
}

/// Top-level entry point.
pub fn validate(ctx: &NormContext) -> ValidationReport {
    let mut issues: Vec<ValidationIssue> = Vec::new();

    // A) Registry tree & magnitudes & baseline pairing (scaffold — fill as fields land)
    issues.extend(check_registry_tree(ctx.reg));
    issues.extend(check_unit_magnitudes(&ctx.reg.units));
    issues.extend(check_baseline_pairing(&ctx.reg.units));

    // B) Options: canonical order & order_index uniqueness (fully implemented now)
    issues.extend(check_options_order(ctx.options));

    // C) Params ↔ tally shape (scaffold)
    // if let (Some(params), Some(tallies)) = (ctx.params_opt, ctx.tallies_opt) { ... }
    // issues.extend(check_params_vs_tally(params, tallies));

    // D) Tally sanity per ballot type (scaffold)
    // issues.extend(check_tally_sanity_plurality(tallies, ctx.options));
    // issues.extend(check_tally_sanity_approval(tallies, ctx.options));
    // issues.extend(check_tally_sanity_score(tallies, ctx.options, params));
    // issues.extend(check_tally_sanity_ranked_irv(tallies, ctx.options));
    // issues.extend(check_tally_sanity_ranked_condorcet(tallies, ctx.options));

    // E) WTA constraint (scaffold)
    // issues.extend(check_wta_constraint(&ctx.reg.units, params));

    // F) Quorum data presence/bounds (scaffold)
    // issues.extend(check_quorum_data(&ctx.reg.units, tallies, params));

    // G) Double-majority family preconditions (scaffold)
    // issues.extend(check_double_majority_family(params, ctx.reg));

    // H) Frontier prerequisites (shape-level) (scaffold)
    // issues.extend(check_frontier_prereqs(params, ctx.reg));

    // I) RNG tie knobs (re-assert only) (scaffold)
    // issues.extend(check_tie_seed(params));

    // Deterministic sort of issues (by code, then where, then message) for byte-identical runs.
    sort_issues_stably(&mut issues);

    ValidationReport {
        pass: !issues.iter().any(|i| i.severity == Severity::Error),
        issues,
    }
}

// ------------------------------------------------------------------------------------------------
// Helpers / checks
// ------------------------------------------------------------------------------------------------

fn check_registry_tree(_reg: &DivisionRegistry) -> Vec<ValidationIssue> {
    // Current vm_core::entities::DivisionRegistry has no parent/root fields yet.
    // Stub to be filled once the tree representation lands.
    Vec::new()
}

fn check_unit_magnitudes(_units: &[Unit]) -> Vec<ValidationIssue> {
    // Current Unit has no magnitude field yet; fill once available.
    Vec::new()
}

fn check_baseline_pairing(_units: &[Unit]) -> Vec<ValidationIssue> {
    // Current Unit has no baseline fields; fill once (population_baseline, year) exist.
    Vec::new()
}

/// Enforce canonical option ordering and unique/non-negative order_index.
///
/// Errors:
/// - "Option.OrderIndexDuplicate" on duplicate `order_index`
/// Warnings:
/// - "Option.OutOfOrder" if slice is not sorted by (order_index, OptionId)
fn check_options_order(options: &[OptionItem]) -> Vec<ValidationIssue> {
    use alloc::collections::BTreeSet;

    let mut issues = Vec::new();

    // Duplicate order_index?
    let mut seen = BTreeSet::<u16>::new();
    for o in options {
        // non-negative is guaranteed by u16 type
        if !seen.insert(o.order_index) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "Option.OrderIndexDuplicate",
                message: format!("duplicate order_index {}", o.order_index),
                where_: EntityRef::Option(o.option_id.clone()),
            });
        }
    }

    // Sorted by (order_index, OptionId)?
    let mut prev: Option<(&OptionItem, (u16, &OptionId))> = None;
    for o in options {
        let key = (o.order_index, &o.option_id);
        if let Some((_po, pk)) = prev.as_ref() {
            if key < *pk {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    code: "Option.OutOfOrder",
                    message: "options are not in canonical (order_index, option_id) order".to_string(),
                    where_: EntityRef::Option(o.option_id.clone()),
                });
                // Keep scanning to collect all warnings
            }
        }
        prev = Some((o, key));
    }

    issues
}

// ------------------------------- Scaffolds for future data --------------------------------------

fn check_params_vs_tally(
    _params: &(), /* vm_core::variables::Params */
    _tallies: &(), /* vm_core::tallies::UnitTallies */
) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_tally_sanity_plurality(_tallies: &(), _options: &[OptionItem]) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_tally_sanity_approval(_tallies: &(), _options: &[OptionItem]) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_tally_sanity_score(
    _tallies: &(),
    _options: &[OptionItem],
    _params: &(), /* Params for scale/domain */
) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_tally_sanity_ranked_irv(_tallies: &(), _options: &[OptionItem]) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_tally_sanity_ranked_condorcet(
    _tallies: &(),
    _options: &[OptionItem],
) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_wta_constraint(_units: &[Unit], _params: &()) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_quorum_data(_units: &[Unit], _tallies: &(), _params: &()) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_double_majority_family(_params: &(), _reg: &DivisionRegistry) -> Vec<ValidationIssue> {
    Vec::new()
}

fn check_frontier_prereqs(_params: &(), _reg: &DivisionRegistry) -> Vec<ValidationIssue> {
    Vec::new()
}

// ------------------------------------------------------------------------------------------------
// Utilities
// ------------------------------------------------------------------------------------------------

fn sort_issues_stably(issues: &mut [ValidationIssue]) {
    use core::cmp::Ordering;
    issues.sort_by(|a, b| {
        // primary: code
        match a.code.cmp(b.code) {
            Ordering::Equal => {
                // secondary: where_
                match cmp_where(&a.where_, &b.where_) {
                    Ordering::Equal => {
                        // tertiary: message text
                        a.message.cmp(&b.message)
                    }
                    o => o,
                }
            }
            o => o,
        }
    });
}

fn cmp_where(a: &EntityRef, b: &EntityRef) -> core::cmp::Ordering {
    use EntityRef::*;
    use core::cmp::Ordering::*;
    match (a, b) {
        (Root, Root) => Equal,
        (Root, _) => Less,
        (_, Root) => Greater,
        (Param(pa), Param(pb)) => pa.cmp(pb),
        (Param(_), _) => Less,
        (_, Param(_)) => Greater,
        (Option(oa), Option(ob)) => oa.cmp(ob),
        (Option(_), _) => Less,
        (_, Option(_)) => Greater,
        (Unit(ua), Unit(ub)) => ua.cmp(ub),
        (Unit(_), _) => Less,
        (_, Unit(_)) => Greater,
        (TallyUnit(ua), TallyUnit(ub)) => ua.cmp(ub),
        (TallyUnit(_), _) => Less,
        (_, TallyUnit(_)) => Greater,
        (Adjacency(a1, a2), Adjacency(b1, b2)) => match a1.cmp(b1) {
            Equal => a2.cmp(b2),
            o => o,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vm_core::entities::OptionItem;

    #[test]
    fn detects_duplicate_order_index() {
        let opts = vec![
            OptionItem::new("A".parse().unwrap(), "A".into(), 0).unwrap(),
            OptionItem::new("B".parse().unwrap(), "B".into(), 0).unwrap(), // duplicate
        ];
        let issues = check_options_order(&opts);
        assert!(issues.iter().any(|i| i.code == "Option.OrderIndexDuplicate" && matches!(i.severity, Severity::Error)));
    }

    #[test]
    fn warns_on_out_of_order() {
        let opts = vec![
            OptionItem::new("B".parse().unwrap(), "B".into(), 0).unwrap(),
            OptionItem::new("A".parse().unwrap(), "A".into(), 0).unwrap(), // out of canonical order by id
        ];
        let issues = check_options_order(&opts);
        assert!(issues.iter().any(|i| i.code == "Option.OutOfOrder" && matches!(i.severity, Severity::Warning)));
    }

    #[test]
    fn ok_when_sorted_and_unique() {
        let opts = vec![
            OptionItem::new("A".parse().unwrap(), "A".into(), 0).unwrap(),
            OptionItem::new("B".parse().unwrap(), "B".into(), 1).unwrap(),
        ];
        let issues = check_options_order(&opts);
        assert!(issues.is_empty());
    }
}
