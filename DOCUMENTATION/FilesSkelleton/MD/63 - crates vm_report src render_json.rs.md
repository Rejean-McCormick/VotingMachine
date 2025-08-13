````markdown
Pre-Coding Essentials (Component: crates/vm_report/src/render_json.rs, Version/FormulaID: VM-ENGINE v0) — 64/89

1) Goal & Success
Goal: Serialize a `ReportModel` into **deterministic JSON** that mirrors Doc 7’s fixed section order and field names, using the model’s already-formatted values (one-decimal %), with **no recomputation**.
Success: Same `ReportModel` → byte-identical JSON string across OS/arch; sections appear in the exact Doc 7 order; the approval-denominator sentence is emitted only for approval ballots; no extra or missing keys.

2) Scope
In scope: Deterministic construction of a JSON tree, stable key order within each object, omission of `None`/empty optionals per spec, and footer identifiers from artifacts.
Out of scope: Any I/O or canonical hashing (lives in `vm_io`), HTML rendering (other module), math/formatting (already done in the model).

3) Inputs → Outputs
Input: `&ReportModel` (built from `Result`, optional `FrontierMap`, and `RunRecord`).
Output: `String` (UTF-8 JSON) with sections:
`cover`, `eligibility`, `ballot`, `legitimacy_panel`, `outcome`, optional `frontier`, optional `sensitivity`, `integrity`, `footer`.

4) Entities/Tables (minimal)
Pure data serializer. Uses `serde_json::Value` and **insertion-ordered** `serde_json::Map` to keep stable order; internally builds maps from sorted sources (e.g., `BTreeMap`) then inserts in required order.

5) Variables (render-only)
None computed. Displayed values (percent strings, pp strings, policy names, thresholds) come **verbatim** from `ReportModel`.

6) Functions (signatures only)
```rust
use serde_json::{Map, Value};

// Public API
pub fn render_json(model: &ReportModel) -> String;

// Internal builders (pure; no I/O)
fn to_ordered_json(model: &ReportModel) -> Value;

// One builder per section (keeps stable field order)
fn cover_json(m: &ReportModel) -> Value;
fn eligibility_json(m: &ReportModel) -> Value;
fn ballot_json(m: &ReportModel) -> Value;                 // adds approval sentence flag
fn panel_json(m: &ReportModel) -> Value;                  // quorum, majority, double-majority, symmetry
fn outcome_json(m: &ReportModel) -> Value;                // label + reason + national margin (pp)
fn frontier_json(m: &ReportModel) -> Option<Value>;       // only if model.frontier.is_some()
fn sensitivity_json(m: &ReportModel) -> Option<Value>;    // table or “N/A”
fn integrity_json(m: &ReportModel) -> Value;              // engine/FID/seed/UTCs
fn footer_json(m: &ReportModel) -> Value;                 // IDs: RES/RUN/FR?/REG/PS/TLY
````

7. Algorithm Outline (implementation plan)

* **Top-level ordering:** Build a `serde_json::Map` and **insert sections in Doc-order**:

  1. `cover`
  2. `eligibility`
  3. `ballot`
  4. `legitimacy_panel`
  5. `outcome`
  6. `frontier` (insert only if present)
  7. `sensitivity` (insert only if present)
  8. `integrity`
  9. `footer`
* **Stable field order inside each section:** Fill objects via helper builders that push keys in a fixed sequence. When mapping collections (e.g., family members), iterate already-sorted inputs (`BTreeMap`/`Vec` in model).
* **No recomputation:** Every numeric is already a string (e.g., `"55.0%"`, `"+3 pp"`). Do **not** parse or round again.
* **Approval sentence:** In `ballot_json`, emit `approval_denominator_sentence: true` iff `model.ballot.approval_denominator_sentence` is true; renderer wording is fixed elsewhere.
* **Optional keys:** Omit keys whose values are `None` in the model (don’t emit `null` unless Doc 7 requires it; prefer omission).
* **Footer/IDs:** Copy verbatim from `RunRecord/Result/FrontierMap` via the model; never modify casing or add prefixes.

8. State Flow (very short)
   `vm_report::build_model` → **this** serializer → JSON string returned to caller (CLI/app). Any canonicalization/hashing happens upstream/downstream, not here.

9. Determinism & Numeric Rules

* Deterministic order via explicit insert sequence and sorted inputs.
* No floats; strings only for percents/pp. The serializer **never** formats numbers.
* UTF-8 only; no BOM; no trailing newline appended.

10. Edge Cases & Failure Policy

* **Invalid/gates-fail:** `panel_json` shows pass=false rows; `frontier` omitted; `outcome` contains “Invalid” and reason—copied from model.
* **No sensitivity:** Emit `"sensitivity": "N/A (not executed)"` or omit the section per Doc 7 binding (pick one policy and keep it consistent—default: include with that string).
* **Empty families/counters:** Emit empty arrays/zeros; do not invent placeholders.
* **Unknown policy strings (custom roll policy):** Render the raw value; do not error.

11. Test Checklist (must pass)

* **Order:** Keys at top level appear exactly in Doc 7 order (cover → … → footer).
* **Approval sentence:** Present only for approval ballots.
* **Frontier conditional:** Emitted only when `model.frontier.is_some()`.
* **One-decimal integrity:** All percent strings in output match model (no changes).
* **Stability:** Serializing the same `ReportModel` twice yields identical bytes.
* **Footer correctness:** All IDs/engine/FID/seed/UTCs match the model; seed present only if tie\_policy=`random`.

```
```
