//! IRV tabulation (deterministic, integers-only, no RNG).
//!
//! Contract (aligned with Doc 1B & engine rules):
//! - Inputs are compressed ballot groups: (ranking, multiplicity).
//! - Options are provided in canonical order: (order_index, option_id).
//! - Majority test is against the **continuing denominator** (reduce-on-exhaustion).
//! - Elimination tie-break is deterministic by canonical option order.
//! - We recompute tallies from scratch each round given the current continuing set
//!   (pure, simple, and deterministic), and separately log transfers/exhausted for
//!   the eliminated option in that round.
//!
//! Outputs:
//! - `UnitScores.scores` are the **final-round tallies** in canonical option order
//!   (eliminated options end at 0).
//! - `IrvLog` contains per-round eliminations, transfers, and exhausted counts,
//!   plus the winner option_id.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use vm_core::{
    entities::{OptionItem, TallyTotals},
    ids::{OptionId, UnitId},
    variables::Params,
};

use crate::UnitScores;

/// One IRV elimination round.
#[derive(Debug, Clone)]
pub struct IrvRound {
    /// The eliminated option in this round.
    pub eliminated: OptionId,
    /// Transfers from the eliminated option to next continuing preferences.
    /// Keys are destination option IDs; values are transferred multiplicities.
    pub transfers: BTreeMap<OptionId, u64>,
    /// Ballots that had no further continuing preference and thus exhausted this round.
    pub exhausted: u64,
}

/// Full IRV audit log for a unit.
#[derive(Debug, Clone)]
pub struct IrvLog {
    pub rounds: Vec<IrvRound>,
    pub winner: OptionId,
}

/// Deterministic IRV tabulation with reduce-on-exhaustion denominator.
pub fn tabulate_ranked_irv(
    unit_id: UnitId,
    ballots: &[(Vec<OptionId>, u64)],
    options: &[OptionItem],
    turnout: TallyTotals,
    params: &Params, // currently unused (policy fixed); present for forward compatibility
) -> (UnitScores, IrvLog) {
    let _ = params; // suppress unused for now

    // Canonical option order (tie-break order).
    let order: Vec<OptionId> = options.iter().map(|o| o.option_id.clone()).collect();
    let ord_idx: HashMap<&OptionId, usize> = order
        .iter()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect();

    // Continuing set starts as all options.
    let mut continuing: BTreeSet<OptionId> = order.iter().cloned().collect();

    // Continuing denominator starts at valid ballots; shrinks by per-round exhausted.
    let mut continuing_total: u64 = turnout.valid_ballots;

    // Zero-valid case: deterministic fallback to the first option by canonical order.
    if continuing_total == 0 {
        let winner = order
            .get(0)
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".parse().unwrap_or_else(|_| OptionId::from("UNKNOWN")));
        let final_scores = finalize_scores(BTreeMap::new(), options);
        let scores = UnitScores {
            unit_id,
            turnout,
            scores: final_scores,
        };
        let log = IrvLog {
            rounds: Vec::new(),
            winner,
        };
        return (scores, log);
    }

    let mut rounds: Vec<IrvRound> = Vec::new();
    let winner: OptionId;

    loop {
        // 1) Tally first preferences among continuing options.
        let tallies = first_preferences(ballots, &continuing);

        // 2) Majority check (strict > 50% of continuing_total).
        if let Some(w) = majority_winner(&tallies, continuing_total) {
            winner = w;
            break;
        }

        // 3) Single remaining => winner.
        if continuing.len() == 1 {
            winner = continuing.iter().next().unwrap().clone();
            break;
        }

        // 4) Pick lowest tally; tie-break by canonical order.
        let eliminated = pick_lowest(&tallies, &ord_idx, &continuing);

        // 5) Compute transfers and exhausted (from ballots whose current top is `eliminated`).
        let (transfers, exhausted) =
            transfer_from_eliminated(ballots, &eliminated, &continuing);

        // 6) Apply exhaustion policy: reduce continuing denominator.
        continuing_total = continuing_total.saturating_sub(exhausted);

        // 7) Remove eliminated from continuing set.
        continuing.remove(&eliminated);

        // 8) Log round.
        rounds.push(IrvRound {
            eliminated,
            transfers,
            exhausted,
        });

        // Continue to next round.
    }

    // Final tallies with the last continuing set.
    let final_tallies = first_preferences(ballots, &continuing);
    let final_scores = finalize_scores(final_tallies, options);

    let scores = UnitScores {
        unit_id,
        turnout,
        scores: final_scores,
    };
    let log = IrvLog { rounds, winner };
    (scores, log)
}

