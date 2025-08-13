//! build_run_record.rs — Compose the RunRecord artifact (deterministic).
//!
//! Responsibilities:
//! - Validate UTCs, hex64 digests, one-of inputs (ballots vs tally), RNG policy rules
//! - Build an *idless* record and canonicalize it (vm_io::canonical_json)
//! - Hash (SHA-256) the idless bytes and form `RUN:<started_utc>-<short-hash>`
//! - Return a fully-populated `RunRecordDoc` including ties (Result never carries tie logs)

use std::collections::BTreeMap;

use vm_core::ids::{ParamSetId, RegId, ResultId, TallyId};
use vm_io::{
    canonical_json::to_canonical_bytes,
    hasher::sha256_hex,
};

/// Engine identifiers (mirrors pipeline-wide metadata).
#[derive(Clone, Debug)]
pub struct EngineMeta {
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub build: String,
}

/// Tie policy recorded in RunRecord. RNG seed is only included when `Random`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TiePolicy {
    StatusQuo,
    Deterministic,
    Random,
}

impl TiePolicy {
    fn as_str(&self) -> &'static str {
        match self {
            TiePolicy::StatusQuo => "status_quo",
            TiePolicy::Deterministic => "deterministic",
            TiePolicy::Random => "random",
        }
    }
}

/// References to inputs and their canonical digests.
#[derive(Clone, Debug)]
pub struct InputRefs {
    pub manifest_id: Option<String>,
    pub reg_id: RegId,
    pub parameter_set_id: ParamSetId,
    pub ballots_id: Option<String>,
    pub ballot_tally_id: Option<TallyId>,
    /// path (or logical key) → sha256 hex64
    pub digests: BTreeMap<String, String>,
}

/// References to outputs (Result + optional Frontier) and their digests.
#[derive(Clone, Debug)]
pub struct OutputRefs {
    pub result_id: ResultId,
    pub result_sha256: String,                 // hex64
    pub frontier_map_id: Option<String>,       // FrontierId
    pub frontier_map_sha256: Option<String>,   // hex64; required iff frontier_map_id
}

/// Determinism block written to RunRecord.
#[derive(Clone, Debug)]
pub struct Determinism {
    pub tie_policy: TiePolicy,
    /// 64-hex string, present only when tie_policy == Random
    pub rng_seed_hex64: Option<String>,
}

/// One tie event; kept flexible for audit. Stored verbatim in RunRecord.
#[derive(Clone, Debug)]
pub struct TieEvent {
    pub context: String,             // e.g., "WTA U:…"
    pub candidates: Vec<String>,     // OptionId strings
    pub policy: String,              // "status_quo" | "deterministic" | "random"
    pub detail: Option<String>,      // e.g., "order_index" or "rng:seed=<…>,word=<…>"
    pub winner: String,              // OptionId
}

/// The final RunRecord document (schema-shaped; serialization happens upstream).
#[derive(Clone, Debug)]
pub struct RunRecordDoc {
    pub id: String, // RUN:<ts>-<short>
    pub started_utc: String,
    pub finished_utc: String,
    pub engine: EngineMeta,
    pub formula_id: String,
    pub formula_manifest_sha256: String,
    pub inputs: InputRefs,
    pub determinism: Determinism,
    pub outputs: OutputRefs,
    pub ties: Vec<TieEvent>,
}

