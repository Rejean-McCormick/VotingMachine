# **Doc 4A — Algorithm: Step Order, Tabulation & Global Flow (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **end-to-end step order** and tabulation flow the engine MUST follow to produce deterministic, byte-identical results. 4A fixes *when* each rule applies; 4B covers **gates & edge cases**; 4C covers **Frontier, Ties, and Labels**. No legacy numbering.

**Inputs:** `DivisionRegistry`, `BallotTally`, `ParameterSet`  
 **Outputs:** `Result`, `RunRecord`, optional `FrontierMap`  
 **Determinism primitives:** Doc 1A §5 (ordering), VM-VARs per Doc 2/Annex A

---

## **2\) Preconditions (must hold before counting)**

1. **Schema/refs valid** (Doc 1B).

2. **ParameterSet** includes all outcome-affecting VM-VARs listed “Included” in Annex A.

3. **Order keys ready:** each Registry unit has unique `option.order_index` (Doc 1B).

4. **Run scope fixed:** apply **VM-VAR-021** to derive the working set of units (default: `all_units`). Record in `RunRecord.summary` if filtered.

---

## **3\) Step order (canonical pipeline)**

### **S0. Normalize & seed (determinism)**

* Canonicalize inputs (Doc 1A §2.1); compute input digests for `RunRecord.inputs`.

* Bind **Algorithm family & rounding** constants (**VM-VAR-001…007**) and optional **algorithm\_variant** (**VM-VAR-073**).

* Initialize RNG **only** if needed later: store **VM-VAR-052** (no draws yet).

### **S1. Per-unit tallies**

For each included `unit_id` (iterate in ascending `unit_id`):

1. Load `valid_ballots`, `invalid_ballots`, and per-option `votes`.

2. Compute raw shares (engine precision policy; stable across builds).

3. Compute any base metrics required by 4B/4C (e.g., margins, turnout).

### **S2. Eligibility & validity gates (4B)**

Apply gates in a fixed order using **VM-VAR-010…017**, **020…029**, and advanced **029–031** where present:

* If a gate marks the unit **invalid**, branch as defined in 4B (result label `Invalid`, allocations empty or per family rule).

* Record reasons in `RunRecord.summary`/notes as required by 4B.

### **S3. Frontier model hook (4C)**

If **VM-VAR-040** enables Frontier:

* Select band/cut params via **041**; apply strategy via **042**.

* Optionally refine by **047–049** (advanced).

* Emit per-unit diagnostics into `FrontierMap` (if **VM-VAR-034=true**), using `band_met` etc.

* Frontier never violates canonical ordering.

### **S4. Core allocation**

Using the algorithm family constants **001…007** (and **073** if defined):

* Compute per-unit **allocations\[\]** deterministically.

* Respect Registry option order (`order_index`) for any rank-sensitive operation.

* No randomization at this stage.

### **S5. Tie resolution (4C)**

If a tie affects an allocation/ordering decision:

* Read **VM-VAR-050 tie\_policy**:

  * `status_quo`: apply family’s status-quo rule.

  * `deterministic_order`: break ties by ascending `option.order_index` (no variable; **051 reserved**).

  * `random`: use deterministic RNG seeded with **VM-VAR-052**; log each event in `RunRecord.ties[]` and set `RunRecord.determinism.rng_seed`.

* Never consume RNG draws unless `tie_policy=random` **and** a tie actually occurs.

### **S6. National/aggregate metrics & labels (4C/7)**

* Aggregate required national metrics (e.g., national margin).

* Compute **Outcome label** per policy:

  * Threshold **VM-VAR-060** and policy **VM-VAR-061** (“fixed” vs “dynamic\_margin”) influence labeling **only** (presentation).

  * Labels do not affect allocations; they are excluded from FID.

### **S7. Emit artifacts (Doc 1A)**

* Build `Result` (units ordered by `unit_id`; allocations per `order_index`).

* Build `RunRecord` (engine info, nm\_digest, vars\_effective, tie log).

* Optionally build `FrontierMap` (if enabled and applicable).

* Canonicalize JSON and compute IDs (`result_id`, `run_id`, `frontier_id`).

* Set `formula_id` from the Normative Manifest (Doc 1A/Annex A).

