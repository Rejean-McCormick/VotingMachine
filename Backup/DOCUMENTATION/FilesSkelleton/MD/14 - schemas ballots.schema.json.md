Pre-Coding Essentials (Component: schemas/ballots.schema.json, Version/FormulaID: VM-ENGINE v0) — 14/89

STATUS: Non-Normative (ingestion only). The engine and Canonical Test Pack accept only DivisionRegistry, BallotTally, ParameterSet. This schema is not consumed by the engine or the test harness.

1) Goal & Success
Goal: Define a strict JSON Schema for upstream **raw ballots** that an ingestion tool can normalize and aggregate into the canonical **BallotTally** input.
Success: Validates a single ballot payload (plurality | approval | score | ranked_irv | ranked_condorcet); rejects malformed ballots; produces a deterministic mapping target for tally conversion. Does not redefine canonical inputs/IDs.

2) Scope
In scope (ingestion only): Top-level metadata, one-of payload selection, per-ballot shapes by type, basic ID/token domains.
Out of scope (canonical): Any assertion that the engine/test-pack read this file; cross-file referential checks (unit/option existence); determinism/FID; final counts (done in BallotTally).

3) Inputs → Outputs
Inputs: ballots.json (raw ballots; source format normalized to this schema).
Outputs: Pass/fail against this schema. On pass, an **ingestion converter** emits canonical `tally.json` (BallotTally) for the engine. No direct engine consumption.

4) Entities/Fields (schema shape to encode)
Root object
- schema_version (string, required) — e.g., "1.x".
- bal_id (string, optional) — project-local identifier, if used: "BAL:" + token.
- ballot_type (string, required, enum) — plurality | approval | score | ranked_irv | ranked_condorcet.
- payload (object, required) — exactly one of: { plurality | approval | score | ranked_irv | ranked_condorcet }.
- notes (string, optional).

ID/token domains (applies to unit_id, option_id, and the free token part of bal_id)
- Regex: ^[A-Za-z0-9_.:-]{1,64}$  // non-empty, ≤64, allowed chars.

Payloads (mutually exclusive)
plurality
- ballots (array, required) of:
  { unit_id: string, vote: string | null }  // null = blank (ingestion semantics).
approval
- ballots (array, required) of:
  { unit_id: string, approvals: array<string> }  // approvals may be empty = blank.
score
- scale_min (integer, required)
- scale_max (integer, required)  // must be > scale_min
- ballots (array, required) of:
  { unit_id: string, scores: object{ <option_id>: integer } }
  // each integer ∈ [scale_min .. scale_max]
ranked_irv
- ballots (array, required) of:
  { unit_id: string, ranking: array<string /* option_id */> }  // unique elements
ranked_condorcet
- same shape as ranked_irv

General
- Arrays may be empty (edge cases); size limits are enforced upstream/downstream, not here.
- All objects use additionalProperties: false.

5) Variables
None (schema-only component; no VM-VARs referenced).

6) Functions
None.

7) Schema authoring outline (JSON Schema Draft 2020-12)
- $schema: "https://json-schema.org/draft/2020-12/schema"
- $id: "https://…/schemas/non-normative/ballots.schema.json"
- $defs:
  * id_token: { "type": "string", "pattern": "^[A-Za-z0-9_.:-]{1,64}$" }
  * unit_id: { "$ref": "#/$defs/id_token" }
  * option_id: { "$ref": "#/$defs/id_token" }
  * bal_id:   { "type": "string", "pattern": "^BAL:[A-Za-z0-9_.:-]{1,64}$" }
  * ranked: {
      "type":"object",
      "required":["unit_id","ranking"],
      "properties":{
        "unit_id":{"$ref":"#/$defs/unit_id"},
        "ranking":{"type":"array","items":{"$ref":"#/$defs/option_id"},"uniqueItems":true}
      },
      "additionalProperties":false
    }
  * approval_item: {
      "type":"object",
      "required":["unit_id","approvals"],
      "properties":{
        "unit_id":{"$ref":"#/$defs/unit_id"},
        "approvals":{"type":"array","items":{"$ref":"#/$defs/option_id"}}
      },
      "additionalProperties":false
    }
  * plurality_item: {
      "type":"object",
      "required":["unit_id","vote"],
      "properties":{
        "unit_id":{"$ref":"#/$defs/unit_id"},
        "vote":{"oneOf":[{"$ref":"#/$defs/option_id"},{"type":"null"}]}
      },
      "additionalProperties":false
    }
  * score_item: {
      "type":"object",
      "required":["unit_id","scores"],
      "properties":{
        "unit_id":{"$ref":"#/$defs/unit_id"},
        "scores":{
          "type":"object",
          "additionalProperties":{"type":"integer"}
        }
      },
      "additionalProperties":false
    }
