# **Annex A — VM-VAR Registry (Updated)**

**Part 1 of 3 — Scope, FID/Manifest Rules, Schema & Conventions**

## **1\) Status & scope**

* **Normative.** Single source of truth for VM-VAR **IDs, names, domains, defaults, FID inclusion, and usage pointers**.

* No backward compatibility notes. The scheme in this edition is: **ties at 050/052; 051 reserved; presentation 060–062 excluded from FID.**

## **2\) Normative Manifest & FID rules**

**What goes into the Formula ID (FID):**

* All **outcome-affecting rules** \+ the **Included** VM-VAR keys/values from this annex.

* Canonicalized and hashed per Doc 1A.

**Included (FID) — variable ranges/IDs**  
 `001–007, 010–017, 020–031, 040–049, 050, 021, 029, 030, 031, 045, 046, 047, 048, 049, 073`  
 (De-duplicated list; same meaning: *all outcome-affecting groups, including 050; seed 052 is excluded*.)

**Excluded (non-FID)**  
 `032–035, 052, 060–062` (presentation/report toggles and the tie seed).

**Ordering when building the manifest:**

1. Sort variables by **numeric ID** ascending (e.g., 001, 002, …, 073).

2. For each, serialize as canonical JSON key/value (`"VM-VAR-###": value`).

3. Do not include absent **Excluded** vars; may be echoed in `RunRecord` but never change FID.

**Canonicalization:** bytes \= UTF-8, LF, **sorted keys** at all object levels (Doc 1A).

## **3\) Registry entry schema (authoritative fields)**

Every variable is defined with these fields:

