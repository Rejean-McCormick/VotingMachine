# **Annex C — Glossary & Definitions (Updated)**

## **1\) Scope**

Canonical, one-page definitions for recurring terms and tokens used across Docs **1–7** and Annexes **A–B**. Aligns IDs to the **current scheme** (e.g., ties \= **VM-VAR-050/052**, **051 reserved**; presentation **060–062** are non-FID).

---

## **2\) Terms**

**Algorithm family**  
 The counted method and rounding/denominator rules selected by **VM-VAR-001..007** (+ optional **VM-VAR-073**). Controls allocation semantics (Doc 4A). *Outcome-affecting; in FID.*

**Algorithm variant (VM-VAR-073)**  
 Documented micro-variant within the same family (e.g., a prescribed rounding preference). Printed in report footer if not default (Doc 3B, 7A). *In FID.*

**Allocation**  
 Per-unit mapping of `option_id → votes/share` produced after gates/frontier/ties (Docs 4A–4C). Ordered by Registry `order_index`.

**Annex A — VM-VAR Registry**  
 Single source of truth for variable **domains, defaults, and FID inclusion**. The “Included” list defines the **Normative Manifest** inputs (Doc 1A).

**Annex B — Canonical Test Pack**  
 Machine-readable fixtures (inputs, expected outputs/hashes) for suites 6A/6B/6C, plus RNG profile (Annex B §7).

**Band / band\_met**  
 Frontier predicate indicating whether a unit lies within the effective band/cut for the configured model. Emitted in `FrontierMap.units[i].band_met` (Doc 4C). Field name is **`band_met`**.

**BallotTally**  
 Input JSON giving per-unit totals and per-option votes (Doc 1B). Must align to **DivisionRegistry**.

**Canonical JSON**  
 Serialization used for hashing: UTF-8, LF newlines, **sorted keys** at all object levels; arrays ordered per Doc 1A §5; no BOM (Doc 1A §2.1).

**Determinism (byte-identical outputs)**  
 Given identical inputs \+ ParameterSet (incl. seed), all artifacts are bit-for-bit identical across OS/arch (Doc 3A). Enforced by ordering rules, canonical JSON, and fixed RNG profile (Annex B §7).

**DivisionRegistry**  
 Input JSON defining the universe of `units` and their `options`, including deterministic `order_index` per option (Docs 1B–1C). Source of all FK references.

**FID — Formula ID**  
 64-hex SHA-256 over the **Normative Manifest** (algorithm rules \+ “Included” VM-VAR values). Recorded at `Result.formula_id` and `RunRecord.formula_id` (Doc 1A §2.3). *Presentation toggles are excluded.*

**Frontier (040–042, 047–049)**  
 Outcome-affecting gating model (e.g., `banded`, `ladder`) applied pre-allocation (Doc 4C). Advanced refinements: window/backoff/strictness. *In FID.*

**FrontierMap**  
 Optional canonical artifact with per-unit frontier diagnostics (Doc 1A §4.6). Emitted only if **VM-VAR-034=true** and frontier executed.

**Gate (sanity/eligibility/validity)**  
 Deterministic checks applied before allocation (Doc 4B) using **010–017**, **020–031** (+ **045**, **029**, **030** precedence). Failure ⇒ unit `Invalid`.

**Included / Excluded (FID)**  
 A VM-VAR is **Included** if it can change outcomes (in FID); **Excluded** if it is presentation-only (not in FID). See Annex A §5.

**Invalid (label)**  
 Unit state when any gate fails; `allocations=[]`; label is `"Invalid"` (Docs 4B, 7A).

**Label (Decisive/Marginal)**  
 Presentation-only outcome text computed **after** allocation using **VM-VAR-060/061**; never affects counts or FID (Doc 4C).

**Normative Manifest**  
 Ordered, canonical snapshot of all outcome-affecting rules \+ Included VM-VAR values used to compute **FID** (Doc 1A §2.3; Annex A §5).

**Option / option\_id**  
 A selectable alternative within a unit (e.g., party/candidate). Identified by stable `option_id` and deterministic `order_index` (Docs 1B–1C).

**Order index (order\_index)**  
 Deterministic integer ordering key for options within a unit. Used to break ties under `deterministic_order` policy (Docs 1A §5, 4C).

**ParameterSet**  
 Input JSON map `vars{ "VM-VAR-###": value }`. Must provide explicit values for all **Included** VM-VARs; presentation vars may be present but are non-FID (Docs 1B, 2A–2C).

