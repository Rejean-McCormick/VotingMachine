// crates/vm_cli/src/main.rs
//
// VM-ENGINE v0 — CLI entrypoint
// Drives the fixed pipeline end-to-end, writes canonical artifacts, and (optionally) renders reports.
// Strictly offline & deterministic: no network, no OS RNG.
//
// This file stays aligned with the pipeline & report crates built earlier in the series.

mod args;

use args::{parse_and_validate, Args, CliError};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use vm_io::canonical_json::to_canonical_bytes;
use vm_pipeline as pipeline;

#[cfg(feature = "report-json")]
use vm_report::render_json as render_json_report;
#[cfg(feature = "report-html")]
use vm_report::render_html as render_html_report;

// Filenames for emitted artifacts in the output directory.
const RESULT_FILE: &str = "result.json";
const RUN_FILE: &str = "run_record.json";
const FRONTIER_FILE: &str = "frontier_map.json";
const SYN_MANIFEST_FILE: &str = "manifest.synthetic.json";

fn main() -> ExitCode {
    let args = match parse_and_validate() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("vm: error: {e}");
            return ExitCode::from(1);
        }
    };

    match run(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("vm: error: {e}");
            ExitCode::from(1)
        }
    }
}

/// Run the whole orchestration. Returns the process exit code per policy.
/// Notes:
/// - This CLI uses the pipeline's consolidated entrypoint (`run_from_manifest_path`)
///   to guarantee the fixed stage order and artifact construction.
/// - We still keep the file I/O and rendering here, deterministically.
fn run(args: Args) -> Result<ExitCode, String> {
    // Ensure output directory exists
    fs::create_dir_all(&args.out)
        .map_err(|e| format!("cannot create output directory {}: {e}", args.out.display()))?;

    // Resolve a manifest path: either the one provided or a synthesized one from explicit flags.
    let manifest_path = if let Some(p) = &args.manifest {
        p.clone()
    } else {
        let synth_bytes = synthesize_manifest_from_explicit(&args)?;
        let path = args.out.join(SYN_MANIFEST_FILE);
        write_bytes_atomically(&path, &synth_bytes)
            .map_err(|e| format!("cannot write synthetic manifest {}: {e}", path.display()))?;
        path
    };

    // Run the pipeline in one go; it returns Result, RunRecord, and optional FrontierMap.
    let outs = pipeline::run_from_manifest_path(&manifest_path)
        .map_err(|e| format!("pipeline failed: {e:?}"))?;

    // Persist canonical artifacts
    write_artifacts(&args.out, &outs).map_err(|e| format!("write artifacts: {e}"))?;

    // Render requested report formats (if any)
    if !args.render.is_empty() {
        render_reports(&args.out, &outs, &args.render)
            .map_err(|e| format!("render reports: {e}"))?;
    }

    // Exit policy based on label and gates
    let (label_upper, gates_pass) = extract_label_and_gates(&outs);
    if !args.quiet {
        println!("vm: completed — label={}", label_upper);
        if let Some(passed) = gates_pass {
            println!("vm: gates_pass={passed}");
        }
        println!("vm: artifacts written to {}", args.out.display());
    }

    let code = match label_upper.as_str() {
        "DECISIVE" | "MARGINAL" => ExitCode::from(0),
        "INVALID" => ExitCode::from(3),
        _ => ExitCode::from(0),
    };
    Ok(code)
}

fn write_artifacts(out_dir: &Path, outs: &pipeline::PipelineOutputs) -> io::Result<()> {
    let result_path = out_dir.join(RESULT_FILE);
    let run_path = out_dir.join(RUN_FILE);

    let result_bytes = to_canonical_bytes(&outs.result);
    let run_bytes = to_canonical_bytes(&outs.run_record);

    write_bytes_atomically(&result_path, &result_bytes)?;
    write_bytes_atomically(&run_path, &run_bytes)?;

    if let Some(fr) = &outs.frontier_map {
        let frontier_path = out_dir.join(FRONTIER_FILE);
        let frontier_bytes = to_canonical_bytes(fr);
        write_bytes_atomically(&frontier_path, &frontier_bytes)?;
    }
    Ok(())
}

