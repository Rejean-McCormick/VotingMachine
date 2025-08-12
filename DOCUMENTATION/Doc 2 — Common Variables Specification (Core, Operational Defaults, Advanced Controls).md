# **Doc 2A — Common Variables: Core (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **outcome-affecting variables** (“VM-VARs”) that the engine reads at runtime and that are **included in the Formula ID (FID)** per Doc 1A. This part is **normative** and replaces any prior numbering/legacy notes. Presentation/reporting toggles are **not** here (they live in Doc 2B and are excluded from FID).

Result: with the **same inputs** and the **same 2A values**, outputs are **byte-identical** across OS/arch.

---

## **2\) Canonical registry & format**

* Every variable is registered as **`VM-VAR-###`** (three digits, zero-padded).

* Values are carried in the **ParameterSet** (`ParameterSet.vars["VM-VAR-###"]`) as canonical JSON (Doc 1A §2.1).

* **Domains** (type, allowed values/ranges), **defaults**, and **FID inclusion** are centralized in **Annex A — VM-VAR Registry**.  
   Doc 2A defines *which* variables are normative core and how they are used by Docs 4/5.

---

## **3\) Inclusion policy (FID)**

* **Included**: only variables that can change outcomes (the “2A core set”).

* **Excluded**: presentation/reporting toggles (Doc 2B) — e.g., labels, language, formatting.

* FID recomputation uses the 2A core set **and** the algorithm rules (Doc 4). See Doc 1A §2.3.

---

## **4\) Core set (IDs & groups)**

Doc 2A groups the **included** variables by function. The **precise per-variable spec** (domain, default, semantics) is in **Annex A**; this section fixes **membership** and **cross-doc usage**.

### **4.1 Global algorithm & scope**

* **VM-VAR-001 … 007** — Global algorithm family/rounding/flow constants (see Annex A).  
   *Used by*: Doc 4A step order; Doc 5A state machine.

* **VM-VAR-021** — Run **scope** / inclusion guard (e.g., filter or eligibility scope).  
   *Used by*: Doc 4A preconditions; Doc 6A validity tests.

### **4.2 Thresholds & eligibility (per-unit / national)**

* **VM-VAR-010 … 017** — Eligibility/validity gates (core percentages/flags).

* **VM-VAR-020 … 029** — Outcome-affecting thresholds (e.g., minimum shares, gating cutoffs).  
   *Used by*: Doc 4B gates & edge cases; Doc 6B conformance.

### **4.3 Frontier & gating model**

* **VM-VAR-040** — Frontier/gating **mode** (model selector).

* **VM-VAR-041** — Frontier **band**/cut param(s).

* **VM-VAR-042** — Frontier **application strategy** (how bands affect flow).  
   *Used by*: Doc 4C Frontier rules; Doc 5B Frontier stage; optional FrontierMap (Doc 1A §4.6).

### **4.4 Protected & autonomy controls**

* **VM-VAR-045** — **Protected-area override** (allow/deny policy when flagged).

* **VM-VAR-046** — **Autonomy package map** (selection for autonomy ladder mode).  
   *Used by*: Doc 4C §Protected/Autonomy; Doc 6C edge-case tests.

### **4.5 Ties — pointer only (variables live in Doc 2B)**

 (Referential note): Tie controls are **VM-VAR-050** (*tie\_policy*) and **VM-VAR-052** (*tie\_seed*). They are specified and defaulted in Doc 2B. **VM-VAR-050 is Included in FID (affects outcomes); VM-VAR-052 is Excluded from FID (seed only).** The RNG algorithm/profile used for random ties is pinned in **Annex B** (`rng_profile.json`).  
 **Used by:** Doc 4C (tie resolution); Doc 5B (`VM-FUN-008 ResolveTies`); Doc 5C (TieLog); Doc 6C (determinism).

**Complete 2A membership for FID (by range):**  
 `001–007, 010–017, 020–031 (incl. 021, 029–031), 040–049, 050, 073.`

**Excluded (presentation/reporting or seed):**  
 `032–035, 052, 060–062` (see Doc 2B).

