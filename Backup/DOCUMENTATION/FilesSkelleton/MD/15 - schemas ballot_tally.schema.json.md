

```
Pre-Coding Essentials (Component: schemas/ballot_tally.schema.json, Version/FormulaID: VM-ENGINE v0) — 15/89

1) Goal & Success
Goal: JSON Schema for the canonical BallotTally input: per-unit totals and per-option votes aligned to the DivisionRegistry.
Success: Accepts only the BallotTally shape prescribed by Doc 1B; rejects extra/missing fields or bad domains; preserves array ordering contracts for determinism (units by unit_id; options by registry order_index).

2) Scope
In scope (normative for this schema):
- Root: { schema_version, units[] }.
- Unit: { unit_id, totals{ valid_ballots, invalid_ballots }, options[] }.
- Option tally: { option_id, votes }.
- Domains: ID charset/length; non-negative integers for counts.
Out of scope (validated by engine/tests):
- Referential integrity to Registry; sum(options[].votes) ≤ totals.valid_ballots; global ordering enforcement and hashing.

3) Inputs → Outputs
Inputs: tally.json (BallotTally).
Outputs: Pass/fail against this schema; on pass, loader builds typed UnitTallies consumed at S1 of the algorithm. 

4) Entities/Fields (schema shape to encode)
Root object
- schema_version (string, required) — e.g., "1.x".
- units (array, required, minItems ≥ 1) — list of Unit tallies.

Unit object
- unit_id (string, required) — ID token (see rule below).
- totals (object, required)
  - valid_ballots (integer ≥ 0, required)
  - invalid_ballots (integer ≥ 0, required)
- options (array, required, minItems ≥ 1) — per-option tallies; array order mirrors Registry `order_index`.

Option tally object
- option_id (string, required) — ID token; FK to the unit’s option in Registry.
- votes (integer ≥ 0, required)

ID token rule (for unit_id/option_id)
- Non-empty string, max 64 chars; allowed: A–Z a–z 0–9 underscore _ hyphen - colon : dot .
- Regex: ^[A-Za-z0-9_.:-]{1,64}$

5) Variables
None (schema-only component).

6) Functions
None.

7) Schema authoring outline (JSON Schema Draft 2020-12)
- $schema: "https://json-schema.org/draft/2020-12/schema"
- $id: "https://…/schemas/ballot_tally.schema.json"
- $defs:
  * id_token: { "type":"string", "pattern":"^[A-Za-z0-9_.:-]{1,64}$" }
  * unit_id: { "$ref":"#/$defs/id_token" }
  * option_id: { "$ref":"#/$defs/id_token" }
  * option_tally: {
      "type":"object",
      "required":["option_id","votes"],
      "properties":{
        "option_id":{"$ref":"#/$defs/option_id"},
        "votes":{"type":"integer","minimum":0}
      },
      "additionalProperties":false
    }
  * unit_tally: {
      "type":"object",
      "required":["unit_id","totals","options"],
      "properties":{
        "unit_id":{"$ref":"#/$defs/unit_id"},
        "totals":{
          "type":"object",
          "required":["valid_ballots","invalid_ballots"],
          "properties":{
            "valid_ballots":{"type":"integer","minimum":0},
            "invalid_ballots":{"type":"integer","minimum":0}
          },
          "additionalProperties":false
        },
        "options":{
          "type":"array",
          "minItems":1,
          "items":{"$ref":"#/$defs/option_tally"}
        }
      },
      "additionalProperties":false
    }
- Root:
  {
    "type":"object",
    "required":["schema_version","units"],
    "properties":{
      "schema_version":{"type":"string"},
      "units":{
        "type":"array",
        "minItems":1,
        "items":{"$ref":"#/$defs/unit_tally"}
      }
    },
    "additionalProperties":false,
    "$comment":"Ordering contract (informative): units ordered by ascending unit_id; options ordered by Registry order_index (ties by option_id). Enforced by validation/tests, not JSON Schema."
  }

8) State Flow
Loader: schema-validate → construct UnitTallies → S1 Per-unit tallies loads {valid_ballots, invalid_ballots, votes[]} for counting.

9) Determinism & Ordering (informative)
- Canonical JSON (UTF-8, LF, sorted keys) and array ordering per spec underpin hashing and byte-identical outputs.
- Arrays must reflect: units ↑ unit_id; options ↑ order_index (tie: option_id). Enforced by conformance checks/tests.

10) Edge Cases & Failure Policy
Schema-level rejects:
- Negative counts; missing required fields; unknown fields (additionalProperties: false).
Engine/test validation rejects (beyond schema):
- Sum(options[].votes) > totals.valid_ballots; unknown unit_id/option_id vs Registry; mis-ordered arrays.

11) Test Checklist (must pass)
Happy path (minimal):
{
  "schema_version":"1.x",
  "units":[
    {
      "unit_id":"U-001",
      "totals":{"valid_ballots":12345,"invalid_ballots":67},
      "options":[
        {"option_id":"O-A1","votes":6000},
        {"option_id":"O-B1","votes":5000}
      ]
    }
  ]
}
Failing patterns:
- Extra top-level fields (e.g., id/label/ballot_type) → schema fail.
- Negative votes or totals → schema fail.
- options missing or empty when Registry has options → engine/test fail.
- Sum(options[].votes) > totals.valid_ballots → engine/test fail.
```

### Why these adjustments (with references)

* **BallotTally shape & fields** — per-unit totals and per-option votes; **no maps** for options; use `options[]` with `{option_id, votes}` and totals as `valid_ballots`/`invalid_ballots`. &#x20;
* **Ordering contract** — arrays must reflect: units ↑ `unit_id`, options ↑ `order_index` (tie by `option_id`). &#x20;
* **Per-unit tallies used at S1** — engine loads `valid_ballots`, `invalid_ballots`, and per-option `votes`.&#x20;
* **ID domains** — `unit_id`/`option_id` allowed chars and max length.&#x20;
* **Non-negativity & integrity** — counts ≥ 0; sum of `options[].votes` ≤ `totals.valid_ballots`; FK to Registry. (Sum & FK enforced by validation/tests, not pure JSON Schema.)  &#x20;
* **Test-pack contract** — cases feed exactly `registry.json`, `tally.json`, `params.json`; keeping naming/order makes hashing predictable.&#x20;

### Primary deltas vs your draft

* Replaced `votes{ OPT: int }` maps with **`options[]` array** of `{ option_id, votes }`.&#x20;
* Renamed `ballots_cast`/`invalid_or_blank` → **`totals.valid_ballots` / `totals.invalid_ballots`**.&#x20;
* Dropped top-level `id/label/reg_id/ballot_type/tallies` and all per-mode payloads. BallotTally is **one canonical format**.&#x20;
* Tightened **ID charset/length** and **non-negativity** rules; made `additionalProperties:false` at all levels. &#x20;


