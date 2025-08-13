//! vm_report/src/render_html.rs
//! Deterministic, offline HTML renderer for ReportModel (Doc 7).
//! No I/O, no RNG, no external assets.

#![deny(unsafe_code)]

use crate::structure::ReportModel;

// ===== Public API =====

/// Render a ReportModel into a single (or bilingual) HTML string.
/// Deterministic: same model → identical bytes.
pub fn render_html(model: &ReportModel, opts: HtmlRenderOpts) -> String {
    match &opts.bilingual {
        Some(pair) => {
            // Two full documents back-to-back (simple & deterministic).
            let mut out = String::new();
            out.push_str(&render_one(model, &opts, &pair.primary_lang));
            out.push_str("\n<!-- ---- bilingual separator ---- -->\n");
            out.push_str(&render_one(model, &opts, &pair.secondary_lang));
            out
        }
        None => render_one(model, &opts, "en"),
    }
}

// ===== Options & writer =====

#[derive(Clone, Default)]
pub struct HtmlRenderOpts {
    pub bilingual: Option<BilingualPair>, // (primary_lang, secondary_lang)
    pub embed_assets: bool,               // inline CSS/JS/fonts from bundle
}

#[derive(Clone)]
pub struct BilingualPair {
    pub primary_lang: String,
    pub secondary_lang: String,
}

// Minimal writer with deterministic push order.
struct Html {
    buf: String,
}
impl Html {
    fn new() -> Self { Self { buf: String::with_capacity(32 * 1024) } }
    fn push<S: AsRef<str>>(&mut self, s: S) { self.buf.push_str(s.as_ref()); }
    fn finish(self) -> String { self.buf }
}

// ===== Internal: render one full document =====

fn render_one(model: &ReportModel, opts: &HtmlRenderOpts, lang: &str) -> String {
    let mut w = Html::new();

    // Document head
    w.push("<!DOCTYPE html><html lang=\"");
    w.push(esc(lang));
    w.push("\"><head><meta charset=\"utf-8\">\
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
            <meta http-equiv=\"x-ua-compatible\" content=\"ie=edge\">\
            <title>Voting Model Report</title>");
    if opts.embed_assets {
        w.push("<style>");
        w.push(embed_assets_css());
        // optional fonts (kept empty by default; placeholder included for determinism)
        let fonts = embed_assets_fonts_base64();
        if !fonts.is_empty() {
            w.push(fonts);
        }
        w.push("</style>");
    } else {
        // Still avoid external links; include a minimal default CSS.
        w.push("<style>");
        w.push(MINIMAL_CSS);
        w.push("</style>");
    }
    w.push("</head><body>");

    let (main_role, contentinfo_role) = aria_landmarks();
    w.push("<main "); w.push(main_role); w.push(">");

    // Sections in Doc 7 order
    write_section_cover_snapshot(&mut w, model);
    write_section_eligibility(&mut w, model);
    write_section_ballot(&mut w, model);
    write_section_panel(&mut w, model);
    write_section_outcome(&mut w, model);
    write_section_frontier(&mut w, model);
    write_section_sensitivity(&mut w, model);
    write_section_integrity_footer(&mut w, model);

    w.push("</main>");
    w.push("<footer "); w.push(contentinfo_role); w.push(" class=\"footer-ids\">");
    // IDs line (fixed order; seed shown upstream within Integrity section when applicable)
    w.push("<div class=\"ids\">");
    w.push("<span>Result: "); w.push(esc(&model.footer.result_id.to_string())); w.push("</span>");
    w.push(" · <span>Run: "); w.push(esc(&model.footer.run_id.to_string())); w.push("</span>");
    if let Some(fr) = &model.footer.frontier_id {
        w.push(" · <span>Frontier: "); w.push(esc(&fr.to_string())); w.push("</span>");
    }
    w.push(" · <span>Registry: "); w.push(esc(&model.footer.reg_id.to_string())); w.push("</span>");
    w.push(" · <span>Params: "); w.push(esc(&model.footer.param_set_id.to_string())); w.push("</span>");
    if let Some(tly) = &model.footer.tally_id {
        w.push(" · <span>Tally: "); w.push(esc(&tly.to_string())); w.push("</span>");
    }
    w.push("</div></footer>");

    w.push("</body></html>");
    let mut html = w.finish();
    ensure_keyboard_order(&mut html);
    html
}

