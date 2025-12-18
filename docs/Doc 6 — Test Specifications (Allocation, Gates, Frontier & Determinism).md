# **Doc 6A — Test Harness & Allocation Correctness (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **official test harness**, **fixtures format**, and the **allocation-correctness** test set. This part ensures engines implement 4A flow correctly with no reliance on ties, frontier, or presentation toggles. Determinism, ties, gates, and frontier have focused suites in **6B/6C**.

Outcomes from these tests must be **byte-identical** across OS/arch when inputs and ParameterSet match.

---

## **2\) Test harness (normative)**

### **2.1 Invocation**

Each test case supplies three canonical JSON files and a directory for outputs:

vm\_cli \--registry \<case\>/registry.json \\  
      \--tally    \<case\>/tally.json \\  
      \--params   \<case\>/params.json \\  
      \--out      \<run-dir\>

No network. Exit codes per Doc 3A/5A.

### **2.2 Required outputs per test**

* `result.json` (canonical; Doc 1A §2.1, §4.4)

* `run_record.json` (canonical; Doc 1A §4.5)

* Optional `frontier_map.json` **only** if frontier is enabled **and** VM-VAR-034=true (Doc 1A §4.6)

### **2.3 Verification workflow**

Implement the following assertions for every case:

1. **Canonical form**

   * UTF-8, LF, sorted keys; arrays ordered per Doc 1A §5.

2. **IDs & hashes**

   * `result_id == "RES:" + sha256(canonical(result.json))`

   * `run_id` suffix (after timestamp and hyphen) equals sha256(canonical(run\_record.json))

   * If `frontier_map.json` exists: `frontier_id == "FR:" + sha256(canonical(frontier_map.json))`

3. **Referential integrity** (Doc 1B)

4. **FID integrity**

   * Recompute FID from the **Included** set (Annex A) and confirm it equals both `Result.formula_id` and `RunRecord.formula_id`.

5. **Vars echo**

   * `RunRecord.vars_effective` lists **all** outcome-affecting VM-VARs actually used; 032–035 and 060–062 may appear but are **non-FID**.

6. **No RNG**

   * For all 6A cases: `RunRecord.determinism.rng_seed` MUST be absent; `RunRecord.ties[]` MUST be empty.

Annex B (Canonical Test Pack) provides machine-readable “expected” for allocations, labels, and selected aggregates.

---

## **3\) Fixture format (normative)**

### **3.1 `registry.json`**

See Doc 1B/1C. Must contain:

* Stable `unit_id`, `option_id`

* Unique `order_index` per unit (determinism primitive)

### **3.2 `tally.json`**

* Non-negative integers; `sum(option.votes) ≤ totals.valid_ballots`

* Units/options aligned to the Registry

### **3.3 `params.json` (ParameterSet)**

* `schema_version`

* `vars` map with explicit values for **all** VM-VARs listed “Included” in Annex A **except** tie/advanced/frontier where each 6A case **disables** them:

  * **Ties**: `VM-VAR-050="deterministic_order"` (051 reserved; 052 ignored)

  * **Frontier**: `VM-VAR-040="none"` (047–049 irrelevant)

  * **Presentation** (optional, non-FID): `060=55`, `061="dynamic_margin"`, `062="auto"`

  * **034/035**: `frontier_map_enabled=false`, `sensitivity_analysis_enabled=false`

---

## **4\) Allocation-correctness test set (normative)**

Each case includes **expected allocations** (unit-ordered; options by `order_index`) and **expected labels** (presentation; informative). Hashes for canonical artifacts are provided in Annex B.

### **VM-TST-101 — Simple 2-option majority**

* **Intent:** Baseline allocation; no ties; no gates triggered.

* **Params:** `050="deterministic_order"`, `040="none"`

* **Expect:** Allocations match vote shares; label depends on 060/061 but is non-FID.

### **VM-TST-102 — Three options, strict registry order**

* **Intent:** Confirms allocations preserve `order_index` in arrays.

* **Expect:** Output `allocations[]` appear in Registry option order even if votes are descending/ascending differently.

