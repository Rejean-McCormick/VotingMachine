//! Condorcet tabulation (deterministic, integers-only, no RNG).
//!
//! Contract (Doc 1 / Annex A aligned):
//! - Inputs are compressed ballot groups: (ranking, multiplicity).
//! - Options come in canonical order: (order_index, option_id).
//! - If a Condorcet winner exists (strictly beats every other), select it.
//! - Otherwise resolve cycles via a deterministic completion rule
//!   (default Schulze; Minimax supported). No RNG here.
//!
//! Output:
//! - `UnitScores.scores` are **winner-only** tallies (winner gets V, others 0) in
//!   canonical key order to keep downstream deterministic and simple.
//! - `Pairwise` matrix is emitted for audit; `CondorcetLog` records rule + winner.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use vm_core::{
    entities::{OptionItem, TallyTotals},
    ids::{OptionId, UnitId},
    variables::Params,
};

use crate::UnitScores;

/// Pairwise audit matrix: wins[(A,B)] = number of ballots that prefer A over B.
#[derive(Debug, Clone)]
pub struct Pairwise {
    pub wins: BTreeMap<(OptionId, OptionId), u64>,
}

/// Completion rule used when no strict Condorcet winner exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionRule {
    Schulze,
    Minimax,
}

/// Log for Condorcet tabulation.
#[derive(Debug, Clone)]
pub struct CondorcetLog {
    pub completion_rule: CompletionRule,
    pub winner: OptionId,
    pub pairwise_summary: Pairwise,
}

/// Deterministic Condorcet tabulation.
///
/// Returns `(UnitScores /*winner-only*/, Pairwise, CondorcetLog)`.
pub fn tabulate_ranked_condorcet(
    unit_id: UnitId,
    ballots: &[(Vec<OptionId>, u64)],
    options: &[OptionItem],
    turnout: TallyTotals,
    params: &Params, // used to choose completion rule; default Schulze
) -> (UnitScores, Pairwise, CondorcetLog) {
    // Canonical option list & index map (tie-break order).
    let order: Vec<OptionId> = options.iter().map(|o| o.option_id.clone()).collect();
    let ord_idx: HashMap<&OptionId, usize> = order
        .iter()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect();

    // Build the pairwise matrix.
    let pairwise = build_pairwise(ballots, &order);

    // Zero-valid case: deterministic fallback to first option by canonical order.
    let v = turnout.valid_ballots;
    let (winner, rule_used) = if v == 0 {
        (
            order
                .get(0)
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".parse().unwrap_or_else(|_| OptionId::from("UNKNOWN"))),
            CompletionRule::Schulze,
        )
    } else if let Some(w) = condorcet_winner(&pairwise, &order) {
        (w, CompletionRule::Schulze) // rule is moot when strict winner exists
    } else {
        // Resolve via completion rule.
        let rule = completion_rule_from_params(params).unwrap_or(CompletionRule::Schulze);
        let w = match rule {
            CompletionRule::Schulze => schulze_winner(&pairwise, &order, &ord_idx),
            CompletionRule::Minimax => minimax_winner(&pairwise, &order, &ord_idx),
        };
        (w, rule)
    };

    // Winner-only scores map: winner gets V, others 0, in canonical key order.
    let scores_map = winner_scores(&winner, v, options);

    let scores = UnitScores {
        unit_id,
        turnout,
        scores: scores_map,
    };
    let log = CondorcetLog {
        completion_rule: rule_used,
        winner: winner.clone(),
        pairwise_summary: pairwise.clone(),
    };
    (scores, pairwise, log)
}

