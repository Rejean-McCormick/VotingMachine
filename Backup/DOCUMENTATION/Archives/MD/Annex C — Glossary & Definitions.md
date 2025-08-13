# **Annex C — Glossary & Definitions**

**Scope.** Compact, neutral definitions of terms used across Docs 1–7. Where helpful, cross-references are shown as *(Doc §)*. All percentages in variables are **integer %**. Internal math uses exact integer/rational comparisons; reporting rounds to **one decimal**; comparisons use **round half to even** *(Docs 3A, 4A, 7A)*.

---

# **Ballots & Tabulation**

**Approval ballot.** Voters mark any option(s) they approve. Support for gates uses the **approval rate** denominator *(below)*. *(Doc 4A §2.1)*

**Approval rate (gate denominator).** `approvals_for_change / valid_ballots`. Fixed; blanks/invalids are not in the divisor. *(Doc 4A §3, Doc 4C §1.2)*

**Valid ballots.** `ballots_cast − invalid_or_blank` per unit. Used as the denominator for approval gates and most tally rates. *(Doc 4A §1)*

**Include blanks in denominator.** Optional display/tabulation switch; if **on**, some reported percentages may use `valid + blank`. Does **not** affect approval gate denominator. *(VM-VAR-007; Doc 4A §3)*

**Plurality ballot.** One mark per ballot; most votes wins (used by WTA). *(Doc 4A §2.0)*

**Score ballot.** Each option receives a numeric score; may use normalization. *(VM-VAR-002..004; Doc 4A §2.3)*

**Ranked IRV.** Instant-runoff with eliminations and transfers; **exhausted** ballots are removed from the **continuing** denominator after their last valid preference. *(VM-VAR-006; Doc 4A §2.4)*

**Exhausted ballots (IRV).** Ballots that list no remaining active options; excluded from subsequent round denominators. *(Doc 4A §2.4)*

**Condorcet / completion.** Pairwise majority relation; if cycles exist, a completion method (e.g., **Schulze**) selects the winner. *(VM-VAR-005; Doc 4A §2.5)*

---

# **Allocation & Aggregation**

**Unit magnitude.** Seats associated with a unit (`magnitude ≥ 1`). WTA requires `magnitude = 1`. *(Doc 1B; Doc 4B §1)*

**WTA (winner-take-all).** Highest votes wins the single seat; others receive none. *(VM-VAR-010; Doc 4B §1)*

**Highest averages (PR).** Divisor methods: **D’Hondt** (favor\_big) and **Sainte-Laguë** (favor\_small). *(VM-VAR-010; Doc 4B §2)*

**Largest remainder (PR).** Quotas \+ remainder assignment. *(VM-VAR-010; Doc 4B §2.4)*

**PR entry threshold.** Minimum share (in %) to enter seat allocation. *(VM-VAR-012; Doc 4A §4, Doc 4B)*

**Weighting method.** How unit results aggregate nationally: **equal\_unit** (each unit \= 1\) or **population\_baseline** (weights by baseline). *(VM-VAR-030; Doc 4B §4)*

**Population baseline / year.** Reference population weight and its year; required if population weighting is used. *(Doc 1B; VM-VAR-030)*

**Aggregate level.** Current version aggregates at **country** level. *(VM-VAR-031; Doc 4B §4)*

---

# **Mixed-Member Proportional (MMP)**

**Mixed local correction (MMP).** Two tiers: local WTA seats \+ proportional top-ups toward a target share. *(VM-VAR-010=mixed\_local\_correction; Doc 4B §3)*

**Top-up share % (mlc\_topup\_share\_pct).** Portion of total seats assigned via top-ups. *(VM-VAR-013; Doc 4B §3.2)*

**Correction level.** Where proportional correction is computed: **national** or **regional**. Affects final totals. *(VM-VAR-016; Doc 4B §3.1; Tests 013\)*

**Overhang policy.** Handling when a party’s local wins exceed its target: **allow\_overhang**, **compensate\_others**, or **add\_total\_seats**. *(VM-VAR-014; Doc 4B §3.5)*

**Total seats model.** **fixed\_total** or **variable\_add\_seats** when overhang exists. *(VM-VAR-017; Doc 4B §3.5)*

**Target share basis.** Basis for proportional targets; v1 fixed to **natural\_vote\_share**. *(VM-VAR-015; Doc 4B §3.3)*

---

# **Legitimacy Gates & Families**

**Quorum (global).** Minimum turnout, computed from **eligible\_roll**: `Σ ballots_cast / Σ eligible_roll`. *(VM-VAR-020; Doc 4C §1.1)*

**Quorum (per-unit).** Optional local quorum; scope may be **frontier\_only** or **frontier\_and\_family**. *(VM-VAR-021, 021\_scope; Doc 4C §1.1/2.3)*

**Majority / supermajority.** Pass if support ≥ threshold (e.g., 55%). For approval ballots, support uses the **approval rate** denominator. *(VM-VAR-022/023; Doc 4C §1.2)*

**Double-majority.** National threshold **and** minimum support across the **affected-region family**. *(VM-VAR-024; Doc 4C §1.3)*

**Affected-region family.** The set of units subject to regional checks: **by\_list**, **by\_tag**, or **by\_proposed\_change**. When **frontier=none** and double-majority is **on**, must be **by\_list** or **by\_tag** with a non-empty reference. *(VM-VAR-026/027; Doc 4C §1.3)*

**Symmetry.** Mirror proposals are evaluated under identical rules; exceptions may be registered with rationale. *(VM-VAR-025/029; Doc 4C §1.4)*

