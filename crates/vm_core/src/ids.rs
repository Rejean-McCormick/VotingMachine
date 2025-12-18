//! crates/vm_core/src/ids.rs
//! Canonical engine/output IDs and token IDs (no input IDs here).
//! Deterministic, ASCII-only, strict shapes; no I/O.

#![allow(clippy::result_large_err)]

use core::fmt;
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Errors returned when validating or parsing IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdError {
    NonAscii,
    TooLong,
    BadShape,
}

const MAX_ID_LEN: usize = 256;
const HEX64_LEN: usize = 64;
const TOKEN_MAX_LEN: usize = 64;

/// Quickly verify ASCII (no NUL).
#[inline]
fn is_ascii_no_nul(s: &str) -> bool {
    !s.as_bytes().iter().any(|&b| b == 0 || b > 0x7F)
}

/// Lowercase hex (length must be exactly 64).
#[inline]
pub fn is_valid_sha256(s: &str) -> bool {
    if s.len() != HEX64_LEN || !is_ascii_no_nul(s) {
        return false;
    }
    s.as_bytes()
        .iter()
        .all(|&b| (b'0'..=b'9').contains(&b) || (b'a'..=b'f').contains(&b))
}

/// Token for UnitId/OptionId: ^[A-Za-z0-9_.:-]{1,64}$ (ASCII only)
#[inline]
pub fn is_valid_token(s: &str) -> bool {
    let bs = s.as_bytes();
    let len = bs.len();
    if len == 0 || len > TOKEN_MAX_LEN || !is_ascii_no_nul(s) {
        return false;
    }
    bs.iter().all(|&b| {
        (b'A'..=b'Z').contains(&b)
            || (b'a'..=b'z').contains(&b)
            || (b'0'..=b'9').contains(&b)
            || b == b'_'
            || b == b'.'
            || b == b':'
            || b == b'-'
    })
}

macro_rules! simple_string_newtype {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[cfg_attr(feature = "serde", serde(transparent))]
        pub struct $name(String);

        impl $name {
            #[inline] pub fn as_str(&self) -> &str { &self.0 }
        }

        impl fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
        }

        impl TryFrom<&str> for $name {
            type Error = IdError;
            #[inline]
            fn try_from(value: &str) -> Result<Self, Self::Error> { value.parse() }
        }
    }
}

// === Hex-only newtypes: FormulaId, Sha256 ===

simple_string_newtype!(
    /// 64-hex lowercase formula identifier (FID component).
    FormulaId
);
simple_string_newtype!(
    /// Generic 64-hex lowercase SHA-256 digest newtype.
    Sha256
);

impl FromStr for FormulaId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_ascii_no_nul(s) { return Err(IdError::NonAscii); }
        if s.len() > MAX_ID_LEN { return Err(IdError::TooLong); }
        if !is_valid_sha256(s) { return Err(IdError::BadShape); }
        Ok(FormulaId(s.to_owned()))
    }
}
impl FormulaId {
    #[inline] pub fn as_hex(&self) -> &str { &self.0 }
}

impl FromStr for Sha256 {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_ascii_no_nul(s) { return Err(IdError::NonAscii); }
        if s.len() > MAX_ID_LEN { return Err(IdError::TooLong); }
        if !is_valid_sha256(s) { return Err(IdError::BadShape); }
        Ok(Sha256(s.to_owned()))
    }
}
impl Sha256 {
    #[inline] pub fn as_hex(&self) -> &str { &self.0 }
}

// === Token IDs: UnitId, OptionId (no prefixes) ===

simple_string_newtype!(
    /// Registry Unit token: ^[A-Za-z0-9_.:-]{1,64}$
    UnitId
);
simple_string_newtype!(
    /// Registry Option token: ^[A-Za-z0-9_.:-]{1,64}$
    OptionId
);

impl FromStr for UnitId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_ascii_no_nul(s) { return Err(IdError::NonAscii); }
        if s.len() > MAX_ID_LEN { return Err(IdError::TooLong); }
        if !is_valid_token(s) { return Err(IdError::BadShape); }
        Ok(UnitId(s.to_owned()))
    }
}

impl FromStr for OptionId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_ascii_no_nul(s) { return Err(IdError::NonAscii); }
        if s.len() > MAX_ID_LEN { return Err(IdError::TooLong); }
        if !is_valid_token(s) { return Err(IdError::BadShape); }
        Ok(OptionId(s.to_owned()))
    }
}

// === Prefixed output IDs: RES, RUN, FR ===

simple_string_newtype!(
    /// "RES:" + 64-hex lowercase
    ResultId
);
simple_string_newtype!(
    /// "RUN:" + <RFC3339 UTC 'YYYY-MM-DDTHH:MM:SSZ'> + "-" + 64-hex lowercase
    RunId
);
simple_string_newtype!(
    /// "FR:" + 64-hex lowercase
    FrontierMapId
);

#[inline]
fn is_res_shape(s: &str) -> bool {
    s.len() == 4 + HEX64_LEN
        && s.as_bytes().get(0..4) == Some(b"RES:")
        && is_valid_sha256(&s[4..])
}