/// Build first-preference tallies among the provided `continuing` set.
fn first_preferences(
    ballots: &[(Vec<OptionId>, u64)],
    continuing: &BTreeSet<OptionId>,
) -> BTreeMap<OptionId, u64> {
    let mut out: BTreeMap<OptionId, u64> = BTreeMap::new();
    for (ranking, n) in ballots {
        if let Some(first) = first_pref_in_set(ranking, continuing) {
            *out.entry(first).or_insert(0) += *n;
        }
    }
    out
}

/// Return the first option in `ranking` that is a member of `set`.
fn first_pref_in_set<'a>(
    ranking: &'a [OptionId],
    set: &BTreeSet<OptionId>,
) -> Option<OptionId> {
    for id in ranking {
        if set.contains(id) {
            return Some(id.clone());
        }
    }
    None
}

/// Choose the lowest-tally continuing option; tie-break by canonical order via `ord_idx`.
fn pick_lowest(
    tallies: &BTreeMap<OptionId, u64>,
    ord_idx: &HashMap<&OptionId, usize>,
    continuing: &BTreeSet<OptionId>,
) -> OptionId {
    // Build (tally, order_index, id) tuples for continuing options; missing tallies treated as 0.
    let mut best: Option<(u64, usize, OptionId)> = None;
    for id in continuing {
        let t = *tallies.get(id).unwrap_or(&0);
        let oi = *ord_idx.get(id).unwrap_or(&usize::MAX);
        let cand = (t, oi, id.clone());
        if best.is_none() || cand < best.as_ref().unwrap().clone() {
            best = Some(cand);
        }
    }
    best.expect("continuing non-empty").3
}

/// For ballots currently allocated to `eliminated`, compute the transfers to next continuing
/// preferences (with `eliminated` removed) and the exhausted count.
fn transfer_from_eliminated(
    ballots: &[(Vec<OptionId>, u64)],
    eliminated: &OptionId,
    continuing: &BTreeSet<OptionId>,
) -> (BTreeMap<OptionId, u64>, u64) {
    let mut transfers: BTreeMap<OptionId, u64> = BTreeMap::new();
    let mut exhausted: u64 = 0;

    // Continuing set without the eliminated option (borrow-then-clone on demand).
    // We'll test membership dynamically for clarity.
    for (ranking, n) in ballots {
        // Is this ballot currently allocated to `eliminated`?
        if let Some(first_now) = first_pref_in_set(ranking, continuing) {
            if &first_now == eliminated {
                // Find the next preference in the *remaining* continuing set (excluding `eliminated`).
                if let Some(next_dest) =
                    next_pref_after_eliminated(ranking, eliminated, continuing)
                {
                    *transfers.entry(next_dest).or_insert(0) += *n;
                } else {
                    exhausted = exhausted.saturating_add(*n);
                }
            }
        }
    }

    (transfers, exhausted)
}

/// Scan forward in `ranking` to find the *next* continuing preference after the first encounter
/// of `eliminated`. If none exists, returns None (the ballot exhausts on this round).
fn next_pref_after_eliminated(
    ranking: &[OptionId],
    eliminated: &OptionId,
    continuing: &BTreeSet<OptionId>,
) -> Option<OptionId> {
    let mut passed_elim = false;
    for id in ranking {
        if !continuing.contains(id) {
            continue;
        }
        if !passed_elim {
            if id == eliminated {
                passed_elim = true;
            } else {
                // First continuing is NOT the eliminated; this ballot isn't allocated to eliminated.
                return None;
            }
        } else {
            // This is the next continuing choice after the eliminated one.
            return Some(id.clone());
        }
    }
    // No further continuing preference.
    if passed_elim {
        None
    } else {
        // Never encountered the eliminated as a continuing choice => not allocated to it.
        None
    }
}

