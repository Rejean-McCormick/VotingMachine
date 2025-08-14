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

    // B) Options: canonical order & order_index uniqueness (per unit)
    for u in &ctx.reg.units {
        issues.extend(check_unit_options_order(u));
    }

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

/// Per-unit option validations: uniqueness of `order_index` and canonical ordering.
fn check_unit_options_order(unit: &Unit) -> Vec<ValidationIssue> {
    use alloc::collections::{BTreeMap, BTreeSet};

    let mut issues = Vec::new();

    // A) order_index uniqueness within the unit (Doc 1B §4/§5; Doc 6A VM-TST-108)
    let mut seen: BTreeMap<u16, Vec<OptionId>> = BTreeMap::new();
    for o in &unit.options {
        seen.entry(o.order_index).or_default().push(o.option_id.clone());
    }
    for (idx, ids) in seen.into_iter() {
        if ids.len() > 1 {
            // Emit one error per duplicated index at the unit scope
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "E-DR-ORD-UNIQ",
                message: format!("unit has duplicate order_index {} for options {:?}", idx, ids),
                where_: EntityRef::Unit(unit.unit_id.clone()),
            });
        }
    }

    // B) canonical ordering check: (order_index, option_id)
    let mut prev: Option<(u16, &OptionId)> = None;
    for o in &unit.options {
        let key = (o.order_index, &o.option_id);
        if let Some(pk) = prev {
            if key < pk {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    code: "Option.OutOfOrder",
                    message: "options are not in canonical (order_index, option_id) order".to_string(),
                    where_: EntityRef::Unit(unit.unit_id.clone()),
                });
                // continue scanning to collect all warnings
            }
        }
        prev = Some(key);
    }

    issues
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
// ------------------------------------------------------------------------------------------------
// Sorting / utilities
// ------------------------------------------------------------------------------------------------

fn sort_issues_stably(issues: &mut Vec<ValidationIssue>) {
    issues.sort_by(|a, b| issue_sort_key(a).cmp(&issue_sort_key(b)));
}

/// Primary stable key: (vm_var_bucket, vm_var_num, code, where_key, message)
/// - vm_var_bucket: 0 if tied to a VM-VAR id (Param("VM-VAR-###")), else 1
/// - vm_var_num: parsed numeric id when present, else u16::MAX
fn issue_sort_key(i: &ValidationIssue) -> (u8, u16, &str, (u8, String), &str) {
    let (bucket, vmvar_num) = match &i.where_ {
        EntityRef::Param(name) => {
            if let Some(n) = parse_vm_var_num(name) {
                (0u8, n)
            } else {
                (1u8, u16::MAX)
            }
        }
        _ => (1u8, u16::MAX),
    };

    (bucket, vmvar_num, i.code, entity_ref_sort_key(&i.where_), i.message.as_str())
}

/// Extract a deterministic ordering key for EntityRef.
/// Variant rank ensures stable cross-run ordering, then a string key.
fn entity_ref_sort_key(r: &EntityRef) -> (u8, String) {
    match r {
        EntityRef::Root => (0, "root".to_string()),
        EntityRef::Unit(id) => (1, format!("unit:{:?}", id)),
        EntityRef::Option(id) => (2, format!("option:{:?}", id)),
        EntityRef::Param(name) => (3, format!("param:{name}")),
        EntityRef::TallyUnit(id) => (4, format!("tally_unit:{:?}", id)),
        EntityRef::Adjacency(a, b) => (5, format!("adj:{:?}->{:?}", a, b)),
    }
}

/// Parse "VM-VAR-###" → ### (u16). Returns None if not in the expected form.
fn parse_vm_var_num(s: &str) -> Option<u16> {
    const PREFIX: &str = "VM-VAR-";
    if !s.starts_with(PREFIX) {
        return None;
    }
    let digits = &s[PREFIX.len()..];
    if digits.is_empty() || digits.len() > 5 {
        return None;
    }
    digits.parse::<u16>().ok()
}

