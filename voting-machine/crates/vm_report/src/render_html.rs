// crates/vm_report/src/render_html.rs — Part 1/3 (patched)
//
// Deterministic, offline HTML renderer with i18n and numeric formatting.
// This part defines helpers, the HTML builder, and renders through the
// end of the Eligibility section (Cover → Snapshot → Eligibility).
//
// Spec anchors (Docs 1–7 + Annexes A–C):
// • All strings deterministic & offline (no external assets).
// • IETF language tag honored for static strings (VM-VAR-062).
// • Integers rendered with consistent thousands separators (Doc 7).
// • HTML-escaped user/content fields.
// • Section order: Cover → Snapshot → Eligibility → … (next parts).
//
// NOTE: Paths to the model types may need adjusting to your crate layout.
// We assume `crate::model::ReportModel` and the nested field names used below.

use std::fmt::Write as _;

use crate::model::ReportModel;

// ------------------------- i18n phrasebook -------------------------
//
// Minimal, compile-time phrasebook. Extend as needed. We fall back to English
// if a phrase or language isn’t found.

#[derive(Copy, Clone)]
struct Phrase {
    key: &'static str,
    en: &'static str,
    fr: &'static str,
}

const PHRASES: &[Phrase] = &[
    Phrase { key: "title_report", en: "Voting Model Report", fr: "Rapport du modèle de vote" },
    Phrase { key: "snapshot",     en: "Snapshot",            fr: "Aperçu" },
    Phrase { key: "eligibility",  en: "Eligibility & Rolls", fr: "Éligibilité et registres" },
    Phrase { key: "roll_policy",  en: "Roll policy",         fr: "Règle du registre" },
    Phrase { key: "registry_src", en: "Registry source",     fr: "Source du registre" },
    Phrase { key: "eligible",     en: "Eligible",            fr: "Éligibles" },
    Phrase { key: "cast",         en: "Cast",                fr: "Bulletins déposés" },
    Phrase { key: "valid",        en: "Valid",               fr: "Valides" },
    Phrase { key: "reason",       en: "Reason",              fr: "Raison" },
    Phrase { key: "outcome",      en: "Outcome",             fr: "Résultat" },
];

fn t(lang: &str, key: &str) -> &'static str {
    // Normalize a few common tags to a 2-letter canonical form
    let lang2 = match lang {
        "fr" | "fr-FR" | "fr_CA" | "fr-CA" => "fr",
        _ => "en",
    };
    for p in PHRASES {
        if p.key == key {
            return if lang2 == "fr" { p.fr } else { p.en };
        }
    }
    // fallback
    key
}

// ------------------------- formatting helpers -------------------------

/// Escape text for HTML (minimal, deterministic).
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Format a non-negative integer with a *narrow no-break space* thousands separator (U+202F).
/// This satisfies Doc 7’s “consistent thousands separator” requirement and prevents line breaks.
fn fmt_int<T: Into<u128>>(n: T) -> String {
    let mut x = n.into();
    let mut buf = [0u8; 40]; // enough for u128 with separators
    let mut i = buf.len();
    let mut digits = 0usize;

    if x == 0 {
        return "0".to_string();
    }

    while x > 0 {
        if digits > 0 && digits % 3 == 0 {
            i -= 3; // U+202F is 3 bytes in UTF-8 (0xE2 0x80 0xAF)
            buf[i..i + 3].copy_from_slice(&[0xE2, 0x80, 0xAF]);
        }
        let d = (x % 10) as u8;
        i -= 1;
        buf[i] = b'0' + d;
        x /= 10;
        digits += 1;
    }
    String::from_utf8(buf[i..].to_vec()).unwrap()
}

// ------------------------- HTML builder -------------------------

pub struct HtmlBuilder<'a> {
    lang: &'a str,
    buf: String,
}

impl<'a> HtmlBuilder<'a> {
    pub fn new(lang: &'a str) -> Self {
        Self {
            lang,
            buf: String::with_capacity(32 * 1024),
        }
    }

