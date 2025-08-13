````md
Pre-Coding Essentials (Component: fixtures/annex_b/part_0/ballots.json, Version/FormulaID: VM-ENGINE v0) — 71/89

1) Goal & Success
Goal: Ship a **canonical BallotTally fixture** for Part-0 that exactly matches the selected ballot type and the registry’s options/order, so TABULATE can run deterministically.
Success: JSON validates; per-unit tallies are sane; option IDs line up with Registry; bytes canonicalize (UTF-8, LF, sorted keys) and produce stable hashes across OS/arch.

2) Scope
In scope: One tally dataset for **one** ballot type (plurality | approval | score | ranked_irv | ranked_condorcet).
Out of scope: Parameters (separate fixture), algorithms, report wording.

3) Inputs → Outputs
Input file: `fixtures/annex_b/part_0/ballots.json`.
Consumed by: LOAD → VALIDATE (tally sanity) → TABULATE (type-specific) → ALLOCATE/…  
Output of TABULATE: `UnitScores` per unit (deterministic BTree order).

4) Top-level Shape (common)
```json
{
  "id": "TLY:part0",
  "schema_version": "1",
  "label": "Part 0 BallotTally",
  "ballot_type": "approval | plurality | score | ranked_irv | ranked_condorcet",
  "units": [ /* type-specific blocks (see §8) */ ]
}
````

Rules:

* `id` is stable; `ballot_type` must match Params VM-VAR-001.
* Every `units[*].unit_id` must exist in the Registry.
* For each unit: `valid_ballots = ballots_cast - invalid_or_blank` (must be ≥ 0).

5. Variables (read by engine in later stages)

* VM-VAR-001 ballot\_type (must match this file).
* Score only: VM-VAR-002/003 scale\_min/max; VM-VAR-004 normalization.
* IRV only: VM-VAR-006 exhaustion policy (engine uses reduce\_continuing\_denominator).
* Condorcet only: VM-VAR-005 completion rule (Schulze/Minimax).
* Gates note: Approval majority uses **approval rate** = approvals\_for\_change / **valid\_ballots** (fixed rule); VM-VAR-007 may widen **gate** denominators (not tabulation).

6. Functions
   N/A (fixture). Loader → typed structs; Validator → sanity checks; Tabulation → per family.

7. How the engine consumes it

* VALIDATE: per-unit tally sanity + option/ID cross-refs + score caps (from VM-VAR).
* TABULATE: builds `UnitScores` from the natural tallies (no floats, no RNG).
* Allocation & gates later use integers/ratios derived from these tallies.

8. Type-Specific Unit Shapes (author exactly one family)

**A) Plurality**

```json
{
  "unit_id": "U:...",
  "ballots_cast": 100,
  "invalid_or_blank": 0,
  "votes": { "OPT:A": 10, "OPT:B": 20, "OPT:C": 30, "OPT:D": 40 }
}
```

Sanity: Σvotes ≤ valid\_ballots.

**B) Approval**

```json
{
  "unit_id": "U:...",
  "ballots_cast": 100,
  "invalid_or_blank": 0,
  "approvals": { "OPT:Change": 55, "OPT:SQ": 60 }
}
```

Sanity: for every option, approvals\_opt ≤ valid\_ballots. (Σapprovals may exceed valid\_ballots.)

**C) Score**

```json
{
  "unit_id": "U:...",
  "ballots_cast": 100,
  "invalid_or_blank": 0,
  "ballots_counted": 100,                  // = valid_ballots when all counted
  "score_sum": { "OPT:A": 210, "OPT:B": 340, "OPT:C": 450, "OPT:D": 560 },
  "scale": { "min": 0, "max": 5 },         // mirrors VM-VAR-002/003
  "normalization": "off | linear"          // mirrors VM-VAR-004
}
```

Sanity: for each option, score\_sum\_opt ≤ ballots\_counted \* max. If ballots\_counted==0 then all sums must be 0.

**D) Ranked IRV**
*Compressed groups of identical rankings; order = top→down.*

```json
{
  "unit_id": "U:...",
  "ballots_cast": 100,
  "invalid_or_blank": 0,
  "ballots": [
    { "ranking": ["OPT:A","OPT:C","OPT:B","OPT:D"], "count": 28 },
    { "ranking": ["OPT:B","OPT:D"], "count": 27 },
    { "ranking": ["OPT:C","OPT:B"], "count": 25 },
    { "ranking": ["OPT:D"], "count": 20 }
  ]
}
```

Sanity: Σcount = valid\_ballots. Unknown/duplicate IDs in a ranking are invalid.

**E) Ranked Condorcet**

```json
{
  "unit_id": "U:...",
  "ballots_cast": 100,
  "invalid_or_blank": 0,
  "ballots": [
    { "ranking": ["OPT:A","OPT:C","OPT:B","OPT:D"], "count": 28 },
    { "ranking": ["OPT:B","OPT:D"], "count": 27 },
    { "ranking": ["OPT:C","OPT:B"], "count": 25 },
    { "ranking": ["OPT:D"], "count": 20 }
  ]
}
```

Sanity: Σcount = valid\_ballots. Pairwise abstention for unranked pairs is implied.

9. Determinism & Numeric Rules

* Option keys must be **OptionId**s from Registry; missing options are treated as 0 in tabulators (but unknown extras are an error).
* Engine iterates options by canonical order (order\_index, OptionId); stores tallies in BTreeMap.
* Pure integers; no floats; round-half-even is used later only where spec allows.

10. Edge Cases & Failure Policy

* Negative or non-integer counts ⇒ schema/validate error.
* Plurality: Σvotes > valid\_ballots ⇒ error.
* Approval: any approvals\_opt > valid\_ballots ⇒ error; valid\_ballots=0 with non-zero approvals ⇒ error.
* Score: caps violated (sum > ballots\_counted\*max) ⇒ error; missing scale/normalization fields ⇒ error.
* Ranked: Σgroup counts ≠ valid\_ballots, unknown option IDs, or malformed rankings ⇒ error.

11. Test Checklist (must pass)

* Schema validation for the chosen `ballot_type`.
* Registry alignment: every option key exists; no extras.
* Tally sanity holds (per-family rules above).
* Baselines reproduce Part-0 expectations:

  * Approval + Sainte-Laguë with m=10 over {10,20,30,40} ⇒ seats 1/2/3/4.
  * Plurality + WTA m=1 over {10,20,30,40} ⇒ winner D gets 100%.
  * PR convergence (shares 34/33/33, m=7) ⇒ D’Hondt/Sainte-Laguë/LR all 3/2/2.
* Canonicalization: shuffled key orders re-serialize to identical canonical bytes & SHA-256.

12. Authoring Notes

* Provide **all** options present in Registry; omit none (zeros allowed).
* Keep IDs/strings NFC; prefer LF newlines; avoid trailing spaces.
* If multiple ballot families are needed for different scenarios, ship separate files; this fixture is for **one** family only.

```
```
