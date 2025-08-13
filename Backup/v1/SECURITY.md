````markdown
# SECURITY.md — VM-ENGINE v0
_Component 12/89 • Threat model, reporting, and hard guarantees_

## 1) Disclosure policy

**Where to report**
- Email: **<security@your-domain.example>** (replace before publishing)
- Optional PGP: publish a key in `SECURITY-KEYS.md` and on a public keyserver. Include fingerprint in PR.

**What to include**
- Affected version/commit SHA, OS/arch, minimal repro manifest/inputs, observed/expected behavior, crash logs (if any), and whether the issue is already public.

**Coordinated disclosure**
- Acknowledge within **3 business days**.  
- Initial triage/classification within **7 business days**.  
- Target fix window **≤ 90 days** from acknowledgment for High/Critical, sooner if exploitation is likely.  
- No public PoCs or details before a fix is available (unless mutually agreed). We will issue a security release and notes.

> If the report concerns a third-party dependency, we may forward relevant details to upstream under similar timelines.

---

## 2) Supported versions

- We provide security fixes for the **latest tagged release** and the **main branch**.  
- Older tags are **EOL** unless explicitly listed in `docs/release_policy.md`.

---

## 3) Threat model (high-level)

**In scope**
- **Malicious or malformed local inputs** (manifests, ballots, tallies, adjacency/frontier files).
- **Path traversal / symlink abuse** via user-provided paths.
- **Schema bypass / validation gaps**, including unknown fields when strict mode is enabled.
- **Report rendering safety** (HTML/JSON output escaping; no active content).
- **Tie-break RNG misuse** (seed handling, recording).
- **Determinism breakage** (non-canonical serialization, nondeterministic iteration).
- **Supply-chain drift** (toolchain/deps moving under us).

**Out of scope**
- **Network adversaries at runtime**: the engine is **offline** by design (no network I/O).
- **Multi-tenant sandboxing**: CLI is single-user; do not execute untrusted code inside the same process.
- **Untrusted plugin execution**: none supported.

---

## 4) Hard guarantees (must hold)

- **No network I/O** at runtime. Builds/tests run with `--locked`; default Cargo mode is **offline**.
- **Canonical JSON** everywhere: **UTF-8**, **LF** line endings, **sorted object keys**; arrays use spec-defined order.
- **Deterministic math**: integer/rational comparisons; no floating-point in outcome logic.
- **Ties**: policy-driven. When `tie_policy=random` is configured, use a provided **seed**; record it in `RunRecord` **only if** a random tie actually occurred. Changing the seed without a random tie must not change outputs.
- **Byte-identical artifacts** across OS/arch for the same inputs (+seed): `result.json` (RES), `run_record.json` (RUN), optional `frontier_map.json` (FR).

---

## 5) Operator guidance (secure-by-default)

- Run with **read-only inputs** and a **separate output directory**:
  ```bash
  ./target/release/vm_cli run --manifest <path>/manifest.json --output artifacts/run
````

* Use **locked** dependency resolution:

  ```bash
  cargo build --locked
  cargo test  --locked
  ```
* Keep Cargo **offline** by default; vendor dependencies if needed:

  ```bash
  # initial fetch only
  CARGO_NET_OFFLINE=0 cargo fetch
  # optional: cargo vendor  (ensure vendor/ is tracked)
  ```
* When `tie_policy=random`, **provide a seed** and retain `run_record.json` for auditability.

---

## 6) Input handling & validation

* Enforce **JSON Schema** validation first; fail closed on malformed documents.
* Cross-validate invariants per spec (e.g., entity trees, tally magnitudes, unique IDs).
* Reject **symlinks** and `..` path traversal in manifest-referenced files; resolve to **canonical paths** before opening.
* Apply **maximum file size** and **object depth** guards to prevent DoS (configure in loader; fail fast with clear errors).
* Optionally enable **strict mode** to reject unknown fields when required by the spec.

---

## 7) Report rendering safety

* Reports are **self-contained**: no remote fonts, scripts, or tiles.
* Escape **all** user-derived strings; sanitize HTML where rich text is allowed.
* If viewed inside the app, enforce a restrictive **Content-Security-Policy** and disable inline scripts.

---

## 8) Build & supply chain

* Pin the toolchain in `rust-toolchain.toml`; use **resolver = "2"** and **--locked** to prevent drift.
* Prefer checked-in **vendor/** for air-gapped builds; keep upstream **LICENSE/NOTICE** files.
* Release archives in `dist/` should be **signed** and accompanied by **SHA-256** checksums. Provide verification steps in release notes.
* Review third-party licenses; avoid dynamic code downloads at build/test time.

---

## 9) Security testing

* **Fuzz** parsers/loaders for manifests, ballots, tallies, and schemas (structured fuzzing).
* Run `cargo audit` / `cargo deny`; treat advisories seriously.
* Keep `clippy` clean with `-D warnings`.
* **Determinism test** (must pass):

  ```bash
  SEED=42
  ./target/release/vm_cli run --manifest fixtures/annex_b/VM-TST-001/manifest.json --rng-seed $SEED --output artifacts/a
  ./target/release/vm_cli run --manifest fixtures/annex_b/VM-TST-001/manifest.json --rng-seed $SEED --output artifacts/b
  cmp -s artifacts/a/result.json artifacts/b/result.json
  cmp -s artifacts/a/run_record.json artifacts/b/run_record.json
  ```

---

## 10) Contact & acknowledgments

* Primary: **[security@your-domain.example](mailto:security@your-domain.example)**
* Please include whether you want public credit after resolution. We maintain an optional **Hall of Thanks** in release notes.
* CVEs: if applicable, we can request an ID after triage.

---

## 11) Process (state flow)

1. Reporter sends details via email/PGP.
2. We acknowledge (≤ 3 business days) and triage (≤ 7 business days).
3. Fix developed on supported branches; drafts shared privately if needed.
4. Security release cut; checksums/signatures published; coordinated disclosure.

---

## 12) Determinism & numeric rules (restated)

* No floats for comparisons; round-half-to-even only at defined points; reporting rounds **once** to **one decimal**.
* Seeded RNG only for random ties; seed recorded in `RunRecord` **iff** used.
* Hashes/IDs computed over **canonical bytes**.

---

## 13) Edge cases → fail with explicit error

* `tie_policy=random` **without** a seed.
* Mixed **CRLF/LF** or unsorted JSON in canonical inputs.
* Inputs exceeding configured **size/depth** limits.

---

## 14) Self-checks (operator)

* **Air-gap** or firewall the host; confirm zero network connections during runs.
* Reproduce the **determinism test** above.
* Verify release **signatures/checksums** before deployment.

```
```
