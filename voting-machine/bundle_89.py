#!/usr/bin/env python3
import sys
from pathlib import Path

# ---- 89 files (relative to repo root) ----
FILES = [
    "Cargo.toml",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    "Makefile",
    ".gitignore",
    ".gitattributes",
    ".editorconfig",
    ".pre-commit-config.yaml",
    "README.md",
    "LICENSE",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "schemas/division_registry.schema.json",
    "schemas/ballots.schema.json",
    "schemas/ballot_tally.schema.json",
    "schemas/parameter_set.schema.json",
    "schemas/manifest.schema.json",
    "schemas/result.schema.json",
    "schemas/run_record.schema.json",
    "schemas/frontier_map.schema.json",
    "crates/vm_core/Cargo.toml",
    "crates/vm_core/src/lib.rs",
    "crates/vm_core/src/ids.rs",
    "crates/vm_core/src/entities.rs",
    "crates/vm_core/src/variables.rs",
    "crates/vm_core/src/determinism.rs",
    "crates/vm_core/src/rounding.rs",
    "crates/vm_core/src/rng.rs",
    "crates/vm_io/Cargo.toml",
    "crates/vm_io/src/lib.rs",
    "crates/vm_io/src/canonical_json.rs",
    "crates/vm_io/src/manifest.rs",
    "crates/vm_io/src/hasher.rs",
    "crates/vm_io/src/loader.rs",
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
    "crates/vm_report/Cargo.toml",
    "crates/vm_report/src/lib.rs",
    "crates/vm_report/src/structure.rs",
    "crates/vm_report/src/render_json.rs",
    "crates/vm_report/src/render_html.rs",
    "crates/vm_cli/Cargo.toml",
    "crates/vm_cli/src/args.rs",
    "crates/vm_cli/src/main.rs",
    "fixtures/annex_b/part_0/parameter_set.json",
    "fixtures/annex_b/part_0/division_registry.json",
    "fixtures/annex_b/part_0/ballots.json",
    "fixtures/annex_b/part_0/manifest.json",
    "fixtures/annex_b/part_0/expected_result.json",
    "tests/vm_tst_core.rs",
    "tests/vm_tst_gates.rs",
    "tests/vm_tst_ranked.rs",
    "tests/vm_tst_mmp.rs",
    "tests/determinism.rs",
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
]

# ---- Selection policy (code only) ----
INCLUDE_FIXTURES = False  # set True if you want fixtures/*.json included as "code"
ALLOWED_EXTS = {".rs", ".toml", ".ts", ".tsx", ".js", ".json", ".html", ".css", ".yml", ".yaml"}
ALLOWED_BASENAMES = {"Makefile"}  # treated as code
EXCLUDE_BASENAMES = {"README.md", "LICENSE", "CONTRIBUTING.md", "SECURITY.md"}
EXCLUDE_PREFIXES_DEFAULT = [
    "crates/vm_app/src-tauri/icons/",        # images/icons
    "crates/vm_app/ui/public/map/tiles/",    # mbtiles/binaries
]
if not INCLUDE_FIXTURES:
    EXCLUDE_PREFIXES_DEFAULT.append("fixtures/")

def skip_reason(rel: str) -> str | None:
    p = Path(rel)
    if p.name in EXCLUDE_BASENAMES:
        return "docs"
    for pref in EXCLUDE_PREFIXES_DEFAULT:
        if rel.startswith(pref):
            return "assets/binaries" if "tiles" in pref or "icons" in pref else "fixtures"
    if (p.suffix.lower() not in ALLOWED_EXTS) and (p.name not in ALLOWED_BASENAMES):
        return "non-code"
    return None  # keep

# ---- Bundling (max 10 bundles) ----
def group_for(path: str) -> str:
    if path.startswith("schemas/"):
        return "schemas"
    if path.startswith("crates/vm_core/"):
        return "vm_core"
    if path.startswith("crates/vm_io/"):
        return "vm_io"
    if path.startswith("crates/vm_algo/"):
        return "vm_algo"
    if path.startswith("crates/vm_pipeline/"):
        return "vm_pipeline"
    if path.startswith("crates/vm_report/"):
        return "vm_report"
    if path.startswith("crates/vm_cli/"):
        return "vm_cli"
    if path.startswith("crates/vm_app/"):
        return "vm_app"
    if path.startswith("tests/"):
        return "data_tests"  # tests counted as code
    return "root"

