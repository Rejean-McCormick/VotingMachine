<!-- Converted from: 56 - crates vm_pipeline src resolve_ties.rs, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:47.027641Z -->

```
Pre-Coding Essentials (Component: crates/vm_pipeline/src/resolve_ties.rs, Version/FormulaID: VM-ENGINE v0) — 56/89
Goal & Success
Goal: Resolve only ties that block a decision (WTA winner, last seat, IRV elimination) using the configured policy and, if needed, a seeded deterministic RNG, and emit a TieLog consumed by Result/Report.
Success: With the same inputs (and same tie_seed when policy = random), tie outcomes and TieLog are byte-identical across OS/arch.
Scope
In scope: Policy order and contexts; deterministic selection; TieLog entries. If gates failed earlier, enter here only to log a blocking tie (if any).
Out of scope: Tabulation/quotients, gates math, frontier mapping, labeling (handled in other stages).
Inputs → Outputs (with schemas/IDs)
Inputs
Pending TieContext items from prior stages (e.g., “WTA winner in U:…”, “last seat in …”, “IRV elimination …”).
ParameterSet snapshot (notably VM-VAR-032 tie_policy, VM-VAR-033 tie_seed).
Outputs
TieLog entries embedded in Result; tie_seed echoed in RunRecord when used.
A mapping of resolved winners for the caller to finalize the blocked step.
Entities/Tables (minimal)
(N/A — structures are local to the tie stage and serialized into Result/RunRecord per Doc 5.)
Variables (used here)
VM-VAR-032 tie_policy ∈ {status_quo, deterministic, random} (default: status_quo)
VM-VAR-033 tie_seed ∈ integer (≥ 0) (default: 0) — used only when tie_policy = random
Functions (signatures only)
pub enum TiePolicy {
StatusQuo,
Deterministic,
Random { seed: u64 }, // constructed from VM-VAR-033
}

pub enum TieKind { WtaWinner, LastSeat, IrvElimination }

pub struct TieContext {
pub kind: TieKind,
pub unit: UnitId,
pub candidates: Vec<OptionId>,
}

pub struct TieLogEntry {
pub context: String,             // human-readable
pub candidates: Vec<OptionId>,   // sorted, stable
pub policy: &'static str,        // "status_quo" | "deterministic" | "random"
pub detail: &'static str,        // "order_index" | "seed"
pub seed: Option<u64>,           // present iff random
pub winner: OptionId,
}

pub fn resolve_ties(
contexts: &[TieContext],
order_index: &BTreeMap<OptionId, u32>,   // deterministic key
policy: TiePolicy
) -> (Vec<TieLogEntry>, BTreeMap<(TieKind, UnitId), OptionId>);

// helpers
fn pick_status_quo(cands: &[OptionId], is_sq: &BTreeSet<OptionId>) -> Option<OptionId>;
fn pick_by_order(cands: &[OptionId], order_idx: &BTreeMap<OptionId,u32>) -> OptionId;
fn pick_by_rng(cands: &[OptionId], seed: u64) -> OptionId; // ChaCha20; reproducible
Algorithm Outline
Iteration order
Process contexts in a stable order (as provided: already sorted by (kind, unit, candidates) upstream).
Policy application
For each context:
a) status_quo → if any candidate has is_status_quo = true, choose it; if none, fall through to deterministic.
b) deterministic → choose the smallest (order_index, OptionId) among candidates (uses Option.order_index).
c) random → initialize ChaCha20 with tie_seed (VM-VAR-033); draw uniformly among candidates; log the seed in the entry.
Logging
Emit TieLogEntry { context, candidates, policy, detail, seed?, winner }.
detail = "order_index" for deterministic; detail = "seed" for random (with seed set).
Return both: the TieLog vector and a map of resolved winners keyed by (TieKind, UnitId).
State Flow
Pipeline: … → MAP_FRONTIER → RESOLVE_TIES (only if blocking) → LABEL_DECISIVENESS → BUILD_RESULT → BUILD_RUN_RECORD.
If gates failed, caller may still enter here only to log a blocking tie.
Determinism & Numeric Rules
Stable iteration orders; integer-only logic; no floats.
RNG: ChaCha20 seeded only by VM-VAR-033 tie_seed (no OS RNG/time; no parallel RNG).
Same inputs + same seed ⇒ identical winners & logs across OS/arch.
Edge Cases & Failure Policy
If status_quo policy but no SQ in candidates → fall through to deterministic.
Missing seed when policy=random → configuration error; never fallback to OS randomness.
Condorcet cycle ≠ “tie” here (resolved upstream by completion rule).
Threshold equality (e.g., exactly 55%) is not a tie.
Test Checklist (must pass)
Deterministic order: candidates {A,B}, policy=deterministic ⇒ winner has lower order_index (A before B).
Seeded RNG: policy=random, seed=1337 ⇒ two runs produce identical winners and TieLog rows.
Context coverage: WTA winner tie, last-seat tie, and IRV elimination tie each yield a valid entry and unblock the pipeline.
Result/RunRecord wiring: TieLog appears in Result; tie_seed appears in RunRecord when used.
```
