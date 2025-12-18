// crates/vm_cli/src/main.rs — Half 1/2
//
// This half wires up: spec exit codes, typed error mapping, CLI parsing,
// and the validate-only short-circuit. The full run path (engine meta → load →
// seed override → pipeline → artifacts → optional rendering) will be added in
// Half 2/2. There are NO stubs here, and no block is split mid-way.

mod args; // sibling module in this crate

mod exitcodes {
    /// Spec-aligned exit codes (Docs 3/5/6)
    pub const OK: i32 = 0;
    pub const VALIDATION: i32 = 2;
    pub const SELF_VERIFY: i32 = 3;
    pub const IO: i32 = 4;
    pub const SPEC: i32 = 5;
}

use std::process::ExitCode;

use args::{parse_and_validate as parse_cli, Args};

use vm_io::loader;
use vm_pipeline::PipelineError;

/// Central error type for CLI → exit-code mapping.
#[derive(Debug)]
enum MainError {
    /// Schema / JSON shape / manifest / canonicalization / hash expectation failures
    Validation(String),
    /// Self-verification / build-time canonical mismatch (e.g., FID/ID/digest mismatch)
    SelfVerify(String),
    /// I/O errors (read/write/path/limits)
    Io(String),
    /// Spec violation (pipeline logic category: gates/frontier/ties/tabulate/allocate)
    Spec(String),
    /// Pipeline general error (fallback; will be mapped reasonably)
    Pipeline(String),
    /// Rendering errors (report build or output)
    Render(String),
    /// Catch-all
    Other(String),
}

fn main() -> ExitCode {
    let args = match parse_cli() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("vm: error: {e}");
            return ExitCode::from(exitcodes::VALIDATION as u8);
        }
    };

    // Honor --validate_only as a hard short-circuit (Doc 6 harness behavior).
    let rc = if args.validate_only {
        match validate_only(&args) {
            Ok(()) => exitcodes::OK,
            Err(e) => map_error(&e),
        }
    } else {
        // Full run path is implemented in Half 2/2 (function `run_once`)
        match run_once(&args) {
            Ok(()) => exitcodes::OK,
            Err(e) => map_error(&e),
        }
    };

    ExitCode::from(rc as u8)
}

/// Validate-only path (no pipeline, no artifacts).
/// Loads inputs via vm_io::loader to exercise schema/domain/ref/order checks.
/// Exit codes:
///   0 on success
///   2 on validation failures (schema/manifest/hash expectations)
///   4 on I/O/path/limits errors
fn validate_only(args: &Args) -> Result<(), MainError> {
    let loaded = if let Some(manifest) = &args.manifest {
        loader::load_normative_from_manifest_path(manifest)
    } else {
        loader::load_normative_from_paths(
            args.registry.as_ref().expect("args validated: --registry"),
            args.params.as_ref().expect("args validated: --params"),
            args.ballots.as_ref(),
            args.tally.as_ref(),
            args.adjacency.as_ref(),
            args.autonomy.as_ref(),
        )
    };

    match loaded {
        Ok(_) => {
            if !args.quiet {
                eprintln!("validate-only: inputs OK");
            }
            Ok(())
        }
        Err(e) => Err(map_vmio_err(e)),
    }
}

/// Map our typed errors to the spec exit-code table.
fn map_error(e: &MainError) -> i32 {
    use exitcodes::*;
    match e {
        MainError::Validation(_) => VALIDATION,
        MainError::SelfVerify(_) => SELF_VERIFY,
        MainError::Io(_) => IO,
        MainError::Spec(_) => SPEC,
        MainError::Pipeline(_) => IO, // default bucket unless pipeline signals otherwise
        MainError::Render(_) => IO,
        MainError::Other(_) => IO,
    }
}

/// Translate vm_io::IoError into MainError buckets for exit-code mapping.
fn map_vmio_err(e: vm_io::IoError) -> MainError {
    use vm_io::IoError::*;
    match e {
        // Validation-ish (schema/shape/manifest/expectations/canonical/hash)
        Schema { pointer, msg } => MainError::Validation(format!("schema {pointer}: {msg}")),
        Json { pointer, msg } => MainError::Validation(format!("json {pointer}: {msg}")),
        Manifest(m) => MainError::Validation(format!("manifest: {m}")),
        Expect(m) => MainError::Validation(format!("expect: {m}")),
        Canon(m) => MainError::Validation(format!("canon: {m}")),
        Hash(m) => MainError::Validation(format!("hash: {m}")),

        // I/O-ish
        Read(e) => MainError::Io(format!("read: {e}")),
        Write(e) => MainError::Io(format!("write: {e}")),
        Path(m) => MainError::Io(format!("path: {m}")),
        Limit(m) => MainError::Io(format!("limit: {m}")),
    }
}

