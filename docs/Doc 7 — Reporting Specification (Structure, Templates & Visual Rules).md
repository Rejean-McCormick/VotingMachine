# **Doc 7A — Reporting: Structure & Visual Rules (Updated)**

## **1\) Purpose & scope**

Defines **how to render** official outputs from the canonical artifacts without changing them. The renderer **must not** re-compute allocations or alter canonical JSON; it only formats and displays data from `Result`, `RunRecord`, and (optionally) `FrontierMap`.

* **Outcome-affecting logic** lives in Docs 4A–4C (already executed by the engine).

* **Presentation toggles** come from Doc 2B (non-FID): `VM-VAR-032..035`, `060..062`.

---

## **2\) Inputs & may/must rules**

**Consumes (read-only):**

* `Result` (Doc 1A §4.4) — allocations, labels, aggregates, IDs.

* `RunRecord` (Doc 1A §4.5) — engine/version, FID, vars echo, tie log, input digests.

* `FrontierMap` (optional; Doc 1A §4.6) — frontier diagnostics when emitted.

**Renderer MUST:**

* Use only the above artifacts (no recalculation).

* Honor **section ordering/visibility** toggles (Doc 2B).

* Print required disclosures (FID, Engine Version, etc.).

* Treat missing optional artifacts (e.g., `FrontierMap`) as “not applicable” (never error).

**Renderer MUST NOT:**

* Change array orders or numeric values.

* Depend on system locale/timezone for numeric formats (use rules below).

* Leak non-canonical diagnostics into canonical artifacts.

---

## **3\) Presentation toggles (Doc 2B recap)**

| ID | Name | Effect on report |
| ----- | ----- | ----- |
| **VM-VAR-032** `unit_sort_order` | Ordering of **unit detail sections**: `unit_id` (default) | `label_priority` | `turnout`. Does **not** change JSON order. |  |
| **VM-VAR-033** `ties_section_visibility` | `auto` (default: show only if `RunRecord.ties[]` non-empty) | `always` | `never`. |  |
| **VM-VAR-034** `frontier_map_enabled` | If `true` **and** frontier executed, include **Frontier Appendix** sourced from `FrontierMap`. |  |
| **VM-VAR-035** `sensitivity_analysis_enabled` | If `true`, include **Sensitivity Appendix** (non-canonical, Doc 5C §4.1). |  |
| **VM-VAR-060** `majority_label_threshold` | Threshold used **by engine** to derive labels; renderer displays label only. |  |
| **VM-VAR-061** `decisiveness_label_policy` | `fixed`/`dynamic_margin` — affects label text computed by engine; renderer displays. |  |
| **VM-VAR-062** `unit_display_language` | `auto` or IETF tag (`en`, `fr`, …) for unit names and static strings. |  |

All above are **non-FID** (presentation only).

---

## **4\) Document structure (sections)**

1. **Cover & metadata (required)**

   * Title, run date (`created_at` from `Result`), jurisdiction.

   * **Disclosures** block (see §7): *Formula ID*, *Engine Version*, optional *Algorithm Variant (VM-VAR-073)*.

2. **Executive summary (required)**

   * Key national metrics from `Result.summary`.

   * Aggregate label context (Decisive/Marginal counts).

3. **National overview (required)**

   * Charts/tables sourced from `Result.summary` (no recompute).

   * If labels depend on `060/061`, show the policy string (e.g., “dynamic margin, threshold 55%”).

4. **Unit detail sections (required)**

   * One section per `Result.units[]`.

   * **Ordering of sections** per `VM-VAR-032` (renderer-only).

   * Each section shows: unit name/ID, label, allocation table (votes & shares), any gate notes if available in `RunRecord` summary.

5. **Ties section (conditional)**

   * Visibility per `VM-VAR-033`.

   * Table from `RunRecord.ties[]`: `{unit_id, type, policy, seed?}`.

   * If any `policy="random"`, echo `RunRecord.determinism.rng_seed`.

6. **Frontier appendix (conditional)**

   * Included iff **`VM-VAR-034=true`** and frontier executed.

   * Table from `FrontierMap.units[]`: `{unit_id, band_met, band_value, notes}`.

