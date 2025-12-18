//! MAP_FRONTIER stage (runs only if gates PASSED).
//! - Assign one status per Unit via configured bands
//! - Build components using allowed edge types
//! - Apply corridor/island policy (modeled via allowed-edge mask here)
//! - Enforce protected/quorum blocks
//! - Flag mediation (isolated pre-block change) and enclaves (post-block surrounded)
//!
//! Pure integer/rational math; no RNG.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use vm_core::{
    ids::UnitId,
    entities::EdgeType,      // expected: Land | Bridge | Water
    rounding::{ge_percent, Ratio},
    variables::Params,
};

// ---------- Public view inputs ----------

#[derive(Clone, Debug, Default)]
pub struct UnitsView {
    /// All known units (stable set; determines iteration order)
    pub all: BTreeSet<UnitId>,
    /// Units that must not change status; frontier assignment is forced to "none"
    pub protected: BTreeSet<UnitId>,
}

#[derive(Clone, Debug, Default)]
pub struct AdjacencyView {
    /// Undirected edges (a,b,kind). Each pair may appear once.
    pub edges: Vec<(UnitId, UnitId, EdgeType)>,
}

// ---------- Local types ----------

pub type AllowedEdges = BTreeSet<EdgeType>;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum FrontierMode {
    None,
    SlidingScale,
    AutonomyLadder,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum IslandCorridorRule {
    None,
    FerryAllowed,
    CorridorRequired,
}

#[derive(Clone, Debug)]
pub struct Band {
    pub min_pct: u8,        // inclusive
    pub max_pct: u8,        // inclusive
    pub status: String,     // machine-readable label
    pub ap_id: Option<String>,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ComponentId(pub u32);

#[derive(Default, Clone, Debug)]
pub struct UnitFlags {
    pub mediation: bool,
    pub enclave: bool,
    pub protected_blocked: bool,
    pub quorum_blocked: bool,
}

#[derive(Clone, Debug)]
pub struct UnitFrontier {
    pub status: String,                // one of bands[].status or "none" when blocked/mode=None
    pub band_index: Option<usize>,     // index in configured bands (None for "none")
    pub component: ComponentId,
    pub flags: UnitFlags,
}

#[derive(Default, Clone, Debug)]
pub struct FrontierMap {
    pub units: BTreeMap<UnitId, UnitFrontier>,
    pub summary_by_status: BTreeMap<String, u32>,
    /// "mediation","enclave","protected_blocked","quorum_blocked"
    pub summary_flags: BTreeMap<&'static str, u32>,
}

// ---------- Public API ----------

pub fn map_frontier(
    units: &UnitsView,
    unit_support_pct: &BTreeMap<UnitId, Ratio>,
    adjacency: &AdjacencyView,
    p: &Params,
    per_unit_quorum: Option<&BTreeMap<UnitId, bool>>,
) -> FrontierMap {
    // Resolve configured mode & bands
    let (mode, bands) = resolve_mode_and_bands(p);

    // Allowed edges + corridor rule
    let allowed = allowed_edges_from_params(p);
    let corridor = corridor_rule_from_params(p);

    // Components first (deterministic numbering)
    let (comp_of, components) = build_components(adjacency, &allowed, corridor);

    // Fast exit when mode=None: assign "none" to all units but still keep components.
    if matches!(mode, FrontierMode::None) {
        let mut out = FrontierMap::default();
        for uid in &units.all {
            let comp = comp_of.get(uid).cloned().unwrap_or(ComponentId(0));
            out.units.insert(
                uid.clone(),
                UnitFrontier {
                    status: "none".to_string(),
                    band_index: None,
                    component: comp,
                    flags: UnitFlags::default(),
                },
            );
        }
        update_summaries(&mut out);
        return out;
    }

    // Pass 1: compute intended (pre-block) status from bands
    let mut planned_status: BTreeMap<UnitId, (String, Option<usize>)> = BTreeMap::new();
    for uid in &units.all {
        let support = unit_support_pct
            .get(uid)
            .cloned()
            .unwrap_or(Ratio { num: 0, den: 1 });
        planned_status.insert(uid.clone(), pick_band_status(support, &bands));
    }

    // Pass 2: instantiate UnitFrontier entries with blocks (protected/quorum), but keep
    // a copy of the original planned status for mediation analysis.
    let mut out = FrontierMap::default();
    for uid in &units.all {
        let (mut status, mut band_idx) = planned_status[uid].clone();
        let comp = comp_of.get(uid).cloned().unwrap_or(ComponentId(0));
        let mut flags = UnitFlags::default();

        apply_protection_and_quorum(uid, &mut status, &mut band_idx, &mut flags, units, per_unit_quorum);

        out.units.insert(
            uid.clone(),
            UnitFrontier {
                status,
                band_index: band_idx,
                component: comp,
                flags,
            },
        );
    }

    // Pass 3: mediation — if a unit planned a non-"none" status, but has no neighbor
    // with the same planned status (under allowed edges), mark mediation and force "none".
    // (We look at planned statuses to capture isolation independent of blocks.)
    {
        // Build adjacency map constrained to allowed edges, to check same-status neighbors.
        let adj = adjacency_map(adjacency, &allowed);

        for uid in &units.all {
            let (planned, _) = &planned_status[uid];
            if planned == "none" {
                continue;
            }
            // Does it have any neighbor planning the same status?
            let mut has_same_neighbor = false;
            if let Some(neis) = adj.get(uid) {
                for v in neis {
                    if let Some((nbr_planned, _)) = planned_status.get(v) {
                        if nbr_planned == planned {
                            has_same_neighbor = true;
                            break;
                        }
                    }
                }
            }
            if !has_same_neighbor {
                if let Some(uf) = out.units.get_mut(uid) {
                    uf.flags.mediation = true;
                    uf.status = "none".to_string();
                    uf.band_index = None;
                }
            }
        }
    }

    // Pass 4: enclaves — after final statuses, a unit with status != "none" is an enclave
    // if all its neighbors (within allowed edges) are "none". Require at least one neighbor.
    {
        let adj = adjacency_map(adjacency, &allowed);
        for uid in &units.all {
            let mut has_neighbor = false;
            let mut all_neighbors_none = true;
            if let Some(uf) = out.units.get(uid) {
                if uf.status == "none" {
                    continue;
                }
                if let Some(neis) = adj.get(uid) {
                    for v in neis {
                        has_neighbor = true;
                        let ns = &out.units[v].status;
                        if ns != "none" {
                            all_neighbors_none = false;
                            break;
                        }
                    }
                }
                if has_neighbor && all_neighbors_none {
                    if let Some(ufm) = out.units.get_mut(uid) {
                        ufm.flags.enclave = true;
                    }
                }
            }
        }
    }

    update_summaries(&mut out);
    out
}

// ---------- Internals ----------

fn resolve_mode_and_bands(p: &Params) -> (FrontierMode, Vec<Band>) {
    // These accessors mirror the variables noted in the spec. If Params exposes different
    // method names in your tree, wire them here.
    let mode = match p.frontier_mode() {
        // Assume vm_core::variables::FrontierMode isomorphic to ours:
        vm_core::variables::FrontierMode::None => FrontierMode::None,
        vm_core::variables::FrontierMode::SlidingScale => FrontierMode::SlidingScale,
        vm_core::variables::FrontierMode::AutonomyLadder => FrontierMode::AutonomyLadder,
    };

    let mut bands = Vec::<Band>::new();
    for b in p.frontier_bands().unwrap_or_default() {
        // Assume `b` exposes (min,max,status,ap_id?) getters
        bands.push(Band {
            min_pct: b.min_pct(),
            max_pct: b.max_pct(),
            status: b.status().to_string(),
            ap_id: b.ap_id(),
        });
    }
    (mode, bands)
}

fn allowed_edges_from_params(p: &Params) -> AllowedEdges {
    // Expect Params to expose a set of EdgeType values; default to {Land, Bridge} if absent.
    let mut set: AllowedEdges = p.frontier_allowed_edge_types().unwrap_or_default();
    if set.is_empty() {
        set.insert(EdgeType::Land);
        set.insert(EdgeType::Bridge);
    }
    set
}

fn corridor_rule_from_params(p: &Params) -> IslandCorridorRule {
    match p.island_corridor_rule() {
        vm_core::variables::IslandCorridorRule::None => IslandCorridorRule::None,
        vm_core::variables::IslandCorridorRule::FerryAllowed => IslandCorridorRule::FerryAllowed,
        vm_core::variables::IslandCorridorRule::CorridorRequired => IslandCorridorRule::CorridorRequired,
    }
}

/// Create adjacency lists filtered by `allowed` edge kinds.
fn adjacency_map(
    adjacency: &AdjacencyView,
    allowed: &AllowedEdges,
) -> BTreeMap<UnitId, BTreeSet<UnitId>> {
    let mut map: BTreeMap<UnitId, BTreeSet<UnitId>> = BTreeMap::new();
    for (a, b, kind) in &adjacency.edges {
        if allowed.contains(kind) {
            map.entry(a.clone()).or_default().insert(b.clone());
            map.entry(b.clone()).or_default().insert(a.clone());
        }
    }
    map
}

fn build_components(
    adjacency: &AdjacencyView,
    allowed: &AllowedEdges,
    _corridor: IslandCorridorRule,
) -> (BTreeMap<UnitId, ComponentId>, Vec<BTreeSet<UnitId>>) {
    // NOTE: Corridor policy is represented by the allowed-edge mask here. If your engine
    // adds corridor metadata, incorporate it into `allowed` before reaching this point.

    let adj = adjacency_map(adjacency, allowed);

    // Collect all vertices mentioned by edges; isolated vertices can be injected by callers via UnitsView.
    let mut all: BTreeSet<UnitId> = BTreeSet::new();
    for (u, neis) in &adj {
        all.insert(u.clone());
        for v in neis {
            all.insert(v.clone());
        }
    }

    // Build connected components via BFS in UnitId order for determinism.
    let mut seen: BTreeSet<UnitId> = BTreeSet::new();
    let mut comps: Vec<BTreeSet<UnitId>> = Vec::new();

    for start in &all {
        if seen.contains(start) {
            continue;
        }
        let mut comp = BTreeSet::<UnitId>::new();
        let mut q = VecDeque::<UnitId>::new();
        q.push_back(start.clone());
        seen.insert(start.clone());
        comp.insert(start.clone());

        while let Some(u) = q.pop_front() {
            if let Some(neis) = adj.get(&u) {
                for v in neis {
                    if !seen.contains(v) {
                        seen.insert(v.clone());
                        comp.insert(v.clone());
                        q.push_back(v.clone());
                    }
                }
            }
        }
        comps.push(comp);
    }

    // Sort components by their smallest UnitId to fix numbering, then assign IDs 0..N-1.
    comps.sort_by(|a, b| a.iter().next().cmp(&b.iter().next()));

    let mut comp_of: BTreeMap<UnitId, ComponentId> = BTreeMap::new();
    for (idx, comp) in comps.iter().enumerate() {
        let cid = ComponentId(idx as u32);
        for u in comp {
            comp_of.insert(u.clone(), cid);
        }
    }

    (comp_of, comps)
}

/// Inclusive band check: pick the first band b where min ≤ pct(support) ≤ max.
fn pick_band_status(support: Ratio, bands: &[Band]) -> (String, Option<usize>) {
    for (i, b) in bands.iter().enumerate() {
        let ge_min = ge_percent(support.num, support.den, b.min_pct).unwrap_or(false);
        // For the inclusive upper bound, check NOT (support ≥ max+1)
        let lt_next = if b.max_pct < 100 {
            !ge_percent(support.num, support.den, b.max_pct.saturating_add(1)).unwrap_or(false)
        } else {
            true // max=100 means anything ≥100% still qualifies
        };
        if ge_min && lt_next {
            return (b.status.clone(), Some(i));
        }
    }
    ("none".to_string(), None)
}

fn apply_protection_and_quorum(
    unit_id: &UnitId,
    intended_status: &mut String,
    band_idx: &mut Option<usize>,
    flags: &mut UnitFlags,
    units: &UnitsView,
    per_unit_quorum: Option<&BTreeMap<UnitId, bool>>,
) {
    if units.protected.contains(unit_id) && intended_status.as_str() != "none" {
        flags.protected_blocked = true;
        *intended_status = "none".to_string();
        *band_idx = None;
    }
    if let Some(map) = per_unit_quorum {
        if let Some(false) = map.get(unit_id).copied() {
            flags.quorum_blocked = true;
            *intended_status = "none".to_string();
            *band_idx = None;
        }
    }
}

fn tag_mediation_and_enclaves(
    _unit_id: &UnitId,
    _unit_map: &mut BTreeMap<UnitId, UnitFrontier>,
    _components: &Vec<BTreeSet<UnitId>>,
    _allowed: &AllowedEdges,
) {
    // (kept for parity with the design doc; mediation/enclave tagging happens inline in map_frontier())
}

fn update_summaries(out: &mut FrontierMap) {
    // Status counts
    let mut by_status: BTreeMap<String, u32> = BTreeMap::new();
    // Flag counts
    let mut f_mediation = 0u32;
    let mut f_enclave = 0u32;
    let mut f_prot = 0u32;
    let mut f_quorum = 0u32;

    for (_u, entry) in &out.units {
        *by_status.entry(entry.status.clone()).or_insert(0) += 1;
        if entry.flags.mediation {
            f_mediation += 1;
        }
        if entry.flags.enclave {
            f_enclave += 1;
        }
        if entry.flags.protected_blocked {
            f_prot += 1;
        }
        if entry.flags.quorum_blocked {
            f_quorum += 1;
        }
    }

    out.summary_by_status = by_status;
    out.summary_flags.insert("mediation", f_mediation);
    out.summary_flags.insert("enclave", f_enclave);
    out.summary_flags.insert("protected_blocked", f_prot);
    out.summary_flags.insert("quorum_blocked", f_quorum);
}
