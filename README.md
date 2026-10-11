# Voting Machine — VM-ENGINE v0

**Deterministic, offline-oriented vote tabulation and allocation engine.** Component 09/89 of a larger system; this README is an implementation entry point, not the normative source of algorithmic policy.

The design targets canonical JSON output, deterministic tie policy, reproducible artifacts and explicit allocation/frontier gates. The normative authority is the accompanying **Docs 1–7 and Annexes A–C**, not a prose summary in this README.

## Requirements and build

The Rust toolchain is pinned in `rust-toolchain.toml`. Use locked dependencies, and check `.cargo/config.toml` for offline configuration.

```bash
rustup show
cargo build --locked -p vm_cli
cargo test --locked --workspace
```

## Run an Annex B fixture

```bash
./target/debug/vm_cli run --manifest fixtures/annex_b/part_0/manifest.json --output artifacts/run
```

PowerShell equivalent:

```powershell
.\target\debug\vm_cli run --manifest fixtures\annex_b\part_0\manifest.json --output artifacts\run
```

The tool is expected to produce canonical `result.json`, provenance `run_record.json` and an optional `frontier_map.json` according to the manifest and policy. Confirm the fixture path and the compiled CLI target before running.

## Determinism and data integrity

- Canonical JSON is specified as UTF-8, LF line endings and sorted object keys.
- Tie-breaking follows an explicit policy. If randomization is enabled, a seeded ChaCha20 RNG is used and its seed is recorded when a randomized tie actually occurs.
- Artifact identifiers include `RES:<hash>` and `RUN:<ts>-<hash>`; frontier output may use `FR:<hash>`.
- The report renderer must not recompute canonical decisions. Displayed percentages follow Doc 7, including one-decimal formatting.

## Compare two runs

```bash
./target/debug/vm_cli run --manifest fixtures/annex_b/part_0/manifest.json --rng-seed 1 --output artifacts/run1
./target/debug/vm_cli run --manifest fixtures/annex_b/part_0/manifest.json --rng-seed 1 --output artifacts/run2
cmp artifacts/run1/result.json artifacts/run2/result.json
```

For JSON-based comparison, an independent Python or `jq` comparison can be used, but **byte identity** is the stronger canonical-output check.

## Repository overview

| Path | Role |
| --- | --- |
| `crates/vm_core/` | Shared types and invariants |
| `crates/vm_io/` | Canonical I/O |
| `crates/vm_algo/` | Allocation and gates |
| `crates/vm_pipeline/` | State machine |
| `crates/vm_report/` | Derived reporting |
| `crates/vm_cli/` | CLI |
| `crates/vm_app/` | Optional desktop UI |
| `fixtures/annex_b/` | Normative fixture candidates |
| `schemas/` | Input and output schemas |

## Verification and authority

Run the repository's actual documented Make targets if present (for example `make ci`, `make fixtures` or `make verify`), but consult Makefile configuration before assuming those commands work on the current checkout. Specs 1–7 and Annexes A–C define canonical decisions; the implementation and GUI must conform to them.

No production security guarantee or end-to-end election certification is implied by a successful local smoke test.