7. **Sensitivity appendix (conditional, non-canonical)**

   * Included iff **`VM-VAR-035=true`** and producer provided data (Doc 5C §4.1).

   * Clearly marked “diagnostic; does not affect results”.

8. **Integrity & audit (required)**

   * Display `result_id`, `run_id`, `formula_id`.

   * Input digests from `RunRecord.inputs.*_sha256`.

   * Non-normative toggles delta (if any) — see §7.3.

---

## **5\) Numeric & text formatting (visual rules)**

* **Percentages**: display with **one decimal place** (e.g., `54.5%`).  
   Rounding: **round half up** (0.05 → 0.1). Do **not** localize decimal separator.

* **Shares**: if shown as decimals, show **three** places (e.g., `0.545`).

* **Integers**: thousands separator thin space or comma; pick one consistently for the whole doc; do not localize by OS.

* **Dates**: render in UTC ISO 8601 or spelled UTC date (e.g., `2025-08-12`).

* **Language (VM-VAR-062)**:

  * `auto`: choose the report bundle language;

  * explicit tag: use provided IETF tag for unit names and static strings; if a localized name is unavailable, fall back to canonical name.

Accessibility:

* Provide text equivalents for charts.

* Use colorblind-safe palettes; never encode information by color alone.

* Minimum font size and contrast per WCAG 2.1 AA.

---

## **6\) Section content mappings (no recomputation)**

### **6.1 Unit detail**

* **Header**: `unit_id` \+ localized `name` (per 062\) \+ `label` from `Result.units[i].label`.

* **Allocations table** (engine order):

  * Columns: Option name, `votes`, `share` (from `Result.units[i].allocations[]`).

  * Do **not** sort by votes; keep registry order.

### **6.2 Gate notes (if present)**

If producer embedded per-unit gate summary in `RunRecord` (Doc 5C §2.4), render:

* Gate status: Valid / Invalid.

* Reasons: ordered tokens as recorded.

* Protected bypass indicator and matched exceptions, if any.

### **6.3 Ties**

From `RunRecord.ties[]` (Doc 5C §2.3):

* Table columns: Unit, Type, Policy, Seed (blank unless policy=`random`).

* If empty and `ties_section_visibility=auto`, omit the section.

### **6.4 Frontier appendix**

If `FrontierMap` present:

* List `{unit_id, band_met, band_value, notes}` in ascending `unit_id`.

* Do not back-fill or compute missing metrics.

### **6.5 Sensitivity appendix**

Render whatever diagnostic structure producer emitted; clearly marked as **non-canonical**.

---

## **7\) Required disclosures & footers**

### **7.1 Identity & provenance (footer on every page)**

* **Formula ID** (64-hex).

* **Engine Version** (e.g., `vX.Y.Z`).

* **Algorithm Variant** (VM-VAR-073) if not the default.

* Page number / total.

### **7.2 Determinism snippet (end matter)**

* If any random tie occurred:  
   `Tie policy: random; RNG seed: <VM-VAR-052>; events: <count>`.

* Otherwise: `Tie policy: <status_quo|deterministic_order>; no RNG used`.

### **7.3 Non-normative toggles delta**

If any **2B toggles** differ from **Annex A defaults**, add a small table:

Non-normative toggles (differences from defaults)  
VM-VAR-032  unit\_sort\_order                 label\_priority  
VM-VAR-033  ties\_section\_visibility         always  
VM-VAR-034  frontier\_map\_enabled            false  
...

This is disclosure only; it must not affect canonical artifacts.

---

## **8\) Section ordering logic (renderer-only)**

* Default: `VM-VAR-032 = unit_id`.

* `label_priority`: sort sections by label rank `Decisive` → `Marginal` → `Invalid`, then `unit_id`.

* `turnout`: if available in `Result.summary`/per-unit metrics, sort descending turnout, then `unit_id`.

* Sorting here affects **report sections only**; **never** reorders canonical JSON.

---

## **9\) Conformance checklist (7A)**

