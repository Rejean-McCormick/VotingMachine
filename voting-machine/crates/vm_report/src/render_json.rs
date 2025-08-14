//! render_json.rs — Part 1/2
//! Report JSON renderer (cover → eligibility → ballot → legitimacy_panel → outcome).
//!
//! IMPORTANT: To keep object key order deterministic (per Doc 7), build this crate
//! with `serde_json`’s `preserve_order` feature enabled. The renderer relies on the
//! *insertion order* of `serde_json::Map<String, Value>`.

use serde_json::{Map as JsonMap, Value};

/// Upstream model (provided by the reporting layer / pipeline).
/// NOTE: We only use read-only fields here; formatting (percents, pp deltas, etc.)
/// must be prepared upstream to avoid renderer-side locale drift.
use crate::report_model::{
    ReportModel,
    CoverBlock, EligibilityBlock, BallotBlock, PanelBlock, OutcomeBlock,
    RunScope, // tagged enum: AllUnits | Selector(String)
};

/// Build the top-level report object in **Doc 7** order.
/// Part 2 will append: frontier, sensitivity, integrity, footer (in that order).
pub fn render_report_json(m: &ReportModel) -> Value {
    let mut root = obj();

    // 1) cover
    root.insert("cover".into(), cover_json(&m.cover));

    // 2) eligibility
    root.insert("eligibility".into(), eligibility_json(&m.eligibility));

    // 3) ballot
    root.insert("ballot".into(), ballot_json(&m.ballot));

    // 4) legitimacy_panel
    root.insert(
        "legitimacy_panel".into(),
        legitimacy_panel_json(&m.panel)
    );

    // 5) outcome
    root.insert("outcome".into(), outcome_json(&m.outcome));

    // Part 2 continues with frontier, sensitivity, integrity, footer
    Value::Object(root)
}

/* ----------------------- sections (Part 1) ----------------------- */

fn cover_json(c: &CoverBlock) -> Value {
    // Order fixed per Doc 7:
    // title → subtitle → provenance → run_scope
    let mut o = obj();

    o.insert("title".into(), Value::String(c.title.clone()));
    o.insert("subtitle".into(), Value::String(c.subtitle.clone()));

    // provenance (optional but often present; omit if empty)
    if let Some(p) = c.provenance.as_ref().filter(|s| !s.is_empty()) {
        o.insert("provenance".into(), Value::String(p.clone()));
    }

    // run_scope (externally tagged: {"type": "...", "value": "..."} only when selector)
    match &c.run_scope {
        RunScope::AllUnits => {
            let mut s = obj();
            s.insert("type".into(), "all_units".into());
            o.insert("run_scope".into(), Value::Object(s));
        }
        RunScope::Selector(sel) => {
            let mut s = obj();
            s.insert("type".into(), "selector".into());
            s.insert("value".into(), Value::String(sel.clone()));
            o.insert("run_scope".into(), Value::Object(s));
        }
    }

    Value::Object(o)
}

fn eligibility_json(e: &EligibilityBlock) -> Value {
    // Order fixed:
    // policy → summary → gates[]
    let mut o = obj();

    // policy (human string prepared upstream)
    o.insert("policy".into(), Value::String(e.policy.clone()));

    // summary (prepared upstream; strings for counts/rates to lock formatting)
    let mut summary = obj();
    summary.insert("eligible_units".into(), Value::String(e.summary.eligible_units.clone()));
    summary.insert("ineligible_units".into(), Value::String(e.summary.ineligible_units.clone()));
    o.insert("summary".into(), Value::Object(summary));

    // gates (ordered array of rows)
    let mut gates = vec![];
    for g in &e.gates {
        gates.push(gate_row(g.label.as_str(), g.value.as_str()));
    }
    o.insert("gates".into(), Value::Array(gates));

    Value::Object(o)
}

fn ballot_json(b: &BallotBlock) -> Value {
    // Order fixed:
    // turnout → approval_rate → approval_denominator_sentence? → corrections?
    let mut o = obj();

    o.insert("turnout".into(), Value::String(b.turnout.clone()));
    o.insert("approval_rate".into(), Value::String(b.approval_rate.clone()));

    // Emit approval_denominator_sentence ONLY when upstream marked it true
    if b.approval_denominator_sentence {
        o.insert(
            "approval_denominator_sentence".into(),
            Value::String("Approval rate is computed over valid ballots only.".into())
        );
    }

    // Optional corrections sub-block
    if let Some(c) = b.corrections.as_ref() {
        let mut corr = obj();
        // Order: duplicates_removed → late_rejected → other
        if let Some(v) = c.duplicates_removed.as_ref() {
            corr.insert("duplicates_removed".into(), Value::String(v.clone()));
        }
        if let Some(v) = c.late_rejected.as_ref() {
            corr.insert("late_rejected".into(), Value::String(v.clone()));
        }
        if let Some(v) = c.other.as_ref() {
            corr.insert("other".into(), Value::String(v.clone()));
        }
        if !corr.is_empty() {
            o.insert("corrections".into(), Value::Object(corr));
        }
    }

    Value::Object(o)
}

