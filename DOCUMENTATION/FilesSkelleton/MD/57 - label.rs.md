```
Pre-Coding Essentials (Component: label.rs, Version/FormulaID: VM-ENGINE v0) — 57/89

1) Goal & Success
Goal: Compute the final decisiveness label — Decisive | Marginal | Invalid — and a concise, machine-readable reason using gate outcomes, national margin vs threshold, and frontier risk flags.
Success: For identical inputs (including VM-VARs), the same label/reason is produced across OS/arch. Rules match Doc 4C and align with Result/RunRecord contracts (Result carries label+reason; tie events live in RunRecord only).

2) Scope
In scope: Pure, deterministic decision function; small helpers to extract the first failing gate; frontier-risk aggregation; thin config wrapper for the decisive-margin threshold (VM-VAR-062).
Out of scope: Gate calculations (apply_rules) and frontier mapping (map_frontier). No I/O, no RNG, no formatting beyond a short reason string.

3) Inputs → Outputs (artifacts)
Inputs (ephemeral):
• LegitimacyReport (from APPLY_DECISION_RULES)
• national_margin_pp: i32 (national support minus required threshold, in percentage points; integer)
• FrontierFlags (optional): { mediation_flagged, enclave, protected_override_used } aggregated at run level
• (Optionally) Params or a small LabelConfig carrying VM-VAR-062

Outputs (ephemeral → consumed by BUILD_RESULT):
• DecisivenessLabel { label: Label, reason: SmolStr/String }

4) Entities/Types (minimal)
use smol_str::SmolStr; // or String if smol_str not used elsewhere

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Label { Decisive, Marginal, Invalid }

#[derive(Clone, Debug)]
pub struct DecisivenessLabel {
  pub label: Label,
  pub reason: SmolStr,   // short, machine-readable; report can rephrase for humans
}

// From gates/frontier stages (light mirrors)
pub struct LegitimacyReport {
  pub pass: bool,
  pub reasons: Vec<String>, // first item explains failure if pass=false
  // … other gate fields not needed here
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct FrontierFlags {
  pub mediation_flagged: bool,
  pub enclave: bool,
  pub protected_override_used: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct LabelConfig {
  /// VM-VAR-062: minimum national margin (pp) required for "Decisive".
  pub decisive_margin_pp: i32,
}

5) Variables (VM-VARs referenced)
• VM-VAR-062 decisive_margin_pp (integer pp)
(Any other VM-VAR influences are already reflected in LegitimacyReport or FrontierFlags.)

6) Functions (signatures only)
/// Main entry (explicit threshold provided via LabelConfig).
pub fn label_decisiveness_cfg(
  legit: &LegitimacyReport,
  national_margin_pp: i32,
  frontier_flags: Option<&FrontierFlags>,
  cfg: LabelConfig,
) -> DecisivenessLabel;

/// Convenience entry that reads VM-VAR-062 from Params.
pub fn label_decisiveness(
  legit: &LegitimacyReport,
  national_margin_pp: i32,
  frontier_flags: Option<&FrontierFlags>,
  params: &vm_core::variables::Params,
) -> DecisivenessLabel;

/// Internal helpers (pure)
fn first_failure_reason(legit: &LegitimacyReport) -> SmolStr;
fn has_frontier_risk(ff: Option<&FrontierFlags>) -> bool;

7) Algorithm Outline (deterministic)
1) If !legit.pass:
   • reason = first_failure_reason(legit) (fallback "gates_failed" if empty)
   • return { label: Invalid, reason }

2) Compute frontier_risk = has_frontier_risk(frontier_flags):
   • true if any of { mediation_flagged, enclave, protected_override_used } is true.

3) Read decisive_margin_pp (via cfg or params):
   • If national_margin_pp < decisive_margin_pp → return { Marginal, "margin_below_threshold" }.
   • Else if frontier_risk → return { Marginal, "frontier_risk_flags_present" }.
   • Else → return { Decisive, "margin_meets_threshold" }.

Notes:
• Exact threshold: national_margin_pp == decisive_margin_pp ⇒ Decisive.
• Reasons are short, stable tokens for reporting; keep ASCII and snake_case.

8) State Flow
… → APPLY_DECISION_RULES → (optional) MAP_FRONTIER → (optional) RESOLVE_TIES → **LABEL_DECISIVENESS** → BUILD_RESULT → BUILD_RUN_RECORD.
Result will include { label, label_reason }. Tie logs are excluded from Result and recorded in RunRecord per Doc 1B/18–19.

9) Determinism & Numeric Rules
• Integer math only (pp as i32).
• No rounding here (margin already computed upstream).
• No RNG; no iteration over unordered maps.

10) Edge Cases & Failure Policy
• legit.pass=false with empty reasons → use "gates_failed".
• frontier_flags=None treated as no risk flags (equivalent to all false).
• Negative national_margin_pp is valid — handled by comparison rule.
• Do not panic on empty inputs; return sensible defaults.

11) Test Checklist (must pass)
• Gates fail ⇒ Invalid/"gates_failed" (or first legit.reasons entry).
• Margin below threshold (e.g., margin=4, threshold=5) & gates pass ⇒ Marginal/"margin_below_threshold".
• Margin equal to threshold (5 vs 5), no frontier flags ⇒ Decisive/"margin_meets_threshold".
• Any frontier flag true with sufficient margin ⇒ Marginal/"frontier_risk_flags_present".
• Determinism: same inputs produce identical outputs across repeated runs/OS.
• Result wiring: BUILD_RESULT copies label & reason into Result fields; RunRecord remains the place for tie logs and engine metadata.

Doc/Schema alignment notes
• Matches Result schema adjustments: Result carries `label` and `label_reason`; no tie_log in Result; shares elsewhere are numbers (not rationals); formula_id is attached at Result root (from normative manifest hash).
• RunRecord alignment: tie events live in RunRecord.ties[]; rng_seed included there only if policy=random.
```
