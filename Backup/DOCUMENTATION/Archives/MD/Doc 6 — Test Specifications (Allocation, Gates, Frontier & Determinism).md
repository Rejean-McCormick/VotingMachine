# **Doc 6A — Tests: Conventions & Core Allocation**

**Scope:** Global test conventions and the three core allocation checks for PR and WTA.  
 **Determinism:** Option order is **A \> B \> C \> D** (deterministic order; lower `Option.order_index` wins under deterministic ties).

---

## **1\) Global Conventions (apply unless overridden in a test)**

* **Options (and order):** A, B, C, D with fixed order **A \> B \> C \> D**.

  * *Refs:* `Option.order_index` (Doc 1B); deterministic ties (Doc 4C §3; VM-VAR-051/052 policy context in Doc 4C).

* **Hierarchy for tests:** Single national district unless specified.

* **Defaults (from Doc 2A):**

  * `VM-VAR-001 ballot_type = approval`

  * `VM-VAR-010 allocation_method = proportional_favor_small`

  * `VM-VAR-012 pr_entry_threshold_pct = 0`

  * `VM-VAR-020 quorum_global_pct = 50`

  * `VM-VAR-022 national_majority_pct = 55`, `VM-VAR-023 regional_majority_pct = 55`

  * `VM-VAR-024 double_majority_enabled = on`, `VM-VAR-025 symmetry_enabled = on`

  * `VM-VAR-030 weighting_method = population_baseline`, `VM-VAR-031 aggregate_level = country`

  * `frontier_mode = none` (no mapping in 6A tests)

  * Tie policy: `status_quo` (only relevant if a tie appears; none in 6A)

* **Rounding/percent display:** internal comparisons use **round half to even**; reporting uses one decimal (Doc 4A, Doc 7A).

---

## **2\) VM-TST-001 — Happy PR baseline**

**Goal.** Lock the baseline Sainte-Laguë behavior.

* **Setup.** One national district, **m \= 10**. Approval tallies (arbitrary units):  
   A \= 10, B \= 20, C \= 30, D \= 40\.

* **Params (delta).** None (defaults apply).

* **Expected.** Seats **A/B/C/D \= 1/2/3/4**. Label: **Decisive**.

* **Accept.** Engine returns exactly the vector **1–2–3–4**; sums to **10**.

* **Cross-refs.**

  * VM-VAR: 001 (approval), 010 (`proportional_favor_small`), 012 (threshold=0).

  * ALG: Doc 4A §2.2 (approval tabulation), Doc 4B §2.3 (Sainte-Laguë).

  * FUN: VM-FUN-003 (TabulateUnit), VM-FUN-004 (AllocateUnit).

  * DB: `Result.UnitBlock.seats_or_power`, `Result.Aggregates`.

---

## **3\) VM-TST-002 — WTA wipe-out**

**Goal.** Confirm winner-take-all semantics and the `m=1` constraint.

* **Setup.** One national unit, **m \= 1**. Plurality votes:  
   A \= 10, B \= 20, C \= 30, D \= 40\.

* **Params (delta).** `VM-VAR-001 = plurality`, `VM-VAR-010 = winner_take_all`.

* **Expected.** **D** wins **100%** power (others 0). Label: **Decisive**.

* **Accept.** Winner **D**; total power=100%.

* **Cross-refs.**

  * VM-VAR: 001 (plurality), 010 (WTA).

  * ALG: Doc 4A §2.1 (plurality), Doc 4B §2.1 (WTA; **WTA ⇒ Unit.magnitude=1** rule).

  * FUN: VM-FUN-002 (ValidateInputs enforces m=1), VM-FUN-004 (AllocateUnit).

  * DB: `Result.UnitBlock.seats_or_power` (single winner \= full allocation).

---

## **4\) VM-TST-003 — Largest remainder vs highest-average (locked)**

**Goal.** Ensure LR, D’Hondt, and Sainte-Laguë converge on the same allocation in this specific split.

* **Setup.** One district, **m \= 7**. Approval shares proportional to **A/B/C \= 34/33/33** (scale any common factor).

* **Params (delta).** Run three times with:

  * `VM-VAR-010 = largest_remainder`

  * `VM-VAR-010 = proportional_favor_small` (Sainte-Laguë)

  * `VM-VAR-010 = proportional_favor_big` (D’Hondt)

* **Expected (all three).** Seats **A/B/C \= 3/2/2**. Label: **Decisive**.

