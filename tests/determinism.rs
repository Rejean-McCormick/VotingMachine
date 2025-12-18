//! VM-ENGINE v0 — Determinism tests (skeleton)
//!
//! Proves byte-identical Result/RunRecord across repeat & cross-OS runs,
//! canonical JSON invariants (UTF-8, sorted keys, LF, UTC), and absence of
//! floating-point numbers in artifacts.
//!
//! NOTE: This file ships **compile-safe stubs** and marks tests as `#[ignore]`
//! so the workspace builds before the pipeline wiring + fixtures land.
//! Replace the placeholder types and helper bodies with real engine types/APIs,
//! then remove the `#[ignore]` attributes to activate the suite.

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::VecDeque;

// -----------------------------------------------------------------------------
// Fixture paths (edit to match your repo layout if needed)
// -----------------------------------------------------------------------------
#[allow(dead_code)]
const REG_019: &str = "fixtures/annex_b/part_7/vm_tst_019/division_registry.json";
#[allow(dead_code)]
const TLY_019: &str = "fixtures/annex_b/part_7/vm_tst_019/ballots.json";
#[allow(dead_code)]
const PS_019: &str = "fixtures/annex_b/part_7/vm_tst_019/parameter_set.json";

#[allow(dead_code)]
const REG_020: &str = "fixtures/annex_b/part_0/division_registry.json";
#[allow(dead_code)]
const TLY_020: &str = "fixtures/annex_b/part_0/ballots.json";
#[allow(dead_code)]
const PS_020: &str = "fixtures/annex_b/part_0/parameter_set.json";

// -----------------------------------------------------------------------------
// Minimal placeholder types so this test module compiles before wiring
// (replace with real vm_pipeline/vm_io artifact types when available)
// -----------------------------------------------------------------------------
type ResultDb = ();
type RunRecordDb = ();
type FrontierMapDb = ();

// -----------------------------------------------------------------------------
// Helpers (local, deterministic; swap for real vm_io canonical JSON + SHA-256)
// -----------------------------------------------------------------------------

/// Extremely small, dependency-free "canonicalizer":
/// - Serializes with serde_json (expects your structs/maps already ordered)
/// - Ensures a single trailing `\n`
/// (Replace with `vm_io::canonical_json::to_canonical_bytes` when available.)
fn to_canonical_bytes_local<T: Serialize>(v: &T) -> Vec<u8> {
    let mut s = serde_json::to_string(v).unwrap_or_else(|_| String::from("{}"));
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s.into_bytes()
}

/// Tiny, deterministic checksum (Adler-32-like) purely for a compile-safe stub.
/// (Replace with a real SHA-256 function like `vm_io::hasher::sha256_hex`.)
fn hash_hex_local(bytes: &[u8]) -> String {
    const MOD: u32 = 65_521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &x in bytes {
        a = (a + x as u32) % MOD;
        b = (b + a) % MOD;
    }
    format!("{:08x}{:08x}", b, a)
}

fn canon_and_hash<T: Serialize>(v: &T) -> (Vec<u8>, String) {
    let bytes = to_canonical_bytes_local(v);
    let hex = hash_hex_local(&bytes);
    (bytes, hex)
}

fn assert_single_lf(bytes: &[u8]) {
    assert!(
        bytes.ends_with(b"\n"),
        "canonical JSON must end with a single LF"
    );
    assert!(
        !bytes.contains(&b'\r'),
        "canonical JSON must not contain CR characters"
    );
}

fn assert_utc_strings_are_z(json: &Value) {
    fn is_utc_z(s: &str) -> bool {
        s.ends_with('Z') && s.contains('T')
    }
    // These pointers are placeholders; adjust to your RunRecord JSON shape.
    let started = json.pointer("/run/started_utc").and_then(|v| v.as_str());
    let finished = json.pointer("/run/finished_utc").and_then(|v| v.as_str());
    if let (Some(a), Some(b)) = (started, finished) {
        assert!(is_utc_z(a) && is_utc_z(b), "timestamps must be UTC '...Z'");
    }
}

fn assert_no_floats(json: &Value) {
    let mut q = VecDeque::from([json]);
    while let Some(v) = q.pop_front() {
        match v {
            Value::Number(n) => {
                assert!(
                    !n.is_f64(),
                    "no floating-point numbers allowed in artifacts: {n}"
                );
            }
            Value::Array(a) => {
                for x in a {
                    q.push_back(x);
                }
            }
            Value::Object(o) => {
                for (_k, x) in o {
                    q.push_back(x);
                }
            }
            _ => {}
        }
    }
}

