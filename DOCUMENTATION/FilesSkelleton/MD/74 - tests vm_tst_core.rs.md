
# Pre-Coding Essentials — 74/89

**Component:** `tests/vm_tst_core.rs`
**Formula/Engine:** VM-ENGINE v0

## 1) Goal & Success

* **Goal:** Lock baseline behaviors of **tabulation**, **allocation**, **gate denominators**, and **pipeline step order/stop rules** using canonical Part-0 fixtures and Doc 6A core cases.
* **Success:** Tests pass identically on Win/macOS/Linux with **no network I/O**; approvals use **approval-rate** denominator; PR/WTA allocations match locked vectors.

## 2) Scope

* **In:** Pipeline orchestration (LOAD→…→BUILD\_\*), plurality/approval/score tabulation smoke, Sainte-Laguë & WTA allocations, the 34/33/33 convergence case, and approval gate denominator rule.
* **Out:** Ranked specifics (in `vm_tst_ranked.rs`), MMP (in `vm_tst_mmp.rs`), cross-OS byte hashes (in `determinism.rs`).

## 3) Inputs → Outputs

**Inputs (fixtures, local only):**

* `fixtures/annex_b/part_0/division_registry.json` (70)
* `fixtures/annex_b/part_0/ballots.json` (71) or tally variant
* `fixtures/annex_b/part_0/parameter_set.json` (69)
* Optional manifest (72) for manifest-mode runs

**Outputs (asserted by tests):**

* Per-option **seats/power** from UnitAllocation
* **Gate panel** raw values & pass/fail (quorum, majority)
* **Label** (Decisive/Marginal/Invalid)
* Invariants (e.g., Σseats==m; WTA==100%)

## 4) Entities (minimal)

* Pipeline entry (49), stage outputs (52–56), report label helper (57), build artifacts (58–59).
* Types: `UnitAllocation`, `AggregateResults`, `LegitimacyReport`, `DecisivenessLabel`.

## 5) Variables (used/assumed)

* VM-VAR-001 (ballot type), 010 (allocation), 012 (PR threshold), 020/022 (quorum/majority), 030/031 (weighting/aggregate level).
* **Fixed rule:** approval **support % = approvals\_for\_change / valid\_ballots** (denominator rule for gates).

## 6) Functions (test signatures only — no bodies)

```rust
// Harness helpers (signatures only)
fn run_with_part0_fixtures(mode: TestMode) -> TestArtifacts;
fn seats_of(alloc: &UnitAllocation) -> Vec<(OptionId, u32)>;
fn power_of(alloc: &UnitAllocation) -> Vec<(OptionId, u32)>;
fn gate_values(legit: &LegitimacyReport) -> GateSnapshot;
fn label_of(res: &ResultDb) -> DecisivenessLabel;

// Core tests
#[test] fn vm_tst_001_pr_baseline_sainte_lague();      // A/B/C/D=10/20/30/40, m=10 → 1/2/3/4
#[test] fn vm_tst_002_wta_winner_take_all_m1();        // plurality, m=1 → D gets 100% power
#[test] fn vm_tst_003_method_convergence_lr_vs_ha();   // 34/33/33, m=7 → 3/2/2 for LR & HA
#[test] fn vm_tst_004_gate_denominator_approval_rate();// support = approvals_change / valid_ballots
#[test] fn vm_tst_005_pipeline_order_and_stop_rules(); // validate fail & gate fail paths

// Optional tiny utilities
fn assert_sum_seats(seats: &[(OptionId,u32)], m: u32);
fn assert_wta_power_100(power: &[(OptionId,u32)]);
fn assert_ge_majority(value_pp1: i32, threshold: i32); // ≥ rule
```

## 7) Test Logic (Arrange → Act → Assert)

### VM-TST-001 — PR baseline (Sainte-Laguë)

* **Arrange:** approvals A/B/C/D = **10/20/30/40**; Unit magnitude **m=10**; allocation **Sainte-Laguë**, threshold **0%**.
* **Act:** run full pipeline (LOAD→…→ALLOCATE).
* **Assert:** seats **1/2/3/4** in canonical option order; Σseats==10; label **Decisive**; no gates fail.

### VM-TST-002 — WTA (winner\_take\_all, m=1)

