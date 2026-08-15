//! SchemaContract — the typed boundary enforced at every source and target.
//!
//! A contract declares the fields a connector must supply (or accept) and their
//! types.  Violations are surfaced as structured `FabricException` events before
//! any data crosses the boundary, not as silent failures downstream.

/// Primitive field types accepted by the Fabric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Text,
    Integer,
    Decimal,
    Boolean,
    Timestamp,
    Bytes,
    Nullable(Box<FieldType>),
}

/// A single field declaration inside a SchemaContract.
#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub name:      &'static str,
    pub attr_hash: u32,
    pub field_type: FieldType,
    pub required:  bool,
}

impl FieldSpec {
    pub const fn required(name: &'static str, attr_hash: u32, field_type: FieldType) -> Self {
        FieldSpec { name, attr_hash, field_type, required: true }
    }

    pub const fn optional(name: &'static str, attr_hash: u32, field_type: FieldType) -> Self {
        FieldSpec { name, attr_hash, field_type, required: false }
    }
}

/// The typed contract that every connector must declare.
///
/// Connectors that produce data declare what they emit.
/// Connectors that consume data declare what they expect.
/// The Fabric enforces alignment before data moves.
#[derive(Debug, Clone)]
pub struct SchemaContract {
    pub name:    &'static str,
    pub version: u16,
    pub fields:  Vec<FieldSpec>,
}

impl SchemaContract {
    pub fn new(name: &'static str, version: u16, fields: Vec<FieldSpec>) -> Self {
        SchemaContract { name, version, fields }
    }

    /// Returns all required field hashes — used for validation.
    pub fn required_hashes(&self) -> Vec<u32> {
        self.fields.iter().filter(|f| f.required).map(|f| f.attr_hash).collect()
    }

    /// Check that a set of attr_hashes satisfies the required fields.
    pub fn validate_presence(&self, present: &[u32]) -> Result<(), Vec<&'static str>> {
        let missing: Vec<&'static str> = self.fields
            .iter()
            .filter(|f| f.required && !present.contains(&f.attr_hash))
            .map(|f| f.name)
            .collect();
        if missing.is_empty() { Ok(()) } else { Err(missing) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_contract() -> SchemaContract {
        SchemaContract::new("erp.invoice", 1, vec![
            FieldSpec::required("invoice_id",  0x1001, FieldType::Integer),
            FieldSpec::required("amount",      0x1002, FieldType::Decimal),
            FieldSpec::optional("description", 0x1003, FieldType::Text),
        ])
    }

    #[test]
    fn validation_passes_when_required_fields_present() {
        let c = sample_contract();
        assert!(c.validate_presence(&[0x1001, 0x1002, 0x1003]).is_ok());
    }

    #[test]
    fn validation_passes_with_only_required_fields() {
        let c = sample_contract();
        assert!(c.validate_presence(&[0x1001, 0x1002]).is_ok());
    }

    #[test]
    fn validation_fails_when_required_field_missing() {
        let c = sample_contract();
        let err = c.validate_presence(&[0x1001]).unwrap_err();
        assert!(err.contains(&"amount"));
    }

    #[test]
    fn required_hashes_excludes_optional() {
        let c = sample_contract();
        let hashes = c.required_hashes();
        assert!(hashes.contains(&0x1001));
        assert!(hashes.contains(&0x1002));
        assert!(!hashes.contains(&0x1003));
    }
}
