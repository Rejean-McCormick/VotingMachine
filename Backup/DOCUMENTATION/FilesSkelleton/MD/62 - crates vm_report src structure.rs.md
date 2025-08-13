<!-- Converted from: 62 - crates vm_report src structure.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.197562Z -->

```
Pre-Coding Essentials (Component: crates/vm_report/src/structure.rs, Version/FormulaID: VM-ENGINE v0) — 63/89
1) Goal & Success
Goal: Define the report data model (pure structs + mappers) that mirrors Doc 7’s section order, bindings, and precision, sourcing data only from Result, optional FrontierMap, and RunRecord.
Success: Building the model from identical artifacts yields identical content; sections appear exactly in Doc 7 order; percentages in the model are already one-decimal formatted for renderers.
2) Scope
In scope: Define ReportModel sub-sections (§1–§10) and mapping helpers from artifacts; embed the approval-denominator statement toggle for approval ballots; carry footer identifiers.
Out of scope: File I/O, template rendering, external assets. (Renderers consume this model.)
3) Inputs → Outputs (with schemas/IDs)
Inputs: Result (RES), optional FrontierMap (FR), RunRecord (RUN); Parameter names indirectly via Result/RunRecord.
Output: ReportModel with sections:
Cover & Snapshot (label; VM-VAR snapshot; registry/date)
Eligibility & Rolls (policy, provenance, totals, per-unit quorum note)
Ballot (method; approval-denominator sentence when applicable)
Legitimacy Panel (quorum/majority/double-majority/symmetry, pass/fail)
Outcome/Label (Decisive/Marginal/Invalid + reason)
Frontier (map/status + diagnostics/counters)
Sensitivity (±1 pp/±5 pp table, or “N/A”)
Integrity & Reproducibility (ID list; footer values)
Fixed footer fields (duplicated for convenience)
4) Entities/Tables (minimal)
5) Variables (only ones used here)
Display only: VM-VAR values printed from ParameterSet snapshot (no computation here). Include approval-rate denominator sentence flag for approval ballots.
6) Functions (signatures only)
rust
CopyEdit
pub struct ReportModel { /* sections per Doc 7A */ }

pub fn model_from_artifacts(result: &ResultDb, run: &RunRecordDb, frontier: Option<&FrontierMapDb>)
-> ReportModel;

fn map_cover_snapshot(..) -> CoverSnapshot;
fn map_eligibility(..) -> EligibilityBlock;
fn map_ballot(..) -> BallotBlock; // sets approval_denominator_sentence = true/false
fn map_panel_from_gates(..) -> LegitimacyPanel;
fn map_outcome_from_result(..) -> OutcomeBlock;
fn map_frontier(..) -> Option<FrontierBlock>;
fn map_sensitivity(..) -> Option<SensitivityBlock>;
fn map_integrity_footer(..) -> (IntegrityBlock, FooterIds);

// helpers: one-decimal formatters (percent, pp), ID/label extractors.

7) Algorithm Outline
Extract identifiers & VM-VAR snapshot for snapshot box.
Eligibility: copy policy VM-VAR-028; provenance; sum turnout; add per-unit quorum note if VM-VAR-021 > 0.
Ballot: choose wording by ballot type; if approval, set approval_denominator_sentence=true.
Panel: bind gate values & pass/fail strictly from Result.gates.
Outcome: copy label + reason from Result.
Frontier: if FR present, map per-Unit status/band and diagnostics; mirror mediation/protected flags.
Sensitivity: include 2×3 table only if scenarios exist; else “N/A (not executed)”.
Integrity & footer: fill identifiers from RunRecord; include Result/Frontier IDs.
Precision: preformat all %/pp to one decimal; seats as integers.
8) State Flow (very short)
Called by vm_report::lib after packaging; reads artifacts only; no external assets.
9) Determinism & Numeric Rules
No recomputation of gate math; no double rounding; all lists keep the same order as artifacts; one-decimal applied once here.
10) Edge Cases & Failure Policy
Validation/gates failed: ensure Frontier is omitted, model carries “Invalid” text blocks accordingly.
Frontier absent: set frontier=None; still build other sections.
Bilingual builds: structure supports full mirrored models; never mix languages in one paragraph.
11) Test Checklist (must pass)
Section order matches Doc 7; all values sourced from artifacts; approval-denominator sentence toggles correctly; footer IDs match RunRecord/Result.
```
