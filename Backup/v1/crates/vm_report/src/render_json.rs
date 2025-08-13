//! crates/vm_report/src/render_json.rs
//! Deterministic JSON renderer for ReportModel (Doc 7 fixed order). No I/O, no math, no hashing.

use serde_json::{Map, Value};

use crate::structure::{
    FrontierBlock, FrontierCounters, GateRow, ReportModel, SensitivityBlock,
};

/// Public API: return a compact UTF-8 JSON string (no trailing newline).
pub fn render_json(model: &ReportModel) -> String {
    let v = to_ordered_json(model);
    // serde_json::to_string is deterministic for a given Value and Map insertion order.
    serde_json::to_string(&v).expect("render_json: serialization must not fail")
}

/// Internal: build a Value tree with **stable insertion order** for all objects.
fn to_ordered_json(m: &ReportModel) -> Value {
    let mut root = Map::new();

    // Insert sections strictly in Doc 7 order.
    root.insert("cover".into(), cover_json(m));
    root.insert("eligibility".into(), eligibility_json(m));
    root.insert("ballot".into(), ballot_json(m));
    root.insert("legitimacy_panel".into(), panel_json(m));
    root.insert("outcome".into(), outcome_json(m));

    if let Some(fr) = frontier_json(m) {
        root.insert("frontier".into(), fr);
    }

    if let Some(sens) = sensitivity_json(m) {
        root.insert("sensitivity".into(), sens);
    }

    root.insert("integrity".into(), integrity_json(m));
    root.insert("footer".into(), footer_json(m));

    Value::Object(root)
}

// ---------------- Section builders (fixed key ordering in each object) ----------------

fn cover_json(m: &ReportModel) -> Value {
    let mut obj = Map::new();
    obj.insert("label".into(), Value::String(m.cover.label.clone()));
    if let Some(r) = &m.cover.reason {
        obj.insert("reason".into(), Value::String(r.clone()));
    }

    // snapshot_vars: preserve model order
    let vars = m
        .cover
        .snapshot_vars
        .iter()
        .map(|kv| {
            let mut kvobj = Map::new();
            kvobj.insert("key".into(), Value::String(kv.key.clone()));
            kvobj.insert("value".into(), Value::String(kv.value.clone()));
            Value::Object(kvobj)
        })
        .collect::<Vec<_>>();
    obj.insert("snapshot_vars".into(), Value::Array(vars));

    obj.insert(
        "registry_name".into(),
        Value::String(m.cover.registry_name.clone()),
    );
    obj.insert(
        "registry_published_date".into(),
        Value::String(m.cover.registry_published_date.clone()),
    );
    Value::Object(obj)
}

fn eligibility_json(m: &ReportModel) -> Value {
    let mut obj = Map::new();
    obj.insert(
        "roll_policy".into(),
        Value::String(m.eligibility.roll_policy.clone()),
    );
    obj.insert(
        "totals_eligible_roll".into(),
        Value::from(m.eligibility.totals_eligible_roll),
    );
    obj.insert(
        "totals_ballots_cast".into(),
        Value::from(m.eligibility.totals_ballots_cast),
    );
    obj.insert(
        "totals_valid_ballots".into(),
        Value::from(m.eligibility.totals_valid_ballots),
    );
    if let Some(note) = &m.eligibility.per_unit_quorum_note {
        obj.insert("per_unit_quorum_note".into(), Value::String(note.clone()));
    }
    obj.insert(
        "provenance".into(),
        Value::String(m.eligibility.provenance.clone()),
    );
    Value::Object(obj)
}

fn ballot_json(m: &ReportModel) -> Value {
    let mut obj = Map::new();
    obj.insert(
        "ballot_type".into(),
        Value::String(m.ballot.ballot_type.clone()),
    );
    obj.insert(
        "allocation_method".into(),
        Value::String(m.ballot.allocation_method.clone()),
    );
    obj.insert(
        "weighting_method".into(),
        Value::String(m.ballot.weighting_method.clone()),
    );
    // Emit the fixed approval denominator sentence flag only when true.
    if m.ballot.approval_denominator_sentence {
        obj.insert(
            "approval_denominator_sentence".into(),
            Value::Bool(true),
        );
    }
    Value::Object(obj)
}

