# **Doc 7A — Report: Structure & Fixed Content**

**Purpose.** Fix the **sections, fields, precision, and data sources** so every report is identical given the same inputs.  
 **Sources.** Only read: **Result**, optional **FrontierMap**, and **RunRecord** (Doc 1). Parameter names shown come from the **ParameterSet** snapshot that is embedded via those objects.  
 **Precision.** All percentages shown to **one decimal place**. Internals follow Doc 4 rounding; **presentation rounding happens once here**.  
 **Approval gate denominator (fixed).** When ballot\_type \= **approval**, the **national support %** for gates is the **approval rate**:  
 `approvals_for_change / valid_ballots` (not share of approvals). This sentence must appear in §3.

---

## **Section order (must appear exactly in this order)**

### **1\) Cover & Snapshot**

**What to show (single-page header \+ snapshot box):**

* Title, jurisdiction name, date.

* **Outcome label**: “Decisive / Marginal / Invalid”.

* **Snapshot box (left→right):**

  * **Ballot:** VM-VAR-001

  * **Allocation:** VM-VAR-010 (and Unit.magnitude or policy)

  * **Weighting:** VM-VAR-030

  * **Thresholds:** quorum VM-VAR-020; national VM-VAR-022; regional VM-VAR-023 (if double-majority on)

  * **Double-majority:** VM-VAR-024 on/off · **Symmetry:** VM-VAR-025 on/off

  * **Frontier mode:** VM-VAR-040 (if any)

**Data mapping:**

* Outcome label → `Result.label`.

* All VM-VAR values → `ParameterSet` snapshot (via `Result`/`RunRecord` linkage).

* Jurisdiction/date → `DivisionRegistry.name/version` and `RunRecord.timestamp`.

---

### **2\) Eligibility & Rolls (Who could vote)**

**What to show (2–4 sentences \+ a small table):**

* **Roll inclusion policy** (verbatim): VM-VAR-028.

* Registry **roll provenance**: DivisionRegistry.provenance `{source, published_date}`.

* Totals: `Σ eligible_roll` and `Σ ballots_cast` at country level.

* Note the per-unit quorum if VM-VAR-021 \> 0 (and whether scope is `frontier_only` or `frontier_and_family` if set).

**Data mapping:**

* Policy → VM-VAR-028.

* Provenance → `DivisionRegistry.provenance`.

* Totals → sum of `Result.UnitBlock.turnout` fields.

* Per-unit quorum note → VM-VAR-021 and (optional) `VM-VAR-021_scope`.

---

### **3\) How Votes Were Counted (Ballot)**

**What to show (method paragraph):**

* State ballot type plain-English rules (plurality / approval / score / ranked IRV / ranked Condorcet).

* If **score**: print scale `[VM-VAR-002..003]` and note normalization VM-VAR-004.

* If **ranked IRV**: note exhaustion policy \= **reduce continuing denominator**.

* If **ranked Condorcet**: note completion rule VM-VAR-005.

* **Mandatory sentence for approval ballots:**  
   “For legitimacy gates, the support % is the **approval rate** \= approvals for the Change option divided by **valid ballots**.”

**Data mapping:**

* Methods & parameters → `ParameterSet`.

* Turnout denominators (valid vs include blanks) → VM-VAR-007 (state if on).

---

### **4\) How Seats/Power Were Allocated (Inside Units)**

**What to show:**

* Allocation method VM-VAR-010.

* If proportional: PR entry threshold VM-VAR-012.

* If **MMP**: top-up share VM-VAR-013, target basis VM-VAR-015, correction level VM-VAR-016, total seats model VM-VAR-017, overhang policy VM-VAR-014.

**Data mapping:**

* All from `ParameterSet`; unit magnitudes from `DivisionRegistry` (summarize as “m=… where applicable”).

---

### **5\) How Results Were Aggregated (Hierarchy & Weighting)**

**What to show:**

* Weighting method VM-VAR-030; if `population_baseline`, cite “registry baseline year(s)”.

* Aggregate level is **country** (VM-VAR-031, v1 fixed).

**Data mapping:**

* Method → `ParameterSet`.

* Baseline year → `Unit.population_baseline_year` (state the range or the common year).

* Aggregates used → `Result.Aggregates`.

---

### **6\) Legitimacy Panel (Decision Gates)**

**Layout:** four lines with badges (✅/❌).

* **Quorum:** `Turnout [X.X%] vs quorum [Y%] — Pass/Fail`.

  * If per-unit quorum set, show “Per-unit quorum applied; \[N\] units below threshold.”

* **Majority/Supermajority:** `Support [X.X%] vs [Y%] — Pass/Fail`.

* **Double-majority:** if on: `National [X.X%] & affected regions [min: Z.Z%] vs [Y%] — Pass/Fail` (and state how family was defined).

* **Symmetry:** `Applied` or `Not respected: <summary from VM-VAR-029>`.

**Data mapping:**

* Values & pass/fail → `Result.gates` section (LegitimacyReport copy).

* Affected-family method → VM-VAR-026/027; enforce “by\_list/by\_tag” mention if `frontier_mode=none`.

---

### **7\) Outcome**