/// Builder errors (deterministic and concise).
#[derive(Debug)]
pub enum BuildRunRecordError {
    BadUtc(&'static str, String),
    BadHex64(&'static str, String),
    InputsContract(String),
    OutputsContract(String),
    DeterminismContract(String),
}

/// Build the RunRecord content; ID is computed from canonical bytes (without `id`) + started_utc.
pub fn build_run_record(
    engine: &EngineMeta,
    formula_id: &str,
    formula_manifest_sha256: &str,
    inputs: &InputRefs,
    determinism: &Determinism,
    outputs: &OutputRefs,
    ties: &[TieEvent],
    started_utc: &str,
    finished_utc: &str,
) -> Result<RunRecordDoc, BuildRunRecordError> {
    // ---- validations (pure, deterministic) ----
    validate_utc(started_utc).map_err(|_| BuildRunRecordError::BadUtc("started_utc", started_utc.to_string()))?;
    validate_utc(finished_utc).map_err(|_| BuildRunRecordError::BadUtc("finished_utc", finished_utc.to_string()))?;

    validate_hex64(formula_manifest_sha256)
        .map_err(|_| BuildRunRecordError::BadHex64("formula_manifest_sha256", formula_manifest_sha256.to_string()))?;

    // Input digests must be hex64
    for (k, v) in &inputs.digests {
        validate_hex64(v).map_err(|_| BuildRunRecordError::BadHex64("inputs.digests", format!("{k}={v}")))?;
    }

    check_inputs_coherence(inputs)?;
    check_outputs_coherence(outputs)?;
    check_determinism(determinism)?;

    // ---- Build the *idless* structure for canonical hashing ----
    // We exclude the "id" field itself from the canonicalization.
    let idless = IdlessRunRecord {
        started_utc: started_utc.to_string(),
        finished_utc: finished_utc.to_string(),
        engine: engine.clone(),
        formula_id: formula_id.to_string(),
        formula_manifest_sha256: formula_manifest_sha256.to_string(),
        inputs: inputs.clone(),
        determinism: determinism.clone(),
        outputs: outputs.clone(),
        ties: ties.to_vec(),
    };

    // Canonical bytes (stable across OS/arch) then SHA-256 → hex64
    let canon_bytes = to_canonical_bytes(&idless)
        .expect("canonicalization should not fail for well-formed idless record");
    let canon_sha256 = sha256_hex(&canon_bytes);

    // RUN ID format: RUN:<YYYY-MM-DDTHH-MM-SSZ>-<short>
    let started_friendly = id_friendly_timestamp(started_utc);
    let short = compute_id_short_hash(&canon_sha256, 16);
    let run_id = format!("RUN:{}-{}", started_friendly, short);

    // ---- Assemble final doc (with id) ----
    Ok(RunRecordDoc {
        id: run_id,
        started_utc: started_utc.to_string(),
        finished_utc: finished_utc.to_string(),
        engine: engine.clone(),
        formula_id: formula_id.to_string(),
        formula_manifest_sha256: formula_manifest_sha256.to_string(),
        inputs: inputs.clone(),
        determinism: determinism.clone(),
        outputs: outputs.clone(),
        ties: ties.to_vec(),
    })
}

// ---------- Internal: idless shape (Serialize only here) ----------
#[derive(serde::Serialize)]
struct IdlessRunRecord {
    started_utc: String,
    finished_utc: String,
    engine: EngineMeta,
    formula_id: String,
    formula_manifest_sha256: String,
    inputs: InputRefs,
    determinism: Determinism,
    outputs: OutputRefs,
    ties: Vec<TieEvent>,
}

// Derives for canonicalization (Serialize only)
impl serde::Serialize for EngineMeta {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("EngineMeta", 4)?;
        st.serialize_field("vendor", &self.vendor)?;
        st.serialize_field("name", &self.name)?;
        st.serialize_field("version", &self.version)?;
        st.serialize_field("build", &self.build)?;
        st.end()
    }
}
impl serde::Serialize for InputRefs {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("InputRefs", 5)?;
        st.serialize_field("manifest_id", &self.manifest_id)?;
        st.serialize_field("reg_id", &self.reg_id)?;
        st.serialize_field("parameter_set_id", &self.parameter_set_id)?;
        st.serialize_field("ballots_id", &self.ballots_id)?;
        st.serialize_field("ballot_tally_id", &self.ballot_tally_id)?;
        // digests serialized separately to keep BTreeMap ordering stable
        st.end()?;
        // Serialize `digests` alongside (outer map stability is guaranteed by BTreeMap)
        let mut map = s.serialize_map(Some(self.digests.len()))?;
        for (k, v) in &self.digests {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}
impl serde::Serialize for OutputRefs {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("OutputRefs", 4)?;
        st.serialize_field("result_id", &self.result_id)?;
        st.serialize_field("result_sha256", &self.result_sha256)?;
        st.serialize_field("frontier_map_id", &self.frontier_map_id)?;
        st.serialize_field("frontier_map_sha256", &self.frontier_map_sha256)?;
        st.end()
    }
}
impl serde::Serialize for Determinism {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("Determinism", 2)?;
        st.serialize_field("tie_policy", &self.tie_policy.as_str())?;
        // seed only when Random (otherwise None)
        let seed = match (self.tie_policy, &self.rng_seed_hex64) {
            (TiePolicy::Random, Some(v)) => Some(v),
            _ => None,
        };
        st.serialize_field("rng_seed", &seed)?;
        st.end()
    }
}
impl serde::Serialize for TieEvent {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("TieEvent", 5)?;
        st.serialize_field("context", &self.context)?;
        st.serialize_field("candidates", &self.candidates)?;
        st.serialize_field("policy", &self.policy)?;
        st.serialize_field("detail", &self.detail)?;
        st.serialize_field("winner", &self.winner)?;
        st.end()
    }
}

// ---------- Validation helpers ----------

/// Validate "YYYY-MM-DDTHH:MM:SSZ" (basic structural check; semantics enforced upstream).
pub fn validate_utc(ts: &str) -> Result<(), BuildRunRecordError> {
    // Minimal fixed-length + character-position validation.
    if ts.len() != 20 {
        return Err(BuildRunRecordError::BadUtc("format", ts.to_string()));
    }
    let b = ts.as_bytes();
    let ok = b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':'
        && b[19] == b'Z'
        && b.iter().enumerate().all(|(i, &c)| match i {
            4