### **VM-TST-103 — Zero-vote minor option**

* **Intent:** Zero votes do not create ties or invalid states by themselves.

* **Expect:** Minor option present with `votes=0`; no ties; valid unit.

### **VM-TST-104 — Multiple units, deterministic iteration**

* **Intent:** Confirms units processed in ascending `unit_id`.

* **Expect:** `Result.units[]` sorted by `unit_id`; per-unit allocations correct.

### **VM-TST-105 — Rounding policy application**

* **Intent:** Verifies family constants **001…007** (and **073** if used) drive rounding deterministically.

* **Expect:** Aggregates and per-unit shares match fixtures to engine precision.

### **VM-TST-106 — Large counts stability**

* **Intent:** 64-bit safety and stable arithmetic with big tallies.

* **Expect:** Correct totals/shares; canonicalization intact.

### **VM-TST-107 — Missing option in tally is not auto-created**

* **Intent:** Enforce Doc 1B referential integrity.

* **Expect:** **Validation error** (exit 2), not a coerced zero row.

### **VM-TST-108 — Order index uniqueness**

* **Intent:** Duplicate `order_index` in a unit is invalid.

* **Expect:** **Validation error** (exit 2\) with code `E-DR-ORD-UNIQ`.

### **VM-TST-109 — Sum of votes \> valid\_ballots**

* **Intent:** Sanity check per Doc 1B.

* **Expect:** **Validation error** (exit 2\) `E-BT-SUM`.

### **VM-TST-110 — FID recomputation lock**

* **Intent:** Confirms FID is independent of 060–062 and section ordering.

* **Params:** Change `061` (`fixed` ↔ `dynamic_margin`) **only**.

* **Expect:** **Same FID**, identical `Result` allocations; only label may differ.

---

## **5\) Per-case acceptance template (normative)**

For every **passing** case (those not designed to fail validation):

1. **IDs & hashes** verified (2.3).

2. **Allocations** exactly match Annex B fixture (unit & option ordering).

3. **Labels** match fixture (informative; presentation-only).

4. **RunRecord**

   * `vars_effective` includes all outcome-affecting VM-VARs

   * `determinism.tie_policy="deterministic_order"`; no `rng_seed`; `ties=[]`

   * `inputs.*_sha256` match canonical inputs

   * `nm_digest.nm_sha256` present and consistent

5. **FID** recomputed equals both artifacts’ `formula_id`.

For **validation-error** cases, assert correct **exit code 2** and the specific error token(s).

---

## **6\) Harness conformance (producer & verifier)**

* **Producer** (engine) MUST emit only canonical artifacts and exit with codes defined in Doc 3A/5A.

* **Verifier** (test runner) MUST:

  * Re-canonicalize artifacts before hashing.

  * Recompute FID from the Included set (Annex A).

  * Compare allocations and labels against Annex B fixtures.

  * Enforce ordering rules strictly (Doc 1A §5).

---

## **7\) Notes & boundaries**

* 6A cases **never** require RNG or frontier; those are covered in **6C** and **6B** respectively.

* Presentation variables (032–035, 060–062) may vary without changing FID; 6A uses fixed defaults for consistency.

* All fixtures are provided machine-readable in **Annex B — Canonical Test Pack**.

*End Doc 6A.*

# **Doc 6B — Gates & Frontier Test Suite (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **official tests** for **unit gating** (Doc 4B) and the **frontier model** (Doc 4C §2). These tests verify:

* Gate **order**, **semantics**, and **reason recording**.

* Protected/override logic (**045**, **029**, **030**, **031**).

* Frontier enablement (**040–042**), advanced tuning (**047–049**), and **FrontierMap** emission (**034**).

This suite is **normative**. Fixtures live in **Annex B — Canonical Test Pack**. Determinism, hashing, and FID rules per Docs **1A/3A/3B**.

---

## **2\) Harness & invariants (reuse of 6A §2–§3)**

* Invocation, required outputs, verification workflow, and canonical JSON rules are identical to **Doc 6A §2–§3**.

