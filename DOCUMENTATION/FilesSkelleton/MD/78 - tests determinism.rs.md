````md
Pre-Coding Essentials (Component: tests/determinism.rs, Version/FormulaID: VM-ENGINE v0) — 78/89

1) Goal & Success
Goal: Prove byte-identical **Result** and **RunRecord** across repeat runs (same OS) and across Win/macOS/Linux (CI), per VM-TST-019/020.  
Success: Identical SHA-256 for canonical JSON bytes; RNG seed echoed when used; artifacts end with a single `\n`; zero floats anywhere.

2) Scope
In: Canonical serialization (UTF-8, sorted keys, LF, UTC), stable iteration order, seeded RNG logging, offline I/O.  
Out: Math correctness (covered elsewhere), report formatting.

3) Inputs → Outputs
Inputs (fixtures):  
• VM-TST-019: large synthetic (same-OS repeat)  
• VM-TST-020: small baseline (cross-OS), optional `rng_seed=424242`  
Outputs (asserted): `sha256(Result)`, `sha256(RunRecord)` equal; canonical JSON invariants; no floats.

4) Fixture Paths (edit if your repo layout differs)
```rust
const REG_019: &str = "fixtures/annex_b/part_7/vm_tst_019/division_registry.json";
const TLY_019: &str = "fixtures/annex_b/part_7/vm_tst_019/ballots.json";
const PS_019:  &str = "fixtures/annex_b/part_7/vm_tst_019/parameter_set.json";

const REG_020: &str = "fixtures/annex_b/part_0/division_registry.json";
const TLY_020: &str = "fixtures/annex_b/part_0/ballots.json";
const PS_020:  &str = "fixtures/annex_b/part_0/parameter_set.json";
````

5. Variables (used here)
   No new VM-VARs. Tests may supply CLI `--seed 424242` (or set Params/manifest) to exercise RNG logging.

6. Test Functions (signatures only)

```rust
#[test] fn vm_tst_019_same_os_repeat_hashes_identical();
#[test] fn vm_tst_020_cross_os_hashes_identical();
#[test] fn canonical_json_sorted_keys_lf_utc();
#[test] fn no_floats_anywhere_in_artifacts();
```

7. Algorithm Outline (what each test asserts)

* **019 same-OS**: Run pipeline twice on 019 fixtures → canonicalize Result/RunRecord → SHA-256 equal; both artifacts end with single LF; no CRs.
* **020 cross-OS**: Run once with optional seed; compute hashes. If environment variables
  `CROSS_OS_EXPECTED_RES_SHA256` / `CROSS_OS_EXPECTED_RUN_SHA256` are set (by CI from a reference OS), assert equality; else just snapshot-print for CI aggregator.
* **Canonical JSON invariants**: bytes end with `\n`, contain no `\r`; timestamps are `...Z`; canonicalization round-trip is idempotent.
* **No floats**: Parse canonical JSON → walk values → every `Number` is `u64` or `i64`, never `f64`.

8. State Flow
   `LOAD → VALIDATE → TABULATE → ALLOCATE → AGGREGATE → APPLY_RULES → (MAP_FRONTIER?) → RESOLVE_TIES → LABEL → BUILD_RESULT → BUILD_RUN_RECORD`, then canonicalize/hash.

9. Determinism & Numeric Rules
   Stable orders (Units by ID; Options by `(order_index, id)`), integer/rational comparisons, round-half-even only where defined; RNG only from explicit seed; canonical JSON: UTF-8, sorted keys, single trailing LF, UTC timestamps.

10. Edge Cases & Failure Policy
    If hashes differ: dump canonical strings; assert no missing seed when RNG policy=random; fail with actionable message (key order, line endings, UTC).

11. Test Skeleton (drop-in; adapt crate paths)

```rust
use anyhow::Result;
use std::{collections::VecDeque, env};
use vm_io::canonical_json::to_canonical_bytes;
use vm_io::hasher::sha256_hex;

// --- Types from your crates (adjust paths) ---
type ResultDb     = vm_core::result::ResultDb;
type RunRecordDb  = vm_core::result::RunRecordDb;
type FrontierMapDb= vm_core::result::FrontierMapDb;

// --- Helpers ---
fn run_pipeline(reg:&str, ps:&str, tly:&str, seed: Option<&str>)
 -> Result<(ResultDb, RunRecordDb, Option<FrontierMapDb>)> {
    // Prefer library entry (vm_pipeline::run_from_manifest / run_with_ctx)
    // or a thin wrapper that loads files and runs the pipeline.
    // Seed can be passed via Params override or CLI-equivalent hook.
    unimplemented!("wire to pipeline for tests");
}

fn canon_and_hash<T: serde::Serialize>(v: &T) -> (Vec<u8>, String) {
    let bytes = to_canonical_bytes(v).expect("canonicalize");
    let hex   = sha256_hex(&bytes);
    (bytes, hex)
}

fn assert_single_lf(bytes: &[u8]) {
    assert!(bytes.ends_with(b"\n"), "must end with single LF");
    assert!(!bytes.contains(&b'\r'), "no CR characters allowed");
}