**Protected area / override (045)**  
 Registry flag `protected_area` on a unit; **VM-VAR-045** may bypass **eligibility** only; never bypasses **sanity** nor **integrity floor (031)** (Doc 4B).

**Random tie (050= random, 052 seed)**  
 Policy that resolves ties via a deterministic RNG seeded by **VM-VAR-052**. Exactly **k** 64-bit draws for a k-way tie; permutation sorted by `(draw, option_id)`; events logged (Docs 3A, 4C, 5C; Annex B §7). *050 is in FID; 052 is not.*

**Renderer**  
 Read-only consumer of canonical artifacts. Applies Doc 7A visual rules and 7B templates; never re-computes allocations; may include optional appendices (Doc 5C) (Docs 7A–7B).

**Result / result\_id**  
 Canonical outcome artifact; `result_id = "RES:" + sha256(canonical(Result))` (Doc 1A §4.4).

**RNG profile**  
 Frozen algorithm/spec used for random ties; defined in Annex B §7 to ensure cross-platform identical sequences. Changing it ⇒ new FID.

**RunRecord / run\_id**  
 Canonical provenance artifact containing engine info, input digests, `vars_effective`, determinism block, and TieLog. `run_id = "RUN:" + <ts> + "-" + sha256(canonical(RunRecord))` (Doc 1A §4.5).

**Scope (run\_scope, 021\)**  
 Selector that fixes which units are included in the run before counting (Docs 2A/2C, 4A).

**Sensitivity analysis (035)**  
 Non-canonical appendix switch for diagnostic comparisons. No changes to canonical artifacts/hashes (Docs 5C, 7A–7B). *Excluded from FID.*

**Symmetry exceptions (029)**  
 Deterministic allow/deny selectors that narrowly override **eligibility** gates; applied after 030 precedence (Doc 4B). *In FID.*

**Tie policy (050) / deterministic order**  
 Tie policy ∈ `{ status_quo | deterministic_order | random }`. Deterministic order uses **Registry `order_index`** (then `option_id`); **VM-VAR-051 is reserved** (Docs 4C, Annex A).

**Tie seed (052)**  
 Run parameter for random ties. Echoed in `RunRecord.determinism.rng_seed` **iff** any random tie occurred; otherwise omitted (Docs 3A, 5C). *Excluded from FID.*

**TieLog**  
 Ordered list of tie events in `RunRecord.ties[]` with `{unit_id, type, policy, seed?}` (Doc 5C §2.3).

**Unit / unit\_id**  
 A counting division (e.g., district). Primary key for per-unit results; arrays of units ordered by ascending `unit_id` (Doc 1A §5).

---

## **3\) Token & naming conventions (normative)**

* **Variables:** `VM-VAR-###` (three digits, zero-padded).

* **Reason tokens (4B):** `"VM-VAR-0xx:<short_name>"` ordered by numeric ID; symbolic reasons (e.g., `"frontier_missing_inputs"`) appear **after** ID-based reasons and are sorted lexicographically.

* **Field names:** `snake_case`. Use **`band_met`** (not variants).

* **IDs & hashes:** lowercase 64-hex; prefixes `RES:`, `RUN:`, `FR:` as defined in Doc 1A.

---

## **4\) Quick cross-reference (where defined)**

| Concept | Primary definition | Also referenced |
| ----- | ----- | ----- |
| Canonical JSON & IDs | Doc 1A §2, §4 | Doc 3A, Doc 5A/S5 |
| VM-VAR registry & FID inclusion | Annex A §3–§5 | Doc 2A–2C |
| Algorithm flow | Doc 4A | Doc 5A/S3 |
| Gates & edge cases | Doc 4B | Doc 6B |
| Frontier model | Doc 4C §2 | Doc 6B, FrontierMap in Doc 1A §4.6 |
| Ties & RNG | Doc 4C §3 | Doc 3A (RNG), Doc 6C, Doc 5C (TieLog) |
| Labels & presentation | Doc 4C §4, Doc 7A | Doc 2B, Doc 7B |
| Test harness & cases | Doc 6A–6C | Annex B |
| Release/versioning | Doc 3B | Footers in Doc 7A |

---

## **5\) ID alignment notes (this edition)**

* **Tie controls** live at **VM-VAR-050 (policy)** and **VM-VAR-052 (seed)**; **VM-VAR-051 reserved**.

* **Presentation & language** (`VM-VAR-060..062`) are **Excluded from FID**.

* Any previous references to ties at **032–033** or to “seed at 033” are obsolete.

*End Annex C.*