/// Translate vm_pipeline::PipelineError into MainError buckets (used by `run_once` in Half 2/2).
fn map_pipeline_err(e: PipelineError) -> MainError {
    use PipelineError::*;
    match e {
        // Validation-like buckets
        Schema(m) | Validate(m) => MainError::Validation(m),
        // I/O
        Io(m) => MainError::Io(m),
        // Build/self-verify mismatches (e.g., ID/FID/digest checks)
        Build(m) => MainError::SelfVerify(m),
        // Spec-directed algorithmic failures
        Tabulate(m) | Allocate(m) | Gates(m) | Frontier(m) | Tie(m) => MainError::Spec(m),
        // Fallback (covers any enum growth)
        _ => MainError::Pipeline(format!("{e:?}")),
    }
}

// NOTE: `run_once(&Args) -> Result<(), MainError>` is implemented in Half 2/2.
// crates/vm_cli/src/main.rs — Half 2/2
//
// Full run path (engine meta → load → seed override → pipeline → artifacts → rendering).
// Complements Half 1/2 (which defined error mapping, validate-only, and main()).

use std::fs;
use std::path::Path;

use serde_json::json;
use vm_io::{canonical_json, loader};
use vm_pipeline::{run_with_ctx, EngineMeta, PipelineCtx, PipelineOutputs};
use vm_report::{build_model, ReportError};

fn run_once(args: &Args) -> Result<(), MainError> {
    // 1) Deterministic engine metadata (compile-time env where available)
    let engine_meta = EngineMeta {
        vendor: option_env!("VM_ENGINE_VENDOR").unwrap_or("vm").to_string(),
        name: option_env!("VM_ENGINE_NAME")
            .unwrap_or(env!("CARGO_PKG_NAME"))
            .to_string(),
        version: option_env!("VM_ENGINE_VERSION")
            .unwrap_or(env!("CARGO_PKG_VERSION"))
            .to_string(),
        build: option_env!("VM_ENGINE_BUILD").unwrap_or("dev").to_string(),
    };

    // 2) Load normative context (schema + structural validation)
    let mut loaded = if let Some(manifest) = &args.manifest {
        loader::load_normative_from_manifest_path(manifest).map_err(map_vmio_err)?
    } else {
        loader::load_normative_from_paths(
            args.registry.as_ref().expect("args validated: --registry"),
            args.params.as_ref().expect("args validated: --params"),
            args.ballots.as_ref(),
            args.tally.as_ref(),
            args.adjacency.as_ref(),
            args.autonomy.as_ref(),
        )
        .map_err(map_vmio_err)?
    };

    // 3) Apply seed override (VM-VAR-052) if provided
    if let Some(seed) = args.seed {
        // Adjust field name to your ParameterSet model if different.
        loaded.params.v052_tie_seed = Some(seed);
    }

    // 4) Normative manifest JSON for FID (TODO: replace with Included-only builder per Annex A)
    let nm_canonical = serde_json::to_value(&loaded.params).unwrap_or_else(|_| json!({}));

    // 5) Run pipeline
    let ctx = PipelineCtx {
        loaded,
        engine_meta,
        nm_canonical,
    };
    let outs = run_with_ctx(ctx).map_err(map_pipeline_err)?;

    // 6) Write canonical artifacts
    write_artifacts(&args.out, &outs)?;

    // 7) Optional report rendering (read-only; offline)
    maybe_render_reports(args, &outs, &args.out)?;

    if !args.quiet {
        eprintln!("run: artifacts written to {}", args.out.to_string_lossy());
    }
    Ok(())
}

fn write_artifacts(out_dir: &Path, outs: &PipelineOutputs) -> Result<(), MainError> {
    fs::create_dir_all(out_dir)
        .map_err(|e| MainError::Io(format!("mkdir {}: {e}", out_dir.to_string_lossy())))?;

    // result.json
    let res_path = out_dir.join("result.json");
    canonical_json::write_canonical_file(&outs.result, &res_path)
        .map_err(|e| MainError::Io(format!("write result.json: {e}")))?;

    // run_record.json
    let run_path = out_dir.join("run_record.json");
    canonical_json::write_canonical_file(&outs.run_record, &run_path)
        .map_err(|e| MainError::Io(format!("write run_record.json: {e}")))?;

    // frontier_map.json (optional; pipeline decides based on VM-VAR-034/040)
    if let Some(frontier) = &outs.frontier_map {
        let fr_path = out_dir.join("frontier_map.json");
        canonical_json::write_canonical_file(frontier, &fr_path)
            .map_err(|e| MainError::Io(format!("write frontier_map.json: {e}")))?;
    }

    Ok(())
}