// ------------------------------------------------------------------------------------------------
// Scaffolds (placeholders to be filled as more fields/types land in vm_core)
// ------------------------------------------------------------------------------------------------

#[allow(unused_variables)]
fn check_params_vs_tally(/*params: &Params,*/ /*tallies: &UnitTallies*/) -> Vec<ValidationIssue> {
    // TODO: Presence & domain checks for Included VM-VARs; shape match to tallies.
    Vec::new()
}

#[allow(unused_variables)]
fn check_tally_sanity_plurality(/*...*/) -> Vec<ValidationIssue> {
    // TODO: Sum(votes) <= valid_ballots; FK to registry.
    Vec::new()
}

#[allow(unused_variables)]
fn check_tally_sanity_approval(/*...*/) -> Vec<ValidationIssue> {
    Vec::new()
}

#[allow(unused_variables)]
fn check_tally_sanity_score(/*...*/) -> Vec<ValidationIssue> {
    Vec::new()
}

#[allow(unused_variables)]
fn check_tally_sanity_ranked_irv(/*...*/) -> Vec<ValidationIssue> {
    Vec::new()
}

#[allow(unused_variables)]
fn check_tally_sanity_ranked_condorcet(/*...*/) -> Vec<ValidationIssue> {
    Vec::new()
}

#[allow(unused_variables)]
fn check_wta_constraint(/*units: &[Unit], params: &Params*/) -> Vec<ValidationIssue> {
    Vec::new()
}

#[allow(unused_variables)]
fn check_quorum_data(/*units: &[Unit], tallies: &UnitTallies, params: &Params*/) -> Vec<ValidationIssue> {
    Vec::new()
}

#[allow(unused_variables)]
fn check_double_majority_family(/*params: &Params, reg: &DivisionRegistry*/) -> Vec<ValidationIssue> {
    Vec::new()
}

#[allow(unused_variables)]
fn check_frontier_prereqs(/*params: &Params, reg: &DivisionRegistry*/) -> Vec<ValidationIssue> {
    // TODO: Ensure frontier variables are present/enabled only with required inputs.
    Vec::new()
}

#[allow(unused_variables)]
fn check_tie_seed(/*params: &Params*/) -> Vec<ValidationIssue> {
    // TODO: If tie_policy=random then tie_seed must be present and within domain; otherwise ignored.
    Vec::new()
}

// ------------------------------------------------------------------------------------------------
// (optional) tests — can be kept or removed based on your policy
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vm_var_num_ok() {
        assert_eq!(parse_vm_var_num("VM-VAR-050"), Some(50));
        assert_eq!(parse_vm_var_num("VM-VAR-5"), Some(5));
    }

    #[test]
    fn parse_vm_var_num_bad() {
        assert_eq!(parse_vm_var_num("VAR-050"), None);
        assert_eq!(parse_vm_var_num("VM-VAR-"), None);
        assert_eq!(parse_vm_var_num("VM-VAR-99999"), None);
    }

    #[test]
    fn sort_vmvar_first() {
        let mut issues = vec![
            ValidationIssue {
                severity: Severity::Error,
                code: "E-OTHER",
                message: "x".into(),
                where_: EntityRef::Unit(UnitId::from_raw(2)),
            },
            ValidationIssue {
                severity: Severity::Error,
                code: "E-PARAM",
                message: "y".into(),
                where_: EntityRef::Param("VM-VAR-060"),
            },
            ValidationIssue {
                severity: Severity::Error,
                code: "E-PARAM",
                message: "z".into(),
                where_: EntityRef::Param("VM-VAR-010"),
            },
        ];
        sort_issues_stably(&mut issues);
        // VM-VAR-010 should come before VM-VAR-060, both before the Unit-scoped issue.
        assert!(matches!(issues[0].where_, EntityRef::Param("VM-VAR-010")));
        assert!(matches!(issues[1].where_, EntityRef::Param("VM-VAR-060")));
    }
}
