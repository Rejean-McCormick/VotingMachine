//! crates/vm_pipeline/src/validate.rs — Part 1/3
//! Structural & semantic validation before any computation.
//!
//! This part defines public types, the top-level `validate()` entry,
//! the per-unit option ordering/uniqueness checks (fixed), and the
//! stable sorting utilities for deterministic output.

#![allow(clippy::result_large_err)]

use std::collections::{BTreeMap, BTreeSet};
use std::string::{String, ToString};
use std::vec::Vec;

use vm_core::{
    entities::{DivisionRegistry, OptionItem, Unit},
    ids::{OptionId, UnitId},
};

/// Context normalized by LOAD (paths → typed, canonical ordering, ids echoed).
/// Declared elsewhere in the pipeline; we only borrow it here.
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
/// This runs the foundational shape checks and the per-unit option checks.
/// Additional families of checks are added in Parts 2/3.
pub fn validate(ctx: &NormContext) -> ValidationReport {
    let mut issues: Vec<ValidationIssue> = Vec::new();

    // A) Registry tree & magnitudes & baseline pairing (stubs now, filled as fields land)
    issues.extend(check_registry_tree(ctx.reg));
    issues.extend(check_unit_magnitudes(&ctx.reg.units));
    issues.extend(check_baseline_pairing(&ctx.reg.units));

    // B) Options: canonical order & order_index uniqueness (per unit) — fixed codes
    for u in &ctx.reg.units {
        issues.extend(check_unit_options_order(u));
    }

    // (Other sections — params vs tally, tally sanity per ballot, WTA constraints,
    //  quorums, double-majority prerequisites, frontier prerequisites, tie seeds —
    //  are implemented in Parts 2/3.)

    // Deterministic sort of issues (by VM-VAR when applicable, then where, then message)
    sort_issues_stably(&mut issues);

    ValidationReport {
        pass: !issues.iter().any(|i| i.severity == Severity::Error),
        issues,
    }
}

/* ------------------------------------------------------------------------------------------------
 * Helpers / checks (this part)
 * ------------------------------------------------------------------------------------------------ */

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
///
/// Errors:
/// - "Option.OrderIndexDuplicate" on duplicate `order_index` within the *unit*
/// Warnings:
/// - "Option.OutOfOrder" if slice is not sorted by (order_index, OptionId)
fn check_unit_options_order(unit: &Unit) -> Vec<ValidationIssue> {
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
                code: "Option.OrderIndexDuplicate", // FIXED: standardized code
                message: format!(
                    "duplicate order_index {} for options {:?}",
                    idx, ids
                ),
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

/* ------------------------------------------------------------------------------------------------
 * Sorting / utilities (deterministic, stable across platforms)
 * ------------------------------------------------------------------------------------------------ */

pub(crate) fn sort_issues_stably(issues: &mut Vec<ValidationIssue>) {
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
/// NOTE: we format ids via Debug to avoid relying on Display implementations.
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

/* ------------------------------------------------------------------------------------------------
 * Tests for Part 1 (optional)
 * ------------------------------------------------------------------------------------------------ */
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

    #[test]
    fn dup_order_index_uses_fixed_code() {
        // Build a tiny unit with duplicate order_index
        let uid = UnitId::from("U:01");
        let o1 = OptionItem { option_id: OptionId::from("OPT:A"), order_index: 1, ..Default::default() };
        let o2 = OptionItem { option_id: OptionId::from("OPT:B"), order_index: 1, ..Default::default() };
        let unit = Unit { unit_id: uid.clone(), options: vec![o1, o2], ..Default::default() };

        let issues = check_unit_options_order(&unit);
        assert!(issues.iter().any(|i| i.code == "Option.OrderIndexDuplicate"));
    }
}
//! crates/vm_pipeline/src/validate.rs — Part 2/3
//! Additional helpers: slice-wide option ordering/uniqueness check
//! (used by callers that validate a consolidated option list).

use std::collections::BTreeSet;

use vm_core::{
    entities::OptionItem,
    ids::OptionId,
};

use super::{EntityRef, Severity, ValidationIssue};

/// Enforce canonical option ordering and unique/non-negative `order_index`
/// on a *slice* of options (not tied to a specific unit).
///
/// Errors:
/// - `"Option.OrderIndexDuplicate"` on duplicate `order_index`
///
/// Warnings:
/// - `"Option.OutOfOrder"` if slice is not sorted by `(order_index, OptionId)`
pub fn check_options_order(options: &[OptionItem]) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Duplicate order_index?
    let mut seen = BTreeSet::<u16>::new();
    for o in options {
        // non-negative is guaranteed by u16 type
        if !seen.insert(o.order_index) {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                code: "Option.OrderIndexDuplicate", // FIXED: standardized code
                message: format!("duplicate order_index {}", o.order_index),
                where_: EntityRef::Option(o.option_id.clone()),
            });
        }
    }

    // Sorted by (order_index, OptionId)?
    let mut prev: Option<(u16, &OptionId)> = None;
    for o in options {
        let key = (o.order_index, &o.option_id);
        if let Some(pk) = prev {
            if key < pk {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    code: "Option.OutOfOrder",
                    message: "options are not in canonical (order_index, option_id) order".to_string(),
                    where_: EntityRef::Option(o.option_id.clone()),
                });
                // keep scanning to collect all warnings
            }
        }
        prev = Some(key);
    }

    issues
}
//! crates/vm_pipeline/src/validate.rs — Part 3/3
//! Scaffolded validators for params↔tally shape, per-ballot sanity,
//! WTA/quorum/double-majority/frontier prerequisites, and tie-seed rules.
//!
//! NOTE: These functions are intentionally minimal to keep the file compiling
//! until the corresponding vm_core fields/types are introduced. Each returns
//! `Vec<ValidationIssue>` and is annotated with the error/warning codes that
//! should be used once implemented.