| Field | Meaning |
| ----- | ----- |
| `id` | `VM-VAR-###` (three digits, zero-padded) |
| `name` | canonical `snake_case` |
| `type` | \`enum |
| `domain` | precise allowed set/range/shape (deterministic) |
| `default` | canonical default value (used if unset) |
| `fid` | `Included` or `Excluded` |
| `used_by` | pointers to consuming docs/sections (e.g., `Doc4C`) |
| `notes` | brief constraints (precedence, ordering, reserved, etc.) |

**Machine-readable envelope (packaged with releases):**

{  
  "schema\_version": "1.x",  
  "vars": \[  
    {  
      "id": "VM-VAR-050",  
      "name": "tie\_policy",  
      "type": "enum",  
      "domain": \["status\_quo","deterministic\_order","random"\],  
      "default": "status\_quo",  
      "fid": "Included",  
      "used\_by": \["Doc4C","Doc5B-008","Doc6C"\],  
      "notes": "051 reserved; deterministic order uses option.order\_index"  
    }  
    // ... all other entries appear here in numeric order  
  \]  
}

## **4\) Determinism & RNG anchors (ties)**

* **VM-VAR-050** (*tie\_policy*) is **Included** in FID.

* **VM-VAR-052** (*tie\_seed*) is **Excluded** from FID; recorded in `RunRecord.determinism.rng_seed` **iff** a random tie occurred.

* **VM-VAR-051** is **reserved** (no meaning).

* RNG algorithm/profile is pinned in **Annex B**; engines must draw **exactly *k* 64-bit values for a *k*\-way tie**, sort tied items by `(draw, option_id)`.

## **5\) Naming & reserved IDs**

* IDs are **stable**; do not renumber/repurpose.

* **051** stays reserved across releases.

* Any future variable must claim an unused ID and be added here with full schema.

## **6\) Cross-doc pointers (where these rules bite)**

* **Doc 1A** — canonicalization & hashing; **Doc 1B** — schema/validation; **Doc 1C** — ER & lifecycle.

* **Doc 2A/2B/2C** — grouping of Included vs Excluded, operational/presentation split.

* **Doc 3A** — determinism & RNG behavior; **Doc 3B** — when FID/Engine Version must change.

* **Doc 4A–4C** — algorithm touchpoints (gates, frontier, ties, labels).

* **Doc 5A–5C** — pipeline, function contracts, RunRecord echo rules.

* **Doc 6A–6C** — conformance tests; **Annex B** — expected hashes, RNG profile.

## **7\) Conformance checklist (annex-level)**

* **A-REG-ID:** Every engine variable used exists here with correct ID/name.

* **A-REG-DOM:** Engine enforces the declared domains/defaults exactly.

* **A-REG-FID:** FID recomputation from the **Included** set equals artifacts’ `formula_id`.

* **A-REG-TIES:** 050/052 handled per rules; 051 ignored; RNG profile matches Annex B.

---

**Next:** *Part 2 of 3 — Full Registry Tables (Outcome-affecting: Global/Thresholds/Frontier/Protected/Ties)*.

# **Annex A — VM-VAR Registry (Updated)**

**Part 2 of 3 — Full Registry Tables (Outcome-affecting variables only; FID \= Included)**

This part lists every **Included** (outcome-affecting) variable with domains, defaults, and consumption points.  
 Excluded/presentation variables (032–035, 052, 060–062) are in **Part 3**.

---

## **A. Global & algorithm family (IDs 001–007, 073, 021\)**

| id | name | type | domain | default | used\_by | notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-001** | `algorithm_family` | enum | per release (e.g., `family_v1`) | `family_v1` | Doc 4A; 5B-007; 6A-105 | Chooses allocation/rounding semantics. |
| **VM-VAR-002** | `rounding_policy` | enum | `half_up` | `bankers` | `half_up` | Doc 4A; 6A-105 | Must be stable across builds. |
| **VM-VAR-003** | `share_precision` | integer | 0..6 | 3 | Doc 4A; 7A | Internal calc/display base; renderer still formats per 7A. |
| **VM-VAR-004** | `denom_rule` | enum | per family | `standard` | Doc 4A | Family-defined denominators. |
| **VM-VAR-005** | `aggregation_mode` | enum | per family | `sum` | Doc 4A | Aggregate strategy. |
| **VM-VAR-006** | `seat_allocation_rule` | enum | per family | `none` | Doc 4A | Use `none` if seats not modeled. |
| **VM-VAR-007** | `tie_scope_model` | enum | `winner_only` | `rank_all` | `winner_only` | Doc 4C | Where ties can trigger. |
| **VM-VAR-073** | `algorithm_variant` | enum | `v1` (others per release) | `v1` | Doc 4A; 3B | Micro-variant anchor; print in footer if ≠ default. |
| **VM-VAR-021** | `run_scope` | enum/object | `all_units` | selector map | `all_units` | Doc 4A (S0/S2); 5A | Fixes working set; record in RunRecord. |

---

## **B. Thresholds, eligibility, overrides & integrity (IDs 010–017, 020–031, 029–031)**

| id | name | type | domain | default | used\_by | notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-010** | `min_turnout_pct` | integer | 0..100 | 0 | Doc 4B; 6B-201 | Eligibility gate. |
| **VM-VAR-011** | `min_valid_share_pct` | integer | 0..100 | 0 | Doc 4B | Eligibility gate. |
| **VM-VAR-012** | `eligibility_gate_1` | enum/number | per release | value | Doc 4B | If unused this release, omit from params. |
| **VM-VAR-013** | `eligibility_gate_2` | enum/number | per release | value | Doc 4B | — |
| **VM-VAR-014** | `participation_floor_pct` | integer | 0..100 | 0 | Doc 4B | — |
| **VM-VAR-015** | `unit_quorum_pct` | integer | 0..100 | 0 | Doc 4B | — |
| **VM-VAR-016** | `option_quorum_pct` | integer | 0..100 | 0 | Doc 4B | Option-level continuation. |
| **VM-VAR-017** | `reserved_threshold` | integer | 0..100 | 0 | Doc 4B | Reserved slot; define only if used. |
| **VM-VAR-020** | `threshold_A` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-022** | `threshold_B` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-023** | `threshold_C` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-024** | `threshold_D` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-025** | `threshold_E` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-026** | `threshold_F` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-027** | `threshold_G` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-028** | `threshold_H` | number/int | per release | value | Doc 4B | Generic named cutoff. |
| **VM-VAR-029** | `symmetry_exceptions` | array\<string\> | deterministic selectors | `[]` | Doc 4B; 6B-203 | Narrow eligibility overrides; deterministic grammar; no regex entropy. |
| **VM-VAR-030** | `eligibility_override_list` | array\<object\> | \`{unit\_id, mode: include | exclude}\` | `[]` | Doc 4B; 5B-005; 6B-204 |
| **VM-VAR-031** | `ballot_integrity_floor` | integer | 0..100 | 0 | Doc 4B; 6B-205/207 | Failure ⇒ unit invalid; cannot be bypassed. |

---

## **C. Frontier & refinements (IDs 040–042, 047–049) \+ Protected/Autonomy (045–046)**

| id | name | type | domain | default | used\_by | notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-040** | `frontier_mode` | enum | `none` | `banded` | `ladder` | `none` | Doc 4C §2; 6B-210..216 | Master switch. |
| **VM-VAR-041** | `frontier_cut` | number/enum | per mode | 0.00 | Doc 4C §2; 6B-211 | Core band/cut parameter. |
| **VM-VAR-042** | `frontier_strategy` | enum | `apply_on_entry` | `apply_on_exit` | `sticky` | `apply_on_entry` | Doc 4C §2; 6B-211 | Application timing. |
| **VM-VAR-047** | `frontier_band_window` | number | 0.00..1.00 | 0.00 | Doc 4C §2; 6B-213 | Expands/contracts around 041\. |
| **VM-VAR-048** | `frontier_backoff_policy` | enum | `none` | `soften` | `harden` | `none` | Doc 4C §2; 6B-214 | Borderline behavior. |
| **VM-VAR-049** | `frontier_strictness` | enum | `strict` | `lenient` | `strict` | Doc 4C §2; 6B-215 | Coarse multiplier on 047/048. |
| **VM-VAR-045** | `protected_area_override` | enum | `deny` | `allow` | `deny` | Doc 4B §3.4; 6B-206/207 | May bypass **eligibility** only; never sanity/integrity. |
| **VM-VAR-046** | `autonomy_package_map` | object | documented map | `{}` | Doc 4C; 6B-216 | Ladder/autonomy step selection; deterministic keys. |

---

## **D. Ties (Outcome-affecting policy; seed is Excluded)**

| id | name | type | domain | default | used\_by | notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-050** | `tie_policy` | enum | `status_quo` | `deterministic_order` | `random` | `status_quo` | Doc 4C §3; 5B-008; 6C | **Included** in FID. `deterministic_order` uses `option.order_index` (then `option_id`). |
| **VM-VAR-051** | — | — | — | — | — | **Reserved**; do not assign. |
| *(see Part 3\)* | `tie_seed` | integer | ≥ 0 | 0 | Doc 3A; 4C; 6C | **VM-VAR-052** is **Excluded** from FID; recorded only if random ties occurred. |

---

### **Notes on domains marked “per release”**

* Where domain is **per release**, you must enumerate allowed values (enums) or bounds (numbers) in the machine-readable registry you ship with the tag.

* Any change to the domain/default/semantics of these **Included** variables ⇒ **new FID** (Doc 3B).

---

**Next:** *Part 3 of 3 — Excluded/Presentation variables (032–035, 052, 060–062) \+ machine-readable export and examples.*

# **Annex A — VM-VAR Registry (Updated)**

**Part 3 of 3 — Excluded / Presentation Variables \+ Machine-Readable Export**

This part lists the **Excluded** (non-FID) variables and provides the machine-readable registry excerpt plus examples. Excluded variables **never** change outcomes or the FID; they only affect rendering or optional diagnostics.

---

## **E. Presentation & pipeline toggles (Excluded from FID)**

### **E.1 Report/pipeline toggles (IDs 032–035)**

| id | name | type | domain | default | used\_by | notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-032** | `unit_sort_order` | enum | `unit_id` | `label_priority` | `turnout` | `unit_id` | Doc 7A §8; 7B §5 | Reorders **report sections only**. No effect on JSON array order. |
| **VM-VAR-033** | `ties_section_visibility` | enum | `auto` | `always` | `never` | `auto` | Doc 7A §4/§6.3 | Shows/omits Ties section. `auto` shows if `RunRecord.ties[]` non-empty. |
| **VM-VAR-034** | `frontier_map_enabled` | boolean | — | `true` | Doc 5C §3; 7A §4/§6.4 | Toggles emission of **FrontierMap** file and appendix visibility; allocations/FID unchanged. |
| **VM-VAR-035** | `sensitivity_analysis_enabled` | boolean | — | `false` | Doc 5C §4; 7A §4/§6.5 | Enables **non-canonical** diagnostics appendix. Never alters canonical artifacts. |

### **E.2 Tie seed (non-FID)**

| id | name | type | domain | default | used\_by | notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-052** | `tie_seed` | integer | ≥ 0 | `0` | Doc 3A RNG; 4C §3; 5C §2.1/§2.3 | Recorded in `RunRecord.determinism.rng_seed` **iff** a random tie occurred. Does not enter FID. |

### **E.3 Labels & language (IDs 060–062)**

| id | name | type | domain | default | used\_by | notes |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| **VM-VAR-060** | `majority_label_threshold` | integer | 0..100 | `55` | Doc 4C §4; 7A §3/§5 | Threshold for label text only. No effect on allocations. |
| **VM-VAR-061** | `decisiveness_label_policy` | enum | `fixed` | `dynamic_margin` | `dynamic_margin` | Doc 4C §4; 7A | Presentation policy; reads deterministic flags produced by the algorithm. |
| **VM-VAR-062** | `unit_display_language` | string | `auto` or IETF tag | `auto` | Doc 7A §5; 7B locales | Localizes unit names/strings in renderer; deterministic fallback. |

---

## **F. Machine-readable registry (release payload excerpt)**

Ship a JSON file (e.g., `annex-a.vars.json`) containing **all variables** (Included and Excluded) in **numeric ID order**. Below is the **Excluded** slice (the Included slice appears in Part 2).

{  
  "schema\_version": "1.x",  
  "vars": \[  
    { "id":"VM-VAR-032","name":"unit\_sort\_order","type":"enum",  
      "domain":\["unit\_id","label\_priority","turnout"\],"default":"unit\_id",  
      "fid":"Excluded","used\_by":\["Doc7A","Doc7B"\],  
      "notes":"Reorders report sections only; canonical JSON unchanged." },

    { "id":"VM-VAR-033","name":"ties\_section\_visibility","type":"enum",  
      "domain":\["auto","always","never"\],"default":"auto",  
      "fid":"Excluded","used\_by":\["Doc7A"\],  
      "notes":"Show ties section based on RunRecord.ties\[\] unless overridden." },

    { "id":"VM-VAR-034","name":"frontier\_map\_enabled","type":"boolean",  
      "default":true,"fid":"Excluded","used\_by":\["Doc5C","Doc7A"\],  
      "notes":"Controls FrontierMap emission and appendix visibility only." },

    { "id":"VM-VAR-035","name":"sensitivity\_analysis\_enabled","type":"boolean",  
      "default":false,"fid":"Excluded","used\_by":\["Doc5C","Doc7A","Doc7B"\],  
      "notes":"Diagnostic appendix; never alters canonical artifacts." },

    { "id":"VM-VAR-052","name":"tie\_seed","type":"integer",  
      "domain":{"min":0},"default":0,"fid":"Excluded",  
      "used\_by":\["Doc3A","Doc4C","Doc5C"\],  
      "notes":"Echoed as RunRecord.determinism.rng\_seed iff any random tie occurred." },

    { "id":"VM-VAR-060","name":"majority\_label\_threshold","type":"integer",  
      "domain":{"min":0,"max":100},"default":55,"fid":"Excluded",  
      "used\_by":\["Doc4C","Doc7A"\],  
      "notes":"Labels only; allocations unaffected." },

    { "id":"VM-VAR-061","name":"decisiveness\_label\_policy","type":"enum",  
      "domain":\["fixed","dynamic\_margin"\],"default":"dynamic\_margin","fid":"Excluded",  
      "used\_by":\["Doc4C","Doc7A"\],  
      "notes":"Presentation policy; reads deterministic flags, does not change outputs." },

    { "id":"VM-VAR-062","name":"unit\_display\_language","type":"string",  
      "domain":\["auto","IETF"\],"default":"auto","fid":"Excluded",  
      "used\_by":\["Doc7A","Doc7B"\],  
      "notes":"Localization for renderer; deterministic fallback to canonical names." }  
  \]  
}

**Packaging rules**

* Place the combined file (Included \+ Excluded) under the release tag.

* The verifier uses **only** entries marked `fid:"Included"` when recomputing the FID.

---

## **G. Examples & rules of use**

### **G.1 ParameterSet excerpt (presentation toggles present; FID unchanged)**

{  
  "schema\_version": "1.x",  
  "vars": {  
    "VM-VAR-050": "random",     // Included (affects FID)  
    "VM-VAR-052": 424242,       // Excluded (seed)  
    "VM-VAR-040": "banded",     // Included  
    "VM-VAR-034": true,         // Excluded (appendix toggle)  
    "VM-VAR-060": 55,           // Excluded (labels)  
    "VM-VAR-061": "dynamic\_margin",  
    "VM-VAR-062": "auto"  
  }  
}

* Runs with different **052** values have **identical FID**.

* Toggling **034/035/032/033/060/061/062** never changes allocations nor FID.

### **G.2 Normative Manifest build rule (reminder)**

* Include **only** variables with `fid:"Included"` (Parts 1–2 list).

* Sort by numeric ID; canonical JSON; then hash to FID (Doc 1A).

### **G.3 RunRecord echo policy (non-FID)**

* Producer **may** echo Excluded vars in `vars_effective` for transparency.

* `rng_seed` appears **only** if a random tie occurred.

---

## **H. Conformance (Excluded set)**

* **A-EX-IMMUT:** Changing any Excluded var **must not** alter canonical artifacts (`Result`, `RunRecord`) **except** optional presence/absence of `FrontierMap.json` (034).

* **A-EX-FID:** FID recomputation **ignores** Excluded vars (including 052).

* **A-EX-RPT:** Renderer behavior aligns with Doc 7A/7B; no recomputation; section ordering changes are presentation-only.

---

## **I. Change policy (Excluded set)**

* You **may** change defaults or allowed values of Excluded vars between releases **without** a new FID, but you **must**:

  * Update Annex A and Doc 7A/7B as needed.

  * Disclose any non-default presentation toggles in the report footer “Non-normative toggles” block (Doc 7A §7.3).

*End Annex A (Part 3 of 3).*

