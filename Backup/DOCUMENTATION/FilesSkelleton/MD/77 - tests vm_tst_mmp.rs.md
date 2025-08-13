<!-- Converted from: 77 - tests vm_tst_mmp.rs.docx on 2025-08-12T18:20:47.651802Z -->

```
Lean pre-coding sheet — 77/89
Component: tests/vm_tst_mmp.rs (Mixed Local + Correction — MMP)
1) Goal & success
Goal: Prove the MMP sequence and controls: top-up share, target basis, correction level (national vs regional), overhang handling, and total-seats model. The canonical case is VM-TST-013 with two assertions: national correction ⇒ A/B/C = 7/3/2; regional correction ⇒ 8/2/2.
Success: Engine returns the exact seat vectors and Decisive labels; audit shows deficit-driven top-up sequence and deterministic tie handling.
2) Scope
In: allocation_method = mixed_local_correction; locals first, then top-ups per deficits; compare mlc_correction_level = national vs regional; keep locals under overhang policy.
Out: Gates/frontier visuals; RNG ties (not triggered by this fixture). General proportional/WTA already covered elsewhere.
3) Inputs → outputs
Inputs (fixtures): Three equal-pop regions; 6 local SMDs (A,A / B,B / C,C) and approval tallies for vote shares; two ParameterSets identical except VM-VAR-016 (national vs regional).
Outputs (asserted): Result.total_seats_by_party at correction level; local_seats_by_party; final label.
4) Entities/Tables (minimal)
5) Variables (used here)
6) Functions (test signatures only)
rust
CopyEdit
#[test] fn vm_tst_013_mmp_national_level();
#[test] fn vm_tst_013_mmp_regional_level();

Asserts exact seat vectors and Decisive labels for both correction levels.
7) Test logic (bullet outline)
Setup: Load VM-TST-013 bundle (REG/Options/Tallies/PS). Locals: A=2, B=2, C=2. National shares ≈ A 56.7%, B 21.7%, C 21.7%. T=12, TopUp=6.
National correction: compute targets on T=12; iteratively assign top-ups to largest positive deficit (tie: higher share → deterministic order). Expect A/B/C = 7/3/2.
Regional correction: targets per region (2 top-ups each), then totals A/B/C = 8/2/2.
8) State flow (very short)
Pipeline: ALLOCATE locals → compute targets (per VM-VAR-016) → assign top-ups from pool → aggregate totals → label. Locals are never taken away; overhang allowed by default.
9) Determinism & numeric rules
Use exact integers; compare deficits deterministically: largest deficit → higher vote share → deterministic order; only then RNG if policy allows (not used here). Stable Unit/Option orders, canonical JSON.
10) Edge cases & failure policy
If total_seats_model = variable_add_seats with add_total_seats: seats may grow to clear remaining deficits; record final T. (Not used in this test; keep branch covered by a separate case if added later.)
If locals already exceed targets (overhang), do not remove them; remaining top-ups prefer non-overhung parties when policy ≠ default.
11) Test checklist (must pass)
National: total_seats_by_party = {A:7,B:3,C:2}; label Decisive.
Regional: {A:8,B:2,C:2}; label Decisive.
Allocation audit shows deficit-driven sequence consistent with Doc 4B; deterministic tie rule applied where deficits equal.
```
