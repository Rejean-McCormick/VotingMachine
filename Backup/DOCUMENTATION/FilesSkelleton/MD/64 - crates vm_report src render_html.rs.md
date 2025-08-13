<!-- Converted from: 64 - crates vm_report src render_html.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.246030Z -->

```
Pre-Coding Essentials (Component: crates/vm_report/src/render_html.rs, Version/FormulaID: VM-ENGINE v0) — 65/89
1) Goal & Success
Goal: Render the ReportModel to HTML using fixed templates/wording, exact section order, and one-decimal presentation; bundle all assets for offline use.
Success: HTML matches Doc 7 wording/sections; includes the mandatory approval-denominator sentence for approval ballots; no external fetches (fonts/styles/tiles).
2) Scope
In scope: Template selection, token substitution from ReportModel, section ordering, one-decimal formatting (no re-rounding), bilingual mirrored output option, and accessibility/keyboard order constraints.
Out of scope: Building the model, computing values, or reading artifacts directly (those happen upstream).
3) Inputs → Outputs (with schemas/IDs)
Input: ReportModel (already bound to Result, RunRecord, optional FrontierMap), plus a handle to bundled assets (templates/CSS/inline map styles).
Output: Deterministic HTML string/document following Doc 7 section order and fixed wording blocks & footers.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
Display-only; no VM-VAR computation here. Ensure approval-denominator sentence appears in §3 for approval ballots.
6) Functions (signatures only)
rust
CopyEdit
pub fn render_html(model: &ReportModel, opts: HtmlRenderOpts) -> String;

pub struct HtmlRenderOpts {
pub bilingual: Option<BilingualPair>, // if Some, produce mirrored full docs
pub embed_assets: bool,               // inline <style>/<script> from bundle
}

fn write_section_cover_snapshot(w: &mut Html, m: &ReportModel);
fn write_section_eligibility(w: &mut Html, m: &ReportModel);
fn write_section_ballot(w: &mut Html, m: &ReportModel); // adds approval sentence if needed
fn write_section_panel(w: &mut Html, m: &ReportModel);
fn write_section_outcome(w: &mut Html, m: &ReportModel);
fn write_section_frontier(w: &mut Html, m: &ReportModel); // only if FrontierMap exists
fn write_section_sensitivity(w: &mut Html, m: &ReportModel);
fn write_section_integrity_footer(w: &mut Html, m: &ReportModel);

fn ensure_keyboard_order(doc: &mut Html); // title → snapshot → sections

(Section writers insert verbatim blocks per Doc 7B.)
7) Algorithm Outline (bullet steps)
Initialize HTML doc, set language/meta; if bilingual, render mirrored full documents (not mixed paragraphs).
Emit sections in order §1→§10 exactly.
Ballot (§3): include the required approval-rate denominator sentence for approval ballots.
Panel (§…): use Doc 7B verbatim wording for quorum/majority/double-majority/symmetry (fill brackets).
Frontier (§8): only if FrontierMap exists; show status + diagnostics (mediation/enclave/protected counts; contiguity basis/island rule).
Sensitivity (§9): render 2×3 ±1pp/±5pp table if scenarios exist, else “N/A (not executed)”.
Integrity & footer (§10 + fixed footer): identifiers from RunRecord/Result; duplicate fixed footer line.
Precision: present one-decimal percentages & pp; seats integers; no double rounding.
Offline assets: inline/bundle all CSS/fonts and any map styles; no external fetch.
8) State Flow (very short)
Called by vm_report::lib after model build; reads model only; emits HTML; assets are local/bundled.
9) Determinism & Numeric Rules
Keyboard order fixed; deterministic template paths; no network; numbers appear as strings with one decimal (already formatted in model).
10) Edge Cases & Failure Policy
Validation failed: render Cover/Eligibility/Ballot + “Why this run is invalid…”, mark Outcome Invalid, omit Frontier.
Gates failed: render panel with ❌, Outcome “Invalid (gate failed: …)”, omit Frontier.
Frontier mediation/protected impacts: add ⚠ callout under Outcome; label becomes Marginal (already set upstream).
Bilingual: two full mirrored docs; do not mix languages within paragraphs.
11) Test Checklist (must pass)
Section order and wording blocks match Doc 7; approval sentence present for approval ballots.
One-decimal everywhere; seats integers; no double rounding.
Frontier only when FR exists; diagnostics & counts mirror artifacts.
Offline: no external asset requests; HTML renders with bundled fonts/styles.
```
