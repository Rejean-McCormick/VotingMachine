[package]
name = "vm_app_tauri"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0 OR MIT"
publish = false
description = "Tauri backend for the VM Engine desktop app (offline, deterministic)."
# Use resolver v2 for correct feature unification.
resolver = "2"

# --- Binary target ---
[[bin]]
name = "vm-app"
path = "src/main.rs"

# --- Features (pass-through; backend remains offline) ---
[features]
default = []
# Enables frontier map support through downstream crates (if they gate it).
frontier = ["vm_pipeline?/frontier"]
# Enables HTML renderer in reporting.
report-html = ["vm_report?/render_html"]

# --- Dependencies (workspace-pinned for determinism) ---
[dependencies]
tauri       = { workspace = true, features = ["fs-all", "dialog-all", "shell-open"] }
serde       = { workspace = true, features = ["derive"] }
serde_json  = { workspace = true }

# Internal crates (path/workspace). Keep optional where not always needed.
vm_core     = { workspace = true }
vm_io       = { workspace = true }
vm_algo     = { workspace = true }
vm_pipeline = { workspace = true, optional = true }
vm_report   = { workspace = true, optional = true, default-features = false }

# If your workspace doesn't define these, replace `workspace = true` with pinned versions or local paths, e.g.:
# tauri = { version = "=1.5.12", features = ["fs-all","dialog-all","shell-open"] }
# vm_pipeline = { path = "../../vm_pipeline", optional = true }
# vm_report   = { path = "../../vm_report",  optional = true, default-features = false }

# --- Build dependencies ---
[build-dependencies]
tauri-build = { workspace = true }

# --- Target-specific (optional, local-only tooling) ---
[target."cfg(windows)".dependencies]
# Example: embed icons/resources locally (kept optional/off by default)
# winresource = { workspace = true, optional = true }

# --- Deterministic profiles ---
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
panic = "abort"
