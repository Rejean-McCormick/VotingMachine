//! VM variables (VM-VAR-###) as typed domains + `Params` snapshot.
//!
//! - Strong enums for frontier/ties/etc.
//! - `Params::default()` provides per-release defaults (placeholder here).
//! - `validate_domains(&Params)` enforces **domain-only** rules (no I/O).
//! - Optional `serde` derives (no `serde_json` here).
//!
//! This module is I/O-free and hashing/canonicalization are handled elsewhere.

#![allow(clippy::upper_case_acronyms)]

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::tokens::UnitId;

/* ------------------------------- Error types ------------------------------ */

/// Validation errors for VM variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarError {
    OutOfRange { var: &'static str },
    BadToken { var: &'static str },
    Unsupported { var: &'static str },
}

/* ---------------------------------- Enums --------------------------------- */

#[cfg(feature = "serde")]
macro_rules! serde_enum {
    ($name:ident $(,$attr:meta)*) => {
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name
    };
}

#[cfg(not(feature = "serde"))]
macro_rules! serde_enum {
    ($name:ident $(,$attr:meta)*) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name
    };
}

/// VM-VAR-050 — tie policy (Included in FID).
serde_enum!(TiePolicy) {
    StatusQuo,
    DeterministicOrder,
    Random,
}

/// VM-VAR-073 — algorithm variant (Included in FID).
/// NOTE: Variants are per-release. Keep this list synchronized with Docs.
serde_enum!(AlgorithmVariant, non_exhaustive) {
    V1,
    // Add release-specific variants here.
}

/// VM-VAR-040 — frontier mode (Included).
serde_enum!(FrontierMode) {
    None,
    Banded,
    Ladder,
}

/// VM-VAR-042 — frontier strategy (Included).
serde_enum!(FrontierStrategy) {
    ApplyOnEntry,
    ApplyOnExit,
    Sticky,
}

/// VM-VAR-045 — protected area override (Included).
serde_enum!(ProtectedAreaOverride) {
    Deny,
    Allow,
}

/// VM-VAR-048 — frontier backoff policy (Included).
serde_enum!(FrontierBackoffPolicy) {
    None,
    Soften,
    Harden,
}

/// VM-VAR-049 — frontier strictness (Included).
serde_enum!(FrontierStrictness) {
    Strict,
    Lenient,
}

/// VM-VAR-032 — (Excluded) presentation.
serde_enum!(UnitSortOrder) {
    UnitId,
    LabelPriority,
    Turnout,
}

/// VM-VAR-033 — (Excluded) presentation.
serde_enum!(TiesSectionVisibility) {
    Auto,
    Always,
    Never,
}

/// VM-VAR-061 — (Excluded) presentation.
serde_enum!(DecisivenessLabelPolicy) {
    Fixed,
    DynamicMargin,
}

/* -------------------------------- Newtypes -------------------------------- */

/// Token used by selectors/symmetry lists — `[A-Za-z0-9_.:-]{1,64}`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringToken(String);

impl StringToken {
    pub fn new(s: &str) -> Result<Self, VarError> {
        if is_token(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(VarError::BadToken { var: "StringToken" })
        }
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Percent in 0..=100 (VM many vars).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pct(u8);

impl Pct {
    pub const fn new(v: u8) -> Result<Self, VarError> {
        if v <= 100 { Ok(Self(v)) } else { Err(VarError::OutOfRange { var: "Pct" }) }
    }
    pub const fn get(self) -> u8 { self.0 }
}

/* --------------------------------- Minor ---------------------------------- */

/// VM-VAR-021 — run scope.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value", rename_all = "snake_case"))]
pub enum RunScope {
    AllUnits,
    Selector(StringToken),
}

/// VM-VAR-030 — eligibility override.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EligibilityOverride {
    pub unit_id: UnitId,
    pub mode: EligibilityMode,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EligibilityMode { Include, Exclude }

/// VM-VAR-046 — deterministic-key map (domain defined per release).
pub type AutonomyPackageMap = BTreeMap<StringToken, StringToken>;

/* --------------------------------- Params --------------------------------- */

/// Typed snapshot of VM variables. Included (FID) vars are required; Excluded are optional.
///
/// NOTE: String-typed enums (001/002/004/005/006/007) are modeled as `String` to allow
/// per-release set evolution without changing the core crate.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Params {
    /* Included (FID) — required */
    /// VM-VAR-001: algorithm family (enum per release)
    pub v001_algorithm_family: String,
    /// VM-VAR-002: rounding policy (enum per release)
    pub v002_rounding_policy: String,
    /// VM-VAR-003: share precision 0..=6
    pub v003_share_precision: u8,
    /// VM-VAR-004..006: strings per family
    pub v004_denom_rule: String,
    pub v005_aggregation_mode: String,
    pub v006_seat_allocation_rule: String,
    /// VM-VAR-007: tie scope model (e.g., "winner_only" | "rank_all")
    pub v007_tie_scope_model: String,

    // Thresholds/gates
    pub v010: Pct, pub v011: Pct, pub v012: Pct, pub v013: Pct,
    pub v014: Pct, pub v015: Pct, pub v016: Pct, pub v017: Pct,
    pub v020: Pct,

    /// VM-VAR-021
    pub v021_run_scope: RunScope,

    pub v022: Pct, pub v023: Pct,
    /// VM-VAR-024..025 — booleans (if defined in release)
    pub v024_flag_a: bool,
    pub v025_flag_b: bool,
    /// VM-VAR-026..028 — numeric knobs (domain per release)
    pub v026: i32,
    pub v027: i32,
    pub v028: i32,

    /// VM-VAR-029 — deterministic selectors
    pub v029_symmetry_exceptions: Vec<StringToken>,
    /// VM-VAR-030
    pub v030_eligibility_override_list: Vec<EligibilityOverride>,
    /// VM-VAR-031
    pub v031_ballot_integrity_floor: Pct,

    /// Frontier (040..049)
    pub v040_frontier_mode: FrontierMode,
    pub v041_frontier_cut: f32,          // domain per mode
    pub v042_frontier_strategy: FrontierStrategy,
    pub v045_protected_area_override: ProtectedAreaOverride,
    pub v046_autonomy_package_map: AutonomyPackageMap,
    pub v047_frontier_band_window: f32,  // 0.0..=1.0
    pub v048_frontier_backoff_policy: FrontierBackoffPolicy,
    pub v049_frontier_strictness: FrontierStrictness,

    /// Ties (050 in FID)
    pub v050_tie_policy: TiePolicy,

    /// Variant (073 in FID)
    pub v073_algorithm_variant: AlgorithmVariant,

    /* Excluded (non-FID) — optional */
    pub v032_unit_sort_order: Option<UnitSortOrder>,
    pub v033_ties_section_visibility: Option<TiesSectionVisibility>,
    pub v034_frontier_map_enabled: Option<bool>,
    pub v035_sensitivity_analysis_enabled: Option<bool>,
    /// VM-VAR-052 — integer seed ≥ 0 (recorded in RunRecord iff a random tie occurred)
    pub v052_tie_seed: Option<u64>,
    pub v060_majority_label_threshold: Option<Pct>,
    pub v061_decisiveness_label_policy: Option<DecisivenessLabelPolicy>,
    /// "auto" or IETF tag (opaque)
    pub v062_unit_display_language: Option<String>,
}

/* -------------------------------- Defaults -------------------------------- */

impl Default for Params {
    fn default() -> Self {
        // NOTE: These are sane placeholders; set to your per-release defaults.
        Self {
            v001_algorithm_family: "family_v1".to_string(),
            v002_rounding_policy: "half_up".to_string(),
            v003_share_precision: 3,
            v004_denom_rule: "standard".to_string(),
            v005_aggregation_mode: "sum".to_string(),
            v006_seat_allocation_rule: "none".to_string(),
            v007_tie_scope_model: "winner_only".to_string(),

            v010: Pct::new(0).unwrap(), v011: Pct::new(0).unwrap(),
            v012: Pct::new(0).unwrap(), v013: Pct::new(0).unwrap(),
            v014: Pct::new(0).unwrap(), v015: Pct::new(0).unwrap(),
            v016: Pct::new(0).unwrap(), v017: Pct::new(0).unwrap(),
            v020: Pct::new(0).unwrap(),

            v021_run_scope: RunScope::AllUnits,

            v022: Pct::new(0).unwrap(),
            v023: Pct::new(0).unwrap(),

            v024_flag_a: true,
            v025_flag_b: true,

            v026: 0,
            v027: 0,
            v028: 0,

            v029_symmetry_exceptions: Vec::new(),
            v030_eligibility_override_list: Vec::new(),
            v031_ballot_integrity_floor: Pct::new(0).unwrap(),

            v040_frontier_mode: FrontierMode::None,
            v041_frontier_cut: 0.0,
            v042_frontier_strategy: FrontierStrategy::ApplyOnEntry,
            v045_protected_area_override: ProtectedAreaOverride::Deny,
            v046_autonomy_package_map: AutonomyPackageMap::new(),
            v047_frontier_band_window: 0.0,
            v048_frontier_backoff_policy: FrontierBackoffPolicy::None,
            v049_frontier_strictness: FrontierStrictness::Strict,

            v050_tie_policy: TiePolicy::StatusQuo,
            v073_algorithm_variant: AlgorithmVariant::V1,

            v032_unit_sort_order: None,
            v033_ties_section_visibility: None,
            v034_frontier_map_enabled: Some(true),
            v035_sensitivity_analysis_enabled: Some(false),
            v052_tie_seed: Some(0),
            v060_majority_label_threshold: None,
            v061_decisiveness_label_policy: Some(DecisivenessLabelPolicy::DynamicMargin),
            v062_unit_display_language: Some("auto".to_string()),
        }
    }
}

/* ------------------------------- Validation -------------------------------- */

/// Domain-only validation (no cross-artifact checks).
pub fn validate_domains(p: &Params) -> Result<(), VarError> {
    // v003: 0..=6
    if p.v003_share_precision > 6 {
        return Err(VarError::OutOfRange { var: "VM-VAR-003" });
    }

    // Percentages are enforced by `Pct`, but double-check to be robust.
    for (name, pct) in [
        ("VM-VAR-010", p.v010.get()), ("VM-VAR-011", p.v011.get()),
        ("VM-VAR-012", p.v012.get()), ("VM-VAR-013", p.v013.get()),
        ("VM-VAR-014", p.v014.get()), ("VM-VAR-015", p.v015.get()),
        ("VM-VAR-016", p.v016.get()), ("VM-VAR-017", p.v017.get()),
        ("VM-VAR-020", p.v020.get()), ("VM-VAR-022", p.v022.get()),
        ("VM-VAR-023", p.v023.get()), ("VM-VAR-031", p.v031_ballot_integrity_floor.get()),
    ] {
        if pct > 100 {
            return Err(VarError::OutOfRange { var: name });
        }
    }

    // v047: 0.0..=1.0
    if !(0.0..=1.0).contains(&p.v047_frontier_band_window) {
        return Err(VarError::OutOfRange { var: "VM-VAR-047" });
    }

    // v041: if constrained per mode, put numeric guards here; we keep it finite.
    if !p.v041_frontier_cut.is_finite() {
        return Err(VarError::OutOfRange { var: "VM-VAR-041" });
    }

    // v052: seed is u64 if present — always >= 0 by type.

    // v029: StringToken ensures domain; still reject empty vector is OK (allowed).

    // v030: UnitId domain handled by type; list may be empty (allowed).
    Ok(())
}

/* ------------------------------ Convenience -------------------------------- */

impl Params {
    #[inline]
    pub fn is_random_ties(&self) -> bool {
        matches!(self.v050_tie_policy, TiePolicy::Random)
    }

    #[inline]
    pub fn frontier_enabled(&self) -> bool {
        !matches!(self.v040_frontier_mode, FrontierMode::None)
    }

    /// Example convenience normalization (may be used by algorithms).
    #[inline]
    pub fn pr_threshold(&self) -> Option<u8> {
        Some(self.v022.get())
    }

    /// Iterate **Included** (FID) variable IDs in canonical order (keys only).
    pub fn iter_fid_keys(&self) -> impl Iterator<Item = &'static str> {
        FID_KEYS.iter().copied()
    }
}

/* ------------------------------ FID Iteration ------------------------------ */

/// Static list of Included (FID) keys, canonical order.
const FID_KEYS: &[&str] = &[
    "VM-VAR-001","VM-VAR-002","VM-VAR-003","VM-VAR-004","VM-VAR-005","VM-VAR-006","VM-VAR-007",
    "VM-VAR-010","VM-VAR-011","VM-VAR-012","VM-VAR-013","VM-VAR-014","VM-VAR-015","VM-VAR-016","VM-VAR-017",
    "VM-VAR-020","VM-VAR-021","VM-VAR-022","VM-VAR-023","VM-VAR-024","VM-VAR-025","VM-VAR-026","VM-VAR-027","VM-VAR-028","VM-VAR-029","VM-VAR-030","VM-VAR-031",
    "VM-VAR-040","VM-VAR-041","VM-VAR-042","VM-VAR-045","VM-VAR-046","VM-VAR-047","VM-VAR-048","VM-VAR-049",
    "VM-VAR-050","VM-VAR-073",
];

/* --------------------------------- Helpers -------------------------------- */

#[inline]
fn is_token(s: &str) -> bool {
    let len = s.len();
    if !(1..=64).contains(&len) { return false; }
    s.bytes().all(|b| matches!(b,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' |
        b'_' | b'-' | b':' | b'.'
    ))
}