fn panel_json(m: &ReportModel) -> Value {
    let mut obj = Map::new();
    obj.insert("quorum".into(), gate_row(&m.panel.quorum));
    obj.insert("majority".into(), gate_row(&m.panel.majority));

    if let Some((nat, fam)) = &m.panel.double_majority {
        let mut dm = Map::new();
        dm.insert("national".into(), gate_row(nat));
        dm.insert("family".into(), gate_row(fam));
        obj.insert("double_majority".into(), Value::Object(dm));
    }

    if let Some(sym) = m.panel.symmetry {
        obj.insert("symmetry".into(), Value::Bool(sym));
    }

    obj.insert("pass".into(), Value::Bool(m.panel.pass));

    // Reasons are already ordered by the builder; emit as-is.
    let reasons = m
        .panel
        .reasons
        .iter()
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();
    obj.insert("reasons".into(), Value::Array(reasons));

    Value::Object(obj)
}

fn gate_row(g: &GateRow) -> Value {
    let mut obj = Map::new();
    obj.insert(
        "value_pct_1dp".into(),
        Value::String(g.value_pct_1dp.clone()),
    );
    obj.insert(
        "threshold_pct_0dp".into(),
        Value::String(g.threshold_pct_0dp.clone()),
    );
    obj.insert("pass".into(), Value::Bool(g.pass));
    if let Some(dn) = &g.denom_note {
        obj.insert("denom_note".into(), Value::String(dn.clone()));
    }
    if let Some(hint) = &g.members_hint {
        let arr = hint.iter().cloned().map(Value::String).collect::<Vec<_>>();
        obj.insert("members_hint".into(), Value::Array(arr));
    }
    Value::Object(obj)
}

fn outcome_json(m: &ReportModel) -> Value {
    let mut obj = Map::new();
    obj.insert("label".into(), Value::String(m.outcome.label.clone()));
    obj.insert("reason".into(), Value::String(m.outcome.reason.clone()));
    obj.insert(
        "national_margin_pp".into(),
        Value::String(m.outcome.national_margin_pp.clone()),
    );
    Value::Object(obj)
}

fn frontier_json(m: &ReportModel) -> Option<Value> {
    let fr: &FrontierBlock = m.frontier.as_ref()?;
    let mut obj = Map::new();
    obj.insert("mode".into(), Value::String(fr.mode.clone()));
    obj.insert("edge_types".into(), Value::String(fr.edge_types.clone()));
    obj.insert("island_rule".into(), Value::String(fr.island_rule.clone()));

    // bands_summary in declared order
    let bands = fr
        .bands_summary
        .iter()
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();
    obj.insert("bands_summary".into(), Value::Array(bands));

    // counters in a fixed key order
    let c: &FrontierCounters = &fr.counters;
    let mut counters = Map::new();
    counters.insert("changed".into(), Value::from(c.changed));
    counters.insert("no_change".into(), Value::from(c.no_change));
    counters.insert("mediation".into(), Value::from(c.mediation));
    counters.insert("enclave".into(), Value::from(c.enclave));
    counters.insert(
        "protected_blocked".into(),
        Value::from(c.protected_blocked),
    );
    counters.insert("quorum_blocked".into(), Value::from(c.quorum_blocked));
    obj.insert("counters".into(), Value::Object(counters));

    Some(Value::Object(obj))
}

/// Policy: include `"sensitivity": "N/A (not executed)"` when absent.
fn sensitivity_json(m: &ReportModel) -> Option<Value> {
    match &m.sensitivity {
        Some(SensitivityBlock { table_2x3 }) => {
            let rows = table_2x3
                .iter()
                .map(|row| Value::Array(row.iter().cloned().map(Value::String).collect()))
                .collect::<Vec<_>>();
            Some(Value::Object(
                [("table_2x3".into(), Value::Array(rows))].into_iter().collect(),
            ))
        }
        None => Some(Value::String("N/A (not executed)".into())),
    }
}

fn integrity_json(m: &ReportModel) -> Value {
    let mut obj = Map::new();
    obj.insert(
        "engine_vendor".into(),
        Value::String(m.integrity.engine_vendor.clone()),
    );
    obj.insert(
        "engine_name".into(),
        Value::String(m.integrity.engine_name.clone()),
    );
    obj.insert(
        "engine_version".into(),
        Value::String(m.integrity.engine_version.clone()),
    );
    obj.insert(
        "engine_build".into(),
        Value::String(m.integrity.engine_build.clone()),
    );
    obj.insert(
        "formula_id_hex".into(),
        Value::String(m.integrity.formula_id_hex.clone()),
    );
    obj.insert(
        "tie_policy".into(),
        Value::String(m.integrity.tie_policy.clone()),
    );
    if let Some(seed) = &m.integrity.tie_seed {
        obj.insert("tie_seed".into(), Value::String(seed.clone()));
    }
    obj.insert(
        "started_utc".into(),
        Value::String(m.integrity.started_utc.clone()),
    );
    obj.insert(
        "finished_utc".into(),
        Value::String(m.integrity.finished_utc.clone()),
    );
    Value::Object(obj)
}

