//! AttrType / AttrTypeRegistry — the Unified Attribute Data Type Registry
//! (EnkiMDB internal registries, 2026-07-24).
//!
//! Solves a real drift problem: two clients writing "the same" numeric
//! attribute (e.g. a price) under different precisions -- 16,2 vs 18,1 vs
//! 20,3 -- silently break comparisons, joins, and aggregates once that
//! data flows across the 7 EnkiDB Types. `Template`/`FieldSpec` (§6.1,
//! `template.rs`) already answer "which attributes does this particle
//! class have, and are they required" -- AttrType answers the question
//! Template never asked: "what is THE canonical physical representation
//! of this specific attribute, everywhere it appears?"
//!
//! One AttrTypeSpec exists per attr_hash, registered once (low
//! cardinality, like Template itself), version-tagged so a later
//! redefinition (e.g. widening Latitude's precision) doesn't silently
//! reinterpret data written under the old definition.

/// Canonical physical representation for an attribute's value bytes.
///
/// Deliberately NOT the same enum as `akkvalue::AkkValue` (that's the
/// value system; this is a narrower schema-level CONSTRAINT on top of
/// it) and deliberately separates fixed-point from float: money-like
/// attributes must never be a binary float (0.10 has no exact binary
/// representation; sums drift), so the registry forces that choice to
/// be explicit at registration time rather than left to whichever
/// client writes first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AttrType {
    /// Scaled fixed-point integer, stored as an 8-byte big-endian i64
    /// (`scale` decimal places implied, not stored per-value). The only
    /// safe representation for anything summed/compared exactly --
    /// currency, counts-with-fractional-units.
    FixedPointScaledInteger { scale: u8 },
    /// IEEE 754 binary32 (4 bytes). Acceptable for physical measurements
    /// where exact decimal representation was never expected (sensor
    /// readings, geometry) -- NOT for money.
    Float32,
    /// IEEE 754 binary64 (8 bytes).
    Float64,
    /// Signed 64-bit integer (8 bytes), no fractional part.
    Integer,
    /// UTF-8 text, no numeric semantics, variable length.
    Text,
    /// Raw bytes, caller-defined structure, variable length.
    Blob,
}

impl AttrType {
    pub fn as_str(self) -> &'static str {
        match self {
            AttrType::FixedPointScaledInteger { .. } => "FIXED_POINT_SCALED_INTEGER",
            AttrType::Float32                        => "FLOAT32",
            AttrType::Float64                        => "FLOAT64",
            AttrType::Integer                        => "INTEGER",
            AttrType::Text                           => "TEXT",
            AttrType::Blob                           => "BLOB",
        }
    }

    /// The exact wire byte length this representation requires, or
    /// `None` for variable-length kinds (Text/Blob) where no length
    /// check applies.
    pub fn expected_byte_len(self) -> Option<usize> {
        match self {
            AttrType::FixedPointScaledInteger { .. } => Some(8),
            AttrType::Float32                        => Some(4),
            AttrType::Float64                        => Some(8),
            AttrType::Integer                        => Some(8),
            AttrType::Text | AttrType::Blob           => None,
        }
    }
}

/// One canonical, registered type for a specific attribute.
#[derive(Debug, Clone)]
pub struct AttrTypeSpec {
    /// Attribute hash constant (same hash space as `FieldSpec::attr_hash`
    /// / `story_engine::projection::ATTR_*`).
    pub attr_hash: u32,
    /// Human-readable name, e.g. "price_usd", "latitude".
    pub name: &'static str,
    pub attr_type: AttrType,
    /// Bumped on any redefinition. Data written under version N keeps
    /// meaning what it meant under version N -- widening precision later
    /// is a new version, never a silent reinterpretation of old bytes.
    pub version: u16,
}

impl AttrTypeSpec {
    pub const fn new(attr_hash: u32, name: &'static str, attr_type: AttrType, version: u16) -> Self {
        AttrTypeSpec { attr_hash, name, attr_type, version }
    }
}

/// Why a value's bytes don't match its registered AttrType.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeViolation {
    /// No AttrTypeSpec exists for this attr_hash -- nothing to check
    /// against. Not necessarily an error by itself (an unregistered
    /// attribute may be legitimate, stakeholder-extension data); callers
    /// decide whether to require registration.
    UnregisteredAttribute { attr_hash: u32 },
    /// The value's byte length doesn't match what the registered
    /// AttrType requires.
    WrongByteLength { attr_hash: u32, expected: usize, actual: usize },
}

/// Central registry — one canonical AttrType per attr_hash.
pub struct AttrTypeRegistry {
    specs: std::collections::HashMap<u32, AttrTypeSpec>,
}

impl AttrTypeRegistry {
    pub fn new() -> Self {
        AttrTypeRegistry { specs: std::collections::HashMap::new() }
    }