* **Arrange:** plurality votes with clear top **D**; **m=1**; allocation **WTA**.
* **Act:** run pipeline.
* **Assert:** **D** receives **100% power**; enforce **m=1** constraint; label **Decisive**.

### VM-TST-003 — Method convergence (HA vs LR)

* **Arrange:** shares **A/B/C = 34/33/33**, **m=7**; run **Sainte-Laguë**, **D’Hondt**, and **Largest Remainder** on the same tallies/turnout.
* **Act:** allocate seats with each method.
* **Assert:** all three return **3/2/2**; Σseats==7 for each; deterministic order.

### VM-TST-004 — Approval gate denominator

* **Arrange:** approval ballot where **Change approvals / valid\_ballots = 55.0%** (edge).
* **Act:** APPLY\_DECISION\_RULES.
* **Assert:** majority **Pass** via **≥** at **55.0% vs 55%**; verify denominator is **valid\_ballots** (not approvals share); label **Decisive**.

### VM-TST-005 — Pipeline order & stop/continue

* **Arrange A (validation fail):** craft a structural error (e.g., hierarchy violation or tally > ballots\_cast).

* **Act:** run pipeline.

* **Assert:** **Invalid** label; stages **TABULATE..MAP\_FRONTIER** skipped; **Result/RunRecord** still built with reasons.

* **Arrange B (gate fail):** quorum below threshold (e.g., 48% < 50%).

* **Act:** run pipeline.

* **Assert:** **Invalid** (gate failed); **Frontier** skipped; gate panel shows ❌ Quorum; artifacts built.

## 8) State Flow (per test)

`LOAD → VALIDATE → [TABULATE → ALLOCATE → AGGREGATE] → APPLY_DECISION_RULES → [MAP_FRONTIER?] → [RESOLVE_TIES?] → LABEL → BUILD_RESULT → BUILD_RUN_RECORD`.

## 9) Determinism & Numeric Rules

* Integer/rational comparisons only; **≥** cutoffs; **one-decimal appears only in report**, not used for asserts.
* Stable ordering (Units by ID; Options by `(order_index,id)`); **no network**; **no OS RNG**.
* WTA asserts **100** (percent power), PR asserts seat sums equal **m**.

## 10) Edge Cases & Failure Policy

* If Σvalid tallies + invalid\_or\_blank > ballots\_cast, expect **validation fail** path (A).
* All zero tallies with m>0 under PR: allocation degenerate but still deterministic; not core path here.
* Threshold filtering (VM-VAR-012) set to **0%** for baseline tests.

## 11) Test Data & Expectations (concise table)

| Test | Ballot                         | Inputs (key)                     | Method          | m  | Expected                    |
| ---- | ------------------------------ | -------------------------------- | --------------- | -- | --------------------------- |
| 001  | approval                       | A/B/C/D=10/20/30/40              | Sainte-Laguë    | 10 | Seats 1/2/3/4; Decisive     |
| 002  | plurality                      | D top                            | WTA             | 1  | D → 100% power; Decisive    |
| 003  | approval/plurality-like shares | 34/33/33                         | SL, D’Hondt, LR | 7  | Seats 3/2/2 for all         |
| 004  | approval                       | Change approvals / valid = 55.0% | gates           | —  | Majority Pass (≥); Decisive |
| 005A | any                            | structural/tally error           | —               | —  | Invalid; stages 3–8 skipped |
| 005B | any                            | turnout < quorum                 | gates           | —  | Invalid; Frontier skipped   |

## 12) Helpers & Fixtures (paths/placeholders)

* **Fixtures root:** `fixtures/annex_b/part_0/`

  * `division_registry.json` (tree + magnitudes)
  * `ballots.json` (or tally variant)
  * `parameter_set.json` (baseline vars)
  * optional `manifest.json` for manifest-mode smoke

**Helper intentions (no code):**

* `run_with_part0_fixtures(mode)` — dispatches CLI/lib to produce artifacts for a given scenario.
* `assert_sum_seats`, `assert_wta_power_100`, `assert_ge_majority` — invariant checks.
* Snapshot extraction for gate panel & labels without involving report rounding.

## 13) Pass Criteria

* All five tests pass on Win/macOS/Linux, offline.
* Allocations match locked vectors; gate denominator verified as approval-rate; pipeline stop rules observed.
* No panics; no network; no non-deterministic sources.
