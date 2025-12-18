# **Doc 1A — Database Definition: Entities & IDs (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **canonical data model** (entities, fields, relationships) and **identifier rules** for all persistent artifacts produced or consumed by the counting engine and report renderer. This part is **normative**. It integrates the former addenda (FID/canonicalization and cross-refs) so no separate addendum is required.

Applies to local, offline runs. With the **same inputs** and the **same ParameterSet** (including seeds), outputs must be **byte-identical** across OS/arch.

---

## **2\) Canonical serialization & identifiers**

### **2.1 Canonical JSON**

All artifacts are JSON with the following canonical form (used both for storage and hashing):

* **UTF-8**, Unix newlines (**LF**), no BOM.

* **Sorted keys** at every object level (ascending Unicode code point).

* **Numbers** emitted as JSON numbers (no trailing zeros beyond what the algorithm outputs).

* **Stable ordering** for arrays as defined in §5 (never “natural”/incidental order).

### **2.2 Artifact files and IDs**

The engine emits up to three canonical artifacts per run:

* `result.json` — final outcome (**Result**).  
   `result_id`: `"RES:" + <sha256-64hex of canonical result.json>`

* `run_record.json` — provenance & audit (**RunRecord**).  
   `run_id`: `"RUN:" + <UTC-compact-timestamp> + "-" + <sha256-64hex of canonical run_record.json>`

* `frontier_map.json` — optional diagnostic (**FrontierMap**).  
   `frontier_id`: `"FR:" + <sha256-64hex of canonical frontier_map.json>`

### **2.3 Formula ID (FID)**

* **FormulaID** is a **64-hex sha256** over the **Normative Manifest** (the set of outcome-affecting rules and defaults).

* **Inclusion**: only variables/rules that can change outcomes.

* **Exclusion**: presentation/reporting toggles (e.g., labels, display language, formatting) are **not** in FID.

* **Where recorded**: `Result.formula_id` and `RunRecord.formula_id`.

* **Verification aid**: `RunRecord.nm_digest` (see §4.5) enables third parties to recompute the FID.

Full variable coverage (what’s in/out of FID) is centralized in **Annex A — VM-VAR Registry**. This doc references those IDs where needed (e.g., ties).

---

## **3\) Entity overview & relationships**

**Inputs (consumed)**

* **DivisionRegistry** — stable universe of Units and Options (and metadata).

* **BallotTally** — counts by Unit/Option (already validated & normalized).

* **ParameterSet** — effective variable values by **VM-VAR-ID**.

**Outputs (produced)**

* **Result** — elected/allocated outputs and derived metrics.

* **RunRecord** — immutably documents inputs, engine, variables, seeds, hashes.

* **FrontierMap** *(optional)* — per-Unit diagnostics for frontier/band gating.

* **TieLog** *(embedded in RunRecord)* — events when ties are encountered.

**Key relationships**

* **Result** references Units/Options defined in **DivisionRegistry**.

* **RunRecord** references **Result** (by `result_id`) and includes digests of all inputs.

* **FrontierMap** references Units and the gating bands used in the algorithm.

---

## **4\) Schemas (normative)**

Field names are **snake\_case**. “Required” means the field MUST be present.

### **4.1 DivisionRegistry (input)**

Purpose: authoritative catalog of **units** (e.g., districts) and their **options** (e.g., parties/candidates).

Minimum schema:

{  
  "schema\_version": "1.x",  
  "units": \[  
    {  
      "unit\_id": "U-001",            // stable ID (string)  
      "name": "District 1",  
      "protected\_area": false,       // if true, gating rules may apply  
      "options": \[  
        {  
          "option\_id": "O-A1",       // stable ID (string)  
          "name": "Option A",  
          "order\_index": 1           // integer; see §5 for determinism  
        }  
      \]  
    }  
  \]  
}

Constraints:

* `unit_id` and `option_id` are **unique** and **stable** across runs.

* Each `options[].order_index` is **unique within its unit** and a **non-negative integer**.

### **4.2 BallotTally (input)**

Purpose: per-unit counts for each option (already validated upstream).

Minimum schema:

{  
  "schema\_version": "1.x",  
  "units": \[  
    {  
      "unit\_id": "U-001",  
      "totals": {  
        "valid\_ballots": 12345,  
        "invalid\_ballots": 67  
      },  
      "options": \[  
        { "option\_id": "O-A1", "votes": 6000 },  
        { "option\_id": "O-B1", "votes": 5000 }  
      \]  
    }  
  \]  
}

Constraints:

* Every `unit_id` and `option_id` MUST exist in **DivisionRegistry**.

* Totals are non-negative and consistent (engine validation rules apply in tests).

### **4.3 ParameterSet (input)**

Purpose: effective **VM-VAR** values for this run.

Schema:

{  
  "schema\_version": "1.x",  
  "vars": {  
    "VM-VAR-050": "status\_quo",   // tie\_policy  
    "VM-VAR-052": 0,              // tie\_seed (used only if policy=random)  
    "VM-VAR-060": 55,             // majority label threshold (%), presentation  
    "VM-VAR-061": "dynamic\_margin", // label policy, presentation  
    "VM-VAR-062": "auto"          // unit display language, presentation  
    // ... other outcome-affecting variables per Annex A  
  }  
}

Rules:

* **Tie controls**:  
   `VM-VAR-050` (**tie\_policy**) ∈ {`status_quo`, `deterministic_order`, `random`}.  
   `VM-VAR-052` (**tie\_seed**) is an integer ≥ 0; **only used** if policy=`random`.  
   `VM-VAR-051` is **reserved** (no variable; deterministic\_order always uses `option.order_index`).

* **Presentation variables** (e.g., `060–062`) MUST be recorded in `ParameterSet` for transparency but are **excluded from FID**.

* All outcome-affecting variables listed in **Annex A (Included)** MUST appear explicitly (no implicit defaults when hashing the FID).

### **4.4 Result (output)**

Purpose: canonical, minimal outcome record used for hashing and reporting.

Minimum schema:

{  
  "result\_id": "RES:\<64hex\>",  
  "formula\_id": "\<64hex\>",  
  "engine\_version": "vX.Y.Z",  
  "created\_at": "2025-08-12T14:00:00Z",  
  "summary": { /\* global metrics, turnout, thresholds actually used, etc. \*/ },  
  "units": \[  
    {  
      "unit\_id": "U-001",  
      "allocations": \[  
        { "option\_id": "O-A1", "votes": 6000, "share": 0.545 },  
        { "option\_id": "O-B1", "votes": 5000, "share": 0.455 }  
      \],  
      "label": "Decisive"   // derived; presentation logic uses VM-VAR-060/061  
    }  
  \]  
}

Rules:

* `result_id` is computed **after** canonicalization (see §2.2).

* Arrays follow **ordering rules** in §5.

### **4.5 RunRecord (output)**

Purpose: verifiability & full provenance.

Minimum schema:

{  
  "run\_id": "RUN:\<ts\>-\<64hex\>",  
  "result\_id": "RES:\<64hex\>",  
  "formula\_id": "\<64hex\>",  
  "engine": {  
    "vendor": "acme.labs",       // fork identifier  
    "name": "vm\_engine",  
    "version": "vX.Y.Z",  
    "build": "commit:abcd1234"  
  },  
  "inputs": {  
    "division\_registry\_sha256": "\<64hex\>",  
    "ballot\_tally\_sha256": "\<64hex\>",  
    "parameter\_set\_sha256": "\<64hex\>"  
  },  
  "nm\_digest": {  
    "schema\_version": "1.x",  
    "nm\_sha256": "\<64hex\>"       // digest of the Normative Manifest used to compute FID  
  },  
  "vars\_effective": {            // echo of effective VM-VARs actually used  
    "VM-VAR-050": "status\_quo",  
    "VM-VAR-052": 0,  
    /\* ... all outcome-affecting variables; presentation vars may be listed as well \*/  
  },  
  "determinism": {  
    "rng\_seed": 0,               // same as VM-VAR-052 when policy=random; omitted otherwise  
    "tie\_policy": "status\_quo"   // from VM-VAR-050  
  },  
  "ties": \[  
    { "unit\_id": "U-002", "type": "winner\_tie", "policy": "random", "seed": 424242 }  
  \]  
}

