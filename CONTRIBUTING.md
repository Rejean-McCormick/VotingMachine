````markdown
# CONTRIBUTING.md — VM-ENGINE v0
_Component 11/89 • How to propose changes without breaking specs, determinism, or offline policy._

## Principles (spec-first)
- **Docs 1–7 + Annex A/B/C are normative.** If code conflicts with the specs, fix the code (and its skeleton sheet).  
- **Normative changes require spec edits first.** Open an ADR and update the relevant Doc/Annex before code lands.  
- **Determinism is a hard gate.** Same inputs (+ same seed when random ties are enabled) must produce **byte-identical** outputs.

---

## Prerequisites
- Rust toolchain pinned by `rust-toolchain.toml` (`rustup show` should match).  
- `cargo` available; `pre-commit` optional but recommended.  
- Build **offline by default** (see `.cargo/config.toml`). For a first fetch: `CARGO_NET_OFFLINE=0 cargo fetch`.

---

## Workflow overview
1. **Create a short topic branch** from `main`.  
2. Make changes with **Conventional Commits** and **spec references**:
   - `feat(vm_algo): implement Doc4A §2.2 step order (ref VM-TST-002)`
   - `fix(vm_report): one-decimal rounding per Doc7 §5`
3. **Run local gates** (fast → heavy): `pre-commit run -a` → `git push` (triggers pre-push hooks) → open PR.
4. In the PR description, include:
   - Spec refs (e.g., *Doc 4 — Algorithm Specification, §2.2*).  
   - Affected tests/fixtures (e.g., *VM-TST-001/003*).  
   - Whether behavior is normative (FID-affecting) or not.

---

## Formatting, linting, and hygiene
- **Rust format**: `cargo fmt --all -- --check`  
- **Clippy (deny warnings)**: `cargo clippy --all-targets -- -D warnings`  
- **LF/UTF-8 only**: enforced by `.gitattributes` and hooks.  
- **Canonical JSON**: UTF-8, **LF**, **sorted keys** (no ad-hoc pretty printing).  
- **Editor defaults**: see `.editorconfig`.

> Tip: install and enable `pre-commit`; the repo includes `.pre-commit-config.yaml` with fast checks and a pre-push smoke test.

---

## Tests you must run locally
```bash
# Unit/integration tests
cargo test --locked --workspace

# Minimal Annex B fixture (example)
./target/release/vm_cli run \
  --manifest fixtures/annex_b/VM-TST-001/manifest.json \
  --output artifacts/tst001

# Determinism smoke (same seed → identical bytes)
SEED=42
./target/release/vm_cli run --manifest fixtures/annex_b/VM-TST-001/manifest.json --rng-seed $SEED --output artifacts/a
./target/release/vm_cli run --manifest fixtures/annex_b/VM-TST-001/manifest.json --rng-seed $SEED --output artifacts/b
cmp -s artifacts/a/result.json artifacts/b/result.json
cmp -s artifacts/a/run_record.json artifacts/b/run_record.json
````

**Offline rule:** once dependencies are fetched or vendored, builds/tests must pass with `CARGO_NET_OFFLINE=1`.

---

## Changing schemas, fixtures, or specs

### Schemas (`schemas/**`)

1. Update schema shape (keep canonical ordering stable).
2. Update loader/validator code and any impacted tests.
3. Re-run affected fixtures; if expected outputs must change, explain **why** (spec-driven) in the PR.

### Fixtures (`fixtures/annex_b/**`)

* **Never** change fixtures merely to “make a test pass”. Fix the code or the spec.
* When normative outputs change, regenerate expected `result.json`/`run_record.json` and document the spec deltas.

### Specs & variables (Doc 2 / Annex A)

* **VM-VAR additions/modifications** only via PRs that:

  * Update **Doc 2** (definitions, ranges, defaults).
  * Update **Annex A** (canonical reference table, FID membership).
  * Include migration notes and explicitly call out whether the change is **FID-affecting**.

---

## Algorithm & determinism rules (must-follow)

* **Ordering**: Follow Doc 4/5 step order; do not rely on map/hash iteration order.
* **Math**: Integers/rationals only for comparisons; avoid floats in outcome logic.
* **Rounding**: Apply **once**, at the defined comparison/presentation points (reports show **one decimal**).
* **Ties**:

  * Default deterministic policy: by `order_index` (spec).
  * If `tie_policy=random` (VM-VAR-050), use the provided seed (VM-VAR-052, **non-FID**) and record it in `RunRecord` **only when a random tie actually occurred**.
* **IDs**: `RES:<hash>` / `RUN:<ts>-<hash>` computed over **canonical bytes** (UTF-8/LF/sorted keys; arrays in defined order).

---

## Versioning & Formula ID (FID)

* **Engine version**: bump when binaries/public API change **without** altering outcomes.
* **Formula ID**: bump when any **normative** behavior changes (variables, constants, steps, ordering).

  * Include the regenerated FID manifest and a short ADR summarizing the rationale and impacted tests.

---

## Commit & PR conventions

* **Conventional Commits**: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `build:`, `chore:`
* Keep subject ≤ 72 chars; wrap body at \~100 cols.
* Include spec refs and test IDs in the body:

  ```
  Implements Doc4A §2.2; updates Annex A (VM-VAR-050 note).
  Affects: VM-TST-002/003; FID bumped: yes.
  ```

---

## ADRs (Architecture/Decision Records)

* Place ADRs in `docs/adr/NNN-title.md` with:

  * Context → Decision → Consequences → Spec sections affected → Tests affected → FID impact (yes/no).

---

## Reporting (Doc 7)

* Renderer reads canonical artifacts; **no recomputation**.
* Show **one-decimal** percentages (round-half-up) and keep section order fixed.
* If adding sections/visual rules, update **Doc 7** first and add matching tests.

---

## Review & merge flow

1. Author: run local hooks/tests (offline).
2. PR: include spec refs, tests touched, FID/engine version impact, ADR link if normative.
3. Reviewer: verify spec alignment, determinism (double-run bytes), and offline policy.
4. CI mirrors local gates. Merge when **all green**.

---

## Edge cases → reject or fix before review

* Random tie policy enabled **without** seed validation.
* WTA with `magnitude ≠ 1`.
* CRLF introduced or unsorted JSON in canonical artifacts.
* Any dependency that forces network at build/test time post-vendoring.

---

## Pre-PR checklist (must pass)

* [ ] `cargo fmt --all -- --check`
* [ ] `cargo clippy --all-targets -- -D warnings`
* [ ] `cargo test --locked --workspace`
* [ ] Minimal Annex-B fixture(s) pass and **determinism smoke** is byte-identical with same seed
* [ ] Spec references included (Doc/Annex sections)
* [ ] If normative: ADR added, FID impact stated, fixtures/expectations updated

---

## Post-merge notes

* Keep `CHANGELOG.md` with sections: **Spec compliance** and **Behavioral changes**.
* Release bundles in `dist/` must include `LICENSE` (and `NOTICE` if Apache terms are exercised) and third-party attributions.

```
```
