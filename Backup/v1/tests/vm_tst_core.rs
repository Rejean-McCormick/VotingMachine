//! VM-ENGINE v0 — Core pipeline tests (skeleton)
//!
//! This file wires the **test intentions and signatures** for baseline behaviors
//! across tabulation, allocation, gate denominators, and pipeline stop rules,
//! aligned with Docs 4–7 and Annex-B Part-0 fixtures.
//!
//! Notes:
//! - These tests are provided as an integration **skeleton**. They are marked
//!   `#[ignore]` to avoid failing CI until the full engine wiring and fixtures
//!   are present in the workspace.
//! - Replace the `unimplemented!()` / `todo!()` calls with real orchestration
//!   once `vm_pipeline`/`vm_io` are available and the Annex-B fixtures (69–73)
//!   are checked into the repo at `fixtures/annex_b/part_0/`.

use std::collections::BTreeMap;

// --- Intentional imports (kept behind allow to avoid warnings until wired) ---
#[allow(unused_imports)]
use vm_core::ids::{OptionId, UnitId};
#[allow(unused_imports)]
use vm_core::variables::Params;
#[allow(unused_imports)]
use vm_core::rounding::ge_percent;
#[allow(unused_imports)]
use vm_pipeline::{
    // Public API per component 49
    run_from_manifest_path,
    PipelineOutputs,
};
#[allow(unused_imports)]
use vm_report as _; // report crate is not used directly here

// -------- Test harness types (minimal mirror of intentions) --------

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum TestMode {
    /// Drive via manifest: fixtures/annex_b/part_0/manifest.json
    Manifest,
    /// Drive via explicit paths (registry/params/tally)
    Explicit,
}

#[derive(Clone, Debug)]
struct TestArtifacts {
    // Keep this lean; real engine types can be added as needed.
    pub outputs: PipelineOutputs,
}

// Snapshots used by asserts (kept simple and engine-agnostic)
#[derive(Clone, Debug)]
struct GateSnapshot {
    pub quorum_pass: bool,
    pub majority_present: bool,
    pub majority_pass: bool,
    pub majority_threshold_pct: u8,
    pub majority_num: u64,
    pub majority_den: u64,
}

#[allow(dead_code)]
fn run_with_part0_fixtures(_mode: TestMode) -> TestArtifacts {
    // Resolve manifest path relative to workspace root
    let _man_path = std::path::Path::new("fixtures/annex_b/part_0/manifest.json");

    // When the pipeline is available, uncomment:
    // let outputs = run_from_manifest_path(_man_path)
    //     .expect("pipeline run from manifest should succeed for Part-0");
    // TestArtifacts { outputs }

    unimplemented!("Wire vm_pipeline::run_from_manifest_path and Part-0 fixtures");
}

#[allow(dead_code)]
fn seats_of(_alloc: &BTreeMap<OptionId, u32>) -> Vec<(OptionId, u32)> {
    // In the real engine, UnitAllocation exposes seats (PR) or power=100 (WTA).
    // This helper converts the internal map into a stable (OptionId, seats) vec.
    unimplemented!("Extract seats per option in stable order");
}

#[allow(dead_code)]
fn power_of(_alloc: &BTreeMap<OptionId, u32>) -> Vec<(OptionId, u32)> {
    // For WTA, return { winner → 100 }.
    unimplemented!("Extract WTA 100% power vector");
}

#[allow(dead_code)]
fn gate_values(_legit: &/* vm_pipeline::LegitimacyReport */ ()) -> GateSnapshot {
    // Pull raw numerators/denominators and pass flags from LegitimacyReport.
    // Majority must use approval_rate = approvals_for_change / valid_ballots.
    unimplemented!("Map LegitimacyReport → GateSnapshot");
}

#[allow(dead_code)]
fn label_of(_res: &/* vm_pipeline::ResultDoc */ ()) -> /* vm_pipeline::DecisivenessLabel */ () {
    unimplemented!("Extract decisiveness label from Result");
}

