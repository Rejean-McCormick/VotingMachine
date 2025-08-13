````markdown
Pre-Coding Essentials (Component: crates/vm_report/src/structure.rs, Version/FormulaID: VM-ENGINE v0) — 63/89

1) Goal & Success
Goal: Define the **report data model** (pure structs + mappers) that mirrors Doc 7’s section order and precision, sourcing data **only** from `Result`, optional `FrontierMap`, and `RunRecord`.
Success: For identical artifacts, `model_from_artifacts` yields byte-identical, fully-populated `ReportModel`; all percentages are pre-formatted to **one decimal**; approval ballots include the “approval rate = approvals / valid ballots” sentence.

2) Scope
In scope: Section structs, model container, pure mappers from pipeline artifacts, integer-based formatting helpers (1-dp % and signed pp), snapshot extraction of VM-VARs for display.
Out of scope: Rendering (JSON/HTML lives in `lib.rs` renderers), file I/O, schema/validation, any recomputation of gate/frontier math.

3) Inputs → Outputs
Inputs:
- `ResultDb` (RES) — gates, label, aggregates, per-unit data, optional `frontier_map_id`.
- `RunRecordDb` (RUN) — engine/vendor/version/build, FID, seed/policy, timestamps, input IDs.
- `FrontierMapDb` (FR, optional) — statuses, bands, flags/counters.
Outputs:
- `ReportModel` with sections in Doc 7 order, all human-visible numbers already formatted.

4) Entities/Tables (minimal)
No DB writes. This module defines **view** structs only; maps use `BTreeMap` for deterministic iteration; lists preserve artifact order.

5) Variables (display-only)
VM-VARs displayed from the ParameterSet snapshot embedded in artifacts:
001, 010, 012, 020, 021 (+scope), 022/023, 024, 025, 028, 030, 031, 040, 042 (outline), 047, 048, 032 (tie policy), 033 (seed shown only if random).  
No computation here—just mapping/formatting.

