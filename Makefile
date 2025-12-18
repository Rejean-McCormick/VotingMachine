# 04 - Makefile — VM-ENGINE v0
# Deterministic, offline-by-default build/test/package pipeline.
# Use with a POSIX shell (Git Bash on Windows).

# -------- Settings (override via: make <tgt> MK.PROFILE=release MK.OFFLINE=1 MK.SEED=42) ----------
MK.PROFILE ?= release
MK.OFFLINE ?= 1
MK.SEED    ?= 42

BIN_NAME   ?= vm_cli
FIXDIR     ?= fixtures/annex_b
ARTIFACTS  ?= artifacts
DISTDIR    ?= dist
STAGE      ?= .stage
SOURCE_DATE_EPOCH ?= 0    # seconds since epoch; 0 = 1970-01-01 (reproducible mtimes)

# Cargo runs offline unless explicitly overridden.
CARGO = env CARGO_NET_OFFLINE=$(MK.OFFLINE) cargo

# OS/ARCH detection for bundle names
UNAME_S := $(shell uname -s 2>/dev/null || echo Windows)
ifeq ($(findstring MINGW,$(UNAME_S)),MINGW)
  OS := windows
  EXE := .exe
else ifeq ($(UNAME_S),Windows)
  OS := windows
  EXE := .exe
else ifeq ($(UNAME_S),Darwin)
  OS := macos
  EXE :=
else
  OS := linux
  EXE :=
endif

ARCH_RAW := $(shell uname -m 2>/dev/null || echo amd64)
ifeq ($(ARCH_RAW),x86_64)
  ARCH := x86_64
else ifeq ($(ARCH_RAW),aarch64)
  ARCH := aarch64
else
  ARCH := $(ARCH_RAW)
endif

BIN      := target/$(MK.PROFILE)/$(BIN_NAME)$(EXE)
DISTNAME := $(BIN_NAME)-$(OS)-$(ARCH)
DISTSTAGE := $(STAGE)/$(DISTNAME)

.DEFAULT_GOAL := help
.SHELLFLAGS := -eu -o pipefail -c

.PHONY: help fmt lint build test fixtures verify hash dist clean ci

# ----------------------------------- Meta -------------------------------------
help:
	@echo "Targets:"
	@echo "  fmt        - cargo fmt (check)"
	@echo "  lint       - cargo clippy (deny warnings)"
	@echo "  build      - cargo build --locked (profile=$(MK.PROFILE), offline=$(MK.OFFLINE))"
	@echo "  test       - cargo test --locked"
	@echo "  fixtures   - run small Annex B set (VM-TST-001/2/3 if present) and compare to expected"
	@echo "  verify     - determinism smoke test (repeat run, compare bytes)"
	@echo "  hash       - SHA-256 for results/dist artifacts"
	@echo "  dist       - reproducible archives (tar.gz, zip)"
	@echo "  ci         - fmt → lint → build → test → fixtures → verify → hash"
	@echo "  clean      - remove target/, artifacts/, dist/, .stage/"

# --------------------------------- Quality ------------------------------------
fmt:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets -- -D warnings

# --------------------------------- Build/Test ---------------------------------
build:
	$(CARGO) build --locked --profile $(MK.PROFILE)

test:
	$(CARGO) test --locked --profile $(MK.PROFILE)

# ------------------------------ Fixtures (Annex B) ----------------------------
# Runs a minimal canonical pack if present:
#   $(FIXDIR)/VM-TST-00{1,2,3}/manifest.json
fixtures: build
	@set -e; \
	for T in VM-TST-001 VM-TST-002 VM-TST-003; do \
	  if [ -d "$(FIXDIR)/$$T" ]; then \
	    OUT="$(ARTIFACTS)/fixtures/$$T"; \
	    mkdir -p "$$OUT"; \
	    echo "==> Running $$T"; \
	    "$(BIN)" run --manifest "$(FIXDIR)/$$T/manifest.json" --output "$$OUT" --rng-seed "$(MK.SEED)"; \
	    if [ -f "$(FIXDIR)/$$T/expected/result.json" ]; then \
	      if command -v jq >/dev/null 2>&1; then \
	        diff -u <(jq -S . "$(FIXDIR)/$$T/expected/result.json") <(jq -S . "$$OUT/result.json"); \
	      else \
	        cmp -s "$(FIXDIR)/$$T/expected/result.json" "$$OUT/result.json"; \
	      fi; \
	    fi; \
	    if [ -f "$(FIXDIR)/$$T/expected/run_record.json" ]; then \
	      if command -v jq >/dev/null 2>&1; then \
	        diff -u <(jq -S . "$(FIXDIR)/$$T/expected/run_record.json") <(jq -S . "$$OUT/run_record.json"); \
	      else \
	        cmp -s "$(FIXDIR)/$$T/expected/run_record.json" "$$OUT/run_record.json"; \
	      fi; \
	    fi; \
	  fi; \
	done