---

## **5\) Cross-doc contract (how 2A is consumed)**

| Group | Consumed by | Contract highlights |
| ----- | ----- | ----- |
| Global & scope (001–007, 021\) | Doc 4A, Doc 5A | Fix step order/rounding and run scope before any counting. |
| Thresholds (010–017, 020–029) | Doc 4B, Doc 6B | Apply before allocation; failing gates invalidate/branch as defined. |
| Frontier (040–042) | Doc 4C, Doc 5B, FrontierMap | Drive band selection and gating; emit `band_met` diagnostics if enabled. |
| Protected/Autonomy (045–046) | Doc 4C, Doc 6C | Override rules when units are protected; map autonomy packages deterministically. |
| Ties (050, 052\) | Doc 4C, Doc 5C, Doc 6C | Policy & deterministic RNG; events logged in RunRecord.ties. |

---

## **6\) Defaults & mutability (where set)**

* **Defaults** for all 2A variables are declared in **Annex A** and surfaced in **Doc 2B** tables for operational clarity.

* Any change to a **2A** default or domain is a **normative change** → **new FID** (Doc 3B change policy).

* Variables in 2A are **stable IDs**: do not renumber or repurpose.

---

## **7\) Conformance checks**

* **C-2A-INC**: ParameterSet contains explicit values for **all** 2A variables listed as “Included” in Annex A.

* **C-2A-DOM**: Each value is within the Annex A domain; engine rejects out-of-range.

* **C-2A-USE**: Engine consumes 2A variables exactly at the documented points (Doc 4/5 references).

* **C-FID-LOCK**: Recomputing FID with these variables yields `Result.formula_id` (Doc 1A §2.3).

---

## **8\) Notes for implementers**

* Treat **Annex A** as the *single source of truth* for per-variable domains & defaults; Doc 2A fixes **membership** and **usage points**.

* Do **not** add engine-specific hidden toggles that alter outcomes; propose new VM-VAR IDs via Annex A if a feature becomes normative.

* When adding diagnostics, ensure they don’t alter any 2A flow or array ordering (Doc 1A §5).

---

### **Appendix (traceability stubs)**

* **VM-VAR-040/041/042 →** Doc 4C Frontier; Doc 5B Frontier stage; FrontierMap schema (Doc 1A §4.6).

* **VM-VAR-045 →** Doc 4C Protected override; Doc 6C tests.

* **VM-VAR-050/052 →** Doc 4C Ties; Doc 5C ResolveTies; Doc 6C determinism.

*End Doc 2A.*

# **Doc 2B — Operational Defaults & Presentation (Updated)**

## **1\) Purpose & scope**

Defines the **operational defaults** and **presentation/reporting toggles** the engine and renderer read at runtime. This part integrates the former integration/addendum notes and removes legacy numbering.  
 Two classes of variables live here:

* **Outcome-affecting operational defaults** (e.g., *tie policy*). Some of these **are included** in the Formula ID (FID) per Doc 1A/Annex A.

* **Presentation/report toggles** (labels, language, layout). These are **not included** in FID.

Unless stated otherwise, values are carried in `ParameterSet.vars["VM-VAR-###"]` and are read deterministically by Docs 4/5/7. Canonicalization rules are in Doc 1A.

---

## **2\) Grouping & FID policy (at a glance)**

| Group | IDs | FID? | What they influence |
| ----- | ----- | ----- | ----- |
| A. Tie & RNG controls | **050, 051 (reserved), 052** | **Policy: YES** / **Seed: NO** | Winner/rank resolution when ties occur (Doc 4C, Doc 5C). |
| B. Pipeline/Report toggles | **032–035** | **NO** | Whether to emit diagnostic artifacts/sections; report section ordering. |
| C. Labeling & language | **060–062** | **NO** | Outcome *labels* and display language (Doc 4C labels, Doc 7). |

FID inclusion/exclusion is authoritative in **Annex A — VM-VAR Registry**. This doc mirrors that policy.

---

## **3\) Variables — specifications**

### **A) Tie & RNG controls (operational; outcome-affecting)**