// ===== Section writers =====

fn write_section_cover_snapshot(w: &mut Html, m: &ReportModel) {
    w.push("<section id=\"cover\" role=\"region\" aria-labelledby=\"h-cover\">");
    w.push("<h1 id=\"h-cover\">Voting Model Report</h1>");

    // Badge for outcome label on cover
    let label_class = match m.cover.label.as_str() {
        "Decisive" => "badge-dec",
        "Marginal" => "badge-mar",
        _ => "badge-inv",
    };
    w.push("<div class=\"cover-card\">");
    w.push("<div class=\"label-row\"><span class=\"badge ");
    w.push(label_class);
    w.push("\">");
    w.push(esc(&m.cover.label));
    w.push("</span>");
    if let Some(reason) = &m.cover.reason {
        w.push("<span class=\"reason\">");
        w.push(esc(reason));
        w.push("</span>");
    }
    w.push("</div>");

    // Registry provenance
    if !m.cover.registry_name.is_empty() || !m.cover.registry_published_date.is_empty() {
        w.push("<div class=\"registry\">");
        if !m.cover.registry_name.is_empty() {
            w.push("<span class=\"k\">Registry</span><span class=\"v\">");
            w.push(esc(&m.cover.registry_name));
            w.push("</span>");
        }
        if !m.cover.registry_published_date.is_empty() {
            w.push("<span class=\"k\">Published</span><span class=\"v\">");
            w.push(esc(&m.cover.registry_published_date));
            w.push("</span>");
        }
        w.push("</div>");
    }

    // Snapshot variables
    if !m.cover.snapshot_vars.is_empty() {
        w.push("<dl class=\"snapshot\">");
        for kv in &m.cover.snapshot_vars {
            w.push("<dt>");
            w.push(esc(&kv.key));
            w.push("</dt><dd>");
            w.push(esc(&kv.value));
            w.push("</dd>");
        }
        w.push("</dl>");
    }

    w.push("</div></section>");
}

fn write_section_eligibility(w: &mut Html, m: &ReportModel) {
    w.push("<section id=\"eligibility\" role=\"region\" aria-labelledby=\"h-elig\">");
    w.push("<h2 id=\"h-elig\">Eligibility & Rolls</h2>");

    w.push("<div class=\"elig-grid\">");
    w.push("<div class=\"row\"><span class=\"k\">Roll policy</span><span class=\"v\">");
    w.push(esc(&m.eligibility.roll_policy));
    w.push("</span></div>");

    w.push("<div class=\"row\"><span class=\"k\">Eligible roll (Σ)</span><span class=\"v\">");
    w.push(m.eligibility.totals_eligible_roll.to_string());
    w.push("</span></div>");

    w.push("<div class=\"row\"><span class=\"k\">Ballots cast (Σ)</span><span class=\"v\">");
    w.push(m.eligibility.totals_ballots_cast.to_string());
    w.push("</span></div>");

    w.push("<div class=\"row\"><span class=\"k\">Valid ballots (Σ)</span><span class=\"v\">");
    w.push(m.eligibility.totals_valid_ballots.to_string());
    w.push("</span></div>");

    if let Some(note) = &m.eligibility.per_unit_quorum_note {
        w.push("<div class=\"note\">");
        w.push(esc(note));
        w.push("</div>");
    }
    if !m.eligibility.provenance.is_empty() {
        w.push("<div class=\"provenance\">");
        w.push(esc(&m.eligibility.provenance));
        w.push("</div>");
    }
    w.push("</div></section>");
}

