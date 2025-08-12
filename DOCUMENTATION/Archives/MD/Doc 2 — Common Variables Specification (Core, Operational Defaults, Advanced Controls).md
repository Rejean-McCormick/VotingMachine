**Doc 2A — Common Variables: Core Parameters**  
**Scope.** Core parameters that shape outcomes: **Ballot (001–007)**, **Allocation (010–015)**, **Thresholds & Double-Majority (020–027)**, **Aggregation (030–031)**, and **Frontier core (040, 042\)**.  
**Rule.** All percentages are **integer %** (e.g., 55 means 55%). Defaults are in **bold**.  
**Non-variable canonicals.** Some rules are fixed constants (see section F).  
---

**A) Ballot**

| ID | Name | Allowed values | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-001** | ballot\_type | plurality | approval | score | ranked\_irv | ranked\_condorcet | **approval** | — | Selects tabulation family. |
| **VM-VAR-002** | score\_scale\_min | integer 0..10 | **0** | ballot\_type \= score | Lower bound of score scale. |
| **VM-VAR-003** | score\_scale\_max | integer 1..10 and \> min | **5** | ballot\_type \= score | Upper bound of score scale. |
| **VM-VAR-004** | score\_normalization | off | linear | **off** | ballot\_type \= score | Per-ballot normalization if linear. |
| **VM-VAR-005** | condorcet\_completion | schulze | minimax | **schulze** | ballot\_type \= ranked\_condorcet | Completion rule when no strict Condorcet winner. |
| **VM-VAR-006** | ranked\_exhaustion\_policy | *(fixed)* reduce\_continuing\_denominator | *(fixed)* | ballot\_type \= ranked\_irv | Policy is fixed; listed for clarity. |
| **VM-VAR-007** | include\_blank\_in\_denominator | on | off | **off** | any ballot | If on, blanks count in majority denominators. |

---

**B) Allocation (unit-level seats/power)**

| ID | Name | Allowed values | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-010** | allocation\_method | winner\_take\_all | proportional\_favor\_big | proportional\_favor\_small | largest\_remainder | mixed\_local\_correction | **proportional\_favor\_small** | — | D’Hondt/Sainte-Laguë/LR/MMP families. |
| **VM-VAR-011** | use\_unit\_magnitudes | on | off | **on** | — | Respect Unit.magnitude where applicable. |
| **VM-VAR-012** | pr\_entry\_threshold\_pct | integer % 0..10 | **0** | proportional methods | Entry floor for PR. |
| **VM-VAR-013** | mlc\_topup\_share\_pct | integer % 0..60 | **30** | allocation\_method \= mixed\_local\_correction | Size of MMP top-up tier. |
| **VM-VAR-014** | overhang\_policy | allow\_overhang | compensate\_others | add\_total\_seats | **allow\_overhang** | allocation\_method \= mixed\_local\_correction | Overhang handling. |
| **VM-VAR-015** | target\_share\_basis | *(v1 fixed)* natural\_vote\_share | **natural\_vote\_share** | MMP | Basis for target shares in MMP. |

---

**C) Thresholds, Quorum & Double-Majority**

| ID | Name | Allowed values | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-020** | quorum\_global\_pct | integer % 0..100 | **50** | — | National turnout vs eligible roll. |
| **VM-VAR-021** | quorum\_per\_unit\_pct | integer % 0..100 | **0** | — | Per-unit turnout rule; see scope note below. |
| **VM-VAR-022** | national\_majority\_pct | integer % 50..75 | **55** | — | Majority / supermajority at national level. |
| **VM-VAR-023** | regional\_majority\_pct | integer % 50..75 | **55** | double\_majority\_enabled \= on | Regional/affected-family threshold. |
| **VM-VAR-024** | double\_majority\_enabled | on | off | **on** | — | Require national **and** affected-family majority. |
| **VM-VAR-025** | symmetry\_enabled | on | off | **on** | — | Mirror fairness for change vs status-quo. |
| **VM-VAR-026** | affected\_region\_family\_mode | by\_list | by\_tag | by\_proposed\_change | **by\_proposed\_change** | double\_majority\_enabled \= on | How to define the affected family. |
| **VM-VAR-027** | affected\_region\_family\_ref | list of Unit IDs **or** a tag | *(none)* | mode ∈ {by\_list, by\_tag} | Required when mode is by\_list/by\_tag. |

**Scope note (021).** The *per-unit quorum scope* (frontier-only vs frontier-and-family exclusion) is defined alongside advanced frontier options (see Part C).  
---

**D) Aggregation**

| ID | Name | Allowed values | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-030** | weighting\_method | equal\_unit | population\_baseline | **population\_baseline** | — | How unit results roll up. |
| **VM-VAR-031** | aggregate\_level | country *(v1 fixed)* | **country** | — | Aggregation level is fixed to country in v1. |