These control **how** ties are resolved. They affect outcomes; thus **tie\_policy** is in FID. The **seed** is a run parameter and **not** in FID; it is recorded in RunRecord.

| ID | Name | Type / Domain | Default | FID? | Used by | Notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-050** | `tie_policy` | enum: `status_quo` | `deterministic_order` | `random` | `status_quo` | **Yes** | Doc 4C (§Ties), Doc 5C (`ResolveTies`), Doc 6C | `deterministic_order` uses `Option.order_index`; no separate variable exists for the order key. |
| **VM-VAR-051** | *(reserved)* | — | — | — | — | Intentionally unused (kept to avoid future renumbering). |
| **VM-VAR-052** | `tie_seed` | integer ≥ 0 | `0` | **No** | Doc 3A (RNG), Doc 4C (§Ties), Doc 5C, Doc 6C | Used **only** when `tie_policy= random`. Recorded in `RunRecord.determinism.rng_seed` and `RunRecord.ties[]`. |

---

### **B) Pipeline / report toggles (operational; non-FID)**

These **do not** change outcomes. They gate diagnostics or influence **report layout only**. JSON serialization and algorithm arrays still follow Doc 1A ordering rules.

| ID | Name | Type / Domain | Default | FID? | Used by | Notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-032** | `unit_sort_order` | enum: `unit_id` | `label_priority` | `turnout` | `unit_id` | **No** | Doc 7 (rendering) | Affects **report section order only**. JSON arrays remain ordered by canonical rules (Doc 1A §5). |
| **VM-VAR-033** | `ties_section_visibility` | enum: `auto` | `always` | `never` | `auto` | **No** | Doc 7 (report templates) | `auto` shows the Ties section only if any event exists in `RunRecord.ties[]`. |
| **VM-VAR-034** | `frontier_map_enabled` | boolean | `true` | **No** | Doc 5B (Frontier stage), Doc 7 (appendix), FrontierMap | When `false`, skip emitting `frontier_map.json` and hide the appendix. |
| **VM-VAR-035** | `sensitivity_analysis_enabled` | boolean | `false` | **No** | Doc 5C (`CompareScenarios`), Doc 7 (appendix) | Runs diagnostic comparisons (does **not** alter `Result`). |

---

### **C) Labeling & language (presentation; non-FID)**

These variables influence **labels** and language in outputs. They never alter counts/allocations.

| ID | Name | Type / Domain | Default | FID? | Used by | Notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-060** | `majority_label_threshold` | integer % in **0..100** | `55` | **No** | Doc 4C (§Labels), Doc 7 | National (or configured scope) margin ≥ threshold ⇒ “Decisive” if no blocking flags (policy-dependent). |
| **VM-VAR-061** | `decisiveness_label_policy` | enum: `fixed` | `dynamic_margin` | `dynamic_margin` | **No** | Doc 4C (§Labels), Doc 7 | `fixed`: label by margin only. `dynamic_margin`: also consider mediation/protected flags when labeling. |
| **VM-VAR-062** | `unit_display_language` | `auto` | IETF tag (e.g., `en`, `fr`) | `auto` | **No** | Doc 7 (bilingual handling) | Controls display language for unit names in the rendered report(s). |

---

## **4\) Cross-doc integration map**

| Variable(s) | Consumed by | Contract highlights |
| ----- | ----- | ----- |
| 050, 052 | Doc 4C (Ties), Doc 5C (`ResolveTies`), Doc 6C | Policy drives branch; if `random`, RNG seeded by 052; events captured in `RunRecord.ties[]`. |
| 034 | Doc 5B, Doc 7, FrontierMap | If `false`, skip Frontier stage emission and hide appendix; never affects allocations. |
| 035 | Doc 5C, Doc 7 | If `true`, run scenario comparisons; results live in report appendix only. |
| 032 | Doc 7 | Affects ordering of **sections in the report**; JSON ordering stays canonical (Doc 1A §5). |
| 033 | Doc 7 | Show/hide “Ties” section (`auto` when any tie occurred). |
| 060–062 | Doc 4C (labels), Doc 7 | Compute and render labels & language; do **not** enter FID. |

