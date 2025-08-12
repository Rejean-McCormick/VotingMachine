# -*- coding: utf-8 -*-
"""
Create the exact empty repo skeleton for the voting-machine project.
- Uses Unicode em dashes in /docs filenames.
- Adds Tauri icons folder with a placeholder icon.png.
Run from the directory where you want the "voting-machine/" folder created:
    python create_vm_tree_perfect.py
"""

from pathlib import Path

ROOT = Path("voting-machine")

FILES = [
    # root
    "Cargo.toml","Cargo.lock","rust-toolchain.toml","Makefile",
    ".gitignore",".gitattributes",".editorconfig",".pre-commit-config.yaml",
    ".github/workflows/ci.yml","LICENSE","README.md","CONTRIBUTING.md","SECURITY.md",
    ".cargo/config.toml",

    # docs (Unicode em dashes)
    "docs/Annex A — Variable Canonical Reference Table.md",
    "docs/Annex B — Part 0_ Schema & Conventions.md",
    "docs/Annex C — Glossary & Definitions.md",
    "docs/Doc 1 — Database Specification (Entities, Fields, Relationships) (1).md",
    "docs/Doc 2 — Common Variables Specification (Core, Operational Defaults, Advanced Controls).md",
    "docs/Doc 3 — Technical Platform & Release Policy.md",
    "docs/Doc 4 — Algorithm Specification (Steps, Allocation, Gates & Edge Cases).md",
    "docs/Doc 5 — Processing Pipeline Specification (State Machine & Functions).md",
    "docs/Doc 6 — Test Specifications (Allocation, Gates, Frontier & Determinism).md",
    "docs/Doc 7 — Reporting Specification (Structure, Templates & Visual Rules).md",

    # schemas
    "schemas/division_registry.schema.json",
    "schemas/ballots.schema.json",
    "schemas/ballot_tally.schema.json",
    "schemas/parameter_set.schema.json",
    "schemas/manifest.schema.json",
    "schemas/result.schema.json",
    "schemas/run_record.schema.json",
    "schemas/frontier_map.schema.json",

    # fixtures
    "fixtures/annex_b/part_0/parameter_set.json",
    "fixtures/annex_b/part_0/division_registry.json",
    "fixtures/annex_b/part_0/ballots.json",
    "fixtures/annex_b/part_0/manifest.json",
    "fixtures/annex_b/part_0/expected_result.json",

    # ci
    "ci/determinism.yml",
    "ci/perf_profile.json",

    # crates: vm_core
    "crates/vm_core/Cargo.toml",
    "crates/vm_core/src/lib.rs",
    "crates/vm_core/src/ids.rs",
    "crates/vm_core/src/entities.rs",
    "crates/vm_core/src/variables.rs",
    "crates/vm_core/src/determinism.rs",
    "crates/vm_core/src/rng.rs",
    "crates/vm_core/src/rounding.rs",

    # crates: vm_io
    "crates/vm_io/Cargo.toml",
    "crates/vm_io/src/lib.rs",
    "crates/vm_io/src/canonical_json.rs",
    "crates/vm_io/src/loader.rs",
    "crates/vm_io/src/manifest.rs",
    "crates/vm_io/src/hasher.rs",

    # crates: vm_algo
    "crates/vm_algo/Cargo.toml",
    "crates/vm_algo/src/lib.rs",
    "crates/vm_algo/src/tabulation/plurality.rs",
    "crates/vm_algo/src/tabulation/approval.rs",
    "crates/vm_algo/src/tabulation/score.rs",
    "crates/vm_algo/src/tabulation/ranked_irv.rs",
    "crates/vm_algo/src/tabulation/ranked_condorcet.rs",
    "crates/vm_algo/src/allocation/wta.rs",
    "crates/vm_algo/src/allocation/dhondt.rs",
    "crates/vm_algo/src/allocation/sainte_lague.rs",
    "crates/vm_algo/src/allocation/largest_remainder.rs",
    "crates/vm_algo/src/mmp.rs",
    "crates/vm_algo/src/gates_frontier.rs",

    # crates: vm_pipeline
    "crates/vm_pipeline/Cargo.toml",
    "crates/vm_pipeline/src/lib.rs",
    "crates/vm_pipeline/src/load.rs",
    "crates/vm_pipeline/src/validate.rs",
    "crates/vm_pipeline/src/tabulate.rs",
    "crates/vm_pipeline/src/allocate.rs",
    "crates/vm_pipeline/src/aggregate.rs",
    "crates/vm_pipeline/src/apply_rules.rs",
    "crates/vm_pipeline/src/map_frontier.rs",
    "crates/vm_pipeline/src/resolve_ties.rs",
    "crates/vm_pipeline/src/label.rs",
    "crates/vm_pipeline/src/build_result.rs",
    "crates/vm_pipeline/src/build_run_record.rs",

    # crates: vm_report
    "crates/vm_report/Cargo.toml",
    "crates/vm_report/src/lib.rs",
    "crates/vm_report/src/structure.rs",
    "crates/vm_report/src/render_json.rs",
    "crates/vm_report/src/render_html.rs",

    # crates: vm_cli
    "crates/vm_cli/Cargo.toml",
    "crates/vm_cli/src/main.rs",
    "crates/vm_cli/src/args.rs",

    # crates: vm_app (Tauri shell) + icons
    "crates/vm_app/Cargo.toml",
    "crates/vm_app/src-tauri/Cargo.toml",
    "crates/vm_app/src-tauri/src/main.rs",
    "crates/vm_app/src-tauri/tauri.conf.json",
    "crates/vm_app/src-tauri/icons/icon.png",
    "crates/vm_app/ui/package.json",
    "crates/vm_app/ui/index.html",
    "crates/vm_app/ui/vite.config.ts",
    "crates/vm_app/ui/src/main.ts",
    "crates/vm_app/ui/public/map/style.json",
    "crates/vm_app/ui/public/map/tiles/world.mbtiles",

    # tests
    "tests/vm_tst_core.rs",
    "tests/vm_tst_gates.rs",
    "tests/vm_tst_ranked.rs",
    "tests/vm_tst_mmp.rs",
    "tests/determinism.rs",
]

def main() -> None:
    created = 0
    for rel in FILES:
        p = ROOT / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        if not p.exists():
            if p.suffix.lower() in {".png", ".mbtiles"}:
                # still zero-byte placeholders
                p.write_bytes(b"")
            else:
                p.touch()
            created += 1
    print(f"Created {created} files under {ROOT.resolve()}")

if __name__ == "__main__":
    main()
