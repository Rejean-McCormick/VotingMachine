# **Doc 5A — Pipeline: State Machine & Data Exchange (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **canonical run pipeline**, its **states**, **transitions**, and the **data exchanged** between stages. This is the engine’s single source of truth for execution order and I/O. Normative wherever it affects reproducibility.

Inputs: `DivisionRegistry`, `BallotTally`, `ParameterSet`  
 Outputs: `Result`, `RunRecord`, optional `FrontierMap`  
 Ordering, canonical JSON, RNG, and FID rules: see Docs **1A–1B–3A–3B**, algorithm details in **4A–4C**, variables in **2A–2C**.

---

## **2\) State machine (canonical)**

### **2.1 States & transitions**

S0  INIT & LOAD  
    └─\> S1 VALIDATE  
          ├─(fail)-\> E\_VALIDATE  
          └─\> S2 MANIFEST & SEED  
                └─\> S3 PER-UNIT LOOP  
                      ├─\> S3.1 GATES (4B)  
                      │     ├─invalid→ S3.4 EMIT-UNIT-INVALID  
                      │     └─valid→   S3.2 FRONTIER HOOK (4C)  
                      │                   └─\> S3.3 ALLOCATE (4A)  
                      │                         ├─tie?→ S3.3a TIES (4C)  
                      │                         └─\> S3.4 EMIT-UNIT  
                      └─(all units done)→ S4 AGGREGATE & LABELS (4C/7)  
                            └─\> S5 BUILD ARTIFACTS  
                                  └─\> S6 SELF-VERIFY  
                                        ├─(fail)-\> E\_VERIFY  
                                        └─\> S7 DONE

### **2.2 Stage purposes**

* **S0 INIT & LOAD** — Open/read three inputs; normalize to in-memory canonical forms.

* **S1 VALIDATE** — Enforce Doc 1B schemas, referential integrity, and ordering preconditions.

* **S2 MANIFEST & SEED** — Build **Normative Manifest**, compute **FID**, capture `nm_digest`; stash `VM-VAR-052` (no RNG draws yet).

* **S3 PER-UNIT LOOP** — Deterministic iteration over units (ascending `unit_id`):

  * **S3.1 GATES** (Doc 4B) — sanity → eligibility → validity → frontier pre-check.

  * **S3.2 FRONTIER HOOK** (Doc 4C) — evaluate 040–042 (+047–049) if enabled.

  * **S3.3 ALLOCATE** (Doc 4A) — compute allocations; if tie, go to **S3.3a TIES** (Doc 4C with 050/052).

  * **S3.4 EMIT-UNIT** — append per-unit result record; optional frontier diagnostics.

* **S4 AGGREGATE & LABELS** — compute national/aggregate metrics; compute labels with 060/061 (presentation-only).

* **S5 BUILD ARTIFACTS** — assemble canonical `Result`, `RunRecord`, optional `FrontierMap`; set IDs and `formula_id`.

* **S6 SELF-VERIFY** — recompute hashes/IDs; fail if mismatch.

* **S7 DONE** — exit `0`.

**Error states**  
 `E_VALIDATE` (exit 2): schema/domain/ref/order errors.  
 `E_VERIFY` (exit 3): any post-emit hash/FID mismatch.  
 Other runtime I/O/parse issues: exit 4; spec violation (e.g., RNG misuse): exit 5 (Doc 3A).

---

## **3\) Pipeline context & data exchange**

### **3.1 RunContext (in-memory, normative fields)**

{  
  "registry": { /\* DivisionRegistry canonical \*/ },  
  "tally":    { /\* BallotTally canonical \*/ },  
  "params":   { /\* ParameterSet canonical \*/ },

  "normative\_manifest": { /\* Included rules & 2A/2C values \*/ },  
  "formula\_id": "\<64hex\>",  
  "nm\_digest": { "schema\_version":"1.x", "nm\_sha256":"\<64hex\>" },

  "engine": { "vendor":"...", "name":"...", "version":"vX.Y.Z", "build":"commit:..." },

  "rng": { "seed": 0, "used": false },  // 052; used=true iff a random tie occurred

  "frontier\_enabled": true,  
  "frontier\_map\_enabled": true,         // 034  
  "sensitivity\_enabled": false,         // 035 (appendix only)

  "per\_unit": {  
    "U-001": {  
      "gate\_status": "valid|invalid",  
      "reasons": \[ /\* ordered tokens per 4B §5 \*/ \],  
      "frontier": { "band\_met": true, "band\_value": 0.12 },   // if enabled  
      "allocations": \[ /\* ordered by order\_index \*/ \],  
      "label": "Decisive|Marginal|Invalid"  
    }  
    /\* ... \*/  
  },

  "run\_record\_scaffold": {  
    "inputs\_sha256": { "registry":"\<64hex\>", "tally":"\<64hex\>", "params":"\<64hex\>" },  
    "vars\_effective": { "VM-VAR-\#\#\#": \<value\>, /\* all outcome-affecting \*/ },  
    "determinism": { "tie\_policy":"...", "rng\_seed": 0? },  
    "ties": \[ /\* events appended in unit order \*/ \]  
  }  
}