/// Compute the pairwise matrix from ranked ballots in canonical option set `order`.
pub fn build_pairwise(ballots: &[(Vec<OptionId>, u64)], order: &[OptionId]) -> Pairwise {
    let allowed: BTreeSet<OptionId> = order.iter().cloned().collect();
    let mut wins: BTreeMap<(OptionId, OptionId), u64> = BTreeMap::new();

    for (ranking, count) in ballots {
        if *count == 0 {
            continue;
        }
        // Filter to unique, allowed options in ballot order (ignore unknowns/dups).
        let mut seen = HashSet::<&OptionId>::new();
        let mut seq: Vec<&OptionId> = Vec::with_capacity(ranking.len());
        for id in ranking {
            if allowed.contains(id) && !seen.contains(id) {
                seen.insert(id);
                seq.push(id);
            }
        }
        // For each ordered pair (i < j), increment wins[(A,B)] by count.
        for i in 0..seq.len() {
            for j in (i + 1)..seq.len() {
                let a = seq[i].clone();
                let b = seq[j].clone();
                *wins.entry((a.clone(), b.clone())).or_insert(0) += *count;
            }
        }
    }

    Pairwise { wins }
}

/// Return a strict Condorcet winner if one exists.
pub fn condorcet_winner(pw: &Pairwise, order: &[OptionId]) -> Option<OptionId> {
    for x in order {
        let mut beats_all = true;
        for y in order {
            if x == y {
                continue;
            }
            let xy = get_win(pw, x, y);
            let yx = get_win(pw, y, x);
            if xy <= yx {
                beats_all = false;
                break;
            }
        }
        if beats_all {
            return Some(x.clone());
        }
    }
    None
}

/// Schulze method winner (with deterministic tie-break by canonical order).
pub fn schulze_winner(
    pw: &Pairwise,
    order: &[OptionId],
    ord_idx: &HashMap<&OptionId, usize>,
) -> OptionId {
    // d[i][j] = wins(i,j) if wins(i,j) > wins(j,i), else 0
    let n = order.len();
    let mut d = vec![vec![0u64; n]; n];
    for (i, a) in order.iter().enumerate() {
        for (j, b) in order.iter().enumerate() {
            if i == j {
                continue;
            }
            let ab = get_win(pw, a, b);
            let ba = get_win(pw, b, a);
            d[i][j] = if ab > ba { ab } else { 0 };
        }
    }
    // p[i][j] = strength of strongest path from i to j
    let mut p = d.clone();
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            for k in 0..n {
                if i == k || j == k {
                    continue;
                }
                let via = std::cmp::min(p[j][i], p[i][k]);
                if p[j][k] < via {
                    p[j][k] = via;
                }
            }
        }
    }
    // Candidate i is a winner if for all j != i, p[i][j] >= p[j][i].
    // Collect all winners, then choose the earliest in canonical order.
    let mut winners: Vec<usize> = Vec::new();
    'outer: for i in 0..n {
        for j in 0..n {
            if i != j && p[i][j] < p[j][i] {
                continue 'outer;
            }
        }
        winners.push(i);
    }
    // Deterministic tie-break: pick the one with smallest canonical order index.
    let best = winners
        .into_iter()
        .min_by_key(|&i| ord_idx.get(&order[i]).copied().unwrap_or(usize::MAX))
        .unwrap_or(0);
    order[best].clone()
}

/// Minimax (aka Simpson/Smith) winner: pick candidate minimizing its maximum defeat.
/// Tie-break deterministically by canonical order.
pub fn minimax_winner(
    pw: &Pairwise,
    order: &[OptionId],
    ord_idx: &HashMap<&OptionId, usize>,
) -> OptionId {
    let n = order.len();
    // For each i, compute max defeat margin: max over j of max( wins(j,i) - wins(i,j), 0 )
    let mut max_defeat: Vec<(u64, usize)> = Vec::with_capacity(n);
    for (i, a) in order.iter().enumerate() {
        let mut worst: u64 = 0;
        for (j, b) in order.iter().enumerate() {
            if i == j {
                continue;
            }
            let ai = get_win(pw, a, b);
            let ia = get_win(pw, b, a);
            if ia > ai {
                let margin = ia - ai;
                if margin > worst {
                    worst = margin;
                }
            }
        }
        let oi = ord_idx.get(a).copied().unwrap_or(usize::MAX);
        max_defeat.push((worst, oi));
    }
    // Choose minimal (max_defeat, order_index)
    let mut best_i = 0usize;
    let mut best_key = (u64::MAX, usize::MAX);
    for (i, key) in max_defeat.into_iter().enumerate() {
        if key < best_key {
            best_key = key;
            best_i = i;
        }
    }
    order[best_i].clone()
}

