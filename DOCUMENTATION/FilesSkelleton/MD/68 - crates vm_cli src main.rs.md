

# Pre-Coding Essentials — 68/89

**Component:** `crates/vm_cli/src/main.rs`
**Formula/Engine:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Drive the fixed pipeline end-to-end (LOAD → VALIDATE → TABULATE → ALLOCATE → AGGREGATE → APPLY\_DECISION\_RULES → MAP\_FRONTIER → RESOLVE\_TIES → LABEL\_DECISIVENESS → BUILD\_RESULT → BUILD\_RUN\_RECORD), write canonical artifacts, and render reports — strictly offline and deterministic.
* **Success:** Identical inputs (+ same seed when used) ⇒ byte-identical **Result**, **RunRecord**, and optional **FrontierMap** across OS/arch; reports show one-decimal percentages; no network or OS RNG.

## 2) Scope

* **In scope:** Parse CLI args; call `vm_pipeline` stages in fixed order; enforce stop/continue semantics; write canonical JSON artifacts; invoke `vm_report` renderers.
* **Out of scope:** Algorithm math (tabulation/allocation/gates/frontier); schema definitions; any network I/O.

## 3) Inputs → Outputs (with schemas/IDs)

**Inputs (local only):**

* DivisionRegistry, BallotTally **or** Ballots (or a Manifest pointing to them), ParameterSet; optional Adjacency; render flags; optional RNG seed.

**Outputs (canonical JSON: UTF-8, LF, sorted keys, UTC):**

* `Result` (`RES:<hash>`), `RunRecord` (`RUN:<utc>-<hash>`), optional `FrontierMap` (`FR:<hash>`).
* Optional report(s): JSON/HTML (presentation only, one-decimal).

## 4) Entities (minimal)

* From CLI: `Args` (already validated in 66).
* From pipeline: `LoadedContext`, `ValidationReport`, `UnitScores`, `UnitAllocation`, `AggregateResults`, `LegitimacyReport`, `FrontierMap`, `TieContext/TieLog`, `DecisivenessLabel`, `ResultDb`, `RunRecordDb`.

## 5) Variables (used here)

* None new. Respect snapshot from `ParameterSet`; RNG policy uses tie variables (policy/seed) already validated upstream.

## 6) Functions (signatures only; no code)

```rust
use std::{path::Path, process::ExitCode};
use vm_cli::args::Args;

// Entry
fn main() -> anyhow::Result<()>;

// Orchestration
fn run(args: Args) -> anyhow::Result<ExitCode>;

// Stage adapters (thin wrappers over vm_pipeline)
fn load_inputs(args: &Args) -> anyhow::Result<LoadedContext>;                // LOAD
fn validate(ctx: &LoadedContext) -> anyhow::Result<ValidationReport>;        // VALIDATE
fn tabulate(ctx: &LoadedContext) -> anyhow::Result<UnitScoresByUnit>;        // TABULATE
fn allocate(ctx: &LoadedContext, t: &UnitScoresByUnit) -> anyhow::Result<UnitAllocationByUnit>; // ALLOCATE
fn aggregate(ctx: &LoadedContext, a: &UnitAllocationByUnit) -> anyhow::Result<AggregateResults>;// AGGREGATE
fn apply_rules(ctx: &LoadedContext, agg: &AggregateResults) -> anyhow::Result<LegitimacyReport>;// APPLY_DECISION_RULES
fn map_frontier_if_enabled(
  ctx: &LoadedContext,
  agg: &AggregateResults,
  gates: &LegitimacyReport
) -> anyhow::Result<Option<FrontierMap>>;                                     // MAP_FRONTIER (optional)
fn resolve_ties_if_blocking(ctx: &LoadedContext, pending: &[TieContext]) -> anyhow::Result<TieLog>; // RESOLVE_TIES
fn label_decisiveness(
  gates: &LegitimacyReport,
  agg: &AggregateResults,
  frontier_flags: Option<&FrontierFlags>
) -> anyhow::Result<DecisivenessLabel>;                                       // LABEL_DECISIVENESS
fn build_result(
  ctx: &LoadedContext,
  agg: &AggregateResults,
  gates: &LegitimacyReport,
  label: &DecisivenessLabel,
  ties: &TieLog,
  frontier_id: Option<FrontierId>
) -> anyhow::Result<ResultDb>;                                                // BUILD_RESULT
fn build_run_record(
  ctx: &LoadedContext,
  result_id: ResultId,
  frontier_id: Option<FrontierId>,
  started_utc: &str,
  finished_utc: &str
) -> anyhow::Result<RunRecordDb>;                                             // BUILD_RUN_RECORD

// Artifact IO & reporting
fn write_artifacts(
  out_dir: &Path,
  result: &ResultDb,
  run: &RunRecordDb,
  frontier: Option<&FrontierMap>
) -> std::io::Result<()>;
fn render_reports(
  out_dir: &Path,
  result: &ResultDb,
  run: &RunRecordDb,
  frontier: Option<&FrontierMap>,
  formats: &[ReportFormat]  // e.g., ["json","html"]
) -> anyhow::Result<()>;

// Exit policy
fn choose_exit_code(label: &DecisivenessLabel, gates_failed: bool) -> ExitCode;
```

