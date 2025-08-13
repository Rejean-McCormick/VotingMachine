````md
Pre-Coding Essentials (Component: tests/vm_tst_gates.rs, Version/FormulaID: VM-ENGINE v0) — 75/89

1) Goal & Success
Goal: Verify legitimacy gates and their **fixed denominators**: quorum → national majority/supermajority → double-majority (national + affected family) → symmetry; plus pipeline stop/continue when a gate fails.
Success: Canonical gates fixtures (VM-TST-004/005/006/007) yield the specified Pass/Fail and final label, using **approval rate = approvals_for_change / valid_ballots** when ballot_type=approval. Results are identical across OS/arch.

2) Scope
In: Gate math & thresholds (VM-VAR-020..027), approval-rate rule, symmetry neutrality, Invalid path semantics.  
Out: Ranked tabulation internals, MMP, frontier visuals (only indirectly via “frontier skipped on gate fail”).

3) Inputs → Outputs
Inputs: Annex-B gates fixtures (registries, tallies, parameter sets). Defaults commonly: quorum=50, national_majority=55, regional_majority=55, `double_majority_enabled=on`, `symmetry_enabled=on`, weighting=population_baseline.  
Outputs (assert): Gate panel (quorum / majority / double-majority / symmetry), final label, and for edge cases the exact comparison string (e.g., `Support 55.0% vs 55% — Pass`).

4) Fixture Paths (constants)
```rust
const REG_004: &str = "fixtures/annex_b/gates/s004/division_registry.json";
const PS_004:  &str = "fixtures/annex_b/gates/s004/parameter_set.json";
const TLY_004: &str = "fixtures/annex_b/gates/s004/ballots.json";

const REG_005: &str = "fixtures/annex_b/gates/s005/division_registry.json";
const PS_005:  &str = "fixtures/annex_b/gates/s005/parameter_set.json";
const TLY_005: &str = "fixtures/annex_b/gates/s005/ballots.json";

const REG_006: &str = "fixtures/annex_b/gates/s006/division_registry.json";
const PS_006:  &str = "fixtures/annex_b/gates/s006/parameter_set.json";
const TLY_006: &str = "fixtures/annex_b/gates/s006/ballots.json";

const REG_007A: &str = "fixtures/annex_b/gates/s007_a/division_registry.json"; // A→B
const PS_007A:  &str = "fixtures/annex_b/gates/s007_a/parameter_set.json";
const TLY_007A: &str = "fixtures/annex_b/gates/s007_a/ballots.json";
const REG_007B: &str = "fixtures/annex_b/gates/s007_b/division_registry.json"; // B→A
const PS_007B:  &str = "fixtures/annex_b/gates/s007_b/parameter_set.json";
const TLY_007B: &str = "fixtures/annex_b/gates/s007_b/ballots.json";
````

*(If your repo keeps all gates under Part-0, map these constants to the appropriate Part-0 subfolders.)*

5. Variables (used here)

* VM-VAR-020 `quorum_global_pct` (int % 0..100)
* VM-VAR-021 `quorum_per_unit_pct` + `VM-VAR-021_scope`
* VM-VAR-022 `national_majority_pct` (50..75), VM-VAR-023 `regional_majority_pct` (50..75)
* VM-VAR-024 `double_majority_enabled`, VM-VAR-025 `symmetry_enabled`
* VM-VAR-026 `affected_region_family_mode`, VM-VAR-027 `affected_region_family_ref`
* VM-VAR-029 `symmetry_exceptions` (not used in these tests; must be empty)
* VM-VAR-007 `include_blank_in_denominator` (gates-only toggle)
  **Fixed rule (not a VM-VAR):** approval ballots use **approval rate = approvals\_for\_change / valid\_ballots** for legitimacy support.

6. Test functions (signatures only)

```rust
#[test] fn vm_tst_004_supermajority_edge_ge_rule();     // 55.000% edge → Pass (≥ rule), Decisive
#[test] fn vm_tst_005_quorum_failure_invalid();         // turnout 48% → Invalid; Frontier omitted
#[test] fn vm_tst_006_double_majority_family_fail();    // national pass, family 53% → Invalid; DM regional fail
#[test] fn vm_tst_007_symmetry_mirrored_pass();         // A→B & B→A at 56% → both Pass; neutral denominators
```

7. Algorithm Outline (per test)

* **004 (≥ edge, approval)**
  Arrange approval ballots with *exactly* 55.000% approvals\_for\_change / valid\_ballots; quorum met.
  Assert: Majority panel **Pass** (≥), label **Decisive**, printed compare string includes `Support 55.0% vs 55% — Pass`.

* **005 (Quorum fail)**
  Arrange Σ ballots\_cast / Σ eligible\_roll = 48%.
  Assert: Quorum **Fail**, run **Invalid**, Frontier **omitted** (skip MAP\_FRONTIER). Majority not evaluated for label.

* **006 (Double-majority family fail)**
  Arrange national support 57% (Pass) but affected-family support 53% (<55).
  Preconditions: `double_majority_enabled=on`; `frontier_mode=none`; `affected_region_family_mode ∈ {by_list, by_tag}`; `affected_region_family_ref` non-empty.
  Assert: DM **Fail** (regional), run **Invalid**, reason mentions regional threshold.

* **007 (Symmetry)**
  Two runs: A→B and B→A with identical support 56%.
  Assert: Both **Pass** with identical thresholds & denominators; only option names differ. No symmetry exceptions.

8. Determinism & Numeric Rules

* Integer/rational math; cutoffs use **≥**.
* Approval gate uses **approval rate** (never approvals share).
* Turnout uses **eligible\_roll**.
* Stable ordering; canonical JSON elsewhere. No RNG used.

9. Edge Cases & Failure Policy

* If DM enabled & frontier\_mode=none with `affected_region_family_mode=by_proposed_change` ⇒ **validation error** (fix fixture to by\_list/by\_tag with non-empty ref).
* If symmetry exceptions present, symmetry not respected (not expected in these tests).
* If per-unit quorum is set, ensure the scope is honored in family computations (already covered by pipeline gate step; not directly asserted here).

10. Helper API (pure; no net)

```rust
/// Run full pipeline from explicit file paths; returns (Result, RunRecord, Frontier?).
fn run_pipeline(reg:&str, ps:&str, tly:&str)
 -> anyhow::Result<(ResultDb, RunRecordDb, Option<FrontierMapDb>)>;

