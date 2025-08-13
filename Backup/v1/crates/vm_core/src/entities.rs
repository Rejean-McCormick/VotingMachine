//! crates/vm_core/src/entities.rs
//! Domain entities shared across the engine (registry, units, options, tallies).
//! Pure types + invariants + deterministic ordering helpers. No I/O.

#![allow(clippy::result_large_err)]

use core::cmp::Ordering;
use core::fmt;

use crate::ids::{OptionId, UnitId};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Construction/validation errors for domain entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityError {
    EmptyCollection,
    InvalidName,
}

impl fmt::Display for EntityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityError::EmptyCollection => f.write_str("empty collection"),
            EntityError::InvalidName => f.write_str("invalid name"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EntityError {}

const NAME_MIN_LEN: usize = 1;
const NAME_MAX_LEN: usize = 200;

#[inline]
fn is_valid_name(s: &str) -> bool {
    let len = s.chars().count();
    (NAME_MIN_LEN..=NAME_MAX_LEN).contains(&len)
}

/// Canonical registry of divisions/units/options.
/// Invariant: `units.len() >= 1`, units kept (or sortable) in ↑ UnitId order.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DivisionRegistry {
    pub schema_version: String,
    pub units: Vec<Unit>,
}

impl DivisionRegistry {
    /// Construct a registry, enforcing non-empty & per-unit invariants.
    pub fn new(schema_version: String, mut units: Vec<Unit>) -> Result<Self, EntityError> {
        if units.is_empty() {
            return Err(EntityError::EmptyCollection);
        }
        // Validate units & their options
        for u in &units {
            u.assert_invariants()?;
        }
        // Canonicalize order
        sort_units_by_id(&mut units);
        Ok(Self { schema_version, units })
    }

    /// Read-only view of all units.
    #[inline]
    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    /// Find a unit by id (linear scan; call after canonical sort if you prefer binary_search).
    #[inline]
    pub fn unit(&self, id: &UnitId) -> Option<&Unit> {
        self.units.iter().find(|u| &u.unit_id == id)
    }
}

/// A voting/geographic unit (e.g., district, ward).
/// Invariant: name: 1..=200 chars; `options.len() >= 1`; options kept in ↑(order_index, option_id).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Unit {
    pub unit_id: UnitId,
    pub name: String,
    pub protected_area: bool,
    pub options: Vec<OptionItem>,
}

impl Unit {
    pub fn new(
        unit_id: UnitId,
        name: String,
        protected_area: bool,
        mut options: Vec<OptionItem>,
    ) -> Result<Self, EntityError> {
        if !is_valid_name(&name) {
            return Err(EntityError::InvalidName);
        }
        if options.is_empty() {
            return Err(EntityError::EmptyCollection);
        }
        for o in &options {
            o.assert_invariants()?;
        }
        sort_options_canonical(&mut options);
        Ok(Self {
            unit_id,
            name,
            protected_area,
            options,
        })
    }

    /// Root-ness is a pipeline concern; keep API but return false to avoid leaking policy here.
    #[inline]
    pub fn is_root(&self) -> bool {
        false
    }

    #[inline]
    fn assert_invariants(&self) -> Result<(), EntityError> {
        if !is_valid_name(&self.name) {
            return Err(EntityError::InvalidName);
        }
        if self.options.is_empty() {
            return Err(EntityError::EmptyCollection);
        }
        for o in &self.options {
            o.assert_invariants()?;
        }
        Ok(())
    }
}

/// An option/party/candidate entry within a unit.
/// Invariant: name 1..=200 chars; order_index is a u16; canonical cmp is (order_index, option_id).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OptionItem {
    pub option_id: OptionId,
    pub name: String,
    pub order_index: u16,
}

impl OptionItem {
    pub fn new(option_id: OptionId, name: String, order_index: u16) -> Result<Self, EntityError> {
        if !is_valid_name(&name) {
            return Err(EntityError::InvalidName);
        }
        Ok(Self {
            option_id,
            name,
            order_index,
        })
    }

    #[inline]
    fn assert_invariants(&self) -> Result<(), EntityError> {
        if !is_valid_name(&self.name) {
            return Err(EntityError::InvalidName);
        }
        Ok(())
    }
}

/// Per-unit totals, mirroring the BallotTally aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TallyTotals {
    pub valid_ballots: u64,
    pub invalid_ballots: u64,
}