fn assert_utc_strings_are_z(json: &serde_json::Value) {
    fn is_utc_z(s:&str)->bool { s.ends_with('Z') && s.contains('T') }
    let started = json.pointer("/run/started_utc").and_then(|v| v.as_str());
    let finished= json.pointer("/run/finished_utc").and_then(|v| v.as_str());
    if let (Some(a), Some(b)) = (started, finished) {
        assert!(is_utc_z(a) && is_utc_z(b), "timestamps must be UTC '...Z'");
    }
}

fn assert_no_floats(json: &serde_json::Value) {
    let mut q = VecDeque::from([json]);
    while let Some(v) = q.pop_front() {
        match v {
            serde_json::Value::Number(n) => {
                assert!(!n.is_f64(), "no floating-point numbers allowed: {n}");
            }
            serde_json::Value::Array(a) => for x in a { q.push_back(x); }
            serde_json::Value::Object(o) => for (_k,x) in o { q.push_back(x); }
            _ => {}
        }
    }
}

// --- Tests ---

#[test]
fn vm_tst_019_same_os_repeat_hashes_identical() -> Result<()> {
    // First run
    let (res1, run1, _fr1) = run_pipeline(REG_019, PS_019, TLY_019, None)?;
    // Second run (same process/OS)
    let (res2, run2, _fr2) = run_pipeline(REG_019, PS_019, TLY_019, None)?;

    // Canonicalize & hash
    let (res_b1, res_h1) = canon_and_hash(&res1);
    let (run_b1, run_h1) = canon_and_hash(&run1);
    let (res_b2, res_h2) = canon_and_hash(&res2);
    let (run_b2, run_h2) = canon_and_hash(&run2);

    // Hash equality
    assert_eq!(res_h1, res_h2, "Result hashes must match on repeat");
    assert_eq!(run_h1, run_h2, "RunRecord hashes must match on repeat");

    // Canonical JSON invariants
    assert_single_lf(&res_b1);
    assert_single_lf(&run_b1);

    // No floats
    let res_json: serde_json::Value = serde_json::from_slice(&res_b1)?;
    let run_json: serde_json::Value = serde_json::from_slice(&run_b1)?;
    assert_no_floats(&res_json);
    assert_no_floats(&run_json);

    Ok(())
}

#[test]
fn vm_tst_020_cross_os_hashes_identical() -> Result<()> {
    let seed = Some("424242"); // optional: ensures RNG path is fixed if triggered
    let (res, run, _fr) = run_pipeline(REG_020, PS_020, TLY_020, seed)?;

    let (res_b, res_h) = canon_and_hash(&res);
    let (run_b, run_h) = canon_and_hash(&run);

    // CI can export reference hashes from a canonical OS:
    if let Ok(expect_res) = env::var("CROSS_OS_EXPECTED_RES_SHA256") {
        assert_eq!(res_h, expect_res, "Result cross-OS hash mismatch");
    }
    if let Ok(expect_run) = env::var("CROSS_OS_EXPECTED_RUN_SHA256") {
        assert_eq!(run_h, expect_run, "RunRecord cross-OS hash mismatch");
    }

    // Echo for logs
    eprintln!("Result SHA256={}", res_h);
    eprintln!("RunRecord SHA256={}", run_h);

    // Invariants
    assert_single_lf(&res_b);
    assert_single_lf(&run_b);

    // UTC timestamps & no floats
    let res_json: serde_json::Value = serde_json::from_slice(&res_b)?;
    let run_json: serde_json::Value = serde_json::from_slice(&run_b)?;
    assert_utc_strings_are_z(&run_json);
    assert_no_floats(&res_json);
    assert_no_floats(&run_json);

    Ok(())
}

#[test]
fn canonical_json_sorted_keys_lf_utc() -> Result<()> {
    // Minimal smoke via small baseline
    let (res, run, _fr) = run_pipeline(REG_020, PS_020, TLY_020, None)?;
    let (res_b, _) = canon_and_hash(&res);
    let (run_b, _) = canon_and_hash(&run);

    // Single LF, no CR
    assert_single_lf(&res_b);
    assert_single_lf(&run_b);

    // Canonicalization idempotency: parse → canonicalize again → bytes equal
    let res_val: serde_json::Value = serde_json::from_slice(&res_b)?;
    let run_val: serde_json::Value = serde_json::from_slice(&run_b)?;
    let res_b2 = to_canonical_bytes(&res_val)?;
    let run_b2 = to_canonical_bytes(&run_val)?;
    assert_eq!(res_b, res_b2, "Result canonicalization must be idempotent");
    assert_eq!(run_b, run_b2, "RunRecord canonicalization must be idempotent");

    // UTC Z timestamps in run
    let run_json: serde_json::Value = serde_json::from_slice(&run_b)?;
    assert_utc_strings_are_z(&run_json);

    Ok(())
}

#[test]
fn no_floats_anywhere_in_artifacts() -> Result<()> {
    let (res, run, _fr) = run_pipeline(REG_020, PS_020, TLY_020, None)?;
    let (res_b, _) = canon_and_hash(&res);
    let (run_b, _) = canon_and_hash(&run);

    let res_json: serde_json::Value = serde_json::from_slice(&res_b)?;
    let run_json: serde_json::Value = serde_json::from_slice(&run_b)?;
    assert_no_floats(&res_json);
    assert_no_floats(&run_json);
    Ok(())
}
```

```
```
