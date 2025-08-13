//! Loader: read local JSON artifacts (manifest → registry → params → ballot_tally),
//! validate via Draft 2020-12 schemas, normalize ordering, and return a typed
//! `LoadedContext` for the pipeline. No network I/O.

#![forbid(unsafe_code)]

use crate::{IoError, canonical_json, hasher, manifest as man, schema};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs::File, io::Read, path::{Path, PathBuf}};
use vm_core::{
    determinism::StableOrd,
    entities::{DivisionRegistry, OptionItem, Unit},
    ids::{OptionId, UnitId},
    variables::{self, Params},
};

// ----------------------------- Public wire-facing types -----------------------------

/// Per-unit totals (mirrors `schemas/ballot_tally.schema.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Totals {
    pub valid_ballots: u64,
    pub invalid_ballots: u64,
}

/// One option’s count within a unit. JSON field is `votes`; we map into `count`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionCount {
    pub option_id: OptionId,
    #[serde(rename = "votes")]
    pub count: u64,
}

/// Tally for a single unit (array-based options, ordered by registry `order_index`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitTotals {
    pub unit_id: UnitId,
    pub totals: Totals,
    pub options: Vec<OptionCount>,
}

/// Ballot type (informative; not present in canonical input). Defaults to `Plurality`.
#[derive(Debug, Clone, Copy)]
pub enum BallotType {
    Plurality,
    Approval,
    Score,
    RankedIrv,
    RankedCondorcet,
}

/// Aggregated unit tallies (normative, array-based).
#[derive(Debug, Clone)]
pub struct UnitTallies {
    pub ballot_type: BallotType, // not from JSON; set by loader (default: Plurality)
    pub units: Vec<UnitTotals>,
}

/// Input digests (sha256 hex) of the three canonical inputs (+adjacency if present).
#[derive(Debug)]
pub struct InputDigests {
    pub division_registry_sha256: String,
    pub ballot_tally_sha256:      String,
    pub parameter_set_sha256:     String,
    pub adjacency_sha256:         Option<String>,
}

/// Loaded, validated, normalized context for the pipeline.
#[derive(Debug)]
pub struct LoadedContext {
    pub registry: DivisionRegistry,
    pub params:   Params,
    pub tally:    UnitTallies,
    pub adjacency_inline: Option<Vec<Adjacency>>,
    pub digests:  InputDigests,
}

// --- Adjacency placeholder import (domain lives in vm_core if/when defined) ---
#[allow(unused_imports)]
use vm_core::entities::Adjacency; // Keep aligned with repo plan; optional in this revision.

// ----------------------------- Orchestration -----------------------------

/// Load everything from a **manifest file path**: manifest → registry → params → tally (+adjacency).
pub fn load_all_from_manifest(path: &Path) -> Result<LoadedContext, IoError> {
    // 1) Manifest
    let man = man::load_manifest(path)?;
    let resolved = man::resolve_paths(path, &man)?;

    // 2) Registry
    let mut registry = load_registry(&resolved.reg)?;
    // normalize registry (units ↑ unit_id; each unit.options ↑ (order_index, option_id))
    for u in &mut registry.units {
        u.options = normalize_options(std::mem::take(&mut u.options));
    }
    registry.units = normalize_units(std::mem::take(&mut registry.units));

    // quick uniqueness check for order_index within each unit
    for u in &registry.units {
        let mut seen = BTreeSet::new();
        for opt in &u.options {
            if !seen.insert(opt.order_index) {
                return Err(IoError::Manifest(format!(
                    "duplicate order_index {} in unit {}", opt.order_index, u.unit_id
                )));
            }
        }
    }

    // 3) Params
    let params = load_params(&resolved.params)?;
    variables::validate_domains(&params)
        .map_err(|e| IoError::Manifest(format!("parameter domain error: {:?}", e)))?;

    // 4) Ballot Tally
    let mut tally = load_ballot_tally(&resolved.tally)?;
    // 5) Optional adjacency
    let adjacency_inline = match &resolved.adjacency {
        Some(p) => Some(load_adjacency(p)?),
        None => None,
    };

    // 6) Cross-refs & normalization of tally option order to registry order
    check_cross_refs(&registry, &[], &tally, adjacency_inline.as_deref())?;
    // Build per-unit canonical order from registry and re-order each tally.options list accordingly.
    let reg_unit_map: BTreeMap<&UnitId, &Unit> = registry.units.iter().map(|u| (&u.unit_id, u)).collect();
    for u in &mut tally.units {
        if let Some(reg_u) = reg_unit_map.get(&u.unit_id) {
            normalize_tally_options_unit(u, &reg_u.options);
        }
    }
    // And sort units ↑ unit_id
    tally.units.sort_by(|a, b| a.unit_id.cmp(&b.unit_id));

    // 7) Digests of canonical bytes (normative inputs only)
    let division_registry_sha256 = hasher::sha256_canonical(&registry)?;
    let ballot_tally_sha256      = {
        // For hashing, serialize back to the canonical on-wire shape (votes, not count).
        #[derive(Serialize)]
        struct OnWireUnit<'a> {
            unit_id: &'a UnitId,
            totals:  &'a Totals,
            options: Vec<OnWireOpt<'a>>,
        }
        #[derive(Serialize)]
        struct OnWireOpt<'a> { option_id: &'a OptionId, votes: u64 }
        #[derive(Serialize)]
        struct OnWire<'a> { schema_version: &'a str, units: Vec<OnWireUnit<'a>> }

        // By contract, schema_version is an opaque string; we pass through "1.x" here
        // to maintain a stable shape for canonical hashing when the source omitted it.
        let onwire = OnWire {
            schema_version: "1.x",
            units: tally.units.iter().map(|u| OnWireUnit {
                unit_id: &u.unit_id,
                totals:  &u.totals,
                options: u.options.iter().map(|o| OnWireOpt { option_id: &o.option_id, votes: o.count }).collect(),
            }).collect(),
        };
        hasher::sha256_canonical(&onwire)?
    };
    let parameter_set_sha256     = hasher::sha256_canonical(&params)?;
    let adjacency_sha256         = match &adjacency_inline {
        Some(adj) => Some(hasher::sha256_canonical(adj)?),
        None => None,
    };

    Ok(LoadedContext {
        registry,
        params,
        tally,
        adjacency_inline,
        digests: InputDigests {
            division_registry_sha256,
            ballot_tally_sha256,
            parameter_set_sha256,
            adjacency_sha256,
        },
    })
}

