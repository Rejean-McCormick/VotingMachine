
Pre-Coding Essentials (Component: crates/vm_report/src/render_html.rs, Version/FormulaID: VM-ENGINE v0) — 65/89

1) Goal & Success
Goal: Render a `ReportModel` into **deterministic, offline HTML** that matches Doc 7 section order & wording, with **one-decimal presentation** and the mandatory approval-denominator sentence when applicable.
Success: Same model → byte-identical HTML across OS/arch; zero network requests; section and keyboard order fixed; bilingual (if requested) produces mirrored full docs; assets bundled/embedded.

2) Scope
In scope: Template/token substitution from `ReportModel`, fixed section order, conditional blocks (approval sentence, frontier, sensitivity), bilingual output, accessibility (landmarks/headings/tab order), offline asset embedding.
Out of scope: Building `ReportModel` (caller), JSON canonicalization/hashing (io crate), map tiles or dynamic scripts.

3) Inputs → Outputs
Input: `&ReportModel`, `HtmlRenderOpts` (embed/bilingual).
Output: `String` HTML (UTF-8; no external refs).

4) Entities/Tables (minimal)
Pure view layer; no DB access. Uses static, bundled templates/partials and CSS.

5) Variables (display-only)
No computation here; render exactly what `ReportModel` provides (preformatted one-decimal %/pp, integers).

6) Functions (signatures only)
```rust
// Public API
pub fn render_html(model: &ReportModel, opts: HtmlRenderOpts) -> String;

#[derive(Clone, Default)]
pub struct HtmlRenderOpts {
    pub bilingual: Option<BilingualPair>, // (primary_lang, secondary_lang)
    pub embed_assets: bool,               // inline CSS/JS/fonts from bundle
}

// Internal writer (simple buffer)
struct Html { buf: String }
impl Html {
    fn new() -> Self; fn push<S: AsRef<str>>(&mut self, s: S); fn finish(self) -> String;
}

// Section writers (Doc 7 exact order)
fn write_head(w: &mut Html, opts: &HtmlRenderOpts, lang: &str);
fn write_section_cover_snapshot(w: &mut Html, m: &ReportModel);
fn write_section_eligibility(w: &mut Html, m: &ReportModel);
fn write_section_ballot(w: &mut Html, m: &ReportModel);        // inserts approval sentence if needed
fn write_section_panel(w: &mut Html, m: &ReportModel);         // quorum/majority/double-majority/symmetry
fn write_section_outcome(w: &mut Html, m: &ReportModel);       // Decisive/Marginal/Invalid + reason
fn write_section_frontier(w: &mut Html, m: &ReportModel);      // only if frontier exists
fn write_section_sensitivity(w: &mut Html, m: &ReportModel);   // table or N/A
fn write_section_integrity_footer(w: &mut Html, m: &ReportModel);

// Utilities
fn embed_assets_css() -> &'static str;           // bundled CSS (inline)
fn embed_assets_fonts_base64() -> &'static str;  // optional @font-face
fn ensure_keyboard_order(html: &mut String);     // main → sections; anchors in tab order
fn aria_landmarks() -> (&'static str, &'static str); // role mappings
````

7. Algorithm Outline (implementation plan)

* Initialize writer; set `<html lang=…>`; include `<meta charset="utf-8">`, viewport, and **inline CSS** if `embed_assets`.
* Emit sections in **exact Doc 7 order**:

  1. Cover & Snapshot
  2. Eligibility & Rolls
  3. Ballot (append mandatory sentence if `model.ballot.approval_denominator_sentence == true`)
  4. Legitimacy Panel
  5. Outcome/Label
  6. Frontier (emit only if `model.frontier.is_some()`)
  7. Sensitivity (render 2×3 table if present else “N/A (not executed)”)
  8. Integrity & Reproducibility
  9. Fixed footer line with IDs (from RunRecord/Result; tie seed line only when policy = random)
* **Bilingual mode**: render two full documents back-to-back (or tabbed sections) with mirrored content; never mix languages in one paragraph.
* **Accessibility**: wrap major blocks in `<section>` with `role="region"` and `aria-labelledby="…"`; use h1→h2→h3 hierarchy; ensure link/anchor tab order.
* **Determinism**: Templates are static; no dates/time; no hashing here. Do not reformat numbers—use model strings verbatim (one-decimal already applied).

8. State Flow (very short)
   `build_model` → `render_html` → caller writes HTML to disk (offline). No network calls.

9. Determinism & Numeric Rules
   Deterministic template insertion; one-decimal strings passed through; integer seats only. No client JS affecting layout beyond optional inline print CSS.

10. Edge Cases & Failure Policy

* **Validation/gates failed**: show panel with ❌, Outcome “Invalid …”, **omit Frontier** section.
* **Frontier mediation/protected flags**: Outcome includes ⚠ callout; label should already be Marginal upstream—do not override.
* **Missing assets**: if `embed_assets=false`, still avoid external URLs (use minimal inline fallback CSS); fail CI if template not bundled.
* **Unknown policy strings**: print verbatim (no crash).

11. Test Checklist (must pass)

* Section order strictly matches Doc 7; keyboard order starts at title → snapshot → sections.
* Approval ballots include mandatory denominator sentence in §3.
* One-decimal percent/pp strings appear verbatim (no re-rounding).
* Frontier section rendered only when model has Frontier; diagnostics match counts/flags.
* Offline: generated HTML contains **no** external `<link>`/`<script>`/`<img src="http(s)://…">`.
* Bilingual option renders two complete, mirrored documents with correct `lang` attributes.

```
```
