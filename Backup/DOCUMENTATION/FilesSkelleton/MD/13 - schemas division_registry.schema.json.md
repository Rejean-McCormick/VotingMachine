Pre-Coding Essentials (Component: schemas/division_registry.schema.json, Version/FormulaID: VM-ENGINE v0) — 13/89

1) Goal & Success
Goal: JSON Schema that locks the DivisionRegistry structure exactly as defined in the database spec.
Success: Accepts only the canonical DivisionRegistry shape (root + units + options) with correct field names/types/domains; rejects any extra/missing fields or malformed IDs.

2) Scope
In scope (normative for this schema):
- Root object: { schema_version, units[] }.
- Unit object: { unit_id, name, protected_area, options[] }.
- Option object: { option_id, name, order_index }.
- Basic domains: string length/charset for IDs; name length; integer bounds for order_index.
Out of scope (checked elsewhere in validation/pipeline/tests):
- Ordering contracts (units sorted by unit_id; options sorted by order_index).
- Uniqueness by property (unit_id across units; option_id/order_index uniqueness within a unit).
- Cross-artifact referential checks and hashing/FID rules.

3) Inputs → Outputs
Inputs: DivisionRegistry JSON (e.g., /cases/<ID>/registry.json).
Outputs: Pass/fail against this schema with precise error paths; on pass, downstream loader receives strongly-typed data.

4) Entities/Fields (schema shape to encode)
Root object
- schema_version (string, required) — e.g., "1.x".
- units (array, required, minItems ≥ 1) — list of Unit objects.

Unit object
- unit_id (string, required) — ID token; see ID charset/length rule below.
- name (string, required) — 1..200 chars.
- protected_area (boolean, required).
- options (array, required, minItems ≥ 1) — list of Option objects.

Option object
- option_id (string, required) — ID token; see ID charset/length rule below.
- name (string, required) — 1..200 chars.
- order_index (integer, required) — ≥ 0 (unique within the unit; enforced outside the schema).

ID token rule (applies to unit_id and option_id)
- Non-empty string, max 64 characters.
- Allowed characters: A–Z a–z 0–9 underscore _ hyphen - colon : dot .
- (Regex example) ^[A-Za-z0-9_.:-]{1,64}$

Field naming
- snake_case only (e.g., schema_version, protected_area, order_index).

5) Variables
None (schema-only component).

6) Functions
None (schema-only component).

7) Schema authoring outline (JSON Schema Draft 2020-12)
- $schema: "https://json-schema.org/draft/2020-12/schema"
- $id: "https://…/schemas/division_registry.schema.json"
- $defs:
  * id_token: type string, pattern ^[A-Za-z0-9_.:-]{1,64}$
  * name_200: type string, minLength: 1, maxLength: 200
  * option: object
      - required: ["option_id","name","order_index"]
      - properties:
          option_id: { $ref: "#/$defs/id_token" }
          name: { $ref: "#/$defs/name_200" }
          order_index: { type: "integer", minimum: 0 }
      - additionalProperties: false
  * unit: object
      - required: ["unit_id","name","protected_area","options"]
      - properties:
          unit_id: { $ref: "#/$defs/id_token" }
          name: { $ref: "#/$defs/name_200" }
          protected_area: { type: "boolean" }
          options: {
            type: "array",
            minItems: 1,
            items: { $ref: "#/$defs/option" }
          }
      - additionalProperties: false
- Root:
  * type: object
  * required: ["schema_version","units"]
  * properties:
      schema_version: { type: "string" }
      units: {
        type: "array",
        minItems: 1,
        items: { $ref: "#/$defs/unit" }
      }
  * additionalProperties: false
- Notes ($comment fields encouraged):
  * Ordering contracts (units by unit_id; options by order_index) are normative but enforced by conformance checks, not by JSON Schema.
  * Property-uniqueness constraints (e.g., option_id/order_index uniqueness within a unit; unit_id uniqueness across units) are enforced in validation/pipeline.

8) State flow (where this schema sits)
Loader: schema-validate → on success, construct in-memory model → validation layer enforces ordering/uniqueness/integrity → algorithm pipeline executes.

9) Determinism & ordering (informative)
- Canonical JSON (UTF-8, LF, sorted keys; arrays in spec order) governs hashing/IDs.
- Arrays must be ordered: units ascending unit_id; options ascending order_index (tie by option_id). Verified by conformance checks/tests.

10) Edge cases & failure policy
Schema-level failures (reject at schema):
- Extra/missing fields; wrong field names (non-snake_case).
- Bad ID charset/length; name length out of bounds.
- Negative order_index or non-integer.
Validation/pipeline failures (beyond schema):
- Duplicate unit_id in units[]; duplicate option_id or duplicate order_index within a unit.
- Arrays not in required order.

11) Test checklist (must pass)
Happy path (minimal):
- {"schema_version":"1.x","units":[{"unit_id":"U-001","name":"District 1","protected_area":false,"options":[{"option_id":"O-A1","name":"Option A","order_index":1},{"option_id":"O-B1","name":"Option B","order_index":2}]}]}
Malformed IDs:
- unit_id with illegal chars → schema fail.
Bounds:
- order_index < 0 → schema fail.
Extraneous fields:
- Any field not specified (e.g., adjacency, parent, magnitude) → schema fail.
Ordering/uniqueness (checked by validation/tests, not by schema):
- Unsorted units/options, duplicate IDs/order_index → validation fail with specific error codes.

Authoring notes
- All objects use additionalProperties: false.
- Field names are snake_case.
- Keep schema minimal and mirror Doc 1 exactly; do not introduce fields from other artifacts.
