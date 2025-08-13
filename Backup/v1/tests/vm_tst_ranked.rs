//! VM-ENGINE v0 — Ranked methods tests (skeleton)
//!
//! Locks behavior for:
//!   (a) IRV with exhaustion (reduce_continuing_denominator)
//!   (b) Condorcet with Schulze completion.
//!
//! This file wires test *signatures* and deterministic assertions, but leaves the
//! engine orchestration as TODO. Until the ranked fixtures + pipeline wiring land,
//! the tests are `#[ignore]` so the crate compiles cleanly on all OSes.

use anyhow::Result;

// -----------------------------------------------------------------------------
// Fixture paths (adjust to your repo layout if needed)
// -----------------------------------------------------------------------------
#[allow(dead_code)]
const REG_IRV: &str = "fixtures/annex_b/part_3/vm_tst_010/division_registry.json";
#[allow(dead_code)]
const PS_IRV: &str = "fixtures/annex_b/part_3/vm_tst_010/parameter_set.json";
#[allow(dead_code)]
const TLY_IRV: &str = "fixtures/annex_b/part_3/vm_tst_010/ballots.json";

#[allow(dead_code)]
const REG_COND: &str = "fixtures/annex_b/part_3/vm_tst_011/division_registry.json";
#[allow(dead_code)]
const PS_COND: &str = "fixtures/annex_b/part_3/vm_tst_011/parameter_set.json";
#[allow(dead_code)]
const TLY_COND: &str = "fixtures/annex_b/part_3/vm_tst_011/ballots.json";

// -----------------------------------------------------------------------------
// Minimal placeholder types so this test module compiles before wiring
// (replace with real vm_pipeline/vm_io artifact types when available)
// -----------------------------------------------------------------------------
type ResultDb = ();         // replace with concrete Result artifact type
type RunRecordDb = ();      // replace with concrete RunRecord type
type FrontierMapDb = ();    // replace with concrete FrontierMap type
type UnitId = String;       // engine UnitId newtype/string
type OptionId = String;     // engine OptionId newtype/string
type PairwiseMatrix = ();   // replace with concrete pairwise type

// -----------------------------------------------------------------------------
// Public test API (to be implemented for real once the engine is wired)
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

/// Return the first UnitId found in the Result (stable order).
#[allow(dead_code)]
fn first_unit_id(_res: &ResultDb) -> UnitId {
    unimplemented!("first_unit_id: map Result → first UnitId")
}

/// Winner OptionId for a unit from ranked tabulation outcome.
#[allow(dead_code)]
fn winner_of(_res: &ResultDb, _unit: UnitId) -> OptionId {
    unimplemented!("winner_of: extract ranked winner for unit")
}

/// Final tally for an option in a unit (IRV last round or Condorcet winner’s support as needed).
#[allow(dead_code)]
fn final_tally(_res: &ResultDb, _unit: UnitId, _opt: OptionId) -> u64 {
    unimplemented!("final_tally: fetch unit/option final integer tally")
}

/// Pairwise assertion helper: ab vs ba tallies.
#[allow(dead_code)]
fn assert_pair(_pw: &PairwiseMatrix, _a: OptionId, _b: OptionId, _ab: u64, _ba: u64) {
    unimplemented!("assert_pair: check pairwise matrix entry A>B and B>A")
}

/// Assert Result label == "Decisive".
#[allow(dead_code)]
fn assert_decisive(_res: &ResultDb) {
    unimplemented!("assert_decisive: inspect Result.label == Decisive")
}

/// Convenience: map fixture symbol ("A","B","C","D") → OptionId.
#[allow(dead_code)]
fn opt(label: &str) -> OptionId {
    label.to_string()
}

// -----------------------------------------------------------------------------
// Tests (ignored until engine + ranked fixtures are wired)
// -----------------------------------------------------------------------------

/// VM-TST-010 — IRV with exhaustion (reduce_continuing_denominator).
#[test]
#[ignore = "Enable once vm_pipeline + ranked fixtures VM-TST-010 are wired"]
fn run_irv_exhaustion_case() -> Result<()> {
    let (res, _run, _fr) = match run_pipeline(REG_IRV, PS_IRV, TLY_IRV) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            return Ok(()); // keep compiling/runnable; real assertions once wired
        }
    };

    let unit = first_unit_id(&res);

    // Round log expectations:
    //   R1 tallies: A=35, B=40, C=25 → eliminate C
    //   Transfers: 15 → B; 10 exhaust → continuing denominator 90
    //
    // Access via helpers (hide concrete types until wired):
    assert_eq!(final_tally(&res, unit.clone(), opt("B")), 55, "IRV final B");
    assert_eq!(final_tally(&res, unit.clone(), opt("A")), 35, "IRV final A");
    assert_eq!(winner_of(&res, unit.clone()), opt("B"), "IRV winner");

    // When wired, also assert IrvLog round count and exhausted=10 via a helper accessor.

    assert_decisive(&res);
    Ok(())
}

/// VM-TST-011 — Condorcet with Schulze completion (cycle resolved to B).
#[test]
#[ignore = "Enable once vm_pipeline + ranked fixtures VM-TST-011 are wired"]
fn run_condorcet_schulze_cycle() -> Result<()> {
    let (res, _run, _fr) = match run_pipeline(REG_COND, PS_COND, TLY_COND) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            return Ok(());
        }
    };

    let unit = first_unit_id(&res);

    // Pairwise cycle (example target):
    //   A > B : 55–45
    //   B > C : 60–40
    //   C > A : 60–40
    // Schulze strongest paths → winner B.

    // Retrieve the pairwise matrix view from the Result with a helper,
    // then assert entries and winner. We keep the concrete type abstract here.
    let pw: PairwiseMatrix = unimplemented!("obtain pairwise matrix from Result for {unit}");
    assert_pair(&pw, opt("A"), opt("B"), 55, 45);
    assert_pair(&pw, opt("B"), opt("C"), 60, 40);
    assert_pair(&pw, opt("C"), opt("A"), 60, 40);

    assert_eq!(winner_of(&res, unit), opt("B"), "Condorcet-Schulze winner");
    assert_decisive(&res);
    Ok(())
}
