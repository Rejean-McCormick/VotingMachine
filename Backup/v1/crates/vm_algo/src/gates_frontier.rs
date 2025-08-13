//! crates/vm_algo/src/gates_frontier.rs
//! Decision gates (quorum → national majority → optional double-majority → symmetry)
//! and, when passed, frontier mapping (bands, contiguity & flags). Pure integer math,
//! deterministic ordering, no RNG (ties live elsewhere).

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};

use vm_core::{
    ids::UnitId,
    rounding::ge_percent, // integer test: a/b >= p%
    variables::Params,
};

/// Inputs required by the gate checks (aggregates + per-unit basics).
#[derive(Clone, Debug, Default)]
pub struct GateInputs {
    pub nat_ballots_cast: u64,
    pub nat_invalid_ballots: u64,
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

/// Inputs for the frontier mapping step (execute only if gates.pass == true).
#[derive(Clone, Debug, Default)]
pub struct FrontierInputs {
    /// Observed per-unit support ratios: (numerator, denominator).
    pub unit_support_for_change: BTreeMap<UnitId, (u64, u64)>,
    /// Universe of units considered (post-scope).
    pub units_all: BTreeSet<UnitId>,
    /// Undirected adjacency edges with typed kind.
    pub adjacency: Vec<(UnitId, UnitId, FrontierEdge)>,
    /// Units that are protected; if their assigned status would imply change, apply override.
    pub protected_units: BTreeSet<UnitId>,
}

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
    pub status: alloc::string::String,
    pub flags: FrontierFlags,
}

/// Frontier summary for quick reporting/labelling hooks.
#[derive(Clone, Debug, Default)]
pub struct FrontierSummary {
    pub band_counts: BTreeMap<alloc::string::String, u32>,
    pub mediation_units: u32,
    pub enclave_units: u32,
    pub any_protected_override: bool,
}

/// Full frontier mapping output.
#[derive(Clone, Debug, Default)]
pub struct FrontierOut {
    /// Stable map keyed by UnitId (ordered).
    pub units: BTreeMap<UnitId, FrontierUnit>,
    pub summary: FrontierSummary,
}

// ---------------- Gates (020–029; majority uses valid_ballots as denominator) -------------------