use super::{EntityRef, Severity, ValidationIssue};

/// Presence & domain checks for **Included** VM-VARs; shape match to tallies.
/// Planned error codes:
/// - E-PARAM-MISSING / E-PARAM-OUT-OF-DOMAIN
/// - E-PARAM-TALLY-SHAPE (mismatch between params & tallies)
#[allow(unused_variables)]
pub fn check_params_vs_tally(/*params: &vm_core::variables::Params,*/ /*tallies: &vm_core::tallies::UnitTallies*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Plurality: Sum(votes) ≤ valid_ballots; options FK to registry.
/// Planned error codes:
/// - E-TALLY-PLU-OVERFLOW (sum > valid)
/// - E-TALLY-PLU-UNKNOWN-OPTION (FK)
#[allow(unused_variables)]
pub fn check_tally_sanity_plurality(/*tallies: &vm_core::tallies::UnitTallies, options: &[vm_core::entities::OptionItem]*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Approval: each option approvals ≤ valid_ballots; sum approvals unconstrained; FK.
/// Planned error codes:
/// - E-TALLY-APP-OVERFLOW (option approvals > valid)
//  - E-TALLY-APP-UNKNOWN-OPTION (FK)
#[allow(unused_variables)]
pub fn check_tally_sanity_approval(/*tallies: &vm_core::tallies::UnitTallies, options: &[vm_core::entities::OptionItem]*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Score: per-option score_sums within domain (max_score * valid); FK.
/// Planned error codes:
/// - E-TALLY-SCO-OVERFLOW
/// - E-TALLY-SCO-UNKNOWN-OPTION (FK)
#[allow(unused_variables)]
pub fn check_tally_sanity_score(/*tallies: &vm_core::tallies::UnitTallies, options: &[vm_core::entities::OptionItem], params: &vm_core::variables::Params*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Ranked IRV: ballot multiplicities sum ≤ valid; ranks are unique & within options; FK.
/// Planned error codes:
/// - E-TALLY-RIRV-OVERFLOW
/// - E-TALLY-RIRV-DUP-RANK
/// - E-TALLY-RIRV-UNKNOWN-OPTION (FK)
#[allow(unused_variables)]
pub fn check_tally_sanity_ranked_irv(/*tallies: &vm_core::tallies::UnitTallies, options: &[vm_core::entities::OptionItem]*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Ranked Condorcet: same FK & rank constraints as IRV; pairwise matrix bounds.
/// Planned error codes:
/// - E-TALLY-RCND-BAD-MATRIX
/// - E-TALLY-RCND-UNKNOWN-OPTION (FK)
#[allow(unused_variables)]
pub fn check_tally_sanity_ranked_condorcet(/*tallies: &vm_core::tallies::UnitTallies, options: &[vm_core::entities::OptionItem]*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// WTA constraint: units using WinnerTakeAll must have magnitude == 1.
/// Planned error code:
/// - E-WTA-MAGNITUDE (per Unit)
#[allow(unused_variables)]
pub fn check_wta_constraint(/*units: &[vm_core::entities::Unit], params: &vm_core::variables::Params*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Quorum data presence/bounds: national + optional per-unit if configured.
/// Planned error codes:
/// - E-QUORUM-NATIONAL-MISSING / E-QUORUM-UNIT-MISSING
/// - E-QUORUM-OUT-OF-RANGE
#[allow(unused_variables)]
pub fn check_quorum_data(/*units: &[vm_core::entities::Unit], tallies: &vm_core::tallies::UnitTallies, params: &vm_core::variables::Params*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Double-majority prerequisites: regional families resolvable; thresholds coherent.
/// Planned error codes:
/// - E-DM-FAMILY-UNRESOLVED
/// - E-DM-THRESHOLD-INVALID
#[allow(unused_variables)]
pub fn check_double_majority_family(/*params: &vm_core::variables::Params, reg: &vm_core::entities::DivisionRegistry*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Frontier prerequisites (shape-level): bands, allowed edges, corridor/island policy toggles.
/// Planned error codes:
/// - E-FRONTIER-BANDS-EMPTY / E-FRONTIER-BAND-RANGE
/// - E-FRONTIER-EDGESET-EMPTY
#[allow(unused_variables)]
pub fn check_frontier_prereqs(/*params: &vm_core::variables::Params, reg: &vm_core::entities::DivisionRegistry*/) -> Vec<ValidationIssue> {
    Vec::new()
}

/// Tie seed policy: if tie_policy = "random" then VM-VAR-052 must be present and in range;
/// if tie_policy ≠ "random" seed must be ignored (but allowed to exist for logging).
/// Planned error/warning codes:
/// - E-TIE-SEED-MISSING (policy=random)
/// - E-TIE-SEED-OUT-OF-RANGE
/// - W-TIE-SEED-IGNORED (policy≠random)
#[allow(unused_variables)]
pub fn check_tie_seed(/*params: &vm_core::variables::Params*/) -> Vec<ValidationIssue> {
    Vec::new()
}