* **Accept.** Each method returns **3–2–2** exactly (sum \= 7).

* **Cross-refs.**

  * VM-VAR: 010 (method), 012 (threshold=0).

  * ALG: Doc 4B §2.2 (D’Hondt), §2.3 (Sainte-Laguë), §2.4 (Largest Remainder).

  * FUN: VM-FUN-004 (allocation trails for divisors/remainders).

  * DB: `Result.UnitBlock.seats_or_power`; audit: allocation trail (Annex C in report per Doc 7B).

---

## **5\) Acceptance for 6A**

* Deterministic order **A \> B \> C \> D** is respected; no ties occur in these cases.

* Seat vectors match **exactly** as specified (1–2–3–4 and 3–2–2) and totals equal **m**.

* WTA test enforces **m=1** rule at validation.

* Cross-references to VM-VAR/ALG/FUN/DB are correct and sufficient for auditors to trace logic.

# **Doc 6B — Tests: Gates, Ranked, Weighting, MMP Level**

**Scope.** Exercise legitimacy gates, ranked methods, weighting flip, and the **MMP correction level** effect.  
 **Conventions.** Unless a test says otherwise, defaults from Doc 2A apply, notably:

* `ballot_type = approval` (so **support % for gates \= approval rate \= approvals\_for\_change / valid\_ballots**, per Doc 4A/4C)

* `allocation_method = proportional_favor_small`, `pr_entry_threshold_pct = 0`

* `quorum_global_pct = 50`, `national_majority_pct = 55`, `regional_majority_pct = 55`

* `double_majority_enabled = on`, `symmetry_enabled = on`

* `weighting_method = population_baseline`, `aggregate_level = country`

* Deterministic option order **A \> B \> C \> D**; rounding as in Docs 4/7.

---

## **VM-TST-004 — Exact supermajority edge (≥ rule)**

**Setup.** Single national vote (binary: Change vs Status Quo). **Approval ballots.** Valid approvals for **Change \= 55.000%** of **valid ballots**. Quorum met.

**Params (delta).** none.

**Expected.** **Pass** (≥ 55%). **Label:** **Decisive** (margin 0.0 pp over threshold is treated as meeting it; with defaults, no mediation flags).

**Accept.** Majority gate shows `Support 55.0% vs 55% — Pass`. Outcome not blocked by other gates.

**Refs.** VM-VAR-022; Doc 4A (approval rate denominator), Doc 4C §1.2; VM-FUN-006.

---

## **VM-TST-005 — Quorum failure**

**Setup.** National turnout **48%** (from `Σ ballots_cast / Σ eligible_roll`). Change would have 60% support **among valid ballots** (approval rate), but quorum is not met.

**Params (delta).** none.

**Expected.** **Invalid**, reason **Quorum failed**.

**Accept.** Legitimacy Panel shows `Turnout 48.0% vs 50% — Fail`. Frontier omitted. Final label **Invalid**.

**Refs.** VM-VAR-020; Doc 4C §1.1; VM-FUN-006.

---

## **VM-TST-006 — Double-majority failure (family by proposed change)**

**Setup.** National **approval rate** for Change **57%** (≥55). Affected region family (derived by proposed change) has minimum regional support **53%** (\<55).

**Params (delta).** `double_majority_enabled=on`; `affected_region_family_mode=by_proposed_change`.

**Expected.** **Invalid**, reason **Regional threshold not met**.

**Accept.** Panel shows national **Pass**, regional **Fail** with lowest region printed; label **Invalid**.

**Refs.** VM-VAR-023/024/026; Doc 4C §1.3; VM-FUN-006.

---

## **VM-TST-007 — Symmetry respected (mirrored scenarios)**

**Setup.** Two mirrored proposals (A→B and B→A) with identical participation patterns; each has **56%** national support (approval rate) where it is the “Change.”

**Params (delta).** `symmetry_enabled=on`.

**Expected.** Both runs **Pass** (or both **Fail** if another gate blocks) with **identical** thresholds/denominators.

**Accept.** No direction-specific differences; panel lines match aside from option labels.

**Refs.** VM-VAR-025; Doc 4C §1.4; VM-FUN-006.

---

## **VM-TST-010 — IRV with exhaustion**

**Setup.** 100 ballots (ranked IRV):

* 40: **B \> A \> C**

* 35: **A \> C** (stop)

* 25: **C \> B** (10 of these stop)

