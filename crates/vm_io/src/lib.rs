//! crates/vm_io/src/lib.rs — Part 1/2
//! Minimal, single-source-of-truth I/O crate.
//!
//! - No inline implementations: we re-export the **file modules** to avoid drift.
//! - Shared error type (`IoError`) with `From` conversions used across modules.
//! - Public surface kept stable; details live in submodules.
//!
//! Part 2 adds a small prelude and convenience re-exports.

#![forbid(unsafe_code)]

use std::fmt;

use serde::{de::Error as _, Deserialize, Serialize};
use thiserror::Error;

/// Unified error for vm_io (used by canonical_json/manifest/hasher/schema).
#[derive(Debug, Error)]
pub enum IoError {
    /// Filesystem / path errors (create_dir_all, rename, fsync, etc.)
    #[error("io/path error: {0}")]
    Path(String),

    /// JSON serialization/deserialization errors with an optional JSON Pointer.
    #[error("json error at {pointer}: {msg}")]
    Json {
        pointer: String,
        msg: String,
    },

    /// Hashing-related errors (e.g., feature disabled, read failures).
    #[error("hash error: {0}")]
    Hash(String),

    /// Schema-related errors (JSON Schema validation failures).
    #[error("schema error: {0}")]
    Schema(String),

    /// Generic validation / invariants.
    #[error("invalid: {0}")]
    Invalid(String),
}

pub type IoResult<T> = Result<T, IoError>;

/* ---------------- From conversions (used by file modules) ---------------- */

impl From<std::io::Error> for IoError {
    fn from(e: std::io::Error) -> Self {
        IoError::Path(e.to_string())
    }
}

impl From<serde_json::Error> for IoError {
    fn from(e: serde_json::Error) -> Self {
        // If available, include a pointer-like hint; serde_json doesn't keep a pointer,
        // so we default to root. Callers may enrich this at higher layers.
        IoError::Json {
            pointer: "/".to_string(),
            msg: e.to_string(),
        }
    }
}

/* ---------------- Public modules (single source of truth) ----------------
   IMPORTANT: These correspond to files:
     - src/canonical_json.rs
     - src/hasher.rs
     - src/manifest.rs
     - src/schema.rs
   Remove ALL inline duplicates to prevent drift.
------------------------------------------------------------------------- */

pub mod canonical_json;
pub mod hasher;
pub mod manifest;
pub mod schema;
//! crates/vm_io/src/lib.rs — Part 2/2
//! Prelude & convenience wrappers. Keep this file minimal; real logic lives in submodules.

#![forbid(unsafe_code)]

use crate::IoError;

/* ---------------- Convenience: fallible hash wrapper ----------------
   Rationale: some historical implementations returned an empty string when the
   `hash` feature was disabled. That’s dangerous. Callers should prefer this
   wrapper, which fails loudly when hashing isn’t available.
--------------------------------------------------------------------- */

/// Compute SHA-256 hex of `bytes` or return an error when hashing is unavailable.
pub fn try_sha256_hex(bytes: &[u8]) -> Result<String, IoError> {
    #[cfg(feature = "hash")]
    {
        Ok(crate::hasher::sha256_hex(bytes))
    }
    #[cfg(not(feature = "hash"))]
    {
        Err(IoError::Hash("hash feature disabled".into()))
    }
}

/* ---------------- Optional helper: strict URL detector ----------------
   Manifest loading in this crate follows a strict offline posture.
   Use this helper when you need to reject any "<scheme>://" path early.
--------------------------------------------------------------------- */

/// Returns true if `s` looks like a URL (any `<scheme>://`, including `file://`).
#[inline]
pub fn looks_like_url_strict(s: &str) -> bool {
    s.trim().contains("://")
}

/* ---------------- Public prelude ----------------
   Lightweight re-exports so downstream crates can do:
     use vm_io::prelude::*;
------------------------------------------------- */

pub mod prelude {
    pub use crate::{IoError, IoResult, looks_like_url_strict, try_sha256_hex};

    // Re-export modules (callers can choose granular imports from here)
    pub use crate::canonical_json;
    pub use crate::hasher;
    pub use crate::manifest;
    pub use crate::schema;

    // Commonly used items (stable symbols used across the workspace)
    pub use crate::canonical_json::to_canonical_bytes;
    #[cfg(feature = "hash")]
    pub use crate::hasher::sha256_hex;
}
