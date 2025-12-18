// crates/vm_cli/src/args.rs — Part 1/2
//
// Deterministic, offline CLI argument parsing surface (types + basic helpers).
// Part 2/2 will add parse_and_validate(), mode checks, filesystem checks,
// manifest quick-check, normalization, and tests.
//
// Spec-aligned rules (Docs 1–7 + Annexes A–C):
// - No networked paths (reject any scheme:// like http/https/file)
// - Exactly one of: --manifest  XOR  (--registry + --params + (--ballots XOR --tally))
// - Optional inputs: --adjacency, --autonomy
// - Output: --out dir, --render [json|html]*
// - Seed override is VM-VAR-052 (u64 decimal or 0x-hex up to 16 nybbles)
// - --validate_only performs load+schema checks without running pipeline

use clap::Parser;
use std::path::{Path, PathBuf};

/// Parsed CLI arguments (raw).
#[derive(Debug, Parser, Clone)]
#[command(
    name = "vm",
    disable_help_subcommand = true,
    about = "Offline, deterministic CLI for the VM engine"
)]
pub struct Args {
    // --- Mode selection ---
    /// Path to a manifest JSON describing inputs (mutually exclusive with explicit file flags).
    #[arg(long, conflicts_with_all = ["registry", "params", "ballots", "tally"])]
    pub manifest: Option<PathBuf>,

    // --- Explicit mode (when --manifest is not used) ---
    /// DivisionRegistry JSON path.
    #[arg(long)]
    pub registry: Option<PathBuf>,
    /// ParameterSet JSON path.
    #[arg(long)]
    pub params: Option<PathBuf>,
    /// Raw ballots JSON path (mutually exclusive with --tally).
    #[arg(long, conflicts_with = "tally")]
    pub ballots: Option<PathBuf>,
    /// Pre-aggregated BallotTally JSON path (mutually exclusive with --ballots).
    #[arg(long, conflicts_with = "ballots")]
    pub tally: Option<PathBuf>,

    // --- Optional inputs ---
    /// Adjacency JSON path (frontier/contiguity analysis).
    #[arg(long)]
    pub adjacency: Option<PathBuf>,
    /// Autonomy/protected-areas JSON path.
    #[arg(long)]
    pub autonomy: Option<PathBuf>,

    // --- Output & rendering ---
    /// Output directory (default: current directory).
    #[arg(long, default_value = ".")]
    pub out: PathBuf,
    /// Renderer(s) to emit. Choose up to 2 (json, html). Omit to skip rendering.
    #[arg(long, value_parser = ["json", "html"], num_args = 0..=2)]
    pub render: Vec<String>,

    // --- Determinism & control ---
    /// Tie RNG seed override (VM-VAR-052). Accepts decimal u64 or 0x-hex (≤16 hex digits).
    #[arg(long, value_parser = parse_seed)]
    pub seed: Option<u64>,

    /// Validate inputs only (load + schema/domain/ref/order), do not run the engine.
    #[arg(long)]
    pub validate_only: bool,

    /// Suppress non-essential stdout logs.
    #[arg(long)]
    pub quiet: bool,
}

/// Errors surfaced by argument parsing/validation.
/// Keep messages short/stable (handy for scripts/tests).
#[derive(Debug)]
pub enum CliError {
    BadCombo(&'static str),
    Missing(&'static str),
    BallotsTallyChoice,
    NonLocalPath(String),
    NotFound(String),
    BadSeed(String),
    ManifestQuick(&'static str),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CliError::*;
        match self {
            BadCombo(s) => write!(f, "invalid flag combination: {s}"),
            Missing(s) => write!(f, "missing required flag: {s}"),
            BallotsTallyChoice => write!(f, "both or neither of --ballots/--tally provided"),
            NonLocalPath(p) => write!(f, "path must be local file (no scheme): {p}"),
            NotFound(p) => write!(f, "file not found: {p}"),
            BadSeed(s) => write!(f, "invalid seed: {s}"),
            ManifestQuick(s) => write!(f, "manifest quick-check failed: {s}"),
        }
    }
}
impl std::error::Error for CliError {}

/// Seed parser for VM-VAR-052: decimal u64 or 0x-hex (1..=16 nybbles).
pub fn parse_seed(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty seed".into());
    }
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        if rest.is_empty() || rest.len() > 16 || !rest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("hex seed must be 1..16 hex digits".into());
        }
        u64::from_str_radix(rest, 16).map_err(|_| "hex seed out of range".into())
    } else {
        s.parse::<u64>().map_err(|_| "decimal seed must be a valid u64".into())
    }
}