* **C-7A-CANON**: Renderer reads canonical artifacts only; never mutates them.

* **C-7A-TOGGLES**: Honors 032–035 and 060–062 exactly as defined; no effect on FID.

* **C-7A-NUM**: One-decimal percent with round-half-up; stable thousands separators; no OS locale leakage.

* **C-7A-ORDER**: Unit sections ordered per 032; allocation tables in registry order.

* **C-7A-TIES**: Ties section per 033; seed shown only for random policy.

* **C-7A-APPX**: Frontier appendix shown only if `FrontierMap` exists and 034=true; Sensitivity appendix when 035=true; both non-canonical.

* **C-7A-DISCLOSE**: Footer shows FID, Engine Version, (optional) Variant; non-normative toggles delta included when applicable.

* **C-7A-A11Y**: Accessibility rules applied (text alternatives, contrast, color-safe).

---

## **10\) Minimal wireframe (illustrative)**

\[Cover\]  
 Title  
 Date (UTC) • Formula ID • Engine vX.Y.Z • Variant (if any)

\[Executive summary\]  
  • Valid ballots: …  
  • Invalid ballots: …  
  • Decisive units: … / Marginal: … / Invalid: …

\[National overview\]  
  Figure 1: Share chart (text alternate)  
  Table 1: National metrics

\[Units — ordered by VM-VAR-032\]  
  Unit U-001 — District 1 — Label: Decisive  
    Option   Votes   Share  
    O-A1     6000    54.5%  
    O-B1     5000    45.5%

\[Ties\] (conditional by VM-VAR-033)  
  Unit     Type        Policy         Seed  
  U-003    winner\_tie  random         424242

\[Frontier appendix\] (conditional by VM-VAR-034)  
  Unit     band\_met   band\_value   Notes

\[Sensitivity appendix\] (conditional by VM-VAR-035)  
  Scenario  Summary (diagnostic)

\[Integrity & audit\]  
  result\_id: RES:…  
  run\_id: RUN:…  
  inputs: registry sha256=…, tally sha256=…, params sha256=…  
  Non-normative toggles (diffs): …

*End Doc 7A.*

# **Doc 7B — Reporting Templates, Data Binding & Export Profiles (Updated)**

## **1\) Purpose & scope**

Defines the **template system**, **data bindings**, and **export rules** for rendering official reports from canonical artifacts. Templates are **presentation-only** (non-FID). They must **never** recompute allocations or alter canonical JSON.

Upstream truths: data & IDs (Doc 1A), variables (Doc 2), algorithm (Doc 4), pipeline (Doc 5), platform/determinism (Doc 3). Visual structure rules live in **Doc 7A**; this part makes them executable.

---

## **2\) Template model (engine-agnostic)**

* Any text templating engine is acceptable (Mustache/Handlebars/Jinja/ETC) provided:

  * **No code execution** inside templates (logicless or restricted logic).

  * Only **formatting** helpers are allowed (no arithmetic that could change outcomes).

  * Rendering is locale-neutral unless driven by **VM-VAR-062**.

* Canonical artifacts are loaded **read-only** into a **RenderContext** (below).  
   The renderer **must not** write back to canonical files.

---

## **3\) RenderContext (read-only)**

Renderer builds a single context object for templates:

{  
  "result": { /\* Result (Doc 1A §4.4) \*/ },  
  "run\_record": { /\* RunRecord (Doc 1A §4.5) \*/ },  
  "frontier\_map": { /\* optional (Doc 1A §4.6) \*/ },

  "toggles": {  
    "unit\_sort\_order": "unit\_id|label\_priority|turnout",   // VM-VAR-032  
    "ties\_section\_visibility": "auto|always|never",        // VM-VAR-033  
    "frontier\_map\_enabled": true|false,                    // VM-VAR-034  
    "sensitivity\_analysis\_enabled": true|false,            // VM-VAR-035  
    "label\_threshold": 55,                                  // VM-VAR-060  
    "label\_policy": "fixed|dynamic\_margin",                 // VM-VAR-061  
    "unit\_display\_language": "auto|en|fr|..."               // VM-VAR-062  
  },

  "computed": {  
    "units\_ordered": \[ /\* result.units\[\] reordered for display only per 032 \*/ \],  
    "ties\_present": true|false,      // run\_record.ties\[\].length \> 0  
    "frontier\_present": true|false,  // toggles.frontier\_map\_enabled && frontier\_map exists  
    "non\_normative\_diffs": \[ { "id":"VM-VAR-033", "from":"auto", "to":"always" }, ... \],  
    "counts": { "decisive": 0, "marginal": 0, "invalid": 0 } // derived from result.units\[\].label  
  }  
}