### **3.2 Artifact construction (S5)**

* **Result** pulls `formula_id`, `engine_version`, aggregates, and `per_unit[*].allocations/label`.

* **RunRecord** pulls `engine`, `inputs_sha256`, `nm_digest`, `vars_effective`, `determinism`, `ties`, and any per-unit gate summaries (4B §6).

* **FrontierMap** is emitted iff `frontier_map_enabled=true` **and** frontier evaluated in run; array ordered by `unit_id`.

---

## **4\) Stage contracts (what each stage MUST do)**

| Stage | Consumes | Produces | Determinism/notes |
| ----- | ----- | ----- | ----- |
| **S0 INIT & LOAD** | file paths | `registry`,`tally`,`params` | No network; parse to canonical in-memory forms (Doc 1A §2.1). |
| **S1 VALIDATE** | S0 outputs | — | Enforce Doc 1B domains; establish Option `order_index` uniqueness; fail → E\_VALIDATE. |
| **S2 MANIFEST & SEED** | S0,S1 | `normative_manifest`,`formula_id`,`nm_digest`,`rng.seed` | FID built from Included set (Annex A); stash **052**; do not draw RNG. |
| **S3.1 GATES** | ctx \+ 2A/2C | `gate_status`,`reasons` | Evaluate in fixed order (4B §2); never short-circuit recording of reasons. |
| **S3.2 FRONTIER HOOK** | ctx \+ 040–042 (+047–049) | `frontier.*` | If `040!="none"`; invalid config → treat as 4B validity failure. |
| **S3.3 ALLOCATE** | ctx \+ family 001–007 (+073) | `allocations[]` | Deterministic; obey option order. |
| **S3.3a TIES** | 050,052 | adjusted `allocations[]`, `ties[]`, `rng.used=true` | Consume **k** draws for a **k-way** tie; never draw otherwise. |
| **S3.4 EMIT-UNIT** | unit ctx | append to `per_unit` | Unit records ordered by `unit_id`; allocations by `order_index`. |
| **S4 AGG & LABELS** | per-unit data | aggregates, labels | Labels via 060/061 (presentation only); do not alter allocations. |
| **S5 BUILD** | ctx | `Result`,`RunRecord`,`FrontierMap?` | Canonical JSON (Doc 1A); set IDs. |
| **S6 SELF-VERIFY** | artifacts | — | Recompute sha256; verify IDs; verify FID; fail → E\_VERIFY. |

---

## **5\) Canonical functions (IDs reserved; details in 5B)**

| VM-FUN | Name | Summary |
| ----- | ----- | ----- |
| **001** | `LoadInputs` | Read files; normalize JSON. |
| **002** | `ValidateInputs` | Doc 1B schema/domain/ref/order checks. |
| **003** | `ComputeNormativeManifest` | Build Included set snapshot; hash → FID; fill `nm_digest`. |
| **004** | `PrepareUnit` | Assemble unit view (registry+tally). |
| **005** | `ApplyGates` | Run gates in order; produce reasons. |
| **006** | `FrontierHook` | Evaluate 040–042 (+047–049); per-unit diagnostics. |
| **007** | `ComputeAllocations` | Deterministic allocation per family 001–007 (+073). |
| **008** | `ResolveTies` | Apply 050 policy; use 052 only when random. |
| **009** | `LabelDecisiveness` | Compute label via 060/061 (presentation). |
| **010** | `BuildResult` | Assemble canonical Result. |
| **011** | `BuildRunRecord` | Assemble canonical RunRecord (vars\_effective, ties, determinism). |
| **012** | `EmitFrontierMap` | Optional canonical FrontierMap. |
| **013** | `CompareScenarios` | Optional sensitivity appendix when 035=true (report-only). |
| **014** | `SelfVerify` | Recompute hashes/IDs; verify FID/engine disclosures. |

