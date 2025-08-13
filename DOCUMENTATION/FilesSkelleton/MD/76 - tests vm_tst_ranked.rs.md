````md
Pre-Coding Essentials (Component: tests/vm_tst_ranked.rs, Version/FormulaID: VM-ENGINE v0) — 76/89

1) Goal & Success
Goal: Lock ranked-method behavior: (a) IRV with exhaustion (reduce_continuing_denominator), (b) Condorcet with Schulze completion.  
Success: Winners, round/pairwise audit payloads, and labels match Annex B Part 3 expectations; byte-stable across OS/arch.

2) Scope
In: Full-pipeline tests over ranked fixtures VM-TST-010 (IRV) / VM-TST-011 (Condorcet-Schulze).  
Out: Frontier/MMP/RNG ties (not triggered here).

3) Inputs → Outputs
Inputs: DivisionRegistry (+Options order), BallotTally (ranked), ParameterSet (ballot_type + ranked knobs).  
Outputs (asserted): Result winner per unit, IRV IrvLog {rounds, exhausted}, Condorcet Pairwise matrix + winner; final label Decisive.

4) Fixture Paths (constants)
```rust
const REG_IRV: &str = "fixtures/annex_b/part_3/vm_tst_010/division_registry.json";
const PS_IRV:  &str = "fixtures/annex_b/part_3/vm_tst_010/parameter_set.json";
const TLY_IRV: &str = "fixtures/annex_b/part_3/vm_tst_010/ballots.json";

const REG_COND: &str = "fixtures/annex_b/part_3/vm_tst_011/division_registry.json";
const PS_COND:  &str = "fixtures/annex_b/part_3/vm_tst_011/parameter_set.json";
const TLY_COND: &str = "fixtures/annex_b/part_3/vm_tst_011/ballots.json";
````

5. Variables (used here)

* VM-VAR-001 `ballot_type ∈ {ranked_irv, ranked_condorcet}`
* VM-VAR-005 `condorcet_completion = schulze`
* VM-VAR-006 `irv_exhaustion_policy = reduce_continuing_denominator`
  (Other VM-VARs default; gates not under test here.)

6. Test functions (signatures only)

```rust
#[test] fn run_irv_exhaustion_case();        // VM-TST-010
#[test] fn run_condorcet_schulze_cycle();    // VM-TST-011
```

7. Algorithm Outline (assertions)
   IRV (VM-TST-010)

* Input profile (grouped ballots): 40×B>A>C, 35×A>C, 15×C>B, 10×C.
* Round 1 tallies: A=35, B=40, C=25 → eliminate C.
* Transfers: 15 → B; 10 exhaust (no next continuing).
* Continuing denominator shrinks: 100 → 90.
* Final tallies: B=55, A=35 → winner B.
* Assert IrvLog: `rounds.len() == 1`, `rounds[0].eliminated = C`, `rounds[0].exhausted = 10`.

Condorcet (VM-TST-011, Schulze)

* Pairwise wins (totals): A>B 55–45, B>C 60–40, C>A 60–40 (cycle).
* Schulze strongest paths select winner B.
* Assert Pairwise matrix entries and final winner B.

8. State Flow
   LOAD → VALIDATE → TABULATE (ranked) → (ALLOCATE if needed by exec model) → AGGREGATE → APPLY\_DECISION\_RULES (pass) → LABEL (Decisive) → BUILD\_RESULT/RUN\_RECORD.

9. Determinism & Numeric Rules

* Integer counts only; no RNG; stable order (options by order\_index, id).
* IRV majority over **continuing** ballots; exhausted ballots reduce denominator.
* Condorcet completion = Schulze; deterministic tie-breaks by canonical order only if method requires.

10. Edge Cases & Failure Policy

* Malformed rankings/rounds → MethodConfigError (not in these happy-path tests).
* Zero-valid unit would yield zeros; not part of these fixtures.

11. Test Checklist (must pass)

* IRV: winner B; exhausted=10; continuing=90; IrvLog round exactly as specified; label Decisive.
* Condorcet-Schulze: pairwise entries match; winner B; label Decisive.

12. Rust file skeleton (ready to fill)

```rust
use anyhow::Result;

#[test]
fn run_irv_exhaustion_case() -> Result<()> {
    let (res, _run, _fr) = run_pipeline(REG_IRV, PS_IRV, TLY_IRV)?;
    let unit = first_unit_id(&res);
    let irv = res.units[&unit].irv_log.as_ref().expect("IRV log missing");
    assert_eq!(irv.rounds.len(), 1);
    let r1 = &irv.rounds[0];
    assert_eq!(r1.exhausted, 10);
    assert_eq!(winner_of(&res, unit), opt("B"));
    assert_eq!(final_tally(&res, unit, opt("B")), 55);
    assert_eq!(final_tally(&res, unit, opt("A")), 35);
    assert_decisive(&res);
    Ok(())
}

#[test]
fn run_condorcet_schulze_cycle() -> Result<()> {
    let (res, _run, _fr) = run_pipeline(REG_COND, PS_COND, TLY_COND)?;
    let unit = first_unit_id(&res);
    let pw = res.units[&unit].pairwise.as_ref().expect("Pairwise missing");
    assert_pair(pw, opt("A"), opt("B"), 55, 45);
    assert_pair(pw, opt("B"), opt("C"), 60, 40);
    assert_pair(pw, opt("C"), opt("A"), 60, 40);
    assert_eq!(winner_of(&res, unit), opt("B"));
    assert_decisive(&res);
    Ok(())
}

// ---- helper stubs (implement or import from a shared test util) ----
fn run_pipeline(reg:&str, ps:&str, tly:&str)
 -> Result<(ResultDb, RunRecordDb, Option<FrontierMapDb>)> { /* … */ }
fn first_unit_id(res:&ResultDb) -> UnitId { /* … */ }
fn winner_of(res:&ResultDb, unit:UnitId) -> OptionId { /* … */ }
fn final_tally(res:&ResultDb, unit:UnitId, opt:OptionId) -> u64 { /* … */ }
fn opt(label:&str) -> OptionId { /* map fixture label→OptionId */ }
fn assert_pair(pw:&PairwiseMatrix, a:OptionId, b:OptionId, ab:u64, ba:u64) { /* … */ }
fn assert_decisive(res:&ResultDb) { /* label == "Decisive" */ }
```

```
```
