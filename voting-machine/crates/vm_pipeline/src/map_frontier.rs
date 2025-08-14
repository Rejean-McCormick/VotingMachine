//! crates/vm_pipeline/src/map_frontier.rs
//! NOTE: This module implements *allocation-affecting tie resolution* per VM-VAR-050 (tie_policy)
//! and VM-VAR-052 (tie_seed). RNG is used **only** when policy = "random" and a real tie occurs.
//!
//! Log vocabulary (Doc 6 / Annex A):
//!   policy ∈ {"status_quo","deterministic_order","random"}
//!   context is a stable code (e.g., "WTA U:<unit>", "LAST_SEAT U:<unit>", "IRV_ELIM U:<unit>").
//!
//! This file is delivered in two parts; this is Part 1 (types, policy surface, and models).

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    entities::OptionItem,
    ids::{OptionId, UnitId},
    variables::Params,
};
use vm_core::rng::{tie_rng_from_seed, TieRng}; // seed: u64 → deterministic RNG

/* ------------------------------ Public policy surface ------------------------------ */

/// Effective tie policy derived from `Params` (VM-VAR-050,052).
#[derive(Clone, Debug)]
pub enum TiePolicy {
    /// Prefer the status-quo option if (and only if) exactly one candidate matches it.
    StatusQuo { status_quo: OptionId },
    /// Deterministic by registry order (order_index, then OptionId).
    DeterministicOrder,
    /// Random among candidates, seeded by VM-VAR-052.
    Random { seed: u64 }, // VM-VAR-052
}

impl TiePolicy {
    /// Build the effective policy from `Params` and an optional status-quo option id for this context.
    pub fn from_params(p: &Params, status_quo: OptionId) -> Self {
        match p.v050_tie_policy.as_str() {
            // Allowed spellings; align to canonical tokens upstream if needed.
            "status_quo" | "status-quo" | "sq" => TiePolicy::StatusQuo { status_quo },
            "deterministic_order" | "deterministic" | "order" => TiePolicy::DeterministicOrder,
            "random" => TiePolicy::Random { seed: p.v052_tie_seed },
            other => {
                // Unknown → fall back to deterministic (spec-safe default).
                tracing::warn!("unknown tie_policy '{}', falling back to deterministic_order", other);
                TiePolicy::DeterministicOrder
            }
        }
    }
}

/* ----------------------------------- Input model ----------------------------------- */

/// What kind of tie is being resolved (stable code used in logging).
#[derive(Clone, Debug)]
pub enum TieKind {
    Wta,          // winner-take-all
    LastSeat,     // last seat fill
    IrvElim,      // IRV elimination
    Custom(&'static str),
}

impl TieKind {
    #[inline]
    pub fn code(&self) -> &'static str {
        match self {
            TieKind::Wta => "WTA",
            TieKind::LastSeat => "LAST_SEAT",
            TieKind::IrvElim => "IRV_ELIM",
            TieKind::Custom(s) => s,
        }
    }
}

/// One tie to resolve.
#[derive(Clone, Debug)]
pub struct TieContext {
    pub unit_id: UnitId,
    pub kind: TieKind,
    /// Candidates among which to pick a winner.
    pub candidates: Vec<OptionId>,
    /// Optional status-quo candidate for this context (used only if policy = StatusQuo).
    pub status_quo: Option<OptionId>,
    /// Canonical options for this unit (ordered by (order_index, OptionId)).
    pub options: Vec<OptionItem>,
}

/* ------------------------------------ Outputs -------------------------------------- */

#[derive(Clone, Debug)]
pub struct TieOutcome {
    pub unit_id: UnitId,
    pub winner: OptionId,
    pub log: TieLogEntry,
}

/// Log entry for one tie resolution (Doc 6 acceptance).
#[derive(Clone, Debug)]
pub struct TieLogEntry {
    pub context: String,       // e.g., "WTA U:<unit>"
    pub policy: &'static str,  // "status_quo" | "deterministic_order" | "random"
    pub detail: TieDetail,
    pub candidates: Vec<OptionId>, // canonicalized candidates (stable order)
}

#[derive(Clone, Debug)]
pub enum TieDetail {
    /// Winner is the status-quo option (must be uniquely present among candidates).
    StatusQuo { option: OptionId },
    /// Winner is the lowest (order_index, OptionId) among candidates.
    OrderIndex { order_index: u32 },
    /// Winner chosen uniformly at random among candidates using VM-VAR-052.
    Rng { seed: u64 /*, word_index: u128 (optional if available)*/ },
}

/// Result for a batch of ties.
#[derive(Clone, Debug, Default)]
pub struct ResolveResult {
    pub outcomes: Vec<TieOutcome>,
    /// True iff at least one tie used the "random" policy (callers should echo rng_seed in RunRecord).
    pub used_random: bool,
}

/* ------------------------------------ Errors --------------------------------------- */

#[derive(thiserror::Error, Debug)]
pub enum ResolveError {
    #[error("empty candidate set")]
    EmptyCandidates,
    #[error("unknown candidate id: {0}")]
    UnknownCandidate(String),
    #[error("status_quo not present among candidates")]
    StatusQuoNotInCandidates,
}

/* ------------- Part 2 (functions & helpers) continues in the next chunk ------------ */
/* --------------------------------- Entry points ------------------------------------ */