Function specs (inputs/outputs/pre/postconditions) are defined in **Doc 5B**.

---

## **6\) Determinism requirements (reiterated)**

* Iterate **units in ascending `unit_id`**; options **by `order_index`** (ties by `option_id`).

* Never depend on map/dict iteration order.

* RNG draws occur **only** within `ResolveTies` and **only** when `tie_policy="random"` and a tie exists; exactly **k** draws for a **k-way** tie.

* Presentation toggles (032–035, 060–062) **never** change canonical JSON or FID.

---

## **7\) Exit codes & failure mapping**

* **2** — Validation failure (Doc 1B): any schema/domain/ref/order violation.

* **3** — Self-verification failure: any artifact hash or FID mismatch.

* **4** — I/O/parse/runtime error.

* **5** — Spec violation (ordering, RNG misuse, non-canonical JSON, network I/O).

---

## **8\) Conformance checklist (5A)**

* **C-5A-ORDER**: All loops honor Doc 1A ordering; no nondeterministic aggregation.

* **C-5A-FID**: FID built from Included set (Annex A) and equals both artifact fields.

* **C-5A-RNG**: RNG seeded from **052**; no draws outside `ResolveTies`; draws counted per tie size.

* **C-5A-REC**: `RunRecord` contains `vars_effective`, `inputs_sha256`, `nm_digest`, and `ties[]` in unit order.

* **C-5A-PRES**: Labels/language (060–062) affect only presentation; artifacts remain canonical.

* **C-5A-VERIFY**: Self-verify passes before exit.

---

## **9\) Minimal pseudocode (reference)**

ctx \= LoadInputs(paths)  
ValidateInputs(ctx)                // exit 2 on failure

ctx.normative\_manifest, ctx.formula\_id, ctx.nm\_digest \= ComputeNormativeManifest(ctx.params)  
ctx.rng.seed \= VM\_VAR\_052

for unit in sort\_by\_unit\_id(ctx.registry.units):  
  u \= PrepareUnit(unit, ctx.tally)  
  valid, reasons \= ApplyGates(u, ctx.params)              // 4B  
  if \!valid:  
    record\_invalid\_unit(ctx, u, reasons)  
    continue

  if VM\_VAR\_040 \!= "none":  
    u.frontier \= FrontierHook(u, ctx.params)              // 4C

  u.allocations \= ComputeAllocations(u, ctx.params)       // 4A  
  if has\_tie(u.allocations):  
    u.allocations \= ResolveTies(u.allocations, ctx.params, ctx.rng)  // 4C

  u.label \= LabelDecisiveness(u, ctx.params)              // 4C (presentation)  
  append\_unit(ctx, u)

Result \= BuildResult(ctx)  
RunRecord \= BuildRunRecord(ctx)  
if VM\_VAR\_034: FrontierMap \= EmitFrontierMap(ctx)

SelfVerify(Result, RunRecord, FrontierMap?)               // exit 3 on failure  
exit 0

*End Doc 5A.*

# **Doc 5B — Canonical Function Specs (Updated, Normative)**

**Scope.** This part defines each pipeline function’s **inputs**, **outputs**, **pre/postconditions**, **side-effects**, and **determinism** rules. Function IDs are stable. JSON formats & ordering: Doc 1A. Variables: Docs 2A/2B/2C (+ Annex A). Platform & RNG: Doc 3A. Release: Doc 3B.

---

## **VM-FUN-001 `LoadInputs(paths)` *(S0)***

**In:** file paths `{registry, tally, params}`  
 **Out:** in-memory canonical objects `{registry, tally, params}`  
 **Pre:** paths exist; readable  
 **Post:**

* Parsed to canonical in-mem forms (UTF-8, LF, sorted keys when re-emitting).

* No network I/O.  
   **Fail:** exit 4 on I/O/parse error.  
   **Determinism:** independent of locale/timezone.

---

## **VM-FUN-002 `ValidateInputs(ctx)` *(S1)***