---

**E) Frontier (core)**

| ID | Name | Allowed values / shape | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-040** | frontier\_mode | none | sliding\_scale | autonomy\_ladder | **none** | — | Enables frontier mapping after gates. |
| **VM-VAR-042** | frontier\_bands | **Ordered, non-overlapping band list**; each band defines a support interval and optional action/AP id | *(no single default)* | frontier\_mode ≠ none | Validated for order, non-overlap; binary behavior \= single cutoff band. |

*(Advanced adjacency/island controls live in Part C: 047/048.)*  
---

**F) Fixed canonical rules (not variables)**

* **Approval gate denominator:** When ballot\_type \= approval, legitimacy **support %** is the **approval rate**  
  approvals\_for\_change / valid\_ballots (fixed; not configurable).  
* **Rounding for internal comparisons:** *round half to even* at defined decision points.  
* **IRV exhaustion policy:** reduce\_continuing\_denominator (fixed).  
* **Stable ordering:** Units by **Unit ID**; Options by **Option.order\_index**, then by ID.  
* **Offline/determinism:** No network at runtime; canonical JSON; identical outputs for identical inputs.

---

---

**Doc 2B — Operational Defaults & Determinism Controls**  
**Scope.** Parameters that shape *how* the engine behaves operationally and how results are presented, without altering the core formula except in bona fide tie situations.  
**IDs covered here:** **032–033 (ties/RNG), 044–046 (reporting/presentation)**.  
**Out of scope:** Core variables (see Doc 2A), advanced frontier/adjacency (see Doc 2C).  
---

**A) Ties & RNG**

| ID | Name | Allowed values / type | Default | Notes |
| :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-032** | tie\_policy | status\_quo | deterministic | random | **status\_quo** | Applies **only** when a blocking tie occurs (e.g., WTA winner, last seat, IRV elimination). deterministic uses **Option.order\_index** (fixed; not configurable). |
| **VM-VAR-033** | tie\_seed | integer ≥ 0 | **0** | Used **only** if tie\_policy \= random. Seeds a deterministic stream RNG (ChaCha20). Echoed in **RunRecord** and TieLog. |

*Notes.* No VM-VAR-050/051/052. The deterministic key is **always** Option.order\_index (no separate variable).  
---

**B) Reporting & Labeling**

| ID | Name | Allowed values / type | Default | Notes |
| :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-044** | default\_majority\_label\_threshold | integer % 0..100 | **50** | Presentation-only guard used by the report when deciding whether to label an outcome “Decisive” under the **fixed** policy. Does not change gate math (see Doc 2A). |
| **VM-VAR-045** | decisiveness\_label\_policy | fixed | dynamic\_margin | **fixed** | fixed: use **044** as the single cutoff. dynamic\_margin: may tighten/relax around the core gates and consider flags (e.g., mediation/enclave) for “Marginal” labeling. Exact presentation rules live in Doc 7\. |
| **VM-VAR-046** | unit\_display\_language | string (BCP-47 tag) | auto | **auto** | UI/report display language for Unit names. Presentation-only; no effect on outcomes. |

*Notes.* Labeling variables affect **reporting language only**. Core pass/fail outcomes are determined by Doc 2A gates and are not altered here.  
---

**C) Clarifications & Placement**

* **Frontier variables.** VM-VAR-040 frontier\_mode and VM-VAR-042 frontier\_bands live in **Doc 2A** (core).  
  Adjacency/island controls **047/048** live in **Doc 2C** (advanced).  
* **Per-unit quorum scope.** The scope selector for per-unit quorum (used by gates/frontier) is specified alongside advanced frontier options in **Doc 2C**.  
* **Approval denominator.** For approval ballots, the legitimacy **support %** uses the **approval rate** approvals\_for\_change / valid\_ballots. This is a **fixed canonical rule**, not a variable (see Doc 2A).

---

**D) Reserved / Not used here**

* **029** reserved in **Doc 2C** for symmetry\_exceptions.  
* **050–052**: *not used* (legacy numbering superseded by **032–033**).

---

Here’s a clean rewrite of **Doc 2C — Advanced & Special Rules** that aligns with Parts A/B and the fixes we’ve been applying.  
---

**Doc 2C — Advanced & Special Rules**  
**Scope.** Controls that refine gate scoping, eligibility/roll policy, frontier adjacency/island handling, symmetry exceptions, and executive-specific toggles. These settings interact with the **core** variables in Doc 2A and the **operational** controls in Doc 2B.  
**Rule.** All percentages are **integer %**. Defaults are in **bold**. Items here change outcomes when activated (except where noted as presentation-only).  
---

**A) Double-Majority Family & Quorum Scope**

