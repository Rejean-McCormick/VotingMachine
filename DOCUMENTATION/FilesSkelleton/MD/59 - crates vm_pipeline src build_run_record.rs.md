````
Pre-Coding Essentials (Component: crates/vm_pipeline/src/build_run_record.rs, Version/FormulaID: VM-ENGINE v0) — 59/89

1) Goal & Success
Goal: Assemble a **RunRecord** that proves reproducibility: exact inputs (IDs + 64-hex digests), engine (vendor/name/version/build), FormulaID + NM digest, determinism policy (tie_policy; rng_seed iff random), UTC timestamps, and pointers to produced artifacts (Result + optional FrontierMap).
Success: Fully aligned to reference ADJUSTMENTS:
• Records **formula_id** and **formula_manifest_sha256** (NM digest).
• `engine{vendor,name,version,build}` present.
• Determinism keeps only `tie_policy`; include `rng_seed` **only if** policy==random (no “deterministic_order_key”).
• Inputs section carries canonical **64-hex** digests for all loaded artifacts.
• **ties[]** are recorded here (Result carries none).

2) Scope
In scope: Populate RunRecord fields from pipeline context and prior artifacts; compute/accept content digests for outputs; form RUN id from canonical content (idless hash) + started_utc.
Out of scope: Running algorithms, schema I/O wiring (vm_io handles canonical JSON + hashing).

3) Inputs → Outputs
Inputs (from pipeline and vm_io):
• ids: `reg_id`, `parameter_set_id`, exactly one of `ballots_id` or `ballot_tally_id`, optional `manifest_id`
• digests: `BTreeMap<String, Hex64>` for every input path (registry, params, ballots/tally, adjacency, manifest, etc.)
• engine: `{ vendor, name, version, build }`
• formula: `formula_id`, `formula_manifest_sha256`
• determinism: `tie_policy`, optional `rng_seed` (when random)
• outputs: `result_id`, `result_sha256`, optional `frontier_map_id`, optional `frontier_map_sha256`
• timestamps: `started_utc`, `finished_utc` (ISO-8601 Z)
• ties: `Vec<TieEvent>` (full entries; pipeline aggregates crumbs)
Output:
• `RunRecord` with `id = RUN:<started_utc-IDfmt>-<short-hash>`

4) Entities/Tables (minimal)
```rust
pub struct EngineMeta { pub vendor: String, pub name: String, pub version: String, pub build: String }
pub enum TiePolicy { StatusQuo, Deterministic, Random } // rng_seed recorded only if Random
pub struct InputRefs {
  pub manifest_id: Option<String>,
  pub reg_id: RegId,
  pub parameter_set_id: ParamSetId,
  pub ballots_id: Option<String>,
  pub ballot_tally_id: Option<TallyId>,
  pub digests: BTreeMap<String, String>, // path → sha256 (64-hex)
}
pub struct OutputRefs {
  pub result_id: ResultId,
  pub result_sha256: String,             // 64-hex
  pub frontier_map_id: Option<FrontierId>,
  pub frontier_map_sha256: Option<String>, // required iff frontier_map_id
}
pub struct Determinism { pub tie_policy: TiePolicy, pub rng_seed_hex64: Option<String> }
pub struct RunRecordDoc { /* mirrors schema; no serialization here */ }
pub struct TieEvent { /* context, candidates, policy, seed?, winner */ }
````

5. Variables
   None new; use values computed upstream (FID/NM digest, digests of inputs/outputs).

6. Functions (signatures only)

```rust
/// Build the RunRecord content; ID is computed from canonical bytes (without `id`) + started_utc.
pub fn build_run_record(
  engine: &EngineMeta,
  formula_id: &str,
  formula_manifest_sha256: &str,
  inputs: &InputRefs,
  determinism: &Determinism,
  outputs: &OutputRefs,
  ties: &[TieEvent],
  started_utc: &str,
  finished_utc: &str,
) -> Result<RunRecordDoc, BuildRunRecordError>;

fn validate_utc(ts: &str) -> Result<(), BuildRunRecordError>; // "YYYY-MM-DDTHH:MM:SSZ"
fn validate_hex64(s: &str) -> Result<(), BuildRunRecordError>;
fn check_inputs_coherence(inputs: &InputRefs) -> Result<(), BuildRunRecordError>; // exactly one of ballots_id|ballot_tally_id
fn check_outputs_coherence(outputs: &OutputRefs) -> Result<(), BuildRunRecordError>; // frontier sha iff id
fn check_determinism(d: &Determinism) -> Result<(), BuildRunRecordError>; // rng_seed required iff Random
fn id_friendly_timestamp(ts: &str) -> String; // replace ":" with "-"
fn compute_id_short_hash(canon_without_id_sha256: &str, len: usize) -> String; // typically 12–16
```

7. Algorithm Outline

8. Validate inputs: UTC formats, hex64 digests, one-of ballots/tally, frontier sha iff id, rng\_seed iff Random.

9. Assemble a **temporary** struct with **no `id` field**; include:
   • `timestamp_utc = finished_utc`
   • `engine{vendor,name,version,build}`
   • `formula_id`, `formula_manifest_sha256`
   • `inputs{manifest_id?, reg_id, parameter_set_id, ballots_id? | ballot_tally_id?, digests{...}}`
   • `determinism{tie_policy, rng_seed?}` (seed only if Random)
   • `outputs{result_id, result_sha256, frontier_map_id?, frontier_map_sha256?}`
   • `ties: Vec<TieEvent>` (full entries; stable order)

10. Canonicalize to bytes (sorted keys, LF) **without `id`**, hash SHA-256 (vm\_io).

11. Form `run_id = format!("RUN:{}-{}", id_friendly_timestamp(started_utc), short_hash)`.

12. Produce `RunRecordDoc` with `id = run_id`.

13. Return doc; caller persists via vm\_io canonical writer.

14. State Flow
    BUILD\_RESULT → **BUILD\_RUN\_RECORD** → write artifacts (vm\_io). RunRecord references Result (and optional FrontierMap) by ID + sha256.

15. Determinism & Numeric Rules
    • No wall-clock reads here; timestamps are provided.
    • Canonical JSON of the **idless** struct determines the hash; short\_hash is a prefix of that digest.
    • Stable ordering for `ties[]` and all maps (BTree) before serialization.

16. Edge Cases & Failure Policy
    • Missing rng\_seed while policy=Random ⇒ error.
    • Both ballots\_id and ballot\_tally\_id present (or neither) ⇒ error.
    • frontier\_map\_id present without sha256 ⇒ error.
    • Any digest not 64-hex ⇒ error.
    • Invalid UTC ⇒ error.

17. Test Checklist (must pass)
    • Engine block contains vendor/name/version/build.
    • `formula_id` and `formula_manifest_sha256` are present.
    • Inputs have exactly one of ballots\_id|ballot\_tally\_id; all listed digests are 64-hex.
    • Determinism: rng\_seed only serialized when policy=Random.
    • Ties recorded in RunRecord (Result contains none).
    • Frontier pointer & sha coherency enforced.
    • ID format `RUN:YYYY-MM-DDTHH-MM-SSZ-<short>`; two builds with identical inputs produce identical canonical bytes and short\_hash across OS.

```
```
