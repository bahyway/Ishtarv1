//! VgcaBeam — template validation + structural EAV check (§10.3).
//!
//! VGCA = Validation Gate with ColorID Assignment.
//! The beam validates that the incoming EAV set satisfies the template's
//! required fields, returning the list of missing required attr_hashes.

use enkidb_journal::entry::EavTriple;
use template_engine::{validate_required, AttrTypeRegistry, Template, TypeViolation};

/// Result of VGCA beam validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// attr_hashes of required fields that are absent from the EAV set.
    pub missing_required: Vec<u32>,
    /// Number of EAV triples that don't appear in the template at all.
    pub unknown_attrs: usize,
    /// AttrType Registry mismatches found by `validate_with_types`. Empty
    /// when `validate` was used instead, or when no violation occurred.
    pub type_violations: Vec<TypeViolation>,
}

impl ValidationResult {
    /// A missing required field always fails. A `WrongByteLength`
    /// type violation always fails. An `UnregisteredAttribute` does NOT
    /// fail by itself -- per its own doc comment, an attribute with no
    /// canonical AttrType registered yet is not necessarily an error.
    pub fn is_valid(&self) -> bool {
        self.missing_required.is_empty()
            && !self
                .type_violations
                .iter()
                .any(|v| matches!(v, TypeViolation::WrongByteLength { .. }))
    }
}

/// Validate an EAV set against a template (required-field presence only).
pub fn validate(template: &Template, eav: &[EavTriple]) -> ValidationResult {
    let present: Vec<u32> = eav.iter().map(|t| t.attr_hash).collect();
    let missing_required = validate_required(template, &present);
    let unknown_attrs = eav
        .iter()
        .filter(|t| !template.contains_attr(t.attr_hash))
        .count();
    ValidationResult {
        missing_required,
        unknown_attrs,
        type_violations: Vec::new(),
    }
}

/// Same as `validate`, plus a second, independent check: each EAV triple's
/// value bytes against the Unified Attribute Type Registry (precision /
/// physical representation -- e.g. catching a price written as a 4-byte
/// f32 when the registry says 8-byte scaled integer). This is the real
/// ingest-time enforcement point for `template_engine::AttrTypeRegistry`.
pub fn validate_with_types(
    template: &Template,
    eav: &[EavTriple],
    attr_types: &AttrTypeRegistry,
) -> ValidationResult {
    let mut result = validate(template, eav);
    for triple in eav {
        if let Err(violation) = attr_types.validate_value(triple.attr_hash, &triple.value) {
            result.type_violations.push(violation);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use template_engine::{FieldSpec, Template};

    fn make_template() -> Template {
        Template::default_template(
            "t.test",
            "Test",
            &[
                FieldSpec::new(0x1A4B, "state", true),
                FieldSpec::new(0x2C5E, "quality", false),
            ],
        )
    }

    #[test]
    fn valid_when_all_required_present() {
        let t = make_template();
        let eav = vec![EavTriple::new(0x1A4B, b"1".to_vec())];
        let res = validate(&t, &eav);
        assert!(res.is_valid());
        assert_eq!(res.unknown_attrs, 0);
    }

    #[test]
    fn invalid_when_required_missing() {
        let t = make_template();
        let eav = vec![EavTriple::new(0x2C5E, b"0.9".to_vec())];
        let res = validate(&t, &eav);
        assert!(!res.is_valid());
        assert_eq!(res.missing_required, vec![0x1A4B]);
    }

    #[test]
    fn unknown_attr_counted() {
        let t = make_template();
        let eav = vec![
            EavTriple::new(0x1A4B, b"1".to_vec()),
            EavTriple::new(0xBEEF, b"extra".to_vec()),
        ];
        let res = validate(&t, &eav);
        assert!(res.is_valid());
        assert_eq!(res.unknown_attrs, 1);
    }

    use template_engine::{AttrType, AttrTypeRegistry, AttrTypeSpec};

    const PRICE_USD: u32 = 0x1A4B;

    fn price_template() -> Template {
        Template::default_template(
            "t.price",
            "Price",
            &[FieldSpec::new(PRICE_USD, "price_usd", true)],
        )
    }

    fn price_registry() -> AttrTypeRegistry {
        let mut r = AttrTypeRegistry::new();
        assert!(r.register(AttrTypeSpec::new(
            PRICE_USD,
            "price_usd",
            AttrType::FixedPointScaledInteger { scale: 2 },
            1
        )));
        r
    }

    #[test]
    fn validate_with_types_passes_correctly_shaped_value() {
        let t = price_template();
        let r = price_registry();
        let cents: i64 = 1999;
        let eav = vec![EavTriple::new(PRICE_USD, cents.to_be_bytes().to_vec())];
        let res = validate_with_types(&t, &eav, &r);
        assert!(res.is_valid());
        assert!(res.type_violations.is_empty());
    }

    #[test]
    fn validate_with_types_catches_wrong_byte_length_and_fails() {
        let t = price_template();
        let r = price_registry();
        let wrong: f32 = 19.99; // 4 bytes, registry expects 8
        let eav = vec![EavTriple::new(PRICE_USD, wrong.to_be_bytes().to_vec())];
        let res = validate_with_types(&t, &eav, &r);
        assert!(!res.is_valid());
        assert_eq!(res.type_violations.len(), 1);
    }

    #[test]
    fn validate_with_types_unregistered_attribute_does_not_fail_alone() {
        let t = make_template(); // fields not present in an empty AttrTypeRegistry
        let r = AttrTypeRegistry::new();
        let eav = vec![EavTriple::new(0x1A4B, b"1".to_vec())];
        let res = validate_with_types(&t, &eav, &r);
        assert!(
            res.is_valid(),
            "an unregistered AttrType alone must not fail validation"
        );
        assert_eq!(res.type_violations.len(), 1);
    }
}