Rules:

* `vars_effective` MUST include the **exact values** used at runtime for every **outcome-affecting** variable (Annex A Included).  
   Presentation variables SHOULD be echoed for transparency.

* `rng_seed` MUST be present **only** when `tie_policy="random"` was invoked at least once.

* `engine.vendor/name/version` are **required** to identify forks and builds.

### **4.6 FrontierMap (optional output)**

Purpose: diagnostics for frontier/band gating decisions.

Schema (excerpt):

{  
  "frontier\_id": "FR:\<64hex\>",  
  "units": \[  
    {  
      "unit\_id": "U-001",  
      "band\_met": true,          // boolean (normalized name)  
      "band\_value": 0.12,        // numeric metrics used by gates  
      "notes": "within band-1 threshold"  
    }  
  \]  
}

Rules:

* Field name is **`band_met`** (not “band met” or other variants).

* Units are ordered per §5.

---

## **5\) Global ordering & determinism**

To guarantee byte-identical outputs:

* **Units** arrays are ordered by ascending `unit_id` (string compare).

* **Options within a Unit** are ordered by ascending `order_index`; ties by ascending `option_id`.

* **Allocations** and similar per-unit arrays reflect the **same option order**.

* **Ties**

  * If `tie_policy = status_quo`: apply the policy as defined by the algorithm (no reordering).

  * If `tie_policy = deterministic_order`: break ties by ascending `order_index` (no extra variable).

  * If `tie_policy = random`: break ties using the deterministic RNG seeded with `VM-VAR-052`; record events in `RunRecord.ties[]`.

---

## **6\) Validation & integrity constraints**

* **Referential integrity**: every `unit_id`/`option_id` in **BallotTally**, **Result**, **FrontierMap** MUST exist in **DivisionRegistry**.

* **Hash integrity**:

  * `result_id`, `run_id`, `frontier_id` MUST match the sha256 of the **canonical** payloads (see §2.1).

  * `RunRecord.inputs.*_sha256` MUST match the canonical inputs used.

* **FID integrity**:

  * `Result.formula_id` and `RunRecord.formula_id` MUST match the FID recomputed from **Annex A (Included)** and the algorithm code at the declared `engine.version`.

  * Presentation toggles (e.g., `VM-VAR-060..062`) MUST NOT participate in FID hashing.

* **Determinism**: with identical inputs \+ ParameterSet (including `VM-VAR-052` when applicable) on any supported platform, artifacts must be **byte-identical**.

---

## **7\) File layout & naming**

Default output filenames in the run directory:

* `result.json`, `run_record.json`, optionally `frontier_map.json`

* It is permitted to emit compressed mirrors (e.g., `.json.zst`) **in addition** to canonical JSON, but IDs and hashes are computed over the **canonical JSON**.

---

## **8\) Notes for implementers**

* Keep **DivisionRegistry** and **BallotTally** schemas stable; additions MUST be strictly additive and non-reordering.

* Treat **`order_index`** as a **hard determinism primitive**; never infer order from display names or input order.

* Echo the **effective** values you actually used in `RunRecord.vars_effective` (no “implicit defaults” during hashing).

* If you extend diagnostics, do so under **new fields**; never mutate existing canonical fields or their ordering semantics.

---

### **Appendix: VM-VAR touchpoints referenced in this part**

* **Ties**: `VM-VAR-050 tie_policy`, `VM-VAR-052 tie_seed` (051 reserved).

* **Presentation (excluded from FID)**: `VM-VAR-060 majority_label_threshold`, `VM-VAR-061 decisiveness_label_policy`, `VM-VAR-062 unit_display_language`.

---

# **Doc 1B — Field Catalog & Validation Constraints (Updated, Normative)**

## **1\) Purpose & scope**

Defines **every field**, its **type/domain**, **cardinality/keys**, **size limits**, and **validation rules** for all artifacts named in Doc 1A. No legacy or back-compat paths are provided.

Artifacts covered: `DivisionRegistry`, `BallotTally`, `ParameterSet`, `Result`, `RunRecord`, `FrontierMap`. JSON is canonical as per Doc 1A (§2.1). All field names are `snake_case`.

