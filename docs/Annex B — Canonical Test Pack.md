# **Annex B — Canonical Test Pack (Updated)**

**Part 1 of 3 — Scope, Repository Layout, Schemas & File Contracts**

## **1\) Status & scope**

* **Normative.** Annex B is the single source of truth for machine verification of Docs **6A–6C**.

* **Oracle:** `expected/hashes.json` is the **only** normative comparator. Any `expected/*.json` files are **informative** convenience copies.

* **Alignment:** Uses the updated ID scheme (ties **050/052**, **051 reserved**; presentation **060–062** excluded from FID).

## **2\) Repository layout (must)**

/annex-b/  
  manifest.json                     \# Case index (schema §3.1)  
  rng\_profile.json                  \# Pinned RNG spec for random ties (Part 2 §2)  
  /schemas/                         \# JSON Schemas (normative)  
    manifest.schema.json  
    hashes.schema.json  
  /cases/  
    \<CASE-ID\>/  
      registry.json                 \# Canonical inputs (Doc 1B)  
      tally.json  
      params.json  
      /expected/                    \# Informative (fat mode) \+ Normative hashes  
        result.json                 \# (informative) canonical Result  
        run\_record.json             \# (informative) canonical RunRecord  
        frontier\_map.json           \# (informative) only when frontier\_expected=true  
        hashes.json                 \# (normative) see §3.2

**Thin mode:** Only `expected/hashes.json` is required. If fat and thin artifacts are both present, **`hashes.json` governs**.

## **3\) File contracts (normative)**

### **3.1 `manifest.json`**

Purpose: enumerate all test cases and their essential properties so runners can select/route without opening each case folder.

**Required structure**

* `schema_version: "1.x"`

* `engine_matrix: string[]` — target OS/arch identifiers (e.g., `"linux-x86_64"`).

* `cases: Case[]`

**Case object (required fields)**

* `id: string` — stable ID (e.g., `"VM-TST-211"`).

* `suite: "6A" | "6B" | "6C"` — maps to Doc 6 parts.

* `title: string`

* `purpose: string`

* `files: ["registry.json","tally.json","params.json"]` (fixed)

* `expected_mode: "fat" | "thin"`

* `frontier_expected: boolean` — `true` only if a `frontier_map.json` is part of expected artifacts.

* `tie_expected: boolean` — `true` when `RunRecord.ties[]` must be non-empty.

* `features: string[]` — tags to speed routing (e.g., `["gates","frontier","ties","protected","overrides"]`).

Schema constraints: `id` unique; `suite` ∈ {6A,6B,6C}; `files` exactly the three filenames above; `features` values are free-form but lower\_snake\_case.

### **3.2 `expected/hashes.json` (the oracle)**

Purpose: bind **inputs** to **expected outputs** via hashes, and pin the expected **FID** and tie expectations.

**Required structure**

* `schema_version: "1.x"`

* `expected_fid: "<64hex>"` — the FID both artifacts must report.

* `inputs_sha256: { registry, tally, params }` — 64-hex of the **canonical** input JSON bytes.

* `result_sha256: "<64hex>"`

* `run_record_sha256: "<64hex>"`

* `frontier_map_sha256: "<64hex>" | null` — must be non-null iff `frontier_expected=true` in `manifest.json`.

* `tie_expectations` (object, required; fields conditional):

  * `policy: "status_quo" | "deterministic_order" | "random"`

  * `rng_seed_expected: integer ≥ 0` — **required iff** `policy="random"`.

  * `events_expected: integer ≥ 0` — optional sanity check count for `RunRecord.ties[]`.

**Binding rule:** Verifiers **must** check that the producer’s input digests match `inputs_sha256` **before** comparing output hashes.

### **3.3 Canonical input files (per case)**

* `registry.json`, `tally.json`, `params.json` **must** already be in canonical JSON form (Doc 1A §2.1: UTF-8, LF, sorted keys; arrays ordered).

* They **must** validate against Doc 1B schemas and Annex A domains.

### **3.4 Informative expected artifacts (fat mode only)**

* `expected/result.json`, `expected/run_record.json`, `expected/frontier_map.json`

  * If present, they are **informative**. Tools may display diffs for human debugging, but **hash comparisons are authoritative**.