#[allow(dead_code)]
fn assert_sum_seats(seats: &[(OptionId, u32)], m: u32) {
    let sum: u32 = seats.iter().map(|(_, s)| *s).sum();
    assert_eq!(
        sum, m,
        "Σ seats must equal magnitude (expected {m}, got {sum})"
    );
}

#[allow(dead_code)]
fn assert_wta_power_100(power: &[(OptionId, u32)]) {
    let sum: u32 = power.iter().map(|(_, p)| *p).sum();
    assert_eq!(sum, 100, "WTA must sum to 100% power (got {sum})");
}

#[allow(dead_code)]
fn assert_ge_majority(value_pp1: i32, threshold: i32) {
    // value in tenths of a percent points vs integer threshold (pp)
    assert!(
        value_pp1 >= threshold * 10,
        "majority must use ≥ comparison: value {value_pp1}/10 < {threshold}"
    );
}

// --------------------- Core tests (ignored until wired) ----------------------

#[test]
#[ignore = "Enable once vm_pipeline + fixtures are wired"]
fn vm_tst_001_pr_baseline_sainte_lague() {
    // Arrange: approvals A/B/C/D=10/20/30/40; m=10; SL; threshold 0%
    let _art = run_with_part0_fixtures(TestMode::Manifest);
    // Act: extract unit allocation seats
    // let seats = seats_of(...);
    // Assert: 1/2/3/4 in canonical option order; Σ seats == 10
    // assert_eq!(seats, vec![(opt_a,1),(opt_b,2),(opt_c,3),(opt_d,4)]);
    // assert_sum_seats(&seats, 10);
    todo!("Plug real extraction and assertions");
}

#[test]
#[ignore = "Enable once vm_pipeline + fixtures are wired"]
fn vm_tst_002_wta_winner_take_all_m1() {
    // Arrange: plurality votes with D top; m=1; WTA
    let _art = run_with_part0_fixtures(TestMode::Manifest);
    // Act: extract WTA power vector
    // let power = power_of(...);
    // Assert: D → 100% power
    // assert_eq!(power, vec![(opt_d, 100)]);
    // assert_wta_power_100(&power);
    todo!("Plug real extraction and assertions");
}

#[test]
#[ignore = "Enable once vm_pipeline + fixtures are wired"]
fn vm_tst_003_method_convergence_lr_vs_ha() {
    // Arrange: shares 34/33/33; m=7; run SL, D’Hondt, LR
    let _art = run_with_part0_fixtures(TestMode::Manifest);
    // Act: seat vectors for each method
    // Assert: each returns 3/2/2; Σ seats == 7 for each
    todo!("Run three allocation methods on same tallies, assert 3/2/2");
}

#[test]
#[ignore = "Enable once vm_pipeline + fixtures are wired"]
fn vm_tst_004_gate_denominator_approval_rate() {
    // Arrange: approval ballot, Change approvals / valid = 55.0%
    let _art = run_with_part0_fixtures(TestMode::Manifest);
    // Act: take LegitimacyReport → GateSnapshot
    // let g = gate_values(&art.outputs.result.gates);
    // Assert: majority uses approvals_for_change / valid_ballots; pass at ≥ 55
    // assert!(g.majority_present && g.majority_pass);
    // assert_eq!(g.majority_threshold_pct, 55);
    // assert_eq!((g.majority_num, g.majority_den), (55, 100));
    todo!("Extract majority gate; assert approval-rate denominator and ≥ rule");
}

#[test]
#[ignore = "Enable once vm_pipeline + fixtures are wired"]
fn vm_tst_005_pipeline_order_and_stop_rules() {
    // Arrange A: craft a validation fail (e.g., hierarchy violation) → expect Invalid; skip TABULATE..FRONTIER
    // Arrange B: a quorum-below-threshold case → expect Invalid; frontier skipped
    // For A and B, ensure Result and RunRecord are still produced.
    todo!("Run two scenarios and assert stop/continue semantics per Doc-5");
}
