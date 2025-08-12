# **Doc 3A — Tech Platform & Determinism (Updated)**

## **1\) Purpose & scope**

Defines the **execution environment**, **determinism guarantees**, and **build/runtime constraints** for the engine and renderer. Normative where it affects reproducibility. Integrates prior addenda so no separate platform addendum is needed.

Outputs from identical inputs \+ ParameterSet (incl. seeds) MUST be **byte-identical** across OS/arch.

---

## **2\) Determinism targets (normative)**

* **Offline, pure computation**: no network, no nondeterministic external calls.

* **Locale-neutral**: behavior MUST NOT depend on OS locale/timezone.

* **Time-agnostic**: wall-clock/timezone MUST NOT affect any computed field (timestamps are metadata only, set in UTC).

* **Canonical JSON** and hashing as per Doc 1A §2.1–2.2.

* **Stable ordering**: all algorithmic arrays obey Doc 1A §5 (never rely on input or map iteration order).

---

## **3\) Runtime environment constraints**

* **Process I/O**

  * Inputs: file paths to **DivisionRegistry**, **BallotTally**, **ParameterSet**.

  * Outputs: `result.json`, `run_record.json`, optional `frontier_map.json` (Doc 1A §2.2 / §4).

  * No temp artifacts may influence output content; temp files are optional and ignored for hashing.

* **Filesystem**

  * Treat paths case-sensitively internally.

  * Normalize line endings to **LF** for all emitted JSON.

* **Concurrency**

  * Parallelism permitted, but **observable order** MUST match Doc 1A §5.

  * Reductions/aggregations MUST be order-stable (e.g., sort before fold; avoid nondeterministic hash-map iteration).

  * No data races that could change floating-point summation order.

* **Numeric model**

  * IEEE-754 semantics. Use deterministic rounding paths; avoid hardware/BLAS paths with nondeterministic reduction.

  * Percentages & shares formatting controlled by reporting rules (Doc 7). Internal precision is engine-defined but MUST be stable across builds.

---

## **4\) RNG profile for ties (normative)**

* **Controls**: `VM-VAR-050 tie_policy`, `VM-VAR-052 tie_seed` (051 reserved).

* **Seeding**: Initialize the run’s RNG with **exactly** `VM-VAR-052` (integer ≥ 0).

* **Usage**: Consume draws **only** when `tie_policy="random"` and a tie event actually requires resolution.

* **Event recording**: Each random tie creates a `RunRecord.ties[]` entry; `RunRecord.determinism.rng_seed` is present iff any random tie occurred (Doc 1A §4.5).

* **Reproducibility**: The RNG algorithm/profile is fixed by **Annex B (`rng_profile.json`)** to ensure identical sequences across languages/platforms.

Deterministic and deterministic\_order paths MUST NOT consult RNG.

---

## **5\) CLI contract & exit codes**

* **CLI**

  * `vm_cli --registry path --tally path --params path --out dir [--seed N overrides VM-VAR-052]`

  * MUST refuse unknown flags; MUST error if inputs are missing/invalid.

* **Exit codes**

  * `0` success (all artifacts emitted & hashes verified)

  * `2` validation error (Doc 1B domains/refs/order)

  * `3` hash/FID mismatch on self-verify

  * `4` runtime error (I/O, parse)

  * `5` spec violation (ordering, determinism, disallowed features)

---

## **6\) Build & release reproducibility (engine)**

* **Dependency pinning**: compiler/toolchain and libs MUST be version-pinned (lockfiles or exact versions).

* **Reproducible builds**: remove timestamps from binaries where possible; record `engine.build` metadata (e.g., VCS commit) in **RunRecord.engine**.

* **Hermeticity**: no optional system-wide plugins that can alter numeric behavior; all runtime feature flags must be explicit.

* **Verification**: a release MUST pass the full Doc 6 canonical test pack on all supported platforms before tagging (see Doc 3B for tag policy).

---

## **7\) Hashing, FID & manifest linkage**

* **SHA-256** for `result_id`, `run_id`, `frontier_id`, and input digests (Doc 1A §2.2, §4.5).

* **FID scope**: only outcome-affecting rules & variables (see **Annex A — Included**).  
   Presentation/report toggles (e.g., **VM-VAR-060..062**) are **excluded** from FID.

* **RunRecord.nm\_digest**: MUST include `nm_sha256` over the Normative Manifest used to compute FID, enabling independent recomputation.

---

## **8\) Logging & integrity checks**

* **Self-verify**: after emitting each artifact, recompute its sha256 and compare to the embedded ID; fail with code `3` on mismatch.

