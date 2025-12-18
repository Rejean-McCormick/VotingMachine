//! crates/vm_algo/src/gates_frontier.rs
//! Decision gates (quorum → national majority → optional double-majority → symmetry)
//! and, when passed, frontier mapping (bands, contiguity & flags). Pure integer math,
//! deterministic ordering, no RNG (ties live elsewhere).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;

use vm_core::{
    ids::UnitId,
    rounding::ge_percent, // integer test: a/b >= p%
    variables::Params,
};

// ---------------- Types -------------------------------------------------------------------------

/// Inputs required by the gate checks (aggregates + per-unit basics).
#[derive(Clone, Debug, Default)]
pub struct GateInputs {
    pub nat_ballots_cast: u64,
    pub nat_invalid_ballots: u64, // currently unused in gates; keep if policy evolves
    pub nat_valid_ballots: u64,
    pub nat_eligible_roll: u64,

    /// Per-region valid ballots and support for change (aggregated upstream).
    pub region_valid_ballots: BTreeMap<String, u64>,
    pub region_support_for_change: BTreeMap<String, u64>,

    /// Per-unit basics for quorum and support.
    pub unit_valid_ballots: BTreeMap<UnitId, u64>,
    pub unit_ballots_cast: BTreeMap<UnitId, u64>,
    pub unit_eligible_roll: BTreeMap<UnitId, u64>,
    /// Numerator for approval/support rate per unit (denominator is valid_ballots, per spec).
    pub unit_support_for_change: BTreeMap<UnitId, u64>,
}

/// Outcome of decision gates (used by pipeline to decide whether to run frontier).
#[derive(Clone, Debug, Default)]
pub struct GateResult {
    pub quorum_national: bool,
    /// Units meeting per-unit quorum (021). A policy outside this module decides how to use it.
    pub quorum_per_unit_passset: BTreeSet<UnitId>,
    pub majority_national: bool,
    pub majority_regional: bool, // meaningful iff double_majority enabled
    pub double_majority: bool,
    pub symmetry: bool,
    pub pass: bool,
}

// ---------------- Param access view -------------------------------------------------------------
//
// NOTE: These helpers assume `Params` exposes deterministic getters for the needed VM-VARs.
// Exact domains/defaults live in Annex A and Doc 2; this module reads them only at the
// documented touchpoints (4B gates, 4C frontier). See docs cited in the file header.

pub trait GatesFrontierParamView {
    // 020–029 (gates)
    fn quorum_global_pct_020(&self) -> u8;
    fn quorum_per_unit_pct_021(&self) -> u8;
    fn national_majority_pct_022(&self) -> u8;

    /// Regional majority cutoff (023). Applied per-region to compute a count of "passing" regions.
    fn regional_majority_pct_023(&self) -> u8;

    /// Whether regional double-majority is enabled (024).
    fn double_majority_enabled_024(&self) -> bool;

    /// Symmetry toggle (025). Policy for exceptions lives in params (029).
    fn symmetry_enabled_025(&self) -> bool;
    fn symmetry_breaks_due_to_exceptions_029(&self) -> bool;

    // 040–042, 047–049 (frontier) — used in part 2
    fn frontier_mode_is_none_040(&self) -> bool;
    /// Ordered, non-overlapping bands: (min_pct, max_pct, status)
    fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, String)>;
    fn frontier_allow_land_047(&self) -> bool;
    fn frontier_allow_bridge_047(&self) -> bool;
    fn frontier_allow_water_047(&self) -> bool;
    fn frontier_island_rule_ferry_allowed_048(&self) -> bool;
}

// Blanket impl forwards to `Params`. Implement the getters in vm_core.
impl GatesFrontierParamView for Params {
    #[inline] fn quorum_global_pct_020(&self) -> u8 { self.quorum_global_pct_020() }
    #[inline] fn quorum_per_unit_pct_021(&self) -> u8 { self.quorum_per_unit_pct_021() }
    #[inline] fn national_majority_pct_022(&self) -> u8 { self.national_majority_pct_022() }
    #[inline] fn regional_majority_pct_023(&self) -> u8 { self.regional_majority_pct_023() }
    #[inline] fn double_majority_enabled_024(&self) -> bool { self.double_majority_enabled_024() }
    #[inline] fn symmetry_enabled_025(&self) -> bool { self.symmetry_enabled_025() }
    #[inline] fn symmetry_breaks_due_to_exceptions_029(&self) -> bool { self.symmetry_breaks_due_to_exceptions_029() }