GROUP_ORDER = [
    "root",
    "schemas",
    "vm_core",
    "vm_io",
    "vm_algo",
    "vm_pipeline",
    "vm_report",
    "vm_cli",
    "vm_app",
    "data_tests",
]

def is_binary_file(p: Path) -> bool:
    try:
        with open(p, "rb") as f:
            chunk = f.read(2048)
        if b"\x00" in chunk:
            return True
        try:
            chunk.decode("utf-8")
            return False
        except UnicodeDecodeError:
            return True
    except Exception:
        return True

def main() -> int:
    repo_root = Path(".").resolve()
    out_dir = repo_root / "bundles"
    out_dir.mkdir(parents=True, exist_ok=True)

    # Partition into groups (only files that pass selection)
    groups: dict[str, list[str]] = {name: [] for name in GROUP_ORDER}
    skipped = 0
    for rel in FILES:
        why = skip_reason(rel)
        if why is not None:
            print(f"[SKIP:{why}] {rel}")
            skipped += 1
            continue
        grp = group_for(rel)
        groups.setdefault(grp, []).append(rel)

    missing = 0
    oks = 0
    errors = 0
    binaries = 0

    print("\n== Validating and bundling (code only) ==\n")
    for grp in GROUP_ORDER:
        files = groups.get(grp, [])
        if not files:
            continue
        out_path = out_dir / f"{grp}.txt"
        with open(out_path, "w", encoding="utf-8", newline="\n") as out:
            out.write(f"# bundle: {grp}\n")
            for rel in files:
                fpath = repo_root / rel
                out.write("\n" + "-" * 80 + "\n")
                out.write(f"FILE: {rel}\n")
                out.write("-" * 80 + "\n")
                if not fpath.exists():
                    print(f"[MISSING] {rel}")
                    out.write(f"[MISSING] {rel}\n")
                    missing += 1
                    continue
                if is_binary_file(fpath):
                    # Shouldn't happen given selection, but guard anyway
                    size = fpath.stat().st_size
                    print(f"[BINARY]  {rel} ({size} bytes) — skipped")
                    out.write(f"[BINARY SKIPPED] {rel} ({size} bytes)\n")
                    binaries += 1
                    continue
                try:
                    text = fpath.read_text(encoding="utf-8")
                    print(f"[OK]      {rel}")
                    out.write(text)
                    if not text.endswith("\n"):
                        out.write("\n")
                    oks += 1
                except UnicodeDecodeError:
                    try:
                        raw = fpath.read_bytes()
                        text = raw.decode("latin-1", errors="replace")
                        print(f"[OK*]     {rel} (latin-1 fallback)")
                        out.write(text)
                        if not text.endswith("\n"):
                            out.write("\n")
                        oks += 1
                    except Exception as e:
                        print(f"[ERROR]   {rel} ({e})")
                        out.write("[UNREADABLE TEXT CONTENT]\n")
                        errors += 1

    # Manifest
    manifest = out_dir / "manifest.txt"
    with open(manifest, "w", encoding="utf-8", newline="\n") as m:
        total = 0
        for grp in GROUP_ORDER:
            files = groups.get(grp, [])
            if not files:
                continue
            total += len(files)
            m.write(f"{grp}: {len(files)} files\n")
            for rel in files:
                m.write(f"  - {rel}\n")
        m.write(f"\nTotal code files bundled: {total}\n")
        m.write(f"OK: {oks} | SKIP: {skipped} | BINARY: {binaries} | MISSING: {missing} | ERRORS: {errors}\n")

    print("\n== Summary ==")
    print(f"OK: {oks} | SKIP: {skipped} | BINARY: {binaries} | MISSING: {missing} | ERRORS: {errors}")
    print(f"Bundles written to: {out_dir}")

    # Fail if anything missing or unreadable
    return 1 if (missing > 0 or errors > 0) else 0

if __name__ == "__main__":
    sys.exit(main())
