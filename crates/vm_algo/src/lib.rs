// crates/vm_algo/src/lib.rs
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;

// Core IDs and basic tallies
pub use vm_core::{
    entities::TallyTotals as Turnout,
    ids::{OptionId, UnitId},
};

// ----------------------------- Canonical per-unit scores -----------------------------

/// Raw scores per unit (plurality votes, approvals, score sums, or ranked outputs).
/// Downstream emission must follow registry order; keep scores keyed by OptionId.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitScores {
    pub unit_id: UnitId,
    pub turnout: Turnout,
    pub scores: BTreeMap<OptionId, u64>,
}

/// Allocation bundle (used by WTA or when a per-unit seat vector is needed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allocation {
    pub unit_id: UnitId,
    /// Seats (PR) or 1 for the winner in WTA.
    pub seats_or_power: BTreeMap<OptionId, u32>,
    /// True iff a last seat/winner involved a tie break.
    pub last_seat_tie: bool,
}

// ----------------------------- Tabulation (public surface) ---------------------------

pub mod tabulation {
    // File modules (actual implementations)
    pub mod plurality;
    pub mod approval;
    pub mod score;
    pub mod ranked_irv;
    pub mod ranked_condorcet;

    // Re-export entry points and errors (simple tabulators are fallible).
    pub use approval::{tabulate_approval, TabError as ApprovalError};
    pub use plurality::{tabulate_plurality, TabError as PluralityError};
    pub use score::{tabulate_score, TabError as ScoreError};

    // Ranked tabulators + their audit types are exposed from their modules.
    pub use ranked_irv::{tabulate_ranked_irv, IrvLog, IrvRound};
    pub use ranked_condorcet::{
        tabulate_ranked_condorcet, CompletionRule, CondorcetLog, Pairwise,
    };
}

// Convenience re-exports (pipeline imports these from crate root)
pub use tabulation::{IrvLog, IrvRound, Pairwise};

// ----------------------------- Allocation (public surface) ---------------------------

pub mod allocation {
    // File modules (actual implementations)
    pub mod dhondt;
    pub mod sainte_lague;
    pub mod largest_remainder;
    pub mod wta;

    // Name aliases to match caller expectations:
    //  - keep the module definitions (allocate_*) as-is,
    //  - export pipeline-friendly names (dhondt_allocate, …) as plain aliases.
    pub use dhondt::allocate_dhondt as dhondt_allocate;
    pub use sainte_lague::allocate_sainte_lague as sainte_lague_allocate;
    pub use largest_remainder::allocate_largest_remainder as largest_remainder_allocate;
    pub use wta::allocate_wta;

    // Error type aliases for ergonomic matching in callers.
    pub type DhondtError = dhondt::AllocError;
    pub type SainteLagueError = sainte_lague::AllocError;
    pub type LrError = largest_remainder::AllocError;
    pub type WtaError = wta::AllocError;

    // Quota kind is needed by callers; re-export here and via `enums` shim below.
    pub use largest_remainder::QuotaKind;
}

// ----------------------------- Enums shim (caller ergonomics) ------------------------

/// Shims to keep external imports stable, e.g. `use vm_algo::enums::{LrQuotaKind, TiePolicy}`.
pub mod enums {
    pub use vm_core::variables::TiePolicy;
    pub use crate::allocation::QuotaKind as LrQuotaKind;
}

// ----------------------------- MMP & Gates/Frontier ---------------------------------

// File modules
pub mod mmp;
pub mod gates_frontier;

// Tight, explicit re-exports (avoid wildcard export drift).
pub use gates_frontier::{
    apply_decision_gates, map_frontier, FrontierEdge, FrontierFlags, FrontierIn,
    FrontierInputs, FrontierOut, FrontierSummary, FrontierUnit, GateInputs, GateOutcome,
    GateResult,
};

// Re-export MMP outcome type and helpers
pub use mmp::{compute_topups_and_apply_overhang, compute_total_from_share, apportion_targets, MmpOutcome};