---

## **2\) Identifiers, strings, numbers**

**2.1 IDs & hashes**

* `unit_id`, `option_id`: non-empty strings, max **64** chars; allowed: `A–Z a–z 0–9 _ - : .`.

* `result_id` \= `"RES:"` \+ **64-hex** (lowercase).  
   `run_id` \= `"RUN:"` \+ `<UTC-compact-ISO8601>` \+ `-` \+ **64-hex**.  
   `frontier_id` \= `"FR:"` \+ **64-hex**.

* All sha256 digests are **64 lowercase hex**.

**2.2 Strings**

* Unicode **UTF-8**, NFC-normalized.

* No leading/trailing spaces. No control chars except LF in free-text `notes`.

**2.3 Numbers**

* Integers: 64-bit signed non-negative unless noted.

* Percentages stored as integers in **0…100** (%).

* Ratios/shares as JSON numbers; engine sets precision; reporters round per Doc 7\.

**2.4 Date/times**

* `created_at` in **UTC**, RFC 3339/ISO 8601 (e.g., `2025-08-12T14:00:00Z`).

---

## **3\) Entity catalog (fields, domains, keys)**

### **3.1 DivisionRegistry (input)**

{  
  "schema\_version": "1.x",  
  "units": \[ { ... } \]  
}

* `schema_version` — string, required.

* `units[]` — array ≥ 1, required. **Order**: ascending `unit_id`.

**Unit object**

* `unit_id` — string, required, **PK** in this document.

* `name` — string 1..200, required.

* `protected_area` — boolean, required.

* `options[]` — array ≥ 1, required. **Order**: ascending `order_index`.

**Option object**

* `option_id` — string, required, **PK** within unit.

* `name` — string 1..200, required.

* `order_index` — integer ≥ 0, **unique within unit**, required.

**Integrity**

* (`unit_id`, `option_id`) pairs define the **universe**. Must be stable across runs.

---

### **3.2 BallotTally (input)**

{  
  "schema\_version": "1.x",  
  "units": \[ { ... } \]  
}

**Unit tally**

* `unit_id` — string, required, **FK → DivisionRegistry.units.unit\_id**.

* `totals.valid_ballots` — int ≥ 0, required.

* `totals.invalid_ballots` — int ≥ 0, required.

* `options[]` — array of per-option tallies.

**Option tally**

* `option_id` — string, required, **FK → corresponding unit option\_id**.

* `votes` — int ≥ 0, required.

**Integrity**

* Sum of `options[].votes` ≤ `totals.valid_ballots`.

* Every tallied `option_id` must exist for that `unit_id`.

* Units **ordered** as in Registry; options **ordered** by Registry `order_index`.

---

### **3.3 ParameterSet (input)**

{  
  "schema\_version": "1.x",  
  "vars": { "VM-VAR-\#\#\#\#": \<value\>, ... }  
}

* `vars` is a map keyed by **VM-VAR-\#\#\#\#** strings.

* **Outcome-affecting variables** listed in Annex A/“Included in FID”: **MUST** be present with explicit values.

* **Presentation/reporting variables** (e.g., `VM-VAR-060..062`) **MAY** be present for transparency; they are **excluded from FID**.

* Domains for each `VM-VAR` are defined in **Annex A**. This doc only enforces that keys are syntactically `VM-VAR-000…999`.

---

### **3.4 Result (output)**

{  
  "result\_id": "RES:\<64hex\>",  
  "formula\_id": "\<64hex\>",  
  "engine\_version": "vX.Y.Z",  
  "created\_at": "2025-08-12T14:00:00Z",  
  "summary": { ... },  
  "units": \[ { ... } \]  
}

* `result_id` — required; sha256 over canonical `Result`.

* `formula_id` — required; **64-hex** FID per Doc 1A.

* `engine_version` — string 1..32, required.

* `created_at` — RFC3339 UTC, required.

* `summary` — object, required (global metrics; schema below).

* `units[]` — array ≥ 1, required. **Order**: ascending `unit_id`.

**summary (minimum)**

* `valid_ballots_total` — int ≥ 0, required.

* `invalid_ballots_total` — int ≥ 0, required.