# ---------------------------- Determinism (Smoke) -----------------------------
# Repeat the same run twice with the same seed and compare bytes.
verify: build
	@set -e; \
	TESTDIR="$(FIXDIR)/VM-TST-001"; \
	if [ ! -d "$$TESTDIR" ]; then \
	  echo "verify: $$TESTDIR not found; skipping (add Annex B minimal pack)."; \
	  exit 0; \
	fi; \
	OUT="$(ARTIFACTS)/verify"; rm -rf "$$OUT"; mkdir -p "$$OUT/a" "$$OUT/b"; \
	echo "==> Determinism check on VM-TST-001"; \
	"$(BIN)" run --manifest "$$TESTDIR/manifest.json" --output "$$OUT/a" --rng-seed "$(MK.SEED)"; \
	"$(BIN)" run --manifest "$$TESTDIR/manifest.json" --output "$$OUT/b" --rng-seed "$(MK.SEED)"; \
	cmp -s "$$OUT/a/result.json" "$$OUT/b/result.json"; \
	cmp -s "$$OUT/a/run_record.json" "$$OUT/b/run_record.json"

# ---------------------------------- Hashes ------------------------------------
hash:
	@set -e; \
	if [ -d "$(ARTIFACTS)" ]; then \
	  find "$(ARTIFACTS)" -type f \( -name '*.json' -o -name '*.tar.gz' -o -name '*.zip' \) -print0 | \
	  while IFS= read -r -d '' f; do \
	    if command -v sha256sum >/dev/null 2>&1; then \
	      sha256sum "$$f" > "$$f.sha256"; \
	    else \
	      shasum -a 256 "$$f" > "$$f.sha256"; \
	    fi; \
	  done; \
	fi; \
	if [ -d "$(DISTDIR)" ]; then \
	  find "$(DISTDIR)" -maxdepth 1 -type f \( -name '*.tar.gz' -o -name '*.zip' \) -print0 | \
	  while IFS= read -r -d '' f; do \
	    if command -v sha256sum >/dev/null 2>&1; then \
	      sha256sum "$$f" > "$$f.sha256"; \
	    else \
	      shasum -a 256 "$$f" > "$$f.sha256"; \
	    fi; \
	  done; \
	fi

# ---------------------------------- Dist --------------------------------------
# Produce reproducible tar.gz and zip bundles (sorted entries, fixed mtimes/owners).
dist: build
	@set -e; \
	rm -rf "$(DISTSTAGE)"; mkdir -p "$(DISTSTAGE)"; \
	cp -a "$(BIN)" "$(DISTSTAGE)/$(BIN_NAME)$(EXE)"; \
	[ -f LICENSE ]   && cp -a LICENSE   "$(DISTSTAGE)/" || true; \
	[ -f README.md ] && cp -a README.md "$(DISTSTAGE)/" || true; \
	# Normalize mtimes for reproducibility:
	find "$(DISTSTAGE)" -print0 | xargs -0r touch -d "@$(SOURCE_DATE_EPOCH)"; \
	mkdir -p "$(DISTDIR)"; \
	# Prefer GNU tar (gtar) for --sort=name; fall back to system tar if needed.
	if command -v gtar >/dev/null 2>&1; then \
	  gtar --sort=name --owner=0 --group=0 --numeric-owner --mtime=@$(SOURCE_DATE_EPOCH) \
	       -C "$(STAGE)" -czf "$(DISTDIR)/$(DISTNAME).tar.gz" "$(DISTNAME)"; \
	else \
	  tar --owner=0 --group=0 --numeric-owner --mtime=@$(SOURCE_DATE_EPOCH) \
	      -C "$(STAGE)" -czf "$(DISTDIR)/$(DISTNAME).tar.gz" "$(DISTNAME)"; \
	fi; \
	# Zip without extra metadata; file order comes from directory listing (already normalized).
	( cd "$(STAGE)" && zip -X -q -r "../$(DISTDIR)/$(DISTNAME).zip" "$(DISTNAME)" ); \
	echo "Wrote: $(DISTDIR)/$(DISTNAME).tar.gz and .zip"

# ----------------------------------- CI ---------------------------------------
ci: fmt lint build test fixtures verify hash

# ---------------------------------- Clean -------------------------------------
clean:
	@rm -rf target "$(ARTIFACTS)" "$(DISTDIR)" "$(STAGE)"