* Self-verify hashes; fail if any mismatch (Doc 3A/3B gates).

---

## **4\) Deterministic ordering (reiterated for algorithm use)**

* **Units:** iterate strictly in ascending `unit_id`.

* **Options within a unit:** ascending `order_index`; on equal, ascending `option_id`.

* **Allocations arrays:** mirror Registry option order.

* **No iteration over map/dict order** may influence results.

---

## **5\) Variable touchpoints used in 4A**

* **Global & scope:** **001…007**, **021**, **073**

* **Gates/thresholds:** **010…017**, **020…029**, **030–031** (if set)

* **Frontier (hook only; details in 4C):** **040–042**, **047–049**

* **Ties (delegated to 4C):** **050 (policy)**, **052 (seed), 051 reserved**

* **Presentation (labels only):** **060–062** (do not alter counts; excluded from FID)

---

## **6\) Pseudocode (normative skeleton)**

init\_context(params, registry, tally):  
  assert validate\_all()  
  scope\_units \= select\_units(registry.units, VM\_VAR\_021)  
  bind\_family(VM\_VAR\_001..007, VM\_VAR\_073)  
  rng\_seed \= VM\_VAR\_052

for unit in sort\_by\_unit\_id(scope\_units):  
  u \= prepare\_unit(unit, tally\[unit\])  
  apply\_gates(u, VM\_VAR\_010..017, VM\_VAR\_020..029, VM\_VAR\_030..031)   // 4B  
  if u.invalid:  
    result.units.append(invalid\_record(u))  
    continue

  frontier\_ctx \= frontier\_hook(u, VM\_VAR\_040, VM\_VAR\_041, VM\_VAR\_042, VM\_VAR\_047..049) // 4C  
  allocations \= compute\_allocations(u, family\_consts)                                   // deterministic  
  if has\_tie(allocations):  
    allocations \= resolve\_ties(allocations, VM\_VAR\_050, rng\_seed)                      // 4C; uses 052 only if random

  label \= compute\_label(u, allocations, VM\_VAR\_060, VM\_VAR\_061)                         // presentation only  
  emit\_unit\_result(u.unit\_id, allocations, label, frontier\_ctx)

finalize\_and\_emit(Result, RunRecord, FrontierMap)                                       // Doc 1A

---

## **7\) Conformance checklist (4A)**

* **C-4A-ORDER:** All loops honor Doc 1A ordering; no nondeterministic iteration affects results.

* **C-4A-GATES:** Gates applied before allocation, in fixed order, with recorded reasons when invalid.

* **C-4A-TIES:** Ties handled strictly per **VM-VAR-050**; RNG used only when required, with events logged.

* **C-4A-FRONTIER:** Frontier hook executes when enabled; diagnostics emitted only if **VM-VAR-034=true**.

* **C-4A-LABELS:** Labels computed per **060/061**; allocations unaffected.

* **C-4A-EMIT:** Artifacts canonicalized; IDs verified; FID matches Normative Manifest.

*End Doc 4A.*

# **Doc 4B — Gates & Edge Cases (Updated, Normative)**

## **1\) Purpose & scope**

Defines **when a unit is valid/invalid** for allocation and how **edge cases** are handled, in a **fixed, deterministic order**. Variables come from **Doc 2A/2C**; exact domains/defaults live in **Annex A**. No legacy numbering.

* Gate families (outcome-affecting; **FID \= YES**):  
   **VM-VAR-010…017**, **020…029**, **030–031**.

* Interplay with other parts: Frontier hook (4C), Ties (4C), Labels (4C/Doc 7), Protected/Autonomy (2A §4.4).

Outputs must be **byte-identical** across OS/arch when inputs \+ ParameterSet match.

---

## **2\) Evaluation order (canonical)**

Gates execute **before allocation** (4A S2) in the following fixed order. The first failing stage **does not short-circuit**; record **all** reasons, then branch as defined in §3.1.

1. **Sanity gates** (data plausibility) — uses 010…017 group if defined for sanity.

2. **Eligibility gates** (thresholds & scope) — uses 020…029, 021, 029\.

3. **Validity gates** (integrity floors & overrides) — uses 030–031, 045\.

4. **Frontier pre-check** (if enabled) — consistency with 040–042, 047–049 (4C).