    /// Register a canonical type. Returns false if this attr_hash is
    /// already registered -- use a new `version` and re-register
    /// deliberately rather than silently overwriting (mirrors
    /// `TemplateRegistry::register`'s duplicate-rejection).
    pub fn register(&mut self, spec: AttrTypeSpec) -> bool {
        if self.specs.contains_key(&spec.attr_hash) {
            return false;
        }
        self.specs.insert(spec.attr_hash, spec);
        true
    }

    pub fn get(&self, attr_hash: u32) -> Option<&AttrTypeSpec> {
        self.specs.get(&attr_hash)
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// Validate a value's raw bytes against its attribute's registered
    /// AttrType. This is the real enforcement point: call it from the
    /// same ingest path `template_engine::validate_required` already
    /// guards (presence), as a second, independent check (precision/
    /// representation).
    pub fn validate_value(&self, attr_hash: u32, value: &[u8]) -> Result<(), TypeViolation> {
        let spec = self.get(attr_hash)
            .ok_or(TypeViolation::UnregisteredAttribute { attr_hash })?;
        if let Some(expected) = spec.attr_type.expected_byte_len() {
            if value.len() != expected {
                return Err(TypeViolation::WrongByteLength {
                    attr_hash,
                    expected,
                    actual: value.len(),
                });
            }
        }
        Ok(())
    }
}

impl Default for AttrTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRICE_USD: u32 = 0x1001;
    const LATITUDE:  u32 = 0x1002;
    const NOTES:     u32 = 0x1003;

    fn registry() -> AttrTypeRegistry {
        let mut r = AttrTypeRegistry::new();
        assert!(r.register(AttrTypeSpec::new(PRICE_USD, "price_usd", AttrType::FixedPointScaledInteger { scale: 2 }, 1)));
        assert!(r.register(AttrTypeSpec::new(LATITUDE, "latitude", AttrType::Float64, 1)));
        assert!(r.register(AttrTypeSpec::new(NOTES, "notes", AttrType::Text, 1)));
        r
    }

    #[test]
    fn register_and_get() {
        let r = registry();
        assert_eq!(r.len(), 3);
        let spec = r.get(PRICE_USD).unwrap();
        assert_eq!(spec.name, "price_usd");
        assert_eq!(spec.attr_type, AttrType::FixedPointScaledInteger { scale: 2 });
    }

    #[test]
    fn duplicate_attr_hash_rejected() {
        let mut r = registry();
        assert!(!r.register(AttrTypeSpec::new(PRICE_USD, "price_usd_v2", AttrType::Float64, 2)));
        assert_eq!(r.len(), 3, "duplicate must not overwrite");
        assert_eq!(r.get(PRICE_USD).unwrap().name, "price_usd");
    }

    #[test]
    fn unregistered_attribute_returns_violation() {
        let r = registry();
        let result = r.validate_value(0xDEAD_BEEF, &[0u8; 8]);
        assert_eq!(result, Err(TypeViolation::UnregisteredAttribute { attr_hash: 0xDEAD_BEEF }));
    }

    #[test]
    fn correct_fixed_point_length_passes() {
        let r = registry();
        let cents: i64 = 1999; // $19.99 at scale=2
        assert!(r.validate_value(PRICE_USD, &cents.to_be_bytes()).is_ok());
    }

    #[test]
    fn wrong_length_for_fixed_point_rejected() {
        let r = registry();
        // A price stored as a 4-byte f32 instead of the registered
        // 8-byte scaled integer -- exactly the drift this registry
        // exists to catch.
        let wrong: f32 = 19.99;
        let result = r.validate_value(PRICE_USD, &wrong.to_be_bytes());
        assert_eq!(result, Err(TypeViolation::WrongByteLength { attr_hash: PRICE_USD, expected: 8, actual: 4 }));
    }

    #[test]
    fn float64_length_check() {
        let r = registry();
        let lat: f64 = 33.3152;
        assert!(r.validate_value(LATITUDE, &lat.to_be_bytes()).is_ok());
        assert!(r.validate_value(LATITUDE, &[0u8; 4]).is_err());
    }

    #[test]
    fn text_has_no_length_constraint() {
        let r = registry();
        assert!(r.validate_value(NOTES, b"any length at all, arbitrarily long text").is_ok());
        assert!(r.validate_value(NOTES, b"").is_ok());
    }

    #[test]
    fn attr_type_as_str() {
        assert_eq!(AttrType::FixedPointScaledInteger { scale: 2 }.as_str(), "FIXED_POINT_SCALED_INTEGER");
        assert_eq!(AttrType::Float64.as_str(), "FLOAT64");
    }

    #[test]
    fn versioned_redefinition_is_a_new_spec_not_a_mutation() {
        // Widening latitude's precision later must be an explicit new
        // registration under a fresh attr_hash (or, operationally, a
        // deliberate migration) -- never a silent overwrite of what
        // version 1 meant. This test documents that `register` refuses
        // the naive "just insert again" path.
        let mut r = registry();
        let widened = AttrTypeSpec::new(LATITUDE, "latitude", AttrType::Float64, 2);
        assert!(!r.register(widened), "must not silently reinterpret an existing attr_hash");
    }
}