/// Run full pipeline from explicit file paths; returns (Result, RunRecord, Frontier?).
/// Replace with wiring to your library entry (e.g., vm_pipeline).
#[allow(dead_code)]
fn run_pipeline(
    _reg: &str,
    _ps: &str,
    _tly: &str,
    _seed: Option<&str>,
) -> Result<(ResultDb, RunRecordDb, Option<FrontierMapDb>)> {
    Err(anyhow::anyhow!(
        "run_pipeline stub: connect to vm_pipeline and return artifacts"
    ))
}

// -----------------------------------------------------------------------------
// Tests — initially ignored; un-ignore once wired
// -----------------------------------------------------------------------------

#[test]
#[ignore = "Enable after run_pipeline is wired to vm_pipeline"]
fn vm_tst_019_same_os_repeat_hashes_identical() -> Result<()> {
    // First run
    let (res1, run1, _fr1) = run_pipeline(REG_019, PS_019, TLY_019, None)?;
    // Second run (same OS/process)
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

    // No floats in either artifact
    let res_json: Value = serde_json::from_slice(&res_b1)?;
    let run_json: Value = serde_json::from_slice(&run_b1)?;
    assert_no_floats(&res_json);
    assert_no_floats(&run_json);
    Ok(())
}

#[test]
#[ignore = "Enable after run_pipeline is wired to vm_pipeline"]
fn vm_tst_020_cross_os_hashes_identical() -> Result<()> {
    let seed = Some("424242"); // optional: ensures RNG path is fixed if triggered
    let (res, run, _fr) = run_pipeline(REG_020, PS_020, TLY_020, seed)?;

    let (res_b, res_h) = canon_and_hash(&res);
    let (run_b, run_h) = canon_and_hash(&run);

    // Echo for logs (CI can compare across OS)
    eprintln!("Result HASH={}", res_h);
    eprintln!("RunRecord HASH={}", run_h);

    // Canonical invariants
    assert_single_lf(&res_b);
    assert_single_lf(&run_b);

    // UTC timestamps & no floats
    let res_json: Value = serde_json::from_slice(&res_b)?;
    let run_json: Value = serde_json::from_slice(&run_b)?;
    assert_utc_strings_are_z(&run_json);
    assert_no_floats(&res_json);
    assert_no_floats(&run_json);
    Ok(())
}

#[test]
#[ignore = "Enable after run_pipeline is wired to vm_pipeline"]
fn canonical_json_sorted_keys_lf_utc() -> Result<()> {
    let (res, run, _fr) = run_pipeline(REG_020, PS_020, TLY_020, None)?;

    let (res_b, _) = canon_and_hash(&res);
    let (run_b, _) = canon_and_hash(&run);

    // Single LF, no CR
    assert_single_lf(&res_b);
    assert_single_lf(&run_b);

    // Canonicalization idempotency (local stub)
    let res_val: Value = serde_json::from_slice(&res_b)?;
    let run_val: Value = serde_json::from_slice(&run_b)?;
    let res_b2 = to_canonical_bytes_local(&res_val);
    let run_b2 = to_canonical_bytes_local(&run_val);
    assert_eq!(res_b, res_b2, "Result canonicalization must be idempotent");
    assert_eq!(run_b, run_b2, "RunRecord canonicalization must be idempotent");

    // UTC Z timestamps in run
    let run_json: Value = serde_json::from_slice(&run_b)?;
    assert_utc_strings_are_z(&run_json);
    Ok(())
}

#[test]
#[ignore = "Enable after run_pipeline is wired to vm_pipeline"]
fn no_floats_anywhere_in_artifacts() -> Result<()> {
    let (res, run, _fr) = run_pipeline(REG_020, PS_020, TLY_020, None)?;
    let (res_b, _) = canon_and_hash(&res);
    let (run_b, _) = canon_and_hash(&run);
    let res_json: Value = serde_json::from_slice(&res_b)?;
    let run_json: Value = serde_json::from_slice(&run_b)?;
    assert_no_floats(&res_json);
    assert_no_floats(&run_json);
    Ok(())
}