* **Determinism log** (optional JSON): MAY include timing and thread counts, but MUST NOT influence artifacts.

* **No network**: engine MUST refuse network I/O unless explicitly running in a non-deterministic debug mode (not for official runs).

---

## **9\) Security & trust boundaries**

* Treat all inputs as untrusted: validate schema (Doc 1B) before use.

* Sandboxed execution recommended for public data runs.

* No code-loading from inputs; ParameterSet and Registry are data only.

---

## **10\) Renderer constraints (Doc 7 interplay)**

* Renderer MUST consume **Result** and **RunRecord** only; it MUST NOT re-compute allocations.

* Renderer MAY use **FrontierMap** when present.

* Section ordering and visibility controlled by **Doc 2B (032–035)**; does **not** affect canonical JSON or FID.

---

## **11\) Conformance checklist**

* **C-PLAT-01**: No network calls during official runs.

* **C-PLAT-02**: Canonical JSON (UTF-8, LF, sorted keys) for all artifacts.

* **C-PLAT-03**: Ordering contract satisfied regardless of parallelism.

* **C-PLAT-04**: RNG seeded **only** from `VM-VAR-052`; identical sequences on all supported platforms.

* **C-PLAT-05**: Self-verification passes (`result_id`, `run_id`, `frontier_id`, input digests).

* **C-PLAT-06**: FID recomputation matches `Result.formula_id` and `RunRecord.formula_id`.

---

## **12\) Minimal example (CLI → artifacts)**

vm\_cli \--registry reg.json \--tally tally.json \--params params.json \--out ./run01  
\# Emits:  
\#   ./run01/result.json  
\#   ./run01/run\_record.json  
\#   ./run01/frontier\_map.json   (only if VM-VAR-034=true and feature used)  
\# run\_record.json contains:  
\#   engine { vendor, name, version, build }  
\#   inputs { \*\_sha256 }  
\#   nm\_digest { nm\_sha256 }  
\#   determinism { tie\_policy, rng\_seed? }  
\#   ties \[ ... \]   // if any random tie occurred

*End Doc 3A.*

# **Doc 3B — Build & Release Policy (Updated)**

## **1\) Purpose & scope**

Defines how we **version**, **tag**, **verify**, and **publish** the engine and renderer so runs are reproducible and auditable. Integrates all former addendum content—no separate addendum is needed.

This part is **normative** wherever it governs FID/Engine versioning and release gates.

---

## **2\) Versioning model (two tracks)**

* **Formula ID (FID)** — **64-hex** digest of the **Normative Manifest** (rules \+ outcome-affecting defaults).

  * Printed in `Result.formula_id` and `RunRecord.formula_id`.

  * Changes **only** when outcome logic or outcome-affecting defaults change.

* **Engine Version** — semantic version **vMAJOR.MINOR.PATCH** of the implementation.

  * Printed in `Result.engine_version` and `RunRecord.engine.version`.

  * Changes for code changes (including non-normative), build/tooling, or packaging.

The **Normative Manifest** content and canonicalization are defined in Doc 1A \+ Annex A. Presentation/report toggles (e.g., **VM-VAR-060..062**) are **excluded** from FID.

---

## **3\) What requires a new FID (and Engine Version)**

Any change that can change outcomes across any valid input set:

1. **Algorithmic rules**

   * Step order, allocation/gate semantics, tie-resolution logic, rounding/denominator rules.

2. **Outcome-affecting VM-VAR set**

   * Adding/removing a variable in the **Included** list (Annex A).

   * Changing a default, domain/range, or enumerated value semantics of an **Included** variable.

3. **Advanced/frontier semantics**

   * Behavior of **040–048**, **045–046**, **029–031**, **073** that alters gating/eligibility/frontier results.

4. **Determinism primitives**

   * Changing the deterministic tie key (must remain `Option.order_index`).

   * RNG profile/sequence for ties (when policy \= `random`).

5. **Canonicalization rules** (Doc 1A §2.1)

   * JSON formatting, key sorting, array ordering, or hash inputs.

**Release action:**

* Compute a **new FID**, **bump Engine Version** (see §5), update Annex A, and regenerate all golden fixtures (Doc 6).

* Update Doc 7 footer rules if display or disclosure changes.

---

## **4\) What does not change the FID (Engine Version only)**

* **Performance** improvements; memory usage; parallelization refactors (ordering preserved).

* **I/O/CLI UX** changes; logging; error messages; packaging; build toolchain updates.

* **Renderer** changes that affect **only presentation** (layout, language selection, section visibility).

* Changing **presentation/report variables** (**032–035**, **060–062**) or their defaults (they remain outside FID).