* **Gate failures are not schema errors**: a unit may become `Invalid` but the run still **succeeds** (exit `0`).  
   Only schema/ref/order violations (Doc 1B) yield exit `2`.

**ParameterSet conventions for this suite (unless a case overrides):**

* **Ties**: `VM-VAR-050="deterministic_order"`; `VM-VAR-052` ignored (no RNG).

* **Frontier**: varies per case (`VM-VAR-040`), with **047–049** as needed.

* **Presentation** (non-FID): `060=55`, `061="dynamic_margin"`, `062="auto"`.

* **034 frontier\_map\_enabled** and **035 sensitivity\_analysis\_enabled** set per case.

---

## **3\) Test cases — Gates (020–031, 045, 029\)**

### **VM-TST-201 — Minimum turnout gate (eligibility)**

**Intent:** An eligibility threshold forces `Invalid`.  
 **Setup:** Set a turnout-like threshold (e.g., `VM-VAR-020`) above the unit’s value.  
 **Expect:**

* `Result.units[i].label="Invalid"`, `allocations=[]`.

* `RunRecord.summary.units[i].reasons` contains a token for **020**.

* Exit `0`; hashes/FID valid.

### **VM-TST-202 — Multiple eligibility gates; reason ordering**

**Intent:** Multiple failing gates record **all** reasons in **ascending VM-VAR ID**.  
 **Setup:** Make **020** and **022** both fail.  
 **Expect:** `reasons=["VM-VAR-020:…","VM-VAR-022:…"]` in that order; unit `Invalid`.

### **VM-TST-203 — Symmetry exceptions (029) narrow override**

**Intent:** **029** selectively exempts a unit from an eligibility failure.  
 **Setup:** Threshold via **020** would fail; `VM-VAR-029` lists this unit.  
 **Expect:** Unit **valid**; `applied_exceptions=["VM-VAR-029:<selector>"]`; no **020** reason recorded.

### **VM-TST-204 — Eligibility override list (030) before exceptions**

**Intent:** **030** applies **before** **029** per fixed precedence.  
 **Setup:** Same as 203, but also set `VM-VAR-030` to `exclude` the unit.  
 **Expect:** Unit `Invalid`; `reasons` includes **030** token; **029** recorded under `applied_exceptions` only if applicable; precedence documented.

### **VM-TST-205 — Integrity floor (031) cannot be bypassed**

**Intent:** **031** invalidates a unit even if 029/030/045 would allow eligibility.  
 **Setup:** Integrity KPI below **031**; other gates pass or are bypassed.  
 **Expect:** `Invalid`; `reasons` includes **031**; no protected bypass allowed.

### **VM-TST-206 — Protected-area override (045=allow) bypasses eligibility only**

**Intent:** **045=allow** may bypass an **eligibility** failure but never sanity/integrity.  
 **Setup:** Mark unit `protected_area=true`; make an eligibility gate fail.  
 **Expect:** Unit **valid**; `protected_bypass=true`; **no** reason for that eligibility gate.

### **VM-TST-207 — Protected-area with integrity floor still invalid**

**Intent:** **045** cannot bypass **031**.  
 **Setup:** As 206, but integrity KPI below **031**.  
 **Expect:** `Invalid`; `reasons` includes **031**; `protected_bypass` absent/false.

### **VM-TST-208 — Frontier pre-check failure recorded as validity reason**

**Intent:** Missing required inputs for frontier triggers a **validity** failure token, not a schema error.  
 **Setup:** Enable frontier (**040≠"none"**), remove a required metric.  
 **Expect:** Unit `Invalid`; `reasons` includes `"frontier_missing_inputs"` (ordered after VM-VAR tokens); exit `0`.

---

## **4\) Test cases — Frontier core (040–042) & diagnostics (034)**

### **VM-TST-210 — Frontier disabled**

**Intent:** With `VM-VAR-040="none"`, frontier does not run.  
 **Setup:** Any inputs; `034=true`.  
 **Expect:** No `frontier_map.json` emitted; results unchanged; FID unaffected.

### **VM-TST-211 — Frontier banded (040), cut (041), strategy (042)**