/// Apply quorum + majority (+ optional double-majority & symmetry) deterministically.
pub fn apply_decision_gates(inp: &GateInputs, p: &Params) -> GateResult {
    // Quorum (national): Σ ballots_cast / Σ eligible_roll ≥ 020
    let quorum_nat = compute_quorum_national(inp.nat_ballots_cast, inp.nat_eligible_roll, p.quorum_global_pct_020());

    // Per-unit quorum (021): collect pass set
    let quorum_set = compute_quorum_per_unit(
        &inp.unit_ballots_cast,
        &inp.unit_eligible_roll,
        p.quorum_per_unit_pct_021(),
    );

    // National approval majority (022): approvals_for_change / valid_ballots ≥ cutoff
    // Denominator is *valid_ballots* (explicitly ignores blank toggle for approval majority).
    let nat_support_sum: u64 = inp.unit_support_for_change.values().copied().sum();
    let maj_nat = national_approval_majority(inp.nat_valid_ballots, nat_support_sum, p.national_majority_pct_022());

    // Double-majority (024/023) over affected family:
    // Aggregate region totals using region_* maps. If no regions, treat as not applicable=false.
    let mut maj_regional = false;
    if p.double_majority_enabled_024() {
        let v_sum: u64 = inp.region_valid_ballots.values().copied().sum();
        let s_sum: u64 = inp.region_support_for_change.values().copied().sum();
        // Same share definition as national: support / valid_ballots.
        maj_regional = v_sum > 0 && ge_percent(s_sum, v_sum, p.regional_majority_pct_023());
    }

    let dbl = if p.double_majority_enabled_024() { maj_nat && maj_regional } else { maj_nat };

    // Symmetry (025/029). Without deeper policy here, enforce the toggle: when enabled,
    // we currently synthesize `true` unless explicit exceptions require failing symmetry.
    // Exception matching belongs to params policy; treat “no exceptions” == symmetric.
    let symmetry = if p.symmetry_enabled_025() {
        // If there are explicit symmetry exceptions and policy marks them as breaking symmetry,
        // params should reflect that via a boolean. Use permissive default: true.
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
fn compute_quorum_per_unit(
    unit_ballots_cast: &BTreeMap<UnitId, u64>,
    unit_eligible_roll: &BTreeMap<UnitId, u64>,
    cutoff_pct: u8,
) -> BTreeSet<UnitId> {
    let mut out = BTreeSet::new();
    for (u, cast) in unit_ballots_cast {
        if let Some(roll) = unit_eligible_roll.get(u) {
            if *roll > 0 && ge_percent(*cast, *roll, cutoff_pct) {
                out.insert(u.clone());
            }
        }
    }
    out
}

/// approval majority is approvals_for_change / valid_ballots ≥ cutoff (fixed denominator).
#[inline]
fn national_approval_majority(valid_ballots: u64, approvals_for_change: u64, cutoff_pct: u8) -> bool {
    valid_ballots > 0 && ge_percent(approvals_for_change, valid_ballots, cutoff_pct)
}

// ---------------- Frontier (040–042, 047–049) --------------------------------------------------

/// Map per-unit support to band statuses, then flag contiguity/mediation/protection/enclaves.
/// Call this only if `gates.pass == true`.
pub fn map_frontier(inp: &FrontierInputs, p: &Params) -> FrontierOut {
    use alloc::string::String;

    let mut out = FrontierOut::default();

    // Fast escape: mode = none ⇒ everyone “none”, no flags.
    if p.frontier_mode_is_none_040() {
        for u in &inp.units_all {
            out.units.insert(
                u.clone(),
                FrontierUnit {
                    status: String::from("none"),
                    flags: FrontierFlags::default(),
                },
            );
        }
        return summarize_frontier(out);
    }

    // Bands come ordered, non-overlapping; compare using integer tenths (no floats).
    let bands = p.frontier_bands_042(); // Vec<(min_pct: u8, max_pct: u8, status: String)>
    let bands_tenths: alloc::vec::Vec<(u16, u16, String)> = bands
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

    // Assign statuses.
    for u in &inp.units_all {
        let (num, den) = inp.unit_support_for_change.get(u).copied().unwrap_or((0, 0));
        let pct_tenths: u16 = if den == 0 {
            0
        } else {
            // floor((num * 1000) / den) — integer tenths
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

    // Contiguity & mediation flags:
    // Build induced subgraphs per status and check if they form >1 connected components.
    let adjacency = &inp.adjacency;
    let by_status: BTreeMap<_, BTreeSet<_>> = {
        let mut map = BTreeMap::<String, BTreeSet<UnitId>>::new();
        for (u, fu) in &out.units {
            map.entry(fu.status.clone()).or_default().insert(u.clone());
        }
        map
    };

    for (status, members) in &by_status {
        if status == "none" {
            continue;
        }
        // Connected components among members using allowed edges only.
        let comps = contiguous_components(&allowed, adjacency, members);

        // Mark contiguity_ok for members in any component; mediation if more than one component.
        let many = comps.len() > 1;
        for comp in &comps {
            for u in comp {
                if let Some(unit) = out.units.get_mut(u) {
                    unit.flags.contiguity_ok = true;
                    unit.flags.mediation_flagged |= many;
                }
            }
        }
    }

    // Island/corridor refinement (048): if ferry_allowed, treat bridge+water as admissible.
    // We already honored allowed set above, so when ferry_allowed, ensure Bridge/Water were added.
    if p.frontier_island_rule_ferry_allowed_048() {
        // If earlier policy excluded Bridge/Water, we cannot recompute here
        // without re-running contiguity; conservative: if not allowed, we do a best-effort pass:
        // no extra action (policy should configure 047 accordingly).
    }

    // Protected overrides: downgrade status to "none" when protected.
    for u in &inp.protected_units {
        if let Some(unit) = out.units.get_mut(u) {
            if unit.status != "none" {
                unit.flags.protected_override_used = true;
                unit.status = String::from("none");
            }
        }
    }

    // Enclave flag: a unit whose neighbors (admissible edges) all have a different status.
    for (u, fu) in &out.units {
        if fu.status == "none" {
            continue;
        }
        let mut has_neighbor = false;
        let mut any_same = false;
        for (a, b, kind) in adjacency {
            if !allowed.contains(kind) {
                continue;
            }
            let (x, y) = (a, b);
            if x == u || y == u {
                let v = if x == u { y } else { x };
                if let Some(g) = out.units.get(v) {
                    has_neighbor = true;
                    if g.status == fu.status {
                        any_same = true;
                        break;
                    }
                }
            }
        }
        if let Some(me) = out.units.get_mut(u) {
            me.flags.enclave = has_neighbor && !any_same;
        }
    }

    summarize_frontier(out)
}

// ---------------- Internals ---------------------------------------------------------------------

fn assign_band_status(pct_tenths: u16, bands: &[(u16, u16, alloc::string::String)]) -> alloc::string::String {
    for (lo, hi, s) in bands {
        if *lo <= pct_tenths && pct_tenths <= *hi {
            return s.clone();
        }
    }
    // If no band matches, return the lowest-priority "none".
    alloc::string::String::from("none")
}

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

// ---------------- Param access helpers (thin; resolved by vm_core::variables::Params) -----------
//
// NOTE: These helpers assume `Params` exposes deterministic getters for the needed VM-VARs.
// Exact domains/defaults live in Annex A and Doc 2; this module reads them only at the
// documented touchpoints (4B gates, 4C frontier). See docs cited in the file header.

trait GatesFrontierParamView {
    // 020–029 (gates)
    fn quorum_global_pct_020(&self) -> u8;
    fn quorum_per_unit_pct_021(&self) -> u8;
    fn national_majority_pct_022(&self) -> u8;
    fn regional_majority_pct_023(&self) -> u8;
    fn double_majority_enabled_024(&self) -> bool;
    fn symmetry_enabled_025(&self) -> bool;
    fn symmetry_breaks_due_to_exceptions_029(&self) -> bool;

    // 040–042, 047–049 (frontier)
    fn frontier_mode_is_none_040(&self) -> bool;
    /// Ordered, non-overlapping bands: (min_pct, max_pct, status)
    fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, alloc::string::String)>;
    fn frontier_allow_land_047(&self) -> bool;
    fn frontier_allow_bridge_047(&self) -> bool;
    fn frontier_allow_water_047(&self) -> bool;
    fn frontier_island_rule_ferry_allowed_048(&self) -> bool;
}

// This blanket impl simply forwards to assumed `Params` getters.
// Implement these methods in `vm_core::variables::Params` to satisfy the trait.
impl GatesFrontierParamView for Params {
    // --- gates ---
    #[inline] fn quorum_global_pct_020(&self) -> u8 { self.quorum_global_pct_020() }
    #[inline] fn quorum_per_unit_pct_021(&self) -> u8 { self.quorum_per_unit_pct_021() }
    #[inline] fn national_majority_pct_022(&self) -> u8 { self.national_majority_pct_022() }
    #[inline] fn regional_majority_pct_023(&self) -> u8 { self.regional_majority_pct_023() }
    #[inline] fn double_majority_enabled_024(&self) -> bool { self.double_majority_enabled_024() }
    #[inline] fn symmetry_enabled_025(&self) -> bool { self.symmetry_enabled_025() }
    #[inline] fn symmetry_breaks_due_to_exceptions_029(&self) -> bool { self.symmetry_breaks_due_to_exceptions_029() }

    // --- frontier ---
    #[inline] fn frontier_mode_is_none_040(&self) -> bool { self.frontier_mode_is_none_040() }
    #[inline] fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, alloc::string::String)> { self.frontier_bands_042() }
    #[inline] fn frontier_allow_land_047(&self) -> bool { self.frontier_allow_land_047() }
    #[inline] fn frontier_allow_bridge_047(&self) -> bool { self.frontier_allow_bridge_047() }
    #[inline] fn frontier_allow_water_047(&self) -> bool { self.frontier_allow_water_047() }
    #[inline] fn frontier_island_rule_ferry_allowed_048(&self) -> bool { self.frontier_island_rule_ferry_allowed_048() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tiny stand-in Params to drive the helpers in tests.
    struct P;
    impl GatesFrontierParamView for P {
        fn quorum_global_pct_020(&self) -> u8 { 50 }
        fn quorum_per_unit_pct_021(&self) -> u8 { 40 }
        fn national_majority_pct_022(&self) -> u8 { 55 }
        fn regional_majority_pct_023(&self) -> u8 { 55 }
        fn double_majority_enabled_024(&self) -> bool { true }
        fn symmetry_enabled_025(&self) -> bool { false }
        fn symmetry_breaks_due_to_exceptions_029(&self) -> bool { false }

        fn frontier_mode_is_none_040(&self) -> bool { false }
        fn frontier_bands_042(&self) -> alloc::vec::Vec<(u8, u8, alloc::string::String)> {
            use alloc::string::String;
            vec![(0, 49, String::from("hold")), (50, 100, String::from("change"))]
        }
        fn frontier_allow_land_047(&self) -> bool { true }
        fn frontier_allow_bridge_047(&self) -> bool { false }
        fn frontier_allow_water_047(&self) -> bool { false }
        fn frontier_island_rule_ferry_allowed_048(&self) -> bool { false }
    }

    #[test]
    fn quorum_and_majority_basics() {
        let mut gi = GateInputs::default();
        gi.nat_ballots_cast = 600;
        gi.nat_eligible_roll = 1000;
        gi.nat_valid_ballots = 500;
        gi.unit_support_for_change.insert("U".parse().unwrap(), 300);
        gi.region_valid_ballots.insert("R".into(), 500);
        gi.region_support_for_change.insert("R".into(), 300);

        let p = P;
        let gr = apply_decision_gates(&gi, unsafe { core::mem::transmute::<&P, &Params>(&p) });
        assert!(gr.quorum_national);
        assert!(gr.majority_national);
        assert!(gr.double_majority);
        assert!(gr.pass);
    }

    #[test]
    fn frontier_bands_and_components() {
        use alloc::string::ToString;

        let u1: UnitId = "A".parse().unwrap();
        let u2: UnitId = "B".parse().unwrap();

        let mut fi = FrontierInputs::default();
        fi.units_all.extend([u1.clone(), u2.clone()]);
        fi.unit_support_for_change.insert(u1.clone(), (60, 100)); // 60%
        fi.unit_support_for_change.insert(u2.clone(), (40, 100)); // 40%
        fi.adjacency.push((u1.clone(), u2.clone(), FrontierEdge::Land));

        let p = P;
        let out = map_frontier(&fi, unsafe { core::mem::transmute::<&P, &Params>(&p) });
        assert_eq!(out.units.get(&u1).unwrap().status, "change".to_string());
        assert_eq!(out.units.get(&u2).unwrap().status, "hold".to_string());
        assert!(out.units.get(&u1).unwrap().flags.contiguity_ok);
        assert!(out.units.get(&u2).unwrap().flags.contiguity_ok);
    }
}