    #[inline] fn frontier_mode_is_none_040(&self) -> bool { self.frontier_mode_is_none_040() }
    #[inline] fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, String)> { self.frontier_bands_042() }
    #[inline] fn frontier_allow_land_047(&self) -> bool { self.frontier_allow_land_047() }
    #[inline] fn frontier_allow_bridge_047(&self) -> bool { self.frontier_allow_bridge_047() }
    #[inline] fn frontier_allow_water_047(&self) -> bool { self.frontier_allow_water_047() }
    #[inline] fn frontier_island_rule_ferry_allowed_048(&self) -> bool { self.frontier_island_rule_ferry_allowed_048() }
}

// ---------------- Gates (020–029; majority uses valid_ballots as denominator) -------------------

/// Apply quorum + majority (+ optional double-majority & symmetry) deterministically.
pub fn apply_decision_gates(inp: &GateInputs, p: &impl GatesFrontierParamView) -> GateResult {
    // Quorum (national): Σ ballots_cast / Σ eligible_roll ≥ 020
    let quorum_nat = compute_quorum_national(
        inp.nat_ballots_cast,
        inp.nat_eligible_roll,
        p.quorum_global_pct_020(),
    );

    // Per-unit quorum (021): collect pass set
    let quorum_set = compute_quorum_per_unit(
        &inp.unit_ballots_cast,
        &inp.unit_eligible_roll,
        p.quorum_per_unit_pct_021(),
    );

    // National approval majority (022): approvals_for_change / valid_ballots ≥ cutoff
    // Denominator is *valid_ballots* (explicitly ignores blank toggle for approval majority).
    let nat_support_sum: u64 = inp.unit_support_for_change.values().copied().sum();
    let maj_nat = national_approval_majority(
        inp.nat_valid_ballots,
        nat_support_sum,
        p.national_majority_pct_022(),
    );

    // Double-majority (024/023) over regions:
    // Policy here = "majority in a majority of regions" (common scheme):
    // - For each region with v_r > 0, mark pass_r = support_r / valid_r ≥ cutoff_023.
    // - maj_regional = passed_regions ≥ ceil(R/2). If no regions with v_r > 0, treat as N/A → pass.
    let maj_regional = if p.double_majority_enabled_024() {
        let mut regions = 0u64;
        let mut passed = 0u64;
        for (rid, v_r) in &inp.region_valid_ballots {
            if *v_r == 0 {
                continue;
            }
            regions += 1;
            let s_r = *inp.region_support_for_change.get(rid).unwrap_or(&0);
            if ge_percent(s_r, *v_r, p.regional_majority_pct_023()) {
                passed += 1;
            }
        }
        if regions == 0 {
            true // N/A ⇒ pass (do not accidentally fail the gate when regions are not configured)
        } else {
            // majority-of-regions test
            let half_up = (regions + 1) / 2;
            passed >= half_up
        }
    } else {
        // regional component disabled
        true
    };

    let dbl = maj_nat && maj_regional;

    // Symmetry (025/029). If enabled, rely on params to flag exceptions that break symmetry.
    // With no explicit exception policy here, we accept symmetry when there is no flagged break.
    let symmetry = if p.symmetry_enabled_025() {
        !p.symmetry_breaks_due_to_exceptions_029()
    } else {
        true
    };

    let pass = quorum_nat && dbl && symmetry;

    GateResult {
        quorum_national: quorum_nat,
        quorum_per_unit_passset: quorum_set,
        majority_national: maj_nat,
        majority_regional: maj_regional,
        double_majority: dbl,
        symmetry,
        pass,
    }
}

/// turnout ≥ cutoff? (a/b ≥ p%)
#[inline]
fn compute_quorum_national(ballots_cast: u64, eligible_roll: u64, cutoff_pct: u8) -> bool {
    eligible_roll > 0 && ge_percent(ballots_cast, eligible_roll, cutoff_pct)
}