**In:** `{registry, tally, params}`  
 **Out:** none (throws on error)  
 **Pre:** FUN-001 done  
 **Checks (Doc 1B):** schema domains, referential integrity, uniqueness of `order_index` per unit; non-negativity; vote sums ≤ valid ballots.  
 **Fail:** exit 2 with first error code; MUST list **all** per-unit reasons if aggregating.  
 **Determinism:** error listing order \= ascending VM-VAR (if applicable) then lexicographic.

---

## **VM-FUN-003 `ComputeNormativeManifest(ctx)` *(S2)***

**In:** `{params}`, algorithm constants (001..007, 073\)  
 **Out:** `{normative_manifest, formula_id, nm_digest}`  
 **Pre:** FUN-002 passed  
 **Rules:**

* Manifest \= outcome-affecting rules \+ Included VM-VARs (Annex A).

* Canonicalize then `sha256 → formula_id` (64-hex).

* `nm_digest = {schema_version, nm_sha256}` for verifier use.  
   **Determinism:** identical manifest ⇒ identical FID.  
   **Fail:** exit 5 if Included list/values incomplete.

---

## **VM-FUN-004 `PrepareUnit(unit, ctx)` *(S3 loop)***

**In:** `registry.units[unit_id]`, `tally.units[unit_id]`  
 **Out:** per-unit working view `{unit_id, totals, option_rows[], flags}`  
 **Pre:** FUN-002 passed  
 **Rules:**

* Build `option_rows[]` in **Registry order** (`order_index`, then `option_id`).

* Compute base metrics needed by 4B/4C (e.g., shares, margin scaffolding).  
   **Determinism:** no map-order dependence.

---

## **VM-FUN-005 `ApplyGates(u, params)` *(S3.1)***

**In:** unit view `u`, VM-VARs **010..017, 020..029, 021, 030..031, 045**  
 **Out:** `{valid: bool, reasons[], protected_bypass?: bool, applied_exceptions[]}`  
 **Rules:** stage order **Sanity → Eligibility → Validity → Frontier pre-check**, ascending ID within stage (Doc 4B).

* `045=allow` may bypass **eligibility** only; never sanity or integrity floor (031).

* `030` overrides applied before `029` exceptions.  
   **Recording:** reasons ordered: ascending VM-VAR ID, then symbolic tokens.  
   **Fail:** none; returns `valid=false` when any gate fails.

---

## **VM-FUN-006 `FrontierHook(u, params)` *(S3.2)***

**In:** `u`, VM-VARs **040–042**, **047–049**  
 **Out:** `{band_met: bool, band_value?: number}`  
 **Rules:** apply 040/041/042; refine 047→048→049 precedence. If `040="none"`, return `{band_met:false}` without side-effects.  
 **Fail:** if config invalid/missing inputs ⇒ treat as **validity failure** per Doc 4B; producer MUST record reason `frontier_missing_inputs`.  
 **Determinism:** pure given inputs/params.

---

## **VM-FUN-007 `ComputeAllocations(u, family)` *(S3.3)***

**In:** `u`, family constants **001..007**, optional **073**  
 **Out:** `allocations[]` (ordered by Registry order)  
 **Rules:** deterministic; no RNG; no presentation vars.  
 **Fail:** exit 5 on under-specified family behavior.  
 **Determinism:** invariant to thread count.

---

## **VM-FUN-008 `ResolveTies(allocations, params, rng)` *(S3.3a)***

**In:** `allocations[]`, VM-VARs **050 (tie\_policy)**, **052 (tie\_seed)**; RNG state `{seed, used}`  
 **Out:** adjusted `allocations[]`; append to `RunRecord.ties[]`; set `rng.used=true` iff random tie occurred  
 **Policy:**

* `status_quo` ⇒ apply family rule; no RNG.

* `deterministic_order` ⇒ sort tied subset by `order_index`, then `option_id`; **no VM-VAR controls order key**; **051 reserved**.

* `random` ⇒ seed RNG with **052** (once per run). For a k-way tie draw exactly **k** 64-bit values, sort tied subset by `(draw, option_id)`.  
   **Side-effects:** add `{unit_id, type, policy, seed?}` to `RunRecord.ties[]` when policy=`random`.  
   **Determinism:** draw counts MUST equal **k per tie**; no draws otherwise.  
   **Fail:** exit 5 if RNG used outside `random` policy or draw count deviates.