**Intent:** Baseline frontier gating is deterministic and reflected in diagnostics.  
 **Setup:** `040="banded"`, `041=<cut>`, `042="apply_on_entry"`, `034=true`.  
 **Expect:** `frontier_map.json` present; each unit has `band_met`/`band_value` per fixture; `Result` matches expected gating effect.

### **VM-TST-212 — FrontierMap emission toggle (034)**

**Intent:** **Only** toggles presence of the file; allocations & FID unchanged.  
 **Setup:** Same as 211 but run twice with `034=true` then `034=false`.  
 **Expect:** Identical `Result` and FID; `frontier_map.json` emitted only when `034=true`.

### **VM-TST-213 — Advanced window (047) affects band\_met at margins**

**Intent:** **047** expands/contracts the effective band around **041**.  
 **Setup:** Units near the cut; compare `047=0.00` vs `047=0.02`.  
 **Expect:** Borderline units flip `band_met` exactly as fixtures specify; deterministic across runs.

### **VM-TST-214 — Backoff policy (048) softens/hardens borderline**

**Intent:** **048** resolves edges; compare `none` vs `soften` vs `harden`.  
 **Setup:** Units at the threshold.  
 **Expect:** `band_met` differences per fixture; order and hashes stable.

### **VM-TST-215 — Strictness (049) multiplies effects**

**Intent:** **049** coarsely strengthens/weakens 047/048.  
 **Setup:** Fix 047/048; vary `049` between `strict` and `lenient`.  
 **Expect:** Predictable, documented change in `band_met`; allocations follow accordingly.

### **VM-TST-216 — Ladder mode uses autonomy map (046)**

**Intent:** In ladder mode, autonomy package selection is deterministic.  
 **Setup:** `040="ladder"`, define **046** map; provide tallies that traverse steps.  
 **Expect:** Selected packages match fixture; stable across OS/arch.

---

## **5\) Per-case acceptance template (normative)**

For every **passing** case (non-schema-failure):

1. **Canonical form & IDs** verified (as in 6A §2.3).

2. **Gate behavior**

   * Units `Invalid` when any gate fails; `allocations=[]`; `label="Invalid"`.

   * `reasons[]` complete and ordered: **ascending VM-VAR ID**, then symbolic (e.g., `"frontier_missing_inputs"`).

   * `protected_bypass` appears **only** when 045=allow bypassed **eligibility**.

   * `applied_exceptions[]` lists **029** matches deterministically.

3. **Frontier**

   * If `040!="none"` and `034=true`: `frontier_map.json` exists; entries match fixture (`band_met`, `band_value`); units ordered by `unit_id`.

   * If `034=false`: **no** `frontier_map.json`; `Result`/FID unchanged (compare to `034=true` run).

4. **FID integrity**

   * Changing **034** or any presentation variables (060–062) does **not** change FID.

   * Changes to outcome-affecting frontier/gate variables **do** produce the expected allocation differences and FID remains consistent with the Included set.

---

## **6\) Conformance checklist (6B)**

* **C-6B-ORDER**: Reasons ordered by VM-VAR ID then symbolic; unit arrays by `unit_id`.

* **C-6B-PROT**: 045 can bypass **eligibility** only; cannot bypass sanity or **031**.

* **C-6B-PREC**: 030 precedence over 029 is honored and recorded.

* **C-6B-FRONTIER**: 040–042 (+047–049) produce deterministic `band_met`; invalid config recorded as `"frontier_missing_inputs"`, not a schema error.

* **C-6B-FRMAP**: `frontier_map.json` emitted **only** if `034=true` and frontier executed; FID unaffected by 034\.

---

## **7\) Notes**

* Keep ties **off** in this suite (`050="deterministic_order"`) to isolate gate/frontier behavior.

* RNG and tie behavior are exercised in **Doc 6C — Determinism & Ties** (next).

*End Doc 6B.*

# **Doc 6C — Determinism & Ties Test Suite (Updated, Normative)**

## **1\) Purpose & scope**