/// For each unit, test turnout ≥ cutoff and return the pass set.
/// NOTE: iterate the **eligible roll** as ground truth; treat missing ballots_cast as 0.
fn compute_quorum_per_unit(
    unit_ballots_cast: &BTreeMap<UnitId, u64>,
    unit_eligible_roll: &BTreeMap<UnitId, u64>,
    cutoff_pct: u8,
) -> BTreeSet<UnitId> {
    let mut out = BTreeSet::new();
    for (u, roll) in unit_eligible_roll {
        let cast = *unit_ballots_cast.get(u).unwrap_or(&0);
        if *roll > 0 && ge_percent(cast, *roll, cutoff_pct) {
            out.insert(u.clone());
        }
    }
    out
}

/// approval majority is approvals_for_change / valid_ballots ≥ cutoff (fixed denominator).
#[inline]
fn national_approval_majority(valid_ballots: u64, approvals_for_change: u64, cutoff_pct: u8) -> bool {
    valid_ballots > 0 && ge_percent(approvals_for_change, valid_ballots, cutoff_pct)
}

// --------- Frontier types (used in part 2) -----------------------------------------------------

/// Edge kinds allowed when checking contiguity (subset chosen by VM-VAR-047/048).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum FrontierEdge {
    Land,
    Bridge,
    Water,
}

/// Per-unit flags emitted with the assigned frontier status.
#[derive(Clone, Debug, Default)]
pub struct FrontierFlags {
    pub contiguity_ok: bool,
    pub mediation_flagged: bool,
    pub protected_override_used: bool,
    pub enclave: bool,
}

/// Per-unit frontier result (status string drawn from configured bands).
#[derive(Clone, Debug, Default)]
pub struct FrontierUnit {
    pub status: String,
    pub flags: FrontierFlags,
}

/// Frontier summary for quick reporting/labelling hooks.
#[derive(Clone, Debug, Default)]
pub struct FrontierSummary {
    pub band_counts: BTreeMap<String, u64>,
    pub mediation_units: u64,
    pub enclave_units: u64,
    pub any_protected_override: bool,
}

/// Full frontier mapping output.
#[derive(Clone, Debug, Default)]
pub struct FrontierOut {
    /// Stable map keyed by UnitId (ordered).
    pub units: BTreeMap<UnitId, FrontierUnit>,
    pub summary: FrontierSummary,
}

/// Inputs for the frontier mapping step (execute only if gates.pass == true).
#[derive(Clone, Debug, Default)]
pub struct FrontierInputs {
    /// Observed per-unit support ratios: (numerator, denominator).
    pub unit_support_for_change: BTreeMap<UnitId, (u64, u64)>,
    /// Universe of units considered (post-scope).
    pub units_all: BTreeSet<UnitId>,
    /// Undirected adjacency edges with typed kind.
    pub adjacency: alloc::vec::Vec<(UnitId, UnitId, FrontierEdge)>,
    /// Units that are protected; if their assigned status would imply change, apply override.
    pub protected_units: BTreeSet<UnitId>,
}
// ---------------- Frontier mapping (040–042, 047–049) ------------------------------------------

