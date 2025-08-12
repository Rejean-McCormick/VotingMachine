<!-- Converted from: 71 - fixtures annex_b part_0 ballots.json.docx on 2025-08-12T18:20:47.493257Z -->

```
Lean pre-coding sheet — 71/89
Component: fixtures/annex_b/part_0/ballots.json (Part 0 fixture: BallotTally dataset)
 Version/FormulaID: Data fixture (not part of FID; FID covers rule primitives only).
1) Goal & success
Goal: Provide the canonical BallotTally for Part 0 in the exact shape required for each ballot type. Must align with Registry options/order and with ParameterSet variables.
Success: Schema-valid; tally sanity holds per unit; deterministic option order respected; loads into pipeline and drives TABULATE correctly.
2) Scope
In scope: One BallotTally dataset with ID/label; per-unit tallies by ballot type (approval/plurality/score/ranked IRV/ranked Condorcet).
Out of scope: Parameter values (separate fixture), allocation/aggregation logic, reporting prose.
3) Inputs → outputs
Input artifact: fixtures/annex_b/part_0/ballots.json (BallotTally).
Used by pipeline: Feeds TABULATE (step 2) after VALIDATE; then flows into allocation/aggregation.
4) Entities/Tables (minimal)
5) Variables (only ones used here)
6) Functions (signatures only)
N/A (fixture only).
7) Algorithm outline (how it’s consumed)
VALIDATE checks tally sanity per unit: Σ(valid option tallies) + invalid_or_blank ≤ ballots_cast.
TABULATE interprets shape per VM-VAR-001: plurality→vote counts; approval→approval counts; score→score sums (with scale/normalization context); IRV→round logs with exhaustion; Condorcet→pairwise from rankings.
Gate denominators: approval gate is fixed to approval rate = approvals_for_change / valid_ballots (not approvals share). Others use support/valid ballots unless VM-VAR-007=on (valid+blank for gates only).
8) Fixture shapes (must match exactly) — per Annex B Part 0
Approval: per Unit: ballots_cast, invalid_or_blank, approvals { Option → count }.
Plurality: per Unit: ballots_cast, invalid_or_blank, votes { Option → count }.
Score: per Unit: ballots_cast, invalid_or_blank, score_sum { Option → sum }, ballots_counted; plus scale (VM-VAR-002..003) and normalization (VM-VAR-004).
Ranked IRV: rounds[{ ranking[], count }]; exhaustion policy is reduce_continuing_denominator (VM-VAR-006).
Ranked Condorcet: ballots[{ ranking[], count }]; completion rule per VM-VAR-005.
9) State flow (very short)
Loaded at LOAD; validated at VALIDATE; consumed by TABULATE to produce UnitScores, which then feed allocation/aggregation.
10) Determinism & numeric rules
Stable option order (by Option.order_index) and sorted JSON keys; counts are integers; presentation rounding occurs only in reports (one decimal).
Approval gate denominator remains approval rate; internal comparisons use round half to even.
11) Edge cases & failure policy
Mismatch with registry options/order; negative counts; sum tallies > ballots_cast; missing scale/normalization for score; malformed IRV rounds or Condorcet rankings → VALIDATE error; run goes down Invalid path.
WTA interplay: if later allocation_method=winner_take_all, ensure involved units have magnitude=1 (checked elsewhere but affects acceptance).
12) Test checklist (must pass)
Schema validates for the selected ballot_type; tally sanity passes in all units.
Baseline 6A cases reproduce expected allocations (PR 1–2–3–4; WTA winner D; LR/D’Hondt/Sainte-Laguë → 3–2–2).
```