---

## **5\) Stability & change policy**

* IDs **032–035** and **060–062** are **stable**; they are **never** part of FID.

* **050/052** are stable; **050** participates in FID, **052** does not.

* Any proposal to move a variable between FID/non-FID classes requires updating **Annex A** and a release decision per Doc 3B.

---

## **6\) Conformance checks**

* **C-2B-REF-01**: Engine/renderer **must** ignore these toggles for canonical JSON ordering (Doc 1A §5).

* **C-2B-TIE-02**: If `tie_policy = random`, the engine uses `tie_seed` and records seed and events in `RunRecord`.

* **C-2B-LBL-03**: Labels rendered per 060/061; allocations and tallies unaffected.

* **C-2B-EMIT-04**: `frontier_map_enabled=false` results in no `frontier_map.json`; report omits the appendix.

* **C-2B-SCN-05**: `sensitivity_analysis_enabled=true` does not change `Result` or any hashable artifact other than the added appendix.

---

## **7\) ParameterSet example (excerpt)**

{  
  "schema\_version": "1.x",  
  "vars": {  
    "VM-VAR-050": "status\_quo",  
    "VM-VAR-052": 0,  
    "VM-VAR-032": "unit\_id",  
    "VM-VAR-033": "auto",  
    "VM-VAR-034": true,  
    "VM-VAR-035": false,  
    "VM-VAR-060": 55,  
    "VM-VAR-061": "dynamic\_margin",  
    "VM-VAR-062": "auto"  
  }  
}

---

## **8\) Notes**

* There is **no variable** for the deterministic tie order key: it is always `Option.order_index` (Doc 1A §5).

* Report **decimal precision** is fixed by Doc 7; there is **no** VM-VAR controlling it.

* This document supersedes any earlier text that placed tie variables at 032–033 or labeled 050–052 as legacy.

# **Doc 2C — Advanced Controls (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **expert/outcome-affecting** controls that are not part of everyday operation but must be fixed for reproducibility. These variables are **included in the FID** (Doc 1A) when present because they can change outcomes. Presentation/report toggles remain in Doc 2B and are excluded from FID.

This part completes **Doc 2 (A/B/C)** so the engine, pipeline, and tests have a single canonical map of variables.

---

## **2\) FID policy (for 2C variables)**

* All variables in this section are **outcome-affecting ⇒ FID \= YES**.

* Defaults are declared in **Annex A — VM-VAR Registry** and surfaced in `ParameterSet`.

* If a 2C variable is **unset**, the engine uses its Annex A default (still part of FID via the normative manifest).

---

## **3\) Variables — specifications**

### **D) Exceptions & scope refinements**

Controls that narrowly refine eligibility, symmetry, or validity guardrails.