fn render_reports(
    out_dir: &Path,
    outs: &pipeline::PipelineOutputs,
    formats: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    // Build once; reuse for all selected renderers.
    let model = vm_report::build_model(
        &outs.result,
        &outs.run_record,
        outs.frontier_map.as_ref(),
        None,
    )?;

    for fmt in formats {
        match fmt.as_str() {
            #[cfg(feature = "report-json")]
            "json" => {
                let s = render_json_report(&model)?;
                let path = out_dir.join("report.json");
                write_bytes_atomically(&path, s.as_bytes())?;
            }
            #[cfg(feature = "report-html")]
            "html" => {
                let s = render_html_report(
                    &model,
                    vm_report::HtmlRenderOpts {
                        bilingual: None,
                        embed_assets: true,
                    },
                );
                let path = out_dir.join("report.html");
                write_bytes_atomically(&path, s.as_bytes())?;
            }
            other => eprintln!("vm: warning: unknown --render format: {other}"),
        }
    }
    Ok(())
}

/// Create a tiny manifest from explicit CLI flags so we can drive the pipeline’s
/// manifest-based loader deterministically. Paths are already local & validated.
fn synthesize_manifest_from_explicit(a: &Args) -> Result<Vec<u8>, String> {
    let mut m = serde_json::Map::new();

    if let Some(reg) = &a.registry {
        m.insert("reg_path".into(), serde_json::Value::String(reg.display().to_string()));
    } else {
        return Err("missing --registry (explicit mode)".into());
    }
    if let Some(ps) = &a.params {
        m.insert("params_path".into(), serde_json::Value::String(ps.display().to_string()));
    } else {
        return Err("missing --params (explicit mode)".into());
    }

    match (&a.ballots, &a.tally) {
        (Some(b), None) => m.insert("ballots_path".into(), serde_json::Value::String(b.display().to_string())),
        (None, Some(t)) => m.insert("ballot_tally_path".into(), serde_json::Value::String(t.display().to_string())),
        _ => return Err("exactly one of --ballots or --tally is required".into()),
    };

    if let Some(adj) = &a.adjacency {
        m.insert("adjacency_path".into(), serde_json::Value::String(adj.display().to_string()));
    }
    if let Some(ap) = &a.autonomy {
        m.insert(
            "autonomy_packages_path".into(),
            serde_json::Value::String(ap.display().to_string()),
        );
    }

    let value = serde_json::Value::Object(m);
    Ok(to_canonical_bytes(&value))
}

/// Write bytes with a single trailing LF and atomic rename.
fn write_bytes_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = tmp_path_for(path);
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        if !bytes.last().is_some_and(|b| *b == b'\n') {
            f.write_all(b"\n")?;
        }
        f.sync_all()?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

fn tmp_path_for(final_path: &Path) -> PathBuf {
    let mut s = final_path.as_os_str().to_owned();
    s.push(".tmp");
    PathBuf::from(s)
}

/// Pull label (uppercased) and overall gates pass from the Result artifact
/// without tightly coupling to concrete types here.
fn extract_label_and_gates(outs: &pipeline::PipelineOutputs) -> (String, Option<bool>) {
    let v = serde_json::to_value(&outs.result).ok();

    let label_upper = v
        .as_ref()
        .and_then(|r| r.get("label"))
        .and_then(|l| l.as_str())
        .map(|s| s.to_ascii_uppercase())
        .unwrap_or_else(|| "UNKNOWN".into());

    let gates_pass = v
        .as_ref()
        .and_then(|r| r.get("gates"))
        .and_then(|g| g.get("pass"))
        .and_then(|p| p.as_bool());

    (label_upper, gates_pass)
}
