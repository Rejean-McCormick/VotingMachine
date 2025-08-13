```toml
Pre-Coding Essentials (Component: crates/vm_pipeline/src/lib.rs, Version FormulaID VM-ENGINE v0) — 49/89

1) Goal & Success
Goal: Expose the pipeline API that runs the fixed, normative state machine and emits canonical artifacts: Result (RES:…), RunRecord (RUN:…), optional FrontierMap (FR:…).
Success: Stage order and stop/continue semantics exactly match Doc-5; inputs are {registry + ballot_tally + parameter_set} only; outputs’ fields/IDs/naming align with adjusted schemas (#15–#20); determinism (ordering/rounding/RNG) follows vm_core; canonical JSON + hashes via vm_io; Annex-B test pack validates cleanly.

2) Scope
In scope: Public entry points; wiring for LOAD → VALIDATE → TABULATE → ALLOCATE → AGGREGATE → APPLY_RULES → (FRONTIER?) → (TIES?) → LABEL → BUILD_RESULT → BUILD_RUN_RECORD; unified error surface.
Out of scope: JSON/schema code (vm_io), algorithm math (vm_algo), UI/report rendering.

3) Inputs → Outputs (normative)
Inputs (offline, local files): DivisionRegistry (REG:…), BallotTally (TLY:…), ParameterSet (PS:…). No raw ballots path in normative runs.
Outputs:
• Result (RES:…) — includes formula_id; no input IDs; shares as JSON numbers; no tie_log.  
• RunRecord (RUN:…) — includes engine.vendor/name/version/build, formula_id, normative manifest digest, input IDs + 64-hex digests, tie_policy and rng_seed iff tie_policy = random, optional FR:… pointers.  
• FrontierMap (FR:…) — optional; only when frontier_mode ≠ none and gates pass.

4) Public Types (signatures only)
use vm_core::{
  ids::*, entities::*, variables::Params,
};
use vm_io::{
  manifest::Manifest,
  loader::{LoadedContext},   // registry + params + tally; options/units normalized
};
use vm_algo as algo;

pub struct PipelineCtx {
  pub loaded: LoadedContext,           // from vm_io::loader (registry/params/tally)
  pub engine_meta: EngineMeta,         // vendor/name/version/build
  pub nm_canonical: serde_json::Value, // Normative Manifest (for FID)
}

pub struct PipelineOutputs {
  pub result: ResultDoc,               // typed mirror of schemas/result.schema.json
  pub run_record: RunRecordDoc,        // typed mirror of schemas/run_record.schema.json
  pub frontier_map: Option<FrontierMapDoc>,
}

#[derive(Debug)]
pub enum PipelineError {
  Io(String),
  Schema(String),
  Validate(String),
  Tabulate(String),
  Allocate(String),
  Gates(String),
  Frontier(String),
  Tie(String),
  Build(String),
}

5) Variables (usage here)
Reads VM-VARs from Params only; no new variables introduced. Tie controls:
• VM-VAR-050 tie_policy ∈ {status_quo, deterministic, random} (Included in FID)  
• VM-VAR-052 tie_seed ∈ integer ≥0 (Excluded from FID) — used only when tie_policy = random

6) Public API (signatures only)
/// Run the pipeline using a pre-loaded context (normative inputs only).
pub fn run_with_ctx(ctx: PipelineCtx) -> Result<PipelineOutputs, PipelineError>;

/// Convenience: parse+validate manifest, load artifacts, then run.
pub fn run_from_manifest_path<P: AsRef<std::path::Path>>(path: P) -> Result<PipelineOutputs, PipelineError>;

/// Engine identifiers (for RunRecord + manifest “expect” checks).
pub fn engine_identifiers() -> EngineMeta;                 // {vendor,name,version,build}
pub fn compute_formula_id(nm: &serde_json::Value) -> String; // sha256 over NM per vm_io::hasher

7) Algorithm Outline (fixed stage order)
LOAD
  • vm_io::manifest::load/resolve → loader::load_* → LoadedContext (registry, params, ballot_tally). Reject raw ballots (non-normative).
VALIDATE (structural & referential)
  • IDs parse; unit/option refs in tally exist; reg_id matches; option order_index uniqueness; magnitude ≥1; bounds integer-only.
TABULATE
  • Per-unit tallies → UnitScores (plurality/approval/score/ranked per ballot_type).
ALLOCATE
  • Per-unit allocation: WTA/PR family per Params; WTA requires magnitude=1.
AGGREGATE
  • By levels expected in Doc-1B; produce totals and compute shares (JSON numbers for Result).
APPLY_DECISION_RULES (gates)
  • Quorum / majority / double-majority / symmetry; integers/ratios only; approval gate uses approval rate over valid ballots.
MAP_FRONTIER (optional)
  • If gates pass and frontier_mode ≠ none: compute statuses/flags; contiguity via allowed edge types.
RESOLVE_TIES (only if blocking)
  • If tie blocks allocation outcomes, apply tie_policy (VM-VAR-050): status_quo → SQ; deterministic → option.order_index then OptionId; random → vm_core::rng with tie_seed (VM-VAR-052).
LABEL_DECISIVENESS
  • Decisive | Marginal | Invalid per spec and flags; reasons captured.
BUILD_RESULT
  • ResultDoc with: id (RES:…), formula_id, label(+reason), gates panel, per-unit blocks, aggregates with shares as numbers; no reg_id/TLY/PS IDs; no tie_log.
BUILD_RUN_RECORD
  • RunRecordDoc with: id (RUN:…), timestamp_utc (Z), engine {vendor,name,version,build}, formula_id, normative manifest digest, inputs {IDs + 64-hex digests}, policy {tie_policy, rng_seed iff random}, platform, outputs {RES: (+sha256), optional FR:(+sha256)}, optional tie summary.

8) State Flow
Stops early on VALIDATE failure → still produce Result with label="Invalid" and RunRecord capturing reasons; skip frontier. Gates fail ⇒ label="Invalid"; FrontierMap omitted.

9) Determinism & Numeric Rules
• Ordering: Units by UnitId; Options by (order_index, OptionId).  
• Integer/rational math only; half-even only where mandated (e.g., MMP totals).  
• RNG used only if tie_policy = random; seeded by VM-VAR-052; outcome identical for same seed and inputs.  
• Canonical JSON bytes & SHA-256 via vm_io::canonical_json/hasher; shares emitted as JSON numbers in Result (engine precision).

10) Edge Cases & Failure Policy
• Manifest lacking ballot_tally_path ⇒ error (normative runs require tallies).  
• Any *_path URL ⇒ error (offline only).  
• WTA with magnitude≠1 ⇒ error.  
• Missing rng_seed while tie_policy=random ⇒ error; run aborts before BUILD_RESULT.  
• Frontier_mode="none" ⇒ no FrontierMap.  
• All-zero tallies: gates compute false; label becomes Invalid per rules.

11) Test Checklist (must pass)
• Stage order and stop/continue semantics match Doc-5 precisely.  
• Result fields: includes formula_id; shares are numbers; no input IDs; no tie_log.  
• RunRecord fields: includes vendor/name/version/build; formula_id; normative manifest digest; canonical 64-hex digests for inputs; rng_seed only when random.  
• Determinism: same inputs + same seed ⇒ identical canonical bytes (Result/RunRecord/FrontierMap).  
• Annex-B tallies validate and round-trip; option arrays order matches registry order_index; gates thresholds compare via integers/half-even where specified.
```
