// crates/vm_cli/src/args.rs
//
// Deterministic, offline CLI argument parsing & validation.
// - No networked paths (reject http/https schemes)
// - Exactly one of: manifest  XOR  (registry + params + (ballots XOR tally))
// - Optional seed parsing (u64 decimal or 0x-hex up to 16 nybbles)
// - Light manifest “quick-check” without pulling schema I/O

use clap::Parser;
use std::{
    env,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Debug, Parser)]
pub struct Args {
    // Mode selection
    #[arg(long, conflicts_with_all = ["registry", "params", "ballots", "tally"])]
    pub manifest: Option<PathBuf>,

    // Explicit mode
    #[arg(long)]
    pub registry: Option<PathBuf>,
    #[arg(long)]
    pub params: Option<PathBuf>,
    #[arg(long, conflicts_with = "tally")]
    pub ballots: Option<PathBuf>,
    #[arg(long, conflicts_with = "ballots")]
    pub tally: Option<PathBuf>,

    // Optional inputs
    #[arg(long)]
    pub adjacency: Option<PathBuf>,
    #[arg(long)]
    pub autonomy: Option<PathBuf>,

    // Output & rendering
    #[arg(long, default_value = ".")]
    pub out: PathBuf,
    #[arg(long, value_parser = ["json", "html"], num_args = 0..=2)]
    pub render: Vec<String>,

    // Determinism & control
    /// Optional override for VM-VAR-033; accepts decimal u64 or 0x-prefixed hex (≤16 hex digits).
    #[arg(long)]
    pub seed: Option<String>,
    #[arg(long)]
    pub validate_only: bool,
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Debug)]
pub enum CliError {
    BadCombo(&'static str),
    Missing(&'static str),
    BallotsTallyChoice,
    NonLocalPath(String),
    NotFound(String),
    BadSeed(String),
    ManifestQuick(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CliError::*;
        match self {
            BadCombo(s) => write!(f, "invalid flag combination: {}", s),
            Missing(s) => write!(f, "missing required flag: {}", s),
            BallotsTallyChoice => write!(f, "both or neither of --ballots/--tally provided"),
            NonLocalPath(p) => write!(f, "path must be local file (no scheme): {}", p),
            NotFound(p) => write!(f, "file not found: {}", p),
            BadSeed(s) => write!(f, "invalid seed: {}", s),
            ManifestQuick(s) => write!(f, "manifest quick-check failed: {}", s),
        }
    }
}
impl std::error::Error for CliError {}

/// Entry point used by main.rs
pub fn parse_and_validate() -> Result<Args, CliError> {
    let mut args = Args::parse();

    // Default renderers: JSON if unspecified
    if args.render.is_empty() {
        args.render.push("json".to_string());
    }

    // Validate modes
    if args.manifest.is_some() {
        validate_manifest_mode(&args)?;
    } else {
        validate_explicit_mode(&args)?;
    }

    // Normalize all paths (best-effort) after basic validation
    args.out = normalize_path(&args.out);
    if let Some(p) = &args.manifest {
        let _ = ensure_local_exists(p, "--manifest")?;
        // replace with normalized
        // (safe unwrap: we just checked Some)
        args.manifest = Some(normalize_path(p));
    } else {
        if let Some(p) = &args.registry {
            ensure_local_exists(p, "--registry")?;
        }
        if let Some(p) = &args.params {
            ensure_local_exists(p, "--params")?;
        }
        if let Some(p) = &args.ballots {
            ensure_local_exists(p, "--ballots")?;
        }
        if let Some(p) = &args.tally {
            ensure_local_exists(p, "--tally")?;
        }
        if let Some(p) = &args.adjacency {
            ensure_local_exists(p, "--adjacency")?;
        }
        if let Some(p) = &args.autonomy {
            ensure_local_exists(p, "--autonomy")?;
        }

        args.registry = args.registry.as_ref().map(normalize_path);
        args.params = args.params.as_ref().map(normalize_path);
        args.ballots = args.ballots.as_ref().map(normalize_path);
        args.tally = args.tally.as_ref().map(normalize_path);
        args.adjacency = args.adjacency.as_ref().map(normalize_path);
        args.autonomy = args.autonomy.as_ref().map(normalize_path);
    }

    // Seed quick-parse to fail early (we keep string in Args; main can re-parse to u64)
    if let Some(s) = &args.seed {
        let _ = parse_seed_u64(s)?;
    }

    Ok(args)
}

/// Manifest mode validation: require local file, quick-check shape.
fn validate_manifest_mode(a: &Args) -> Result<(), CliError> {
    let path = a
        .manifest
        .as_ref()
        .ok_or(CliError::Missing("--manifest"))?;

    ensure_local_exists(path, "--manifest")?;

    // Read a small-ish file defensively (hard cap e.g. 4 MiB)
    let mut f = fs::File::open(path).map_err(|_| CliError::NotFound(path.display().to_string()))?;
    let mut buf = Vec::new();
    const MAX_BYTES: usize = 4 * 1024 * 1024;
    f.take(MAX_BYTES as u64)
        .read_to_end(&mut buf)
        .map_err(|_| CliError::ManifestQuick("cannot read manifest bytes".into()))?;

    quick_check_manifest_bytes(&buf)
}

/// Explicit mode validation: require registry+params and exactly one of ballots XOR tally.
fn validate_explicit_mode(a: &Args) -> Result<(), CliError> {
    let reg = a
        .registry
        .as_ref()
        .ok_or(CliError::Missing("--registry"))?;
    let par = a.params.as_ref().ok_or(CliError::Missing("--params"))?;

    // exactly one of ballots XOR tally
    match (&a.ballots, &a.tally) {
        (Some(_), Some(_)) | (None, None) => return Err(CliError::BallotsTallyChoice),
        _ => {}
    }

    ensure_local_exists(reg, "--registry")?;
    ensure_local_exists(par, "--params")?;

    if let Some(b) = &a.ballots {
        ensure_local_exists(b, "--ballots")?;
    }
    if let Some(t) = &a.tally {
        ensure_local_exists(t, "--tally")?;
    }
    if let Some(adj) = &a.adjacency {
        ensure_local_exists(adj, "--adjacency")?;
    }
    if let Some(ap) = &a.autonomy {
        ensure_local_exists(ap, "--autonomy")?;
    }

    Ok(())
}

/// Ensure a path is local (no scheme) and exists as a file.
fn ensure_local_exists(p: &PathBuf, label: &'static str) -> Result<(), CliError> {
    let s = p.to_string_lossy().to_string();
    if has_scheme(&s) {
        return Err(CliError::NonLocalPath(format!("{} {}", label, s)));
    }
    let meta = fs::metadata(p).map_err(|_| CliError::NotFound(format!("{} {}", label, s)))?;
    if !meta.is_file() {
        // Allow directories only for --out (handled elsewhere). Inputs must be files.
        return Err(CliError::NotFound(format!("{} {}", label, s)));
    }
    Ok(())
}

/// Best-effort normalization to an absolute canonical path.
fn normalize_path(p: &PathBuf) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| {
        if p.is_absolute() {
            p.clone()
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(p)
        }
    })
}