/// Map per-unit support to band statuses, then flag contiguity/mediation/protection/enclaves.
/// Call this only if the pipeline decided to run frontier after gates.
pub fn map_frontier(inp: &FrontierInputs, p: &impl GatesFrontierParamView) -> FrontierOut {
    let mut out = FrontierOut::default();

    // Fast escape: mode = none ⇒ everyone “none”, no flags.
    if p.frontier_mode_is_none_040() {
        for u in &inp.units_all {
            out.units.insert(
                u.clone(),
                FrontierUnit {
                    status: alloc::string::String::from("none"),
                    flags: FrontierFlags::default(),
                },
            );
        }
        return summarize_frontier(out);
    }

    // Bands: ordered, non-overlapping; compare using integer tenths (floor; no floats).
    let bands = p.frontier_bands_042(); // Vec<(min_pct: u8, max_pct: u8, status: String)>
    let bands_tenths: alloc::vec::Vec<(u16, u16, alloc::string::String)> = bands
        .into_iter()
        .map(|(lo, hi, s)| ((lo as u16) * 10, (hi as u16) * 10, s))
        .collect();

    // Allowed edge kinds for contiguity.
    let mut allowed: BTreeSet<FrontierEdge> = BTreeSet::new();
    if p.frontier_allow_land_047() {
        allowed.insert(FrontierEdge::Land);
    }
    if p.frontier_allow_bridge_047() {
        allowed.insert(FrontierEdge::Bridge);
    }
    if p.frontier_allow_water_047() {
        allowed.insert(FrontierEdge::Water);
    }
    // Island/ferry rule (048): when enabled, ensure Bridge/Water are admissible.
    if p.frontier_island_rule_ferry_allowed_048() {
        allowed.insert(FrontierEdge::Bridge);
        allowed.insert(FrontierEdge::Water);
    }

    // Assign statuses from observed support.
    for u in &inp.units_all {
        let (num, den) = inp.unit_support_for_change.get(u).copied().unwrap_or((0, 0));
        let pct_tenths: u16 = if den == 0 {
            0
        } else {
            // floor((num * 1000) / den) — integer tenths; saturating to u16 range.
            ((num.saturating_mul(1000)) / den).min(u16::MAX as u64) as u16
        };
        let status = assign_band_status(pct_tenths, &bands_tenths);
        out.units.insert(
            u.clone(),
            FrontierUnit {
                status,
                flags: FrontierFlags::default(),
            },
        );
    }

    // Apply protected overrides BEFORE contiguity so components/flags reflect final statuses.
    for u in &inp.protected_units {
        if let Some(unit) = out.units.get_mut(u) {
            if unit.status != "none" {
                unit.flags.protected_override_used = true;
                unit.status = alloc::string::String::from("none");
            }
        }
    }

    // Group units by status for component analysis.
    let by_status: BTreeMap<_, BTreeSet<_>> = {
        let mut map = BTreeMap::<alloc::string::String, BTreeSet<UnitId>>::new();
        for (u, fu) in &out.units {
            map.entry(fu.status.clone()).or_default().insert(u.clone());
        }
        map
    };

    // For each non-"none" status, compute connected components and flag mediation if fragmented.
    let adjacency = &inp.adjacency;
    for (status, members) in &by_status {
        if status == "none" {
            continue;
        }
        let comps = contiguous_components(&allowed, adjacency, members);
        let fragmented = comps.len() > 1;
        if fragmented {
            for comp in &comps {
                for u in comp {
                    if let Some(unit) = out.units.get_mut(u) {
                        unit.flags.mediation_flagged = true;
                    }
                }
            }
        }
    }

    // Set contiguity_ok and enclave flags:
    // - contiguity_ok: unit has ≥1 admissible neighbor with the same status.
    // - enclave: unit has ≥2 admissible neighbors and none share its status.
    for (u, fu) in out.units.iter_mut() {
        if fu.status == "none" {
            continue;
        }
        let mut neighbors_total = 0usize;
        let mut neighbors_same = 0usize;

        for (a, b, kind) in adjacency {
            if !allowed.contains(kind) {
                continue;
            }
            // Undirected: consider both orientations.
            if a == u || b == u {
                let v = if a == u { b } else { a };
                if let Some(g) = out.units.get(v) {
                    neighbors_total += 1;
                    if g.status == fu.status {
                        neighbors_same += 1;
                    }
                }
            }
        }

        fu.flags.contiguity_ok = neighbors_same >= 1;
        fu.flags.enclave = neighbors_total >= 2 && neighbors_same == 0;
    }

    summarize_frontier(out)
}

// ---------------- Internals (helpers) -----------------------------------------------------------

fn assign_band_status(
    pct_tenths: u16,
    bands: &[(u16, u16, alloc::string::String)],
) -> alloc::string::String {
    for (lo, hi, s) in bands {
        if *lo <= pct_tenths && pct_tenths <= *hi {
            return s.clone();
        }
    }
    // If no band matches, return the lowest-priority "none".
    alloc::string::String::from("none")
}

use alloc::collections::VecDeque;