* Bug fixes that **do not** alter any computed outcome (verified by the Doc 6 test pack).

**Release action:**

* **Bump Engine Version** only. No FID change.

If a “bug fix” alters any outcome on any supported test, it is **normative** ⇒ **new FID** (and Engine Version).

---

## **5\) Engine Version bump rules (semver)**

* **MAJOR**: removal/incompatible behavior in CLI or artifacts; support matrix change; or any normative change shipping **with** new FID.

* **MINOR**: new non-breaking features; CLI flags added; report appendices added.

* **PATCH**: bug fixes; internal refactors; performance improvements.

**Tag example:** `engine/v1.4.2`  
 **Build metadata** recorded in `RunRecord.engine.build` (e.g., `commit:abcd1234`).

---

## **6\) Release gates (must pass before tagging)**

1. **Determinism checks**

   * Canonical JSON conformance (Doc 1A §2.1).

   * Cross-OS/arch byte-identical artifacts on the official matrix.

2. **Test pack** (Doc 6\)

   * All **A/B/C** suites pass; hashes and expected `Result` match exactly.

   * Random-tie tests repeatability with fixed `VM-VAR-052`.

3. **FID audit**

   * Independent recomputation of FID from the Normative Manifest equals `Result.formula_id`.

4. **Security/IO policy**

   * No network I/O; sandboxed run OK.

   * Inputs validated (Doc 1B); self-hash verification passes.

5. **Annex A alignment**

   * Included/Excluded lists, domains, and defaults match the code.

   * Any new VM-VAR IDs registered and documented.

Only after all gates pass may the release be tagged and published.

---

## **7\) Publication requirements**

**Artifacts to publish per release:**

* **Binaries/containers** (pinned toolchain).

* **Spec bundle**: Docs 1–7 \+ Annex A/B/C at the release tag.

* **Canonical Test Pack** (Annex B): machine-readable fixtures (inputs, expected outputs, hashes).

* **Change log**: human-readable summary (see §8 template).

* **Provenance**: checksums/signatures for binaries and spec bundle.

**Runtime disclosure (renderer/report footer):**

* Show **Formula ID (64-hex)**, **Engine Version**, and (if any) `algorithm_variant` (VM-VAR-073).

* If any **2B toggles** differ from Annex A defaults, append a **“Non-normative toggles”** note listing key/value pairs.

---

## **8\) Change log template (per release)**

Release: engine vX.Y.Z   |   Formula ID: \<64hex\> (if changed)  
Date (UTC): YYYY-MM-DD

Normative changes (FID):  
\- \<summary\>  \[Doc/Section; VM-VAR impact\]  
\- \<summary\>

Non-normative changes:  
\- \<summary\>  \[perf/UX/build\]

Spec & Annex updates:  
\- Annex A: \<IDs/domains/defaults updated\>  
\- Doc 6: \<tests added/updated\>

---

## **9\) Forks & reproducibility**

* Forks MUST set `RunRecord.engine.vendor` and SHOULD rename `engine.name`.

* Forks **must** preserve Doc 1A canonicalization, Doc 1A §5 ordering, RNG profile for ties, and Annex A Included/Excluded unless they intentionally create a **new FID**.

* Public results SHOULD include the spec bundle commit or URL for independent verification.

---

## **10\) Hotfix protocol**

* **PATCH** hotfixes are allowed **only** if Doc 6 passes with **no** output diffs; otherwise it is normative ⇒ new FID and at least **MINOR** bump.

* Re-run release gates (§6); republish hashes.

---

## **11\) Deprecation policy (IDs & flags)**

* **VM-VAR IDs are stable.** Do not renumber or repurpose.

* To retire a variable: mark **Deprecated** in Annex A with a sunset version; keep behavior until the next **MAJOR**.

* **VM-VAR-051** remains **reserved** (tie deterministic key is always `order_index`, not a variable).

---

## **12\) CLI compatibility**

* New flags require at least **MINOR** bump.

* Removing/renaming flags requires **MAJOR**.

* `--seed` MAY override `VM-VAR-052` at runtime; this does **not** alter FID (seed is non-FID) but must be echoed in `RunRecord.determinism.rng_seed`.

---

## **13\) Compliance checklist**

* **C-REL-FID**: Any outcome change ⇒ new FID; Engine Version bumped.

* **C-REL-SEMVER**: Changes categorized per §5.

* **C-REL-GATES**: All §6 gates pass on matrix.

* **C-REL-PUB**: Required artifacts & checksums published; footer disclosures correct.

* **C-REL-ANNEX**: Annex A is the single source of truth for Included/Excluded and domains; bundled with the release.

*End Doc 3B.*

