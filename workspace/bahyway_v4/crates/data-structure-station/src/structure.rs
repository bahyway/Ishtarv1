//! StructureStation — maps raw key-value pairs to typed EAV triples (§10.4).
//!
//! The station uses the template's FieldSpec list to order and label fields.
//! Unknown attrs (not in template) are passed through as-is per the
//! "open EAV" principle — the template is advisory, not a hard schema.

use enkidb_journal::entry::EavTriple;
use template_engine::Template;

/// Structuring report.
pub struct StructureReport {
    pub eav: Vec<EavTriple>,
    /// Number of attrs mapped via template field spec.
    pub matched_fields: usize,
    /// Number of attrs not found in the template (still included in eav).
    pub extra_fields: usize,
}

/// Structure raw (attr_hash, value) pairs into EAV triples, annotated
/// against the template.
pub fn structure(template: &Template, raw: Vec<(u32, Vec<u8>)>) -> StructureReport {
    let mut matched = 0usize;
    let mut extra = 0usize;
    let eav: Vec<EavTriple> = raw
        .into_iter()
        .map(|(attr, val)| {
            if template.contains_attr(attr) {
                matched += 1;
            } else {
                extra += 1;
            }
            EavTriple::new(attr, val)
        })
        .collect();
    StructureReport {
        eav,
        matched_fields: matched,
        extra_fields: extra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use template_engine::{FieldSpec, Template};

    fn make_template() -> Template {
        Template::default_template("t.test", "Test", &[FieldSpec::new(0x1A4B, "state", true)])
    }

    #[test]
    fn known_attr_counted_as_matched() {
        let t = make_template();
        let raw = vec![(0x1A4B, b"1".to_vec())];
        let r = structure(&t, raw);
        assert_eq!(r.matched_fields, 1);
        assert_eq!(r.extra_fields, 0);
        assert_eq!(r.eav.len(), 1);
    }

    #[test]
    fn unknown_attr_counted_as_extra() {
        let t = make_template();
        let raw = vec![(0xBEEF, b"extra".to_vec())];
        let r = structure(&t, raw);
        assert_eq!(r.matched_fields, 0);
        assert_eq!(r.extra_fields, 1);
        assert_eq!(r.eav[0].attr_hash, 0xBEEF);
    }

    #[test]
    fn mixed_attrs() {
        let t = make_template();
        let raw = vec![(0x1A4B, b"1".to_vec()), (0xBEEF, b"x".to_vec())];
        let r = structure(&t, raw);
        assert_eq!(r.matched_fields, 1);
        assert_eq!(r.extra_fields, 1);
    }
}