impl TallyTotals {
    #[inline]
    pub fn new(valid_ballots: u64, invalid_ballots: u64) -> Self {
        Self {
            valid_ballots,
            invalid_ballots,
        }
    }

    /// Total ballots cast (saturating to guard against theoretical overflow).
    #[inline]
    pub fn ballots_cast(&self) -> u64 {
        self.valid_ballots.saturating_add(self.invalid_ballots)
    }
}

/// Deterministic canonical comparison for options: ↑ (order_index, option_id).
#[inline]
pub fn cmp_options(a: &OptionItem, b: &OptionItem) -> Ordering {
    match a.order_index.cmp(&b.order_index) {
        Ordering::Equal => a.option_id.cmp(&b.option_id),
        ord => ord,
    }
}

/// Sort units by UnitId ascending (stable).
#[inline]
pub fn sort_units_by_id(units: &mut [Unit]) {
    units.sort_by(|a, b| a.unit_id.cmp(&b.unit_id));
}

/// Sort options canonically by (order_index, option_id) (stable).
#[inline]
pub fn sort_options_canonical(opts: &mut [OptionItem]) {
    opts.sort_by(cmp_options);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(s: &str) -> UnitId {
        s.parse().unwrap()
    }
    fn oid(s: &str) -> OptionId {
        s.parse().unwrap()
    }

    #[test]
    fn name_bounds() {
        assert!(is_valid_name("A"));
        assert!(is_valid_name(&"x".repeat(200)));
        assert!(!is_valid_name(""));
        assert!(!is_valid_name(&"x".repeat(201)));
    }

    #[test]
    fn option_new_and_invariants() {
        let ok = OptionItem::new(oid("A"), "Alpha".into(), 0).unwrap();
        assert_eq!(ok.order_index, 0);
        assert!(OptionItem::new(oid("B"), "".into(), 1).is_err());
    }

    #[test]
    fn unit_new_and_sorting() {
        let o3 = OptionItem::new(oid("Z"), "Zed".into(), 2).unwrap();
        let o1 = OptionItem::new(oid("A"), "Alpha".into(), 0).unwrap();
        let o2 = OptionItem::new(oid("B"), "Bravo".into(), 1).unwrap();
        let mut u = Unit::new(uid("U1"), "Unit".into(), false, vec![o3.clone(), o2.clone(), o1.clone()]).unwrap();
        // Canonical order check: (order_index, option_id)
        assert_eq!(u.options[0].option_id, oid("A"));
        assert_eq!(u.options[1].option_id, oid("B"));
        assert_eq!(u.options[2].option_id, oid("Z"));

        // Bad name
        assert!(Unit::new(uid("U2"), "".into(), false, vec![o1.clone()]).is_err());
        // Empty options
        assert!(Unit::new(uid("U3"), "Ok".into(), false, vec![]).is_err());
    }

    #[test]
    fn registry_new_and_lookup() {
        let u1 = Unit::new(uid("A"), "A".into(), false, vec![OptionItem::new(oid("X"), "X".into(), 1).unwrap()]).unwrap();
        let u2 = Unit::new(uid("B"), "B".into(), true,  vec![OptionItem::new(oid("Y"), "Y".into(), 0).unwrap()]).unwrap();
        let reg = DivisionRegistry::new("1.0".into(), vec![u2.clone(), u1.clone()]).unwrap();

        // Sorted by unit_id
        assert_eq!(reg.units[0].unit_id, uid("A"));
        assert_eq!(reg.units[1].unit_id, uid("B"));

        // Lookup
        assert!(reg.unit(&uid("A")).is_some());
        assert!(reg.unit(&uid("Z")).is_none());
    }

    #[test]
    fn totals_sum_is_saturating() {
        let t = TallyTotals::new(u64::MAX, 10);
        assert_eq!(t.ballots_cast(), u64::MAX); // saturates, never panics
    }

    #[test]
    fn cmp_and_sort_options() {
        let mut v = vec![
            OptionItem::new(oid("B"), "B".into(), 0).unwrap(),
            OptionItem::new(oid("A"), "A".into(), 0).unwrap(),
            OptionItem::new(oid("C"), "C".into(), 1).unwrap(),
        ];
        sort_options_canonical(&mut v);
        assert_eq!(v.iter().map(|o| o.option_id.as_str()).collect::<Vec<_>>(), ["A", "B", "C"]);
    }
}