Validates **determinism** and **tie resolution** behavior per Docs **3A**, **4C**, **5A–5C**:

* Reproducibility with identical inputs \+ ParameterSet (incl. seed).

* Correct application of **`tie_policy`** (**VM-VAR-050**) and **`tie_seed`** (**VM-VAR-052**).

* No RNG used unless `tie_policy="random"` **and** a tie actually occurs.

* Canonical logging of tie events in `RunRecord.ties[]`.

* FID behavior: **050 affects FID; 052 does not** (seed is non-FID).

Fixtures live in **Annex B — Canonical Test Pack**. Canonical JSON, hashing, and exit codes per **Docs 1A, 3A, 5A**.

---

## **2\) Harness & invariants (reuse from 6A §2–§3)**

Invocation, required outputs, verification workflow, canonical form, IDs/hashes, FID recomputation, and `vars_effective` checks are identical to **Doc 6A §2–§3**. Differences for this suite:

* `tie_policy` varies by case.

* Some cases require multiple runs (same/different seeds).

* Frontier disabled unless a case states otherwise (`VM-VAR-040="none"`).

Common ParameterSet defaults unless overridden:

050: varies by case  
052: 424242 (or as specified)  
040: "none"  
034: false  
035: false  
060: 55  
061: "dynamic\_margin"  
062: "auto"

---

## **3\) Test cases — determinism (no RNG path)**

### **VM-TST-301 — Full-run reproducibility (no RNG)**

**Intent:** Identical runs produce byte-identical artifacts.  
 **Setup:** `050="deterministic_order"`. Execute the same case twice.  
 **Expect:** `result.json`, `run_record.json` identical; same `result_id`, `run_id` (except timestamp prefix is same format), same `formula_id`; `ties=[]`; `rng_seed` **absent**.

### **VM-TST-302 — Deterministic tie by order\_index**

**Intent:** Two-way and three-way ties resolved by **Registry order**.  
 **Setup:** Construct ties; `050="deterministic_order"`.  
 **Expect:** Within tied groups, ascending `order_index` then `option_id`; `ties=[]`; `rng_seed` **absent**.

### **VM-TST-303 — Status quo policy path**

**Intent:** Uses the family’s status-quo rule; no RNG.  
 **Setup:** Ties exist; `050="status_quo"`.  
 **Expect:** Allocations follow the family rule; `ties[]` entries present with `"policy":"status_quo"`; no `seed` field; `rng_seed` **absent**.

---

## **4\) Test cases — random ties (RNG path)**

### **VM-TST-304 — Random tie with fixed seed (2-way)**

**Intent:** Seeded random tie is reproducible.  
 **Setup:** 2-way tie; `050="random"`, `052=424242`. Run twice.  
 **Expect:** Same permutation of the tied pair both runs; `ties[0].policy="random"`, `ties[0].seed=424242`; `RunRecord.determinism.rng_seed=424242`.

### **VM-TST-305 — Random tie with fixed seed (3-way)**

**Intent:** k-way permutation is stable and sorted by `(draw, option_id)`.  
 **Setup:** 3-way tie; `050="random"`, `052=424242`.  
 **Expect:** Tied subset ordering exactly matches Annex B fixture (canonical RNG draws); `rng_seed=424242`.

### **VM-TST-306 — Seed variation changes outcome, FID unchanged**

**Intent:** Outcomes may differ across seeds; FID stays the same.  
 **Setup:** Same input; run A: `052=111111`, run B: `052=222222`; `050="random"`.  
 **Expect:**

* Allocations differ **only** where ties exist.

* **Same `formula_id`** in both runs (seed excluded from FID).

* `RunRecord.determinism.rng_seed` equals the chosen seed for each run.

### **VM-TST-307 — Random policy but no ties ⇒ no RNG use**

**Intent:** Seed recorded only if a random tie actually occurred.  
 **Setup:** No ties; `050="random"`, `052=999`.  
 **Expect:** `ties=[]`; `RunRecord.determinism.rng_seed` **absent**; artifacts identical to a run with `050="deterministic_order"`.

### **VM-TST-308 — Multiple tie events consume exact draws**

