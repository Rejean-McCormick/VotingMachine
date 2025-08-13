Here’s the clean, no-code skeleton for **File 67 — `vm_cli/src/main.rs`**, aligned with the pipeline refs (48–56) and the CLI surface (65–66).

# Pre-Coding Essentials — 67/89

**Component:** `vm_cli/src/main.rs`
**Formula/Engine:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Orchestrate the fixed pipeline (LOAD → VALIDATE → TABULATE → ALLOCATE → AGGREGATE → APPLY\_DECISION\_RULES → MAP\_FRONTIER → RESOLVE\_TIES → LABEL\_DECISIVENESS → BUILD\_RESULT → BUILD\_RUN\_RECORD), write canonical artifacts, and render reports — all offline and deterministic.
* **Success:** Same inputs (+ seed if used) ⇒ byte-identical **Result**, **RunRecord**, and optional **FrontierMap** across OS/arch; reports show one-decimal percentages; no network or OS RNG.

## 2) Scope

* **In scope:** Parse CLI args; call stage APIs from `vm_pipeline`; canonical JSON write; optional JSON/HTML rendering via `vm_report`; exit code policy.
* **Out of scope:** Algorithm math (tabulation/allocation/gates/frontier); schema definitions; UI; any network I/O.

## 3) Inputs → Outputs

**Inputs (local only):** paths to DivisionRegistry, BallotTally/Ballots (or Manifest), ParameterSet, optional Adjacency; render flags; optional RNG seed.
**Outputs (canonical JSON, UTF-8, LF, sorted keys, UTC):**

* `result.json` (`RES:<hash>`)
* `run_record.json` (`RUN:<utc>-<hash>`)
* Optional `frontier_map.json` (`FR:<hash>`)
* Optional report(s): JSON/HTML (presentation only, one-decimal)

## 4) Entities (minimal)

* From `vm_cli::args`: `Args` (validated flags & paths)
* From `vm_pipeline`: `PipelineCtx`, `PipelineOutputs` (or discrete artifacts)
* Artifacts: `Result`, `RunRecord`, `FrontierMap` (optional)

## 5) Variables (used here)

* No new VM-VARs. Respect snapshot from `ParameterSet` and RNG tie settings (`tie_policy`, `tie_seed`) already validated upstream.

## 6) Functions (signatures only; no code)

```rust
use std::{path::Path, process::ExitCode};
use vm_cli::args::Args;

// entry
fn main() -> ExitCode;

// orchestration
fn run(args: Args) -> Result<RunSummary, CliError>;

// pipeline stepping (thin wrappers around vm_pipeline)
fn load_inputs(args: &Args) -> Result<LoadedContext, CliError>;
fn validate(ctx: &LoadedContext) -> ValidationReport;
fn tabulate(ctx: &LoadedContext) -> TabulateOut;
fn allocate(t: &TabulateOut, ctx: &LoadedContext) -> AllocateOut;
fn aggregate(a: &AllocateOut, ctx: &LoadedContext) -> AggregateResults;
fn apply_rules(agg: &AggregateResults, ctx: &LoadedContext) -> LegitimacyReport;
fn map_frontier_if_applicable(
  agg: &AggregateResults,
  ctx: &LoadedContext,
  gates: &LegitimacyReport
) -> Option<FrontierMap>;
fn resolve_ties_if_blocking(ctxs: &[TieContext], params: &Params) -> TieLog;
fn label_decisiveness(
  gates: &LegitimacyReport,
  agg: &AggregateResults,
  frontier_flags: Option<&FrontierFlags>
) -> DecisivenessLabel;
fn build_result(
  ctx: &LoadedContext,
  agg: &AggregateResults,
  gates: &LegitimacyReport,
  label: &DecisivenessLabel,
  ties: &TieLog,
  frontier_id: Option<FrontierId>
) -> ResultDb;
fn build_run_record(
  ctx: &LoadedContext,
  result_id: ResultId,
  frontier_id: Option<FrontierId>,
  started_utc: &str,
  finished_utc: &str
) -> RunRecordDb;

// artifacts & reports
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
  formats: &[ReportFormat] // e.g., ["json","html"]
) -> Result<(), CliError>;

// policy
fn choose_exit_code(label: DecisivenessLabel, gates_failed: bool) -> ExitCode;
```