/// If any option has a strict majority of the continuing denominator, return it.
fn majority_winner(
    tallies: &BTreeMap<OptionId, u64>,
    continuing_total: u64,
) -> Option<OptionId> {
    let threshold = continuing_total / 2; // need > threshold
    let mut best: Option<(u64, OptionId)> = None;
    for (id, &t) in tallies {
        if t > threshold {
            let cand = (t, id.clone());
            if best.as_ref().map_or(true, |b| cand.0 > b.0) {
                best = Some(cand);
            }
        }
    }
    best.map(|(_, id)| id)
}

/// Render final-round tallies for *all* options in canonical order;
/// eliminated options (not in the final tallies map) receive 0.
fn finalize_scores(
    last_round_tallies: BTreeMap<OptionId, u64>,
    options: &[OptionItem],
) -> BTreeMap<OptionId, u64> {
    let mut out = BTreeMap::<OptionId, u64>::new();
    for opt in options {
        let v = last_round_tallies.get(&opt.option_id).copied().unwrap_or(0);
        out.insert(opt.option_id.clone(), v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use vm_core::entities::OptionItem;

    fn opt(id: &str, idx: u16) -> OptionItem {
        OptionItem::new(
            id.parse().expect("opt id"),
            format!("Name {id}"),
            idx,
        )
        .expect("option")
    }

    #[test]
    fn simple_majority_in_first_round() {
        let options = vec![opt("A", 0), opt("B", 1)];
        let ballots = vec![
            (vec!["A".parse().unwrap()], 60),
            (vec!["B".parse().unwrap()], 40),
        ];
        let turnout = TallyTotals::new(100, 0);
        let params = Params::default();

        let (scores, log) = tabulate_ranked_irv("U-1".parse().unwrap(), &ballots, &options, turnout, &params);
        assert_eq!(log.rounds.len(), 0, "no rounds when majority in R1");
        assert_eq!(log.winner.to_string(), "A");
        assert_eq!(*scores.scores.get(&"A".parse().unwrap()).unwrap(), 60);
        assert_eq!(*scores.scores.get(&"B".parse().unwrap()).unwrap(), 40);
    }

    #[test]
    fn elimination_with_exhaustion() {
        // R1: A=35, B=40, C=25 (100 valid)
        // Eliminate C; suppose 15 go to B, 10 exhaust → continuing_total becomes 90.
        // R2: A=35, B=55 → winner B.
        let options = vec![opt("A", 0), opt("B", 1), opt("C", 2)];
        let ballots = vec![
            (vec!["A".parse().unwrap(), "B".parse().unwrap()], 35),
            (vec!["B".parse().unwrap()], 40),
            (vec!["C".parse().unwrap(), "B".parse().unwrap()], 15),
            (vec!["C".parse().unwrap()], 10), // will exhaust when C eliminated
        ];
        let turnout = TallyTotals::new(100, 0);
        let params = Params::default();

        let (_scores, log) =
            tabulate_ranked_irv("U-1".parse().unwrap(), &ballots, &options, turnout, &params);

        assert_eq!(log.rounds.len(), 1);
        assert_eq!(log.rounds[0].eliminated.to_string(), "C");
        assert_eq!(log.rounds[0].exhausted, 10);
        assert_eq!(
            *log.rounds[0]
                .transfers
                .get(&"B".parse().unwrap())
                .unwrap_or(&0),
            15
        );
        assert_eq!(log.winner.to_string(), "B");
    }

    #[test]
    fn zero_valid_ballots_deterministic_winner() {
        let options = vec![opt("A", 0), opt("B", 1), opt("C", 2)];
        let ballots: Vec<(Vec<OptionId>, u64)> = vec![];
        let turnout = TallyTotals::new(0, 0);
        let params = Params::default();

        let (scores, log) =
            tabulate_ranked_irv("U-1".parse().unwrap(), &ballots, &options, turnout, &params);

        // No rounds, winner is the first by canonical order.
        assert_eq!(log.rounds.len(), 0);
        assert_eq!(log.winner.to_string(), "A");
        // All final tallies are zero.
        for opt in &options {
            assert_eq!(*scores.scores.get(&opt.option_id).unwrap(), 0);
        }
    }
}
