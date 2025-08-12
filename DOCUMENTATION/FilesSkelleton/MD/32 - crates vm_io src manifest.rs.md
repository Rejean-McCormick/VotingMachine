<!-- Converted from: 32 - crates vm_io src manifest.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.380446Z -->

```
Pre-Coding Essentials (Component: crates/vm_io/src/manifest.rs, Version/FormulaID: VM-ENGINE v0) — 32/89
1) Goal & Success
Goal: Parse, validate, and resolve the run manifest into concrete, local file paths and expectations for a deterministic offline run.
Success: Given a manifest.json, return a typed Manifest + ResolvedPaths with exactly one ballots source selected, no URLs, paths resolved against the manifest’s directory, and (if present) expectations/digests verified.
2) Scope
In scope: JSON parse → schema check → typed struct; relative-path resolution; “one-of” ballots vs tally; optional expect{formula_id, engine_version} check; optional digests verification; precise error mapping.
Out of scope: Reading the target files (done in loader.rs), hashing bytes (in hasher.rs), canonical JSON writing.
3) Inputs → Outputs
Inputs: Path to manifest.json on disk.
Outputs:
Manifest (typed view of fields).
ResolvedPaths (absolute or base-relative normalized paths for registry, params, ballots or tally, optional adjacency).
Optional checks run: expectations verified; digests verified (if provided).
4) Entities/Tables
5) Variables (module knobs)
6) Functions (signatures only)
rust
CopyEdit
// Public API
pub struct Manifest {
pub id: String,                    // MAN:…
pub reg_path: String,
pub params_path: String,
pub ballots_path: Option<String>,
pub ballot_tally_path: Option<String>,
pub adjacency_path: Option<String>,
pub expect: Option<Expect>,
pub digests: Option<BTreeMap<String, DigestEntry>>,
}

pub struct Expect {
pub formula_id: Option<String>,
pub engine_version: Option<String>,
}

pub struct DigestEntry { pub sha256: String }

pub enum BallotSource { Ballots, Tally }

pub struct ResolvedPaths {
pub base_dir: camino::Utf8PathBuf,
pub reg: camino::Utf8PathBuf,
pub params: camino::Utf8PathBuf,
pub ballots: Option<camino::Utf8PathBuf>,
pub tally: Option<camino::Utf8PathBuf>,
pub adjacency: Option<camino::Utf8PathBuf>,
pub source: BallotSource,
}

// Top-level
pub fn load_manifest<P: AsRef<std::path::Path>>(path: P) -> Result<Manifest, IoError>;
pub fn validate_manifest(man: &Manifest) -> Result<(), IoError>;
pub fn resolve_paths<P: AsRef<std::path::Path>>(manifest_file: P, man: &Manifest)
-> Result<ResolvedPaths, IoError>;
pub fn enforce_expectations(man: &Manifest, engine_version: &str, formula_id_hex: &str)
-> Result<(), IoError>;
pub fn verify_digests(paths: &ResolvedPaths, digests: &BTreeMap<String, DigestEntry>)
-> Result<(), IoError>;

7) Algorithm Outline (implementation plan)
Read & parse
Read file (cap mf.max_bytes).
Parse into serde_json::Value; map parse errors to IoError::Json { pointer, msg } (pointer “/” if not available).
Schema validation
Validate against schemas/manifest.schema.json when mf.strict_schema.
Fail with IoError::Schema on first violation (carry JSON Pointer).
To typed Manifest
Deserialize to the Manifest struct; additionalProperties: false enforced by schema, not by struct.
Quick surface checks:
Exactly one of ballots_path or ballot_tally_path is Some.
All path strings do not start with http:// or https:// when mf.reject_urls.
Resolve paths
base_dir = manifest_file.parent().unwrap_or(".").
For each present path, join to base_dir using camino::Utf8PathBuf, then normalize (.normalize() or manual dot-segment removal).
If mf.allow_parent_traversal == 0, reject any resolved path that escapes base_dir after normalization.
Do not require files to exist here (loader does that), but you may optionally metadata to give earlier errors.
Decide source
Set BallotSource::Ballots if ballots_path.is_some() else Tally.
Expectations (optional)
If expect.formula_id present, compare to provided formula_id_hex (case-insensitive hex compare).
If expect.engine_version present, exact string compare.
Mismatch → IoError::Manifest("expectation mismatch: …").
Digests (optional)
If digests present and mf.verify_digests, compute SHA-256 for each listed path as written in the manifest (relative to base). Compare hex (case-insensitive).
Any mismatch → IoError::Manifest("digest mismatch for <path>").
Return
On success, return Manifest + ResolvedPaths (or ResolvedPaths only if the loader calls resolve_paths directly).
8) State Flow
vm_cli/vm_pipeline calls:
load_manifest → validate_manifest.
resolve_paths → enforce_expectations (with engine/formula data).
hand ResolvedPaths to loader.rs to actually read Registry/Params and Ballots or Tally.
9) Determinism & Numeric Rules
Determinism supported by forcing local files and optional digest checks prior to execution.
No numeric operations here.
10) Edge Cases & Failure Policy
Both ballots_path and ballot_tally_path present (or neither) → error.
Any *_path begins with http(s):// → error.
Path normalization escapes base when traversal disallowed → error.
digests map includes a path not present in the manifest → ignore or warn? Choose error for strictness.
Hex digest not 64-hex → error.
expect provided but engine/formula not passed to enforce_expectations → caller bug; document in API contract.
11) Test Checklist (must pass)
Happy (tally): reg_path, params_path, ballot_tally_path, no URLs → parse/validate/resolve succeeds; source=Tally.
Happy (ballots): same but with ballots_path; source=Ballots.
Both/None ballot sources: validate_manifest fails with clear message.
URL rejection: any http(s)://… in paths → fail.
Traversal: with mf.allow_parent_traversal=0, path ../outside.json rejects; with it on, resolving succeeds.
Expectations: mismatch in formula_id or engine_version → fail; match → pass.
Digests: correct hex passes; wrong hex or mismatched file content → fail; non-hex → fail.
Determinism: resolving the same manifest on different OS yields identical normalized Utf8PathBuf strings (relative to base).
```
