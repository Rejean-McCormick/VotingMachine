//! variables.rs — Part 1/2
//! Canonical variable types, enums, and Params with safe defaults.
//! Part 2 will add validation helpers and FID inclusion lists (no mid-block splits).

use std::collections::BTreeMap;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{Error as DeError, Unexpected};
use serde_json::{self as json};

/// ------------ Utilities ------------

/// Allow only safe token chars and length (1..=64).
fn is_token(s: &str) -> bool {
    if s.is_empty() || s.len() > 64 { return false; }
    s.chars().all(|c|
        c.is_ascii_alphanumeric() ||
        matches!(c, '_' | '-' | '.' | ':')
    )
}

/// ------------ Macros ------------

/// Define a serde’d enum with explicit wire tokens.
/// (No feature gate on the macro itself; inner derives remain feature-aware.)
macro_rules! serde_enum {
    ($name:ident => { $($variant:ident = $token:expr),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(
                #[serde(rename = $token)]
                $variant,
            )+
        }
    };
}

/// ------------ Newtypes with invariants (validated on de/ser) ------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StringToken(String);

impl StringToken {
    pub fn new(s: impl Into<String>) -> Result<Self, String> {
        let s = s.into();
        if is_token(&s) { Ok(Self(s)) } else { Err(format!("invalid token: {s}")) }
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl<'de> Deserialize<'de> for StringToken {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if is_token(&s) { Ok(StringToken(s)) }
        else { Err(D::Error::invalid_value(Unexpected::Str(&s), &"token [A-Za-z0-9_.:-], len 1..=64")) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pct(u8); // 0..=100

impl Pct {
    pub fn new(v: u8) -> Result<Self, String> {
        if v <= 100 { Ok(Self(v)) } else { Err(format!("pct out of range: {v}")) }
    }
    pub fn as_u8(self) -> u8 { self.0 }
}

impl<'de> Deserialize<'de> for Pct {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = u8::deserialize(d)?;
        if v <= 100 { Ok(Pct(v)) }
        else { Err(D::Error::invalid_value(Unexpected::Unsigned(v as u64), &"0..=100")) }
    }
}

/// ------------ Canonical enums (wire tokens explicit) ------------

serde_enum!(AlgorithmVariant => {
    V1 = "v1"
});

serde_enum!(FrontierMode => {
    NoneMode     = "none",
    Basic        = "basic",
    Advanced     = "advanced"
});

serde_enum!(FrontierStrategy => {
    ApplyOnEntry     = "apply_on_entry",
    ApplyContinuously= "apply_continuously"
});

serde_enum!(ProtectedAreaOverride => {
    Allow = "allow",
    Deny  = "deny"
});

serde_enum!(FrontierBackoffPolicy => {
    None = "none",
    Linear = "linear",
    Exponential = "exponential"
});

serde_enum!(FrontierStrictness => {
    Strict = "strict",
    Lenient = "lenient"
});

serde_enum!(TiePolicy => {
    StatusQuo          = "status_quo",
    DeterministicOrder = "deterministic_order",
    Random             = "random"
});

serde_enum!(UnitSortOrder => {
    ByUnitIdAsc   = "by_unit_id_asc",
    ByScoreDesc   = "by_score_desc"
});

serde_enum!(TiesSectionVisibility => {
    Hidden   = "hidden",
    Collapsed= "collapsed",
    Expanded = "expanded"
});

serde_enum!(DecisivenessLabelPolicy => {
    StaticThreshold = "static_threshold",
    DynamicMargin   = "dynamic_margin"
});

serde_enum!(EligibilityMode => {
    Allow = "allow",
    Deny  = "deny"
});

/// ------------ Complex shapes ------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityOverride {
    pub unit_id: StringToken,
    pub mode: EligibilityMode,
}

/// Map from package token → percentage (0..=100)
pub type AutonomyPackageMap = BTreeMap<StringToken, Pct>;

/// Scope of a run (externally tagged shape kept for clarity)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RunScope {
    #[serde(rename = "all_units")]
    AllUnits,
    #[serde(rename = "selector")]
    Selector(StringToken),
}

/// ------------ Params (Included vs Excluded variables) ------------
/// Included vars affect FID; excluded vars are presentation-only or runtime noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    // Included — identification / release
    pub v001_release: StringToken,
    pub v002_region: StringToken,
    pub v003_phase: u8,                 // domain: 0..=6 (validated later)
    pub v004_dataset_id: StringToken,
    pub v005_run_id: StringToken,
    pub v006_engine_version: StringToken,
    pub v007_formula_name: StringToken,

