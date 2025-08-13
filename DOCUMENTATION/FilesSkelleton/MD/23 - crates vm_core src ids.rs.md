
````
Pre-Coding Essentials (Component: crates/vm_core/src/ids.rs, Version/FormulaID: VM-ENGINE v0) — 23/89

1) Goal & Success
Goal: Provide typed, validated, comparable ID newtypes for canonical outputs and token IDs used across the engine, with zero ambiguity and stable ordering.
Success: Every ID round-trips (Display ⇄ FromStr), enforces allowed charset/shape, exposes minimal helpers, and derives Eq/Ord/Hash. Optional serde is gated by the `serde` feature. No I/O.

2) Scope
In scope:
- Output IDs: `ResultId (RES:…), RunId (RUN:…), FrontierMapId (FR:…)`, plus `FormulaId` (64-hex FID) and `Sha256` (64-hex digest).
- Token IDs: `UnitId`, `OptionId` (registry tokens; **no prefixes**).
- Validators/regex, constructors, `Display`/`FromStr`/`TryFrom`, stable ordering, lightweight helpers.
Out of scope: file paths, JSON/FS I/O, “input IDs” (no `REG:`/`TLY:`/`PS:`), hashing/canonical bytes (lives in vm_io).

3) Inputs → Outputs
Inputs: ASCII strings from loaders/tests.
Outputs (strong types used across core/algo/pipeline):
- `ResultId`, `RunId`, `FrontierMapId`, `FormulaId`, `Sha256`
- `UnitId`, `OptionId`

4) Types (inventory)
- `ResultId`        → `"RES:" + <sha256 64-hex lowercase>`
- `RunId`           → `"RUN:" + <UTC timestamp RFC3339 Z> + "-" + <sha256 64-hex>`  // canonical format example: `RUN:2025-08-12T14:00:00Z-<hex>`
- `FrontierMapId`   → `"FR:" + <sha256 64-hex>`
- `FormulaId`       → `<sha256 64-hex>`
- `Sha256`          → `<sha256 64-hex>`
- `UnitId`          → token pattern `^[A-Za-z0-9_.:-]{1,64}$`
- `OptionId`        → token pattern `^[A-Za-z0-9_.:-]{1,64}$`

All are case-sensitive ASCII. `UnitId`/`OptionId` have **no embedded registry prefix** and no path semantics here.

5) Public API (signatures only)
Parsing / construction
- `impl FromStr for ResultId/RunId/FrontierMapId/FormulaId/Sha256`
- `impl FromStr for UnitId/OptionId`
- `impl Display for all of the above`
- `impl TryFrom<&str> for …`

Helpers
- `pub fn is_valid_sha256(s: &str) -> bool`
- `pub fn is_valid_token(s: &str) -> bool`  // for UnitId/OptionId domain
- `impl RunId { pub fn timestamp_utc(&self) -> &str }`   // fast accessor to the embedded RFC3339
- `impl ResultId/FrontierMapId/FormulaId/Sha256 { pub fn as_hex(&self) -> &str }`

Serde (behind feature)
- `#[cfg(feature = "serde")]` `#[serde(transparent)]` newtypes serialize/deserialize as strings via `FromStr`.

Derives / traits
- `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Clone`, `Debug`
- Feature-gated `Serialize`, `Deserialize` where enabled.

6) Implementation plan (outline)
Newtypes
- Small, `pub struct ResultId(String);` etc. Consider `SmolStr` if you already use it; otherwise `String`.

Constants / patterns
- `const HEX64: &str = "^[0-9a-f]{64}$";`
- `const TOKEN: &str = "^[A-Za-z0-9_.:-]{1,64}$";`
- `const RES:   &str = "^RES:[0-9a-f]{64}$";`
- `const FR:    &str = "^FR:[0-9a-f]{64}$";`
- `const RUN:   &str = r"^RUN:\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z-[0-9a-f]{64}$";`  // canonical RFC3339 Z
- Compile once with `once_cell::sync::Lazy<regex::Regex>` (or a tiny in-house matcher if you want to avoid `regex` here; keeping vm_core lean is preferred—ok to implement manual checks).

Validation rules
- ASCII-only, no NULs; reject on length > 256 (IDs) / > 64 (tokens).
- `ResultId/FrontierMapId/FormulaId/Sha256` require **lowercase** hex (normalize to lowercase on parse if you must, but prefer rejecting non-lowercase to keep strict).
- `RunId` must contain RFC3339 `YYYY-MM-DDTHH:MM:SSZ`; the embedded sha must be 64-hex.

Ordering
- Lexicographic order on the stored canonical string is the total order for all ID types. This matches deterministic sorting needs.

7) Module layout (sketch)
```rust
//! ids.rs — canonical engine/output IDs and token IDs (no input IDs here)

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

pub mod error {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum IdError { NonAscii, TooLong, BadShape }
}

use error::IdError;

// Newtypes (ResultId, RunId, FrontierMapId, FormulaId, Sha256, UnitId, OptionId)
// impl FromStr/TryFrom<&str>/Display/Eq/Ord/Hash for each
// Helper fns: is_valid_sha256, is_valid_token
// Accessors: RunId::timestamp_utc(), *_Id::as_hex()
````

8. Determinism & Numeric Rules

* IDs are pure strings with fixed shapes; ordering is lexicographic on canonical strings.
* No numeric math here; any hashing/canonical byte work is done outside (vm\_io).

9. Edge Cases & Failure Policy

* Empty or over-length strings ⇒ `IdError::TooLong` / `BadShape`.
* Non-ASCII ⇒ `IdError::NonAscii`.
* Mixed-case hex for sha fields ⇒ `BadShape` (prefer strict lowercase).
* `RunId` with non-UTC or missing `Z` ⇒ `BadShape`.

10. Test Checklist
    Round-trip

* `format!("{}", "RES:<hex>".parse::<ResultId>()?) == "RES:<hex>"` (likewise for FR, RUN, FormulaId, Sha256, UnitId, OptionId).

Negative cases

* Wrong prefix (e.g., `REZ:`), hex not 64 chars, uppercase hex, non-ASCII, whitespace, empty strings.
* `RunId` timestamp missing seconds or `Z`, or using space instead of `T`.

Ordering

* Sorting `Vec<UnitId>` or `Vec<OptionId>` equals lexicographic sort of their strings.
* Sorting `Vec<ResultId>`/`Vec<FrontierMapId>`/`Vec<RunId>` is stable and deterministic.

Serde (feature)

* Serialize → string; Deserialize → identical ID; bad shapes fail with a clear error.

DoS guard

* Reject token strings > 64 chars and IDs > 256 chars quickly.

```