fn footer_json(m: &ReportModel) -> Value {
    let mut obj = Map::new();
    // Strong types display as canonical strings via Display
    obj.insert("result_id".into(), Value::String(m.footer.result_id.to_string()));
    obj.insert("run_id".into(), Value::String(m.footer.run_id.to_string()));
    if let Some(fr) = &m.footer.frontier_id {
        obj.insert("frontier_id".into(), Value::String(fr.to_string()));
    }
    obj.insert("reg_id".into(), Value::String(m.footer.reg_id.clone()));
    obj.insert(
        "param_set_id".into(),
        Value::String(m.footer.param_set_id.clone()),
    );
    if let Some(tly) = &m.footer.tally_id {
        obj.insert("tally_id".into(), Value::String(tly.clone()));
    }
    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::*;

    fn demo_model() -> ReportModel {
        ReportModel {
            cover: CoverSnapshot {
                label: "Decisive".into(),
                reason: Some("All gates passed".into()),
                snapshot_vars: vec![SnapshotVar{ key:"VAR-001".into(), value:"plurality".into() }],
                registry_name: "Demo Registry".into(),
                registry_published_date: "2025-01-01".into(),
            },
            eligibility: EligibilityBlock {
                roll_policy: "Resident roll".into(),
                totals_eligible_roll: 1000,
                totals_ballots_cast: 700,
                totals_valid_ballots: 680,
                per_unit_quorum_note: None,
                provenance: "demo v1".into(),
            },
            ballot: BallotBlock {
                ballot_type: "approval".into(),
                allocation_method: "wta".into(),
                weighting_method: "none".into(),
                approval_denominator_sentence: true,
            },
            panel: LegitimacyPanel {
                quorum: GateRow {
                    value_pct_1dp: "70.0%".into(),
                    threshold_pct_0dp: "50%".into(),
                    pass: true,
                    denom_note: None,
                    members_hint: None,
                },
                majority: GateRow {
                    value_pct_1dp: "55.0%".into(),
                    threshold_pct_0dp: "55%".into(),
                    pass: true,
                    denom_note: Some("approval rate = approvals / valid ballots".into()),
                    members_hint: None,
                },
                double_majority: None,
                symmetry: None,
                pass: true,
                reasons: vec![],
            },
            outcome: OutcomeBlock {
                label: "Decisive".into(),
                reason: "All gates passed".into(),
                national_margin_pp: "+10 pp".into(),
            },
            frontier: None,
            sensitivity: None,
            integrity: IntegrityBlock {
                engine_vendor: "KOA".into(),
                engine_name: "VM".into(),
                engine_version: "0.1.0".into(),
                engine_build: "abc123".into(),
                formula_id_hex: "0".repeat(64),
                tie_policy: "deterministic".into(),
                tie_seed: None,
                started_utc: "2025-08-12T14:00:00Z".into(),
                finished_utc: "2025-08-12T14:00:05Z".into(),
            },
            footer: FooterIds {
                result_id: "RES:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd".parse().unwrap(),
                run_id: "RUN:2025-08-12T14:00:00Z-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd".parse().unwrap(),
                frontier_id: None,
                reg_id: "REG:demo".into(),
                param_set_id: "PS:demo".into(),
                tally_id: None,
            },
        }
    }

    #[test]
    fn deterministic_and_in_order() {
        let m = demo_model();
        let s1 = render_json(&m);
        let s2 = render_json(&m);
        assert_eq!(s1, s2);

        // Top-level order sanity (cover → eligibility → ballot → legitimacy_panel → outcome → integrity → footer)
        let cover_pos = s1.find("\"cover\"").unwrap();
        let elig_pos = s1.find("\"eligibility\"").unwrap();
        let ballot_pos = s1.find("\"ballot\"").unwrap();
        let panel_pos = s1.find("\"legitimacy_panel\"").unwrap();
        let outcome_pos = s1.find("\"outcome\"").unwrap();
        let integ_pos = s1.find("\"integrity\"").unwrap();
        let footer_pos = s1.find("\"footer\"").unwrap();
        assert!(cover_pos < elig_pos && elig_pos < ballot_pos && ballot_pos < panel_pos);
        assert!(panel_pos < outcome_pos && outcome_pos < integ_pos && integ_pos < footer_pos);

        // Approval sentence present
        assert!(s1.contains("\"approval_denominator_sentence\":true"));
    }
}