    // Included — core switches
    pub v024_flag_a: bool,
    pub v025_flag_b: bool,
    pub v026: i64,
    pub v027: i64,
    pub v028: i64,

    pub v029_symmetry_exceptions: Vec<StringToken>,       // will be dedup/sorted in validation
    pub v030_eligibility_override_list: Vec<EligibilityOverride>, // dedup/sorted in validation
    pub v031_ballot_integrity_floor: Pct,

    // Included — frontier behavior
    pub v040_frontier_mode: FrontierMode,
    pub v041_frontier_cut: f32,         // consider fixed-point if cross-lang canonicalization is an issue
    pub v042_frontier_strategy: FrontierStrategy,
    pub v045_protected_area_override: ProtectedAreaOverride,
    pub v046_autonomy_package_map: AutonomyPackageMap,
    pub v047_frontier_band_window: f64, // 0.0..=1.0
    pub v048_frontier_backoff_policy: FrontierBackoffPolicy,
    pub v049_frontier_strictness: FrontierStrictness,

    // Included — determinism / algorithm
    pub v050_tie_policy: TiePolicy,
    pub v073_algorithm_variant: AlgorithmVariant,

    // Excluded — presentation / runtime toggles (do NOT enter FID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v032_unit_sort_order: Option<UnitSortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v033_ties_section_visibility: Option<TiesSectionVisibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v034_frontier_map_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v035_sensitivity_analysis_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v052_tie_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v060_majority_label_threshold: Option<Pct>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v061_decisiveness_label_policy: Option<DecisivenessLabelPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v062_explanations_enabled: Option<bool>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            // Included — identification / release
            v001_release: StringToken::new("default").unwrap(),
            v002_region: StringToken::new("global").unwrap(),
            v003_phase: 0,
            v004_dataset_id: StringToken::new("dataset").unwrap(),
            v005_run_id: StringToken::new("run").unwrap(),
            v006_engine_version: StringToken::new("vm-engine-v0").unwrap(),
            v007_formula_name: StringToken::new("baseline").unwrap(),

            // Included — core switches
            v024_flag_a: true,
            v025_flag_b: true,
            v026: 0,
            v027: 0,
            v028: 0,

            v029_symmetry_exceptions: Vec::new(),
            v030_eligibility_override_list: Vec::new(),
            v031_ballot_integrity_floor: Pct::new(0).unwrap(),

            // Included — frontier behavior
            v040_frontier_mode: FrontierMode::NoneMode,
            v041_frontier_cut: 0.0,
            v042_frontier_strategy: FrontierStrategy::ApplyOnEntry,
            v045_protected_area_override: ProtectedAreaOverride::Deny,
            v046_autonomy_package_map: AutonomyPackageMap::new(),
            v047_frontier_band_window: 0.0,
            v048_frontier_backoff_policy: FrontierBackoffPolicy::None,
            v049_frontier_strictness: FrontierStrictness::Strict,

            // Included — determinism / algorithm
            v050_tie_policy: TiePolicy::StatusQuo,
            v073_algorithm_variant: AlgorithmVariant::V1,

            // Excluded — presentation / runtime toggles
            // Keep consistent with frontier_mode: map disabled by default when mode is none.
            v032_unit_sort_order: None,
            v033_ties_section_visibility: None,
            v034_frontier_map_enabled: Some(false), // FIX: do not default to true when frontier is off
            v035_sensitivity_analysis_enabled: Some(false),
            v052_tie_seed: None,                    // FIX: do not set a seed by default
            v060_majority_label_threshold: None,
            v061_decisiveness_label_policy: Some(DecisivenessLabelPolicy::DynamicMargin),
            v062_explanations_enabled: None,
        }
    }
}
//! variables.rs — Part 2/2
//! Validation helpers, normalization for FID, and the canonical Included/Excluded lists.

use std::collections::{BTreeMap, BTreeSet};
use serde_json::{self as json, Value};

use super::{
    StringToken, Pct,
    AlgorithmVariant, FrontierMode, FrontierStrategy, ProtectedAreaOverride,
    FrontierBackoffPolicy, FrontierStrictness, TiePolicy, UnitSortOrder,
    TiesSectionVisibility, DecisivenessLabelPolicy, EligibilityMode,
    EligibilityOverride, Params,
};