/// Return connected components among `members`, using only `allowed` edge kinds.
fn contiguous_components(
    allowed: &BTreeSet<FrontierEdge>,
    adjacency: &[(UnitId, UnitId, FrontierEdge)],
    members: &BTreeSet<UnitId>,
) -> alloc::vec::Vec<BTreeSet<UnitId>> {
    // Build adjacency list restricted to members + allowed edges
    let mut graph: BTreeMap<UnitId, alloc::vec::Vec<UnitId>> = BTreeMap::new();
    for u in members {
        graph.entry(u.clone()).or_default();
    }
    for (a, b, k) in adjacency {
        if !allowed.contains(k) {
            continue;
        }
        if members.contains(a) && members.contains(b) {
            graph.entry(a.clone()).or_default().push(b.clone());
            graph.entry(b.clone()).or_default().push(a.clone());
        }
    }

    // BFS over graph
    let mut seen: BTreeSet<UnitId> = BTreeSet::new();
    let mut comps: alloc::vec::Vec<BTreeSet<UnitId>> = alloc::vec::Vec::new();
    for u in members {
        if seen.contains(u) {
            continue;
        }
        let mut comp: BTreeSet<UnitId> = BTreeSet::new();
        let mut q = VecDeque::new();
        q.push_back(u.clone());
        seen.insert(u.clone());
        comp.insert(u.clone());

        while let Some(x) = q.pop_front() {
            if let Some(nei) = graph.get(&x) {
                for v in nei {
                    if seen.insert(v.clone()) {
                        q.push_back(v.clone());
                        comp.insert(v.clone());
                    }
                }
            }
        }
        comps.push(comp);
    }
    comps
}