| ID | Name | Type / Domain (see Annex A for exact domain) | Default | FID? | Used by | Notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-021** | `run_scope` | enum/map (scope selector) | `all_units` | **Yes** | Doc 4A preconditions; Doc 5A | Defines inclusion scope (e.g., all units vs filtered set). Included here for traceability although grouped with “global” in 2A membership. |
| **VM-VAR-029** | `symmetry_exceptions` | array of selectors (unit/option patterns) | `[]` | **Yes** | Doc 4B gates; Doc 6B | Narrow, explicit exceptions to otherwise symmetric rules. Engine must match deterministically and only where permitted. |
| **VM-VAR-030** | `eligibility_override_list` | array of \`{unit\_id: string, mode: include | exclude}\` | `[]` | **Yes** | Doc 4B gates; Doc 5A |
| **VM-VAR-031** | `ballot_integrity_floor` | integer % in **0..100** | `0` | **Yes** | Doc 4B invalidation; Doc 6B | If a unit’s integrity KPI \< floor ⇒ unit invalid/branch per algorithm family. |

Rationale: these are rarely used, but when set they can alter eligibility/validity and therefore outcomes.

---

### **E) Frontier tuning (advanced)**

Fine-grained controls for band/window behavior beyond the core frontier settings (**040–042** in 2A).

| ID | Name | Type / Domain | Default | FID? | Used by | Notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-047** | `frontier_band_window` | number in **\[0,1\]** or enum per Annex A | `0.00` | **Yes** | Doc 4C Frontier; Doc 5B | Expands/contracts the effective band around the core cut(s) in 041\. |
| **VM-VAR-048** | `frontier_backoff_policy` | enum: `none` | `soften` | `harden` | `none` | **Yes** | Doc 4C Frontier; Doc 6C | How the engine resolves borderline cases at the edge of bands. |
| **VM-VAR-049** | `frontier_strictness` | enum: `strict` | `lenient` | `strict` | **Yes** | Doc 4C Frontier | Coarse toggle that multiplies effects of 047/048 in a defined way. |

These do **not** replace **040–042**; they refine them. Annex A formalizes the combination rules (e.g., precedence, clamping).

---

### **F) Algorithm minor variant anchor**

A controlled switch for sanctioned micro-variants (use sparingly).

| ID | Name | Type / Domain | Default | FID? | Used by | Notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-073** | `algorithm_variant` | enum (e.g., `v1`) | `v1` | **Yes** | Doc 4A step order; Doc 3B change policy | Locks a documented micro-variant (e.g., rounding tie-break preference within identical formulas). Not a presentation switch. |

---

## **4\) Cross-doc integration map**

| Variable(s) | Consumed by | Contract highlights |
| ----- | ----- | ----- |
| 021, 029–031 | Doc 4A/4B; Doc 5A; Doc 6A/6B | Evaluated before allocation; alter eligibility/validity branches deterministically. |
| 047–049 | Doc 4C; Doc 5B; Doc 6C | Modify frontier behavior around the core band selection (040–042). Effects must be fully documented in RunRecord summary if they change gating outcomes. |
| 073 | Doc 4A; Doc 3B | Selects a documented micro-variant; any change requires new FID (and Engine Version per Doc 3B). |

---

## **5\) Conformance checks**

* **C-2C-SCOPE**: If `run_scope` ≠ `all_units`, the filtered set is recorded in `RunRecord.summary` and applied consistently across inputs.

* **C-2C-EXC**: `symmetry_exceptions` are matched deterministically (no regex entropy); unmatched patterns are rejected.

* **C-2C-ELIG**: `eligibility_override_list` is applied before threshold gates; conflicts with core rules are resolved per Annex A precedence.

* **C-2C-INT**: If `ballot_integrity_floor` causes invalidation, the reason is logged (Doc 7 integrity note).

* **C-2C-FRONTIER**: 047–049 tuning cannot invert the meaning of 040–042; only refine within documented bounds.

* **C-2C-VARIANT**: `algorithm_variant` MUST be printed in the report footer beside Formula ID and Engine Version.

---

## **6\) ParameterSet example (excerpt)**

{  
  "schema\_version": "1.x",  
  "vars": {  
    "VM-VAR-021": "all\_units",  
    "VM-VAR-029": \[\],  
    "VM-VAR-030": \[\],  
    "VM-VAR-031": 0,  
    "VM-VAR-047": 0.00,  
    "VM-VAR-048": "none",  
    "VM-VAR-049": "strict",  
    "VM-VAR-073": "v1",

    "VM-VAR-040": "banded",           // from 2A  
    "VM-VAR-041": 0.10,  
    "VM-VAR-042": "apply\_on\_entry",

    "VM-VAR-050": "status\_quo",       // from 2B (outcome-affecting)  
    "VM-VAR-052": 0,

    "VM-VAR-060": 55,                 // presentation (2B)  
    "VM-VAR-061": "dynamic\_margin",  
    "VM-VAR-062": "auto"  
  }  
}

---

## **7\) Notes for implementers**

* Keep 2C switches **well-documented** in Annex A (domains, defaults, precedence).

* If your deployment doesn’t need 2C, leave defaults in place; they still contribute to FID via the normative manifest.

* Do **not** add new hidden “advanced” toggles in code paths; propose new VM-VARs (and update Annex A) when behavior changes outcomes.

*End Doc 2C.*