**Params (delta).** `ballot_type=ranked_irv` (others default).

**Expected.** R1: A=35, B=40, C=25 → eliminate **C**; transfer **15** to B; **10** exhaust. Continuing ballots \= **90**. Final: **B=55**, **A=35** of continuing → **B wins**. **Label:** **Decisive**.

**Accept.** IRV RoundLog shows eliminations, transfers, **exhausted=10**; winner **B**.

**Refs.** VM-VAR-006; Doc 4A §2.4; VM-FUN-003 (RoundLog).

---

## **VM-TST-011 — Condorcet cycle resolved (Schulze)**

**Setup.** Pairwise preferences (head-to-head):

* A vs B: **55–45** (A beats B)

* B vs C: **60–40** (B beats C)

* C vs A: **60–40** (C beats A)

**Params (delta).** `ballot_type=ranked_condorcet`; `condorcet_completion=schulze`.

**Expected.** **B** is the Schulze winner. **Label:** **Decisive**.

**Accept.** Pairwise matrix recorded; winner **B**.

**Refs.** VM-VAR-005; Doc 4A §2.5; VM-FUN-003.

---

## **VM-TST-012 — Weighting flip (equal-unit vs population)**

**Setup.** Four Units (two small, two large). Support for **A** (approval rate proxy):

* Small1=80%, Small2=80% (weight 1 each)

* Large1=40%, Large2=40% (weight 10 each)

**Params (delta).**

* Case 1: `weighting_method=equal_unit`

* Case 2: `weighting_method=population_baseline` (use weights above)

**Expected.**

* **Case 1:** National A \= (80+80+40+40)/4 \= **60%** → **Pass** 55%. **Label:** **Decisive**.

* **Case 2:** Weighted A \= (80*1+80*1+40*10+40*10)/(1+1+10+10) \= **46.7%** → **Fail** majority gate → **Invalid**.

**Accept.** Outcome flips between cases; panel reflects weighting choice and results.

**Refs.** VM-VAR-030; Doc 4B §4; VM-FUN-005/006.

---

## **VM-TST-013 — MMP correction level (national vs regional)**

**Goal.** Show that **mlc\_correction\_level** changes final seat totals.

**Setup.** Three equal-population regions; **12 total seats** (local **B=6**, top-up **6**; top-up share **50%**).

* **Local tier (WTA SMDs):** 2 districts per region; winners:

  * Region 1 → **A** wins both; Region 2 → **B** wins both; Region 3 → **C** wins both.

* **Regional vote shares (for top-up targets):**

  * Region 1: **A 90%**, B 5%, C 5%

  * Region 2: **B 55%**, **A 40%**, C 5%

  * Region 3: **C 55%**, **A 40%**, B 5%  
     (These imply **national shares** averaging to **A 56.7%**, **B 21.7%**, **C 21.7%**.)

* **MMP params:** `allocation_method=mixed_local_correction`; `mlc_topup_share_pct=50`; `target_share_basis=natural_vote_share`; `overhang_policy=allow_overhang`; `total_seats_model=fixed_total`.

**Params (delta).** Compare:

1. `mlc_correction_level = national`

2. `mlc_correction_level = regional`

**Expected.**

* **Case 1 (national):** Targets across **T=12** ≈ **A 6.8**, **B 2.6**, **C 2.6**. With locals A/B/C \= 2/2/2, **top-ups (6)** iteratively to largest deficit yield **A 7, B 3, C 2** (tie for last seat goes to **B** over **C** via deterministic order **A\>B\>C**).

* **Case 2 (regional):** Each region corrects to its **own** targets with **2** top-ups:

  * R1 final **A 4, B 0, C 0**; R2 final **A 2, B 2, C 0**; R3 final **A 2, B 0, C 2**.

  * **Totals:** **A 8, B 2, C 2**.  
     **Labels:** Both **Decisive** (no gates depend on seat totals; no mediation).

**Accept.** Final national seat vectors **differ**: **A/B/C \= 7/3/2 (national)** vs **8/2/2 (regional)**. Allocation audit shows deficit-driven top-up sequence consistent with Doc 4B; deterministic order used on any equal-deficit tie.

**Refs.** VM-VAR-013/014/015/016/017; Doc 4B §3 (MMP); VM-FUN-004; Doc 7A Outcome section.

---

### **Notes common to 6B tests**

* Where “support %” appears under approval ballots, it is always the **approval rate** over **valid ballots** (Doc 4A/4C).