#[inline]
fn is_fr_shape(s: &str) -> bool {
    s.len() == 3 + HEX64_LEN
        && s.as_bytes().get(0..3) == Some(b"FR:")
        && is_valid_sha256(&s[3..])
}

/// Strict RFC3339 "YYYY-MM-DDTHH:MM:SSZ"
#[inline]
fn is_rfc3339_utc_20(ts: &str) -> bool {
    let b = ts.as_bytes();
    if b.len() != 20 { return false; }
    // YYYY-MM-DDTHH:MM:SSZ
    let digits = |r: core::ops::Range<usize>| b[r].iter().all(|&c| (b'0'..=b'9').contains(&c));
    digits(0..4)
        && b[4] == b'-'
        && digits(5..7)
        && b[7] == b'-'
        && digits(8..10)
        && b[10] == b'T'
        && digits(11..13)
        && b[13] == b':'
        && digits(14..16)
        && b[16] == b':'
        && digits(17..19)
        && b[19] == b'Z'
}

#[inline]
fn is_run_shape(s: &str) -> bool {
    // "RUN:" + ts(20) + "-" + hex64
    if s.len() != 4 + 20 + 1 + HEX64_LEN { return false; }
    let b = s.as_bytes();
    if b.get(0..4) != Some(b"RUN:") { return false; }
    let ts = &s[4..24];
    if !is_rfc3339_utc_20(ts) { return false; }
    if b[24] != b'-' { return false; }
    is_valid_sha256(&s[25..])
}

impl FromStr for ResultId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_ascii_no_nul(s) { return Err(IdError::NonAscii); }
        if s.len() > MAX_ID_LEN { return Err(IdError::TooLong); }
        if !is_res_shape(s) { return Err(IdError::BadShape); }
        Ok(ResultId(s.to_owned()))
    }
}
impl ResultId {
    #[inline] pub fn as_hex(&self) -> &str { &self.0[4..] }
}

impl FromStr for FrontierMapId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_ascii_no_nul(s) { return Err(IdError::NonAscii); }
        if s.len() > MAX_ID_LEN { return Err(IdError::TooLong); }
        if !is_fr_shape(s) { return Err(IdError::BadShape); }
        Ok(FrontierMapId(s.to_owned()))
    }
}
impl FrontierMapId {
    #[inline] pub fn as_hex(&self) -> &str { &self.0[3..] }
}

impl FromStr for RunId {
    type Err = IdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_ascii_no_nul(s) { return Err(IdError::NonAscii); }
        if s.len() > MAX_ID_LEN { return Err(IdError::TooLong); }
        if !is_run_shape(s) { return Err(IdError::BadShape); }
        Ok(RunId(s.to_owned()))
    }
}

impl RunId {
    /// Fast accessor to the embedded timestamp (RFC3339 UTC).
    #[inline]
    pub fn timestamp_utc(&self) -> &str {
        // "RUN:" + <ts 20> + "-" + hex64
        &self.0[4..24]
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha_and_formula() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd";
        assert!(is_valid_sha256(hex));
        let fid: FormulaId = hex.parse().unwrap();
        assert_eq!(fid.as_hex(), hex);
        let dig: Sha256 = hex.parse().unwrap();
        assert_eq!(format!("{dig}"), hex);
        assert!("0123XYZ".parse::<Sha256>().is_err());
    }

    #[test]
    fn tokens() {
        for ok in ["A", "a", "9", "_", ".", ":", "-", "A_b:9.Z"] {
            assert!(is_valid_token(ok));
            let _u: UnitId = ok.parse().unwrap();
            let _o: OptionId = ok.parse().unwrap();
        }
        for bad in ["", " ", "é", "toolong_________________________________________________________________"] {
            assert!(!is_valid_token(bad));
            assert!(bad.parse::<UnitId>().is_err());
        }
    }

    #[test]
    fn res_fr_run() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcd";
        let res_s = format!("RES:{hex}");
        let fr_s  = format!("FR:{hex}");
        let run_s = format!("RUN:2025-08-12T14:00:00Z-{hex}");

        let res: ResultId = res_s.parse().unwrap();
        let fr:  FrontierMapId = fr_s.parse().unwrap();
        let run: RunId = run_s.parse().unwrap();

        assert_eq!(res.as_hex(), hex);
        assert_eq!(fr.as_hex(), hex);
        assert_eq!(run.timestamp_utc(), "2025-08-12T14:00:00Z");

        // Round-trip
        assert_eq!(format!("{res}"), res_s);
        assert_eq!(format!("{fr}"), fr_s);
        assert_eq!(format!("{run}"), run_s);

        // Bad shapes
        assert!("RES:DEADBEAF".parse::<ResultId>().is_err());
        assert!("FR:0123XYZ...".parse::<FrontierMapId>().is_err());
        assert!("RUN:2025-08-12T14:00:00-0123".parse::<RunId>().is_err()); // missing Z and '-'
        assert!("RUN:2025-08-12 14:00:00Z-".to_string() + hex
            .as_str()).parse::<RunId>().is_err(); // space instead of 'T'
    }
}
