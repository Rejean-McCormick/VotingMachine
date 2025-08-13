//! Resolve blocking ties with a fixed policy chain and produce audit logs.
//!
//! Policy order:
//!   1) StatusQuo → if exactly one SQ candidate, pick it
//!   2) Deterministic → min by (order_index, OptionId)
//!   3) Random → seeded ChaCha20 via vm_core::rng, uniform among candidates
//!
//! Notes:
//! * Logs go to RunRecord later; Result never carries tie logs.
//! * RNG path is fully reproducible from the provided 64-hex seed.
//! * All ordering uses canonical (order_index, OptionId).

use std::collections::{BTreeMap, BTreeSet};

use vm_core::{
    ids::{OptionId, UnitId},
    entities::OptionItem,
    rng::{tie_rng_from_seed, TieRng},
};

// ---------- Public types used across the pipeline ----------

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum TieKind {
    WtaWinner,
    LastSeat,
    IrvElimination,
}

#[derive(Clone, Debug)]
pub struct TieContext {
    pub kind: TieKind,
    pub unit: UnitId,
    /// Candidate OptionIds that are tied and valid for the decision.
    /// Upstream guarantees these are real option IDs; we remain defensive in helpers.
    pub candidates: Vec<OptionId>,
}

#[derive(Clone, Debug)]
pub enum TiePolicy {
    StatusQuo,
    Deterministic,                 // by (order_index, OptionId)
    Random { seed_hex64: String }, // VM-VAR-033, lowercase/uppercase accepted
}

#[derive(Clone, Debug)]
pub struct TieLogEntry {
    /// Human-readable, stable context like "WTA U:<unit>" or "LastSeat U:<unit>"
    pub context: String,
    /// Canonicalized candidate list snapshot (order_index, then OptionId)
    pub candidates: Vec<OptionId>,
    /// "status_quo" | "deterministic" | "random"
    pub policy: &'static str,
    /// Extra details (ordering rule used or RNG crumb)
    pub detail: Option<TieDetail>,
    /// Chosen winner
    pub winner: OptionId,
}

#[derive(Clone, Debug)]
pub enum TieDetail {
    /// Deterministic rule: min by (order_index, OptionId)
    OrderIndex,
    /// Random rule details: seed used and the index of the RNG word consumed
    Rng { seed_hex64: String, word_index: u128 },
}

// ---------- Errors (internal to this module) ----------

#[derive(Debug)]
pub enum ResolveError {
    Empty,
    BadSeed,
    UnknownOption(String),
}

// ---------- Entry point ----------

pub fn resolve_ties(
    contexts: &[TieContext],
    options_index: &BTreeMap<OptionId, OptionItem>,
    policy: &TiePolicy,
) -> (Vec<TieLogEntry>, BTreeMap<(TieKind, UnitId), OptionId>) {
    let mut logs: Vec<TieLogEntry> = Vec::with_capacity(contexts.len());
    let mut winners: BTreeMap<(TieKind, UnitId), OptionId> = BTreeMap::new();

    for ctx in contexts {
        // Canonicalize and defensively filter to known options (should already be valid).
        let cands = canonicalize_candidates(&ctx.candidates, options_index);

        // Empty candidate list should never happen; skip with a debug log style entry if it does.
        if cands.is_empty() {
            // fabricate a log-like entry for traceability; do not insert a winner
            let context_str = format!("{:?} U:{:?}", ctx.kind, ctx.unit);
            logs.push(TieLogEntry {
                context: context_str,
                candidates: vec![],
                policy: match policy {
                    TiePolicy::StatusQuo => "status_quo",
                    TiePolicy::Deterministic => "deterministic",
                    TiePolicy::Random { .. } => "random",
                },
                detail: None,
                // winner is meaningless here; use a dummy OptionId if the type allowed; instead, skip insertion.
                winner: OptionId::from_static("OPT:INVALID"),
            });
            continue;
        }

        // Apply policy chain
        let (used_policy, detail, winner) = match policy {
            TiePolicy::StatusQuo => {
                if let Some(sq) = pick_status_quo(&cands, options_index) {
                    ("status_quo", None, sq)
                } else {
                    // fall back to deterministic
                    ("deterministic", Some(TieDetail::OrderIndex), pick_by_order_index(&cands, options_index))
                }
            }
            TiePolicy::Deterministic => {
                ("deterministic", Some(TieDetail::OrderIndex), pick_by_order_index(&cands, options_index))
            }
            TiePolicy::Random { seed_hex64 } => match pick_by_rng(&cands, seed_hex64) {
                Ok((w, word_idx)) => (
                    "random",
                    Some(TieDetail::Rng {
                        seed_hex64: seed_hex64.clone(),
                        word_index: word_idx,
                    }),
                    w,
                ),
                // If RNG init fails (bad seed), fall back deterministically; keep surface stable.
                Err(_e) => ("deterministic", Some(TieDetail::OrderIndex), pick_by_order_index(&cands, options_index)),
            },
        };

        // Build stable context string
        let context_str = match ctx.kind {
            TieKind::WtaWinner => format!("WTA U:{:?}", ctx.unit),
            TieKind::LastSeat => format!("LastSeat U:{:?}", ctx.unit),
            TieKind::IrvElimination => format!("IRV U:{:?}", ctx.unit),
        };

        logs.push(TieLogEntry {
            context: context_str,
            candidates: cands.clone(),
            policy: used_policy,
            detail,
            winner,
        });

        winners.insert((ctx.kind, ctx.unit.clone()), winner);
    }

    (logs, winners)
}