## **4\) JSON Schemas (normative envelopes)**

### **4.1 `/schemas/manifest.schema.json` (outline)**

* Enforces `schema_version`, `engine_matrix` (array of non-empty strings), and `cases` as an array of Case objects with required fields in §3.1.

* Constraints:

  * `cases[].id` unique across the array.

  * `cases[].files` **exactly** `["registry.json","tally.json","params.json"]`.

  * `cases[].expected_mode` ∈ `{"fat","thin"}`.

  * `cases[].frontier_expected` and `cases[].tie_expected` are booleans.

### **4.2 `/schemas/hashes.schema.json` (outline)**

* Enforces `schema_version`, `expected_fid` as 64-hex, each `*_sha256` as 64-hex or null per §3.2, and `tie_expectations`.

* Conditional logic:

  * If `tie_expectations.policy = "random"`, then `rng_seed_expected` is **required**.

  * If `frontier_expected` is `true` (from manifest for this case), then `frontier_map_sha256` is **non-null**.

*(Full JSON Schemas can be generated from these outlines; they’re normative once committed.)*

## **5\) Norms & invariants (applies to all cases)**

* **Canonicalization:** All inputs and expected outputs are measured as canonical JSON bytes (Doc 1A §2.1).

* **FID linkage:** `expected_fid` is authoritative; the producer’s `Result.formula_id` and `RunRecord.formula_id` **must equal** it.

* **Thin vs fat:** When both are present, **hashes.json wins**; content of informative files must not override hashes.

* **No network:** Test runs must not perform network I/O (Docs 3A/5A).

* **Variable inclusion:** Verifiers recompute FID using only variables marked **Included** in Annex A; Excluded vars (e.g., 032–035, 052, 060–062) are ignored for FID.

---

**Next (Part 2):** Verification algorithm (step-by-step, incl. `run_id` suffix rule), RNG profile (pinned spec), and matrix conformance.

# **Annex B — Canonical Test Pack (Updated)**

**Part 2 of 3 — Verification Algorithm, RNG Profile, Matrix Conformance**

## **1\) Verification algorithm (normative)**

Given a case folder `/cases/<ID>/` and its `expected/hashes.json` (Part 1 §3.2):

### **1.1 Inputs & canonicalization**

1. Read `registry.json`, `tally.json`, `params.json`.

2. Canonicalize each (Doc 1A §2.1: UTF-8, LF, **sorted keys**, arrays in spec order).

3. Compute sha256 of the canonical bytes and compare to `inputs_sha256.{registry,tally,params}`. **Fail** the case if any mismatch.

### **1.2 Run producer (engine)**

4. Invoke CLI per Doc 6 harness (no network; exit codes per Doc 3A/5A).

5. On non-zero exit, only cases explicitly designed to fail validation may do so (Doc 6A/6B describe those). Otherwise **fail**.

### **1.3 Artifact identity & structure**

6. Canonicalize produced `result.json` and `run_record.json` (and `frontier_map.json` if it exists).

7. Compute:

   * `RES:` \+ sha256(canonical(result.json)) and compare to `Result.result_id`.

   * `RUN:` \+ `<timestamp>` \+ `-` \+ sha256(canonical(run\_record.json)) and compare to `RunRecord.run_id`.

     * **Rule:** Only the **hash suffix after the first hyphen** must match; the timestamp prefix may vary but **MUST** be RFC3339 UTC (`YYYY-MM-DDThh:mm:ssZ` or with `Z` offset).

   * If `frontier_map.json` exists: `FR:` \+ sha256(canonical(frontier\_map.json)) vs `frontier_id` (if embedded) and vs `frontier_map_sha256` (expected).

### **1.4 Hash oracle (authoritative)**

8. Compare computed output hashes to `expected/hashes.json`:

   * `result_sha256` (must match).

   * `run_record_sha256` (must match).

   * `frontier_map_sha256`:

     * If `manifest.frontier_expected = true` ⇒ **must be non-null and match**.

     * If `manifest.frontier_expected = false` ⇒ **must be null** and producer **must not** emit a `frontier_map.json`.

### **1.5 FID integrity**