## 7) Algorithm Outline (orchestration)

1. **Parse args** (already validated by 66).
2. **LOAD** inputs → `LoadedContext`. On error: CLI error (no artifacts).
3. **VALIDATE**: if `pass=false`

   * Mark run **Invalid**; **skip** TABULATE…MAP\_FRONTIER.
   * Still **LABEL**, **BUILD\_RESULT**, **BUILD\_RUN\_RECORD**, write artifacts, render invalid-path report.
4. If valid: **TABULATE → ALLOCATE → AGGREGATE**.
5. **APPLY\_DECISION\_RULES**: if any gate **Fail** ⇒ **Invalid**, **skip MAP\_FRONTIER**; continue to ties only if blocking; then label/build/write/render.
6. If gates **Pass** and frontier enabled/data present: **MAP\_FRONTIER** (never invalidates; flags may imply **Marginal** later).
7. **RESOLVE\_TIES** only for blocking contexts (WTA winner, last seat, IRV elimination); if `tie_policy=random`, use provided seed; log in `TieLog`.
8. **LABEL\_DECISIVENESS** using gates outcome, national margin, and frontier flags.
9. **BUILD\_RESULT** → **BUILD\_RUN\_RECORD** (caller provides UTC strings; no system clock reads).
10. **Write artifacts** (canonical JSON) and **render reports** (one-decimal).
11. **Exit** per policy below.

## 8) State Flow (short)

`args → LOAD → VALIDATE → (TABULATE → ALLOCATE → AGGREGATE) → APPLY_RULES → [MAP_FRONTIER?] → [RESOLVE_TIES?] → LABEL → BUILD_RESULT → BUILD_RUN_RECORD → write → render → exit`.

## 9) Determinism & Numeric Rules

* Offline only; no network/telemetry; no OS RNG.
* Stable iteration (Units by ID; Options by `(order_index,id)`); integer/rational math inside pipeline.
* Canonical JSON for artifacts: UTF-8, **LF**, **sorted keys**, **UTC**; SHA-256 hashing.
* RNG only when `tie_policy=random`, seeded; seed echoed in **RunRecord**.

## 10) Edge Cases & Failure Policy

* **Validation fail:** still emit **Invalid** Result/RunRecord; omit Frontier; render fallback.
* **Gate fail:** **Invalid**, skip Frontier; render panel with ❌ and reason.
* **Frontier inputs absent:** frontier step skipped without invalidating the run.
* **Report requested without artifacts:** treat as CLI usage error (pre-pipeline).
* **Any unrecoverable I/O/config error before artifacts:** exit with CLI error; do not write partial outputs.

## 11) Exit Codes (policy)

* `0` — **Decisive** or **Marginal** (artifacts & requested reports written).
* `2` — **Invalid** due to **validation fail**.
* `3` — **Invalid** due to **gate fail**.
* `1` — CLI/config/I/O error **before** any artifacts were produced.

## 12) Test Checklist (must pass)

* **Determinism:** identical Result/RunRecord hashes on repeat & across OS (see determinism tests).
* **Pipeline order & stops:** matches Doc 5; invalid and gate-fail paths honored.
* **Reporting:** JSON/HTML include one-decimal percentages; approval ballots include the mandatory approval-denominator sentence; integrity footer echoes engine/FID/IDs/seed.
* **Artifacts:** canonical JSON ends with single LF; sorted keys; UTC timestamps; IDs match computed hashes.