* Labels follow Doc 4C: **Invalid** if any gate fails; **Marginal** only if gates pass but margin \< VM-VAR-062 or frontier flags exist (none used here).

**Status.** Gates (004–007), ranked counting (010–011), weighting flip (012), and **MMP level effect** (013) are fully exercised with precise expectations and cross-references.

# **Doc 6C — Tests: Frontier, Executive, Determinism/Perf**

**Scope.** Frontier mapping behaviors (binary/sliding/ladder with contiguity & protections), executive \+ council combo, and reproducibility/performance.  
 **Conventions.** Unless stated, global defaults from Doc 2A apply; option order is **A \> B \> C \> D** (deterministic). Approval gates use the **approval rate** denominator (Doc 4A/4C). Frontier tests focus on mapping; assume gates already **Pass** unless noted. Mediation/protected flags force **Marginal** (Doc 4C).

---

## **VM-TST-014 — Binary cutoff with a contiguity break**

**Goal.** Changing status requires both support ≥ cutoff and contiguity per policy.

**Setup.** Five Units on one Region. Supports for **Change** (approval rate):

* U1=62%, U2=61%, U3=45%, U4=65%, U5=30%.  
   **Adjacency:** land edges: U1—U2—U3—U5; **U4 is separated by water from U3** (no land/bridge to U1/U2).  
   **Params (delta).** `frontier_mode=binary_cutoff`; `cutoff_pct=60`; `contiguity_modes_allowed={land}`; `island_exception_rule=none`. Per-unit quorum off.

**Expected.**

* U1 & U2 meet cutoff **and** are contiguous ⇒ **immediate\_change**.

* U4 meets cutoff but is **non-contiguous** (water only) ⇒ **Mediation** (no change).

* U3 (45%) & U5 (30%) ⇒ **no\_change**.

* Mediation zones: **1** (U4). **Label:** **Marginal** (mediation present).

**Accept.** FrontierMap statuses match above; `mediation_flagged=true` for U4; Result label \= Marginal with frontier reason.

**Refs.** VM-VAR-040/041/047/048; Doc 4C §2.1; VM-FUN-007; Doc 7A §8.

---

## **VM-TST-015 — Sliding-scale bands (with autonomy)**

**Goal.** Single-band assignment per Unit; AP mapping applied.

**Setup.** Four Units U1–U4 with supports: **25%, 35%, 52%, 61%**.  
 **Bands (ordered, non-overlapping):**

* `<30 → no_change`

* `30–49 → autonomy(AP:Base)`

* `50–59 → phased_change`

* `≥60 → immediate_change`  
   **Adjacency:** U1—U2—U3—U4 all by **land**.  
   **Params (delta).** `frontier_mode=sliding_scale`; `bands=as above` (validated non-overlap); `autonomy_package_map` maps “autonomy(AP:Base)” to **AP:Base:v1**; `contiguity_modes_allowed={land,bridge}`; `island_exception_rule=none`.

**Expected.**

* U1 → **no\_change**; U2 → **autonomy(AP:Base)**; U3 → **phased\_change**; U4 → **immediate\_change**.

* Each Unit has **exactly one** status; contiguous merges are informational (all distinct here).

* No mediation/enclave/protected flags ⇒ **Label: Decisive** (gates assumed pass; no frontier flags).

**Accept.** FrontierMap shows four statuses as listed; Result label **Decisive**.

**Refs.** VM-VAR-040/042/046/047; Doc 4C §2.4; VM-FUN-007; Doc 7A §8.

---

## **VM-TST-016 — Protected area blocks change (no override)**

**Goal.** Protected areas cannot change without an explicit override.

**Setup.** Three Units: U1 (protected), U2, U3. Supports: U1=70%, U2=62%, U3=41%.  
 **Adjacency:** U1—U2—U3 by land.  
 **Params (delta).** `frontier_mode=binary_cutoff`; `cutoff_pct=60`; `protected_override_allowed=off`.

**Expected.**

* U2 ⇒ **immediate\_change** (meets cutoff).

* **U1 protected** ⇒ **no\_change** despite 70% support; `protected_override_used=false`.

* U3 ⇒ **no\_change**.

* Protected constraint triggered ⇒ **Label: Marginal** (protected flag present).

**Accept.** FrontierMap flags **U1** as protected (unchanged); Result label **Marginal** with protected reason.