- Root:
  {
    "type":"object",
    "required":["schema_version","ballot_type","payload"],
    "properties":{
      "schema_version":{"type":"string"},
      "bal_id":{"$ref":"#/$defs/bal_id"},
      "ballot_type":{"enum":["plurality","approval","score","ranked_irv","ranked_condorcet"]},
      "payload":{
        "type":"object",
        "properties":{
          "plurality":{"type":"object","required":["ballots"],"properties":{"ballots":{"type":"array","items":{"$ref":"#/$defs/plurality_item"}}},"additionalProperties":false},
          "approval":{"type":"object","required":["ballots"],"properties":{"ballots":{"type":"array","items":{"$ref":"#/$defs/approval_item"}}},"additionalProperties":false},
          "score":{"type":"object","required":["scale_min","scale_max","ballots"],"properties":{"scale_min":{"type":"integer"},"scale_max":{"type":"integer"},"ballots":{"type":"array","items":{"$ref":"#/$defs/score_item"}}},"allOf":[{"properties":{"scale_max":{"type":"integer","exclusiveMinimum":{"$data":"1/scale_min"}}}}],"additionalProperties":false},
          "ranked_irv":{"type":"object","required":["ballots"],"properties":{"ballots":{"type":"array","items":{"$ref":"#/$defs/ranked"}}},"additionalProperties":false},
          "ranked_condorcet":{"type":"object","required":["ballots"],"properties":{"ballots":{"type":"array","items":{"$ref":"#/$defs/ranked"}}},"additionalProperties":false}
        },
        "additionalProperties":false
      },
      "notes":{"type":"string"}
    },
    "additionalProperties":false,
    "allOf":[
      // enforce ballot_type ↔ payload key consistency
      {"if":{"properties":{"ballot_type":{"const":"approval"}},"required":["ballot_type"]},"then":{"required":["payload"],"properties":{"payload":{"required":["approval"]}}}},
      {"if":{"properties":{"ballot_type":{"const":"plurality"}},"required":["ballot_type"]},"then":{"required":["payload"],"properties":{"payload":{"required":["plurality"]}}}},
      {"if":{"properties":{"ballot_type":{"const":"score"}},"required":["ballot_type"]},"then":{"required":["payload"],"properties":{"payload":{"required":["score"]}}}},
      {"if":{"properties":{"ballot_type":{"const":"ranked_irv"}},"required":["ballot_type"]},"then":{"required":["payload"],"properties":{"payload":{"required":["ranked_irv"]}}}},
      {"if":{"properties":{"ballot_type":{"const":"ranked_condorcet"}},"required":["ballot_type"]},"then":{"required":["payload"],"properties":{"payload":{"required":["ranked_condorcet"]}}}}
    ]
  }

8) State Flow
Ingestion → schema-validate (this file) → normalize → **aggregate into BallotTally** (units/totals/options) → engine consumes BallotTally with Registry & ParameterSet. No direct engine read of ballots.json.

9) Determinism & Numeric Rules
Counts/scores are integers; floats not permitted in ballots. Canonical JSON rules (UTF-8, LF, sorted keys; ordered arrays) apply to canonical artifacts; ingestion may be looser, but converter must output canonical BallotTally. 

10) Edge Cases & Failure Policy
- Multiple payloads present → schema fail.
- ballot_type/payload mismatch → schema fail.
- Out-of-range scores or duplicate rankings (when unique enforced) → schema fail.
- Unknown fields anywhere → schema fail.
- Cross-file issues (unknown unit_id/option_id) → deferred to conversion/validation against Registry.

11) Test Checklist (ingestion)
- Minimal valid example per payload type → pass.
- File with both approval and plurality payloads → fail.
- ballot_type="score" with ranked_irv payload → fail.
- scale_min ≥ scale_max → fail.
- Ranked ballots with duplicate options in ranking (uniqueItems) → fail.