9. Recompute the **Normative Manifest** (Annex A “Included” set only; Doc 1A) from the **producer’s** ParameterSet; hash to FID; compare to:

   * `expected_fid` (from `hashes.json`),

   * `Result.formula_id`,

   * `RunRecord.formula_id`.  
      All three must be equal.

### **1.6 Variable echo & policy checks**

10. Verify `RunRecord.vars_effective` **includes all outcome-affecting** VM-VARs actually used (Annex A Included set). Excluded vars may appear.

11. **Tie expectations** (from `hashes.json.tie_expectations`):

* `policy` must equal the effective `VM-VAR-050` in `vars_effective`.

* If `policy="random"`:

  * `RunRecord.determinism.rng_seed` **present** and equals `rng_seed_expected`.

  * Each `ties[]` entry has `"policy":"random"` and `"seed":<same>`.

  * If `events_expected` provided, `ties.length` must equal it.

* If `policy!="random"`: `RunRecord.determinism.rng_seed` **absent**; `ties[]` may be empty or contain non-random events consistent with the policy (e.g., `"status_quo"`).

### **1.7 Suite-specific assertions (Doc 6\)**

12. Apply additional assertions from the relevant suite:

* **6A**: no RNG, no frontier; ordering checks.

* **6B**: gate `reasons[]` ordering (by VM-VAR ID then symbolic), protected bypass rules, frontier presence per case.

* **6C**: RNG usage only when a real tie exists; “**exactly k draws for a k-way tie**” (see RNG profile).

**Pass criteria:** All steps above succeed. Any deviation ⇒ **fail** the case.

---

## **2\) Verifier pseudocode (normative)**

load expected \= read\_json("expected/hashes.json")  
canon\_inputs \= { r \= canon("registry.json"), t \= canon("tally.json"), p \= canon("params.json") }  
assert sha256(canon\_inputs.r) \== expected.inputs\_sha256.registry  
assert sha256(canon\_inputs.t) \== expected.inputs\_sha256.tally  
assert sha256(canon\_inputs.p) \== expected.inputs\_sha256.params

run\_engine("--registry registry.json \--tally tally.json \--params params.json \--out outdir")  
assert exit\_code in allowed\_for\_case

res \= canon("outdir/result.json")  
rr  \= canon("outdir/run\_record.json")  
fm? \= exists("outdir/frontier\_map.json") ? canon("outdir/frontier\_map.json") : null

assert ("RES:" \+ sha256(res)) \== read\_json(res).result\_id  
assert suffix\_after\_hyphen(read\_json(rr).run\_id) \== sha256(rr)

if expected.frontier\_map\_sha256 \!= null:  
  assert fm? \!= null  
  assert sha256(fm?) \== expected.frontier\_map\_sha256  
else:  
  assert fm? \== null

assert sha256(res) \== expected.result\_sha256  
assert sha256(rr)  \== expected.run\_record\_sha256

fid \= recompute\_fid(IncludedVarsFromAnnexA, params\_from(rr or inputs))  
assert fid \== expected.expected\_fid  
assert fid \== read\_json(res).formula\_id \== read\_json(rr).formula\_id

check\_vars\_effective(rr.vars\_effective, AnnexA.Included)  
check\_tie\_expectations(rr, expected.tie\_expectations)

apply\_suite\_assertions(case.suite, res, rr, fm?, manifest)

---

## **3\) RNG profile (pinned, normative)**

The RNG used for `VM-VAR-050="random"` ties is frozen by `/annex-b/rng_profile.json`. Engines and verifiers **must** implement it exactly.

### **3.1 Required fields**

{  
  "name": "xorshift128plus",        // example; choose and freeze per release  
  "state\_bits": 128,  
  "seed\_type": "u64",  
  "endianness": "little",  
  "next\_u64\_spec": "formula or reference defining next()",  // exact spec or paper ref  
  "draws\_per\_tie\_item": 1,  
  "tiebreak\_sort\_key": \["draw","option\_id"\],  
  "test\_vectors": \[  
    { "seed": 0,       "next\_u64\_first5": \["...","...","...","...","..."\] },  
    { "seed": 424242,  "next\_u64\_first5": \["...","...","...","...","..."\] }  
  \],  
  "notes": "Use exactly k draws for a k-way tie; do not draw when no tie exists."  
}