/// Parse seed as u64: decimal or 0x-hex (1..=16 nybbles).
pub fn parse_seed_u64(s: &str) -> Result<u64, CliError> {
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        if rest.is_empty() || rest.len() > 16 || !rest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CliError::BadSeed(s.to_string()));
        }
        u64::from_str_radix(rest, 16).map_err(|_| CliError::BadSeed(s.to_string()))
    } else {
        if !s.chars().all(|c| c.is_ascii_digit()) {
            return Err(CliError::BadSeed(s.to_string()));
        }
        s.parse::<u64>().map_err(|_| CliError::BadSeed(s.to_string()))
    }
}

/// Lightweight manifest quick-check (no schema):
/// - exactly one of "ballots_path" XOR "ballot_tally_path" present
/// - "reg_path" and "params_path" present
/// - reject any "http://" or "https://" substrings anywhere
/// - if any `sha256` fields appear, ensure they look like 64-hex adjacent to the key (best-effort)
pub fn quick_check_manifest_bytes(bytes: &[u8]) -> Result<(), CliError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| CliError::ManifestQuick("manifest must be UTF-8 JSON".into()))?;

    // Very light sanity (not a JSON parser).
    let has_ballots = text.contains("\"ballots_path\"") || text.contains("'ballots_path'");
    let has_tally = text.contains("\"ballot_tally_path\"") || text.contains("'ballot_tally_path'");
    let has_reg = text.contains("\"reg_path\"") || text.contains("'reg_path'");
    let has_params = text.contains("\"params_path\"") || text.contains("'params_path'");

    if !(has_ballots ^ has_tally) {
        return Err(CliError::ManifestQuick(
            "exactly one of ballots_path | ballot_tally_path is required".into(),
        ));
    }
    if !has_reg {
        return Err(CliError::ManifestQuick("missing reg_path".into()));
    }
    if !has_params {
        return Err(CliError::ManifestQuick("missing params_path".into()));
    }
    if text.contains("http://") || text.contains("https://") {
        return Err(CliError::ManifestQuick(
            "URLs are not allowed (offline only)".into(),
        ));
    }

    // Best-effort check for 64-hex sha256 values when keys exist.
    // We scan for "sha256" tokens and look ahead for the next quoted string of 64 hex chars.
    let mut idx = 0usize;
    while let Some(pos) = text[idx..].find("sha256") {
        let start = idx + pos;
        // Find the next double-quote after the key:
        if let Some(q1_rel) = text[start..].find('"') {
            let q1 = start + q1_rel;
            if let Some(q2_rel) = text[q1 + 1..].find('"') {
                let q2 = q1 + 1 + q2_rel;
                let candidate = &text[q1 + 1..q2];
                if candidate.len() == 64 && candidate.chars().all(is_hex) {
                    // ok
                } else {
                    // Not strictly an error; only complain if it "looks like" a sha256 field with a non-hex or wrong length
                    return Err(CliError::ManifestQuick(
                        "sha256 field present but not 64-hex".into(),
                    ));
                }
                idx = q2 + 1;
                continue;
            }
        }
        // If we can't find a quoted value, give up silently (shape-only quick-check).
        idx = start + 6;
    }

    Ok(())
}

fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn has_scheme(s: &str) -> bool {
    // Reject any explicit scheme://
    // (Windows drive letters like C:\ are fine; UNC paths are fine.)
    s.contains("://")
      || s.starts_with("http:")  // defensive
      || s.starts_with("https:")
}

// ------------------------------
// Tests (light, compile-time only)
// ------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_decimal_ok() {
        assert_eq!(parse_seed_u64("12345").unwrap(), 12_345u64);
    }
    #[test]
    fn seed_hex_ok() {
        assert_eq!(
            parse_seed_u64("0xDEADBEEFCAFE1234").unwrap(),
            0xDEADBEEFCAFE1234u64
        );
    }
    #[test]
    fn seed_bad() {
        assert!(parse_seed_u64("0x").is_err());
        assert!(parse_seed_u64("0xZZ").is_err());
        assert!(parse_seed_u64("-1").is_err());
        assert!(parse_seed_u64("0x1234567890ABCDEF12").is_err()); // >16 nybbles
    }

    #[test]
    fn quick_check_manifest_minimal_tally() {
        let src = br#"{
            "reg_path":"reg.json",
            "params_path":"ps.json",
            "ballot_tally_path":"tly.json"
        }"#;
        assert!(quick_check_manifest_bytes(src).is_ok());
    }

    #[test]
    fn quick_check_manifest_exclusive_ballots_or_tally() {
        let both = br#"{"reg_path":"r","params_path":"p","ballots_path":"b","ballot_tally_path":"t"}"#;
        assert!(quick_check_manifest_bytes(both).is_err());

        let neither = br#"{"reg_path":"r","params_path":"p"}"#;
        assert!(quick_check_manifest_bytes(neither).is_err());
    }

    #[test]
    fn quick_check_manifest_reject_urls() {
        let src = br#"{
            "reg_path":"https://example.com/reg.json",
            "params_path":"ps.json",
            "ballot_tally_path":"tly.json"
        }"#;
        assert!(quick_check_manifest_bytes(src).is_err());
    }

    #[test]
    fn quick_check_manifest_sha256_shape() {
        let ok = br#"{
            "reg_path":"r","params_path":"p","ballot_tally_path":"t",
            "formula_manifest_sha256":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        }"#;
        assert!(quick_check_manifest_bytes(ok).is_ok());

        let bad = br#"{
            "reg_path":"r","params_path":"p","ballot_tally_path":"t",
            "formula_manifest_sha256":"NOT-HEX"
        }"#;
        assert!(quick_check_manifest_bytes(bad).is_err());
    }

    #[test]
    fn non_local_path_detection() {
        assert!(has_scheme("http://x"));
        assert!(has_scheme("scheme://x"));
        assert!(!has_scheme(r"C:\file.json"));
        assert!(!has_scheme(r"/tmp/file.json"));
    }

    #[test]
    fn normalize_path_best_effort() {
        let p = PathBuf::from("does/not/exist.txt");
        let abs = normalize_path(&p);
        assert!(abs.is_absolute());
    }
}