* `turnout_rate` — number ≥ 0 (engine-chosen precision).

* Any thresholds/parameters that materially affected outcomes (e.g., gating bands actually used) MUST be echoed here or in `RunRecord`.

**unit result**

* `unit_id` — string, required, **FK → Registry**.

* `allocations[]` — array ≥ 1, required. **Order**: by Registry `order_index`.

* `label` — string, required (“Decisive”, “Marginal”, “Invalid”); derived.

**allocation**

* `option_id` — string, required, **FK**.

* `votes` — int ≥ 0, required.

* `share` — number in \[0,1\], required (engine precision policy).

* Optional deriveds (e.g., seats) only if relevant to the algorithm family.

---

### **3.5 RunRecord (output)**

{  
  "run\_id": "RUN:\<ts\>-\<64hex\>",  
  "result\_id": "RES:\<64hex\>",  
  "formula\_id": "\<64hex\>",  
  "engine": { "vendor": "...", "name": "...", "version": "vX.Y.Z", "build": "commit:...." },  
  "inputs": {  
    "division\_registry\_sha256": "\<64hex\>",  
    "ballot\_tally\_sha256": "\<64hex\>",  
    "parameter\_set\_sha256": "\<64hex\>"  
  },  
  "nm\_digest": { "schema\_version": "1.x", "nm\_sha256": "\<64hex\>" },  
  "vars\_effective": { "VM-VAR-\#\#\#\#": \<value\>, ... },  
  "determinism": { "tie\_policy": "status\_quo|deterministic\_order|random", "rng\_seed": 0 },  
  "ties": \[ { ... } \]  
}

* `run_id`, `result_id`, `formula_id` — required, formats as above.

* `engine.vendor`, `.name`, `.version` — non-empty strings ≤ 64, required.  
   `engine.build` — free string ≤ 128 (e.g., commit), required.

* `inputs.*_sha256` — required **64-hex**; digests of **canonical** inputs.

* `nm_digest.nm_sha256` — required **64-hex** digest of the Normative Manifest used for FID.

* `vars_effective` — required map; MUST include all **outcome-affecting** VM-VARs with the **exact** values used. Presentation vars MAY be included.

* `determinism.tie_policy` — required; mirrors `VM-VAR-050`.  
   `determinism.rng_seed` — present **only** if any tie used random policy (mirrors `VM-VAR-052` value used at runtime).

* `ties[]` — optional list of events (see below).

**tie event**

{ "unit\_id": "U-001", "type": "winner\_tie|rank\_tie|other", "policy": "status\_quo|deterministic\_order|random", "seed": 424242 }

* `seed` present only if `policy="random"` on that event.

---

### **3.6 FrontierMap (optional output)**

{  
  "frontier\_id": "FR:\<64hex\>",  
  "units": \[ { "unit\_id": "U-001", "band\_met": true, "band\_value": 0.12, "notes": "..." } \]  
}

* `frontier_id` — required if file emitted.

* `units[]` — required, **order** by ascending `unit_id`.

* `band_met` — **boolean**, required (normalized name).

* `band_value` — number (engine precision policy).

* `notes` — optional string ≤ 280\.

---

## **4\) Cross-entity invariants**

* **Referential integrity:**  
   Every `unit_id`/`option_id` in `BallotTally`, `Result`, `FrontierMap`, and `RunRecord.ties[]` **must** exist in `DivisionRegistry`.

* **Ordering invariants (determinism):**  
   Units: ascending `unit_id`. Options: ascending `order_index` (ties by `option_id`).  
   All arrays reflect these orders (Doc 1A §5).

* **Hash integrity:**  
   `result_id`, `run_id`, `frontier_id` exactly match sha256 of the **canonical** payloads.  
   `inputs.*_sha256` match canonical inputs used.

* **FID integrity:**  
   `Result.formula_id` and `RunRecord.formula_id` equal recomputed FID for the run.  
   Presentation vars (e.g., `VM-VAR-060..062`) are **excluded** from FID.

* **Non-negativity & bounds:**  
   Votes, ballots ≥ 0; percentages 0..100; shares 0..1.

---

## **5\) Indexing & size guidance (implementation-level, informative)**