### **3.2 Norms (must)**

* **Seeding:** Initialize once per run from **VM-VAR-052** (integer ≥ 0). No per-unit reseeding.

* **Consumption:** A **k-way** tie consumes **exactly k** 64-bit draws; subsequent ties resume from the current RNG state.

* **Permutation:** Order tied options by `(draw_value, option_id)` ascending to obtain a stable permutation.

* **Platform identity:** Implementation must produce **identical sequences** across OS/arch.

* **Change control:** Any change to `rng_profile.json` (algorithm, seeding, or sort key) is **normative** ⇒ **new FID** and **regenerate all case hashes**.

---

## **4\) Matrix conformance (OS/arch)**

### **4.1 Engine matrix**

`manifest.json.engine_matrix` lists target platforms (e.g., `["linux-x86_64","macos-arm64","windows-x86_64"]`). A release **passes** only if **all cases** pass on **all** listed targets.

### **4.2 Identity requirement**

For the **same case** and **same inputs/ParameterSet** (including seed), produced artifacts **must** be byte-identical across all targets:

* Same `result_sha256` and `run_record_sha256`.

* If applicable, same `frontier_map_sha256`.

* Same `Result.formula_id` and `RunRecord.formula_id`.

* `run_id` **hash suffix** identical; timestamp prefix may differ but must be RFC3339 UTC.

### **4.3 CI gating (recommended, informative but expected in practice)**

* Validate both JSON Schemas (`manifest.schema.json`, `hashes.schema.json`).

* For each target in `engine_matrix`, run every case, then compare produced hashes to `expected/hashes.json`.

* Fail the release if any target fails; publish per-target logs.

---

## **5\) Error classification during verification**

* **Input bind failure** (inputs don’t match `inputs_sha256`) ⇒ case invalid (stop).

* **Producer exit code** outside the case’s allowance (per Doc 6 case design) ⇒ fail.

* **Canonicalization/ID mismatch** (`RES:`/`RUN:`/`FR:` rules) ⇒ fail.

* **Hash mismatch** vs `expected/hashes.json` ⇒ fail.

* **FID mismatch** (recomputed vs `expected_fid` or artifact fields) ⇒ fail.

* **Tie/frontier expectation mismatch** (policy/seed/events/presence) ⇒ fail.

---

**Next (Part 3):** Suite catalog & examples (concise list of cases 6A (101–110), 6B (201–216), 6C (301–312) with minimal per-case metadata and sample `hashes.json`).

# **Annex B — Canonical Test Pack (Updated)**

**Part 3 of 3 — Suite Catalog & Examples (machine-usable)**

Below is a ready-to-commit **`manifest.json`** case catalog (concise but complete), plus **sample `hashes.json`** for one case in each suite.  
 It follows the contracts from Parts 1–2 (hash oracle; run\_id suffix rule; RNG profile).

---

## **1\) `manifest.json` (full case list)**