fn write_section_ballot(w: &mut Html, m: &ReportModel) {
    w.push("<section id=\"ballot\" role=\"region\" aria-labelledby=\"h-ballot\">");
    w.push("<h2 id=\"h-ballot\">Ballot & Allocation</h2>");

    w.push("<ul class=\"kv\">");
    w.push("<li><span class=\"k\">Ballot type</span><span class=\"v\">");
    w.push(esc(&m.ballot.ballot_type));
    w.push("</span></li>");
    w.push("<li><span class=\"k\">Allocation</span><span class=\"v\">");
    w.push(esc(&m.ballot.allocation_method));
    w.push("</span></li>");
    w.push("<li><span class=\"k\">Weighting</span><span class=\"v\">");
    w.push(esc(&m.ballot.weighting_method));
    w.push("</span></li>");
    w.push("</ul>");

    if m.ballot.approval_denominator_sentence {
        w.push("<p class=\"aside\"><em>Approval rate is computed as ");
        w.push("<code>approvals</code> / <code>valid ballots</code>.</em></p>");
    }

    w.push("</section>");
}

fn write_section_panel(w: &mut Html, m: &ReportModel) {
    w.push("<section id=\"legitimacy\" role=\"region\" aria-labelledby=\"h-leg\">");
    w.push("<h2 id=\"h-leg\">Legitimacy Gates</h2>");

    // Table with quorum + majority (always shown)
    w.push("<table class=\"panel\"><thead><tr>\
            <th>Gate</th><th>Observed</th><th>Threshold</th><th>Status</th>\
            </tr></thead><tbody>");

    // Row helper
    let mut row = |name: &str, val: &str, th: &str, pass: bool| {
        w.push("<tr><td>");
        w.push(esc(name));
        w.push("</td><td>");
        w.push(esc(val));
        w.push("</td><td>");
        w.push(esc(th));
        w.push("</td><td>");
        if pass { w.push("✅ Pass"); } else { w.push("❌ Fail"); }
        w.push("</td></tr>");
    };

    row("Quorum", &m.panel.quorum.value_pct_1dp, &m.panel.quorum.threshold_pct_0dp, m.panel.quorum.pass);
    row("Majority", &m.panel.majority.value_pct_1dp, &m.panel.majority.threshold_pct_0dp, m.panel.majority.pass);

    if let Some((nat, fam)) = &m.panel.double_majority {
        row("Double majority — National", &nat.value_pct_1dp, &nat.threshold_pct_0dp, nat.pass);
        row("Double majority — Family", &fam.value_pct_1dp, &fam.threshold_pct_0dp, fam.pass);
    }

    w.push("</tbody></table>");

    if let Some(sym) = m.panel.symmetry {
        w.push("<p class=\"symmetry\">Symmetry respected: ");
        w.push(if sym { "true" } else { "false" });
        w.push("</p>");
    }

    if !m.panel.reasons.is_empty() && !m.panel.pass {
        w.push("<div class=\"reasons\"><strong>Reasons:</strong><ul>");
        for r in &m.panel.reasons {
            w.push("<li>"); w.push(esc(r)); w.push("</li>");
        }
        w.push("</ul></div>");
    }

    w.push("</section>");
}

fn write_section_outcome(w: &mut Html, m: &ReportModel) {
    w.push("<section id=\"outcome\" role=\"region\" aria-labelledby=\"h-out\">");
    w.push("<h2 id=\"h-out\">Outcome</h2>");
    let label_class = match m.outcome.label.as_str() {
        "Decisive" => "badge-dec",
        "Marginal" => "badge-mar",
        _ => "badge-inv",
    };
    w.push("<p class=\"outcome\"><span class=\"badge ");
    w.push(label_class);
    w.push("\">");
    w.push(esc(&m.outcome.label));
    w.push("</span> — ");
    w.push(esc(&m.outcome.reason));
    w.push("</p>");
    w.push("<p class=\"margin\">National margin: <strong>");
    w.push(esc(&m.outcome.national_margin_pp));
    w.push("</strong></p>");
    w.push("</section>");
}

