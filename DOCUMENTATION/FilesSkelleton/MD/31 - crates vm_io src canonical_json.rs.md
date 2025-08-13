
````
Pre-Coding Essentials (Component: crates/vm_io/src/canonical_json.rs, Version FormulaID VM-ENGINE v0) — 31/89

1) Goal & Success
Goal: Produce byte-identical JSON for hashing and persisted artifacts: UTF-8, lexicographically sorted object keys, LF newlines only, stable escaping, and deterministic number formatting.
Success: Same Rust structures → identical bytes across OS/arch; object keys always sorted; arrays untouched; output feeds SHA-256 hashing and on-disk files exactly.

2) Scope
In scope: canonical serializer (to bytes / to file), recursive key ordering for maps, stable escaping, compact and pretty modes (both LF), atomic file write.
Out of scope: schema validation, hashing implementation, timestamps, business logic.

3) Inputs → Outputs
Inputs: any `serde::Serialize` (Result, RunRecord, FrontierMap, registries, params, tallies).
Outputs: `Vec<u8>` canonical bytes; or file written atomically with those bytes.

4) Entities/Types (minimal)
- Uses `IoError` from `vm_io`.
- Internal `CanonicalValue` walker (Value → Value with sorted objects).

5) Module knobs
- Compact mode (default): no trailing newline, minimal whitespace.
- Pretty mode: 2-space indent, LF newlines, still sorted keys.

6) Functions (signatures only)
```rust
/// Return canonical JSON bytes (UTF-8, sorted keys, compact; no trailing newline).
pub fn to_canonical_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, IoError>;

/// Write canonical JSON file (creates parent dirs; atomic replace via temp+rename).
pub fn write_canonical_file<T: serde::Serialize, P: AsRef<std::path::Path>>(value: &T, path: P) -> Result<(), IoError>;

/// Pretty variant (2-space indent) that still sorts keys and enforces LF.
pub fn to_canonical_pretty_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, IoError>;
````

7. Algorithm Outline (implementation plan)

* **Key ordering (core)**

  * Serialize input to `serde_json::Value`.
  * Transform recursively: every JSON object becomes a `BTreeMap<String, Value>` (keys sorted by UTF-8 byte order); arrays left as-is; scalars unchanged.
  * Reject non-string object keys (shouldn’t occur with `serde_json`), return `IoError::Canon("non-string key")`.

* **Compact writer**

  * Use `serde_json::Serializer` with a **custom Formatter**:

    * Emits no extra spaces; **no trailing newline**.
    * Newlines within strings are escaped per JSON; outside strings we do not emit newlines.
    * Escaping remains default (UTF-8 preserved; control chars escaped).
  * **Numbers**: accept integers and finite JSON numbers (no NaN/Inf). `serde_json`’s ryu formatting is deterministic across platforms.

* **Pretty writer**

  * `PrettyFormatter` with **two-space indentation**.
  * Override newline to `\n` (LF) explicitly.
  * Keys already sorted by the CanonicalValue step.

* **LF enforcement & UTF-8**

  * Writers only emit `\n` if pretty mode is used. Compact mode produces no newlines (except inside strings).
  * Ensure output is **UTF-8** without BOM.

* **Atomic file write**

  * Write to `<path>.tmp` (create parent dirs), flush/sync, then `rename` to final path.

* **Deterministic numbers**

  * Permit JSON numbers for shares/ratios; forbid non-finite (serde\_json already errors).
  * Do **not** add trailing zeros; rely on shortest-round-trip formatting.

8. State Flow
   Pipeline/prep code builds structs → `to_canonical_bytes` → SHA-256 over bytes → `write_canonical_file` persists identical content on every platform.

9. Determinism & Numeric Rules

* Object keys sorted lexicographically by UTF-8 bytes.
* Arrays preserve caller order (use upstream determinism helpers for sorting inputs).
* No locale/time/platform effects; identical inputs ⇒ identical bytes.
* Numbers are deterministic via ryu; integers remain integers.

10. Edge Cases & Failure Policy

* Non-string object key → `IoError::Canon("non-string key")`.
* Attempted NaN/Infinity → bubble serde\_json error as `IoError::Canon("non-finite number")`.
* Very deep/large structures: bound depth/size earlier in vm\_io; surface `IoError::Limit(...)` when enabled.
* Path write failure → `IoError::Write`; fs race on rename surfaces as `Write`.

11. Test Checklist (must pass)

* **Idempotence**: same structure → identical bytes; parse → reserialize → identical.
* **Ordering**: maps with different insertion orders serialize to identical bytes.
* **Cross-platform**: Windows/macOS/Linux produce identical bytes.
* **Pretty vs compact**: differ only in insignificant whitespace; compact output hashes are stable.
* **Numbers**: integers & decimal shares serialize deterministically; NaN/Inf rejected.
* **Files**: atomic write leaves no partial file on crash; directories auto-created.

```


```
