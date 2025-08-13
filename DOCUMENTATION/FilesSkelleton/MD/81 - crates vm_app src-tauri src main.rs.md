````md
Perfect Skeleton Sheet — crates/vm_app/src-tauri/src/main.rs — 81/89  
(Aligned with VM-ENGINE v0: offline, deterministic, fixed pipeline)

1) Goal & Success
Goal: Provide the Tauri backend entry that exposes a **minimal, safe, offline** command surface to load local inputs, run the pipeline (Doc 5), and export reports (Doc 7).  
Success: Builds & runs on Win/macOS/Linux; commands operate on local files only; outputs (Result, RunRecord, optional FrontierMap) are **canonical JSON** (UTF-8, sorted keys, LF, UTC); **no telemetry/network**.

2) Scope
In: `main()` bootstrap, command registration, deterministic panic hook, IPC types, strict FS checks (path scope, no URLs), error mapping.  
Out: UI code, tauri.conf.json FS policy, map assets packaging, algorithm math (lives in vm_* crates).

3) Inputs → Outputs
Inputs (all local): DivisionRegistry, Ballots/Tally, ParameterSet, optional Manifest/Adjacency; optional output dir; report format (json/html).  
Outputs:  
• `Result` (RES:…), `RunRecord` (RUN:…), optional `FrontierMap` (FR:…) — canonical JSON.  
• Reports (JSON/HTML) rendered **only** from Result/RunRecord/FrontierMap with **one-decimal** %.

4) Entities/Tables (minimal)
IPC DTOs only (engine info, loaded summary, run summary, report paths, hash pair). Internal IDs are stringly typed wrappers from vm_io/vm_pipeline.

5) Variables (used here)
No VM-VAR evaluation. This file **echoes** engine identifiers and forwards a seed (if any) to pipeline; determinism (ordering/rounding) enforced downstream.