Deterministic ordering matters: implementations MUST follow this stage order and, within a stage, evaluate gates in ascending **VM-VAR** ID order.

---

## **3\) Gate behavior (normative patterns)**

### **3.1 Branching rule**

* If **any** gate fails for a unit → mark **`unit.invalid=true`** and **do not allocate** in S4.

* Emit a **unit result** with:

  * `allocations`: **empty array**

  * `label`: `"Invalid"` (Doc 7 will render accordingly)

  * Optional diagnostic fields in `RunRecord.summary` (see §6)

No other branch (e.g., “provisional allocation”) is allowed in v1.

### **3.2 Sanity gates (010…017)**

Deterministic checks on tallies and basic ratios (exact set per Annex A). Examples of required behavior (independent of names):

* **Non-negativity**: votes/ballots cannot be negative.

* **Consistency**: `sum(option.votes) ≤ totals.valid_ballots`.

* **Bounds**: any declared percentage thresholds must lie within their domain.

**Failure →** record reason(s); continue evaluating remaining gates; final branch per §3.1.

### **3.3 Eligibility gates (020…029, plus scope 021\)**

Apply **run scope** (**VM-VAR-021**) first to pick included units (4A S2 precondition). Within an included unit, evaluate thresholds in ascending ID order. Required behavior patterns:

* **Minimum participation/turnout** gate(s).

* **Minimum share/eligibility** gate(s) for options or unit-level continuation.

* **Symmetry exceptions** (**VM-VAR-029**): an explicit, deterministic allow/deny list that **narrowly** overrides a corresponding eligibility rule (never the sanity/validity gates). Matching is deterministic (Annex A defines the selector grammar).

**Failure →** record reason(s); final branch per §3.1.

### **3.4 Validity gates (030–031) & overrides**

* **Eligibility override list** (**VM-VAR-030**): explicit `{unit_id, mode}` directives applied **before** integrity floors. `mode=include` can re-include a unit that would be excluded by **eligibility** gates; it **cannot** override **sanity** failures.

* **Ballot integrity floor** (**VM-VAR-031**): if a unit’s integrity KPI falls **below** the floor, mark invalid.

* **Protected-area override** (**VM-VAR-045**, from 2A): when `DivisionRegistry.units[].protected_area=true`, behavior is:

  * If **045 \= deny** (default): protected status does **not** bypass validity; treat like any unit.

  * If **045 \= allow**: a protected unit may **bypass an eligibility gate** (020…029) but **never** a sanity failure (010…017) nor the integrity floor (031). All bypasses must be recorded (see §6).

**Failure →** record reason(s); final branch per §3.1.

### **3.5 Frontier pre-check (040–042, 047–049)**

If Frontier is enabled (4C S3), confirm that required inputs/metrics exist for the unit and that advanced tuning (047–049) is within bounds. Frontier logic itself runs in 4C; this pre-check only detects **configuration errors** (treated as validity failures).

---

## **4\) Edge cases (normative handling)**

* **Zero valid ballots**:

  * Sanity passes (non-negative); eligibility typically fails (share/turnout).

  * Result: `allocations=[]`, `label="Invalid"`, reasons include the failing gate(s).

* **Sum of votes \< valid\_ballots**: Allowed (abstentions/blank). Sanity passes; other gates decide.

* **Missing option tallies**: Missing `option_id` entries are treated as **0 votes** only if **explicitly permitted** by Annex A; otherwise it’s a **sanity failure**.

* **All options tied with zero**: Not a gate failure. If unit remains valid, allocation yields all zeros; 4C ties do **not** trigger because no rank decision is required.

* **Protected area without override**: Protected flag alone has **no effect** unless **045=allow**. Never bypass sanity/integrity.

* **Conflicting directives** (e.g., 029 vs 030): Precedence is fixed — **030 (eligibility override) → 029 (symmetry exceptions)**. Implementations must document the applied precedence in `RunRecord.summary`.

* **Frontier inputs missing** when Frontier enabled: validity failure with reason “frontier\_missing\_inputs”.

---

## **5\) Deterministic order of recording reasons**

When multiple reasons exist, record them in **ascending VM-VAR ID order**, then any symbolic reasons (e.g., `frontier_missing_inputs`) in lexicographic order. This guarantees byte-identical `RunRecord.summary` across platforms.

