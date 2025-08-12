<!-- Converted from: 11 - CONTRIBUTING.md, Version FormulaID VM-ENGINE v0).docx on 2025-08-12T18:20:45.848132Z -->

```
Pre-Coding Essentials (Component: CONTRIBUTING.md, Version/FormulaID: VM-ENGINE v0) — 11/89
1) Goal & Success
Goal: Document how to propose changes without breaking specs, determinism, or offline policy; make reviews mechanical.
Success: Every change references Docs 1–7 / Annex A–B, passes hooks/tests, and keeps canonical artifacts reproducible.
2) Scope
In scope: workflow, commit/PR conventions, formatting/linting, tests/fixtures, schema/spec change process, Formula ID policy.
Out of scope: legal/security policy (lives in SECURITY.md), release packaging.
3) Inputs → Outputs
Inputs: Proposed code/spec/schema changes, Annex B fixtures, pre-commit hooks, vm_cli.
Outputs: Reviewed commits/PRs that pass format/lint/tests, update fixtures/hashes when warranted, and document spec alignment.
4) Entities/Tables (minimal)
5) Variables (process toggles)
6) Functions (signatures only)
(Doc file; no code functions.)
7) Algorithm Outline (document sections to include)
Principles (spec-first).
Ultimate references: Doc 1–7 + Annex A/B. If code conflicts, fix code and update sheets; if spec needs evolution, open an ADR and bump Formula ID rules as required.
Prereqs.
Rust (pinned via rust-toolchain.toml), cargo, pre-commit hooks enabled, ability to run offline.
Branch & commit style.
Short topic branches; Conventional Commits (feat:, fix:, docs:, test:, refactor:).
Include spec refs (e.g., Doc4A §2.2) and test IDs (VM-TST-001) in the body.
Formatting & lint.
cargo fmt -- --check, cargo clippy -D warnings.
LF-only, UTF-8, sorted JSON keys; follow .editorconfig, .gitattributes.
Tests you must run locally.
cargo test --locked --workspace.
Minimal fixtures (Annex B Part 0/1): run twice with same --rng-seed → identical RES:/RUN: IDs.
No network at runtime; builds should succeed with CARGO_NET_OFFLINE=1 once vendored/fetched.
Schemas & fixtures changes.
Never change fixture semantics to “make a test pass.” If a spec bug: open issue + ADR candidate.
When schema shape changes, update: schema file → loader/validator → fixtures → tests → report mapping.
Algorithm/variable changes.
If it alters outcomes, it is normative → it must be reflected in Annex A (variable/constant) and may change Formula ID.
Add/modify VM-VAR only via PR that updates Doc 2 and Annex A; include migration notes.
Tie-breaks, rounding, determinism.
Deterministic ties by Option.order_index unless tie_policy=random (seeded).
Integer/rational math; round half to even only at defined comparison points.
Canonical JSON: UTF-8, LF, sorted keys; UTC timestamps.
Reporting.
One-decimal percentages; mandatory “approval rate” sentence for approval ballots; section order fixed (Doc 7A).
When to bump versions.
Engine version: any non-behavioral refactor that changes binary/API.
Formula ID: any normative change (variables, constants, algorithmic steps). Include regenerated FID manifest in PR.
Update CHANGELOG.md with “Spec compliance” and “Behavioral changes” entries.
8) State Flow (author → review → merge)
Author runs hooks/tests offline → opens PR referencing spec sections and tests → reviewer checks spec alignment & determinism → CI mirrors local checks → merge if clean.
9) Determinism & Numeric Rules (to restate in doc)
Same inputs + same seed ⇒ byte-identical outputs (Result, RunRecord).
No OS RNG/time in algorithms; no floats for comparisons; presentation rounding in report only.
10) Edge Cases & Failure Policy
Missing RNG seed while tie_policy=random ⇒ reject PR or add validation.
WTA with magnitude≠1 ⇒ validation must fail; add/keep tests.
CRLF introduced or unsorted JSON ⇒ hooks must block; fix before review.
11) Test Checklist (must pass before PR)
pre-commit (fast) passes on staged files.
pre-push (clippy/tests/determinism smoke) passes.
Run Annex B Part 1: VM-TST-001/002/003 winners/labels match expected.
Two identical runs (same seed) produce identical RES:/RUN: IDs.
Doc content hint: keep CONTRIBUTING.md practical (≤ ~200 lines). Link out to Docs 1–7, Annex A/B, and any ADR directory for deeper policy changes.
```