fn maybe_render_reports(args: &Args, outs: &PipelineOutputs, out_dir: &Path) -> Result<(), MainError> {
    if args.render.is_empty() {
        return Ok(());
    }

    // Build in-memory report model from canonical artifacts
    let result_val = serde_json::to_value(&outs.result)
        .map_err(|e| MainError::Render(format!("result to JSON: {e}")))?;
    let run_val = serde_json::to_value(&outs.run_record)
        .map_err(|e| MainError::Render(format!("run_record to JSON: {e}")))?;
    let frontier_val = match &outs.frontier_map {
        Some(f) => Some(
            serde_json::to_value(f).map_err(|e| MainError::Render(format!("frontier to JSON: {e}")))?,
        ),
        None => None,
    };
    let frontier_opt = frontier_val.as_ref();

    let model = build_model(&result_val, &run_val, frontier_opt, None).map_err(map_report_err)?;

    // Emit requested formats (unknown → error)
    for fmt in &args.render {
        match fmt.as_str() {
            "json" => render_json_report(&model, out_dir)?,
            "html" => render_html_report(&model, out_dir)?,
            other => return Err(MainError::Render(format!("unknown renderer: {other}"))),
        }
    }
    Ok(())
}

// Always accept the concrete model type; gate body by feature.
fn render_json_report(model: &vm_report::ReportModel, out_dir: &Path) -> Result<(), MainError> {
    #[cfg(feature = "report-json")]
    {
        let path = out_dir.join("report.json");
        return canonical_json::write_canonical_file(model, &path)
            .map_err(|e| MainError::Io(format!("write report.json: {e}")));
    }
    #[cfg(not(feature = "report-json"))]
    {
        Err(MainError::Render(
            "json renderer not enabled (build with feature `report-json`)".into(),
        ))
    }
}