* Recommended indexes if persisted in a DBMS:

  * `DivisionRegistry.units(unit_id)`; `options(unit_id, order_index)` unique.

  * `BallotTally.units(unit_id)`; `options(unit_id, option_id)`.

  * `Result.units(unit_id)`; `allocations(unit_id, option_id)`.

* File size expectations (guidance):  
   `result.json` ≤ few MB for national runs; `run_record.json` may be larger due to `vars_effective` and `ties`.

---

## **6\) Validation failures (non-exhaustive)**

| Code | Condition | Artifact |
| ----- | ----- | ----- |
| `E-DR-UNIT-DUP` | Duplicate `unit_id` in DivisionRegistry | DivisionRegistry |
| `E-DR-OPT-DUP` | Duplicate `option_id` within a unit | DivisionRegistry |
| `E-DR-ORD-UNIQ` | Duplicate `order_index` within a unit | DivisionRegistry |
| `E-BT-FK-UNIT` | `BallotTally.unit_id` missing in Registry | BallotTally |
| `E-BT-FK-OPT` | Tallied `option_id` not in unit’s options | BallotTally |
| `E-BT-SUM` | Sum of option votes exceeds `valid_ballots` | BallotTally |
| `E-PS-MISS` | Required outcome-affecting VM-VAR missing | ParameterSet |
| `E-RR-HASH` | Any recorded hash/digest does not verify | RunRecord |
| `E-RR-FID` | Reported FID cannot be recomputed | Result/RunRecord |

---

## **7\) Minimal worked examples (conformant)**

**DivisionRegistry (excerpt)**

{"schema\_version":"1.x","units":\[  
  {"unit\_id":"U-001","name":"District 1","protected\_area":false,  
   "options":\[{"option\_id":"O-A1","name":"Option A","order\_index":1},  
              {"option\_id":"O-B1","name":"Option B","order\_index":2}\]}  
\]}

**BallotTally (excerpt)**

{"schema\_version":"1.x","units":\[  
  {"unit\_id":"U-001",  
   "totals":{"valid\_ballots":11000,"invalid\_ballots":67},  
   "options":\[{"option\_id":"O-A1","votes":6000},{"option\_id":"O-B1","votes":5000}\]}  
\]}

**ParameterSet (excerpt)**

{"schema\_version":"1.x","vars":{  
  "VM-VAR-050":"status\_quo",  
  "VM-VAR-052":0,  
  "VM-VAR-060":55,  
  "VM-VAR-061":"dynamic\_margin",  
  "VM-VAR-062":"auto"  
}}

**Result.units\[0\].allocations (ordering)**

{"unit\_id":"U-001","allocations":\[  
  {"option\_id":"O-A1","votes":6000,"share":0.545},  
  {"option\_id":"O-B1","votes":5000,"share":0.455}  
\]}

**FrontierMap.units\[0\]**

{"unit\_id":"U-001","band\_met":true,"band\_value":0.12,"notes":"within band-1 threshold"}

---

### **Appendix A — Tie & presentation touchpoints (for traceability)**

* Ties are controlled by **VM-VAR-050** (policy) and **VM-VAR-052** (seed).  
   There is **no** variable for deterministic order; it always uses `order_index`.

* Presentation/report toggles (e.g., **VM-VAR-060..062**) are recorded for transparency and **excluded from FID**.

*End Doc 1B.*

# **Doc 1C — Cross-Artifact Mapping, ER & Worked Examples (Updated)**

## **1\) Purpose & scope**

Binds the schemas from **Doc 1A/1B** into a single, enforceable model: entity-relationships, lifecycle of a run, validation flows, and minimal “golden” examples. Normative where it defines constraints; illustrative where it shows examples. No legacy/back-compat paths.

---

## **2\) Entity–relationship model (canonical)**

### **2.1 Text ER (cardinalities & keys)**

DivisionRegistry  
  └── units \[1..N\]                                  PK: unit\_id  
        └── options \[1..N\]                           PK: (unit\_id, option\_id)  
                                                     UNIQUE within unit: order\_index

BallotTally  
  └── units \[1..N\]                                  FK: unit\_id → DivisionRegistry.units  
        └── options \[0..N\]                           FK: (unit\_id, option\_id) → DivisionRegistry.options

