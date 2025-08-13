Here’s the corrected, reference-aligned file. Key fixes:

* Removed `reg_id` / `parameter_set_id` from the artifact (inputs live in RunRecord).
* Standardized the ID to `frontier_map_id: "FR:<64-hex>"`.
* Aligned naming/enums to ParameterSet’s frontier family (`mode: none|banded|ladder`; dropped invented names like `sliding_scale`).
* Kept support as a JSON number (`support_share` in \[0,1]) for consistency with Result shares.
* Kept the object strict (`additionalProperties:false`) and added sensible conditionals (no `bands` when `mode="none"`).

<!-- Converted from: 20 - schemas division_registry.schema.json, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:46.035741Z -->

```
Pre-Coding Essentials (Component: schemas/frontier_map.schema.json, Version/FormulaID: VM-ENGINE v0) — 20/89

1) Goal & Success
Goal: JSON Schema for the canonical FrontierMap output — per-unit frontier status & contiguity outcomes derived from a run.
Success: Requires FR id; encodes the frontier configuration actually applied (mode, edge types, strategy, optional bands); lists every unit with its status, observed support share, and determinism flags; rejects unknown fields.

2) Scope
In scope (normative for this schema):
- Root: { schema_version, frontier_map_id, frontier_config{}, units[] }.
- frontier_config: echo of frontier knobs that govern mapping/contiguity (not a second ParameterSet).
- UnitFrontier: { unit_id, support_share, status, flags{}, optional adjacency_summary{} }.
Out of scope:
- Input references (registry/params digests/IDs live in RunRecord).
- Geometry/topology beyond identifiers; band overlap checks (pipeline validates).

3) Inputs → Outputs
Inputs: (none — this is an output artifact)
Outputs: One strict `frontier_map.json` object (optionally referenced by RunRecord).

4) Entities/Fields (schema shape to encode)
Root object
- schema_version (string, required) — e.g., "1.x".
- frontier_map_id (string, required) — "FR:" + 64-hex (lowercase).
- frontier_config (object, required)
  - mode (string, required) — enum: "none" | "banded" | "ladder".
  - contiguity_edge_types (array, required) — items enum: "land" | "bridge" | "water"; uniqueItems:true; minItems:1.
  - frontier_strategy (string, required) — enum: "apply_on_entry" | "apply_on_exit" | "sticky".
  - bands (array, optional; REQUIRED iff mode ≠ "none") — each band:
      { status:string (1..40 chars), min_pct:int [0..100], max_pct:int [0..100] }
      // Shape-only here; pipeline ensures non-overlap and ordering; min_pct ≤ max_pct is enforced in schema.
- units (array, required, minItems ≥ 1) — list of UnitFrontier; array is ordered by ascending unit_id (enforced in tests).

UnitFrontier (items of units[])
- unit_id (string, required)
- support_share (number, required) — observed share in [0,1]; engine precision; reporters handle rounding.
- status (string, required) — "none" when mode="none"; otherwise one of the configured band status labels.
- flags (object, required)
  - contiguity_ok (boolean, required)
  - mediation_flagged (boolean, required)
  - protected_override_used (boolean, required)
  - enclave (boolean, required)
- adjacency_summary (object, optional)
  - used_edges (array, required) — items enum: "land" | "bridge" | "water"; uniqueItems:true; minItems:1
  - corridor_used (boolean, optional)
  - reasons (array<string>, optional) — short machine-readable reason codes

5) Variables
None (schema-only component).

6) Functions
None (schema-only component).

7) Schema authoring outline (JSON Schema Draft 2020-12)
- $schema: "https://json-schema.org/draft/2020-12/schema"
- $id: "https://…/schemas/frontier_map.schema.json"
- $defs:
  * hex64: { "type":"string","pattern":"^[0-9a-f]{64}$" }
  * fr_id: { "type":"string","pattern":"^FR:[0-9a-f]{64}$" }
  * edge: { "enum":["land","bridge","water"] }
  * band: {
      "type":"object",
      "required":["status","min_pct","max_pct"],
      "properties":{
        "status":{"type":"string","minLength":1,"maxLength":40},
        "min_pct":{"type":"integer","minimum":0,"maximum":100},
        "max_pct":{"type":"integer","minimum":0,"maximum":100}
      },
      "allOf":[{"properties":{"max_pct":{"type":"integer"}}, "if":{"properties":{"min_pct":{"type":"integer"}}}, "then":{}}],  // min/max typed
      "additionalProperties":false
    }
  * flags: {
      "type":"object",
      "required":["contiguity_ok","mediation_flagged","protected_override_used","enclave"],
      "properties":{
        "contiguity_ok":{"type":"boolean"},
        "mediation_flagged":{"type":"boolean"},
        "protected_override_used":{"type":"boolean"},
        "enclave":{"type":"boolean"}
      },
      "additionalProperties":false
    }
  * adjacency_summary: {
      "type":"object",
      "required":["used_edges"],
      "properties":{
        "used_edges":{"type":"array","minItems":1,"items":{"$ref":"#/$defs/edge"},"uniqueItems":true},
        "corridor_used":{"type":"boolean"},
        "reasons":{"type":"array","items":{"type":"string"}}
      },
      "additionalProperties":false
    }
  * unit_frontier: {
      "type":"object",
      "required":["unit_id","support_share","status","flags"],
      "properties":{
        "unit_id":{"type":"string"},
        "support_share":{"type":"number","minimum":0,"maximum":1},
        "status":{"type":"string"},
        "flags":{"$ref":"#/$defs/flags"},
        "adjacency_summary":{"$ref":"#/$defs/adjacency_summary"}
      },
      "additionalProperties":false
    }
  * frontier_config: {
      "type":"object",
      "required":["mode","contiguity_edge_types","frontier_strategy"],
      "properties":{
        "mode":{"enum":["none","banded","ladder"]},
        "contiguity_edge_types":{"type":"array","items":{"$ref":"#/$defs/edge"},"uniqueItems":true,"minItems":1},
        "frontier_strategy":{"enum":["apply_on_entry","apply_on_exit","sticky"]},
        "bands":{"type":"array","items":{"$ref":"#/$defs/band"}}
      },
      "additionalProperties":false,
      "allOf":[
        { "if":{"properties":{"mode":{"const":"none"}},"then":{"not":{"required":["bands"]}} },
        { "if":{"properties":{"mode":{"enum":["banded","ladder"]}}},"then":{"required":["bands"]} }
      ]
    }
- Root:
  {
    "type":"object",
    "required":["schema_version","frontier_map_id","frontier_config","units"],
    "properties":{
      "schema_version":{"type":"string"},
      "frontier_map_id":{"$ref":"#/$defs/fr_id"},
      "frontier_config":{"$ref":"#/$defs/frontier_config"},
      "units":{"type":"array","minItems":1,"items":{"$ref":"#/$defs/unit_frontier"}}
    },
    "additionalProperties":false,
    "$comment":"Ordering (informative): units ↑ unit_id; arrays are canonicalized by the engine; JSON is UTF-8, LF, sorted keys."
  }

8) State Flow
Produced after gates/allocation by the MAP_FRONTIER phase. May be referenced by RunRecord; reports may render it.

9) Determinism & Numeric Rules (informative)
- All digests/IDs are computed on canonical bytes elsewhere (RunRecord holds input digests).
- `support_share` is a JSON number in [0,1]; reporters handle rounding; engine maintains internal integer math.

10) Edge Cases & Failure Policy
Schema-level:
- mode="none" ⇒ `bands` must be absent; each unit.status must equal "none".
- Empty or unknown `contiguity_edge_types` item ⇒ fail.
- `support_share` outside [0,1] ⇒ fail.
- Band with min_pct > max_pct ⇒ fail.
Pipeline/tests (outside schema):
- Duplicate/missing units vs Registry; band overlap/ordering; status not in bands when mode≠"none"; contiguity inconsistencies.

11) Test Checklist (must pass)
Happy path:
{
  "schema_version":"1.x",
  "frontier_map_id":"FR:0123…abcd",
  "frontier_config":{
    "mode":"banded",
    "contiguity_edge_types":["land","bridge"],
    "frontier_strategy":"apply_on_entry",
    "bands":[{"status":"frontier","min_pct":45,"max_pct":55},{"status":"stable","min_pct":56,"max_pct":100}]
  },
  "units":[
    {"unit_id":"U-001","support_share":0.532,"status":"frontier","flags":{"contiguity_ok":true,"mediation_flagged":false,"protected_override_used":false,"enclave":false},
     "adjacency_summary":{"used_edges":["land"]}}
  ]
}
→ pass.

None mode:
- "frontier_config.mode":"none"; no `bands`; every unit.status == "none" → pass.

Failing patterns:
- Unknown edge in used_edges → fail.
- Band min_pct>max_pct → fail.
- support_share = 1.2 → fail.
```

If you want the concrete JSON Schema file (valid Draft 2020-12 JSON), I can output it verbatim next.