---

## **6\) RunRecord requirements (per-unit)**

For each unit evaluated:

{  
  "unit\_id": "U-001",  
  "gate\_status": "valid" | "invalid",  
  "reasons": \["VM-VAR-020:min\_turnout", "VM-VAR-031:integrity\_floor"\],  // ordered, see §5  
  "protected\_bypass": true | false,           // present only if 045 allowed a bypass  
  "applied\_exceptions": \["VM-VAR-029:U-001"\], // if any selector matched  
  "frontier\_ready": true | false              // pre-check result if Frontier enabled  
}

* `reasons[]` lists **all** failing gates (or empty if none).

* `protected_bypass=true` appears **only** if **045=allow** caused an eligibility bypass.

* `applied_exceptions[]` lists matched 029 selectors (deterministic string form per Annex A).

* This structure may live under `RunRecord.summary.units[]` (producer’s choice), but ordering rules apply (Doc 1A §5).

---

## **7\) Conformance checklist (4B)**

* **C-4B-ORDER**: Gates evaluated in stage order **Sanity → Eligibility → Validity → Frontier pre-check**, ascending ID within stage.

* **C-4B-BRANCH**: Any failure ⇒ `allocations=[]`, `label="Invalid"`. No silent partial allocations.

* **C-4B-PROT**: `protected_area` may bypass **eligibility** only when **045=allow**; never bypasses Sanity or Integrity Floor (031).

* **C-4B-EXC**: 030 overrides applied before 029 and recorded.

* **C-4B-RR**: All reasons and applied exceptions recorded deterministically (ordered as in §5).

* **C-4B-FRONTIER**: Frontier pre-check failures recorded; 4C logic not executed for invalid units.

---

## **8\) Pseudocode (reference)**

reasons \= \[\]  
valid \= true

// Sanity (010..017)  
for v in sort\_ids(010..017):  
  if \!check\_sanity(v, unit, tally): reasons.append(reason(v)); valid \= false

// Eligibility scope  
if \!in\_scope(unit, VM\_VAR\_021): reasons.append("VM-VAR-021:out\_of\_scope"); valid \= false

// Eligibility (020..029)  
for v in sort\_ids(020..029):  
  if \!check\_eligibility(v, unit, tally):  
     if protected(unit) && VM\_VAR\_045 \== "allow" && is\_eligibility\_gate(v):  
        record\_bypass(v); // no reason added  
     else:  
        reasons.append(reason(v)); valid \= false

// Overrides & validity (030..031)  
apply\_overrides(VM\_VAR\_030, unit, reasons)  // may flip prior eligibility failure, not sanity  
if \!check\_integrity\_floor(VM\_VAR\_031, unit): reasons.append(reason(031)); valid \= false

// Frontier pre-check if enabled  
if frontier\_enabled(VM\_VAR\_040):  
  if \!frontier\_ready(unit): reasons.append("frontier\_missing\_inputs"); valid \= false

if \!valid:  
  emit\_invalid\_unit(unit\_id, reasons, protected\_bypass, applied\_exceptions)  
else:  
  proceed\_to\_allocation()

*End Doc 4B.*

# **Doc 4C — Frontier, Ties & Labels (Updated, Normative)**

## **1\) Purpose & scope**

Defines the **frontier model**, **tie resolution**, and **outcome labeling** used by the engine. Frontier and ties are **outcome-affecting** (⇒ included in FID via Doc 2A/2C & Annex A); labels/language are **presentation-only** (⇒ excluded from FID per Doc 1A).

**Inputs:** `DivisionRegistry`, `BallotTally`, `ParameterSet`  
 **Consumes VM-VARs:** Frontier **040–042**, advanced **047–049**; Ties **050 (policy)**, **052 (seed)**; Labels **060–062** (presentation)  
 **Emits:** `Result`, `RunRecord`, optional `FrontierMap` (diagnostics)

---

## **2\) Frontier model (outcome-affecting)**

### **2.1 Enablement & selection**

* **VM-VAR-040 frontier\_mode** — selects the frontier model (e.g., `none`, `banded`, `ladder`).

* **VM-VAR-041 frontier\_band/cut** — primary numeric or enumerated cut parameter(s).