6) Functions (signatures only)
```rust
// Public model root
#[derive(Clone, Debug)]
pub struct ReportModel {
    pub cover: CoverSnapshot,
    pub eligibility: EligibilityBlock,
    pub ballot: BallotBlock,
    pub panel: LegitimacyPanel,
    pub outcome: OutcomeBlock,
    pub frontier: Option<FrontierBlock>,
    pub sensitivity: Option<SensitivityBlock>,
    pub integrity: IntegrityBlock,
    pub footer: FooterIds,
}

// Section structs (minimal, renderer-friendly)
#[derive(Clone, Debug)] pub struct CoverSnapshot {
    pub label: String,               // Decisive|Marginal|Invalid
    pub reason: Option<String>,
    pub snapshot_vars: Vec<SnapshotVar>, // key/value VM-VARs for cover box
    pub registry_name: String,
    pub registry_published_date: String,
}
#[derive(Clone, Debug)] pub struct SnapshotVar { pub key: String, pub value: String }

#[derive(Clone, Debug)] pub struct EligibilityBlock {
    pub roll_policy: String,                // pretty VM-VAR-028
    pub totals_eligible_roll: u64,
    pub totals_ballots_cast: u64,
    pub totals_valid_ballots: u64,
    pub per_unit_quorum_note: Option<String>, // VM-VAR-021 + scope
    pub provenance: String,                 // source/edition
}

#[derive(Clone, Debug)] pub struct BallotBlock {
    pub ballot_type: String,                // VM-VAR-001
    pub allocation_method: String,          // VM-VAR-010
    pub weighting_method: String,           // VM-VAR-030
    pub approval_denominator_sentence: bool,
}

#[derive(Clone, Debug)] pub struct GateRow {
    pub value_pct_1dp: String,              // e.g., "55.0%"
    pub threshold_pct_0dp: String,          // e.g., "55%"
    pub pass: bool,
    pub denom_note: Option<String>,         // “approval rate over valid ballots” etc.
    pub members_hint: Option<Vec<String>>,  // double-majority family (ids or names)
}
#[derive(Clone, Debug)] pub struct LegitimacyPanel {
    pub quorum: GateRow,
    pub majority: GateRow,
    pub double_majority: Option<(GateRow, GateRow)>, // (national, family)
    pub symmetry: Option<bool>,
    pub pass: bool,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug)] pub struct OutcomeBlock {
    pub label: String,
    pub reason: String,
    pub national_margin_pp: String,         // signed "±pp"
}

#[derive(Clone, Debug)] pub struct FrontierCounters {
    pub changed: u32, pub no_change: u32, pub mediation: u32,
    pub enclave: u32, pub protected_blocked: u32, pub quorum_blocked: u32,
}
#[derive(Clone, Debug)] pub struct FrontierBlock {
    pub mode: String,                       // VM-VAR-040
    pub edge_types: String,                 // VM-VAR-047 summary
    pub island_rule: String,                // VM-VAR-048
    pub bands_summary: Vec<String>,         // ladder/sliding descriptors
    pub counters: FrontierCounters,
}

#[derive(Clone, Debug)] pub struct SensitivityBlock { pub table_2x3: Vec<Vec<String>> }

#[derive(Clone, Debug)] pub struct IntegrityBlock {
    pub engine_vendor: String, pub engine_name: String,
    pub engine_version: String, pub engine_build: String,
    pub formula_id_hex: String,
    pub tie_policy: String, pub tie_seed: Option<String>,
    pub started_utc: String, pub finished_utc: String,
}
#[derive(Clone, Debug)] pub struct FooterIds {
    pub result_id: vm_core::ids::ResultId,
    pub run_id: vm_core::ids::RunId,
    pub frontier_id: Option<vm_core::ids::FrontierId>,
    pub reg_id: vm_core::ids::RegId,
    pub param_set_id: vm_core::ids::ParamSetId,
    pub tally_id: Option<vm_core::ids::TallyId>,
}

// Top-level mapping API (pure, no I/O)
pub fn model_from_artifacts(
    result: &ResultDb,
    run: &RunRecordDb,
    frontier: Option<&FrontierMapDb>
) -> ReportModel;

// Mapping helpers (pure)
fn map_cover_snapshot(result: &ResultDb) -> CoverSnapshot;
fn map_eligibility(result: &ResultDb) -> EligibilityBlock;
fn map_ballot(result: &ResultDb) -> BallotBlock;                 // sets approval_denominator_sentence
fn map_panel_from_gates(result: &ResultDb) -> LegitimacyPanel;   // uses precomputed gate ratios
fn map_outcome_from_result(result: &ResultDb) -> OutcomeBlock;
fn map_frontier(fr: &FrontierMapDb, result: &ResultDb) -> FrontierBlock;
fn map_sensitivity(_result: &ResultDb) -> Option<SensitivityBlock>; // N/A by default in v1
fn map_integrity_footer(run: &RunRecordDb, result: &ResultDb, frontier: Option<&FrontierMapDb>)
    -> (IntegrityBlock, FooterIds);

// Formatting helpers (integer math only)
fn pct_1dp(num: i128, den: i128) -> String;       // uses vm_core::rounding::percent_one_decimal_tenths
fn pct0(value_u8: u8) -> String;                  // "55%"
fn pp_signed(pp_i32: i32) -> String;              // "+3 pp" / "−2 pp"
````

7. Algorithm Outline (mapping rules)

* **Cover/Snapshot:** label + reason from `Result.label`; build snapshot variables from Params snapshot (ballot/allocation/weighting/thresholds/frontier switches). Include registry name/date from provenance.
* **Eligibility:** echo VM-VAR-028 (pretty text), totals (Σ eligible\_roll, ballots\_cast, valid\_ballots), per-unit quorum note if VM-VAR-021 > 0 (include scope wording).
* **Ballot:** set `approval_denominator_sentence=true` iff VM-VAR-001 = `approval`; else false. Copy allocation/weighting strings.
* **Panel:** bind gate rows strictly from `Result.gates` (no recompute). For approval, set `denom_note = "approval rate = approvals / valid ballots"`. Double-majority shows (national,family). Symmetry boolean from gates.
* **Outcome:** copy label/reason; render `national_margin_pp` via `pp_signed`.
* **Frontier (optional):** summarize VM-VAR-040/047/048; compute counters from FR flags; list band labels/ids in declared order.
* **Sensitivity (optional):** only if scenarios exist (out of scope in v1) else `None`.
* **Integrity/Footer:** copy engine/vendor/version/build, FID, tie policy/seed (only if random), timestamps; set IDs (RES/RUN/FR; REG/PS/TLY echoes from artifacts).

8. State Flow (very short)
   Called by `vm_report::build_model` after pipeline packaging. No network/FS. Renderers consume this `ReportModel` verbatim.

9. Determinism & Numeric Rules

* One-decimal **applied here** using integer helpers; renderers must not re-round.
* BTreeMap for any keyed collections; lists preserve artifact order.
* No floats, no time reads; UTC/timestamps come from `RunRecord`.

10. Edge Cases & Failure Policy

* Gates/validation failed ⇒ label already “Invalid”; frontier omitted; panel shows failures verbatim.
* Unknown roll policy value ⇒ render raw string (no panic) with no extra logic.
* Missing optional FR ⇒ `frontier=None` but the rest of the model is complete.

11. Test Checklist (must pass)

* Section order matches Doc 7; snapshot lists expected VM-VARs.
* 1-dp formatting: (1,3) → “33.3%”; thresholds as “55%”.
* Approval ballots set `approval_denominator_sentence=true`.
* Determinism: same artifacts → identical `ReportModel` JSON (when serialized).
* Integrity/footer IDs match `RunRecord` & `Result`; tie seed only when policy=`random`.

```
```