/// Reject any explicit URI scheme (e.g., http://, https://, file://).
#[inline]
fn has_scheme(s: &str) -> bool {
    let lower = s.trim().to_ascii_lowercase();
    lower.contains("://") || lower.starts_with("http:") || lower.starts_with("https:") || lower.starts_with("file:")
}

/// Ensure a provided path string is local (no scheme); path existence is checked later.
#[inline]
fn ensure_local_path(p: &Path) -> Result<(), CliError> {
    if let Some(s) = p.to_str() {
        if has_scheme(s) {
            return Err(CliError::NonLocalPath(s.to_string()));
        }
    }
    Ok(())
}

/// Iterate over all path-like flags (including `--out`) for quick scheme checks.
fn iter_all_paths(args: &Args) -> impl Iterator<Item = &Path> {
    [
        args.manifest.as_deref(),
        args.registry.as_deref(),
        args.params.as_deref(),
        args.ballots.as_deref(),
        args.tally.as_deref(),
        args.adjacency.as_deref(),
        args.autonomy.as_deref(),
        Some(args.out.as_path()),
    ]
    .into_iter()
    .flatten()
}

// --------------------------
// Part 2/2 will provide:
// - parse_and_validate()
// - validate_manifest_mode() / validate_explicit_mode()
// - ensure_local_exists() / normalize_path()
// - quick_check_manifest_bytes()
// - unit tests
// --------------------------
// crates/vm_cli/src/args.rs — Part 2/2
//
// parse_and_validate(), mode checks, filesystem checks, manifest quick-check,
// normalization helpers, and unit tests.

use std::{
    env,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Entry point used by main.rs
pub fn parse_and_validate() -> Result<Args, CliError> {
    let mut args = Args::parse();

    // Reject schemes for all provided paths (including --out)
    for p in iter_all_paths(&args) {
        ensure_local_path(p)?;
    }

    // Validate modes + existence, then normalize paths
    if args.manifest.is_some() {
        validate_manifest_mode(&args)?;
        args.manifest = args.manifest.take().map(|p| normalize_path(&p));
    } else {
        validate_explicit_mode(&args)?;
        args.registry = args.registry.take().map(|p| normalize_path(&p));
        args.params = args.params.take().map(|p| normalize_path(&p));
        args.ballots = args.ballots.take().map(|p| normalize_path(&p));
        args.tally = args.tally.take().map(|p| normalize_path(&p));
        args.adjacency = args.adjacency.take().map(|p| normalize_path(&p));
        args.autonomy = args.autonomy.take().map(|p| normalize_path(&p));
    }

    // Normalize output directory even if it doesn't exist yet
    args.out = normalize_path(&args.out);

    Ok(args)
}

/// Manifest mode validation: require local file and quick-check minimal shape.
fn validate_manifest_mode(a: &Args) -> Result<(), CliError> {
    let path = a
        .manifest
        .as_ref()
        .ok_or(CliError::Missing("--manifest"))?;

    ensure_local_exists(path, "--manifest")?;

    // Read bounded bytes for quick-check (no JSON parse here)
    const MAX_BYTES: usize = 4 * 1024 * 1024;
    let mut f = fs::File::open(path)
        .map_err(|_| CliError::NotFound(format!("--manifest {}", path.display())))?;
    let mut buf = Vec::new();
    f.take(MAX_BYTES as u64)
        .read_to_end(&mut buf)
        .map_err(|_| CliError::ManifestQuick("unable to read manifest file"))?;

    quick_check_manifest_bytes(&buf)
}

/// Explicit mode validation: require registry+params and exactly one of ballots XOR tally.
/// Also check existence for all provided input files (adjacency/autonomy optional).
fn validate_explicit_mode(a: &Args) -> Result<(), CliError> {
    let reg = a
        .registry
        .as_ref()
        .ok_or(CliError::Missing("--registry"))?;
    let par = a.params.as_ref().ok_or(CliError::Missing("--params"))?;

    // Exactly one of ballots XOR tally
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

/// Ensure a path is local (no scheme) and exists as a regular file.
fn ensure_local_exists(p: &Path, label: &'static str) -> Result<(), CliError> {
    ensure_local_path(p)?;
    let meta = fs::metadata(p).map_err(|_| CliError::NotFound(format!("{label} {}", p.display())))?;
    if !meta.is_file() {
        return Err(CliError::NotFound(format!("{label} {}", p.display())));
    }
    Ok(())
}

/// Best-effort normalization to an absolute path.
/// If canonicalize fails (e.g., path doesn't exist yet), produce an absolute path relative to CWD.
fn normalize_path(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| {
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(p)
        }
    })
}