* **VM-VAR-042 frontier\_strategy** — how/when the frontier applies (e.g., `apply_on_entry`, `apply_on_exit`, `sticky`).

Advanced refinements (still outcome-affecting):

* **VM-VAR-047 frontier\_band\_window** — expands/contracts effective band around 041\.

* **VM-VAR-048 frontier\_backoff\_policy** — resolves borderline cases (`none`/`soften`/`harden`).

* **VM-VAR-049 frontier\_strictness** — coarse multiplier for 047/048 effects.

### **2.2 Deterministic evaluation**

For each included unit (Doc 4A S3), compute frontier predicates **before** allocation in a deterministic order:

1. Derive required metrics from `BallotTally` (and any prior stage outputs).

2. Apply **040/041/042** exactly; apply **047–049** per Annex A precedence (047 window → 048 backoff → 049 strictness).

3. Produce a boolean **`band_met`** and optional numeric **`band_value`** used by the algorithm to gate/branch.

Rules:

* If `frontier_mode = none`, skip all frontier logic (no gating by frontier).

* Frontier must **not** mutate array ordering (Doc 1A §5).

* If configuration is invalid (missing metrics/out-of-domain value), treat as a **4B validity failure** (unit becomes `Invalid`).

### **2.3 Diagnostics (FrontierMap)**

If **VM-VAR-034 \= true** and frontier is enabled:

Emit `frontier_map.json` with, for each evaluated unit:

 { "unit\_id": "...", "band\_met": true|false, "band\_value": \<number\>, "notes": "..." }

*   
* Arrays ordered per Doc 1A §5. Field name is **`band_met`** (normalized).

* Presence/absence of `frontier_map.json` does **not** affect outcomes.

---

## **3\) Tie resolution (outcome-affecting)**

### **3.1 Controls**

* **VM-VAR-050 tie\_policy** ∈ `{ status_quo, deterministic_order, random }`.

* **VM-VAR-051** is **reserved** (no variable exists for deterministic order key).

* **VM-VAR-052 tie\_seed** ∈ integers ≥ 0; used **only** if `tie_policy = random`.

### **3.2 Where ties apply**

Any stage where a **relative order** among options affects allocation or ranking (post-frontier, pre-emit), including:

* Winner/seat assignment ties.

* Rank ordering ties that drive subsequent algorithm branches.

### **3.3 Deterministic procedures**

* **status\_quo** — Apply the family’s fixed rule (e.g., keep prior holder). Must not rely on input file order; any “prior holder” notion must be derived from explicit, deterministic data.

* **deterministic\_order** — Break ties by ascending `Option.order_index`; if equal, ascending `option_id`. No variable controls this; **051 remains unused**.

* **random** — Use a deterministic RNG seeded with **052**:

  * For a tie among **k** options, generate a **deterministic permutation** of the tied set:

    1. For each tied `option_id`, draw one uniform 64-bit value from the run RNG.

    2. Sort the tied options by `(draw_value, option_id)` ascending to get a stable random order.

  * Consume **exactly k draws** per tie event. Do not draw when no tie exists.

  * Record an entry in `RunRecord.ties[]` with `unit_id`, tie type (e.g., `winner_tie`/`rank_tie`), `policy="random"`.  
     Set `RunRecord.determinism.rng_seed = VM-VAR-052` iff at least one random tie occurred.

Constraints:

* RNG algorithm/profile is fixed in Annex B to produce **identical sequences** across platforms.

* Ties **never** read presentation VM-VARs.

* Random tie resolution must not leak into any ordering beyond the tied subset.

---

## **4\) Outcome labels (presentation-only)**

### **4.1 Controls**

* **VM-VAR-060 majority\_label\_threshold** — integer percent (0..100).

* **VM-VAR-061 decisiveness\_label\_policy** ∈ `{ fixed, dynamic_margin }`.

* **VM-VAR-062 unit\_display\_language** — `auto` or IETF tag (used by renderer; see Doc 7).

### **4.2 Label computation**

Compute per-unit label **after** allocation and tie resolution, without altering allocations:

* **fixed** policy:

  * If `national_or_unit_margin ≥ 060` ⇒ `"Decisive"`, else `"Marginal"` (unless the unit is invalid ⇒ `"Invalid"`).

