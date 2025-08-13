Pre-Coding Essentials (Component: schemas/result.schema.json, Version/FormulaID: VM-ENGINE v0) — 18/89

1) Goal & Success
Goal: JSON Schema for the canonical Result output produced by the engine.
Success: Requires `result_id` and `formula_id`; accepts only the minimal, normalized shape defined in Doc 1 (summary + per-unit results), with shares as JSON numbers and arrays ordered deterministically. No input references or tie logs appear in Result. :contentReference[oaicite:3]{index=3} :contentReference[oaicite:4]{index=4}

2) Scope
In scope (normative for this schema):
- Root: { schema_version, result_id, formula_id, engine_version, created_at, summary{}, units[] }.
- Unit result: { unit_id, allocations[], label }.
- Allocation: { option_id, votes, share, (optional) seats/*family-specific deriveds*/ }.
Out of scope (captured elsewhere):
- Inputs/digests, RNG/tie details → **RunRecord**. :contentReference[oaicite:5]{index=5}
- Frontier diagnostics → **FrontierMap** (separate artifact). :contentReference[oaicite:6]{index=6}

3) Inputs → Outputs
Inputs: (none; this is an output artifact)
Outputs: A strict **Result** JSON used by reports and verification; `result_id` is sha256 of canonical payload; `formula_id` is FID of the Normative Manifest. :contentReference[oaicite:7]{index=7} :contentReference[oaicite:8]{index=8}

4) Entities/Fields (schema shape to encode)
Root object
- schema_version (string, required) — e.g., "1.x".
- result_id (string, required) — "RES:" + 64-hex (lowercase). :contentReference[oaicite:9]{index=9}
- formula_id (string, required) — 64-hex FID (see Doc 1A §2.3). :contentReference[oaicite:10]{index=10}
- engine_version (string, required) — "vX.Y.Z". :contentReference[oaicite:11]{index=11}
- created_at (string, required) — RFC3339 UTC. :contentReference[oaicite:12]{index=12}
- summary (object, required)
  - valid_ballots_total (integer ≥ 0, required)
  - invalid_ballots_total (integer ≥ 0, required)
  - turnout_rate (number ≥ 0, required)  // engine precision; reporter rounds. :contentReference[oaicite:13]{index=13}
- units (array, required, minItems ≥ 1) — ordered by ascending unit_id. :contentReference[oaicite:14]{index=14}

Unit result object (items of units[])
- unit_id (string, required) — FK → Registry.units.unit_id. :contentReference[oaicite:15]{index=15}
- allocations (array, required, minItems ≥ 1) — ordered by Registry `order_index`. :contentReference[oaicite:16]{index=16}
  Allocation item:
  - option_id (string, required) — FK → Registry.options.option_id
  - votes (integer ≥ 0, required)
  - share (number in [0,1], required)       // JSON number, not {num,den}. :contentReference[oaicite:17]{index=17} :contentReference[oaicite:18]{index=18}
  - seats (integer ≥ 0, optional)            // only if relevant to the algorithm family. :contentReference[oaicite:19]{index=19}
- label (string, required) — "Decisive" | "Marginal" | "Invalid" (presentation label). :contentReference[oaicite:20]{index=20}

Notes (informative)
- Do **not** include `reg_id`, `ballot_tally_id`, `parameter_set_id`; those are recorded in **RunRecord.inputs*** as sha256 digests, not in Result. :contentReference[oaicite:21]{index=21}
- Do **not** include `tie_log`; **RunRecord.ties[]** is the authoritative location for tie events. :contentReference[oaicite:22]{index=22}
- Arrays follow Doc 1A ordering rules: units ↑ unit_id; options ↑ order_index (tie by option_id). :contentReference[oaicite:23]{index=23}

5) Variables
None (schema-only component).

6) Functions
None (schema-only component).