{  
  "schema\_version": "1.x",  
  "engine\_matrix": \["linux-x86\_64", "macos-arm64", "windows-x86\_64"\],  
  "cases": \[  
    // \---------- 6A: Allocation correctness (101–110) \----------  
    {  
      "id": "VM-TST-101",  
      "suite": "6A",  
      "title": "Simple 2-option majority",  
      "purpose": "Baseline allocation; no ties; no gates; frontier off.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["allocation"\]  
    },  
    {  
      "id": "VM-TST-102",  
      "suite": "6A",  
      "title": "Three options — preserve registry order",  
      "purpose": "Allocation array keeps option.order\_index regardless of vote order.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["ordering","allocation"\]  
    },  
    {  
      "id": "VM-TST-103",  
      "suite": "6A",  
      "title": "Zero-vote minor option",  
      "purpose": "Zero votes do not imply invalidity or tie consumption.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["allocation"\]  
    },  
    {  
      "id": "VM-TST-104",  
      "suite": "6A",  
      "title": "Multiple units — deterministic iteration",  
      "purpose": "Units sorted by unit\_id; per-unit results correct.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["ordering","allocation"\]  
    },  
    {  
      "id": "VM-TST-105",  
      "suite": "6A",  
      "title": "Rounding policy application",  
      "purpose": "Family constants and rounding drive exact expected shares.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["rounding","allocation"\]  
    },  
    {  
      "id": "VM-TST-106",  
      "suite": "6A",  
      "title": "Large counts stability",  
      "purpose": "64-bit safety; canonicalization with big tallies.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["allocation","bigint"\]  
    },  
    {  
      "id": "VM-TST-107",  
      "suite": "6A",  
      "title": "Missing option in tally ⇒ validation error",  
      "purpose": "Referential integrity failure (no coerced zero rows).",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["validation","fk"\]  
    },  
    {  
      "id": "VM-TST-108",  
      "suite": "6A",  
      "title": "Duplicate order\_index ⇒ validation error",  
      "purpose": "Detect non-unique order\_index within a unit.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["validation","ordering"\]  
    },  
    {  
      "id": "VM-TST-109",  
      "suite": "6A",  
      "title": "Sum of votes \> valid\_ballots ⇒ validation error",  
      "purpose": "Sanity check on tallies.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["validation","sanity"\]  
    },  
    {  
      "id": "VM-TST-110A",  
      "suite": "6A",  
      "title": "FID lock — labels fixed policy",  
      "purpose": "Presentation change only; FID constant across 110A/110B.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["fid","labels"\]  
    },  
    {  
      "id": "VM-TST-110B",  
      "suite": "6A",  
      "title": "FID lock — labels dynamic policy",  
      "purpose": "Presentation change only; FID constant across 110A/110B.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["fid","labels"\]  
    },

    // \---------- 6B: Gates & Frontier (201–216) \----------  
    {  
      "id": "VM-TST-201",  
      "suite": "6B",  
      "title": "Eligibility threshold (020) invalidates unit",  
      "purpose": "Unit becomes Invalid; reasons include 020.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","eligibility"\]  
    },  
    {  
      "id": "VM-TST-202",  
      "suite": "6B",  
      "title": "Multiple eligibility failures — ordered reasons",  
      "purpose": "Record all reasons in ascending VM-VAR ID order.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","reasons\_order"\]  
    },  
    {  
      "id": "VM-TST-203",  
      "suite": "6B",  
      "title": "Symmetry exceptions (029) narrow override",  
      "purpose": "029 exempts a unit; unit remains valid; record applied\_exceptions.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","exceptions"\]  
    },  
    {  
      "id": "VM-TST-204",  
      "suite": "6B",  
      "title": "Precedence 030 over 029",  
      "purpose": "030 exclude wins over 029 exception; reasons reflect 030.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","precedence"\]  
    },  
    {  
      "id": "VM-TST-205",  
      "suite": "6B",  
      "title": "Integrity floor (031) cannot be bypassed",  
      "purpose": "Invalid even if 029/045 would allow eligibility.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","integrity"\]  
    },  
    {  
      "id": "VM-TST-206",  
      "suite": "6B",  
      "title": "Protected area (045=allow) bypasses eligibility",  
      "purpose": "Unit valid; protected\_bypass=true; no reason for bypassed gate.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","protected"\]  
    },  
    {  
      "id": "VM-TST-207",  
      "suite": "6B",  
      "title": "Protected area cannot bypass integrity (031)",  
      "purpose": "Invalid; reasons include 031; protected\_bypass absent/false.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","protected","integrity"\]  
    },  
    {  
      "id": "VM-TST-208",  
      "suite": "6B",  
      "title": "Frontier pre-check failure recorded",  
      "purpose": "Missing frontier inputs ⇒ validity failure token 'frontier\_missing\_inputs'.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","frontier","precheck"\]  
    },  
    {  
      "id": "VM-TST-210",  
      "suite": "6B",  
      "title": "Frontier disabled",  
      "purpose": "No frontier\_map; allocations unaffected; FID excludes 034.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["frontier"\]  
    },  
    {  
      "id": "VM-TST-211",  
      "suite": "6B",  
      "title": "Frontier banded — entry cut",  
      "purpose": "Deterministic gating; diagnostics emitted.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "fat",  
      "frontier\_expected": true,  
      "tie\_expected": false,  
      "features": \["frontier","diagnostics"\]  
    },  
    {  
      "id": "VM-TST-212A",  
      "suite": "6B",  
      "title": "FrontierMap toggle — on",  
      "purpose": "034=true: file present; allocations/FID unchanged vs 212B.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "fat",  
      "frontier\_expected": true,  
      "tie\_expected": false,  
      "features": \["frontier","diagnostics","toggle\_034"\]  
    },  
    {  
      "id": "VM-TST-212B",  
      "suite": "6B",  
      "title": "FrontierMap toggle — off",  
      "purpose": "034=false: file absent; allocations/FID unchanged vs 212A.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["frontier","toggle\_034"\]  
    },  
    {  
      "id": "VM-TST-213",  
      "suite": "6B",  
      "title": "Advanced window (047) near cut",  
      "purpose": "047 expands/contracts effective band; flips at margins per fixture.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "fat",  
      "frontier\_expected": true,  
      "tie\_expected": false,  
      "features": \["frontier","047"\]  
    },  
    {  
      "id": "VM-TST-214",  
      "suite": "6B",  
      "title": "Backoff policy (048)",  
      "purpose": "Borderline handling: none vs soften vs harden.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "fat",  
      "frontier\_expected": true,  
      "tie\_expected": false,  
      "features": \["frontier","048"\]  
    },  
    {  
      "id": "VM-TST-215",  
      "suite": "6B",  
      "title": "Strictness (049) multiplies effects",  
      "purpose": "Compare strict vs lenient with fixed 047/048.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "fat",  
      "frontier\_expected": true,  
      "tie\_expected": false,  
      "features": \["frontier","049"\]  
    },  
    {  
      "id": "VM-TST-216",  
      "suite": "6B",  
      "title": "Ladder mode with autonomy map (046)",  
      "purpose": "Deterministic package selection across steps.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "fat",  
      "frontier\_expected": true,  
      "tie\_expected": false,  
      "features": \["frontier","ladder","046"\]  
    },

    // \---------- 6C: Determinism & Ties (301–312) \----------  
    {  
      "id": "VM-TST-301",  
      "suite": "6C",  
      "title": "Reproducibility (no RNG)",  
      "purpose": "Identical runs produce byte-identical artifacts.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["determinism"\]  
    },  
    {  
      "id": "VM-TST-302",  
      "suite": "6C",  
      "title": "Deterministic tie by order\_index",  
      "purpose": "Resolve 2/3-way ties via registry order; no RNG; ties\[\] may be empty.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["ties","deterministic\_order"\]  
    },  
    {  
      "id": "VM-TST-303",  
      "suite": "6C",  
      "title": "Status quo policy path",  
      "purpose": "Policy applied; ties\[\] entries present (no seed).",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","status\_quo"\]  
    },  
    {  
      "id": "VM-TST-304",  
      "suite": "6C",  
      "title": "Random tie, 2-way, fixed seed",  
      "purpose": "Permutation stable across runs; rng\_seed echoed.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","random","seed"\]  
    },  
    {  
      "id": "VM-TST-305",  
      "suite": "6C",  
      "title": "Random tie, 3-way, fixed seed",  
      "purpose": "k draws, sort by (draw, option\_id).",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","random","seed"\]  
    },  
    {  
      "id": "VM-TST-306A",  
      "suite": "6C",  
      "title": "Seed variation A",  
      "purpose": "Different seed can change outcomes where ties exist; FID unchanged.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","random","seed","fid"\]  
    },  
    {  
      "id": "VM-TST-306B",  
      "suite": "6C",  
      "title": "Seed variation B",  
      "purpose": "Companion to 306A (different seed).",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","random","seed","fid"\]  
    },  
    {  
      "id": "VM-TST-307",  
      "suite": "6C",  
      "title": "Random policy but no ties",  
      "purpose": "No RNG use; rng\_seed absent; identical to deterministic order.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["ties","random","no\_tie"\]  
    },  
    {  
      "id": "VM-TST-308",  
      "suite": "6C",  
      "title": "Multiple tie events (k draws per event)",  
      "purpose": "3-way then 2-way consume draws in order; permutations match fixture.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","random","draw\_count"\]  
    },  
    {  
      "id": "VM-TST-309",  
      "suite": "6C",  
      "title": "Repeated ties within one unit",  
      "purpose": "Independent tie points consume sequential draws.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","random","draw\_order"\]  
    },  
    {  
      "id": "VM-TST-310",  
      "suite": "6C",  
      "title": "Policy change alters FID; seed does not",  
      "purpose": "050 in FID; 052 excluded.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","fid","seed"\]  
    },  
    {  
      "id": "VM-TST-311A",  
      "suite": "6C",  
      "title": "Random ties — frontier off",  
      "purpose": "Baseline with random ties and 040='none'.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": true,  
      "features": \["ties","random"\]  
    },  
    {  
      "id": "VM-TST-311B",  
      "suite": "6C",  
      "title": "Random ties — frontier on",  
      "purpose": "Same permutations as 311A; frontier diagnostics may emit.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "fat",  
      "frontier\_expected": true,  
      "tie\_expected": true,  
      "features": \["ties","random","frontier"\]  
    },  
    {  
      "id": "VM-TST-312",  
      "suite": "6C",  
      "title": "Invalid unit ⇒ no tie resolution",  
      "purpose": "Gates pre-empt ties; no RNG for invalid unit.",  
      "files": \["registry.json","tally.json","params.json"\],  
      "expected\_mode": "thin",  
      "frontier\_expected": false,  
      "tie\_expected": false,  
      "features": \["gates","ties","integrity"\]  
    }  
  \]  
}