/// Winner-only scores: `{winner: V, others: 0}` in canonical key order.
pub fn winner_scores(
    winner: &OptionId,
    valid_ballots: u64,
    options: &[OptionItem],
) -> BTreeMap<OptionId, u64> {
    let mut out = BTreeMap::<OptionId, u64>::new();
    for opt in options {
        let v = if &opt.option_id == winner {
            valid_ballots
        } else {
            0
        };
        out.insert(opt.option_id.clone(), v);
    }
    out
}

/// Helper: get wins(A,B) from the matrix (0 if absent).
#[inline]
fn get_win(pw: &Pairwise, a: &OptionId, b: &OptionId) -> u64 {
    *pw.wins.get(&(a.clone(), b.clone())).unwrap_or(&0)
}

/// Read a completion rule from params; returns `None` if not specified/recognized.
fn completion_rule_from_params(_params: &Params) -> Option<CompletionRule> {
    // The reference spec keeps this per-release; we default to Schulze.
    // If you later name a param (e.g., v005_aggregation_mode == "minimax"),
    // wire it here (lowercase match).
    None
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
    fn strict_condorcet_exists() {
        // A beats B and C; B beats C.
        let options = vec![opt("A", 0), opt("B", 1), opt("C", 2)];
        let ballots = vec![
            (vec!["A".parse().unwrap(), "B".parse().unwrap(), "C".parse().unwrap()], 40),
            (vec!["A".parse().unwrap(), "C".parse().unwrap(), "B".parse().unwrap()], 15),
            (vec!["B".parse().unwrap(), "C".parse().unwrap(), "A".parse().unwrap()], 30),
            (vec!["C".parse().unwrap(), "B".parse().unwrap(), "A".parse().unwrap()], 15),
        ];
        let turnout = TallyTotals::new(100, 0);
        let params = Params::default();

        let (scores, _pw, log) = tabulate_ranked_condorcet(
            "U-1".parse().unwrap(),
            &ballots,
            &options,
            turnout,
            &params,
        );

        assert_eq!(log.winner.to_string(), "A");
        assert_eq!(*scores.scores.get(&"A".parse().unwrap()).unwrap(), 100);
        assert_eq!(*scores.scores.get(&"B".parse().unwrap()).unwrap(), 0);
        assert_eq!(*scores.scores.get(&"C".parse().unwrap()).unwrap(), 0);
    }

    #[test]
    fn cycle_resolved_by_schulze_deterministically() {
        // Rock-Paper-Scissors style cycle:
        // A > B, B > C, C > A with equal margins; Schulze tie-break by canonical order.
        let options = vec![opt("A", 0), opt("B", 1), opt("C", 2)];
        let ballots = vec![
            (vec!["A".parse().unwrap(), "B".parse().unwrap(), "C".parse().unwrap()], 34),
            (vec!["B".parse().unwrap(), "C".parse().unwrap(), "A".parse().unwrap()], 33),
            (vec!["C".parse().unwrap(), "A".parse().unwrap(), "B".parse().unwrap()], 33),
        ];
        let turnout = TallyTotals::new(100, 0);
        let params = Params::default();

        let (_scores, _pw, log) = tabulate_ranked_condorcet(
            "U-1".parse().unwrap(),
            &ballots,
            &options,
            turnout,
            &params,
        );

        // With symmetric strengths, Schulze winners can tie; we pick the earliest by canonical order.
        assert_eq!(log.winner.to_string(), "A");
        assert!(matches!(log.completion_rule, CompletionRule::Schulze));
    }

    #[test]
    fn zero_valid_ballots_fallback() {
        let options = vec![opt("A", 0), opt("B", 1)];
        let ballots: Vec<(Vec<OptionId>, u64)> = vec![];
        let turnout = TallyTotals::new(0, 0);
        let params = Params::default();

        let (scores, _pw, log) = tabulate_ranked_condorcet(
            "U-1".parse().unwrap(),
            &ballots,
            &options,
            turnout,
            &params,
        );

        assert_eq!(log.winner.to_string(), "A");
        for opt in &options {
            assert_eq!(*scores.scores.get(&opt.option_id).unwrap(), 0);
        }
    }
}