// ----------------------------- Targeted loaders -----------------------------

pub fn load_registry(path: &Path) -> Result<DivisionRegistry, IoError> {
    let v = read_json_value_with_limits(path)?;
    schema::validate_value(schema::SchemaKind::DivisionRegistry, &v)?;
    let reg: DivisionRegistry = serde_json::from_value(v)
        .map_err(|e| IoError::Json { pointer: "/".into(), msg: e.to_string() })?;
    Ok(reg)
}

pub fn load_params(path: &Path) -> Result<Params, IoError> {
    let v = read_json_value_with_limits(path)?;
    schema::validate_value(schema::SchemaKind::ParameterSet, &v)?;
    let ps: Params = serde_json::from_value(v)
        .map_err(|e| IoError::Json { pointer: "/".into(), msg: e.to_string() })?;
    Ok(ps)
}

pub fn load_ballot_tally(path: &Path) -> Result<UnitTallies, IoError> {
    // Raw, as per schema (options[].votes).
    #[derive(Deserialize)]
    struct RawOpt { option_id: OptionId, votes: u64 }
    #[derive(Deserialize)]
    struct RawUnit { unit_id: UnitId, totals: Totals, options: Vec<RawOpt> }
    #[derive(Deserialize)]
    struct RawTally { /* schema_version: String (ignored), */ units: Vec<RawUnit> }

    let v = read_json_value_with_limits(path)?;
    schema::validate_value(schema::SchemaKind::BallotTally, &v)?;
    let raw: RawTally = serde_json::from_value(v)
        .map_err(|e| IoError::Json { pointer: "/".into(), msg: e.to_string() })?;

    let units = raw.units.into_iter().map(|ru| UnitTotals {
        unit_id: ru.unit_id,
        totals: ru.totals,
        options: ru.options.into_iter().map(|ro| OptionCount { option_id: ro.option_id, count: ro.votes }).collect(),
    }).collect();

    Ok(UnitTallies { ballot_type: BallotType::Plurality, units })
}

pub fn load_adjacency(_path: &Path) -> Result<Vec<Adjacency>, IoError> {
    // Adjacency is optional and its schema/type are defined elsewhere in the project.
    // Wire it up here when the schema & vm_core::entities::Adjacency are finalized.
    Err(IoError::Manifest("adjacency loader not implemented in this revision".into()))
}

// ----------------------------- Canonicalization & checks -----------------------------

/// Sort units ↑ UnitId (lexicographic).
pub fn normalize_units(mut units: Vec<Unit>) -> Vec<Unit> {
    units.sort_by(|a, b| a.unit_id.cmp(&b.unit_id));
    units
}

/// Sort options ↑ (order_index, OptionId).
pub fn normalize_options(mut opts: Vec<OptionItem>) -> Vec<OptionItem> {
    opts.sort_by(|a, b| {
        match a.order_index.cmp(&b.order_index) {
            core::cmp::Ordering::Equal => a.option_id.cmp(&b.option_id),
            o => o,
        }
    });
    opts
}

/// Reorder the **unit's** tally options to reflect the registry’s canonical order.
fn normalize_tally_options_unit(u: &mut UnitTotals, reg_opts: &[OptionItem]) {
    let mut map: BTreeMap<&OptionId, u64> = BTreeMap::new();
    for oc in &u.options {
        // If duplicates in the tally, later entries overwrite — upstream should prevent this.
        map.insert(&oc.option_id, oc.count);
    }
    let mut out = Vec::with_capacity(reg_opts.len());
    for ro in reg_opts {
        if let Some(&cnt) = map.get(&ro.option_id) {
            out.push(OptionCount { option_id: ro.option_id.clone(), count: cnt });
        } else {
            // If a registry option has no count in tally, treat as zero (common in sparse tallies).
            out.push(OptionCount { option_id: ro.option_id.clone(), count: 0 });
        }
    }
    u.options = out;
}

/// Public wrapper kept for API parity (kept for future multi-unit reorder helpers).
pub fn normalize_tally_options(t: &mut UnitTallies, _order: &[OptionItem]) {
    // Intentionally no-op: we reorder per-unit using the registry’s per-unit option list
    // inside `load_all_from_manifest` where we have access to each unit's options.
}

/// Cross-file referential checks (lightweight, early failures).
pub