7) Schema authoring outline (JSON Schema Draft 2020-12)
- $schema: "https://json-schema.org/draft/2020-12/schema"
- $id: "https://…/schemas/result.schema.json"
- $defs:
  * id64hex: { "type":"string", "pattern":"^[0-9a-f]{64}$" }
  * res_id: { "type":"string", "pattern":"^RES:[0-9a-f]{64}$" }
  * unit_allocation: {
      "type":"object",
      "required":["option_id","votes","share"],
      "properties":{
        "option_id":{"type":"string"},
        "votes":{"type":"integer","minimum":0},
        "share":{"type":"number","minimum":0,"maximum":1},
        "seats":{"type":"integer","minimum":0}
      },
      "additionalProperties":false
    }
  * unit_result: {
      "type":"object",
      "required":["unit_id","allocations","label"],
      "properties":{
        "unit_id":{"type":"string"},
        "allocations":{"type":"array","minItems":1,"items":{"$ref":"#/$defs/unit_allocation"}},
        "label":{"enum":["Decisive","Marginal","Invalid"]}
      },
      "additionalProperties":false
    }
- Root:
  {
    "type":"object",
    "required":["schema_version","result_id","formula_id","engine_version","created_at","summary","units"],
    "properties":{
      "schema_version":{"type":"string"},
      "result_id":{"$ref":"#/$defs/res_id"},
      "formula_id":{"$ref":"#/$defs/id64hex"},
      "engine_version":{"type":"string"},
      "created_at":{"type":"string","format":"date-time"},
      "summary":{
        "type":"object",
        "required":["valid_ballots_total","invalid_ballots_total","turnout_rate"],
        "properties":{
          "valid_ballots_total":{"type":"integer","minimum":0},
          "invalid_ballots_total":{"type":"integer","minimum":0},
          "turnout_rate":{"type":"number","minimum":0}
        },
        "additionalProperties":false
      },
      "units":{"type":"array","minItems":1,"items":{"$ref":"#/$defs/unit_result"}}
    },
    "additionalProperties":false,
    "$comment":"Ordering contract (informative): units ordered by ascending unit_id; allocations reflect Registry option order (order_index, then option_id)."
  }

8) State Flow
Produced by the engine after allocation/labeling and canonicalization; `result_id` computed then; **RunRecord** is built referencing this Result and capturing inputs, NM digest, variables, and ties. :contentReference[oaicite:24]{index=24} :contentReference[oaicite:25]{index=25}

9) Determinism & Numeric Rules (informative)
- Canonical JSON: UTF-8, LF, **sorted keys**; arrays ordered per Doc 1A §5; numbers emitted as JSON numbers (engine precision). :contentReference[oaicite:26]{index=26}
- Shares are JSON numbers in \[0,1\]; reporters handle rounding for display. :contentReference[oaicite:27]{index=27}

10) Edge Cases & Failure Policy
Schema rejects:
- Missing required fields; unknown fields (strict mode); wrong ID formats.
Engine/test validation (outside schema) rejects:
- FK violations against Registry; misordered arrays; mismatched FID or hashes during verification. :contentReference[oaicite:28]{index=28}

11) Test Checklist (must pass)
Happy path (thin result, one unit):
{
  "schema_version":"1.x",
  "result_id":"RES:<64hex>",
  "formula_id":"<64hex>",
  "engine_version":"v1.0.0",
  "created_at":"2025-08-12T14:00:00Z",
  "summary":{"valid_ballots_total":12345,"invalid_ballots_total":67,"turnout_rate":0.95},
  "units":[
    {"unit_id":"U-001","allocations":[
      {"option_id":"O-A1","votes":6000,"share":0.486},
      {"option_id":"O-B1","votes":5000,"share":0.405}
    ],"label":"Decisive"}
  ]
}
→ pass (ordering verified in tests; inputs & ties are checked via RunRecord). :contentReference[oaicite:29]{index=29} :contentReference[oaicite:30]{index=30}
