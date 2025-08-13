````markdown
Pre-Coding Essentials (Component: crates/vm_report/src/lib.rs, Version/FormulaID: VM-ENGINE v0) — 61/89

1) Goal & Success
Goal: Provide a **pure, offline** reporting API that consumes only pipeline artifacts — `Result`, optional `FrontierMap`, and `RunRecord` — and produces a deterministic `ReportModel` plus JSON/HTML renderings that follow **Doc 7**: fixed section order, one-decimal percentages, and exact provenance echo (tie policy/seed, IDs).
Success: Given identical inputs, `build_model` + `render_*` yield byte-identical outputs across OS/arch; approval ballot paragraph includes the “approval rate = approvals / valid ballots” sentence; no network or extra data sources.

2) Scope
In scope: Model construction from artifacts; deterministic formatting helpers; section assembly (§1–§10 per Doc 7); feature-gated JSON and HTML renderers using embedded templates.
Out of scope: File I/O, template fetching, pipeline math, schema validation (already done upstream).

3) Inputs → Outputs (with artifacts)
Inputs:
- `Result` (RES:…) — tabulation/allocations, aggregates, gates, label, tie log, optional `frontier_map_id`.
- `RunRecord` (RUN:…) — engine identifiers (vendor/name/version/build), FormulaID (FID), determinism/rng seed (if used), timestamps, input IDs.
- `FrontierMap` (FR:…) — optional, for frontier section & flags if present.
Outputs:
- `ReportModel` (in-memory view mirroring Doc 7 sections).
- Renderer outputs: JSON string (`render_json`), HTML string (`render_html`) — both **strictly** derived from `ReportModel`.

4) Entities/Tables (minimal)
(Engine artifacts come from vm_pipeline/vm_io types; this crate defines its own `ReportModel`/section structs only.)

5) Variables (rendered, not recomputed)
Echoed from artifacts / Params snapshot:
- VM-VAR-001 (ballot_type), 010 (allocation_method), 012 (pr_entry_threshold_pct),
  020 (quorum_global_pct), 021 (+scope), 022/023 (majority cutoffs),
  024 (double_majority_enabled), 025 (symmetry_enabled),
  028 (roll_inclusion_policy), 030 (weighting_method), 031 (aggregate_level=country),
  040 (frontier_mode), 042 (frontier_bands outline), 047 (contiguity edges), 048 (island rule),
  032 (tie_policy), 033 (tie_seed — shown only if policy=random).

