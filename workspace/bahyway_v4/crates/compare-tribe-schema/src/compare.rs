//! SchemaComparator — cross-tribe template field diff (§10.8, §8.5).
//!
//! Compares two templates to find matching, added, and removed fields.
//! Used by the IDU layer to identify schema evolution between tribes.

use template_engine::Template;

/// Difference report between two tribe templates.
#[derive(Debug, Clone)]
pub struct SchemaDiff {
    /// Attr hashes present in both templates.
    pub common: Vec<u32>,
    /// Attr hashes in template A but not B (template A has extra fields).
    pub only_in_a: Vec<u32>,
    /// Attr hashes in template B but not A (template B has extra fields).
    pub only_in_b: Vec<u32>,
}

impl SchemaDiff {
    /// True when both templates expose exactly the same field set.
    pub fn is_identical(&self) -> bool {
        self.only_in_a.is_empty() && self.only_in_b.is_empty()
    }

    pub fn common_count(&self) -> usize {
        self.common.len()
    }
    pub fn divergence_count(&self) -> usize {
        self.only_in_a.len() + self.only_in_b.len()
    }
}

/// Compute the field-level diff between template `a` and template `b`.
pub fn compare(a: &Template, b: &Template) -> SchemaDiff {
    let attrs_a: std::collections::HashSet<u32> = a.fields.iter().map(|f| f.attr_hash).collect();
    let attrs_b: std::collections::HashSet<u32> = b.fields.iter().map(|f| f.attr_hash).collect();

    let mut common: Vec<u32> = attrs_a.intersection(&attrs_b).copied().collect();
    let mut only_a: Vec<u32> = attrs_a.difference(&attrs_b).copied().collect();
    let mut only_b: Vec<u32> = attrs_b.difference(&attrs_a).copied().collect();

    // Sort for determinism
    common.sort();
    only_a.sort();
    only_b.sort();

    SchemaDiff {
        common,
        only_in_a: only_a,
        only_in_b: only_b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use template_engine::{FieldSpec, Template};

    fn make(fields: &[FieldSpec]) -> Template {
        Template::default_template("t", "test", fields)
    }

    #[test]
    fn identical_templates_no_divergence() {
        let fs = &[FieldSpec::new(0x1A4B, "state", true)];
        let diff = compare(&make(fs), &make(fs));
        assert!(diff.is_identical());
        assert_eq!(diff.common_count(), 1);
    }

    #[test]
    fn extra_field_in_a() {
        let a = make(&[
            FieldSpec::new(0x1A4B, "state", true),
            FieldSpec::new(0x2C5E, "quality", false),
        ]);
        let b = make(&[FieldSpec::new(0x1A4B, "state", true)]);
        let diff = compare(&a, &b);
        assert!(!diff.is_identical());
        assert_eq!(diff.only_in_a, vec![0x2C5E]);
        assert!(diff.only_in_b.is_empty());
    }

    #[test]
    fn completely_different_templates() {
        let a = make(&[FieldSpec::new(0x1111, "a", true)]);
        let b = make(&[FieldSpec::new(0x2222, "b", true)]);
        let diff = compare(&a, &b);
        assert_eq!(diff.common_count(), 0);
        assert_eq!(diff.divergence_count(), 2);
    }

    #[test]
    fn diff_is_deterministic() {
        let a = make(&[
            FieldSpec::new(0x3333, "c", false),
            FieldSpec::new(0x1111, "a", true),
        ]);
        let b = make(&[
            FieldSpec::new(0x2222, "b", false),
            FieldSpec::new(0x1111, "a", true),
        ]);
        let diff = compare(&a, &b);
        assert_eq!(diff.common, vec![0x1111]);
        assert_eq!(diff.only_in_a, vec![0x3333]);
        assert_eq!(diff.only_in_b, vec![0x2222]);
    }
}