/// Resolve a batch of ties. Returns outcomes and whether any random policy fired.
pub fn resolve_many(
    ties: &[TieContext],
    policy: &TiePolicy,
) -> Result<ResolveResult, ResolveError> {
    let mut out = ResolveResult::default();
    for t in ties {
        let outcome = resolve_one(t, policy)?;
        if matches!(outcome.log.policy, "random") {
            out.used_random = true;
        }
        out.outcomes.push(outcome);
    }
    Ok(out)
}

/// Resolve a single tie.
pub fn resolve_one(t: &TieContext, policy: &TiePolicy) -> Result<TieOutcome, ResolveError> {
    // Canonicalize candidates: check existence and sort by (order_index, OptionId)
    let index = index_options(&t.options);
    let cands = canonicalize_candidates(&t.candidates, &index)?;

    // Build stable context string for logs
    let ctx_str = format!("{} U:{}", t.kind.code(), t.unit_id);

    // Apply policy
    let (winner, log) = match policy {
        TiePolicy::StatusQuo { status_quo: _ } => {
            if let Some(sq) = &t.status_quo {
                // Status-quo applies only if the SQ candidate is in the tie set; else deterministic fallback.
                if !cands.iter().any(|id| id == sq) {
                    let (w, oi) = pick_by_order(&cands, &index);
                    let log = TieLogEntry {
                        context: ctx_str,
                        policy: "deterministic_order",
                        detail: TieDetail::OrderIndex { order_index: oi },
                        candidates: cands.clone(),
                    };
                    (w, log)
                } else {
                    let log = TieLogEntry {
                        context: ctx_str,
                        policy: "status_quo",
                        detail: TieDetail::StatusQuo { option: sq.clone() },
                        candidates: cands.clone(),
                    };
                    (sq.clone(), log)
                }
            } else {
                // No SQ provided → deterministic fallback
                let (w, oi) = pick_by_order(&cands, &index);
                let log = TieLogEntry {
                    context: ctx_str,
                    policy: "deterministic_order",
                    detail: TieDetail::OrderIndex { order_index: oi },
                    candidates: cands.clone(),
                };
                (w, log)
            }
        }

        TiePolicy::DeterministicOrder => {
            let (w, oi) = pick_by_order(&cands, &index);
            let log = TieLogEntry {
                context: ctx_str,
                policy: "deterministic_order",
                detail: TieDetail::OrderIndex { order_index: oi },
                candidates: cands.clone(),
            };
            (w, log)
        }

        TiePolicy::Random { seed } => {
            let (w, _idx) = pick_by_rng(&cands, *seed)?; // unbiased, deterministic
            let log = TieLogEntry {
                context: ctx_str,
                policy: "random",
                detail: TieDetail::Rng { seed: *seed },
                candidates: cands.clone(),
            };
            (w, log)
        }
    };

    Ok(TieOutcome {
        unit_id: t.unit_id.clone(),
        winner,
        log,
    })
}

/* --------------------------------- Helpers ----------------------------------------- */

/// Build a quick lookup from OptionId → (order_index, OptionItem)
fn index_options(options: &[OptionItem]) -> BTreeMap<OptionId, (&OptionItem, u32)> {
    let mut m = BTreeMap::new();
    for o in options {
        m.insert(o.option_id.clone(), (o, o.order_index));
    }
    m
}

/// Verify that every candidate exists in the unit's options and return them sorted canonically.
fn canonicalize_candidates(
    candidates: &[OptionId],
    index: &BTreeMap<OptionId, (&OptionItem, u32)>,
) -> Result<Vec<OptionId>, ResolveError> {
    if candidates.is_empty() {
        return Err(ResolveError::EmptyCandidates);
    }
    // Use a set to deduplicate while preserving canonical sort via (order_index, OptionId).
    let mut set: BTreeSet<(u32, OptionId)> = BTreeSet::new();
    for id in candidates {
        if let Some((_, oi)) = index.get(id) {
            set.insert((*oi, id.clone()));
        } else {
            return Err(ResolveError::UnknownCandidate(id.to_string()));
        }
    }
    Ok(set.into_iter().map(|(_, id)| id).collect())
}

/// Deterministic pick by lowest (order_index, OptionId).
fn pick_by_order(
    cands: &[OptionId],
    index: &BTreeMap<OptionId, (&OptionItem, u32)>,
) -> (OptionId, u32) {
    let mut best: Option<(&OptionId, u32)> = None;
    for id in cands {
        let (_, oi) = index.get(id).expect("candidate must exist in index");
        match best {
            None => best = Some((id, *oi)),
            Some((_, cur)) if *oi < cur => best = Some((id, *oi)),
            _ => {}
        }
    }
    let (id, oi) = best.expect("non-empty candidates");
    (id.clone(), oi)
}

/// Unbiased random pick using engine RNG (seeded with VM-VAR-052).
fn pick_by_rng(cands: &[OptionId], seed: u64) -> Result<(OptionId, usize), ResolveError> {
    if cands.is_empty() {
        return Err(ResolveError::EmptyCandidates);
    }
    let mut rng: TieRng = tie_rng_from_seed(seed);
    // Prefer engine helper to avoid modulo bias and to keep determinism stable.
    let idx = rng
        .gen_range(cands.len())
        .expect("engine gen_range must handle non-zero upper bound");
    Ok((cands[idx].clone(), idx))
}

/* --------------------------------- Tests (optional) -------------------------------- */
// See Part 1 for commented tests scaffolding; enable once fixtures are available.