fn legitimacy_panel_json(p: &PanelBlock) -> Value {
    // Order fixed:
    // votes_counted → invalid_ballots → adjudications → remarks?
    let mut o = obj();

    o.insert("votes_counted".into(), Value::String(p.votes_counted.clone()));
    o.insert("invalid_ballots".into(), Value::String(p.invalid_ballots.clone()));

    // adjudications as array of rows (label, value)
    let mut adj = vec![];
    for r in &p.adjudications {
        adj.push(gate_row(r.label.as_str(), r.value.as_str()));
    }
    o.insert("adjudications".into(), Value::Array(adj));

    if let Some(r) = p.remarks.as_ref().filter(|s| !s.is_empty()) {
        o.insert("remarks".into(), Value::String(r.clone()));
    }

    Value::Object(o)
}

fn outcome_json(o_: &OutcomeBlock) -> Value {
    // Order fixed:
    // winner → margin → tie_breaker? → disclaimer?
    let mut o = obj();

    o.insert("winner".into(), Value::String(o_.winner.clone()));
    o.insert("margin".into(), Value::String(o_.margin.clone()));

    if let Some(tb) = o_.tie_breaker.as_ref() {
        // tb is preformatted upstream (e.g., "Random draw, seed 123456")
        o.insert("tie_breaker".into(), Value::String(tb.clone()));
    }

    if let Some(d) = o_.disclaimer.as_ref().filter(|s| !s.is_empty()) {
        o.insert("disclaimer".into(), Value::String(d.clone()));
    }

    Value::Object(o)
}

/* ----------------------- helpers ----------------------- */

#[inline]
fn obj() -> JsonMap<String, Value> {
    JsonMap::new()
}

/// Uniform (label, value) row shape as an object.
fn gate_row(label: &str, value: &str) -> Value {
    let mut o = obj();
    o.insert("label".into(), Value::String(label.to_string()));
    o.insert("value".into(), Value::String(value.to_string()));
    Value::Object(o)
}
//! render_json.rs — Part 2/2
//! Report JSON renderer (frontier → sensitivity → integrity → footer) and
//! a full wrapper that renders **all** sections in Doc 7 order.
//!
//! NOTE: Build with `serde_json/preserve_order` to keep insertion order stable.

use serde_json::{Map as JsonMap, Value};

use crate::report_model::{
    ReportModel,
    FrontierBlock, FrontierCounters,
    SensitivityBlock,
    IntegrityBlock,
    FooterBlock,
};

/// Build the **full** report (cover → … → outcome → frontier → sensitivity → integrity → footer).
/// Prefer this function in callers that need the complete JSON.
pub fn render_report_json_full(m: &ReportModel) -> Value {
    // Reuse the Part 1 section builders for the head:
    let mut root = obj();
    root.insert("cover".into(), super::cover_json(&m.cover));
    root.insert("eligibility".into(), super::eligibility_json(&m.eligibility));
    root.insert("ballot".into(), super::ballot_json(&m.ballot));
    root.insert("legitimacy_panel".into(), super::legitimacy_panel_json(&m.panel));
    root.insert("outcome".into(), super::outcome_json(&m.outcome));

    // Tail in canonical order:
    if let Some(fr) = m.frontier.as_ref() {
        root.insert("frontier".into(), frontier_json(fr));
    }
    // Sensitivity is **included even when not executed** (Doc 7); we emit an explanatory string.
    root.insert(
        "sensitivity".into(),
        match m.sensitivity.as_ref() {
            Some(s) => sensitivity_json(s),
            None => Value::String("N/A (not executed)".into()),
        }
    );

    root.insert("integrity".into(), integrity_json(&m.integrity));
    root.insert("footer".into(), footer_json(&m.footer));

    Value::Object(root)
}

/* ------------------------- sections (Part 2) ------------------------- */

