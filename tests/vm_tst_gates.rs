//! VM-ENGINE v0 — Gates & Denominators tests (skeleton)
//!
//! This file encodes the **test intentions and signatures** for legitimacy gates:
//! Quorum → Majority/Supermajority → Double-majority → Symmetry, including the
//! fixed **approval-rate** denominator rule (approvals_for_change / valid_ballots)
//! and the pipeline stop/continue semantics when a gate fails.
//!
//! Until the full engine wiring + fixtures exist in the workspace, these tests
//! are marked `#[ignore]`. Replace the `unimplemented!()`/`todo!()` stubs with
//! real orchestration once `vm_pipeline`/`vm_io` and Annex-B gates fixtures are
//! present (see fixture constants below).

use std::fmt;

// -----------------------------------------------------------------------------
// Fixture paths (adjust to your repo layout if needed)
// -----------------------------------------------------------------------------
#[allow(dead_code)]
const REG_004: &str = "fixtures/annex_b/gates/s004/division_registry.json";
#[allow(dead_code)]
const PS_004: &str = "fixtures/annex_b/gates/s004/parameter_set.json";
#[allow(dead_code)]
const TLY_004: &str = "fixtures/annex_b/gates/s004/ballots.json";

#[allow(dead_code)]
const REG_005: &str = "fixtures/annex_b/gates/s005/division_registry.json";
#[allow(dead_code)]
const PS_005: &str = "fixtures/annex_b/gates/s005/parameter_set.json";
#[allow(dead_code)]
const TLY_005: &str = "fixtures/annex_b/gates/s005/ballots.json";

#[allow(dead_code)]
const REG_006: &str = "fixtures/annex_b/gates/s006/division_registry.json";
#[allow(dead_code)]
const PS_006: &str = "fixtures/annex_b/gates/s006/parameter_set.json";
#[allow(dead_code)]
const TLY_006: &str = "fixtures/annex_b/gates/s006/ballots.json";

#[allow(dead_code)]
const REG_007A: &str = "fixtures/annex_b/gates/s007_a/division_registry.json"; // A→B
#[allow(dead_code)]
const PS_007A: &str = "fixtures/annex_b/gates/s007_a/parameter_set.json";
#[allow(dead_code)]
const TLY_007A: &str = "fixtures/annex_b/gates/s007_a/ballots.json";

#[allow(dead_code)]
const REG_007B: &str = "fixtures/annex_b/gates/s007_b/division_registry.json"; // B→A
#[allow(dead_code)]
const PS_007B: &str = "fixtures/annex_b/gates/s007_b/parameter_set.json";
#[allow(dead_code)]
const TLY_007B: &str = "fixtures/annex_b/gates/s007_b/ballots.json";

// -----------------------------------------------------------------------------
// Minimal gate view used by assertions (engine-agnostic placeholder)
// -----------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, Default)]
struct GateRowView {
    pass: bool,
    numerator: u64,
    denominator: u64,
    threshold_pct: u8,
}

#[derive(Clone, Copy, Debug, Default)]
struct DoubleMajorityView {
    national_pass: bool,
    family_pass: bool,
    national_num: u64,
    national_den: u64,
    family_num: u64,
    family_den: u64,
    national_threshold_pct: u8,
    family_threshold_pct: u8,
}

#[derive(Clone, Debug, Default)]
struct GatePanelView {
    quorum: GateRowView,
    majority: GateRowView,
    double_majority: Option<DoubleMajorityView>,
    symmetry: Option<bool>,
}

impl fmt::Display for GateRowView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Pure integer one-decimal rendering for diagnostics only
        // (tests compare structured values; this is just a helper string).
        let tenths = if self.denominator == 0 {
            0i128
        } else {
            ((self.numerator as i128) * 1000) / (self.denominator as i128)
        };
        write!(
            f,
            "{}.{:01}% vs {}% — {}",
            tenths / 10,
            (tenths % 10).abs(),
            self.threshold_pct,
            if self.pass { "Pass" } else { "Fail" }
        )
    }
}

// -----------------------------------------------------------------------------
// Public test API expectations (stubs to be implemented when wiring the engine)
// -----------------------------------------------------------------------------

/// Run full pipeline from explicit file paths; returns (Result, RunRecord, Frontier?).
#[allow(dead_code)]
fn run_pipeline(
    _reg: &str,
    _ps: &str,
    _tly: &str,
) -> Result<((), (), Option<()>), String> {
    // Replace this with a real invocation that loads the three local files,
    // runs the fixed pipeline, and returns concrete artifacts.
    Err("run_pipeline is not wired yet".into())
}

/// Extract gate panel with raw numerators/denominators and pass/fail flags.
#[allow(dead_code)]
fn gate_panel(_res: &()) -> GatePanelView {
    // Map engine LegitimacyReport → GatePanelView (deterministic).
    unimplemented!("gate_panel stub: map Result.gates to GatePanelView")
}