---

## **VM-FUN-009 `LabelDecisiveness(u, params)` *(S4)***

**In:** per-unit metrics, `allocations[]`; VM-VARs **060 (threshold)**, **061 (policy)**  
 **Out:** `"Decisive" | "Marginal" | "Invalid"`  
 **Rules:** presentation-only; does not alter allocations or any hash input.

* `fixed` ⇒ label by margin ≥ 060\.

* `dynamic_margin` ⇒ label by margin & blocking flags (deterministic booleans from earlier stages).  
   **Determinism:** identical inputs ⇒ identical label.  
   **Fail:** none (fallback to `Marginal` if inputs insufficient and unit not invalid).

---

## **VM-FUN-010 `BuildResult(ctx)` *(S5)***

**In:** per-unit records, `formula_id`, `engine.version`  
 **Out:** canonical `Result` JSON object  
 **Rules:**

* Units ordered by `unit_id`; allocations by `order_index`.

* Include aggregates & label per Doc 4A/4C/7; exclude diagnostics.

* Compute `result_id = "RES:"+sha256(canonical(Result))`.  
   **Fail:** exit 5 on ordering/canonicalization violation.

---

## **VM-FUN-011 `BuildRunRecord(ctx)` *(S5)***

**In:** `engine{vendor,name,version,build}`, input digests, `nm_digest`, `formula_id`, `params`, tie events, per-unit gate summaries  
 **Out:** canonical `RunRecord` JSON object  
 **Rules:**

* `vars_effective` MUST list **all outcome-affecting** VM-VARs actually used; presentation vars MAY be included.

* `determinism.tie_policy` reflects **050**; `determinism.rng_seed` present iff any random tie occurred (value \= **052** used).

* Compute `run_id = "RUN:"+<ts>+"-"+sha256(canonical(RunRecord))`.  
   **Fail:** exit 5 if any required field missing or non-canonical.

---

## **VM-FUN-012 `EmitFrontierMap(ctx)` *(S5, optional)***

**In:** per-unit frontier diagnostics; VM-VAR-034  
 **Out:** canonical `FrontierMap` JSON (if emitted)  
 **Rules:** emit only when `034=true` **and** frontier evaluated. Use `band_met` field name.

* Compute `frontier_id = "FR:"+sha256(canonical(FrontierMap))`.  
   **Fail:** none (skip emission if disabled or unused).

---

## **VM-FUN-013 `CompareScenarios(ctx)` *(S4/S7 appendix, optional)***

**In:** base context; a fixed set of diagnostic scenario deltas (implementation-defined, non-FID) gated by **VM-VAR-035=true**  
 **Out:** report-only appendix data; **must not** alter `Result` or `RunRecord` hashes  
 **Rules:**

* Runs as a **separate sandbox** after canonical artifacts are built.

* No changes to canonical JSON; renderer may display appendix.  
   **Fail:** non-fatal; appendix omitted on error.

---

## **VM-FUN-014 `SelfVerify(Result, RunRecord, FrontierMap?)` *(S6)***

**In:** artifacts  
 **Out:** none (throws on mismatch)  
 **Checks:**

* Recompute sha256 and compare to `result_id`, `run_id`, `frontier_id?`.

* Independently recompute FID from `nm_digest`/manifest equals `Result.formula_id` and `RunRecord.formula_id`.  
   **Fail:** exit 3 on any mismatch.

---

## **Shared conventions (all functions)**

* **No network** during official runs (Doc 3A).

* **Ordering**: arrays must follow Doc 1A §5 (units by `unit_id`; options by `order_index`, then `option_id`).

* **RNG**: only FUN-008 may consume draws, and only under `tie_policy="random"`.

* **Error classification**:

  * Spec violation / determinism breach ⇒ exit 5\.

  * Validation (inputs) ⇒ exit 2\.

  * Hash/FID mismatch ⇒ exit 3\.

  * I/O/parse ⇒ exit 4\.

---

## **Minimal I/O signatures (reference)**