    /// Start document with minimal head. Deterministic, asset-free.
    pub fn start(&mut self, title: &str) {
        // Use the requested lang on <html lang="…">
        let _ = write!(
            self.buf,
            "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\">\
             <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
             <title>{}</title>\
             <style>\
             body{{font-family:system-ui,-apple-system,Segoe UI,Roboto,Ubuntu,Arial,sans-serif;margin:24px;}}\
             h1,h2,h3{{margin:0.2em 0;}}\
             .kv ul{{list-style:none;padding-left:0}}\
             .kv li{{margin:2px 0}}\
             .muted{{opacity:0.8}}\
             .note{{font-style:italic;opacity:0.9}}\
             .grid{{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:8px}}\
             .pill{{display:inline-block;padding:.2em .6em;border-radius:9999px;background:#eee}}\
             table{{border-collapse:collapse}}\
             td,th{{padding:4px 8px;border-bottom:1px solid #ddd;text-align:left}}\
             </style></head><body>",
            esc(self.lang),
            esc(title)
        );
    }

    /// Close document.
    pub fn finish(mut self) -> String {
        self.buf.push_str("</body></html>");
        self.buf
    }

    /// Cover section: H1 (title), H2 (Outcome label), optional reason.
    pub fn section_cover(&mut self, title: &str, label: &str, reason: Option<&str>) {
        let _ = write!(
            self.buf,
            "<h1>{}</h1><h2>{}: {}</h2>",
            esc(title),
            esc(t(self.lang, "outcome")),
            esc(label)
        );
        if let Some(r) = reason {
            let _ = write!(self.buf, "<p class=\"muted\"><b>{}:</b> {}</p>", esc(t(self.lang, "reason")), esc(r));
        }
    }

    /// Snapshot key-value pairs.
    pub fn section_snapshot<'b, I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (&'b str, &'b str)>,
    {
        let _ = write!(self.buf, "<h3>{}</h3><div class=\"kv\"><ul>", esc(t(self.lang, "snapshot")));
        for (k, v) in items {
            let _ = write!(self.buf, "<li><b>{}</b>: {}</li>", esc(k), esc(v));
        }
        self.buf.push_str("</ul></div>");
    }

    /// Eligibility section with totals, roll policy & source.
    pub fn section_eligibility(
        &mut self,
        roll_policy: &str,
        registry_source: &str,
        eligible_roll: u64,
        ballots_cast: u64,
        valid_ballots: u64,
        per_unit_quorum_note: Option<&str>,
    ) {
        let _ = write!(
            self.buf,
            "<h3>{}</h3>\
             <p>{}: {}<br>{}: {}</p>\
             <div class=\"grid\">\
               <div><div class=\"pill\">{}</div><div><b>{}</b></div></div>\
               <div><div class=\"pill\">{}</div><div><b>{}</b></div></div>\
               <div><div class=\"pill\">{}</div><div><b>{}</b></div></div>\
             </div>",
            esc(t(self.lang, "eligibility")),
            esc(t(self.lang, "roll_policy")), esc(roll_policy),
            esc(t(self.lang, "registry_src")), esc(registry_source),
            esc(t(self.lang, "eligible")), fmt_int(eligible_roll),
            esc(t(self.lang, "cast")),     fmt_int(ballots_cast),
            esc(t(self.lang, "valid")),    fmt_int(valid_ballots),
        );
        if let Some(note) = per_unit_quorum_note {
            let _ = write!(self.buf, "<p class=\"note\">{}</p>", esc(note));
        }
    }
}

// ------------------------- top-level entry (part 1 only) -------------------------

/// Render the initial part of the report (Cover → Snapshot → Eligibility).
/// The caller will continue with parts 2/3 to append further sections.
pub fn render_html_part1(model: &ReportModel, lang: &str) -> HtmlBuilder<'_> {
    let mut h = HtmlBuilder::new(lang);
    h.start(t(lang, "title_report"));

    // Cover
    h.section_cover(
        &model.cover.title,
        &model.cover.label,
        model.cover.reason.as_deref(),
    );

    // Snapshot (key-value)
    h.section_snapshot(model.snapshot.items.iter().map(|it| (it.key.as_str(), it.value.as_str())));

    // Eligibility
    h.section_eligibility(
        &model.eligibility.roll_policy,
        &model.eligibility.registry_source,
        model.eligibility.totals.eligible_roll,
        model.eligibility.totals.ballots_cast,
        model.eligibility.totals.valid_ballots,
        model.eligibility.per_unit_quorum_note.as_deref(),
    );

    h
}

// crates/vm_report/src/render_html.rs — Part 2/3 (patched)
//
// This part appends the Ballot & Allocation, Legitimacy Gates, and Outcome
// sections to the HTML built in Part 1. It uses the same HtmlBuilder defined
// earlier and relies on simple, deterministic formatting only.
//
// NOTE: If you extend i18n, consider adding keys for these section labels
// into the Part 1 phrasebook. For now, headings are English-only.

use crate::model::ReportModel;
use std::fmt::Write as _;

// Reuse helpers & HtmlBuilder from Part 1:
// - esc(&str) -> String
// - fmt_int(..) -> String
// - t(lang, key)
// - HtmlBuilder with start/finish/section_* methods used there.

impl<'a> super::HtmlBuilder<'a> {
    /// Ballot & Allocation section.
    pub fn section_ballot_and_allocation(
        &mut self,
        method: &str,
        allocation: &str,
        weighting: &str,
        approval_denominator_sentence: Option<&str>,
    ) {
        let _ = write!(
            self.buf,
            "<h3>Ballot &amp; Allocation</h3>\
             <p>Method: {} &nbsp;|&nbsp; Allocation: {} &nbsp;|&nbsp; Weighting: {}</p>",
            esc(method),
            esc(allocation),
            esc(weighting)
        );
        if let Some(sent) = approval_denominator_sentence {
            let _ = write!(self.buf, "<p class=\"note\">{}</p>", esc(sent));
        }
    }

    /// Legitimacy gates table (quorum, majority, optional double-majority) + reasons + overall pass.
    #[allow(clippy::too_many_arguments)]
    pub fn section_legitimacy_gates(
        &mut self,
        quorum_value_pct_1dp: &str,
        quorum_threshold_pct_0dp: &str,
        quorum_pass: bool,
        majority_value_pct_1dp: &str,
        majority_threshold_pct_0dp: &str,
        majority_pass: bool,
        double_nat: Option<(&str, &str, bool)>, // (value_pct_1dp, threshold_pct_0dp, pass)
        double_fam: Option<(&str, &str, bool)>,
        reasons: &[String],
        overall_pass: bool,
        denom_note: Option<&str>,   // optional explanatory note (e.g., approval denominator)
        members_hint: Option<&str>, // optional "family members included" hint
    ) {
        let yes = |b: bool| if b { "✅" } else { "❌" };

        // Header
        self.buf.push_str("<h3>Legitimacy Gates</h3>");

        // Table
        self.buf.push_str("<table><thead><tr><th>Gate</th><th>Value</th><th>Threshold</th><th>Pass</th></tr></thead><tbody>");

        // Quorum
        let _ = write!(
            self.buf,
            "<tr><td>Quorum</td><td>{}%</td><td>{}%</td><td>{}</td></tr>",
            esc(quorum_value_pct_1dp),
            esc(quorum_threshold_pct_0dp),
            yes(quorum_pass)
        );

        // Majority
        let _ = write!(
            self.buf,
            "<tr><td>Majority</td><td>{}%</td><td>{}%</td><td>{}</td></tr>",
            esc(majority_value_pct_1dp),
            esc(majority_threshold_pct_0dp),
            yes(majority_pass)
        );

        // Double-majority (if present)
        if let (Some((nat_val, nat_thr, nat_ok)), Some((fam_val, fam_thr, fam_ok))) = (double_nat, double_fam) {
            let _ = write!(
                self.buf,
                "<tr><td>Double majority — National</td><td>{}%</td><td>{}%</td><td>{}</td></tr>",
                esc(nat_val),
                esc(nat_thr),
                yes(nat_ok)
            );
            let _ = write!(
                self.buf,
                "<tr><td>Double majority — Family</td><td>{}%</td><td>{}%</td><td>{}</td></tr>",
                esc(fam_val),
                esc(fam_thr),
                yes(fam_ok)
            );
        }

        self.buf.push_str("</tbody></table>");

        // Notes / hints (optional)
        if denom_note.is_some() || members_hint.is_some() {
            self.buf.push_str("<p class=\"note\">");
            let mut first = true;
            if let Some(n) = denom_note {
                let _ = write!(self.buf, "{}", esc(n));
                first = false;
            }
            if let Some(h) = members_hint {
                if !first {
                    self.buf.push_str(" &nbsp;•&nbsp; ");
                }
                let _ = write!(self.buf, "{}", esc(h));
            }
            self.buf.push_str("</p>");
        }

        // Overall result for the legitimacy panel
        let _ = write!(
            self.buf,
            "<p><b>Pass:</b> {}</p>",
            yes(overall_pass)
        );

        // Reasons (if any)
        if !reasons.is_empty() {
            self.buf.push_str("<ul>");
            for r in reasons {
                let _ = write!(self.buf, "<li>{}</li>", esc(r));
            }
            self.buf.push_str("</ul>");
        }
    }

    /// Outcome section (label, reason, national margin).
    pub fn section_outcome(&mut self, label: &str, reason: &str, national_margin_pp: &str) {
        let _ = write!(
            self.buf,
            "<h3>{}</h3><p><b>Label:</b> {}<br><b>{}:</b> {}<br><b>National margin:</b> {}</p>",
            esc(t(self.lang, "outcome")),
            esc(label),
            esc(t(self.lang, "reason")),
            esc(reason),
            esc(national_margin_pp)
        );
    }
}

// ------------------------- part 2 entrypoint -------------------------

/// Append Ballot & Allocation, Legitimacy, and Outcome to an existing HtmlBuilder.
///
/// Usage:
/// ```ignore
/// let h = render_html_part1(&model, lang);
/// let h = render_html_part2(h, &model);
/// // then call part 3 and finally h.finish()
/// ```
pub fn render_html_part2<'a>(mut h: super::HtmlBuilder<'a>, model: &ReportModel) -> super::HtmlBuilder<'a> {
    // Ballot & Allocation
    h.section_ballot_and_allocation(
        &model.ballot_method.method,
        &model.ballot_method.allocation,
        &model.ballot_method.weighting,
        model.ballot_method.approval_denominator_sentence.as_deref(),
    );

    // Legitimacy
    let leg = &model.legitimacy_panel;
    let dm = &leg.double_majority; // Option<(nat, fam)>
    let (nat_opt, fam_opt) = if let Some((nat, fam)) = dm {
        (
            Some((nat.value_pct_1dp.as_str(), nat.threshold_pct_0dp.as_str(), nat.pass)),
            Some((fam.value_pct_1dp.as_str(), fam.threshold_pct_0dp.as_str(), fam.pass)),
        )
    } else {
        (None, None)
    };

    // Optional notes (if your model exposes them; otherwise None)
    let denom_note: Option<&str> = None;    // wire from model if available
    let members_hint: Option<&str> = None;  // wire from model if available

    h.section_legitimacy_gates(
        &leg.quorum.value_pct_1dp,
        &leg.quorum.threshold_pct_0dp,
        leg.quorum.pass,
        &leg.majority.value_pct_1dp,
        &leg.majority.threshold_pct_0dp,
        leg.majority.pass,
        nat_opt,
        fam_opt,
        &leg.reasons,
        leg.pass,
        denom_note,
        members_hint,
    );

    // Outcome
    h.section_outcome(
        &model.outcome_label.label,
        &model.outcome_label.reason,
        &model.outcome_label.national_margin_pp,
    );

    h
}

// crates/vm_report/src/render_html.rs — Part 3/3 (patched)
//
// This part appends Frontier (optional), Sensitivity (optional), and Integrity,
// then returns the finished HTML string. It reuses the HtmlBuilder and helpers
// (esc, fmt_int, t) defined in Part 1.

use crate::model::ReportModel;
use std::fmt::Write as _;

impl<'a> super::HtmlBuilder<'a> {
    /// Frontier section (optional). Displays mode/policies and counters,
    /// plus an optional band summary list.
    pub fn section_frontier(
        &mut self,
        mode: &str,
        edge_policy: &str,
        island_rule: &str,
        changed: u64,
        no_change: u64,
        mediation: u64,
        enclave: u64,
        protected_blocked: u64,
        quorum_blocked: u64,
        bands_summary: &[String],
    ) {
        let _ = write!(
            self.buf,
            "<h3>Frontier</h3>\
             <p>Mode: {} &nbsp;|&nbsp; Edge policy: {} &nbsp;|&nbsp; Island rule: {}</p>",
            esc(mode),
            esc(edge_policy),
            esc(island_rule),
        );

        // Counters grid
        let _ = write!(
            self.buf,
            "<div class=\"grid\">\
               <div><div class=\"pill\">Changed</div><div><b>{}</b></div></div>\
               <div><div class=\"pill\">No change</div><div><b>{}</b></div></div>\
               <div><div class=\"pill\">Mediation</div><div><b>{}</b></div></div>\
               <div><div class=\"pill\">Enclave</div><div><b>{}</b></div></div>\
               <div><div class=\"pill\">Protected blocked</div><div><b>{}</b></div></div>\
               <div><div class=\"pill\">Quorum blocked</div><div><b>{}</b></div></div>\
             </div>",
            super::fmt_int(changed),
            super::fmt_int(no_change),
            super::fmt_int(mediation),
            super::fmt_int(enclave),
            super::fmt_int(protected_blocked),
            super::fmt_int(quorum_blocked),
        );

        if !bands_summary.is_empty() {
            self.buf.push_str("<ul>");
            for s in bands_summary {
                let _ = write!(self.buf, "<li>{}</li>", esc(s));
            }
            self.buf.push_str("</ul>");
        }
    }

