```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/resolve_ties.rs, Version/FormulaID: VM-ENGINE v0) — 56/89

1) Goal & Success
Goal: Resolve only blocking ties (WTA winner, last seat in PR, IRV elimination) using the configured policy, record deterministic audit entries, and return winners to unblock the pipeline.
Success: Same inputs (+ same 64-hex seed when policy = random) ⇒ identical winners and TieLog bytes across OS/arch. Tie events are recorded in **RunRecord.ties[]** (not Result), with optional summary counts.

2) Scope
In scope: Policy application (status_quo → deterministic → random), deterministic ordering by (order_index, OptionId), seeded RNG path via ChaCha20, tie logging payload for RunRecord.ties[].
Out of scope: Building quotients/tallies, gates/frontier, labeling, I/O. No schema writes here.

3) Inputs → Outputs
Inputs:
• contexts: Vec<TieContext>  // produced by tabulation/allocation stages when a decision is blocked
• options_index: BTreeMap<OptionId, OptionItem> // provides order_index + is_status_quo
• policy: TiePolicy                 // from Params VM-VAR-032/033 (seed is 64-hex when random)

Outputs:
• (ties: Vec<TieLogEntry>, winners: BTreeMap<(TieKind, UnitId), OptionId>)
  - `ties` is embedded into RunRecord.ties[] later; `winners` are fed back to finalize blocked decisions.

4) Entities (minimal)
use std::collections::{BTreeMap, BTreeSet};
use vm_core::{
  ids::{UnitId, OptionId},
  entities::OptionItem,
  rng::{TieRng, tie_rng_from_seed},   // seed = 64-hex
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum TieKind { WtaWinner, LastSeat, IrvElimination }

#[derive(Clone, Debug)]
pub struct TieContext {
  pub kind: TieKind,
  pub unit: UnitId,
  pub candidates: Vec<OptionId>,     // unique; upstream ensures only valid options
}

#[derive(Clone, Debug)]
pub enum TiePolicy {
  StatusQuo,
  Deterministic,                     // by (order_index, OptionId)
  Random { seed_hex64: String },     // VM-VAR-033
}

#[derive(Clone, Debug)]
pub struct TieLogEntry {
  pub context: String,               // e.g., "WTA U:REG:.."
  pub candidates: Vec<OptionId>,     // canonical order snapshot
  pub policy: &'static str,          // "status_quo" | "deterministic" | "random"
  pub detail: Option<TieDetail>,     // deterministic ordering or RNG crumb
  pub winner: OptionId,
}

#[derive(Clone, Debug)]
pub enum TieDetail {
  OrderIndex,                        // deterministic rule used
  Rng { seed_hex64: String, word_index: u128 }, // first RNG word used for this pick
}

5) Functions (signatures only)
pub fn resolve_ties(
  contexts: &[TieContext],
  options_index: &BTreeMap<OptionId, OptionItem>,
  policy: &TiePolicy,
) -> (Vec<TieLogEntry>, BTreeMap<(TieKind, UnitId), OptionId>);

// internals (pure/deterministic)
fn pick_status_quo(
  cands: &[OptionId],
  options_index: &BTreeMap<OptionId, OptionItem>,
) -> Option<OptionId>;

fn pick_by_order_index(
  cands: &[OptionId],
  options_index: &BTreeMap<OptionId, OptionItem>,
) -> OptionId; // min by (order_index, OptionId)

fn pick_by_rng(
  cands: &[OptionId],
  seed_hex64: &str,
) -> Result<(OptionId, u128), ResolveError>; // winner + RNG word index for audit

fn canonicalize_candidates(
  cands: &[OptionId],
  options_index: &BTreeMap<OptionId, OptionItem>,
) -> Vec<OptionId>; // sort by (order_index, OptionId) for stable logs

#[derive(thiserror::Error, Debug)]
pub enum ResolveError {
  #[error("empty candidate set")]
  Empty,
  #[error("bad RNG seed (expect 64-hex)")]
  BadSeed,
  #[error("unknown option id: {0}")]
  UnknownOption(String),
}

6) Algorithm Outline
• Iterate `contexts` in stable order (as provided; upstream must pre-sort).
• For each context:
  1) Build `cands = canonicalize_candidates(context.candidates, options_index)`.
  2) Try policy chain:
     - StatusQuo: if any candidate has is_status_quo=true, choose it → detail=None.
       If none, fall through to Deterministic.
     - Deterministic: choose min (order_index, OptionId) → detail=Some(OrderIndex).
     - Random: init `TieRng` via `tie_rng_from_seed(seed_hex64)`; draw uniformly (rejection sampling) among indices 0..cands.len(); record `word_index` via RNG API; detail=Some(Rng{seed_hex64, word_index}).
  3) Emit `TieLogEntry { context: fmt!(kind/unit), candidates: cands.clone(), policy, detail, winner }`.
  4) winners.insert((kind, unit), winner).

Notes:
• No “deterministic_order_key” knob — deterministic always means Option.order_index then OptionId.
• Only use RNG when policy = Random; never touch OS entropy/time.

7) State Flow
… → (optional) MAP_FRONTIER → **RESOLVE_TIES** → LABEL → BUILD_RESULT → BUILD_RUN_RECORD.
RunRecord builder will attach `ties: Vec<TieLogEntry>` and, if policy was random, set `rng_seed` (64-hex). Result does **not** carry tie logs.

8) Determinism & Numeric Rules
• All candidate orderings derive from (order_index, OptionId) for stability.
• RNG path uses ChaCha20 seeded from the provided 64-hex; identical seeds ⇒ identical picks and `word_index`.
• No floats; integers and indices only.

9) Edge Cases & Failure Policy
• Empty candidate set ⇒ ResolveError::Empty (should not happen; upstream bug).
• Unknown OptionId in candidates ⇒ ResolveError::UnknownOption (defensive).
• Bad 64-hex seed ⇒ ResolveError::BadSeed (policy=random).
• StatusQuo with multiple status_quo candidates: still resolved by deterministic order (spec doesn’t allow multiple SQ; we harden).

10) Test Checklist (must pass)
• Deterministic policy: candidates {B,A} with order_index(A)<(B) ⇒ winner A; TieLog.detail = OrderIndex.
• StatusQuo policy: candidates {Change, SQ} ⇒ winner SQ; if no SQ present, falls back to Deterministic.
• Random policy: fixed seed_hex64 ⇒ two runs yield same winner and same word_index; different seeds differ.
• WTA context, LastSeat context, IRV elimination context each produce a TieLog entry and unblock the caller.
• Logging placement: pipeline writes `ties[]` into RunRecord; Result contains no tie log.
• No “deterministic_order_key” present anywhere; ordering is inherent in OptionItem.

11) Notes for RunRecord wiring
• RunRecord must include:
  - tie_policy (string enum: "status_quo" | "deterministic" | "random")
  - rng_seed (64-hex) **only if** policy = "random"
  - ties[]: Vec<TieLogEntry> (full log) and/or a summary block (counts)
• Ensure IDs and timestamps are added by BUILD_RUN_RECORD; do not hash/log here.
```