fn write_section_frontier(w: &mut Html, m: &ReportModel) {
    if let Some(fr) = &m.frontier {
        w.push("<section id=\"frontier\" role=\"region\" aria-labelledby=\"h-fr\">");
        w.push("<h2 id=\"h-fr\">Frontier</h2>");

        w.push("<ul class=\"kv\">");
        w.push("<li><span class=\"k\">Mode</span><span class=\"v\">");
        w.push(esc(&fr.mode));
        w.push("</span></li>");
        w.push("<li><span class=\"k\">Edge policy</span><span class=\"v\">");
        w.push(esc(&fr.edge_types));
        w.push("</span></li>");
        w.push("<li><span class=\"k\">Island rule</span><span class=\"v\">");
        w.push(esc(&fr.island_rule));
        w.push("</span></li>");
        w.push("</ul>");

        // Counters
        w.push("<div class=\"fr-counters\">");
        let c = &fr.counters;
        w.push(format!("<span>Changed: {}</span>", c.changed).as_str());
        w.push(" · ");
        w.push(format!("<span>No change: {}</span>", c.no_change).as_str());
        w.push(" · ");
        w.push(format!("<span>Mediation: {}</span>", c.mediation).as_str());
        w.push(" · ");
        w.push(format!("<span>Enclave: {}</span>", c.enclave).as_str());
        w.push(" · ");
        w.push(format!("<span>Protected blocked: {}</span>", c.protected_blocked).as_str());
        w.push(" · ");
        w.push(format!("<span>Quorum blocked: {}</span>", c.quorum_blocked).as_str());
        w.push("</div>");

        if !fr.bands_summary.is_empty() {
            w.push("<div class=\"bands\"><strong>Bands:</strong> <ul>");
            for b in &fr.bands_summary {
                w.push("<li>"); w.push(esc(b)); w.push("</li>");
            }
            w.push("</ul></div>");
        }

        w.push("</section>");
    }
}

fn write_section_sensitivity(w: &mut Html, m: &ReportModel) {
    w.push("<section id=\"sensitivity\" role=\"region\" aria-labelledby=\"h-sens\">");
    w.push("<h2 id=\"h-sens\">Sensitivity</h2>");
    if let Some(s) = &m.sensitivity {
        w.push("<table class=\"sens\"><tbody>");
        for row in &s.table_2x3 {
            w.push("<tr>");
            for cell in row {
                w.push("<td>"); w.push(esc(cell)); w.push("</td>");
            }
            w.push("</tr>");
        }
        w.push("</tbody></table>");
    } else {
        w.push("<p class=\"na\">N/A (not executed)</p>");
    }
    w.push("</section>");
}

fn write_section_integrity_footer(w: &mut Html, m: &ReportModel) {
    w.push("<section id=\"integrity\" role=\"region\" aria-labelledby=\"h-int\">");
    w.push("<h2 id=\"h-int\">Integrity & Reproducibility</h2>");
    w.push("<ul class=\"kv\">");
    w.push("<li><span class=\"k\">Engine</span><span class=\"v\">");
    w.push(esc(&m.integrity.engine_vendor));
    w.push(" / ");
    w.push(esc(&m.integrity.engine_name));
    w.push(" ");
    w.push(esc(&m.integrity.engine_version));
    w.push(" (");
    w.push(esc(&m.integrity.engine_build));
    w.push(")</span></li>");
    w.push("<li><span class=\"k\">Formula ID</span><span class=\"v mono\">");
    w.push(esc(&m.integrity.formula_id_hex));
    w.push("</span></li>");
    w.push("<li><span class=\"k\">Tie policy</span><span class=\"v\">");
    w.push(esc(&m.integrity.tie_policy));
    if m.integrity.tie_policy == "random" {
        if let Some(seed) = &m.integrity.tie_seed {
            w.push(" — seed ");
            w.push(esc(seed));
        }
    }
    w.push("</span></li>");
    w.push("<li><span class=\"k\">Started (UTC)</span><span class=\"v\">");
    w.push(esc(&m.integrity.started_utc));
    w.push("</span></li>");
    w.push("<li><span class=\"k\">Finished (UTC)</span><span class=\"v\">");
    w.push(esc(&m.integrity.finished_utc));
    w.push("</span></li>");
    w.push("</ul>");
    w.push("</section>");
}