fn summarize_frontier(mut out: FrontierOut) -> FrontierOut {
    // Fill summary counts and booleans.
    for (_, u) in &out.units {
        *out.summary.band_counts.entry(u.status.clone()).or_insert(0) += 1;
        if u.flags.mediation_flagged {
            out.summary.mediation_units += 1;
        }
        if u.flags.enclave {
            out.summary.enclave_units += 1;
        }
        if u.flags.protected_override_used {
            out.summary.any_protected_override = true;
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    // --- Helpers ------------------------------------------------------------------------------

    fn uid(s: &str) -> UnitId {
        s.parse().expect("unit id")
    }

    // Baseline policy: DM on; frontier bands [0..49]=hold, [50..100]=change; only Land allowed.
    struct PBase;
    impl GatesFrontierParamView for PBase {
        fn quorum_global_pct_020(&self) -> u8 { 50 }
        fn quorum_per_unit_pct_021(&self) -> u8 { 40 }
        fn national_majority_pct_022(&self) -> u8 { 55 }
        fn regional_majority_pct_023(&self) -> u8 { 55 }
        fn double_majority_enabled_024(&self) -> bool { true }
        fn symmetry_enabled_025(&self) -> bool { false }
        fn symmetry_breaks_due_to_exceptions_029(&self) -> bool { false }

        fn frontier_mode_is_none_040(&self) -> bool { false }
        fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, alloc::string::String)> {
            vec![(0, 49, "hold".into()), (50, 100, "change".into())]
        }
        fn frontier_allow_land_047(&self) -> bool { true }
        fn frontier_allow_bridge_047(&self) -> bool { false }
        fn frontier_allow_water_047(&self) -> bool { false }
        fn frontier_island_rule_ferry_allowed_048(&self) -> bool { false }
    }

    // Frontier mode = none
    struct PNone;
    impl GatesFrontierParamView for PNone {
        fn quorum_global_pct_020(&self) -> u8 { 50 }
        fn quorum_per_unit_pct_021(&self) -> u8 { 40 }
        fn national_majority_pct_022(&self) -> u8 { 55 }
        fn regional_majority_pct_023(&self) -> u8 { 55 }
        fn double_majority_enabled_024(&self) -> bool { true }
        fn symmetry_enabled_025(&self) -> bool { false }
        fn symmetry_breaks_due_to_exceptions_029(&self) -> bool { false }

        fn frontier_mode_is_none_040(&self) -> bool { true }
        fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, alloc::string::String)> {
            vec![(0, 49, "hold".into()), (50, 100, "change".into())]
        }
        fn frontier_allow_land_047(&self) -> bool { true }
        fn frontier_allow_bridge_047(&self) -> bool { false }
        fn frontier_allow_water_047(&self) -> bool { false }
        fn frontier_island_rule_ferry_allowed_048(&self) -> bool { false }
    }

    // Ferry allowed (adds Bridge/Water even if 047 denied them)
    struct PWaterFerry;
    impl GatesFrontierParamView for PWaterFerry {
        fn quorum_global_pct_020(&self) -> u8 { 50 }
        fn quorum_per_unit_pct_021(&self) -> u8 { 40 }
        fn national_majority_pct_022(&self) -> u8 { 55 }
        fn regional_majority_pct_023(&self) -> u8 { 55 }
        fn double_majority_enabled_024(&self) -> bool { true }
        fn symmetry_enabled_025(&self) -> bool { false }
        fn symmetry_breaks_due_to_exceptions_029(&self) -> bool { false }

        fn frontier_mode_is_none_040(&self) -> bool { false }
        fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, alloc::string::String)> {
            vec![(0, 49, "hold".into()), (50, 100, "change".into())]
        }
        fn frontier_allow_land_047(&self) -> bool { true }
        fn frontier_allow_bridge_047(&self) -> bool { false }
        fn frontier_allow_water_047(&self) -> bool { false }
        fn frontier_island_rule_ferry_allowed_048(&self) -> bool { true }
    }

    // --- Gates ---------------------------------------------------------------------------------

    #[test]
    fn quorum_and_majority_and_double_majority() {
        let mut gi = GateInputs::default();
        gi.nat_ballots_cast = 600;
        gi.nat_eligible_roll = 1000; // 60% >= 50% → quorum_national true
        gi.nat_valid_ballots = 500;

        // per-unit quorum: U1 passes (40%), U2 fails (0%)
        gi.unit_eligible_roll.insert(uid("U1"), 100);
        gi.unit_eligible_roll.insert(uid("U2"), 100);
        gi.unit_ballots_cast.insert(uid("U1"), 40);

        // national majority: 300/500 = 60% >= 55%
        gi.unit_support_for_change.insert(uid("U1"), 200);
        gi.unit_support_for_change.insert(uid("U2"), 100);

        // regions: R1 passes (60%), R2 fails (40%); 1/2 regions → passes with our ceil(R/2) rule
        gi.region_valid_ballots.insert("R1".into(), 100);
        gi.region_support_for_change.insert("R1".into(), 60);
        gi.region_valid_ballots.insert("R2".into(), 100);
        gi.region_support_for_change.insert("R2".into(), 40);

        let p = PBase;
        let gr = apply_decision_gates(&gi, &p);

        assert!(gr.quorum_national);
        assert!(gr.majority_national);
        assert!(gr.majority_regional);
        assert!(gr.double_majority);
        assert!(gr.pass);

        // per-unit quorum set contains only U1
        assert!(gr.quorum_per_unit_passset.contains(&uid("U1")));
        assert!(!gr.quorum_per_unit_passset.contains(&uid("U2")));
    }

    #[test]
    fn double_majority_no_regions_treated_as_pass() {
        let mut gi = GateInputs::default();
        gi.nat_ballots_cast = 500;
        gi.nat_eligible_roll = 800;
        gi.nat_valid_ballots = 400;
        gi.unit_support_for_change.insert(uid("U1"), 240); // 60%

        let p = PBase;
        let gr = apply_decision_gates(&gi, &p);
        assert!(gr.majority_national);
        assert!(gr.majority_regional); // N/A ⇒ pass
        assert!(gr.double_majority);
        assert!(gr.pass);
    }

    // --- Frontier ------------------------------------------------------------------------------

    #[test]
    fn frontier_mode_none_assigns_none_everywhere() {
        let mut fi = FrontierInputs::default();
        fi.units_all.extend([uid("A"), uid("B"), uid("C")]);

        let out = map_frontier(&fi, &PNone);
        for u in [uid("A"), uid("B"), uid("C")] {
            assert_eq!(out.units.get(&u).unwrap().status, "none".to_string());
        }
        assert_eq!(*out.summary.band_counts.get("none").unwrap_or(&0), 3);
        assert!(!out.summary.any_protected_override);
    }

    #[test]
    fn frontier_mediation_and_contiguity_ok_semantics() {
        // A,B,C are "change" (60%); B—C connected; A isolated → two components → mediation=true.
        let (a, b, c) = (uid("A"), uid("B"), uid("C"));

        let mut fi = FrontierInputs::default();
        fi.units_all.extend([a.clone(), b.clone(), c.clone()]);
        fi.unit_support_for_change.insert(a.clone(), (60, 100));
        fi.unit_support_for_change.insert(b.clone(), (60, 100));
        fi.unit_support_for_change.insert(c.clone(), (60, 100));
        fi.adjacency.push((b.clone(), c.clone(), FrontierEdge::Land));

        let out = map_frontier(&fi, &PBase);

        // A is singleton "change": contiguity_ok=false
        assert_eq!(out.units.get(&a).unwrap().status, "change");
        assert!(!out.units.get(&a).unwrap().flags.contiguity_ok);

        // B and C have same-status neighbor: contiguity_ok=true
        assert!(out.units.get(&b).unwrap().flags.contiguity_ok);
        assert!(out.units.get(&c).unwrap().flags.contiguity_ok);

        // Mediation flagged for all members of fragmented status "change"
        assert!(out.units.get(&a).unwrap().flags.mediation_flagged);
        assert!(out.units.get(&b).unwrap().flags.mediation_flagged);
        assert!(out.units.get(&c).unwrap().flags.mediation_flagged);
        assert!(out.summary.mediation_units >= 3);
    }

    #[test]
    fn frontier_enclave_rule_requires_two_neighbors_and_none_same() {
        // D has two neighbors E,F of different status → enclave=true for D.
        let (d, e, f) = (uid("D"), uid("E"), uid("F"));

        let mut fi = FrontierInputs::default();
        fi.units_all.extend([d.clone(), e.clone(), f.clone()]);
        fi.unit_support_for_change.insert(d.clone(), (40, 100)); // hold
        fi.unit_support_for_change.insert(e.clone(), (60, 100)); // change
        fi.unit_support_for_change.insert(f.clone(), (60, 100)); // change
        fi.adjacency.push((d.clone(), e.clone(), FrontierEdge::Land));
        fi.adjacency.push((d.clone(), f.clone(), FrontierEdge::Land));

        let out = map_frontier(&fi, &PBase);

        let ud = out.units.get(&d).unwrap();
        assert_eq!(ud.status, "hold");
        assert!(ud.flags.enclave);
        assert!(!ud.flags.contiguity_ok); // no same-status neighbor
    }

    #[test]
    fn frontier_ferry_rule_allows_water_edges() {
        // G and H are change, connected only by water. With ferry rule, contiguity_ok=true.
        let (g, h) = (uid("G"), uid("H"));

        let mut fi = FrontierInputs::default();
        fi.units_all.extend([g.clone(), h.clone()]);
        fi.unit_support_for_change.insert(g.clone(), (60, 100));
        fi.unit_support_for_change.insert(h.clone(), (60, 100));
        fi.adjacency.push((g.clone(), h.clone(), FrontierEdge::Water));

        let out = map_frontier(&fi, &PWaterFerry);

        assert_eq!(out.units.get(&g).unwrap().status, "change");
        assert_eq!(out.units.get(&h).unwrap().status, "change");
        assert!(out.units.get(&g).unwrap().flags.contiguity_ok);
        assert!(out.units.get(&h).unwrap().flags.contiguity_ok);
    }

    #[test]
    fn protected_overrides_apply_before_contiguity() {
        // J—K connected change; protect K → K becomes none; J becomes singleton change.
        let (j, k) = (uid("J"), uid("K"));

        let mut fi = FrontierInputs::default();
        fi.units_all.extend([j.clone(), k.clone()]);
        fi.unit_support_for_change.insert(j.clone(), (60, 100));
        fi.unit_support_for_change.insert(k.clone(), (60, 100));
        fi.adjacency.push((j.clone(), k.clone(), FrontierEdge::Land));
        fi.protected_units.insert(k.clone());

        let out = map_frontier(&fi, &PBase);

        assert_eq!(out.units.get(&k).unwrap().status, "none");
        assert!(out.units.get(&k).unwrap().flags.protected_override_used);

        let uj = out.units.get(&j).unwrap();
        assert_eq!(uj.status, "change");
        assert!(!uj.flags.contiguity_ok); // now singleton after protection
        assert!(out.summary.any_protected_override);
    }
}