* **dynamic\_margin** policy (default):

  * `"Decisive"` iff `margin ≥ 060` **and** no **blocking flags** are set.

  * `"Marginal"` if `margin < 060` **or** any blocking flag is set.

  * Blocking flags are deterministic boolean signals produced elsewhere in the algorithm (e.g., mediation in effect, protected override used). Their exact sources are defined in Docs 4A/4B and Annex A; labels **only** read those booleans.

Notes:

* Labels and language are **excluded from FID** (Doc 1A).

* Renderer obeys Doc 7 visual rules; language selection via **062** does not change JSON ordering/content.

---

## **5\) RunRecord requirements (4C-specific)**

* **Frontier**: if enabled, record at least `{ unit_id, frontier_applied: true|false }` per unit in `RunRecord.summary` or equivalent; implementations may also copy `band_met`/`band_value` summary stats.

**Ties**: maintain `RunRecord.ties[]` entries in the canonical order of unit evaluation (ascending `unit_id`), each with:

 { "unit\_id":"...", "type":"winner\_tie|rank\_tie|other", "policy":"status\_quo|deterministic\_order|random" }

*  Include `"seed": <int>` only when `policy="random"`.

* **Determinism**: `RunRecord.determinism.tie_policy` mirrors **050**; `rng_seed` present iff any random tie event occurred.

---

## **6\) Ordering & determinism (reiterated)**

* Evaluate frontier and ties **after** gates (4B) and **before** emit (4A S7).

* Never depend on map/dict iteration order; always use Doc 1A §5 canonical ordering.

* In random ties, consume **exactly k** RNG draws for a **k-way** tie and **no draws otherwise**.

---

## **7\) Conformance checklist (4C)**

* **C-4C-FR-CFG**: Frontier parameters (040–042, 047–049) in domain; invalid config ⇒ validity failure (4B), not undefined behavior.

* **C-4C-FR-DET**: Frontier decisions are deterministic for a given ParameterSet; `FrontierMap` (if emitted) matches those decisions.

* **C-4C-TIE-POL**: Tie resolution strictly follows **050**; `deterministic_order` uses `order_index`; **051 is unused**.

* **C-4C-TIE-RNG**: RNG seeded only from **052**; exactly **k** draws per k-way tie; events logged; seed echoed iff any random tie.

* **C-4C-LBL-PRES**: Labels computed per **060/061** and do not affect allocations or FID.

* **C-4C-ORDER**: Unit and allocation arrays retain canonical order after ties/frontier.

---

## **8\) Reference pseudocode**

// Frontier  
if VM\_VAR\_040 \!= "none":  
  fm \= compute\_frontier\_metrics(unit, tally)  
  frontier \= apply\_frontier(VM\_VAR\_040, VM\_VAR\_041, VM\_VAR\_042, VM\_VAR\_047..049, fm)  
  if VM\_VAR\_034: record\_frontier\_map(unit\_id, frontier.band\_met, frontier.band\_value)

// Allocation already computed (4A S4)

// Ties  
if has\_tie(allocations):  
  switch VM\_VAR\_050:  
    case "status\_quo": allocations \= apply\_status\_quo(allocations)  
    case "deterministic\_order": allocations \= sort\_by(order\_index, option\_id, within\_tied\_groups(allocations))  
    case "random":  
      draws \= {}  
      for opt in tied\_group(allocations):  
        draws\[opt\] \= rng\_next64()     // seeded once at run start from VM\_VAR\_052  
      allocations \= sort\_tied\_by(draws\[opt\], option\_id)  
      RunRecord.ties.append({unit\_id, type, policy:"random", seed: VM\_VAR\_052})

// Labels (presentation)  
if unit\_invalid: label \= "Invalid"  
else:  
  margin \= compute\_margin(unit, allocations)  
  if VM\_VAR\_061 \== "fixed":  
    label \= (margin \>= VM\_VAR\_060) ? "Decisive" : "Marginal"  
  else:  
    flags \= read\_blocking\_flags(unit)    // deterministic booleans from earlier stages  
    label \= (margin \>= VM\_VAR\_060 && \!flags.any) ? "Decisive" : "Marginal"

emit\_unit\_label(unit\_id, label)

*End Doc 4C.*

