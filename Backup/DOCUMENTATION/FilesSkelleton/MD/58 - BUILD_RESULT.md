<!-- Converted from: 58 - BUILD_RESULT, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.075828Z -->

```
Pre-Coding Essentials (Component: BUILD_RESULT, Version/FormulaID: VM-ENGINE v0)
1) Goal & Success
Goal: Compose the Result artifact from prior pipeline outputs (allocations, aggregates, gates, ties, label), ready for canonical serialization and hashing.
Success: Fields match DB spec (top-level IDs; per-unit blocks; gates; TieLog; Label; optional frontier_map_id); deterministic ordering; integer/rational values copied without re-computation.
2) Scope
In scope: Merge LegitimacyReport, DecisivenessLabel, TieLog, AggregateResults, UnitScores/UnitAllocation into Result; attach input IDs (REG/TLY/PS) and optional frontier_map_id.
Out of scope: I/O & hashing (vm_io), RunRecord persistence (next step).
3) Inputs → Outputs (with schemas/IDs)
Inputs:
reg_id, ballot_tally_id, parameter_set_id (from ctx).
AggregateResults (totals/shares/turnout/weighting).
LegitimacyReport (gate values & pass/fail).
DecisivenessLabel.
TieLog (may be empty).
FrontierMap pointer if produced.
Output:
Result (RES:…) with: top-level input IDs; per-unit blocks (tabulation, allocation, turnout, flags); aggregates; gates; TieLog; Label; optional frontier_map_id.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
None computed here. Values already decided upstream; we copy gate results, label, and flags. (Gate semantics reference VM-VAR-020/022/023/025/026/027, but not recalculated here.)
6) Functions (signatures only)
fn build_result(ctx: &PipelineCtx, agg: &AggregateResults, gates: &LegitimacyReport, label: &DecisivenessLabel, ties: &[TieEvent], frontier_id: Option<FrontierId>) -> Result<Result>
 Purpose: Compose Result. Ordering/determinism enforced.
Helpers:
fn write_unit_blocks(..) -> Vec<UnitBlock> (IDs/order canonical).
fn attach_gate_panels(result: &mut Result, gates: &LegitimacyReport)
fn attach_ties_and_label(result: &mut Result, ties: &[TieEvent], label: &DecisivenessLabel)
7) Algorithm Outline (bullet steps)
Initialize Result with input IDs (REG/TLY/PS).
Emit UnitBlocks from unit-level tabulation/alloc data; set flags (unit_data_ok, unit_quorum_met, unit_pr_threshold_met, protected_override_used, mediation_flagged).
Attach Aggregates and weighting.
Attach gates (values + pass/fail + denominators).
Append TieLog; append Label.
If present, set frontier_map_id.
Return Result (ready for canonical JSON + hashing downstream).
8) State Flow (very short)
Previous steps: LABEL_DECISIVENESS → BUILD_RESULT → BUILD_RUN_RECORD.
Stop/continue: Always build a Result; if VALIDATE failed upstream, label is Invalid and gates/frontier may be absent. (Packaging still occurs.)
9) Determinism & Numeric Rules
Ordering: Units by Unit ID; Options by Option.order_index then ID.
Values: Use exact integers; ratios copied from gates; no float recomputation.
TieLog: policy/seed/order recorded verbatim.
10) Edge Cases & Failure Policy
Missing per-unit tallies for a unit: emit unit_data_ok=false; keep allocations/gates if present; totals may be zeroed. (Still a valid Result.) Spec allows packaging even after validation fail.
No FrontierMap produced: frontier_map_id omitted. Reports read only available artifacts.
Ensure seats sum to Unit.magnitude (PR) or 100% power (WTA).
11) Test Checklist (must pass)
Top-level IDs set; unit blocks present and ordered; aggregates match inputs; gates copied with denominators; TieLog & Label preserved; optional frontier pointer set. (Report depends on these fields.)
```