**Intent:** Exactly **k** draws per k-way tie; subsequent ties use subsequent draws.  
 **Setup:** Two units, first has 3-way tie, second has 2-way tie; `050="random"`, `052=424242`.  
 **Expect:** Permutations match Annex B (which encodes the canonical RNG sequence). A regression in draw counts will flip the second unit’s permutation and fail the fixture.

### **VM-TST-309 — Repeated ties within a unit**

**Intent:** Multiple independent tie resolutions within the **same** unit consume draws in event order.  
 **Setup:** One unit, two separate tie points in the algorithm; `050="random"`, `052=424242`.  
 **Expect:** First tie uses first **k1** draws; second tie uses next **k2** draws; permutations match Annex B.

### **VM-TST-310 — Mixed policies across runs don’t collide**

**Intent:** Changing policy alters FID; seed remains non-FID.  
 **Setup:** Run A: `050="deterministic_order"`, Run B: `050="random"` (same inputs).  
 **Expect:** **Different `formula_id`** across runs (policy included in FID). `rng_seed` present only for Run B if a random tie occurs.

### **VM-TST-311 — Random ties with frontier disabled/enabled (no interference)**

**Intent:** Frontier configuration doesn’t change RNG usage rules.  
 **Setup:** Same tie case; (A) `040="none"`, (B) `040="banded";034=true` (frontier diagnostics on). Both `050="random"`, `052=424242`.  
 **Expect:** Identical allocations and tie permutations in A and B; B may emit `frontier_map.json`. `formula_id` may differ only if frontier variables are outcome-affecting in the case; seed handling unchanged.

### **VM-TST-312 — Invalid unit ⇒ no tie resolution**

**Intent:** Gates pre-empt ties.  
 **Setup:** Unit fails a validity gate (e.g., `031`), while a tie would otherwise occur. `050="random"`.  
 **Expect:** Unit is `Invalid`; `allocations=[]`; **no** tie event for that unit; `rng_seed` only present if some **other** unit had a random tie.

---

## **5\) Per-case acceptance template (normative)**

For every **passing** case (non-schema-failure):

1. **Canonical form & IDs** verified (as in 6A §2.3).

2. **FID integrity**

   * Changing **050** (policy) ⇒ FID changes as per Included set.

   * Changing **052** (seed) alone ⇒ **FID unchanged**.

3. **Tie behavior**

   * When `050="random"` and a tie occurs: `RunRecord.determinism.rng_seed = <052>`, and each `ties[]` entry has `"policy":"random"` and `"seed":<052>`.

   * When no random tie occurred (even if `050="random"`): `rng_seed` **absent**; `ties=[]`.

   * Tied subsets are ordered by `(draw, option_id)` using the canonical RNG; counts of draws match **exactly k per tie**.

4. **Determinism**

   * Same inputs \+ ParameterSet (incl. seed) ⇒ **byte-identical** artifacts across repeated runs.

   * Arrays preserve canonical ordering (units by `unit_id`; allocations by `order_index`).

5. **Scope**

   * Ties only resolved for **valid** units (after gates), post-frontier (Doc 4A S5/S3).

---

## **6\) Conformance checklist (6C)**

* **C-6C-RNG-ONLY-WHEN-NEEDED**: RNG used **only** when `050="random"` **and** a tie exists.

* **C-6C-SEED-ECHO**: `rng_seed` echoed in `RunRecord` **iff** any random tie occurred; value equals **052**.

* **C-6C-DRAWS-K**: Exactly **k** draws per k-way tie; permutations match Annex B.

* **C-6C-FID-SEED**: Changing **052** alone does **not** change FID; changing **050** does.

* **C-6C-REPRO**: Re-running with the same seed yields byte-identical artifacts.

---

## **7\) Notes**

* Annex B fixes the **RNG profile** and provides expected permutations; engines must implement the same RNG to pass.

* `VM-VAR-051` remains **reserved**; there is no test that sets it—engines should ignore unknown/non-Included keys for FID while still enforcing Annex A’s Included list.

*End Doc 6C.*