fn frontier_json(f: &FrontierBlock) -> Value {
    // Order fixed (Doc 7):
    // enabled → mode → strategy → strictness → band_window → counters
    let mut o = obj();

    o.insert("enabled".into(), Value::Bool(f.enabled));
    o.insert("mode".into(), Value::String(f.mode.clone()));               // e.g., "none" | "basic" | "advanced"
    o.insert("strategy".into(), Value::String(f.strategy.clone()));       // e.g., "apply_on_entry"
    o.insert("strictness".into(), Value::String(f.strictness.clone()));   // "strict" | "lenient"
    o.insert("band_window".into(), Value::String(f.band_window.clone())); // preformatted string (e.g., "0.15")

    // Counters: keep Doc 7 order exact
    let mut c = obj();
    // Doc-ordered keys:
    // changed → no_change → mediation → enclave → protected_blocked → quorum_blocked
    let FrontierCounters {
        changed, no_change, mediation, enclave, protected_blocked, quorum_blocked
    } = &f.counters;
    c.insert("changed".into(), Value::String(changed.clone()));
    c.insert("no_change".into(), Value::String(no_change.clone()));
    c.insert("mediation".into(), Value::String(mediation.clone()));
    c.insert("enclave".into(), Value::String(enclave.clone()));
    c.insert("protected_blocked".into(), Value::String(protected_blocked.clone()));
    c.insert("quorum_blocked".into(), Value::String(quorum_blocked.clone()));

    o.insert("counters".into(), Value::Object(c));

    Value::Object(o)
}

fn sensitivity_json(s: &SensitivityBlock) -> Value {
    // Order fixed:
    // scenarios → winning_stability → margin_stability → notes?
    let mut o = obj();

    // scenarios: array of rows (label, value)
    let mut scenarios = vec![];
    for r in &s.scenarios {
        scenarios.push(gate_row(r.label.as_str(), r.value.as_str()));
    }
    o.insert("scenarios".into(), Value::Array(scenarios));

    o.insert("winning_stability".into(), Value::String(s.winning_stability.clone()));
    o.insert("margin_stability".into(), Value::String(s.margin_stability.clone()));

    if let Some(n) = s.notes.as_ref().filter(|t| !t.is_empty()) {
        o.insert("notes".into(), Value::String(n.clone()));
    }

    Value::Object(o)
}

fn integrity_json(i: &IntegrityBlock) -> Value {
    // Order fixed:
    // tie_policy → rng_seed? → started_utc? → finished_utc? → checks[]
    // Notes:
    // - tie_policy must be the canonical token: "status_quo" | "deterministic_order" | "random"
    // - rng_seed is a NUMBER (u64), included only if present (upstream should enforce echo rule)
    let mut o = obj();

    o.insert("tie_policy".into(), Value::String(i.tie_policy.clone()));

    if let Some(seed) = i.rng_seed {
        o.insert("rng_seed".into(), Value::from(seed as u64));
    }

    // Timestamps: keep both if upstream provides both (RFC3339Z strings).
    if let Some(ts) = i.started_utc.as_ref() {
        o.insert("started_utc".into(), Value::String(ts.clone()));
    }
    if let Some(ts) = i.finished_utc.as_ref() {
        o.insert("finished_utc".into(), Value::String(ts.clone()));
    }

    // Checks: array of rows
    let mut checks = vec![];
    for r in &i.checks {
        checks.push(gate_row(r.label.as_str(), r.value.as_str()));
    }
    o.insert("checks".into(), Value::Array(checks));

    Value::Object(o)
}

fn footer_json(f: &FooterBlock) -> Value {
    // Order fixed:
    // ids { result_id, run_id, frontier_map_id? } → disclaimer?
    let mut o = obj();

    let mut ids = obj();
    ids.insert("result_id".into(), Value::String(f.result_id.to_string())); // expect "RES:<hex64>"
    ids.insert("run_id".into(), Value::String(f.run_id.to_string()));       // expect "RUN:<RFC3339Z>-<hex64>"
    if let Some(fr) = f.frontier_map_id.as_ref() {
        ids.insert("frontier_map_id".into(), Value::String(fr.to_string())); // "FR:<hex64>"
    }
    o.insert("ids".into(), Value::Object(ids));

    if let Some(d) = f.disclaimer.as_ref().filter(|s| !s.is_empty()) {
        o.insert("disclaimer".into(), Value::String(d.clone()));
    }

    Value::Object(o)
}

/* ------------------------- helpers ------------------------- */

#[inline]
fn obj() -> JsonMap<String, Value> {
    JsonMap::new()
}

/// Uniform (label, value) row shape as an object.
fn gate_row(label: &str, value: &str) -> Value {
    let mut o = obj();
    o.insert("label".into(), Value::String(label.to_string()));
    o.insert("value".into(), Value::String(value.to_string()));
    Value::Object(o)
}