Result  
  └── units \[1..N\]                                  FK: unit\_id → DivisionRegistry.units  
        └── allocations \[1..N\]                       FK: (unit\_id, option\_id) → DivisionRegistry.options

RunRecord  
  ├── inputs.digests of canonical inputs            hashes of DivisionRegistry, BallotTally, ParameterSet  
  ├── vars\_effective VM-VAR map                     MUST include all outcome-affecting variables  
  └── ties \[0..N\]                                   unit\_id present; seed present iff policy=random

FrontierMap (optional)  
  └── units \[0..N\]                                  FK: unit\_id → DivisionRegistry.units

### **2.2 Referential rules (normative)**

* Every `BallotTally.units[].unit_id`, `Result.units[].unit_id`, `FrontierMap.units[].unit_id`, and `RunRecord.ties[].unit_id` **MUST** exist in `DivisionRegistry.units`.

* Every tallied or allocated `(unit_id, option_id)` **MUST** exist in `DivisionRegistry.options` for that unit.

---

## **3\) Run lifecycle (deterministic pipeline)**

1. **Load & canonicalize inputs**

   * Parse `DivisionRegistry`, `BallotTally`, `ParameterSet`.

   * Enforce Doc 1B domains/bounds.

   * Compute and record each input’s sha256 (64-hex) for `RunRecord.inputs`.

2. **Compute Normative Manifest → FID**

   * Collect outcome-affecting rules & VM-VARs (per Annex A “Included”).

   * Canonicalize the manifest and hash → **FormulaID (64-hex)**.

   * Presentation/report toggles (e.g., `VM-VAR-060..062`) are **excluded**.

3. **Count / allocate**

   * Apply algorithm (Doc 4). Respect `option.order_index` as the determinism key.

   * Resolve ties per `VM-VAR-050 tie_policy`; if `random`, use `VM-VAR-052 tie_seed`.

   * Record any tie event into `RunRecord.ties[]`.

4. **Render artifacts**

   * Build **Result**, **RunRecord**, optional **FrontierMap** in canonical JSON (Doc 1A §2.1).

   * Compute `result_id`, `run_id`, `frontier_id` (sha256 over canonical payloads).

   * Set `Result.formula_id` and `RunRecord.formula_id` to the FID from step 2\.

   * Populate `RunRecord.vars_effective` with **exact** outcome-affecting values used; presentation vars may be echoed.

5. **Verify & emit**

   * Self-verify all recorded hashes.

   * Emit files; optional compressed mirrors are allowed but IDs are over canonical JSON.

---

## **4\) Ordering contract (single source of determinism)**

* **Units arrays**: ascending `unit_id` (string compare).

* **Options within a Unit**: ascending `order_index`; ties by `option_id`.

* **Allocations and all per-unit arrays**: mirror Registry option order.

* **Tie policies**:

  * `status_quo` — policy per algorithm; no extra variable.

  * `deterministic_order` — break by `order_index` (no variable; `VM-VAR-051` is reserved).

  * `random` — deterministic RNG seeded by `VM-VAR-052`; record events.

---

## **5\) Validation flow (pseudo-algorithm)**

validate\_division\_registry(reg):  
  assert reg.units.length ≥ 1  
  assert unit\_id unique  
  for u in reg.units:  
    assert options.length ≥ 1  
    assert option\_id unique within u  
    assert order\_index unique within u and ≥ 0

validate\_ballot\_tally(tally, reg):  
  for ut in tally.units:  
    assert ut.unit\_id in reg.units  
    sum\_votes \= 0  
    for ot in ut.options:  
      assert (ut.unit\_id, ot.option\_id) in reg.options  
      assert ot.votes ≥ 0  
      sum\_votes \+= ot.votes  
    assert sum\_votes ≤ ut.totals.valid\_ballots

validate\_parameter\_set(ps):  
  for each REQUIRED VM-VAR in AnnexA.Included:  
    assert present and in domain  
  // presentation vars may be present; excluded from FID