---

## **2\) Example `hashes.json` (one per suite)**

Replace `<64hex>` with real hashes from your canonical outputs. These are **normative**.

### **2.1 6A example — `cases/VM-TST-101/expected/hashes.json`**

{  
  "schema\_version": "1.x",  
  "expected\_fid": "\<64hex\>",  
  "inputs\_sha256": {  
    "registry": "\<64hex\>",  
    "tally": "\<64hex\>",  
    "params": "\<64hex\>"  
  },  
  "result\_sha256": "\<64hex\>",  
  "run\_record\_sha256": "\<64hex\>",  
  "frontier\_map\_sha256": null,  
  "tie\_expectations": {  
    "policy": "deterministic\_order"  
  }  
}

### **2.2 6B example — `cases/VM-TST-211/expected/hashes.json`**

{  
  "schema\_version": "1.x",  
  "expected\_fid": "\<64hex\>",  
  "inputs\_sha256": {  
    "registry": "\<64hex\>",  
    "tally": "\<64hex\>",  
    "params": "\<64hex\>"  
  },  
  "result\_sha256": "\<64hex\>",  
  "run\_record\_sha256": "\<64hex\>",  
  "frontier\_map\_sha256": "\<64hex\>",  
  "tie\_expectations": {  
    "policy": "deterministic\_order"  
  }  
}

