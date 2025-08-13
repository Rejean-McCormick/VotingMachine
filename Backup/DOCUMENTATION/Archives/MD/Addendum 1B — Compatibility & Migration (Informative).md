# **Addendum 1B — Compatibility & Migration (Informative)**

**Status:** Informative guidance. Complements **Addendum 1A** and Docs **3A/3B**.  
 **Purpose:** Define what “compatible” means, when to bump versions, how to migrate datasets/tests, and how forks should publish differences without confusion.

---

## **1\) Versioning model (at a glance)**

* **Formula ID (FID):** Hash of the **normative rule set** (Addendum 1A §1–§4).

  * **Same FID ⇒ same outcomes** for any given inputs and seed.

  * **Different FIDs ⇒ outcomes not comparable**; treat as a different formula.

* **Engine Version:** Semantic version **MAJOR.MINOR.PATCH** for the implementation.

  * **Same FID \+ Engine changes** may improve performance, fix non-outcome bugs, or change packaging/UI. Outcomes **must not** change.

### **Compatibility classes**

| Case | FID | Engine | Expectation |
| ----- | ----- | ----- | ----- |
| A | same | same | Byte-identical Results/RunRecords (Doc 6C-019/020). |
| B | same | different | Results/RunRecords **byte-identical**; performance may differ. |
| C | different | any | Outcomes may differ; Annex B hashes must be regenerated; report must flag formula change. |

---

## **2\) When to bump what**

* **Bump FID (and Engine) if:**

  * Add/remove a **VM-VAR** in covered ranges; change a **default**, **domain**, or **semantics**.

  * Change any constant in Addendum 1A §1.2 (denominator, rounding, IRV exhaustion, MMP sequence, contiguity semantics).

  * Modify canonical serialization rules (Addendum 1A §3).

* **Bump Engine only if:**

  * Performance work, packaging, dependency upgrades, UI/report wording, translations.

  * Bug fixes that **do not** alter computed outcomes for any allowed ParameterSet.

* **Bump both** if a “bug fix” alters outcomes (even if closer to intent).

---

## **3\) Deprecation & reserved space**

* **Reserved IDs** (Annex A Part 3 §L) must remain unused until a future FID defines them.

* If you intend to **tighten or expand** a variable’s domain (e.g., allow a new `frontier_mode`), that requires a new FID.

* No “soft deprecations” of normative items: publish a new FID with clear release notes.

---

## **4\) Migration playbooks**

### **4.1 Upgrading Engine (same FID)**

* **Do:** Re-run **VM-TST-019/020**; verify identical hashes.

* **Don’t:** Change any normative code or defaults.

* **Report footer:** Show **same FID**, new **Engine Version**.

### **4.2 Moving to a new FID**

* **Docs:** Update Annex A, Docs 4A/4B/4C (normative), and Addendum 1A’s NM.

* **Annex B:** Regenerate **all** `expected_canonical_hash` values; bump the **Test Pack version** (e.g., `AnnexB v2`).

* **Report footer:** Print **new FID (short)** and a “Formula changed since previous release” note.

* **RunRecord:** Embed the **full FID** and the NM digest snapshot.

### **4.3 Data compatibility notes**

* **Inputs (registries, tallies):** Schema changes that don’t alter semantics are allowed without FID bump (Doc 1).

* **ParameterSets:** If a variable is **removed/renamed**, provide a migration script or reject old ParameterSets with a clear error (`ERR_INCOMPATIBLE_PARAMETERSET_FID`).

* **Results comparison:** Never compare Results across different FIDs beyond high-level description.

---

## **5\) Forks & interoperability**

* **Forks must publish**:

  * Their **own FID**, the modified NM, and a diff vs upstream (IDs changed, defaults changed).

  * A **Test Pack** (Annex B-equivalent) regenerated under the fork FID.

* **Identification:** Reports and RunRecords must print the fork’s **FID** and an **Engine Vendor/Name** field (Doc 3B).

* **No shadowing:** Do not reuse upstream FID values. Any normative difference ⇒ new FID.

---

## **6\) Effects on tests and reports**

* **Tests (Doc 6):**

  * Same FID: all test expected vectors and hashes remain valid.

  * New FID: expected vectors may change; update fixtures and rebaseline hashes.

* **Reports (Doc 7):**

  * Footer always prints: Formula ID (short), Engine Version, roll inclusion policy, approval denominator rule, and any deviations from Annex A defaults.

  * If FID changed, add a single-line notice: “**Formula updated** since ”.

---

## **7\) Error handling (recommended names)**

| Error code | Trigger | Recommended message |
| ----- | ----- | ----- |
| `ERR_INCOMPATIBLE_FORMULA_ID` | Attempt to load a Result/RunRecord with a different FID | “This artifact was produced with a different Formula ID. Outcomes are not comparable.” |
| `ERR_INCOMPATIBLE_PARAMETERSET_FID` | ParameterSet references variables/domains not present in current FID | “The parameter set targets a different Formula ID. Please migrate or select a matching engine.” |
| `ERR_CANONICALIZATION_MISMATCH` | Cross-OS bytes differ under same FID/Engine | “Serialization is non-canonical. Check Addendum 1A §3.” |

---

## **8\) Examples (concrete)**

* **Minor Engine update (same FID):** Switch RNG backend implementation but keep seeded sequence identical; rerun 019/020 → hashes match. Footer: same FID, Engine `+0.0.1`.

* **New frontier action (new FID):** Introduce `buffer_zone` band action; Annex A adds a variable/domain entry; Addendum 1A constants expand; regenerate Annex B; publish FID change.

* **Default threshold change (new FID):** Move national majority from 55 to 60\. Although users can override per run, the **default** itself is normative ⇒ new FID.

---

## **9\) Release checklist (short)**

1. Confirm **NM** (Addendum 1A) matches Docs 2/4.

2. Compute **FID** from NM (fields §2.1–§2.4 only).

3. Build reproducible binaries (Doc 3B); sign \+ publish checksums.

4. Run **Annex B** full pack; lock expected hashes.

5. Run **VM-TST-019/020**; verify determinism.

6. Publish release notes: changed items, FID, Engine Version, impacts.

7. Ensure report templates print required footer items.

---

## **10\) Support window (suggested)**

* Keep **N-1** Engine release available for download under the **same FID** for 6 months.

* Keep the last Engine that produced the **previous FID** available (clearly labeled “archived”) for audit and replication.

---

**End of Addendum 1B (Informative).**