**Refs.** VM-VAR-040/041/045; Doc 4C §2.2; VM-FUN-007; Doc 7B §5.3.

---

## **VM-TST-017 — Diffuse support floor (no change anywhere)**

**Goal.** Below-band support yields no changes.

**Setup.** Six Units with supports all **\<40%** (e.g., 20, 28, 33, 35, 36, 39).  
 **Params (delta).** `frontier_mode=sliding_scale`; `bands=<40 → no_change; 40–59 → phased_change; ≥60 → immediate_change`. Contiguity default.

**Expected.**

* All Units ⇒ **no\_change**.

* No mediation/protected flags ⇒ **Label: Decisive** (assuming gates pass).

**Accept.** FrontierMap has only `no_change`; Result label **Decisive**.

**Refs.** VM-VAR-040/042; Doc 4C §2.4; VM-FUN-007.

---

## **VM-TST-018 — Executive (IRV) \+ Council (PR) combo**

**Goal.** Mixed institutions: IRV executive \+ proportional council.

**Setup.**  
 **Executive ballots (IRV, 100 ballots):**

* 40: **B \> A \> C**

* 35: **A \> C** (stop)

* 25: **C \> B** (10 stop)  
   **Council:** one national district, **m=15**; approvals by Option **D/C/B/A \= 40/30/20/10%**; PR threshold **5%**.

**Params (delta).**

* `executive_enabled=on`; `executive_ballot_type=ranked_irv`.

* Council: `allocation_method=proportional_favor_small`; `pr_entry_threshold_pct=5`.

**Expected.**

* **Executive:** IRV R1 A=35, B=40, C=25 → eliminate C → transfer 15 to B, 10 exhausted → continuing=90 → **B=55, A=35** ⇒ **B wins**.

* **Council seats:** Sainte-Laguë over m=15 with threshold 5% ⇒ **D/C/B/A ≈ 6/5/3/1** seats.

* **Label:** **Decisive** (no mediation or protected flags).

**Accept.** Result shows executive winner **B** with RoundLog; council seats **6/5/3/1**; gates pass (quorum assumed met).

**Refs.** VM-VAR-001/006/010/012; Doc 4A §2.4; Doc 4B §2.3; VM-FUN-003/004.

---

## **VM-TST-019 — Large deterministic pass (scale & reproducibility on one OS)**

**Goal.** Validate stability and performance at scale.

**Setup.** Synthetic registry with **≈5,000 Units**, four Options, approval ballots; default parameters; no frontier.

**Params (delta).** none. Tie policy can remain `status_quo` (no expected ties).

**Expected.**

* Run completes within the **performance ceiling** specified in Doc 3 (memory/time).

* Repeating the run **twice on the same machine** yields **byte-identical** `Result` and `RunRecord` (IDs, checksums).

**Accept.** Two identical hashes for `Result` and `RunRecord`; recorded timing within Doc 3 limits.

**Refs.** Doc 3 (determinism/perf), Doc 5A/5B/5C (stable order & rounding), VM-FUN-011 (RunRecord).

---

## **VM-TST-020 — Cross-OS reproducibility (Windows/macOS/Linux)**

**Goal.** Prove cross-platform determinism.

**Setup.** Re-run **VM-TST-001** (or any small canonical scenario) on **Windows, macOS, and Linux** using the same engine build.

**Params (delta).** none. If any test case uses random tie-breaking, set `tie_policy=random; rng_seed=424242` (not used in VM-TST-001).

**Expected.**

* `Result` and `RunRecord` are **byte-identical across all three OS** (IDs, digests, serializations).

**Accept.** Matching hashes and files across OS; RNG seed recorded if applicable.

**Refs.** Doc 3 (ordering/rounding/RNG rules), VM-FUN-008 (seed logging), VM-FUN-011.

---

### **Notes on seeds**

* None of 014–018 require randomness. If a local test variant introduces a blocking tie, set `tie_policy=random` with **`rng_seed=424242`** and log it; outputs must still be identical run-to-run and cross-OS.

---

## **Acceptance for 6C**

* Frontier tests (014–017) produce the exact statuses and flags specified; **Mediation/Protected** flags force **Marginal** where noted.

* Executive \+ Council (018) returns IRV winner **B** and council seats **6/5/3/1**.

* Determinism tests (019–020) achieve **byte-identical** outputs within performance ceilings and across OS.

* Any use of randomness includes a **recorded seed** and yields reproducible results.