Rules:

* `computed.units_ordered` affects **document section order only**; JSON remains untouched.

* `non_normative_diffs` compares Doc 2B toggle values to **Annex A defaults** for disclosure (Doc 7A §7.3).

---

## **4\) Allowed helpers (formatting-only)**

Template engines may expose only these pure helpers:

| Helper | Input | Output | Notes |
| ----- | ----- | ----- | ----- |
| `pct1(x)` | number (0..1) | string like `54.5%` | One decimal, round half up (Doc 7A §5). |
| `dec3(x)` | number | string like `0.545` | Three decimals; no locale. |
| `int(x)` | integer | string | Thousands sep consistent within doc. |
| `date_utc(ts)` | RFC3339 | `YYYY-MM-DD` | UTC only. |
| `i18n(key, lang)` | key, IETF tag | localized string | Uses 062; fallback to canonical. |

**Not allowed:** arithmetic that changes inputs, sorting other than per §5, RNG, network access.

---

## **5\) Section iterators & ordering (render-only)**

* **Unit sections iterator** uses `computed.units_ordered`:

  * `unit_id` (default): ascending `unit_id`.

  * `label_priority`: `Decisive` → `Marginal` → `Invalid`, then `unit_id`.

  * `turnout`: descending turnout (if present in `result.summary` or unit metrics), then `unit_id`. If turnout absent, fall back to `unit_id`.

* **Allocations tables**: iterate **exactly** in the order of `result.units[i].allocations[]` (registry `order_index`), no resorting.

---

## **6\) Data binding map (normative)**

Common tokens (illustrative for Mustache/Handlebars style):

### **6.1 Cover & metadata**

* `{{result.created_at}}`

* `{{run_record.engine.vendor}} / {{run_record.engine.name}} {{run_record.engine.version}}`

* `{{result.formula_id}}`, `{{result.result_id}}`, `{{run_record.run_id}}`

### **6.2 Executive summary**

* `{{result.summary.valid_ballots_total}}`, `{{result.summary.invalid_ballots_total}}`

* `{{computed.counts.decisive}}`, `{{computed.counts.marginal}}`, `{{computed.counts.invalid}}`

* If policy display: `{{toggles.label_policy}}` and `{{toggles.label_threshold}}`

### **6.3 Unit section**

Within `{{#computed.units_ordered}} … {{/computed.units_ordered}}`:

* Header: `{{unit_id}}`, localized name via `{{i18n name toggles.unit_display_language}}`, `{{label}}`

* Table row (iterate `allocations`):

  * `{{option_id}}` (or localized option name if provided out-of-band)

  * `{{int votes}}`

  * `{{pct1 share}}`

Optional gate notes if producer embedded them in `run_record` (Doc 5C §2.4):

* `{{gate_status}}`, `{{#reasons}}{{.}}{{/reasons}}`, `{{protected_bypass}}`, `{{#applied_exceptions}}{{.}}{{/applied_exceptions}}`

### **6.4 Ties section**

Shown per **033**:

* Iterate `{{#run_record.ties}}` → `{{unit_id}}`, `{{type}}`, `{{policy}}`, `{{seed}}?`

* If any `policy="random"`: echo `{{run_record.determinism.rng_seed}}`

### **6.5 Frontier appendix**

Shown iff **034=true** and `frontier_map` exists:

* Iterate `{{#frontier_map.units}}` → `{{unit_id}}`, `{{band_met}}`, `{{band_value}}`, `{{notes}}`