**What to show:**

* **Council/Power-sharing:** a table of seats/power by Option (integers for seats; share if power).

* **Executive (if enabled):** “Executive winner: ; margin \[M.pp\] of continuing ballots (IRV) or per Condorcet rule.”

* **Label line:** “Result label: Decisive / Marginal / Invalid — .”

**Data mapping:**

* Seats/power → `Result.UnitBlocks` (rolled) or `Result.Aggregates` at country level.

* Executive winner/margin → IRV RoundLog or Condorcet outcome tracked in `Result` audit.

* Label & reason → `Result.label`, `Result.label_reason`.

---

### **8\) Frontier / Autonomy (if produced)**

**What to show (map \+ paragraph):**

* Map legend: actions (no change / autonomy(AP:Name) / phased change / immediate change).

* One paragraph: counts of mediation zones, enclaves, protected overrides; contiguity basis (VM-VAR-047) and island rule (VM-VAR-048).

* If per-unit quorum blocked status changes, state the count.

**Data mapping:**

* Map/status → **FrontierMap** per-unit `status`, `band_met`.

* Diagnostics → `FrontierMap` flags and summary counters; also mirror `Result.UnitBlock.mediation_flagged` / `protected_override_used`.

---

### **9\) Sensitivity (Flip Points)**

**What to show:**

* 2×3 mini-table: outcome under **±1 pp** and **±5 pp** threshold tweaks (national, regional, cutoff).

* If unavailable (CompareScenarios not run), print **“N/A (not executed)”**.

**Data mapping:**

* From **CompareScenarios** output (VM-FUN-013) linked to the baseline **Result** ID(s).

---

### **10\) Integrity & Reproducibility**

**What to show (bulleted identifiers):**

* **Formula ID**, **Engine Version**, **Division Registry ID**, **Parameter Set ID**, **BallotTally ID/label**, **RNG seed** (if used), **Run timestamp (UTC)**, **Results ID**, optional **FrontierMap ID**.

* One sentence: “Anyone can reproduce this result locally using these inputs.”

**Data mapping:**

* All from **RunRecord**; `Results ID` and optional `FrontierMap ID` from pointers inside **RunRecord**.

---

## **Fixed footer (every page)**

`Formula ID · Engine Version · Division Registry · Parameter Set · BallotTally Label · Run Timestamp · Results ID`

**Data mapping:** footer values are taken **verbatim** from **RunRecord** and `Result.id`; the BallotTally label comes from `BallotTally.label` referenced by **RunRecord**.

---

## **Rendering rules (non-negotiable)**

* **Precision:** show percentages with **one decimal**; margins in **pp** with one decimal; seats as integers.

* **No external assets:** all fonts/styles bundled (Doc 3).

* **Internationalization:** if bilingual, render full mirrored PDFs; do not mix languages within paragraphs.

---

## **Checklist (data-backed, no extras)**

* Section order as above.

* **roll\_inclusion\_policy** and provenance printed in §2.

* Approval gate denominator sentence in §3.

* Panel values come from `Result.gates`; affected-family method stated.

* Outcome, Frontier, Sensitivity mapped only to produced artifacts (Result/FrontierMap/CompareScenarios).

* Footer identifiers present and sourced from **RunRecord/Result**.

**Status:** Report structure & fixed content are final and fully backed by pipeline outputs.

# **Doc 7B — Report: Templates, Visuals & Fallbacks**

**Purpose.** Lock the exact wording blocks, icon/color rules, map patterns, accessibility, bilingual handling, and error fallbacks.  
 **Style.** Neutral, factual. One-decimal percentages. No analytics beyond pipeline outputs (Docs 5/7A).  
 **Inputs.** Everything shown must come from `Result`, optional `FrontierMap`, `RunRecord`, and (if executed) `CompareScenarios`.

---

## **1\) Verbatim wording blocks (fill the \[brackets\] exactly)**

### **1.1 Quorum**

**Pass**

Turnout was **\[X.X%\]**, meeting the **\[Y%\]** quorum — **Pass**.

**Fail**

Turnout was **\[X.X%\]**, below the **\[Y%\]** quorum — **Fail**. The outcome is **Invalid**.

### **1.2 Majority / Supermajority**

**Pass**

Support for **\[Option/Change\]** was **\[X.X%\]**, meeting the **\[Y%\]** threshold — **Pass**.

**Fail**

Support for **\[Option/Change\]** was **\[X.X%\]**, below the **\[Y%\]** threshold — **Fail**. The outcome is **Invalid**.

### **1.3 Double-majority**

**Pass**

National support **\[X.X%\]** and affected-regions support **\[min: Z.Z%\]** both met **\[Y%\]** — **Pass**.

**Fail**

Although national support was **\[X.X%\]**, the affected-regions requirement **\[Y%\]** was not met (**\[lowest region: Z.Z%\]**) — **Fail**. The outcome is **Invalid**.

*(If family is by\_list/by\_tag, append once: “Affected-regions were defined **\[by list/by tag\]**.”)*

### **1.4 Symmetry**

**Respected**

The same thresholds apply to all directions of change — **Respected**.