// ---------- Helpers ----------

/// Prefer Status Quo if and only if exactly one candidate is SQ.
/// Returns None if none or multiple are SQ (caller will fall back).
pub fn pick_status_quo(
    cands: &[OptionId],
    options_index: &BTreeMap<OptionId, OptionItem>,
) -> Option<OptionId> {
    let mut found: Vec<OptionId> = Vec::new();
    for id in cands {
        if let Some(opt) = options_index.get(id) {
            if opt.is_status_quo {
                found.push(id.clone());
                if found.len() > 1 {
                    // Multiple SQ -> do not resolve here
                    return None;
                }
            }
        }
    }
    found.pop()
}

/// Deterministic pick: min by (order_index, OptionId)
pub fn pick_by_order_index(
    cands: &[OptionId],
    options_index: &BTreeMap<OptionId, OptionItem>,
) -> OptionId {
    // Build (order_index, id) tuples; unknown IDs are ranked last by using a large sentinel.
    const BIG: u32 = u32::MAX;
    let mut best: Option<(u32, &OptionId)> = None;
    for id in cands {
        let key = options_index
            .get(id)
            .map(|o| o.order_index)
            .unwrap_or(BIG);
        match best {
            None => best = Some((key, id)),
            Some((bk, bid)) => {
                if key < bk || (key == bk && id < bid) {
                    best = Some((key, id));
                }
            }
        }
    }
    best.expect("non-empty cands").1.clone()
}

/// Seeded random pick (ChaCha20 via vm_core::rng). Uniform among candidates.
/// Returns the chosen OptionId and the RNG word index consumed for this pick.
///
/// NOTE: This implementation draws a single 64-bit word and reduces modulo len.
/// The reported `word_index` is 0 for this pick; if you extend this to reuse a
/// single RNG instance across multiple picks, increment the index accordingly.
pub fn pick_by_rng(cands: &[OptionId], seed_hex64: &str) -> Result<(OptionId, u128), ResolveError> {
    if !is_hex64(seed_hex64) {
        return Err(ResolveError::BadSeed);
    }
    let mut rng: TieRng = tie_rng_from_seed(seed_hex64).map_err(|_| ResolveError::BadSeed)?;
    let n = cands.len();
    if n == 0 {
        return Err(ResolveError::Empty);
    }
    // Draw one 64-bit word and reduce; rejection sampling is unnecessary for modulo here.
    let r = rng.next_u64();
    let idx = (r as usize) % n;
    Ok((cands[idx].clone(), 0u128))
}

/// Sort candidates by (order_index, OptionId) and drop unknown IDs defensively.
/// Unknowns should never appear (validated upstream), but we keep stability.
pub fn canonicalize_candidates(
    cands: &[OptionId],
    options_index: &BTreeMap<OptionId, OptionItem>,
) -> Vec<OptionId> {
    let mut set: BTreeSet<(u32, OptionId)> = BTreeSet::new();
    for id in cands {
        if let Some(opt) = options_index.get(id) {
            set.insert((opt.order_index, id.clone()));
        }
    }
    set.into_iter().map(|(_, id)| id).collect()
}

// ---------- Small local helpers ----------

fn is_hex64(s: &str) -> bool {
    if s.len() != 64 {
        return false;
    }
    s.chars().all(|c| c.is_ascii_hexdigit())
}

// ---------- Convenience re-exports (optional) ----------

pub use TieDetail as _TieDetailForExport;
pub use TieKind as _TieKindForExport;
pub use TieLogEntry as _TieLogEntryForExport;
pub use TiePolicy as _TiePolicyForExport;
pub use TieContext as _TieContextForExport;