/// Lightweight manifest quick-check (no JSON parsing):
/// - exactly one of "ballots_path" XOR "ballot_tally_path" present
/// - "reg_path" and "params_path" present
/// - reject any "http://", "https://", or "file://" substrings anywhere
/// - if any `sha256`-like fields appear, best-effort ensure they look like 64-hex
pub fn quick_check_manifest_bytes(bytes: &[u8]) -> Result<(), CliError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| CliError::ManifestQuick("manifest must be UTF-8"))?;

    let has_ballots = text.contains("\"ballots_path\"") || text.contains("'ballots_path'");
    let has_tally = text.contains("\"ballot_tally_path\"") || text.contains("'ballot_tally_path'");
    let has_reg = text.contains("\"reg_path\"") || text.contains("'reg_path'");
    let has_params = text.contains("\"params_path\"") || text.contains("'params_path'");

    if !(has_ballots ^ has_tally) {
        return Err(CliError::ManifestQuick(
            "exactly one of ballots_path | ballot_tally_path is required",
        ));
    }
    if !has_reg {
        return Err(CliError::ManifestQuick("missing reg_path"));
    }
    if !has_params {
        return Err(CliError::ManifestQuick("missing params_path"));
    }
    if text.contains("http://") || text.contains("https://") || text.contains("file://") {
        return Err(CliError::ManifestQuick("URLs are not allowed (offline only)"));
    }

    // Very light 64-hex scan after occurrences of "sha256"
    let mut i = 0usize;
    while let Some(pos) = text[i..].find("sha256") {
        let start = i + pos;
        // Find the next quoted string after the key
        if let Some(q1_rel) = text[start..].find('"') {
            let q1 = start + q1_rel;
            if let Some(q2_rel) = text[q1 + 1..].find('"') {
                let q2 = q1 + 1 + q2_rel;
                let candidate = &text[q1 + 1..q2];
                if candidate.len() == 64 && candidate.chars().all(|c| c.is_ascii_hexdigit()) {
                    // looks fine
                } else {
                    return Err(CliError::ManifestQuick("sha256 field present but not 64-hex"));
                }
                i = q2 + 1;
                continue;
            }
        }
        // if no quoted value found, just move past "sha256"
        i = start + 6;
    }

    Ok(())
}

// ------------------------------
// Tests (light, compile-time only)
// ------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_parser_decimal_and_hex() {
        assert_eq!(parse_seed("42").unwrap(), 42u64);
        assert_eq!(parse_seed("0x2A").unwrap(), 42u64);
        assert!(parse_seed("0x").is_err());
        assert!(parse_seed("0xFFFFFFFFFFFFFFFFF").is_err()); // 17 nybbles
        assert!(parse_seed("-1").is_err());
    }

    #[test]
    fn quick_check_manifest_ok_tally() {
        let src = br#"{
            "reg_path":"reg.json",
            "params_path":"ps.json",
            "ballot_tally_path":"tly.json"
        }"#;
        assert!(quick_check_manifest_bytes(src).is_ok());
    }

    #[test]
    fn quick_check_manifest_requires_exactly_one() {
        let both = br#"{"reg_path":"r","params_path":"p","ballots_path":"b","ballot_tally_path":"t"}"#;
        assert!(quick_check_manifest_bytes(both).is_err());

        let neither = br#"{"reg_path":"r","params_path":"p"}"#;
        assert!(quick_check_manifest_bytes(neither).is_err());
    }

    #[test]
    fn quick_check_manifest_rejects_urls() {
        let src = br#"{
            "reg_path":"https://x/reg.json",
            "params_path":"ps.json",
            "ballot_tally_path":"tly.json"
        }"#;
        assert!(quick_check_manifest_bytes(src).is_err());
    }

    #[test]
    fn ensure_local_path_rejects_schemes() {
        assert!(super::ensure_local_path(Path::new("http://x")).is_err());
        assert!(super::ensure_local_path(Path::new("file://C:/x.json")).is_err());
        assert!(super::ensure_local_path(Path::new("https://x/y.json")).is_err());
        assert!(super::ensure_local_path(Path::new(r"C:\local\file.json")).is_ok());
        assert!(super::ensure_local_path(Path::new("/tmp/file.json")).is_ok());
    }

    #[test]
    fn normalize_path_returns_absolute() {
        let p = PathBuf::from("does/not/exist.txt");
        let abs = normalize_path(&p);
        assert!(abs.is_absolute());
    }
}