**Not respected (with exceptions)**

The same thresholds do not apply everywhere — **Not respected**: **\[summary of symmetry\_exceptions\]**.

### **1.5 Ties**

**Status quo policy**

A tie occurred **(\[context\])**. By policy, **Status Quo prevails**.

**Deterministic order**

A tie occurred **(\[context\])**. It was resolved by the predeclared ordering: **\[A over B\]**.

**Random (seeded)**

A tie occurred **(\[context\])**. It was resolved by **random draw** with seed **\[\#\#\#\#\]**.

### **1.6 Frontier summary**

Units meeting **\[rule, e.g., ≥T% or band name\]** changed status. **\[N\]** mediation zones and **\[K\]** enclaves were flagged under the contiguity policy **\[land/bridge/water as allowed\]**. **\[P\]** protected units **\[changed with override / were unaffected\]**.

### **1.7 Result label**

Result label: **\[Decisive / Marginal / Invalid\]** (**\[reason\]**).

---

## **2\) Visual rules (fixed)**

### **2.1 Colors (color-blind safe mapping)**

* **Status Quo:** grey

* **A:** blue **B:** orange **C:** green **D:** purple

* **Autonomy bands:** neutral hues with **hatching** (not saturated reds).

* Do not invent new colors; if more than four Options, reuse sequence cyclically with lighter tint.

### **2.2 Icons**

* ✅ **Pass**, ❌ **Fail**, ⚠ **Marginal note** (Legitimacy Panel and callouts only).

### **2.3 Charts**

* Bars only (no 3D, no gradients).

* One chart per figure.

* Percent labels shown to **one decimal**.

### **2.4 Maps**

* Solid fills for statuses.

* **Mediation zones:** diagonal stripes overlay.

* **Enclaves:** dotted overlay.

* Black borders for Units; thin white stroke between Units for legibility.

* Legend must list: *no change*, *phased change*, *immediate change*, *autonomy(AP:Name)*.

### **2.5 Tables**

* Headers include **units**: “%”, “pp”, “seats”.

* Right-align numbers; one decimal for percentages and margins.

---

## **3\) Accessibility & bilingual handling**

* **Fonts & contrast:** body ≥ 10.5pt; high-contrast text/icons; do not encode meaning in color alone.

* **Alt text:** every chart/map has an alt sentence (“Seats by option at country level…”, “Map: mediation zones hatched…”).

* **Keyboard order:** logical reading order (title → snapshot → sections).

* **Bilingual:** produce **mirrored full documents** per language; do not mix languages in a paragraph. Keep numbers/IDs identical across languages.

* **Numerals:** show dot-decimal internally; localized decimal in PDFs/HTML is allowed **only for display** and must not change stored values.

---

## **4\) Sensitivity section rule**

* Render the 2×3 **±1pp / ±5pp** table **only if** `CompareScenarios` (VM-FUN-013) artifacts are present.

* Otherwise print a single line:

   Sensitivity: **N/A (not executed)**.

---

## **5\) Error and fallback behaviors**

### **5.1 Validation failed (before counting)**

* Show Sections: **Cover & Snapshot**, **Eligibility**, **Ballot** (method statement), then a box:

   **Why this run is invalid:** \[bullet list of validation issues\].

* **Legitimacy Panel:** “N/A” for values; show ❌ Invalid.

* **Outcome:** “Invalid (validation failed).”

* **Frontier:** **omit**.

* **Integrity:** still show identifiers from `RunRecord`.

### **5.2 Gates failed**

* Render full report up to **Legitimacy Panel** with ❌ where applicable.

* **Outcome:** “Invalid (gate failed: \[quorum/majority/double-majority/symmetry\]).”

* **Frontier:** **omit**.

* Sensitivity may still be shown if scenarios were executed.

### **5.3 Mediation / protected impacts**

* If **any** mediation/enclave/protected-override flags exist, add a ⚠ callout under **Outcome**:

   Frontier diagnostics: **\[N\]** mediation zones, **\[K\]** enclaves, **\[P\]** protected overrides.

* This condition alone changes the label to **Marginal** per Doc 4C.

---

## **6\) Data binding (where the words/numbers come from)**

* **Quorum/majority/double-majority/symmetry lines:** `Result.gates` (values \+ Pass/Fail).

* **Approval denominator sentence:** included verbatim in §3; denominator \= **valid ballots** (Doc 4A).

* **Frontier counts & flags:** `FrontierMap` \+ `Result.UnitBlock.mediation_flagged` / `protected_override_used`.

* **Label & reason:** `Result.label`, `Result.label_reason`.

* **Sensitivity table:** `CompareScenarios` bundle linked to the baseline `Result.id`.

* **Footer identifiers:** `RunRecord` \+ `Result.id` \+ `BallotTally.label`.

---

## **7\) Do / Don’t**

* **Do** use exactly these templates and icons; keep one decimal; state methods and denominators plainly.

* **Don’t** add commentary, forecasts, polling, demographic analysis, or unproduced metrics; don’t round twice; don’t change icon meanings.

**Status:** Templates, visuals, accessibility, bilingual handling, sensitivity rule, and error fallbacks are locked.