### **6.6 Non-normative toggles delta**

* Iterate `{{#computed.non_normative_diffs}}` → `{{id}} {{from}} → {{to}}`

---

## **7\) Template packs & file layout**

A release must ship a **Template Pack** containing:

/templates  
  cover.hbs  
  summary.hbs  
  unit.hbs  
  ties.hbs  
  frontier.hbs  
  appendix\_sensitivity.hbs  
  audit.hbs  
/locales  
  en.json  
  fr.json  
/theme  
  base.css  
  print.css

Requirements:

* **Locales**: key→string maps; no dynamic code. Missing keys fallback to English or canonical strings.

* **CSS**: deterministic; no external fonts or network. If fonts are bundled, embed WOFF/WOFF2.

---

## **8\) Export profiles (HTML/PDF)**

* **HTML**: single self-contained file (inline CSS and fonts allowed), UTF-8, no external requests.

* **PDF**: A4 or Letter; margins ≥ 12 mm; embed fonts; rasterize figures at ≥ 150 DPI.  
   Page footer on every page: `Formula ID • Engine vX.Y.Z • (Variant if any) • Page X/Y` (Doc 7A §7.1).

Numeric/text formatting exactly per Doc 7A §5. Timezone must be UTC.

---

## **9\) Sensitivity & debug appendices (non-canonical)**

* Render `appendix_sensitivity.hbs` **only** when `toggles.sensitivity_analysis_enabled=true` **and** data provided (Doc 5C §4.1).

* Debug traces (if any) must **not** be referenced by official templates.

---

## **10\) Conformance checks (renderer)**

* **R-7B-BIND**: Every template token resolves to data from `RenderContext`; no hidden computations.

* **R-7B-ORD**: Unit section ordering follows **032**; allocation rows follow engine order.

* **R-7B-VIS**: Sections/appendices appear only per **033/034/035** rules.

* **R-7B-I18N**: Language selection per **062**; fallback deterministic.

* **R-7B-FOOT**: Footer shows Formula ID, Engine Version, (optional) Algorithm Variant.

* **R-7B-SELF**: Renderer never writes back to canonical artifacts; exports are reproducible from the same inputs.

---

## **11\) Minimal example snippets**

### **11.1 Unit template (unit.hbs)**

\<h2\>{{unit\_id}} — {{i18n name toggles.unit\_display\_language}} — {{label}}\</h2\>  
\<table\>  
  \<thead\>\<tr\>\<th\>Option\</th\>\<th\>Votes\</th\>\<th\>Share\</th\>\</tr\>\</thead\>  
  \<tbody\>  
  {{\#allocations}}  
    \<tr\>  
      \<td\>{{option\_id}}\</td\>  
      \<td\>{{int votes}}\</td\>  
      \<td\>{{pct1 share}}\</td\>  
    \</tr\>  
  {{/allocations}}  
  \</tbody\>  
\</table\>  
{{\#gate\_status}}  
  \<p\>Status: {{gate\_status}}\</p\>  
  {{\#reasons}}\<code\>{{.}}\</code\> {{/reasons}}  
  {{\#protected\_bypass}}\<p\>Protected bypass applied.\</p\>{{/protected\_bypass}}  
{{/gate\_status}}

### **11.2 Footer fragment**

\<footer\>  
  Formula ID {{result.formula\_id}} • Engine {{run\_record.engine.version}}  
  {{\#run\_record.engine}}{{\#build}} • {{.}}{{/build}}{{/run\_record.engine}}  
\</footer\>

---

## **12\) Accessibility & theming**

* WCAG 2.1 AA minimum; provide text alternatives for charts; do not encode information by color alone.

* Theme may define light/dark palettes; must preserve contrast ratios.

* Font choices must support required locales for 062; include fallback stack.

---

## **13\) Change policy**

* Template content and theme are **non-FID**. Updating them does **not** change FID.

* Any addition of new canonical fields or changes to binding semantics requires updating Doc 7A/7B and **Annex A** (if variables are involved).

* Default template pack lives with each release tag (Doc 3B §7).

*End Doc 7B.*

