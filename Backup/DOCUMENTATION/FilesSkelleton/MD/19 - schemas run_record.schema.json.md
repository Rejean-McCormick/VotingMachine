<!-- Converted from: 19 - schemas run_record.schema.json, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.025782Z -->

```
Pre-Coding Essentials (Component: schemas/run_record.schema.json, Version/FormulaID: VM-ENGINE v0) — 19/89
1) Goal & Success
Goal: JSON Schema for RunRecord — the signed/attested provenance of one execution.
Success: Validates RUN: ID and UTC timestamp; records engine/version/FormulaID, input IDs + digests, tie/RNG policy, platform info, and pointers to produced artifacts (RES:, optional FR:). Strict (additionalProperties:false), integers/booleans only where applicable.
2) Scope
In scope: Immutable audit envelope for a single run: who/what/when, exact inputs, policies affecting outcomes, and output references.
Out of scope: The Result content itself (lives in result.json), Frontier geometry (own file), presentation/HTML.
3) Inputs → Outputs
Inputs: Manifest + loaded artifacts (Registry, ParameterSet, Ballots or Tally, optional Adjacency), engine metadata.
Output: One run_record.json per run; used by reports and determinism tests.
4) Entities/Fields (schema shape to encode)
Root
id (required, string) — RUN:<timestamp>-<short-hash> (hash over canonical input bytes + engine metadata)
timestamp_utc (required, string, ISO-8601 Z) — e.g., 2025-08-11T14:07:00Z
engine (required, object)
engine_version (string) — semantic version or commit hash
formula_id (string) — hex fingerprint of rule set
formula_manifest_sha256 (string, 64-hex) — digest of the normative manifest used to compute formula_id
inputs (required, object)
manifest_id (string) — MAN:… if a manifest was used
reg_id (string) — REG:…
parameter_set_id (string) — PS:…
exactly one of:
ballots_id (string) — dataset label if raw ballots had an ID (optional in some pipelines)
ballot_tally_id (string) — TLY:…
adjacency_present (boolean)
digests (object) — map <relative_path> → { sha256: <hex64> } for every input file loaded
policy (required, object)
tie_policy (string enum) — status_quo | deterministic_order | random
deterministic_order_key (string, const option_order_index when used)
rng_seed (string, 64-hex; required iff tie_policy = "random")
platform (required, object)
os (string) — windows|macos|linux
arch (string) — x86_64|aarch64 etc.
rustc_version (string)
build_profile (string) — debug|release
outputs (required, object)
result_id (string) — RES:…
result_sha256 (string, 64-hex)
frontier_map_id (string) — FR:… if produced
frontier_map_sha256 (string, 64-hex) — required iff frontier_map_id present
tie_log_summary (object) — optional quick stats:
deterministic_ties (integer ≥ 0)
randomized_ties (integer ≥ 0)
notes (string, optional)
5) Variables (validators to embed in schema)
6) Functions
(Schema only.)
7) Algorithm Outline (schema authoring steps)
$schema = JSON Schema 2020-12; set $id.
$defs: Hex64, RunId, each ID regex, DigestEntry.
Root: type: object, required: ["id","timestamp_utc","engine","inputs","policy","platform","outputs"], additionalProperties:false.
Encode one-of constraint inside inputs: exactly one of ballots_id or ballot_tally_id must be present.
Add if/then for policy.tie_policy = "random" ⇒ require rng_seed (hex64).
Require frontier_map_sha256 iff frontier_map_id present.
For digests, use additionalProperties schema { type:"object", required:["sha256"], properties:{ sha256:{pattern: hex64} }, additionalProperties:false }.
Keep all nested objects strict with additionalProperties:false.
8) State Flow
After pipeline builds Result (and optional FrontierMap), engine assembles RunRecord, computing digests and embedding IDs. Report reads RunRecord (snapshot of VM-VARs is resolved via PS:; tie summary aids audit).
9) Determinism & Numeric Rules
RUN: id derived from canonical bytes (inputs + engine metadata + FormulaID).
All digests SHA-256 (hex) over canonical bytes (UTF-8, LF, sorted keys).
RNG used only if tie_policy="random"; seed recorded here for reproducibility.
10) Edge Cases & Failure Policy
Missing rng_seed while tie_policy="random" ⇒ schema fail.
Both ballots_id and ballot_tally_id present (or neither) ⇒ schema fail.
Non-UTC timestamp or non-ISO format ⇒ schema fail.
frontier_map_id present without frontier_map_sha256 ⇒ schema fail.
Digests map with non-hex value ⇒ schema fail.
11) Test Checklist (must pass)
Happy path (tally): reg_id, parameter_set_id, ballot_tally_id, result_id, all digests hex64, UTC timestamp → pass.
Random ties: tie_policy="random" with valid rng_seed → pass; omit seed → fail.
Deterministic ties: tie_policy="deterministic_order" with deterministic_order_key="option_order_index" → pass.
Frontier present: includes frontier_map_id and matching sha → pass; omit sha → fail.
ID shapes: malformed RUN: or RES: rejected by regex.
```