**Roll inclusion policy.** Which residents are on the eligible roll: **residents\_only**, **residents\_plus\_displaced**, or **custom:list**. Printed in the report. *(VM-VAR-028; Doc 7A §Eligibility)*

---

# **Frontier Mapping**

**Frontier mode.** How territorial outcomes map: **none**, **binary\_cutoff**, **sliding\_scale**, **autonomy\_ladder**. *(VM-VAR-040; Doc 4C §2)*

**Cutoff %.** Threshold for change in **binary** mode. *(VM-VAR-041; Doc 4C §2.4a)*

**Bands.** Ordered, non-overlapping `{min_pct, max_pct, action}` definitions for **sliding**/**ladder** modes. *(VM-VAR-042; Doc 4C §2.4b/c)*

**Autonomy package (AP).** Named bundle of devolved powers; actions like `autonomy(AP:Base)` are mapped to concrete **AP IDs**. *(VM-VAR-046; Doc 4C §2.4c; Doc 1A)*

**Contiguity modes allowed.** Connection types permitted to form contiguous areas: subset of `{land, bridge, water}`. *(VM-VAR-047; Doc 4C §2.1)*

**Island exception rule.** Handling for isolated units: **none**, **ferry\_allowed**, **corridor\_required**. *(VM-VAR-048; Doc 4C §2.1)*

**Protected area.** Unit flagged as protected; change is blocked unless an override is allowed. *(Unit.protected\_area; VM-VAR-045; Doc 4C §2.2)*

**Mediation flagged.** Status indicating a unit met a support rule but violates contiguity/protection constraints; forces **Marginal** label. *(Doc 4C §2; Doc 7A §Frontier)*

---

# **Ties, Labels & Reporting**

**Tie policy.** How exact ties are resolved: **status\_quo**, **deterministic\_order** (by `Option.order_index`), or **random** (seeded). *(VM-VAR-050/051/052; Doc 4C §3)*

**RNG seed.** Fixed integer used when `tie_policy=random`; ensures reproducibility. *(VM-VAR-052; Docs 3A, 6C-020)*

**Decisiveness labels.**

* **Decisive:** all gates pass, national margin ≥ **VM-VAR-062**, and no mediation/protected flags.

* **Marginal:** gates pass but margin \< **VM-VAR-062** or any mediation/protected flags exist.

* **Invalid:** any gate fails. *(VM-VAR-062; Doc 4C §4; Doc 7A §Outcome)*

**Report precision.** One decimal place for presented percentages. *(VM-VAR-032/033; Doc 7A/7B)*

**Unit/option ordering (display).** Units by **Unit ID** (lexicographic); options by **Option.order\_index**, then ID. *(Docs 3A, 7A)*

**Sensitivity analysis.** Optional ±1/±5 pp comparisons; appears only if **CompareScenarios** ran. *(VM-VAR-035; VM-FUN-013; Doc 7A §Sensitivity)*

---

# **Data, IDs & Provenance**

**DivisionRegistry.** Canonical set of units (tree), eligible roll, baselines, and optional adjacency. *(Doc 1A/1B)*

**Adjacency.** Pair list of unit connections with type `{land, bridge, water}`. *(Doc 1A; Doc 4C §2.1)*

**Option.order\_index.** Stable integer to ensure deterministic ordering and tie breaks under `deterministic_order`. *(Doc 1B; Doc 4C §3)*

**BallotTally.** Per-unit counts by ballot type, with `invalid_or_blank`. *(Doc 1B; Doc 4A)*

**ParameterSet.** Concrete values for all **VM-VAR-\#\#\#** used in a run. *(Doc 2A/2B/2C; Doc 5A)*

**Result.** Final aggregated outcomes, labels, and (if requested) seat vectors. Has a stable **Result ID** derived from canonical bytes. *(Doc 1A; Doc 5A)*

**RunRecord.** Audit object: inputs, engine+formula identifiers, seeds, and checksums; used for reproducibility. *(Doc 5A/5C; Doc 3B)*

**FrontierMap.** Per-unit mapping outcomes and flags (mediation, protected). *(Doc 1A; Doc 4C §2; Doc 7A)*

**Formula ID.** Cryptographic hash of the **normative rule set** (primarily Docs 4A/4B/4C and declared defaults). Printed in reports and RunRecord. *(Doc 3B §Release)*

**Engine Version.** Semantic version of the implementation build; printed with Formula ID. *(Doc 3B §Release)*

---

# **Validation & Determinism (engine behavior)**

**Tally sanity rule.** For each unit: `Σ valid option tallies + invalid_or_blank ≤ ballots_cast`. *(Doc 5B VM-FUN-002)*

**WTA magnitude check.** If WTA is selected, all affected units must have `magnitude=1`. *(Doc 5B; VM-VAR-010)*

**Eligible roll presence.** Required when quorum \> 0 (global or per-unit). *(Doc 5B; VM-VAR-020/021)*

**Bands integrity.** Frontier bands must be ordered, non-overlapping; required AP mappings must exist. *(Doc 5B; VM-VAR-042/046)*

**Deterministic serialization.** Sorted keys, LF line endings, UTC timestamps; no network or time-dependent logic. *(Doc 3A/3B; Tests 019–020)*

---

This annex is **informative** and mirrors the **normative** rules in Docs **2A/2B/2C** (variables), **4A/4B/4C** (algorithm), **5A–C** (pipeline), **6A–C** (tests), and **7A/7B** (reporting).