/// -------- Canonical variable lists (for FID computation) --------
/// Included variables affect the Formula ID (FID).
pub const FID_KEYS: &[&str] = &[
    "VM-VAR-001","VM-VAR-002","VM-VAR-003","VM-VAR-004","VM-VAR-005","VM-VAR-006","VM-VAR-007",
    "VM-VAR-024","VM-VAR-025","VM-VAR-026","VM-VAR-027","VM-VAR-028","VM-VAR-029","VM-VAR-030","VM-VAR-031",
    "VM-VAR-040","VM-VAR-041","VM-VAR-042","VM-VAR-045","VM-VAR-046","VM-VAR-047","VM-VAR-048","VM-VAR-049",
    "VM-VAR-050","VM-VAR-073",
];

/// Excluded variables do NOT enter FID (presentation/runtime).
pub const EXCLUDED_KEYS: &[&str] = &[
    "VM-VAR-032","VM-VAR-033","VM-VAR-034","VM-VAR-035",
    "VM-VAR-052","VM-VAR-060","VM-VAR-061","VM-VAR-062",
];

/// Deterministic key iteration for FID construction.
pub fn iter_fid_keys() -> impl Iterator<Item = &'static str> {
    FID_KEYS.iter().copied()
}

/// -------- Normalization (keeps semantically equal manifests byte-equal) --------

impl Params {
    /// Normalize fields that can vary in order / duplicates but should not affect semantics.
    /// Call before serializing for FID.
    pub fn normalize_for_fid(&mut self) {
        self.normalize_v029_symmetry_exceptions();
        self.normalize_v030_eligibility_overrides();
    }

    fn normalize_v029_symmetry_exceptions(&mut self) {
        if self.v029_symmetry_exceptions.is_empty() { return; }
        // Dedup + sort (by token string)
        let mut set: BTreeSet<String> =
            self.v029_symmetry_exceptions.iter().map(|t| t.as_str().to_string()).collect();
        self.v029_symmetry_exceptions = set.into_iter()
            .map(|s| StringToken::new(s).expect("token"))
            .collect();
    }

    fn normalize_v030_eligibility_overrides(&mut self) {
        if self.v030_eligibility_override_list.is_empty() { return; }
        // Dedup on (unit_id, mode), sort by unit_id then mode token
        let mut uniq = BTreeMap::<(String, &'static str), EligibilityMode>::new();
        for e in self.v030_eligibility_override_list.drain(..) {
            let key = (e.unit_id.as_str().to_string(), token_of_eligibility_mode(e.mode));
            uniq.entry(key).or_insert(e.mode);
        }
        let mut out: Vec<EligibilityOverride> = uniq.into_iter().map(|((unit, _), mode)| {
            EligibilityOverride { unit_id: StringToken::new(unit).expect("token"), mode }
        }).collect();
        out.sort_by(|a,b| {
            let ka = (a.unit_id.as_str(), token_of_eligibility_mode(a.mode));
            let kb = (b.unit_id.as_str(), token_of_eligibility_mode(b.mode));
            ka.cmp(&kb)
        });
        self.v030_eligibility_override_list = out;
    }

    /// Frontier enabled at the algorithm level (independent of UI toggle).
    pub fn algo_frontier_enabled(&self) -> bool {
        !matches!(self.v040_frontier_mode, FrontierMode::NoneMode)
    }

    /// Frontier enabled for emission (algo + presentation).
    pub fn frontier_enabled(&self) -> bool {
        self.algo_frontier_enabled() && self.v034_frontier_map_enabled.unwrap_or(false)
    }
}

fn token_of_eligibility_mode(m: EligibilityMode) -> &'static str {
    match m {
        EligibilityMode::Allow => "allow",
        EligibilityMode::Deny  => "deny",
    }
}

/// -------- Validation (domain + cross-field consistency) --------

#[derive(Debug)]
pub enum VarsError {
    Domain(String),
    Consistency(String),
}

pub type VarsResult<T> = Result<T, VarsError>;

