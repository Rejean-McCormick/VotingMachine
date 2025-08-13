```
Pre-Coding Essentials (Component: vm_pipeline/src/load.rs, Version/FormulaID: VM-ENGINE v0) — 50/89

1) Goal & Success
Goal: Implement the pipeline’s LOAD stage for **normative runs**: read local artifacts, delegate schema/ID parsing to vm_io, enforce the canonical 3-file contract (registry + ballot_tally + parameter_set), and return a typed, deterministic bundle for downstream stages plus canonical digests for RunRecord/FID.
Success: Rejects any raw-ballots path; accepts exactly {REG + TLY + PS}; uses vm_io canonicalization for bytes & SHA-256; echoes IDs; confirms option ordering consistency; produces a compact LoadedStage structure.

2) Scope
In scope: Manifest resolution; vm_io loaders; enforcement of “tally-only” input contract; capture of canonical bytes/digests (including **normative manifest digest** for FID); light cross-checks already guaranteed by vm_io (re-assert critical ones).
Out of scope: Heavy validation (tree/cycles/magnitudes), algorithms, gates/frontier, labeling, report output.

3) Inputs → Outputs (normative & IDs)
Inputs (local only):
• manifest.json (optional but recommended) — must specify reg_path, params_path, **ballot_tally_path** (no ballots_path).  
• division_registry.json (REG:…), ballot_tally.json (TLY:…), parameter_set.json (PS:…); optional separate adjacency file (allowed by schema).
Outputs:
• LoadedStage { norm_ctx, digests, nm_digest, formula_id } where:
  – norm_ctx: NormContext { reg: DivisionRegistry, options: Vec<OptionItem>, params: Params, tallies: UnitTallies, ids: {reg_id, tally_id, param_set_id} }  
  – digests: InputDigests { reg_sha256, tally_sha256, params_sha256, adjacency_sha256?: Option<String> } (all 64-hex)  
  – nm_digest: String (64-hex digest of **Normative Manifest**)  
  – formula_id: String (hex FID computed from Normative Manifest per Annex-A)
All maps/lists are canonicalized by vm_io (Units by UnitId; Options by order_index then OptionId).

4) Entities/Tables (minimal, typed wrappers)
pub struct NormContext {
  pub reg: DivisionRegistry,
  pub options: Vec<OptionItem>,     // extracted from registry; canonical order
  pub params: Params,
  pub tallies: UnitTallies,
  pub ids: LoadedIds,               // { reg_id, tally_id, param_set_id }
}
pub struct InputDigests {
  pub reg_sha256: String,
  pub tally_sha256: String,
  pub params_sha256: String,
  pub adjacency_sha256: Option<String>,
}
pub struct LoadedStage {
  pub norm_ctx: NormContext,
  pub digests: InputDigests,
  pub nm_digest: Option<String>,    // present when a manifest was used
  pub formula_id: Option<String>,   // computed from Normative Manifest when available
}

5) Variables (used/observed here)
None evaluated for behavior. The stage only parses Params (VM-VAR map) via vm_io. Tie/RNG variables (050, 052) are not acted on here.

6) Functions (signatures only)
use vm_io::{
  manifest::{Manifest, load_manifest, resolve_paths, enforce_expectations},
  loader::{LoadedContext as IoLoaded, load_all_from_manifest, load_registry, load_params, load_tally},
  hasher::{sha256_file, formula_id_from_nm},
  canonical_json::to_canonical_bytes,
};
use vm_core::{ids::*, entities::*, variables::Params};

#[derive(Debug)]
pub enum LoadError {
  Io(String), Schema(String), Manifest(String), Hash(String), Contract(String),
}

pub struct LoadedIds { pub reg_id: RegId, pub tally_id: TallyId, pub param_set_id: ParamSetId }

// Entry points
pub fn load_normative_from_manifest<P: AsRef<std::path::Path>>(path: P)
  -> Result<LoadedStage, LoadError>;

pub fn load_normative_from_paths<P: AsRef<std::path::Path>>(
  reg_path: P, tally_path: P, params_path: P, adjacency_path: Option<P>
) -> Result<LoadedStage, LoadError>;

// Internals
fn ensure_manifest_contract(man: &Manifest) -> Result<(), LoadError>; // require ballot_tally_path; forbid ballots_path
fn to_norm_context(io: IoLoaded) -> Result<NormContext, LoadError>;   // assert tally source, lift into NormContext
fn compute_nm_fid_if_present(man: &Manifest, base: &std::path::Path)
  -> Result<(String, String), LoadError>;  // (nm_digest, formula_id)
fn collect_input_digests(paths: &ResolvedPaths) -> Result<InputDigests, LoadError>;

7) Algorithm Outline (stage flow)
A) From manifest (preferred)
  1. Read & parse manifest via vm_io::manifest::load_manifest → validate; **ensure exactly one source and it is ballot_tally_path** (ensure_manifest_contract).  
  2. resolve_paths → (base_dir-relative absolute paths).  
  3. Optionally enforce expectations (engine version/formula_id) before load.  
  4. vm_io::loader::load_all_from_manifest → IoLoaded (already schema-checked, IDs parsed, and **options/units canonicalized**).  
  5. Reject if IoLoaded is Ballots (should never happen after step 1).  
  6. compute_nm_fid_if_present: build **Normative Manifest** view (per Annex-A), canonicalize, sha256 → nm_digest; compute **formula_id** from NM.  
  7. collect_input_digests: sha256_file for reg, tally, params, adjacency?.  
  8. Wrap into LoadedStage { to_norm_context(io), digests, Some(nm_digest), Some(formula_id) }.

B) From explicit paths (no manifest)
  1. vm_io::loader::load_registry / load_params / load_tally in that order.  
  2. to_norm_context: lift into NormContext; options already canonical via vm_io.  
  3. collect_input_digests over provided files; nm_digest / formula_id set to None.

Light re-assertions (post vm_io):
  • ids.tally.reg_id == ids.reg_id (already ensured upstream; keep a guard).  
  • options are strictly ordered by (order_index, OptionId) and unique order_index (vm_io guarantees; assert).  
  • Unit magnitudes are ≥1 (shape check; deeper tree/graph checks deferred to VALIDATE stage).

8) State Flow
Pipeline: **LOAD** (this file) → VALIDATE → TABULATE → ALLOCATE → AGGREGATE → APPLY_RULES → (FRONTIER?) → (TIES?) → LABEL → BUILD_RESULT → BUILD_RUN_RECORD.  
On LoadError, pipeline aborts with a clear message; nothing else runs.

9) Determinism & Numeric Rules
• Offline only; no URLs; no network.  
• Canonical bytes/digests via vm_io (UTF-8, LF, sorted keys).  
• Input normalization (Units/Options ordering) is performed in vm_io loader for stable downstream hashing.  
• No floats or RNG here.

10) Edge Cases & Failure Policy
• Manifest provides ballots_path or omits ballot_tally_path ⇒ Contract error (normative runs require tallies).  
• reg_id mismatch between tally and registry ⇒ Contract error.  
• Digest hex not 64 or mismatch when verifying ⇒ Hash/Manifest error.  
• Oversize file / parse depth limits ⇒ Io/Schema errors surfaced from vm_io.  
• Adjacency path present: hash it; absence is allowed (inline adjacency may exist in registry).

11) Test Checklist (must pass)
• Happy path (manifest): reg + tally + params load; options/units canonicalized; nm_digest & formula_id computed; digests are 64-hex.  
• Happy path (paths): same artifacts without manifest; nm fields None; digests computed.  
• Rejection: manifest with ballots_path present ⇒ error; with neither ballots nor tally ⇒ error.  
• Cross-ref: tally.reg_id ≠ registry.id ⇒ error.  
• Determinism: loading the same inputs across OS/arch yields identical canonical bytes/digests and identical NormContext ordering.  
• Annex-B fixtures: all reference cases pass LOAD and proceed to VALIDATE without reordering drift.
```