001 LoadInputs(paths) \-\> ctx.{registry,tally,params}  
002 ValidateInputs(ctx) \-\> void | exit 2  
003 ComputeNormativeManifest(ctx) \-\> {manifest, formula\_id, nm\_digest}  
004 PrepareUnit(unit, ctx) \-\> unit\_ctx  
005 ApplyGates(unit\_ctx, params) \-\> {valid, reasons\[\], protected\_bypass?, applied\_exceptions\[\]}  
006 FrontierHook(unit\_ctx, params) \-\> {band\_met, band\_value?}  
007 ComputeAllocations(unit\_ctx, family) \-\> allocations\[\]  
008 ResolveTies(allocations, params, rng) \-\> allocations'\[\], ties\[\], rng.used?  
009 LabelDecisiveness(unit\_ctx, params) \-\> "Decisive"|"Marginal"|"Invalid"  
010 BuildResult(ctx) \-\> Result  
011 BuildRunRecord(ctx) \-\> RunRecord  
012 EmitFrontierMap(ctx) \-\> FrontierMap?  
013 CompareScenarios(ctx) \-\> AppendixData (non-canonical)  
014 SelfVerify(Result, RunRecord, FrontierMap?) \-\> void | exit 3

*End Doc 5B.*

# **Doc 5C — Audit Data, TieLog & Non-Canonical Appendices (Updated, Normative where stated)**

## **1\) Purpose & scope**

Completes **Doc 5** by fixing the **audit data model**, **TieLog**, and the **optional diagnostics/appendices** that never affect outcomes.

* **Normative**: anything that lands in canonical artifacts (`Result`, `RunRecord`, optional `FrontierMap`) or constrains determinism.

* **Informative**: optional diagnostics/appendices emitted outside canonical artifacts.

Inputs/outputs, ordering, hashing, FID: Docs **1A–1B–3A–3B**. Algorithm: **4A–4C**. Variables: **2A–2C** (+ Annex A).

---

## **2\) Canonical audit content (lives inside `RunRecord`)**

### **2.1 Determinism block (normative)**

"determinism": {  
  "tie\_policy": "status\_quo|deterministic\_order|random",   // mirrors VM-VAR-050  
  "rng\_seed": 424242                                       // present iff any random tie occurred; value \= VM-VAR-052  
}

* `rng_seed` MUST be omitted if no random tie occurred.

### **2.2 Effective variables (normative)**

"vars\_effective": {  
  "VM-VAR-001": "...",  
  ...  
  "VM-VAR-050": "status\_quo",  
  "VM-VAR-052": 0,  
  /\* all outcome-affecting variables included; presentation (e.g., 060–062) MAY be echoed \*/  
}

Rules:

* MUST include **all outcome-affecting** VM-VARs (Annex A “Included”).

* **Presentation/report toggles** (032–035, 060–062) do **not** affect FID; echoing them is optional.

### **2.3 TieLog (normative)**

Array ordered by **ascending `unit_id`**, then event creation order within the unit evaluation.

Event schema:

{  
  "unit\_id": "U-001",  
  "type": "winner\_tie|rank\_tie|other",  
  "policy": "status\_quo|deterministic\_order|random",  
  "seed": 424242               // present only when policy="random"  
}

Rules:

* An entry is added **only** when a tie actually affects allocation/ranking (Doc 4C §3).

* For `random`, engine consumes **exactly k draws** for a k-way tie and records `seed`.

### **2.4 Gate summary (normative)**

Per-unit gate outcome, if producer opts to embed it in `RunRecord`:

{  
  "unit\_id": "U-001",  
  "gate\_status": "valid|invalid",  
  "reasons": \[ "VM-VAR-020:min\_turnout", "VM-VAR-031:integrity\_floor" \],  
  "protected\_bypass": true,                 // only if 045=allow bypassed an eligibility gate  
  "applied\_exceptions": \[ "VM-VAR-029:U-001" \],  
  "frontier\_ready": true                    // frontier pre-check result if frontier is enabled  
}

Ordering:

* `reasons[]` sorted by **VM-VAR numeric ID**, then lexical for symbolic tokens (Doc 4B §5).

* The array of these unit records is ordered by **`unit_id`**.

### **2.5 Inputs digest scaffold (normative)**

"inputs": {  
  "division\_registry\_sha256": "\<64hex\>",  
  "ballot\_tally\_sha256": "\<64hex\>",  
  "parameter\_set\_sha256": "\<64hex\>"  
},  
"nm\_digest": { "schema\_version": "1.x", "nm\_sha256": "\<64hex\>" }

* All digests computed over **canonical JSON** (Doc 1A §2.1).

---

