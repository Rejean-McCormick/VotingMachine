````md
Pre-Coding Essentials (Component: tests/vm_tst_mmp.rs, Version/FormulaID: VM-ENGINE v0) — 77/89

1) Goal & Success
Goal: Lock the Mixed-Member Proportional (MMP) sequence and controls: seat targets, top-ups, correction level (national vs regional), and overhang handling.  
Success: For fixture VM-TST-013 the engine yields exactly: **national correction ⇒ A/B/C = 7/3/2**, **regional correction ⇒ 8/2/2**, with Decisive labels and deterministic audit.

2) Scope
In: Run full pipeline with `allocation_method = mixed_local_correction` and assert totals/local seats, not rendering.  
Out: Frontier visuals, RNG ties (not triggered).

3) Inputs → Outputs
Inputs (fixtures): Registry (3 equal-pop regions; 6 SMDs), BallotTally (locals + approvals/list votes), two ParameterSets differing only in VM-VAR-016 (correction level).  
Outputs (asserted): `total_seats_by_party`, `local_seats_by_party`, label Decisive; (optional) effective total seats == 12.

4) Fixture Paths (constants to adjust to repo layout)
```rust
const REG:     &str = "fixtures/annex_b/part_3/vm_tst_013/division_registry.json";
const TLY:     &str = "fixtures/annex_b/part_3/vm_tst_013/ballots.json";
const PS_NAT:  &str = "fixtures/annex_b/part_3/vm_tst_013/parameter_set_national.json";
const PS_REG:  &str = "fixtures/annex_b/part_3/vm_tst_013/parameter_set_regional.json";
````

5. Variables (used here)

* VM-VAR-010 `allocation_method = mixed_local_correction`
* VM-VAR-013 `mlc_topup_share_pct` (implies T≈L/(1−s)) — fixture picks T=12, L=6, TopUps=6
* VM-VAR-015 `target_share_basis = natural_vote_share`
* VM-VAR-016 `mlc_correction_level ∈ {national, regional}`  ⟵ toggled between tests
* VM-VAR-017 `total_seats_model` (fixed in fixture; no growth needed)
* Overhang policy: default “allow” (locals never removed)

6. Test functions (signatures only)

```rust
#[test] fn vm_tst_013_mmp_national_level();
#[test] fn vm_tst_013_mmp_regional_level();
```

7. Algorithm Outline (assertions)

* Locals first: 6 SMDs → **A=2, B=2, C=2**.
* Intended total seats **T=12** (TopUp pool = 6).
* National correction: apportion targets over country totals with chosen method (e.g., Sainte-Laguë), compute deficits, assign top-ups ⇒ **A/B/C = 7/3/2**.
* Regional correction: apportion per region (2 top-ups each), then sum ⇒ **A/B/C = 8/2/2**.
* Labels: **Decisive** for both; no RNG used.

8. State Flow
   `LOAD → VALIDATE → TABULATE → ALLOCATE (locals) → AGGREGATE → MMP correct (targets, deficits, top-ups) → APPLY_DECISION_RULES → LABEL → BUILD_RESULT → BUILD_RUN_RECORD`.

9. Determinism & Numeric Rules
   Integer/rational math only; half-even only where specified (total-from-share).
   Stable option order; no randomness (tie situations not constructed in this fixture).
   Locals are immutable; overhang allowed (not triggered here).

10. Edge Cases & Failure Policy
    If `total_seats_model = add_total_seats` were set and targets < locals for some option, minimal expansion clears overhang; not exercised here.
    If votes sum to zero (not this fixture), targets/top-ups are zero except locals.

11. Test Checklist (must pass)

* **National**: `total_seats_by_party == {A:7, B:3, C:2}`, `local_seats_by_party == {A:2, B:2, C:2}`, label Decisive, (optional) effective\_total\_seats = 12.
* **Regional**: `total_seats_by_party == {A:8, B:2, C:2}`, `local_seats_by_party == {A:2, B:2, C:2}`, label Decisive, (optional) effective\_total\_seats = 12.
* Determinism: same outputs across OS/arch.

12. Rust test skeleton (drop-in; fill helpers or import from shared test utils)

```rust
use anyhow::Result;
use std::collections::BTreeMap;

// --- expected helpers/types (adapt to your crate paths) ---
type ResultDb = vm_core::result::ResultDb;
type RunRecordDb = vm_core::result::RunRecordDb;
type FrontierMapDb = vm_core::result::FrontierMapDb;
type OptionId = vm_core::ids::OptionId;

fn run_pipeline(reg:&str, ps:&str, tly:&str)
 -> Result<(ResultDb, RunRecordDb, Option<FrontierMapDb>)> { /* … wire CLI/library … */ }

fn totals_by_option(res:&ResultDb) -> BTreeMap<OptionId,u32> { /* read aggregated final seats */ }
fn locals_by_option(res:&ResultDb) -> BTreeMap<OptionId,u32> { /* read aggregated local seats */ }
fn effective_total_seats(res:&ResultDb) -> u32 { /* read from aggregates or MMP outcome */ }
fn oid(label:&str) -> OptionId { /* map "A"/"B"/"C" to OptionId from fixtures */ }
fn assert_decisive(res:&ResultDb) { /* label == "Decisive" */ }

// --- Tests ---
#[test]
fn vm_tst_013_mmp_national_level() -> Result<()> {
    let (res, _run, _fr) = run_pipeline(REG, PS_NAT, TLY)?;
    let a = oid("A"); let b = oid("B"); let c = oid("C");

    let total = totals_by_option(&res);
    let local = locals_by_option(&res);

    assert_eq!(local.get(&a).copied(), Some(2));
    assert_eq!(local.get(&b).copied(), Some(2));
    assert_eq!(local.get(&c).copied(), Some(2));

    assert_eq!(total.get(&a).copied(), Some(7));
    assert_eq!(total.get(&b).copied(), Some(3));
    assert_eq!(total.get(&c).copied(), Some(2));

    assert_eq!(effective_total_seats(&res), 12);
    assert_decisive(&res);
    Ok(())
}

#[test]
fn vm_tst_013_mmp_regional_level() -> Result<()> {
    let (res, _run, _fr) = run_pipeline(REG, PS_REG, TLY)?;
    let a = oid("A"); let b = oid("B"); let c = oid("C");

    let total = totals_by_option(&res);
    let local = locals_by_option(&res);

    assert_eq!(local.get(&a).copied(), Some(2));
    assert_eq!(local.get(&b).copied(), Some(2));
    assert_eq!(local.get(&c).copied(), Some(2));

    assert_eq!(total.get(&a).copied(), Some(8));
    assert_eq!(total.get(&b).copied(), Some(2));
    assert_eq!(total.get(&c).copied(), Some(2));

    assert_eq!(effective_total_seats(&res), 12);
    assert_decisive(&res);
    Ok(())
}
```

```
```