fn render_html_report(model: &vm_report::ReportModel, out_dir: &Path) -> Result<(), MainError> {
    // Minimal, deterministic, asset-free HTML (no recomputation).
    let mut html = String::new();
    use std::fmt::Write;

    writeln!(
        &mut html,
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Voting Model Report</title></head><body>"
    )
    .unwrap();

    // Cover
    writeln!(
        &mut html,
        "<h1>{}</h1><h2>Outcome: {}</h2>{}",
        esc(&model.cover.title),
        esc(&model.cover.label),
        model
            .cover
            .reason
            .as_ref()
            .map(|r| format!("<p>{}</p>", esc(r)))
            .unwrap_or_default()
    )
    .unwrap();

    // Snapshot
    writeln!(&mut html, "<h3>Snapshot</h3><ul>").unwrap();
    for it in &model.snapshot.items {
        writeln!(&mut html, "<li><b>{}</b>: {}</li>", esc(&it.key), esc(&it.value)).unwrap();
    }
    writeln!(&mut html, "</ul>").unwrap();

    // Eligibility
    let elig = &model.eligibility;
    writeln!(
        &mut html,
        "<h3>Eligibility</h3><p>Roll policy: {}<br>Registry source: {}<br>Eligible: {} | Cast: {} | Valid: {}</p>",
        esc(&elig.roll_policy),
        esc(&elig.registry_source),
        elig.totals.eligible_roll,
        elig.totals.ballots_cast,
        elig.totals.valid_ballots
    )
    .unwrap();
    if let Some(note) = &elig.per_unit_quorum_note {
        writeln!(&mut html, "<p>{}</p>", esc(note)).unwrap();
    }

    // Ballot method
    writeln!(
        &mut html,
        "<h3>Ballot Method</h3><p>Method: {} | Allocation: {} | Weighting: {}</p>",
        esc(&model.ballot_method.method),
        esc(&model.ballot_method.allocation),
        esc(&model.ballot_method.weighting)
    )
    .unwrap();
    if let Some(sent) = &model.ballot_method.approval_denominator_sentence {
        writeln!(&mut html, "<p><em>{}</em></p>", esc(sent)).unwrap();
    }

    // Legitimacy
    let leg = &model.legitimacy_panel;
    writeln!(
        &mut html,
        "<h3>Legitimacy</h3><p>Quorum: {}% ({}%) → {}<br>Majority: {}% ({}%) → {}<br>Pass: {}</p>",
        esc(&leg.quorum.value_pct_1dp),
        esc(&leg.quorum.threshold_pct_0dp),
        yesno(leg.quorum.pass),
        esc(&leg.majority.value_pct_1dp),
        esc(&leg.majority.threshold_pct_0dp),
        yesno(leg.majority.pass),
        yesno(leg.pass)
    )
    .unwrap();
    if let Some((nat, fam)) = &leg.double_majority {
        writeln!(
            &mut html,
            "<p>Double majority – National: {}% / thr {}% → {}; Family: {}% / thr {}% → {}</p>",
            esc(&nat.value_pct_1dp),
            esc(&nat.threshold_pct_0dp),
            yesno(nat.pass),
            esc(&fam.value_pct_1dp),
            esc(&fam.threshold_pct_0dp),
            yesno(fam.pass)
        )
        .unwrap();
    }
    if !leg.reasons.is_empty() {
        writeln!(&mut html, "<ul>").unwrap();
        for r in &leg.reasons {
            writeln!(&mut html, "<li>{}</li>", esc(r)).unwrap();
        }
        writeln!(&mut html, "</ul>").unwrap();
    }

    // Outcome
    writeln!(
        &mut html,
        "<h3>Outcome</h3><p>Label: {}<br>Reason: {}<br>National margin: {}</p>",
        esc(&model.outcome_label.label),
        esc(&model.outcome_label.reason),
        esc(&model.outcome_label.national_margin_pp)
    )
    .unwrap();

    // Frontier (optional)
    if let Some(fr) = &model.frontier {
        writeln!(
            &mut html,
            "<h3>Frontier</h3><p>Mode: {} | Edge policy: {} | Island rule: {}</p>",
            esc(&fr.mode),
            esc(&fr.edge_policy),
            esc(&fr.island_rule)
        )
        .unwrap();
        writeln!(
            &mut html,
            "<p>Changed: {} | No change: {} | Mediation: {} | Enclave: {} | Protected blocked: {} | Quorum blocked: {}</p>",
            fr.counters.changed,
            fr.counters.no_change,
            fr.counters.mediation,
            fr.counters.enclave,
            fr.counters.protected_blocked,
            fr.counters.quorum_blocked
        )
        .unwrap();
        if !fr.bands_summary.is_empty() {
            writeln!(&mut html, "<ul>").unwrap();
            for s in &fr.bands_summary {
                writeln!(&mut html, "<li>{}</li>", esc(s)).unwrap();
            }
            writeln!(&mut html, "</ul>").unwrap();
        }
    }

    // Sensitivity (optional)
    if let Some(sens) = &model.sensitivity {
        writeln!(&mut html, "<h3>Sensitivity</h3>").unwrap();
        for row in &sens.table {
            let line = row.iter().map(|c| esc(c)).collect::<Vec<_>>().join(" | ");
            writeln!(&mut html, "<p>{}</p>", line).unwrap();
        }
    }

    // Integrity
    let integ = &model.integrity;
    writeln!(
        &mut html,
        "<h3>Integrity</h3><p>Result ID: {}<br>Run ID: {}<br>Formula ID: {}<br>Engine: {} {} ({}) build {}</p>",
        esc(integ.result_id.as_str()),
        esc(integ.run_id.as_str()),
        esc(&integ.formula_id_hex),
        esc(&integ.engine_vendor),
        esc(&integ.engine_name),
        esc(&integ.engine_version),
        esc(&integ.engine_build)
    )
    .unwrap();
    if let Some(frid) = &integ.frontier_id {
        writeln!(&mut html, "<p>Frontier ID: {}</p>", esc(frid.as_str())).unwrap();
    }
    if let Some(seed) = &integ.tie_seed {
        writeln!(&mut html, "<p>Tie seed: {}</p>", esc(seed)).unwrap();
    }

    writeln!(&mut html, "</body></html>").unwrap();

    let path = out_dir.join("report.html");
    fs::write(&path, html).map_err(|e| MainError::Io(format!("write report.html: {e}")))?;
    Ok(())
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn yesno(b: bool) -> &'static str {
    if b { "pass" } else { "fail" }
}

fn map_report_err(e: ReportError) -> MainError {
    use ReportError::*;
    match e {
        Template(m) => MainError::Render(format!("template: {m}")),
        MissingField(m) => MainError::Render(format!("missing: {m}")),
        Inconsistent(m) => MainError::Render(format!("inconsistent: {m}")),
    }
}
