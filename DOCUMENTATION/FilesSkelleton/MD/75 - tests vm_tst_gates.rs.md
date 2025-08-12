<!-- Converted from: 75 - tests vm_tst_gates.rs.docx on 2025-08-12T18:20:47.614378Z -->

```
Lean pre-coding sheet — 75/89
Component: tests/vm_tst_gates.rs (legitimacy gates)
Goal & success
Goal. Verify legitimacy gates and their fixed denominators: quorum, national majority/supermajority, double-majority (national + affected-region family), and symmetry — plus pipeline stop/continue behavior when a gate fails.
Success. For canonical gates fixtures (VM-TST-004/005/006/007), the engine returns the specified Pass/Fail per gate and final label, using the approval-rate denominator when ballot_type = approval.
Scope
In: Gate math & thresholds (VM-VAR-020..027), fixed approval-rate rule, symmetry neutrality, Invalid path semantics.
Out: Ranked method details (covered in vm_tst_ranked.rs), MMP, frontier mapping visuals (unless indirectly referenced).
Inputs → outputs
Inputs. Gates fixtures from Annex B (registries, tallies, parameter sets). Defaults: quorum 50; national majority 55; regional 55; double_majority_enabled = on; symmetry_enabled = on; weighting population_baseline.
Outputs (assert). Gate panel entries (quorum / majority / double-majority / symmetry) and final label; where relevant, exact printed comparisons (e.g., “Support 55.0% vs 55% — Pass”).
Entities/Tables (minimal)
(N/A)
Variables (used here)
VM-VAR-020 quorum_global_pct ∈ % 0..100 (default 50)
VM-VAR-021 quorum_per_unit_pct ∈ % 0..100 (default 0)
VM-VAR-021_scope ∈ {frontier_only, frontier_and_family} (only relevant if 021 > 0)
VM-VAR-022 national_majority_pct ∈ % 50..75 (default 55)
VM-VAR-023 regional_majority_pct ∈ % 50..75 (default 55)
VM-VAR-024 double_majority_enabled ∈ {on, off} (default on)
VM-VAR-025 symmetry_enabled ∈ {on, off} (default on)
VM-VAR-026 affected_region_family_mode ∈ {by_list, by_tag, by_proposed_change}
VM-VAR-027 affected_region_family_ref (list of Unit IDs or a tag; required for by_list/by_tag)
VM-VAR-029 symmetry_exceptions (optional list/tag with rationale)
VM-VAR-007 include_blank_in_denominator ∈ {on, off} (default off)
Fixed rule (not a variable): For approval ballots, legitimacy support % uses approval rate = approvals_for_change / valid_ballots. (There is no gate_denominator_mode variable.)
Functions (test signatures only)
#[test] fn vm_tst_004_supermajority_edge_ge_rule();
#[test] fn vm_tst_005_quorum_failure_invalid();
#[test] fn vm_tst_006_double_majority_family_fail();
#[test] fn vm_tst_007_symmetry_mirrored_pass();
Test logic (bullet outline)
VM-TST-004 (≥ edge): Approval ballots with exactly 55.000% valid approvals for Change; quorum met → Pass, label Decisive; panel prints “Support 55.0% vs 55% — Pass”.
VM-TST-005 (quorum fail): Turnout 48% (Σ ballots_cast / Σ eligible_roll) → Invalid (Quorum failed); omit Frontier.
VM-TST-006 (DM regional fail): National 57% (Pass) but affected-family 53% (<55) → Invalid, reason “Regional threshold not met”; when frontier_mode = none, require affected_region_family_mode ∈ {by_list, by_tag} and non-empty affected_region_family_ref.
VM-TST-007 (symmetry): Mirrored A→B and B→A scenarios at 56% both Pass with identical thresholds/denominators; only labels differ by option names.
State flow (very short)
On gate Fail, run is Invalid, skip MAP_FRONTIER, then label & build outputs. Mirrored runs must share denominator choices.
Determinism & numeric rules
Approval-rate is fixed: approvals_for_change / valid_ballots.
Turnout uses eligible_roll.
Cutoffs use ≥ comparisons.
Stable ordering and canonical JSON elsewhere.
Edge cases & failure policy
If double_majority_enabled = on and frontier_mode = none but affected_region_family_mode = by_proposed_change ⇒ validation error — switch to by_list or by_tag and provide a non-empty reference.
If symmetry exceptions (VM-VAR-029) are present: mark “Symmetry: Not respected” with rationale; tests here expect symmetry to be respected (no exceptions).
Test checklist (must pass)
004: Majority Pass at 55.0% (edge).
005: Invalid due to Quorum failed at 48%.
006: Invalid with regional Fail (min 53%).
007: Both mirrored runs Pass with identical denominator/threshold handling.
```
