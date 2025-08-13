Pre-Coding Essentials (Component: schemas/manifest.schema.json, Version/FormulaID: VM-ENGINE v0) — 17/89

1) Goal & Success
Goal: JSON Schema for a run/test manifest that names the three canonical inputs and optional expectations/digests for a deterministic, offline run.
Success: Requires exactly the three inputs (registry, ballot_tally, params); rejects any “raw ballots” path; disallows URLs; validates optional SHA-256 digests as lowercase 64-hex.

2) Scope
In scope (normative for this schema):
- Paths to inputs: reg_path, ballot_tally_path, params_path (all required).
- Optional expectations block (formula_id, engine_version) and optional digests map {path → {sha256}}.
Out of scope: Algorithm behavior; computing/validating FID beyond shape; cross-file semantic checks (done by loader/pipeline).

3) Inputs → Outputs
Inputs: manifest.json
Outputs: Pass/fail against schema; on pass, loader builds `{ registry, tally, params }` from local files (no network). :contentReference[oaicite:3]{index=3} :contentReference[oaicite:4]{index=4}

4) Entities/Fields (schema shape to encode)
Root object
- schema_version (string, required) — e.g., "1.x".
- reg_path (string, required) — path to DivisionRegistry (registry.json).
- ballot_tally_path (string, required) — path to BallotTally (tally.json).
- params_path (string, required) — path to ParameterSet (params.json).
- expect (object, optional) — sanity locks (non-normative):
  - formula_id (string) — expected FID (64-hex recommended, not enforced here).
  - engine_version (string)
- digests (object, optional) — map of { "<relative_path>": { "sha256": "<64-hex-lowercase>" } }
- notes (string, optional)

Path rules
- Local filesystem only; **no URLs** (`http://` or `https://` forbidden). This enforces the offline contract for canonical runs. :contentReference[oaicite:5]{index=5}

Digest rule
- All sha256 digests are **64 lowercase hex**. :contentReference[oaicite:6]{index=6}

5) Variables
None.

6) Functions
(Schema only.)

7) Schema authoring outline (JSON Schema Draft 2020-12)
- $schema: "https://json-schema.org/draft/2020-12/schema"
- $id: "https://…/schemas/manifest.schema.json"
- $defs:
  * local_path: { "type":"string", "pattern":"^(?!https?://).+" }
  * sha256_64hex: { "type":"string", "pattern":"^[0-9a-f]{64}$" }
  * digest_obj: {
      "type":"object",
      "required":["sha256"],
      "properties":{ "sha256":{"$ref":"#/$defs/sha256_64hex"} },
      "additionalProperties":false
    }
  * expect_obj: {
      "type":"object",
      "properties":{
        "formula_id":{"type":"string"},   // (FID is 64-hex per spec; loader/CI compares) :contentReference[oaicite:7]{index=7}
        "engine_version":{"type":"string"}
      },
      "additionalProperties":false
    }
- Root:
  {
    "type":"object",
    "required":["schema_version","reg_path","ballot_tally_path","params_path"],
    "properties":{
      "schema_version":{"type":"string"},
      "reg_path":{"$ref":"#/$defs/local_path"},
      "ballot_tally_path":{"$ref":"#/$defs/local_path"},
      "params_path":{"$ref":"#/$defs/local_path"},
      "expect":{"$ref":"#/$defs/expect_obj"},
      "digests":{
        "type":"object",
        "additionalProperties":{"$ref":"#/$defs/digest_obj"}
      },
      "notes":{"type":"string"}
    },
    "additionalProperties":false
  }
- $comment (informative):
  * Canonical inputs for tests/runs are exactly `registry.json`, `tally.json`, `params.json`. No `ballots_path`. :contentReference[oaicite:8]{index=8} :contentReference[oaicite:9]{index=9}
  * Canonicalization (UTF-8, LF, sorted keys; arrays ordered) is enforced when hashing/verification. :contentReference[oaicite:10]{index=10}

8) State Flow
Loader resolves manifest directory → joins relative paths → rejects URLs → reads **registry/tally/params** → canonicalizes and (optionally) verifies digests before running. :contentReference[oaicite:11]{index=11}

9) Determinism & Hashing (informative)
- For verification, `inputs_sha256.{registry,tally,params}` are compared to recomputed sha256 of canonical bytes (64-hex). :contentReference[oaicite:12]{index=12} :contentReference[oaicite:13]{index=13}

10) Edge Cases & Failure Policy
Schema-level:
- Missing any of the three required paths ⇒ fail.
- Any path that matches `^https?://` ⇒ fail (offline requirement). :contentReference[oaicite:14]{index=14}
- `digests.*.sha256` not 64-hex ⇒ fail. :contentReference[oaicite:15]{index=15}
Runtime/loader (outside schema):
- If `expect.formula_id` or `engine_version` mismatches actual values, loader must error before running. (See Annex-B verification algorithm.) :contentReference[oaicite:16]{index=16}

11) Test Checklist (must pass)
Happy path:
{
  "schema_version":"1.x",
  "reg_path":"cases/VM-TST-101/registry.json",
  "ballot_tally_path":"cases/VM-TST-101/tally.json",
  "params_path":"cases/VM-TST-101/params.json",
  "digests":{
    "cases/VM-TST-101/registry.json":{"sha256":"<64hex>"},
    "cases/VM-TST-101/tally.json":{"sha256":"<64hex>"},
    "cases/VM-TST-101/params.json":{"sha256":"<64hex>"}
  }
}
→ pass.

Failing patterns:
- Includes `ballots_path` or omits `ballot_tally_path` → fail (contract is tally, not raw ballots). :contentReference[oaicite:17]{index=17}
- Any path is a URL → fail. :contentReference[oaicite:18]{index=18}
- Any digest not 64-hex → fail. :contentReference[oaicite:19]{index=19}