// ===== Utilities =====

fn embed_assets_css() -> &'static str {
    // Simple, deterministic, offline CSS (no external @import).
    r#"html{font-family:system-ui,-apple-system,Segoe UI,Roboto,Ubuntu,Cantarell,Noto Sans,"Helvetica Neue",Arial,"Apple Color Emoji","Segoe UI Emoji";line-height:1.35}
body{margin:0;padding:0;background:#fff;color:#111}
main{display:block;max-width:980px;margin:0 auto;padding:24px}
h1,h2{margin:.2em 0 .4em 0}
h1{font-size:1.8rem} h2{font-size:1.3rem}
.k{color:#444;margin-right:.5rem}
.v{color:#000}
.mono{font-family:ui-monospace,Menlo,Consolas,monospace}
.cover-card{border:1px solid #ddd;border-radius:8px;padding:16px;margin:8px 0 16px}
.snapshot{display:grid;grid-template-columns:max-content 1fr;gap:8px 16px;margin-top:8px}
.elig-grid .row{display:flex;gap:12px;margin:4px 0}
.note{margin-top:8px;color:#333}
.provenance{margin-top:6px;color:#555}
.kv{list-style:none;padding-left:0} .kv li{margin:4px 0}
.panel{border-collapse:collapse;width:100%;margin:8px 0}
.panel th,.panel td{border:1px solid #ddd;padding:6px 8px;text-align:left}
.badge{display:inline-block;font-weight:600;padding:.15rem .5rem;border-radius:6px}
.badge-dec{background:#e6ffed;color:#046a1d;border:1px solid #b4f2c2}
.badge-mar{background:#fff7e6;color:#7f4b00;border:1px solid #f2deaf}
.badge-inv{background:#ffe6e6;color:#8a0000;border:1px solid #f2bbbb}
.outcome{font-size:1.05rem}
.margin{color:#333}
.fr-counters{margin:.5rem 0;color:#222}
.bands ul{margin:.3rem 0 .6rem 1rem}
.aside{color:#333}
.na{color:#555;font-style:italic}
.footer-ids{background:#f8f8f8;border-top:1px solid #e5e5e5;padding:10px 16px}
.footer-ids .ids{max-width:980px;margin:0 auto;color:#333}
"#
}

fn embed_assets_fonts_base64() -> &'static str {
    // Keep empty by default to avoid inflating output; placeholder for future.
    ""
}

const MINIMAL_CSS: &str = r#"body{font-family:system-ui,Arial,sans-serif}main{max-width:980px;margin:0 auto;padding:24px}"#;

/// Keep tab/anchor order stable; currently a no-op with fixed layout.
fn ensure_keyboard_order(_html: &mut String) {}

/// ARIA roles for main & footer landmarks.
fn aria_landmarks() -> (&'static str, &'static str) { ("role=\"main\"", "role=\"contentinfo\"") }

/// Minimal HTML escaping (no allocations beyond return String).
fn esc<S: AsRef<str>>(s: S) -> String {
    let mut out = String::with_capacity(s.as_ref().len() + 8);
    for ch in s.as_ref().chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\''=> out.push_str("&#39;"),
            _   => out.push(ch),
        }
    }
    out
}