## **3\) `FrontierMap` (optional canonical artifact, recap)**

* Emitted **only** if `VM-VAR-034 = true` and frontier executed in the run.

* Schema per Doc 1A §4.6; field is **`band_met`**.

* Units ordered by `unit_id`. Presence/absence never alters outcomes.

---

## **4\) Non-canonical diagnostics & appendices (informative)**

Diagnostics here are **outside** the hashed canonical artifacts. They MUST NOT change `Result`/`RunRecord` IDs or FID.

### **4.1 Sensitivity appendix (gated by `VM-VAR-035`)**

* Runs **after** canonical artifacts are finalized.

Explores a fixed, documented set of scenario deltas (implementation-defined), e.g.:

 {  
  "scenarios": \[  
    { "name": "Turnout+1pp", "deltas": { "counterfactual\_turnout": "+1pp" } },  
    { "name": "Turnout-1pp", "deltas": { "counterfactual\_turnout": "-1pp" } }  
  \],  
  "results": \[  
    { "name": "Turnout+1pp", "summary\_diff": { /\* report-only \*/ } }  
  \]  
}

*   
* Emission location is a renderer concern (appendix PDF/HTML/JSON).

* MUST NOT write into canonical `Result` or `RunRecord`.

### **4.2 Debug traces (developer mode)**

* Optional JSON/NDJSON with per-stage timings, thread counts, and intermediate metrics.

* File naming SHOULD avoid collisions (e.g., `debug_trace.ndjson`).

* MUST NOT be read by the renderer for official reports; MUST NOT influence canonical artifacts.

---

## **5\) ParameterSet export & echo policy**

* **Input ParameterSet** is hashed into `parameter_set_sha256`.

* **Echo rules**:

  * All **outcome-affecting** VM-VARs MUST be echoed in `vars_effective`.

  * **Tie controls**: `VM-VAR-050` **and** `VM-VAR-052` are echoed; `051` is **reserved** (no value).

  * **Presentation VM-VARs** (032–035, 060–062) MAY be echoed for transparency but are **excluded from FID**.

* **CLI overrides** (e.g., `--seed N`) MUST be reflected in `vars_effective` and, if used for random ties, in `determinism.rng_seed`.

---

## **6\) Error mapping & exit codes (recap, normative)**

* **Validation failure** (Doc 1B): exit **2**; no canonical artifacts emitted.

* **Self-verification failure** (hash/FID mismatch): exit **3**.

* **I/O or parse** errors: exit **4**.

* **Spec violation** (ordering, RNG misuse, network I/O): exit **5**.

---

## **7\) Conformance checklist (5C)**

* **C-5C-VARS**: `vars_effective` lists every outcome-affecting VM-VAR actually used (Annex A “Included”); presentation vars optional.

* **C-5C-TIELOG**: Each recorded tie corresponds to an actual allocation/ranking tie; `seed` present only for `random`.

* **C-5C-GATESUM**: Gate summaries (if emitted) follow ordering and tokenization rules; reasons are complete and deterministic.

* **C-5C-FRMAP**: `FrontierMap` uses `band_met`; units ordered by `unit_id`; emission gated by `VM-VAR-034`.

* **C-5C-APPX**: Sensitivity/debug outputs do not alter canonical artifacts or their hashes.

---

## **8\) Minimal example fragments**

**RunRecord (excerpt)**

{  
  "determinism": { "tie\_policy": "random", "rng\_seed": 424242 },  
  "vars\_effective": {  
    "VM-VAR-050": "random",  
    "VM-VAR-052": 424242,  
    "VM-VAR-040": "banded",  
    "VM-VAR-041": 0.10,  
    "VM-VAR-042": "apply\_on\_entry"  
  },  
  "ties": \[  
    { "unit\_id": "U-003", "type": "winner\_tie", "policy": "random", "seed": 424242 }  
  \],  
  "summary\_units": \[  
    {  
      "unit\_id": "U-003",  
      "gate\_status": "valid",  
      "reasons": \[\],  
      "frontier\_ready": true  
    }  
  \]  
}

**FrontierMap (excerpt)**

{  
  "frontier\_id": "FR:\<64hex\>",  
  "units": \[  
    { "unit\_id": "U-001", "band\_met": true, "band\_value": 0.12, "notes": "within band-1" }  
  \]  
}

*End Doc 5C.*

