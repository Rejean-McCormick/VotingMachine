<!-- Converted from: 57 - label.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.046935Z -->

```
Pre-Coding Essentials (Component: label.rs, Version/FormulaID: VM-ENGINE v0)
1) Goal & Success
Assign the final DecisivenessLabel (Decisive|Marginal|Invalid) with a concise reason, using gates outcome, national margin vs threshold, and frontier flags.
Success: identical label/reason for identical inputs across OS/arch; rules match Doc 4C.
2) Scope
In: consume LegitimacyReport, national margin (pp), optional FrontierMap flags; use VM-VAR-062.
Out: ephemeral DecisivenessLabel {label, reason} to BuildResult.
3) Inputs → Outputs (with schemas/IDs)
Inputs (ephemeral/artifacts): LegitimacyReport (gate pass/fail), AggregateResults.national_margin_pp, FrontierMap.flags{mediation,enclave,protected_override}.
Outputs: DecisivenessLabel {label, reason}. Used by BUILD_RESULT.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
pub fn label_decisiveness(legit: &LegitimacyReport, national_margin_pp: i32, frontier_flags: Option<FrontierFlags>) -> DecisivenessLabel — Applies rules below; deterministic.
7) Algorithm Outline (bullet steps)
If any gate failed or validation failed earlier ⇒ Invalid with reason from gates.
Else, if national_margin_pp < VM-VAR-062 or any mediation|enclave|protected_override flag present ⇒ Marginal with specific reason.
Else ⇒ Decisive with margin reason.
Emit {label, reason}; ready for report.
8) State Flow (very short)
Stage: LABEL_DECISIVENESS after (optional) RESOLVE_TIES, before BUILD_RESULT.
Never halts pipeline; always produces a label.
9) Determinism & Numeric Rules
Ordering keys: N/A (single decision).
Numbers: use integer pp for margin; no float presentation here. Report handles one-decimal formatting.
Tie policy: N/A at this stage (ties resolved earlier).
10) Edge Cases & Failure Policy
Exact threshold hit counts as Pass upstream; if margin == VM-VAR-062 ⇒ Decisive (since only < triggers Marginal).
If FrontierMap absent (because gates failed) ⇒ step 1 applies (Invalid).
If frontier is on and any mediation/enclave/protected override flag exists ⇒ Marginal.
11) Test Checklist (must pass)
Doc 6 note: labels follow Doc 4C (Invalid if any gate fails; Marginal if margin < VM-VAR-062 or frontier flags).
Frontier mediation forces Marginal (e.g., VM-TST-014).
Result carries label & reason; report displays it verbatim.
```