## 7) Algorithm Outline (orchestration)

1. **Parse & validate args** (from `args.rs`). Halt early on format/shape errors (no I/O beyond existence checks there).
2. **LOAD** inputs → `LoadedContext`. On I/O/parse error: CLI error exit (no artifacts).
3. **VALIDATE** → if `pass=false`:

   * Mark run **Invalid**; **skip** TABULATE … MAP\_FRONTIER.
   * Still **LABEL**, **BUILD\_RESULT**, **BUILD\_RUN\_RECORD**, write artifacts, render invalid-path report.
4. If valid: **TABULATE → ALLOCATE → AGGREGATE** in order.
5. **APPLY\_DECISION\_RULES**:

   * If any gate **Fail**: mark **Invalid**, **skip MAP\_FRONTIER**; continue to ties only if blocking; then label/build/write/render.
6. If gates **Pass** and frontier feature enabled/data present: **MAP\_FRONTIER** (never invalidates; may later cause **Marginal** via flags).
7. **RESOLVE\_TIES** only when blocking (WTA winner, last seat, IRV elimination). If `tie_policy=random`, use provided seed; log in `TieLog`.
8. **LABEL\_DECISIVENESS** using gates outcome, national margin (from aggregates), and frontier flags.
9. **BUILD\_RESULT** then **BUILD\_RUN\_RECORD** (UTCs provided by caller; no system clock reads here).
10. **Write artifacts** in canonical JSON (UTF-8, LF, sorted keys, UTC). **Render reports** per requested formats.
11. **Exit** with policy below.

## 8) State Flow (short)

`args → LOAD → VALIDATE → (TABULATE → ALLOCATE → AGGREGATE) → APPLY_RULES → [MAP_FRONTIER?] → [RESOLVE_TIES?] → LABEL → BUILD_RESULT → BUILD_RUN_RECORD → write → render → exit`.

## 9) Determinism & Numeric Rules

* **Offline only**; no network; no OS RNG.
* Stable iteration (Units by ID; Options by `(order_index, id)`); integer/rational math is inside pipeline.
* Canonical JSON: UTF-8, **LF**, **sorted keys**, **UTC** timestamps; hashes via SHA-256 (downstream I/O layer).
* RNG only if `tie_policy=random`, seeded; seed echoed in **RunRecord**.

## 10) Edge Cases & Failure Policy

* **Validation fail:** still produce **Invalid** Result/RunRecord; omit Frontier; render fallback text.
* **Gate fail:** **Invalid**, skip Frontier; render panel with ❌ and reason.
* **Missing frontier inputs:** frontier step skipped without invalidating the run.
* **Report render selected without artifacts:** treat as CLI usage error (before pipeline).
* **Any nonrecoverable I/O/config error before artifacts:** exit with CLI error code (no partial writes).

## 11) Exit Codes (policy)

* `0` — **Decisive** or **Marginal** (artifacts & reports written).
* `2` — **Invalid** due to **validation fail**.
* `3` — **Invalid** due to **gate fail**.
* `1` — CLI/config/I/O error **before** any artifacts were produced.

## 12) Test Checklist (must pass)

* **Determinism:** repeat & cross-OS runs produce identical Result/RunRecord hashes (see determinism tests).
* **Order & stops:** stage order and stop/continue semantics match Doc 5; invalid/gate-fail paths behave as specified.
* **Reporting:** JSON/HTML include one-decimal percentages and mandatory approval-denominator sentence for approval ballots; integrity footer echoes engine/FID/IDs/seed (when used).
* **Artifacts:** canonical JSON ends with single LF; sorted keys; UTC timestamps; IDs match computed hashes.