6) Functions (signatures & skeleton only)
```rust
//! Tauri backend entry — offline, deterministic.

#![forbid(unsafe_code)]
use std::{path::{Path, PathBuf}, sync::Mutex};
use serde::{Serialize, Deserialize};
use tauri::{Manager, State};

use vm_pipeline::{ /* run_from_manifest, run_with_ctx, types… */ };
use vm_report::{ /* build_model, render_json, render_html */ };
use vm_io::{ /* canonical json writer, hasher */ };

/// Global app state (avoid mutable singletons; keep minimal & deterministic).
struct AppState {
    // If you need to cache engine identifiers or feature flags:
    engine: EngineInfo,
}

#[derive(Clone, Serialize)]
pub struct EngineInfo {
    pub formula_id: String,    // e.g., "VM-ENGINE v0"
    pub engine_version: String,// semver or commit
    pub targets: Vec<String>,  // ["windows-x86_64","macos-aarch64",…]
}

#[derive(Clone, Serialize)]
pub struct LoadedContextSummary {
    pub registry_id: String,
    pub ballot_or_tally_id: String,
    pub parameter_set_id: String,
    pub has_adjacency: bool,
}

#[derive(Clone, Serialize)]
pub struct RunSummary {
    pub result_id: String,      // RES:…
    pub run_id: String,         // RUN:…
    pub frontier_id: Option<String>, // FR:…
    pub label: String,          // Decisive|Marginal|Invalid
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ReportFmt { Json, Html }

#[derive(Clone, Serialize)]
pub struct ReportPaths {
    pub json_path: Option<String>,
    pub html_path: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct HashPair {
    pub result_sha256: String,
    pub run_record_sha256: String,
}

/// Deterministic backend error mapped to UI-safe strings.
#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error("path not allowed: {0}")]
    PathNotAllowed(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("pipeline error: {0}")]
    Pipeline(String),
    #[error("report error: {0}")]
    Report(String),
}

type BE<T> = Result<T, BackendError>;

/// ---- Determinism helpers -------------------------------------------------

fn set_deterministic_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{{\"panic\":\"backend\",\"msg\":\"{}\"}}", info);
    }));
}

/// Reject anything outside allowed scope; never follow URLs; block symlink escape.
fn assert_path_in_scope(app: &tauri::AppHandle, p: &Path) -> BE<()> {
    let allow_root = app.path_resolver().app_data_dir()
        .or_else(|| app.path_resolver().app_dir())
        .ok_or_else(|| BackendError::PathNotAllowed("no app dir".into()))?;
    let canon = std::fs::canonicalize(p).map_err(|e| BackendError::Io(e.to_string()))?;
    let root  = std::fs::canonicalize(&allow_root).map_err(|e| BackendError::Io(e.to_string()))?;
    if !canon.starts_with(&root) {
        return Err(BackendError::PathNotAllowed(format!("{}", p.display())));
    }
    Ok(())
}

/// ---- Commands (IPC) ------------------------------------------------------

#[tauri::command]
fn cmd_engine_info(state: State<'_, AppState>) -> EngineInfo {
    state.engine.clone()
}

#[tauri::command]
fn cmd_load_inputs(
    app: tauri::AppHandle,
    registry: PathBuf,
    ballots_or_tally: PathBuf,
    params: PathBuf,
    manifest: Option<PathBuf>,
) -> BE<LoadedContextSummary> {
    // 1) Scope & existence checks
    for p in [&registry, &ballots_or_tally, &params] {
        assert_path_in_scope(&app, p)?;
        if !p.exists() { return Err(BackendError::InvalidInput(format!("missing {}", p.display()))); }
    }
    if let Some(m) = &manifest {
        assert_path_in_scope(&app, m)?;
        if !m.exists() { return Err(BackendError::InvalidInput(format!("missing {}", m.display()))); }
    }

    // 2) Cheap probe (IDs/labels) — leave heavy validation to pipeline loader
    //    (Suggest: vm_io::probe_* helpers if available; else parse minimal fields.)
    let summary = LoadedContextSummary {
        registry_id: "REG:…".into(),
        ballot_or_tally_id: "TLY:…".into(),
        parameter_set_id: "PS:…".into(),
        has_adjacency: false,
    };
    Ok(summary)
}

#[tauri::command]
fn cmd_run_pipeline(
    app: tauri::AppHandle,
    registry: PathBuf,
    ballots_or_tally: PathBuf,
    params: PathBuf,
    manifest: Option<PathBuf>,
    out_dir: PathBuf,
) -> BE<RunSummary> {
    for p in [&registry, &ballots_or_tally, &params, &out_dir] {
        assert_path_in_scope(&app, p)?;
        if p != &out_dir && !p.exists() {
            return Err(BackendError::InvalidInput(format!("missing {}", p.display())));
        }
    }
    // Create output dir if needed
    std::fs::create_dir_all(&out_dir).map_err(|e| BackendError::Io(e.to_string()))?;

    // Orchestrate Doc 5 pipeline via vm_pipeline (LOAD→…→BUILD_RUN_RECORD)
    // NOTE: Actual orchestration lives in vm_pipeline; we just call it.
    // let outputs = vm_pipeline::run_from_manifest_or_paths(...).map_err(|e| BackendError::Pipeline(e.to_string()))?;

    // Canonical writes: UTF-8, sorted keys, LF, UTC (done in vm_io writers).
    // vm_io::write_canonical_json(out_dir.join("result.json"), &outputs.result)?;
    // vm_io::write_canonical_json(out_dir.join("run_record.json"), &outputs.run_record)?;
    // if let Some(fr) = outputs.frontier { vm_io::write_canonical_json(out_dir.join("frontier_map.json"), &fr)?; }

    // Stubbed summary (replace with real IDs from outputs)
    Ok(RunSummary {
        result_id: "RES:…".into(),
        run_id: "RUN:…".into(),
        frontier_id: None,
        label: "Decisive".into(),
    })
}

#[tauri::command]
fn cmd_export_report(
    app: tauri::AppHandle,
    result_path: PathBuf,
    run_record_path: PathBuf,
    out_dir: PathBuf,
    fmt: ReportFmt,
) -> BE<ReportPaths> {
    for p in [&result_path, &run_record_path, &out_dir] {
        assert_path_in_scope(&app, p)?;
    }
    std::fs::create_dir_all(&out_dir).map_err(|e| BackendError::Io(e.to_string()))?;

    // Load artifacts; build ReportModel; render via vm_report
    // let (res, run, frontier_opt) = vm_io::read_artifacts(...)?;
    // let model = vm_report::build_model(&res, &run, frontier_opt.as_ref());
    // let mut paths = ReportPaths { json_path: None, html_path: None };
    // match fmt {
    //   ReportFmt::Json => { let s = vm_report::render_json(&model); write_string(out_dir.join("report.json"), s)?; paths.json_path = Some(...); }
    //   ReportFmt::Html => { let s = vm_report::render_html(&model); write_string(out_dir.join("report.html"), s)?; paths.html_path = Some(...); }
    // }
    Ok(ReportPaths { json_path: Some(out_dir.join("report.json").display().to_string()), html_path: None })
}

#[tauri::command]
fn cmd_hash_artifacts(
    app: tauri::AppHandle,
    result_path: PathBuf,
    run_record_path: PathBuf,
) -> BE<HashPair> {
    for p in [&result_path, &run_record_path] { assert_path_in_scope(&app, p)?; }
    // let res_hex = vm_io::hasher::sha256_file(&result_path).map_err(|e| BackendError::Io(e.to_string()))?;
    // let run_hex = vm_io::hasher::sha256_file(&run_record_path).map_err(|e| BackendError::Io(e.to_string()))?;
    Ok(HashPair { result_sha256: "…".into(), run_record_sha256: "…".into() })
}

/// ---- Tauri bootstrap -----------------------------------------------------

fn main() {
    set_deterministic_panic_hook();

    tauri::Builder::default()
        .manage(AppState {
            engine: EngineInfo {
                formula_id: "VM-ENGINE v0".into(),
                engine_version: env!("CARGO_PKG_VERSION").into(),
                targets: vec![
                    "windows-x86_64".into(), "windows-aarch64".into(),
                    "macos-x86_64".into(),   "macos-aarch64".into(),
                    "linux-x86_64".into(),   "linux-aarch64".into(),
                ],
            }
        })
        .invoke_handler(tauri::generate_handler![
            cmd_engine_info,
            cmd_load_inputs,
            cmd_run_pipeline,
            cmd_export_report,
            cmd_hash_artifacts
        ])
        // FS/network policy is primarily in tauri.conf.json; keep backend command set minimal.
        .run(tauri::generate_context!())
        .expect("app failed to start");
}
````

7. Algorithm Outline (per command)

* `cmd_engine_info` → return static FormulaID/EngineVersion/targets.
* `cmd_load_inputs` → scope+existence checks → probe IDs (cheap) → `LoadedContextSummary`.
* `cmd_run_pipeline` → scope checks → create out dir → call **vm\_pipeline** (LOAD→VALIDATE→TABULATE→ALLOCATE→AGGREGATE→APPLY\_RULES→MAP\_FRONTIER→RESOLVE\_TIES→LABEL→BUILD\_RESULT/RUN\_RECORD) → canonical writes via **vm\_io** → `RunSummary`.
* `cmd_export_report` → read artifacts → `vm_report::build_model` → render JSON/HTML (one-decimal) → file paths.
* `cmd_hash_artifacts` → SHA-256 over canonical bytes (use vm\_io hasher) → hex strings.

8. State Flow
   UI → IPC command → backend delegates to **vm\_* crates*\* → writes canonical artifacts → optional report render → returns small summaries/paths.

9. Determinism & Numeric Rules

* No network/telemetry; all inputs are local paths inside allowed scope.
* Panic hook is deterministic (single JSON line to stderr).
* RNG used **only** by pipeline when seed provided; seed echoed in RunRecord; no OS RNG.
* Canonical JSON rules are enforced by writers (UTF-8, **sorted keys**, **LF**, **UTC**).

10. Edge Cases & Failure Policy

* Path escapes / URLs / symlinks outside scope ⇒ `PathNotAllowed`.
* Missing files / malformed JSON ⇒ `InvalidInput`/`Io` (pipeline still packages **Invalid** Result when appropriate).
* If gates fail, pipeline sets label **Invalid** and **omits Frontier**; backend still returns `RunSummary`.
* Report without FrontierMap still renders (frontier section omitted per Doc 7).

11. Test Checklist (must pass)

* Launch app; run `cmd_run_pipeline` on Part 0 bundle ⇒ writes Result/RunRecord; hashes reproducible across repeats/OS.
* `cmd_export_report` produces JSON/HTML with **one-decimal** percentages; no external assets loaded.
* Attempts to access paths outside scope or any HTTP/DNS are rejected with `PathNotAllowed`.
* Same inputs (+seed if used) ⇒ identical bytes for Result/RunRecord (verify via `cmd_hash_artifacts`).

```
```
