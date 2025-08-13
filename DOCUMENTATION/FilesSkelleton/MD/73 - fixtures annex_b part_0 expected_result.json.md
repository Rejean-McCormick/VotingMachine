````md
Pre-Coding Essentials (Component: fixtures/annex_b/part_0/expected_result.json, Version/FormulaID: VM-ENGINE v0) — 73/89

1) Goal & Success
Goal: Ship a **single, minimal oracle** for the Part-0 run that lets any engine compare its `Result` to fixed expectations (gates, allocations, final label) and, once certified, lock the canonical SHA-256 of the `Result`.
Success: Engines match these fields exactly on all OS/arch; after certification, `expected_canonical_hash` equals the SHA-256 of the canonical `Result` bytes (UTF-8, sorted keys, LF, UTC).

2) Scope
In scope: A compact JSON with only verifiable outcomes (no duplication of full `Result`).  
Out of scope: Rendering concerns, pretty text, or any recomputation.

3) Inputs → Outputs
Inputs (for comparison): Engine-produced `Result` (RES:…) from running Part-0 fixtures.  
Output: `expected_result.json` consumed by the test harness to assert correctness + (after certification) exact canonical hash.

4) What to assert (keep it minimal but decisive)
- **Legitimacy gates**: quorum + (if enabled) majority and double-majority, with raw numerators/denominators and thresholds.
- **Final label**: Decisive | Marginal | Invalid and the short reason.
- **Allocation**: enough structure to unambiguously assert the seat/power outcome. For PR, either per-unit allocations or national sums; for WTA, per-unit winner (100% power).
- **Optional**: IRV/Condorcet winner per unit if a ranked method is used (IDs only), tie policy note when relevant.
- **Canonical lock**: `expected_canonical_hash` (null until certification).

5) Canonical JSON shape (author exactly this)
```jsonc
{
  "id": "EXP:part0",

  "based_on": {
    "manifest_id": "MAN:part0",           // optional, helpful for traceability
    "registry_id": "REG:…",               // echo IDs if known (optional)
    "parameter_set_id": "PS:…",
    "ballot_tally_id": "TLY:…"
  },

  "expected": {
    "gates": {
      "quorum": {
        "pass": true,
        "turnout": { "ballots_cast": 0, "eligible_roll": 0 }, // raw counts
        "threshold_pct": 50
      },
      "majority": {
        "present": true,
        "pass": true,
        "support": {
          // For approval ballots, this MUST be approval rate:
          // approvals_for_change / valid_ballots (not share of approvals)
          "numerator": 0,
          "denominator": 0,
          "denominator_policy": "valid_ballots" // or "valid_plus_blank" or "approval_rate"
        },
        "threshold_pct": 55
      },
      "double_majority": {
        "present": false
        /* when present:
        "pass": true,
        "national_support": { "numerator": 0, "denominator": 0 },
        "family_support":   { "numerator": 0, "denominator": 0 },
        "thresholds_pct": { "national": 55, "regional": 55 }
        */
      }
    },

    "label": { "value": "Decisive", "reason": "≥ threshold and no frontier flags" },

    "allocation": {
      // Choose ONE of the following assertion strategies (keep unused key absent):
      "per_unit": {
        // For WTA or small PR runs; UnitId → outcome
        "U:0001": {
          "method": "wta|dhondt|sainte_lague|largest_remainder",
          "magnitude": 1,
          "seats_or_power": { "OPT:D": 100 }   // WTA: 100% power to winner
        },
        "U:0002": {
          "method": "sainte_lague",
          "magnitude": 10,
          "seats": { "OPT:A": 1, "OPT:B": 2, "OPT:C": 3, "OPT:D": 4 } // sums to magnitude
        }
      },

      "national_totals_by_option": {
        // Alternative for PR checks when per-unit is verbose
        "OPT:A": 1, "OPT:B": 2, "OPT:C": 3, "OPT:D": 4
      }
    },

    "ranked_outcomes": {
      // Optional: only if ranked ballot types are used in Part-0
      "U:0003": { "winner": "OPT:B", "method": "irv" },
      "U:0004": { "winner": "OPT:C", "method": "condorcet", "completion": "schulze" }
    }
  },

  // Set to the canonical SHA-256 of the produced Result once certified.
  // Keep null until the canonical engine run is frozen.
  "expected_canonical_hash": null
}
````

6. Field rules & comparisons

* Use **raw counts** for gate ratios; harness computes the percent for display but compares integers.
* **Approval ballots**: `denominator_policy` MUST be `"approval_rate"` semantics: approvals\_for\_change / valid\_ballots.
* For PR seat checks, totals **must equal Unit.magnitude** (or 100 for WTA power).
* Omit keys that don’t apply (e.g., `"per_unit"` vs `"national_totals_by_option"`); don’t include both unless you intend to validate both.
* All IDs are engine IDs (e.g., `OPT:…`, `U:…`), not display names.

7. Determinism & canonicalization

* This file is a fixture (no hashing here), but the **hash you record later** is the SHA-256 of the engine’s **canonical** `Result` (UTF-8, **LF**, **sorted keys**, UTC timestamps).
* Comparisons must be order-insensitive for maps but sensitive for numeric values.

8. Edge cases & failure policy

* If any gate value or label mismatches → test fail with the JSON Pointer(s) of mismatched fields.
* If `expected_canonical_hash` is non-null and differs from the engine’s `Result` hash → flag **nondeterminism or non-canonical encoding** in the engine build; re-check sorted keys/LF/UTC and stable ordering.

9. Authoring tips

* Keep numbers **integers**; avoid embedding formatted percentages to prevent rounding debates.
* Only assert what you truly need; smaller oracles are easier to maintain yet still decisive.
* When Part-0 covers multiple ballot/allocation families, split per scenario (e.g., `expected_result_approval.json`, `expected_result_plurality_pr.json`) to keep each oracle minimal.

10. Test checklist (must pass)

* Gates (quorum/majority/DM as configured) match exactly via raw numerators/denominators and thresholds.
* Final label matches (`Decisive|Marginal|Invalid`) and reason string equals.
* Allocation assertions hold (per-unit or national sums).
* After certification, `expected_canonical_hash` equals the canonical `Result` hash on Windows/macOS/Linux.

```
```