validate\_referential(result, reg):  
  for ru in result.units:  
    assert ru.unit\_id in reg.units  
    for a in ru.allocations:  
      assert (ru.unit\_id, a.option\_id) in reg.options

Error codes align with Doc 1B §6.

---

## **6\) Worked “golden” examples (minimal)**

### **6.1 Inputs**

**DivisionRegistry**

{"schema\_version":"1.x","units":\[  
  {"unit\_id":"U-001","name":"District 1","protected\_area":false,  
   "options":\[  
     {"option\_id":"O-A1","name":"Option A","order\_index":1},  
     {"option\_id":"O-B1","name":"Option B","order\_index":2}  
   \]}  
\]}

**BallotTally**

{"schema\_version":"1.x","units":\[  
  {"unit\_id":"U-001",  
   "totals":{"valid\_ballots":11000,"invalid\_ballots":67},  
   "options":\[{"option\_id":"O-A1","votes":6000},{"option\_id":"O-B1","votes":5000}\]}  
\]}

**ParameterSet**

{"schema\_version":"1.x","vars":{  
  "VM-VAR-050":"status\_quo",  
  "VM-VAR-052":0,  
  "VM-VAR-060":55,  
  "VM-VAR-061":"dynamic\_margin",  
  "VM-VAR-062":"auto"  
}}

### **6.2 Outputs (canonical form excerpts)**

**Result**

{  
  "result\_id": "RES:\<64hex\>",  
  "formula\_id": "\<64hex\>",  
  "engine\_version": "vX.Y.Z",  
  "created\_at": "2025-08-12T14:00:00Z",  
  "summary": {  
    "valid\_ballots\_total": 11000,  
    "invalid\_ballots\_total": 67,  
    "turnout\_rate": 0.000  
  },  
  "units": \[  
    {  
      "unit\_id": "U-001",  
      "allocations": \[  
        {"option\_id":"O-A1","votes":6000,"share":0.545},  
        {"option\_id":"O-B1","votes":5000,"share":0.455}  
      \],  
      "label": "Decisive"  
    }  
  \]  
}

**RunRecord**

{  
  "run\_id": "RUN:\<ts\>-\<64hex\>",  
  "result\_id": "RES:\<64hex\>",  
  "formula\_id": "\<64hex\>",  
  "engine": {"vendor":"acme.labs","name":"vm\_engine","version":"vX.Y.Z","build":"commit:abcd1234"},  
  "inputs": {  
    "division\_registry\_sha256": "\<64hex\>",  
    "ballot\_tally\_sha256": "\<64hex\>",  
    "parameter\_set\_sha256": "\<64hex\>"  
  },  
  "nm\_digest": {"schema\_version":"1.x","nm\_sha256":"\<64hex\>"},  
  "vars\_effective": { "VM-VAR-050":"status\_quo", "VM-VAR-052":0 /\* ... \*/ },  
  "determinism": { "tie\_policy":"status\_quo" },  
  "ties": \[\]  
}

**FrontierMap (optional)**

{  
  "frontier\_id":"FR:\<64hex\>",  
  "units":\[{"unit\_id":"U-001","band\_met":true,"band\_value":0.12,"notes":"within band-1 threshold"}\]  
}

---

## **7\) Conformance checklist (doc-level)**

* **C-ER-01**: All FK references resolve to DivisionRegistry.

* **C-ORD-02**: All arrays follow ordering contract (§4).

* **C-HASH-03**: `result_id`, `run_id`, `frontier_id` verify against canonical payloads.

* **C-FID-04**: FID recomputes from Annex A “Included” and equals both `Result.formula_id` and `RunRecord.formula_id`.

* **C-TIE-05**: If any random tie occurred, `RunRecord.determinism.rng_seed` is present and `ties[]` contains at least one event.

* **C-PRES-06**: Presentation variables (e.g., `VM-VAR-060..062`) recorded for transparency but do **not** affect FID.

---

## **8\) Implementation notes (informative)**

* Treat `order_index` as a **hard determinism primitive**; never infer order from human labels.

* When emitting diffs or audits, diff **canonical JSON** to avoid false changes from key order or whitespace.

* If you add diagnostic fields, add new keys; **never** mutate existing canonical fields or their ordering semantics.

*End Doc 1C.*

