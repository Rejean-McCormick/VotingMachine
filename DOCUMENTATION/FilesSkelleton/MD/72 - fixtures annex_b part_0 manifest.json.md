````md
Pre-Coding Essentials (Component: fixtures/annex_b/part_0/manifest.json, Version/FormulaID: VM-ENGINE v0) — 72/89

1) Goal & Success
Goal: Ship a **run manifest** that the engine can parse directly (via `vm_io::manifest`) to resolve local files, verify digests, and assert engine/FormulaID expectations for a fully reproducible, offline run.
Success: Schema/shape matches what `vm_io/src/manifest.rs` consumes; paths are local (no URLs); **exactly one** ballots source is chosen; digests verify; expectations match; bytes are canonicalizable (UTF-8, LF, sorted keys) and yield stable SHA-256 across OS/arch.

2) Scope
In scope: Minimal, engine-native manifest fields (paths, expectations, digests).  
Out of scope: Algorithm knobs (live in ParameterSet), RNG control (tie policy/seed live in ParameterSet and/or CLI).

3) Inputs → Outputs
Input artifact: `fixtures/annex_b/part_0/manifest.json` (this file).  
Output to engine: `Manifest` + `ResolvedPaths` used by the loader to read Registry/Params and **either** Ballots **or** Ballot Tally (optional Adjacency).

4) Canonical Engine Shape (author exactly this)
```jsonc
{
  "id": "MAN:part0",
  "reg_path": "division_registry.json",
  "params_path": "parameter_set.json",

  // choose exactly one of the following two:
  "ballots_path": "ballots.json",
  // "ballot_tally_path": "ballot_tally.json",

  // optional:
  "adjacency_path": "adjacency.json",

  // assert we’re running the right code/rules:
  "expect": {
    "formula_id": "fid:xxxxxxxx…",        // lowercase hex (sha256 of Normative Manifest)
    "engine_version": "v0"
  },

  // strong, local reproducibility (hex must be 64-lowercase):
  "digests": {
    "division_registry.json": { "sha256": "<64-hex>" },
    "parameter_set.json":     { "sha256": "<64-hex>" },
    "ballots.json":           { "sha256": "<64-hex>" },   // or "ballot_tally.json"
    "adjacency.json":         { "sha256": "<64-hex>" }    // if present
  }
}
````

5. Field Rules (engine-aligned)

* `reg_path`, `params_path`: required, **local** paths only.
* Exactly one of `ballots_path` **xor** `ballot_tally_path` is present.
* `adjacency_path`: optional (needed only if frontier is enabled downstream).
* `expect`: Optional but recommended. `formula_id` (lowercase hex) and `engine_version` must match what the engine reports; otherwise the loader errors.
* `digests`: Optional but recommended. Keys are the same relative file names you use in the \*\_path fields; values carry `sha256` (64-hex). Engine verifies bytes on read.

6. RNG & Policy Alignment (important)

* **Do not** encode RNG here. Tie behavior is controlled by `Params`:

  * `VM-VAR-032 tie_policy ∈ {status_quo, deterministic, random}`
  * `VM-VAR-033 tie_seed : u64` (used only when tie\_policy = random)
* If you also accept a CLI `--seed`, ensure the pipeline normalizes it to `tie_seed (u64)`; the manifest remains agnostic.

7. How the engine consumes it

* Read & parse → schema/shape check.
* Reject **any** `http(s)://` path (local only).
* Resolve each path relative to the manifest’s directory; normalize `.` and `..`.
* Enforce “exactly one” ballots vs tally.
* If `expect` present → compare against engine identifiers; mismatch ⇒ error.
* If `digests` present → compute SHA-256 of each referenced file and compare (case-insensitive hex); mismatch ⇒ error.

8. Determinism & Numeric Rules

* Manifest is plain data; no numeric computations.
* Reproducibility comes from local files + verified digests + fixed expectations.
* Canonical JSON elsewhere: UTF-8, **LF** newlines, **sorted keys**; SHA-256 over canonical bytes (done by I/O layer, not here).

9. Edge Cases & Failure Policy

* Both or neither of ballots/tally present ⇒ **error**.
* Any path begins with `http://` or `https://` ⇒ **error**.
* Normalized path escapes the manifest base dir (and policy forbids) ⇒ **error**.
* `digests` includes a filename that is not one of the declared paths ⇒ **error** (strict fixture).
* Any `sha256` not 64-hex lowercase ⇒ **error**.
* `expect` provided but wrong engine/FormulaID ⇒ **error**.

10. Minimal Happy Example (tally mode)

```json
{
  "id": "MAN:part0",
  "reg_path": "division_registry.json",
  "params_path": "parameter_set.json",
  "ballot_tally_path": "ballot_tally.json",
  "expect": {
    "formula_id": "1a2b3c…(64-hex)…",
    "engine_version": "v0"
  },
  "digests": {
    "division_registry.json": { "sha256": "aaaaaaaa…(64)…" },
    "parameter_set.json":     { "sha256": "bbbbbbbb…(64)…" },
    "ballot_tally.json":      { "sha256": "cccccccc…(64)…" }
  }
}
```

11. Authoring Notes

* Keep file names stable and relative to the manifest (the engine resolves them against the manifest’s directory).
* Use **lowercase** hex for all digests and FID.
* If you maintain both ballots and tally fixtures, publish two manifests (one per source); don’t put both in one.

12. Test Checklist (must pass)

* **Exactly-one** ballots vs tally.
* All paths are local; normalization doesn’t escape base dir.
* `expect` matches the engine identifiers.
* All `digests` verify (content changes are detected).
* Loading this manifest leads to successful LOAD/VALIDATE in the pipeline, with deterministic downstream hashes on all OSes.

```
```
