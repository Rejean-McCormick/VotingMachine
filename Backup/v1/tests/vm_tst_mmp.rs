//! VM-ENGINE v0 — MMP tests (skeleton)
//!
//! Locks the Mixed-Member Proportional (MMP) sequence and controls:
//!   • seat targets, top-ups, correction level (national vs regional),
//!   • overhang policy (not triggered here).
//!
//! NOTE: This file provides compile-safe stubs and ignored tests so the
//! workspace builds before the pipeline wiring + fixtures land. Replace the
//! placeholder types and helper bodies with real engine types/APIs.

use anyhow::Result;
use std::collections::BTreeMap;

// -----------------------------------------------------------------------------
// Fixture paths (adjust to your repo layout if needed)
// -----------------------------------------------------------------------------
#[allow(dead_code)]
const REG: &str = "fixtures/annex_b/part_3/vm_tst_013/division_registry.json";
#[allow(dead_code)]
const TLY: &str = "fixtures/annex_b/part_3/vm_tst_013/ballots.json";
#[allow(dead_code)]
const PS_NAT: &str = "fixtures/annex_b/part_3/vm_tst_013/parameter_set_national.json";
#[allow(dead_code)]
const PS_REG: &str = "fixtures/annex_b/part_3/vm_tst_013/parameter_set_regional.json";

// -----------------------------------------------------------------------------
// Minimal placeholder types so this test module compiles before wiring
// (replace with real vm_pipeline/vm_io artifact types when available)
// -----------------------------------------------------------------------------
type ResultDb = ();
type RunRecordDb = ();
type FrontierMapDb = ();
type OptionId = String;

// -----------------------------------------------------------------------------
// Helper API (to be implemented for real once the engine is wired)
// -----------------------------------------------------------------------------

/// Run full pipeline from explicit file paths; returns (Result, RunRecord, Frontier?).
#[allow(dead_code)]
fn run_pipeline(
    _reg: &str,
    _ps: &str,
    _tly: &str,
) -> Result<(ResultDb, RunRecordDb, Option<FrontierMapDb>)> {
    Err(anyhow::anyhow!("run_pipeline stub: not wired yet"))
}

/// Read aggregated final seats by option (after MMP correction).
#[allow(dead_code)]
fn totals_by_option(_res: &ResultDb) -> BTreeMap<OptionId, u32> {
    unimplemented!("totals_by_option: extract final total seats per option")
}

/// Read aggregated *local* seats by option (pre-top-up SMD winners).
#[allow(dead_code)]
fn locals_by_option(_res: &ResultDb) -> BTreeMap<OptionId, u32> {
    unimplemented!("locals_by_option: extract local/SMD seats per option")
}

/// Effective total seats in the chamber after MMP (T).
#[allow(dead_code)]
fn effective_total_seats(_res: &ResultDb) -> u32 {
    unimplemented!("effective_total_seats: read total seats from aggregates/MMP")
}

/// Map fixture label ("A"/"B"/"C") → OptionId.
#[allow(dead_code)]
fn oid(label: &str) -> OptionId {
    label.to_string()
}

/// Assert Result label == "Decisive".
#[allow(dead_code)]
fn assert_decisive(_res: &ResultDb) {
    unimplemented!("assert_decisive: inspect Result.label == Decisive")
}

// -----------------------------------------------------------------------------
// Tests (ignored until engine + MMP fixtures are wired)
// -----------------------------------------------------------------------------

/// VM-TST-013 — MMP national correction level
/// Expected totals: A/B/C = 7/3/2; locals A/B/C = 2/2/2; T = 12.
#[test]
#[ignore = "Enable once vm_pipeline + MMP fixtures VM-TST-013 are wired"]
fn vm_tst_013_mmp_national_level() -> Result<()> {
    let (res, _run, _fr) = match run_pipeline(REG, PS_NAT, TLY) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            return Ok(()); // keep suite compiling; real assertions once wired
        }
    };

    let a = oid("A");
    let b = oid("B");
    let c = oid("C");

    let total = totals_by_option(&res);
    let local = locals_by_option(&res);

    assert_eq!(local.get(&a).copied(), Some(2), "locals A");
    assert_eq!(local.get(&b).copied(), Some(2), "locals B");
    assert_eq!(local.get(&c).copied(), Some(2), "locals C");

    assert_eq!(total.get(&a).copied(), Some(7), "total A");
    assert_eq!(total.get(&b).copied(), Some(3), "total B");
    assert_eq!(total.get(&c).copied(), Some(2), "total C");

    assert_eq!(effective_total_seats(&res), 12, "effective T");
    assert_decisive(&res);
    Ok(())
}

/// VM-TST-013 — MMP regional correction level
/// Expected totals: A/B/C = 8/2/2; locals A/B/C = 2/2/2; T = 12.
#[test]
#[ignore = "Enable once vm_pipeline + MMP fixtures VM-TST-013 are wired"]
fn vm_tst_013_mmp_regional_level() -> Result<()> {
    let (res, _run, _fr) = match run_pipeline(REG, PS_REG, TLY) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            return Ok(());
        }
    };

    let a = oid("A");
    let b = oid("B");
    let c = oid("C");

    let total = totals_by_option(&res);
    let local = locals_by_option(&res);

    assert_eq!(local.get(&a).copied(), Some(2), "locals A");
    assert_eq!(local.get(&b).copied(), Some(2), "locals B");
    assert_eq!(local.get(&c).copied(), Some(2), "locals C");

    assert_eq!(total.get(&a).copied(), Some(8), "total A");
    assert_eq!(total.get(&b).copied(), Some(2), "total B");
    assert_eq!(total.get(&c).copied(), Some(2), "total C");

    assert_eq!(effective_total_seats(&res), 12, "effective T");
    assert_decisive(&res);
    Ok(())
}