    /// Sensitivity section (optional). Renders a compact table, each row joined by " | ".
    pub fn section_sensitivity(&mut self, table: &[Vec<String>]) {
        self.buf.push_str("<h3>Sensitivity</h3>");
        if table.is_empty() {
            self.buf.push_str("<p class=\"muted\">(no sensitivity rows)</p>");
            return;
        }
        for row in table {
            let line = row.iter().map(|c| esc(c)).collect::<Vec<_>>().join(" | ");
            let _ = write!(self.buf, "<p>{}</p>", line);
        }
    }

    /// Integrity section: IDs and engine meta; optional fields shown when present.
    pub fn section_integrity(
        &mut self,
        result_id_hex: &str,
        run_id_hex: Option<&str>,
        formula_id_hex: &str,
        frontier_id_hex: Option<&str>,
        engine_vendor: &str,
        engine_name: &str,
        engine_version: &str,
        engine_build: &str,
        tie_seed_hex: Option<&str>,
    ) {
        self.buf.push_str("<h3>Integrity</h3><p>");
        let _ = write!(self.buf, "<b>Result ID:</b> {}<br>", esc(result_id_hex));
        if let Some(run) = run_id_hex {
            let _ = write!(self.buf, "<b>Run ID:</b> {}<br>", esc(run));
        }
        let _ = write!(self.buf, "<b>Formula ID:</b> {}<br>", esc(formula_id_hex));
        if let Some(fr) = frontier_id_hex {
            let _ = write!(self.buf, "<b>Frontier ID:</b> {}<br>", esc(fr));
        }
        let _ = write!(
            self.buf,
            "<b>Engine:</b> {} {} ({}) build {}",
            esc(engine_vendor),
            esc(engine_name),
            esc(engine_version),
            esc(engine_build)
        );
        if let Some(seed) = tie_seed_hex {
            let _ = write!(self.buf, "<br><b>Tie seed:</b> {}", esc(seed));
        }
        self.buf.push_str("</p>");
    }
}

// ------------------------- part 3 entrypoints -------------------------

/// Append Frontier/Sensitivity/Integrity and finish the document, returning HTML.
///
/// Usage:
/// ```ignore
/// let h = render_html_part1(&model, lang);
/// let h = render_html_part2(h, &model);
/// let html = render_html_part3(h, &model); // returns String
/// ```
pub fn render_html_part3<'a>(mut h: super::HtmlBuilder<'a>, model: &ReportModel) -> String {
    // Frontier (optional)
    if let Some(fr) = &model.frontier {
        h.section_frontier(
            &fr.mode,
            &fr.edge_policy,
            &fr.island_rule,
            fr.counters.changed,
            fr.counters.no_change,
            fr.counters.mediation,
            fr.counters.enclave,
            fr.counters.protected_blocked,
            fr.counters.quorum_blocked,
            &fr.bands_summary,
        );
    }

    // Sensitivity (optional)
    if let Some(sens) = &model.sensitivity {
        h.section_sensitivity(&sens.table);
    }

    // Integrity (required)
    let integ = &model.integrity;
    h.section_integrity(
        integ.result_id.as_str(),
        integ.run_id.as_deref(),
        &integ.formula_id_hex,
        integ.frontier_id.as_deref(),
        &integ.engine_vendor,
        &integ.engine_name,
        &integ.engine_version,
        &integ.engine_build,
        integ.tie_seed.as_deref(),
    );

    // Finish document
    h.finish()
}

/// Convenience: render the full report in one call (Parts 1→3).
#[allow(dead_code)]
pub fn render_html_full(model: &ReportModel, lang: &str) -> String {
    let h1 = super::render_html_part1(model, lang);
    let h2 = super::render_html_part2(h1, model);
    super::render_html_part3(h2, model)
}
