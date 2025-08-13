
```
Pre-Coding Essentials (Component: schemas/parameter_set.schema.json, Version/FormulaID: VM-ENGINE v0) — 16/89

1) Goal & Success
Goal: JSON Schema for the canonical ParameterSet input capturing the effective VM-VAR values used for a run.
Success: Accepts only the VM-VAR keys defined in Annex A with correct types/domains; requires every Included (FID) variable to be present; allows Excluded variables but they never affect FID. Loader can build a typed map with zero ambiguity. :contentReference[oaicite:0]{index=0} :contentReference[oaicite:1]{index=1}

2) Scope
In scope (normative for this schema):
- Shape: { schema_version, vars{ "VM-VAR-###": value } }. No PS: identifier at top-level. :contentReference[oaicite:2]{index=2} :contentReference[oaicite:3]{index=3}
- Domains for key families per Annex A (types, integer bounds, enums noted “per release”). :contentReference[oaicite:4]{index=4}
- FID rules: Included vs Excluded sets; canonicalization reminder. :contentReference[oaicite:5]{index=5} :contentReference[oaicite:6]{index=6}
Out of scope: Computing FID (Doc 1A does that), cross-entity checks. :contentReference[oaicite:7]{index=7}

3) Inputs → Outputs
Inputs: parameter_set.json  
Outputs: Pass/fail against schema; on pass, a frozen ParameterSet used by all pipeline stages and echoed into RunRecord.vars_effective. :contentReference[oaicite:8]{index=8}

4) Entities/Fields (schema shape to encode)
Root
- schema_version (string, required) — e.g., "1.x".
- vars (object, required) — map of VM-VAR-### → value.

vars map — keys & domains (selection; types reflect Annex A)
Included (FID) — **required** keys:
- 001 algorithm_family — enum (per release). :contentReference[oaicite:9]{index=9}
- 002 rounding_policy — enum (per release). :contentReference[oaicite:10]{index=10}
- 003 share_precision — integer 0..6. :contentReference[oaicite:11]{index=11}
- 004 denom_rule — enum (per family). :contentReference[oaicite:12]{index=12}
- 005 aggregation_mode — enum (per family). :contentReference[oaicite:13]{index=13}
- 006 seat_allocation_rule — enum (per family). :contentReference[oaicite:14]{index=14}
- 007 tie_scope_model — enum (`winner_only`/`rank_all`, per spec). :contentReference[oaicite:15]{index=15}
- 010–017 thresholds/gates — integers 0..100 (per variable definitions in 4B). :contentReference[oaicite:16]{index=16}
- 020–028 thresholds/families — domain per variable (integers 0..100 or per-release enums). :contentReference[oaicite:17]{index=17}
- 029 symmetry_exceptions — array<string> (deterministic selectors). :contentReference[oaicite:18]{index=18}
- 030 eligibility_override_list — array<object{unit_id: string, mode: "include"|"exclude"}>. :contentReference[oaicite:19]{index=19}
- 031 ballot_integrity_floor — integer 0..100. :contentReference[oaicite:20]{index=20}
- 040 frontier_mode — enum { none, banded, ladder }. :contentReference[oaicite:21]{index=21}
- 041 frontier_cut — number/enum per mode. :contentReference[oaicite:22]{index=22}
- 042 frontier_strategy — enum { apply_on_entry, apply_on_exit, sticky }. :contentReference[oaicite:23]{index=23}
- 045 protected_area_override — enum { deny, allow }. :contentReference[oaicite:24]{index=24}
- 046 autonomy_package_map — object (deterministic keys; release-documented). :contentReference[oaicite:25]{index=25}
- 047 frontier_band_window — number 0.00..1.00. :contentReference[oaicite:26]{index=26}
- 048 frontier_backoff_policy — enum { none, soften, harden }. :contentReference[oaicite:27]{index=27}
- 049 frontier_strictness — enum { strict, lenient }. :contentReference[oaicite:28]{index=28}
- 050 tie_policy — enum { status_quo, deterministic_order, random }. **In FID.** :contentReference[oaicite:29]{index=29} :contentReference[oaicite:30]{index=30}
- 021 run_scope — enum "all_units" or selector object (domain per release). :contentReference[oaicite:31]{index=31}
- 073 algorithm_variant — enum (per release). **In FID.** :contentReference[oaicite:32]{index=32} :contentReference[oaicite:33]{index=33}

Excluded (non-FID) — **optional** keys:
- 032 unit_sort_order — enum { unit_id, label_priority, turnout }. :contentReference[oaicite:34]{index=34}
- 033 ties_section_visibility — enum { auto, always, never }. :contentReference[oaicite:35]{index=35}
- 034 frontier_map_enabled — **boolean** (not "on"/"off"). :contentReference[oaicite:36]{index=36}
- 035 sensitivity_analysis_enabled — **boolean**. :contentReference[oaicite:37]{index=37}
- 052 tie_seed — **integer ≥ 0** (seed recorded only if a random tie occurred). **Excluded** from FID. :contentReference[oaicite:38]{index=38}
- 060 majority_label_threshold — integer 0..100. :contentReference[oaicite:39]{index=39}
- 061 decisiveness_label_policy — enum { fixed, dynamic_margin }. :contentReference[oaicite:40]{index=40}
- 062 unit_display_language — string ("auto" or IETF tag). :contentReference[oaicite:41]{index=41}

5) Variables (rules & FID set)
- **Included (FID) required**: `001–007, 010–017, 020–031 (incl. 021, 029–031), 040–049, 050, 073`.  
- **Excluded (non-FID) optional**: `032–035, 052, 060–062`.  
- Canonicalization reminder: UTF-8, LF, sorted keys at all object levels. :contentReference[oaicite:42]{index=42} :contentReference[oaicite:43]{index=43}

6) Functions
Schema only.

7) Schema authoring outline (JSON Schema Draft 2020-12)
- $schema: "https://json-schema.org/draft/2020-12/schema"
- $id: "https://…/schemas/parameter_set.schema.json"
- $defs:
  * vm_var_id: pattern ^VM-VAR-(\\d{3})$
  * nonneg_int: { type: "integer", minimum: 0 }
  * pct_int: { type: "integer", minimum: 0, maximum: 100 }
  * bool: { type: "boolean" } // no "on"/"off"
  * tie_policy_enum: { "enum": ["status_quo","deterministic_order","random"] }
  * frontier_mode_enum: { "enum": ["none","banded","ladder"] }
  * frontier_strategy_enum: { "enum": ["apply_on_entry","apply_on_exit","sticky"] }
  * label_policy_enum: { "enum": ["fixed","dynamic_margin"] }
  * run_scope: { "oneOf":[ { "const":"all_units" }, { "type":"object" } ] } // selector map defined per release
  * sym_ex_selector: { "type":"string", "pattern":"^[A-Za-z0-9_.:-]{1,64}$" }

- Root:
  type: object
  required: ["schema_version","vars"]
  properties:
    schema_version: { type: "string" }
    vars:
      type: object
      // Require every Included (FID) var; allow Excluded
      required: [
        "VM-VAR-001","VM-VAR-002","VM-VAR-003","VM-VAR-004","VM-VAR-005","VM-VAR-006","VM-VAR-007",
        "VM-VAR-010","VM-VAR-011","VM-VAR-012","VM-VAR-013","VM-VAR-014","VM-VAR-015","VM-VAR-016","VM-VAR-017",
        "VM-VAR-020","VM-VAR-021","VM-VAR-022","VM-VAR-023","VM-VAR-024","VM-VAR-025","VM-VAR-026","VM-VAR-027","VM-VAR-028","VM-VAR-029","VM-VAR-030","VM-VAR-031",
        "VM-VAR-040","VM-VAR-041","VM-VAR-042","VM-VAR-045","VM-VAR-046","VM-VAR-047","VM-VAR-048","VM-VAR-049",
        "VM-VAR-050","VM-VAR-073"
      ]
      properties:
        // A. Global & family
        "VM-VAR-001": { "type":"string" }            // enum per release
        "VM-VAR-002": { "type":"string" }            // enum per release
        "VM-VAR-003": { "$ref":"#/$defs/pct_int" }   // 0..6 per Annex A
        "VM-VAR-004": { "type":"string" }            // enum per family
        "VM-VAR-005": { "type":"string" }
        "VM-VAR-006": { "type":"string" }
        "VM-VAR-007": { "type":"string" }
        // B. Thresholds & gates
        "VM-VAR-010": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-011": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-012": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-013": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-014": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-015": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-016": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-017": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-020": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-021": { "$ref":"#/$defs/run_scope" }
        "VM-VAR-022": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-023": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-024": { "$ref":"#/$defs/bool" }      // if boolean per release
        "VM-VAR-025": { "$ref":"#/$defs/bool" }      // if boolean per release
        "VM-VAR-026": { "type":["integer","number"] } // per release
        "VM-VAR-027": { "type":["integer","number"] }
        "VM-VAR-028": { "type":["integer","number"] }
        "VM-VAR-029": { "type":"array","items":{"$ref":"#/$defs/sym_ex_selector"} }
        "VM-VAR-030": {
          "type":"array",
          "items":{
            "type":"object",
            "required":["unit_id","mode"],
            "properties":{
              "unit_id":{"type":"string"},
              "mode":{"enum":["include","exclude"]}
            },
            "additionalProperties":false
          }
        }
        "VM-VAR-031": { "$ref":"#/$defs/pct_int" }
        // C. Frontier & refinements (+ Protected/Autonomy)
        "VM-VAR-040": { "$ref":"#/$defs/frontier_mode_enum" }
        "VM-VAR-041": { "type":["number","string"] } // per mode
        "VM-VAR-042": { "$ref":"#/$defs/frontier_strategy_enum" }
        "VM-VAR-045": { "enum":["deny","allow"] }
        "VM-VAR-046": { "type":"object" }           // deterministic keys; doc’d map
        "VM-VAR-047": { "type":"number", "minimum":0, "maximum":1 }
        "VM-VAR-048": { "enum":["none","soften","harden"] }
        "VM-VAR-049": { "enum":["strict","lenient"] }
        // D. Ties (policy is Included; seed is Excluded)
        "VM-VAR-050": { "$ref":"#/$defs/tie_policy_enum" }
        "VM-VAR-052": { "$ref":"#/$defs/nonneg_int" }   // Excluded (optional); still allowed here
        // E. Presentation toggles (Excluded; optional)
        "VM-VAR-032": { "enum":["unit_id","label_priority","turnout"] }
        "VM-VAR-033": { "enum":["auto","always","never"] }
        "VM-VAR-034": { "$ref":"#/$defs/bool" }
        "VM-VAR-035": { "$ref":"#/$defs/bool" }
        "VM-VAR-060": { "$ref":"#/$defs/pct_int" }
        "VM-VAR-061": { "$ref":"#/$defs/label_policy_enum" }
        "VM-VAR-062": { "type":"string" } // "auto" or IETF tag
        // F. Variant
        "VM-VAR-073": { "type":"string" }            // enum per release
      },
      additionalProperties: false
  }
  additionalProperties: false
  // Non-normative notes in $comment:
  // * Canonical JSON (UTF-8, LF, sorted keys) applies to this artifact.
  // * FID is built from Included vars only; seed 052 is Excluded.
  // * Booleans are real booleans, not "on"/"off".
  // * Domains marked "per release" must be enumerated in the shippped registry JSON.

8) State Flow
Loader validates → builds typed Params → echoed into RunRecord.vars_effective; RNG is initialized only if 050 = "random", and 052 is recorded in RunRecord iff a random tie occurred. :contentReference[oaicite:44]{index=44} :contentReference[oaicite:45]{index=45}

9) Determinism & FID
- Outcome-affecting Included vars (above) must be present; their canonical JSON snapshot forms the Normative Manifest → FID. Excluded vars (032–035, 052, 060–062) never enter FID. :contentReference[oaicite:46]{index=46} :contentReference[oaicite:47]{index=47}
- 050 is in FID; 052 is not (seed captured only if random ties actually happen). :contentReference[oaicite:48]{index=48}

10) Edge Cases & Failure Policy
- Any Included var missing ⇒ schema fail (`E-PS-MISS`). :contentReference[oaicite:49]{index=49}
- Wrong types (e.g., strings "on"/"off" for booleans) ⇒ schema fail (booleans must be JSON booleans). :contentReference[oaicite:50]{index=50}
- 052 must be integer ≥ 0; 64-hex seeds are invalid here. :contentReference[oaicite:51]{index=51}
- If `tie_policy="random"` and 052 is omitted, engine may still run (seed default 0) but must record 052 in RunRecord iff any random tie occurred. (Schema does not force 052 presence; it is Excluded.) :contentReference[oaicite:52]{index=52} :contentReference[oaicite:53]{index=53}

11) Test Checklist (must pass)
Happy path:
{ "schema_version":"1.x",
  "vars": {
    "VM-VAR-001":"family_v1","VM-VAR-002":"half_up","VM-VAR-003":3,"VM-VAR-004":"standard","VM-VAR-005":"sum","VM-VAR-006":"none","VM-VAR-007":"winner_only",
    "VM-VAR-010":0,"VM-VAR-011":0,"VM-VAR-012":0,"VM-VAR-013":0,"VM-VAR-014":0,"VM-VAR-015":0,"VM-VAR-016":0,"VM-VAR-017":0,
    "VM-VAR-020":0,"VM-VAR-021":"all_units","VM-VAR-022":55,"VM-VAR-023":55,"VM-VAR-024":true,"VM-VAR-025":true,"VM-VAR-026":0,"VM-VAR-027":0,"VM-VAR-028":0,
    "VM-VAR-029":[],"VM-VAR-030":[{"unit_id":"U-001","mode":"include"}],"VM-VAR-031":0,
    "VM-VAR-040":"none","VM-VAR-041":0.00,"VM-VAR-042":"apply_on_entry","VM-VAR-045":"deny","VM-VAR-046":{},"VM-VAR-047":0.00,"VM-VAR-048":"none","VM-VAR-049":"strict",
    "VM-VAR-050":"status_quo","VM-VAR-073":"v1",
    "VM-VAR-052":0, "VM-VAR-034":true, "VM-VAR-060":55, "VM-VAR-061":"dynamic_margin", "VM-VAR-062":"auto"
}}
→ pass schema; FID ignores 052/034/060–062. :contentReference[oaicite:54]{index=54}

Failing patterns:
- Missing any Included var (e.g., omit 050) → `E-PS-MISS`. :contentReference[oaicite:55]{index=55}
- `"VM-VAR-034":"on"` → fail (must be boolean). :contentReference[oaicite:56]{index=56}
- `"VM-VAR-052":"deadbeef..."` → fail (seed is integer ≥ 0, not 64-hex). :contentReference[oaicite:57]{index=57}
```

### What I changed (beyond your notes)

* Removed any top-level `PS:` identifier; **ParameterSet** is `{schema_version, vars{…}}` per Doc 1B.&#x20;
* Switched all boolean-typed variables to real JSON booleans (e.g., 034/035), not `"on"|"off"`.&#x20;
* Modeled **VM-VAR-073** as `algorithm_variant` (enum, per release), not an “executive toggle”. &#x20;
* Restored official names for 029–031 and especially **030/031** (`eligibility_override_list`, `ballot_integrity_floor`); removed invented “weighting\_method”.&#x20;
* Enforced the **Included/Excluded** FID sets exactly as Annex A states (050 in FID; **052 Excluded**).&#x20;
* Left “per-release” enums (001/002/004/005/006/040/042/073…) typed correctly, with a clear note that domains are enumerated in the shipped registry for the tag.&#x20;

