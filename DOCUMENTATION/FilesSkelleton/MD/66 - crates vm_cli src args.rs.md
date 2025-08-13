````md
Pre-Coding Essentials (Component: crates/vm_cli/src/args.rs, Version/FormulaID: VM-ENGINE v0) — 66/89

1) Goal & Success
Goal: Define a small, deterministic CLI surface that maps cleanly to the fixed pipeline and offline policy.
Success: Parsing is OS-agnostic and side-effect free; early cross-field validation catches bad combinations; accepts only local files; enforces “exactly one of ballots|tally” and “manifest XOR explicit paths”; optional RNG seed override is a u64 (decimal or 0x-hex) aligned with vm_core::rng.

2) Scope
In scope: clap-based argument struct; mutual exclusion/requirement validation; path normalization; lightweight seed parsing (u64); quick manifest pre-checks (shape only, no schema I/O).
Out of scope: Running the pipeline; schema validation or hashing; network access (forbidden).

3) Inputs → Outputs
Inputs (flags):
- Manifest mode: `--manifest <path>`
- Explicit mode: `--registry <path> --params <path>` and exactly one of `--ballots <path>` | `--tally <path>`
- Optionals: `--adjacency <path>` (frontier), `--autonomy <path>` (packages), `--out <dir>` (default: `.`)
- Rendering: `--render json|html` (0..=2 values)
- Determinism: `--seed <u64|0xHEX>` (optional override for VM-VAR-032 Random); `--validate-only`; `--quiet`
Outputs: Validated `Args` with normalized local paths, chosen renderers, and an optional `seed: Option<u64>` forwarded to main.

4) Entities/Tables (minimal)
N/A (argument model only).

5) Variables (only ones used here)
CLI does not calculate VM-VARs; it may override VM-VAR-033 (tie seed) via `--seed` (u64) to stay aligned with `TieRng::seed_from_u64`.

6) Functions (signatures only)
```rust
use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
  // Mode selection
  #[arg(long, conflicts_with_all=["registry","params","ballots","tally"])]
  pub manifest: Option<PathBuf>,

  // Explicit mode
  #[arg(long)] pub registry: Option<PathBuf>,
  #[arg(long)] pub params:   Option<PathBuf>,
  #[arg(long, conflicts_with="tally")] pub ballots: Option<PathBuf>,
  #[arg(long, conflicts_with="ballots")] pub tally:  Option<PathBuf>,

  // Optional inputs
  #[arg(long)] pub adjacency: Option<PathBuf>,
  #[arg(long)] pub autonomy:  Option<PathBuf>,

  // Output & rendering
  #[arg(long, default_value = ".")] pub out: PathBuf,
  #[arg(long, value_parser=["json","html"], num_args=0..=2)]
  pub render: Vec<String>,

  // Determinism & control
  /// Optional override for VM-VAR-033; accepts decimal u64 or 0x-prefixed hex (≤16 hex digits).
  #[arg(long)] pub seed: Option<String>,
  #[arg(long)] pub validate_only: bool,
  #[arg(long)] pub quiet: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum CliError {
  #[error("invalid flag combination: {0}")] BadCombo(&'static str),
  #[error("missing required flag: {0}")] Missing(&'static str),
  #[error("both or neither of --ballots/--tally provided")] BallotsTallyChoice,
  #[error("path must be local file (no scheme): {0}")] NonLocalPath(String),
  #[error("file not found: {0}")] NotFound(String),
  #[error("invalid seed: {0}")] BadSeed(String),
  #[error("manifest quick-check failed: {0}")] ManifestQuick(String),
}

pub fn parse_and_validate() -> Result<Args, CliError>;

fn validate_manifest_mode(a: &Args) -> Result<(), CliError>;
fn validate_explicit_mode(a: &Args) -> Result<(), CliError>;
fn ensure_local_exists(p: &PathBuf, label: &'static str) -> Result<(), CliError>;
fn normalize_path(p: &PathBuf) -> PathBuf;

fn parse_seed_u64(s: &str) -> Result<u64, CliError>; // decimal or 0xHEX → u64
fn quick_check_manifest_bytes(bytes: &[u8]) -> Result<(), CliError>; // one-of ballots|tally; hex digests shape if present (no schema)
````

7. Algorithm Outline (implementation plan)

* Parse with `clap::Parser::parse()`.
* Enforce **mode**:

  * If `--manifest`: reject explicit flags; ensure file exists; read bytes (size cap) and **quick-check**: exactly one of `ballots_path|ballot_tally_path` strings, `reg_path` & `params_path` present, reject `http(s)://` prefixes, any digests listed look like `[a-f0-9]{64}` if present. Do **not** validate schema here.
  * Else explicit mode: require `--registry` **and** `--params`; require exactly one of `--ballots` XOR `--tally`. For each provided path: reject URL schemes; ensure file exists.
* Normalize all paths (`canonicalize` best-effort; fall back to `absolutize` join with CWD; keep them local UTF-8 where possible).
* Seed:

  * If `--seed` present: accept decimal `\d+` into `u64`, or `0x[0-9A-Fa-f]{1,16}`; error otherwise. (This aligns with `TieRng::seed_from_u64` and `pipeline::resolve_ties`.)
* Rendering:

  * Default: `["json"]` if empty; allow both `json` and `html`.
* Return validated `Args`.

8. State Flow
   `args.rs` → parsed `Args` → `main.rs` orchestrates fixed pipeline:
   LOAD → VALIDATE → TABULATE → ALLOCATE → AGGREGATE → APPLY\_DECISION\_RULES → (MAP\_FRONTIER?) → RESOLVE\_TIES → LABEL → BUILD\_RESULT → BUILD\_RUN\_RECORD.

9. Determinism & Numeric Rules

* No clocks, no RNG here. Optional `--seed` is a literal, not generated.
* Paths are local only; no network or remote schemes.
* All booleans/options map directly to pipeline behavior; no hidden defaults other than render default.

10. Edge Cases & Failure Policy

* Both or neither of `--ballots/--tally` ⇒ `CliError::BallotsTallyChoice`.
* `--manifest` together with any explicit path ⇒ `CliError::BadCombo`.
* URL-like paths (`http://`, `https://`) ⇒ `CliError::NonLocalPath`.
* Missing files ⇒ `CliError::NotFound`.
* Seed not parseable as u64 (decimal or short 0x-hex) ⇒ `CliError::BadSeed`.
* Manifest quick-check sees both or neither ballots/tally, or bad 64-hex digest strings ⇒ `CliError::ManifestQuick`.
* Do **not** attempt schema validation; that belongs to `vm_io`.

11. Test Checklist (must pass)

* `vm --help` shows flags and exits 0 (clap derives).
* Manifest mode:

  * Minimal Annex-B-style manifest passes quick-check; permutations with both/none ballots|tally fail.
  * Reject `http(s)://` paths inside manifest.
* Explicit mode:

  * Requires `--registry --params` and exactly one of `--ballots | --tally`.
  * Rejects URL-like paths; missing files error out.
* Seed:

  * Accepts `--seed 1234567890` and `--seed 0xDEADBEEFCAFE1234`; rejects non-numeric, over-wide hex (>16 hex digits), or negative.
* Determinism:

  * Same argv on different OS yield identical `Args` content (after normalization) modulo platform absolute prefixes; logic decisions identical.

```
```