impl Params {
    /// Validate basic numeric/string domains and cross-field consistency.
    pub fn validate_domains(&self) -> VarsResult<()> {
        // Phase: 0..=6
        if self.v003_phase > 6 {
            return Err(VarsError::Domain(format!("v003_phase out of range: {}", self.v003_phase)));
        }
        // Percent ranges (already enforced by Pct; treat as defense-in-depth)
        let p = self.v031_ballot_integrity_floor.as_u8();
        if p > 100 {
            return Err(VarsError::Domain(format!("v031_ballot_integrity_floor out of range: {}", p)));
        }
        // Frontier band window: 0.0..=1.0 and finite
        if !self.v047_frontier_band_window.is_finite() ||
            self.v047_frontier_band_window < 0.0 || self.v047_frontier_band_window > 1.0 {
            return Err(VarsError::Domain(format!(
                "v047_frontier_band_window must be finite in [0.0,1.0], got {}",
                self.v047_frontier_band_window
            )));
        }
        // Frontier cut finite (range may be algorithm-specific; ensure finite)
        if !self.v041_frontier_cut.is_finite() {
            return Err(VarsError::Domain("v041_frontier_cut must be finite".into()));
        }
        // Release-specific coarse guards on v026..v028 (prevent unbounded FID drift)
        for (k, v) in [("v026", self.v026), ("v027", self.v027), ("v028", self.v028)] {
            if v.abs() > 10_000 {
                return Err(VarsError::Domain(format!("{k} magnitude too large: {v}")));
            }
        }

        // --- Cross-field consistency ---

        // Frontier presentation toggle cannot be true when algorithmic mode is None
        if matches!(self.v040_frontier_mode, FrontierMode::NoneMode)
            && self.v034_frontier_map_enabled == Some(true)
        {
            return Err(VarsError::Consistency(
                "v034_frontier_map_enabled=true while v040_frontier_mode='none'".into()
            ));
        }

        // RNG seed must not be set unless policy is random (actual tie occurrence is checked at run time)
        if self.v052_tie_seed.is_some() && !matches!(self.v050_tie_policy, TiePolicy::Random) {
            return Err(VarsError::Consistency(
                "v052_tie_seed provided but v050_tie_policy is not 'random'".into()
            ));
        }

        Ok(())
    }
}

/// -------- Helpers to extract Included vars as JSON for FID building --------

fn j<T: serde::Serialize>(v: &T) -> Value {
    json::to_value(v).expect("serialize")
}

/// Return Included (key, value) pairs in deterministic order.
/// Call `normalize_for_fid()` first to ensure stable collections.
pub fn fid_kvs(params: &Params) -> Vec<(&'static str, Value)> {
    let mut out: Vec<(&'static str, Value)> = Vec::with_capacity(FID_KEYS.len());

    // 001..007
    out.push(("VM-VAR-001", j(&params.v001_release)));
    out.push(("VM-VAR-002", j(&params.v002_region)));
    out.push(("VM-VAR-003", j(&params.v003_phase)));
    out.push(("VM-VAR-004", j(&params.v004_dataset_id)));
    out.push(("VM-VAR-005", j(&params.v005_run_id)));
    out.push(("VM-VAR-006", j(&params.v006_engine_version)));
    out.push(("VM-VAR-007", j(&params.v007_formula_name)));

    // 024..031
    out.push(("VM-VAR-024", j(&params.v024_flag_a)));
    out.push(("VM-VAR-025", j(&params.v025_flag_b)));
    out.push(("VM-VAR-026", j(&params.v026)));
    out.push(("VM-VAR-027", j(&params.v027)));
    out.push(("VM-VAR-028", j(&params.v028)));
    out.push(("VM-VAR-029", j(&params.v029_symmetry_exceptions)));
    out.push(("VM-VAR-030", j(&params.v030_eligibility_override_list)));
    out.push(("VM-VAR-031", j(&params.v031_ballot_integrity_floor)));

    // 040..049
    out.push(("VM-VAR-040", j(&params.v040_frontier_mode)));
    out.push(("VM-VAR-041", j(&params.v041_frontier_cut)));
    out.push(("VM-VAR-042", j(&params.v042_frontier_strategy)));
    out.push(("VM-VAR-045", j(&params.v045_protected_area_override)));
    out.push(("VM-VAR-046", j(&params.v046_autonomy_package_map)));
    out.push(("VM-VAR-047", j(&params.v047_frontier_band_window)));
    out.push(("VM-VAR-048", j(&params.v048_frontier_backoff_policy)));
    out.push(("VM-VAR-049", j(&params.v049_frontier_strictness)));

    // 050, 073
    out.push(("VM-VAR-050", j(&params.v050_tie_policy)));
    out.push(("VM-VAR-073", j(&params.v073_algorithm_variant)));

    out
}

/// Quick check that our Included/Excluded sets match expectations.
/// (Optional; useful in tests/invariants.)
pub fn check_included_excluded_sets() -> Result<(), String> {
    let inc: BTreeSet<&'static str> = FID_KEYS.iter().copied().collect();
    let exc: BTreeSet<&'static str> = EXCLUDED_KEYS.iter().copied().collect();
    // No overlap
    if !inc.is_disjoint(&exc) {
        return Err("Included and Excluded sets overlap".into());
    }
    // Spot-check presence of required keys
    for k in ["VM-VAR-001","VM-VAR-007","VM-VAR-050","VM-VAR-073"] {
        if !inc.contains(k) {
            return Err(format!("missing required Included key: {k}"));
        }
    }
    Ok(())
}