/// Extract gate panel with raw numerators/denominators and pass/fail flags.
fn gate_panel(res:&ResultDb) -> GatePanelView; // {quorum, majority, double_majority?, symmetry?}

/// Assert majority text “Support X% vs Y% — Pass/Fail”; X formatted to one decimal.
fn assert_majority_line(view:&GatePanelView, expect_pct_str:&str, threshold_pct:u32, expect_pass:bool);

/// Assert pipeline skipped Frontier when gates failed.
fn assert_frontier_omitted(fr:&Option<FrontierMapDb>);

/// Assert final label == "Decisive" | "Invalid" with reason substring.
fn assert_label(res:&ResultDb, expect:&str, reason_contains:Option<&str>);
```

11. Test Checklist (must pass)

* **004:** Majority Pass at edge 55.0%; label **Decisive**; comparison string exact.
* **005:** **Invalid** due to Quorum failed at 48%; Frontier omitted.
* **006:** **Invalid** with DM regional Fail (family 53%); preconditions satisfied (by\_list/by\_tag & non-empty ref).
* **007:** Symmetry respected in both mirrored runs (56%); both **Pass** with identical denominator/threshold handling.

12. Rust file skeleton (ready to fill)

```rust
use anyhow::Result;

#[test]
fn vm_tst_004_supermajority_edge_ge_rule() -> Result<()> {
    let (res, _run, fr) = run_pipeline(REG_004, PS_004, TLY_004)?;
    let panel = gate_panel(&res);
    assert_majority_line(&panel, "55.0", 55, true);
    assert_label(&res, "Decisive", None);
    Ok(())
}

#[test]
fn vm_tst_005_quorum_failure_invalid() -> Result<()> {
    let (res, _run, fr) = run_pipeline(REG_005, PS_005, TLY_005)?;
    let panel = gate_panel(&res);
    assert!(panel.quorum.pass == false, "expected quorum fail");
    assert_label(&res, "Invalid", Some("Quorum"));
    assert_frontier_omitted(&fr);
    Ok(())
}

#[test]
fn vm_tst_006_double_majority_family_fail() -> Result<()> {
    let (res, _run, fr) = run_pipeline(REG_006, PS_006, TLY_006)?;
    let panel = gate_panel(&res);
    assert!(panel.majority.pass, "national should pass");
    assert!(panel.double_majority.as_ref().unwrap().family_pass == false, "family should fail");
    assert_label(&res, "Invalid", Some("Regional threshold"));
    assert_frontier_omitted(&fr);
    Ok(())
}

#[test]
fn vm_tst_007_symmetry_mirrored_pass() -> Result<()> {
    let (res_a, _run_a, _fr_a) = run_pipeline(REG_007A, PS_007A, TLY_007A)?;
    let (res_b, _run_b, _fr_b) = run_pipeline(REG_007B, PS_007B, TLY_007B)?;
    let a = gate_panel(&res_a);
    let b = gate_panel(&res_b);
    assert!(a.majority.pass && b.majority.pass, "both should pass at 56%");
    // Optional: compare denominator/threshold tuples for equality.
    Ok(())
}

// ---- helper stubs to implement in this test module or a shared test util ----
// fn run_pipeline(..) -> Result<(ResultDb, RunRecordDb, Option<FrontierMapDb>)> { /* … */ }
// fn gate_panel(res:&ResultDb) -> GatePanelView { /* … */ }
// fn assert_majority_line(..) { /* … */ }
// fn assert_frontier_omitted(fr:&Option<FrontierMapDb>) { /* … */ }
// fn assert_label(res:&ResultDb, expect:&str, reason_contains:Option<&str>) { /* … */ }
```

```
```