| ID | Name | Allowed values / type | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-026** | affected\_region\_family\_mode | by\_list | by\_tag | by\_proposed\_change | **by\_proposed\_change** | double\_majority\_enabled \= on | How the affected family is defined for regional majority. |
| **VM-VAR-027** | affected\_region\_family\_ref | list of Unit IDs **or** a single tag | *(none)* | mode ∈ {by\_list, by\_tag} | Required when mode is by\_list/by\_tag. Must resolve to **non-empty** family. |
| **VM-VAR-021\_scope** | quorum\_per\_unit\_scope | frontier\_only | frontier\_and\_family | **frontier\_only** | quorum\_per\_unit\_pct \> 0 | When a Unit fails the per-unit quorum, decide whether it (a) is blocked only for frontier status changes (frontier\_only), or (b) is also excluded from the affected-family calculations (frontier\_and\_family). |

**Constraint (no-frontier double-majority).** If **double\_majority\_enabled \= on** **and** **frontier\_mode \= none**, then:

* **affected\_region\_family\_mode must be by\_list or by\_tag,** and  
* **affected\_region\_family\_ref must be a non-empty set** (IDs or a tag that resolves to IDs).  
  If these are not met, the run is **Invalid** (family undefined).

---

**B) Eligibility & Rolls**

| ID | Name | Allowed values / type | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-028** | roll\_inclusion\_policy | residents\_only | residents\_plus\_displaced | custom:list | **residents\_only** | — | Defines who is included in the **eligible roll** counts used for quorum/turnout. If custom:list, a concrete list is provided alongside the run context. |

*(Presentation of this policy appears in the report; the **value here affects outcomes** via turnout/quorum.)*  
---

**C) Frontier Adjacency & Islands**  
These apply **only when** frontier\_mode ≠ none (see Doc 2A for frontier\_mode and frontier\_bands).

| ID | Name | Allowed values / type | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-047** | contiguity\_modes\_allowed | subset of {land, bridge, water} (non-empty) | **{land, bridge}** | frontier\_mode ≠ none | Which adjacency edge types are considered *connecting* when forming contiguous blocks for status changes. |
| **VM-VAR-048** | island\_exception\_rule | none | ferry\_allowed | corridor\_required | **none** | frontier\_mode ≠ none | How to treat island/peninsula cases: allow water links, or require a designated corridor, etc. |

**Behavioral notes.**

* Contiguity is computed using **only** the allowed edge types; units that meet the band thresholds but are not connected to a qualifying component are flagged for **mediation** (no change).  
* island\_exception\_rule modifies how water separations are handled (e.g., allow ferry to connect islands).

---

**D) Symmetry Exceptions**

| ID | Name | Allowed values / type | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-029** | symmetry\_exceptions | list of Unit IDs **or** a tag \+ short rationale | *(empty)* | symmetry\_enabled \= on | Records explicit, documented exceptions where the mirrored rule is not applied. If non-empty, report should surface the rationale. |

---

**E) Executive Context**

| ID | Name | Allowed values / type | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-073** | executive\_double\_majority\_enabled | on | off | **off** | executive ballot present | When on, executive results are subject to **double-majority** checks (national \+ affected family) using the same thresholds and family rules as legislative runs. |

*(Whether an “executive ballot” exists is part of the run context; this toggle determines if executive outcomes also require double-majority.)*  
---

**F) MMP Clarifications (when MLP/MMP is used)**  
These reaffirm constraints defined in Doc 2A’s Allocation table; listed here to avoid drift in implementations that surface them as “advanced” toggles.

| ID | Name | Allowed values / type | Default | Depends on | Notes |
| :---- | :---- | :---- | :---- | :---- | :---- |
| **VM-VAR-016** | mlc\_correction\_level | national | regional | **national** | allocation\_method \= mixed\_local\_correction | No district option in v1. |
| **VM-VAR-017** | total\_seats\_model | fixed\_total | variable\_add\_seats | **fixed\_total** | allocation\_method \= mixed\_local\_correction | Naming is explicit; replaces older “expand\_on\_overhang”. |

---

**G) Canonical & Determinism Notes (non-variables)**

* **Approval gate denominator** is fixed to **valid ballots** (approval rate), not configurable.  
* **Stable ordering** (Units by ID; Options by order\_index then ID) applies when forming families, iterating units, and reducing components.  
* **Tie handling** is governed by **VM-VAR-032/033** (Doc 2B); no parallel RNG and seeds are recorded.

---

This Part C completes the variable set:

* Core (A): ballot/allocation/gates/aggregation/frontier-mode & bands  
* Operational (B): ties/RNG, labeling/presentation  
* Advanced (C): scoping, rolls, frontier adjacency/islands, symmetry exceptions, executive, and MMP clarifications

If you want, I can drop this into your Doc 2 file as the replacement for Part C.  
