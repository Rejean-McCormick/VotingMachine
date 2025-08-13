<!-- Converted from: 49 - crates vm_pipeline src lib.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.841893Z -->

```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/lib.rs, Version/FormulaID: VM-ENGINE v0) — 49/89
Goal & Success
Goal: Expose the pipeline entry points that orchestrate the fixed state machine and return canonical artifacts (Result, RunRecord, optional FrontierMap).
Success: Stage order and stop/continue semantics match Doc 5; outputs’ fields/IDs match Doc 1; determinism/ordering/RNG behaviors match Doc 3.
Scope
In scope: Public pipeline API; wiring of LOAD→…→BUILD_RUN_RECORD; error surface unification; re-exports of stage structs (LoadedContext, UnitScores, etc.).
Out of scope: Low-level I/O/JSON canonicalization (lives in vm_io), algorithm math (in vm_algo), report rendering (Doc 7).
Inputs → Outputs (with schemas/IDs)
Inputs: IDs/paths for DivisionRegistry, BallotTally, ParameterSet; optional Adjacency; all local (offline).
Outputs:
Result (DB entity, RES:…) and RunRecord (RUN:…); optional FrontierMap (FR:…).
Stage artifacts are assembled exactly per Doc 5 §3.
Entities/Tables (minimal)
(N/A — this module wires stage structs defined elsewhere.)
Variables (only ones used here)
Reads the ParameterSet snapshot and passes VM-VAR values to stages; lib itself declares no new variables.
Note: tie resolution uses VM-VAR-032 tie_policy and VM-VAR-033 tie_seed in the tie stage (no VM-VAR-051).
Functions (signatures only)
/// High-level: run the full pipeline using already-loaded blobs/IDs.
pub fn run_with_ctx(ctx: PipelineCtx) -> Result<PipelineOutputs, PipelineError>;

/// Convenience: parse and verify a manifest (vm_io), then run.
pub fn run_from_manifest(manifest: &Manifest) -> Result<PipelineOutputs, PipelineError>;

/// Accessors for versioning/FormulaID echoes used in RunRecord.
pub fn engine_identifiers() -> (FormulaId, EngineVersion);
(Types mirror Doc 5 artifacts: LoadedContext, UnitScores, UnitAllocation, AggregateResults, LegitimacyReport, FrontierMap, TieLog, Result, RunRecord.)
Algorithm Outline (bullet steps)
LOAD → LoadedContext.
VALIDATE (fail ⇒ mark Invalid, skip 3–8).
TABULATE → UnitScores.
ALLOCATE → UnitAllocation.
AGGREGATE → AggregateResults.
APPLY_DECISION_RULES → LegitimacyReport (Fail ⇒ skip frontier).
MAP_FRONTIER (optional) → FrontierMap.
RESOLVE_TIES (only if blocking; uses VM-VAR-032/033) → TieLog.
LABEL_DECISIVENESS → {label, reason}.
BUILD_RESULT → Result.
BUILD_RUN_RECORD → RunRecord.
State Flow (very short)
Exactly Doc 5 order above; stop/continue semantics enforced (Invalid path, “skip frontier” rule, ties only when blocking).
Determinism & Numeric Rules
Stable ordering (Units by ID; Options by order_index then ID).
Integer/rational math; half-even only at defined decision points.
RNG only if tie_policy = random, seeded by VM-VAR-033 tie_seed; seed is recorded.
Edge Cases & Failure Policy
Any validation failure ⇒ label Invalid but still build Result & RunRecord with reasons.
Gate failure ⇒ Invalid; FrontierMap omitted by design.
If a blocking tie occurs and tie_policy/tie_seed are inconsistent or missing ⇒ surface MethodConfigError / TieUnresolvedError.
Test Checklist (must pass)
Stage order & stop/continue semantics match Doc 5.
Produced Result contains required fields/flags; RunRecord includes FormulaID/EngineVersion and tie_seed (if used).
Determinism: same inputs + same seed ⇒ identical Result/RunRecord bytes (canonical JSON).
```