6) Functions (signatures only; no I/O)
```rust
// Re-exports (types used by signatures; concrete names resolved in vm_io/vm_pipeline)
pub use vm_core::rounding::percent_one_decimal_tenths;
pub use vm_core::ids::{ResultId, RunId, FrontierId};

// Public error type
#[derive(Debug)]
pub enum ReportError {
    Template(&'static str),
    MissingField(&'static str),
    Inconsistent(&'static str),
}

// ===== Model =====
#[derive(Clone, Debug)]
pub struct ReportModel {
    pub cover: SectionCover,
    pub snapshot: SectionSnapshot,
    pub eligibility: SectionEligibility,
    pub ballot_method: SectionBallotMethod,
    pub legitimacy_panel: SectionLegitimacy,
    pub outcome_label: SectionOutcome,
    pub frontier: Option<SectionFrontier>,
    pub sensitivity: Option<SectionSensitivity>, // optional
    pub integrity: SectionIntegrity,
}

// --- Sections (minimal fields; extend as Doc 7 prescribes) ---
#[derive(Clone, Debug)]
pub struct SectionCover { pub title: String, pub label: String /* Decisive|Marginal|Invalid */, pub reason: Option<String> }

#[derive(Clone, Debug)]
pub struct SnapshotVar { pub key: String, pub value: String }
#[derive(Clone, Debug)]
pub struct SectionSnapshot { pub items: Vec<SnapshotVar> }

#[derive(Clone, Debug)]
pub struct SectionEligibility {
    pub roll_policy: String,           // VM-VAR-028 pretty label
    pub registry_source: String,       // provenance text/date
    pub totals: EligibilityTotals,     // Σ eligible_roll, Σ ballots_cast, Σ valid_ballots
    pub per_unit_quorum_note: Option<String>, // mentions 021 + scope
}
#[derive(Clone, Debug)]
pub struct EligibilityTotals { pub eligible_roll: u64, pub ballots_cast: u64, pub valid_ballots: u64 }

#[derive(Clone, Debug)]
pub struct SectionBallotMethod {
    pub method: String,                // VM-VAR-001
    pub allocation: String,            // VM-VAR-010
    pub weighting: String,             // VM-VAR-030
    pub approval_denominator_sentence: Option<String>, // forced for approval ballots
}

#[derive(Clone, Debug)]
pub struct GateRow { pub name: String, pub value_pct_1dp: String, pub threshold_pct_0dp: String, pub pass: bool, pub denom_note: Option<String> }
#[derive(Clone, Debug)]
pub struct SectionLegitimacy {
    pub quorum: GateRow,
    pub majority: GateRow,
    pub double_majority: Option<(GateRow, GateRow)>, // (national, family)
    pub symmetry: Option<bool>,
    pub pass: bool,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SectionOutcome { pub label: String, pub reason: String, pub national_margin_pp: String }

#[derive(Clone, Debug)]
pub struct FrontierCounters { pub changed: u32, pub no_change: u32, pub mediation: u32, pub enclave: u32, pub protected_blocked: u32, pub quorum_blocked: u32 }
#[derive(Clone, Debug)]
pub struct SectionFrontier {
    pub mode: String,                      // VM-VAR-040
    pub edge_policy: String,               // VM-VAR-047
    pub island_rule: String,               // VM-VAR-048
    pub counters: FrontierCounters,
    pub bands_summary: Vec<String>,        // labels or band ids (ladder)
}

#[derive(Clone, Debug)]
pub struct SectionSensitivity { pub table: Vec<Vec<String>> } // optional CompareScenarios

#[derive(Clone, Debug)]
pub struct SectionIntegrity {
    pub result_id: ResultId,
    pub run_id: RunId,
    pub frontier_id: Option<FrontierId>,
    pub engine_vendor: String,
    pub engine_name: String,
    pub engine_version: String,
    pub engine_build: String,
    pub formula_id_hex: String,
    pub tie_policy: String,
    pub tie_seed: Option<String>, // only if tie_policy == "random"
    pub started_utc: String,
    pub finished_utc: String,
}

// ===== API =====

// Construct the model from artifacts (no network, no I/O).
pub fn build_model(
    result: &ResultArtifact,             // concrete type alias in this crate
    run: &RunRecordArtifact,
    frontier: Option<&FrontierMapArtifact>,
    compare: Option<&CompareScenariosArtifact>,
) -> Result<ReportModel, ReportError>;

// Feature-gated renderers; inputs are pure model.
#[cfg(feature = "render_json")]
pub fn render_json(model: &ReportModel) -> Result<String, ReportError>;

#[cfg(feature = "render_html")]
pub fn render_html(model: &ReportModel) -> Result<String, ReportError>;
````

7. Algorithm Outline (implementation plan)

* **Model assembly (`build_model`)**

  * Read label & reason from `Result` → `SectionOutcome`.
  * Snapshot: list `VM-VAR` values (ballot, allocation, weighting, thresholds, double-majority, symmetry, frontier mode). All pulled from the Params snapshot echoed in `Result/RunRecord`.
  * Eligibility: format roll policy (VM-VAR-028), registry provenance, totals Σ(eligible\_roll, ballots\_cast, valid\_ballots); add per-unit quorum note if VM-VAR-021 > 0 (include scope text).
  * Ballot method: string per ballot type; **always** add approval denominator sentence when VM-VAR-001 = `approval`.
  * Legitimacy panel: for quorum/majority/double-majority, copy raw ratios/thresholds from `Result`; convert ratios to **one-decimal** strings via helpers; mark Pass/Fail; include denominator note where relevant.
  * Frontier: only if `frontier` provided. Fill mode/edge/island strings from VM-VARs; compute counters from map flags; list bands (ladder/sliding) labels as given.
  * Sensitivity: if `compare` present, build fixed 2×3 (or spec’d) table; else `None`.
  * Integrity: copy `engine.vendor/name/version/build`, formula ID hex, IDs (RES/RUN/FR), timestamps, and tie policy/seed from `RunRecord` (seed only when policy=random).

* **Formatting helpers (no floats)**

  * Convert `Ratio {num,den}` to tenths of a percent using `vm_core::rounding::percent_one_decimal_tenths(num,den)`; then format as `"55.0%"` (0..=1000 → `0.0..=100.0`).
  * Format integer percentage thresholds as `"55%"` (no decimals).
  * Present national margin in **percentage points** as signed integer string with “pp”.

* **Renderers**

  * JSON: serialize `ReportModel` using serde; ensure field order stability (derive from struct layout or BTree in nested maps).
  * HTML: render via embedded templates (minijinja); **no external assets**. All templates included at compile-time (e.g., `include_dir!`).

8. State Flow
   `build_model` ← artifacts produced by pipeline → (optional) `render_json` / `render_html`. No disk/network at this layer; caller handles file writing.

9. Determinism & Numeric Rules

* No floats; all percentages derived via integer helpers; **one-decimal** only at presentation.
* Stable section order & deterministic template content; same inputs ⇒ identical bytes.
* UTC timestamps are displayed verbatim from `RunRecord`.

10. Edge Cases & Failure Policy

* Missing optional `FrontierMap` ⇒ frontier section omitted (model still valid).
* Unknown `roll_inclusion_policy` value: render verbatim with a neutral note (no panic).
* If any required artifact field is absent: return `ReportError::MissingField`.
* HTML renderer: if a template key is missing, return `ReportError::Template("…")`.

11. Test Checklist (must pass)

* **Determinism:** two runs with identical artifacts produce identical JSON/HTML.
* **One-decimal:** known ratios (1/3 → 33.3%) render as expected; no double-rounding.
* **Approval paragraph:** appears exactly for approval ballots, absent otherwise.
* **Frontier presence:** only when FR provided; counters/flags match the map.
* **Provenance echo:** engine vendor/name/version/build + FID + IDs + tie policy/seed (when random) appear in Integrity section.
* **No floats:** `cargo clippy` denies `float_arithmetic`; tests ensure formatting uses integer helpers only.

```

**Notes to implementers**
- Define local type aliases for `ResultArtifact`, `RunRecordArtifact`, and `FrontierMapArtifact` to the concrete vm_io/vm_pipeline structs you expose in this workspace.
- Gate serde derives for `ReportModel` and sections behind `render_json` to keep the core lean when only HTML is used.
- Keep templates bilingual only if your workspace ships both sets; never fetch fonts/JS/CSS at runtime.
```