### **2.3 6C example — `cases/VM-TST-304/expected/hashes.json`**

{  
  "schema\_version": "1.x",  
  "expected\_fid": "\<64hex\>",  
  "inputs\_sha256": {  
    "registry": "\<64hex\>",  
    "tally": "\<64hex\>",  
    "params": "\<64hex\>"  
  },  
  "result\_sha256": "\<64hex\>",  
  "run\_record\_sha256": "\<64hex\>",  
  "frontier\_map\_sha256": null,  
  "tie\_expectations": {  
    "policy": "random",  
    "rng\_seed\_expected": 424242,  
    "events\_expected": 1  
  }  
}

---

## **3\) Notes for implementers (brief)**

* **IDs:** Do not renumber; subcases (`110A/B`, `212A/B`, `306A/B`, `311A/B`) exist **only** to supply unique expected hashes when a single narrative test requires two distinct runs.

* **Fat vs thin:** If you include `expected/*.json`, ensure they match the hashes; **hashes.json governs** in case of conflict.

* **RNG:** Ensure your `rng_profile.json` (Annex B Part 2 §3) is identical across OS/arch; changing it requires a **new FID** and regenerating **all** hashes.

That’s the full Annex B generated across three passes. If you want, I can save these JSON stubs into your repo structure now so your CI can start wiring against them.