/// Assert majority text “Support X.X% vs Y% — Pass/Fail”; X formatted to one decimal.
#[allow(dead_code)]
fn assert_majority_line(view: &GatePanelView, expect_pct_1dp: &str, threshold_pct: u32, expect_pass: bool) {
    // Build a one-decimal string from majority row for human-friendly assert.
    let m = view.majority;
    let tenths = if m.denominator == 0 {
        0i128
    } else {
        ((m.numerator as i128) * 1000) / (m.denominator as i128)
    };
    let observed_str = format!("{}.{:01}", tenths / 10, (tenths % 10).abs());
    assert_eq!(
        observed_str,
        expect_pct_1dp,
        "majority observed percent (1dp) mismatch"
    );
    assert_eq!(
        m.threshold_pct as u32,
        threshold_pct,
        "majority threshold pct mismatch"
    );
    assert_eq!(m.pass, expect_pass, "majority pass/fail mismatch");
}

/// Assert pipeline skipped Frontier when gates failed.
#[allow(dead_code)]
fn assert_frontier_omitted(fr: &Option<()>) {
    assert!(
        fr.is_none(),
        "Frontier must be omitted when a gate fails (MAP_FRONTIER skipped)"
    );
}

/// Assert final label == "Decisive" | "Invalid" with reason substring.
#[allow(dead_code)]
fn assert_label(_res: &(), _expect: &str, _reason_contains: Option<&str>) {
    // Extract {label, reason} from Result and compare.
    unimplemented!("assert_label stub: inspect Result.label and reason");
}

// -----------------------------------------------------------------------------
// Tests (ignored until engine + fixtures are available)
// -----------------------------------------------------------------------------

#[test]
#[ignore = "Enable once vm_pipeline + gates fixtures are wired"]
fn vm_tst_004_supermajority_edge_ge_rule() {
    // 55.000% approvals_for_change / valid_ballots at the national level must PASS (≥ rule).
    let (res, _run, fr) = match run_pipeline(REG_004, PS_004, TLY_004) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("run_pipeline stub: {e}");
            return; // early return while ignored/wiring pending
        }
    };
    let panel = gate_panel(&res);
    assert_majority_line(&panel, "55.0", 55, true);
    assert_label(&res, "Decisive", None);
    // No assertion on frontier for a pass-case.
    let _ = fr; // silence unused for now
}

#[test]
#[ignore = "Enable once vm_pipeline + gates fixtures are wired"]
fn vm_tst_005_quorum_failure_invalid() {
    // Turnout Σ ballots_cast / Σ eligible_roll = 48% → Quorum FAIL → run Invalid; frontier omitted.
    let (res, _run, fr) = match run_pipeline(REG_005, PS_005, TLY_005) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("run_pipeline stub: {e}");
            return;
        }
    };
    let panel = gate_panel(&res);
    assert!(
        panel.quorum.pass == false,
        "expected quorum fail, got: {}",
        panel.quorum
    );
    assert_label(&res, "Invalid", Some("Quorum"));
    assert_frontier_omitted(&fr);
}

#[test]
#[ignore = "Enable once vm_pipeline + gates fixtures are wired"]
fn vm_tst_006_double_majority_family_fail() {
    // National ≥ threshold but affected-family support below regional threshold ⇒ DM FAIL ⇒ Invalid.
    let (res, _run, fr) = match run_pipeline(REG_006, PS_006, TLY_006) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("run_pipeline stub: {e}");
            return;
        }
    };
    let panel = gate_panel(&res);
    let dm = panel
        .double_majority
        .expect("double-majority should be present in this scenario");
    assert!(panel.majority.pass, "national majority should pass");
    assert!(
        dm.family_pass == false,
        "family (regional) support should fail DM threshold"
    );
    assert_label(&res, "Invalid", Some("Regional"));
    assert_frontier_omitted(&fr);
}

#[test]
#[ignore = "Enable once vm_pipeline + gates fixtures are wired"]
fn vm_tst_007_symmetry_mirrored_pass() {
    // A→B and B→A mirrored runs at 56% both Pass; denominators/thresholds neutral.
    let (res_a, _run_a, _fr_a) = match run_pipeline(REG_007A, PS_007A, TLY_007A) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("run_pipeline stub A: {e}");
            return;
        }
    };
    let (res_b, _run_b, _fr_b) = match run_pipeline(REG_007B, PS_007B, TLY_007B) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("run_pipeline stub B: {e}");
            return;
        }
    };
    let a = gate_panel(&res_a);
    let b = gate_panel(&res_b);
    assert!(a.majority.pass && b.majority.pass, "both should pass at 56%");
    // Optionally assert that observed/threshold tuples are equal across mirrored runs.
    assert_eq!(
        (a.majority.numerator, a.majority.denominator, a.majority.threshold_pct),
        (b.majority.numerator, b.majority.denominator, b.majority.threshold_pct),
        "symmetry neutrality: denominators/thresholds must match between mirrored runs"
    );
}
