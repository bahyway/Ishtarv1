//! Template — named EAV field layout for a particle class (§6.1).
//!
//! Templates are compile-time constants (Default templates) or heap-allocated
//! at runtime (Stakeholder templates). They never modify stored data.

/// Describes how one EAV attribute should be labelled and whether it is
/// mandatory for a valid particle of this template's class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSpec {
    /// Attribute hash constant (matches `story_engine::projection::ATTR_*`).
    pub attr_hash: u32,
    /// Human-readable label used by DubSar IDE / HeptaScript.
    pub label:     &'static str,
    /// If true, absence of this field is flagged as a data quality issue.
    pub required:  bool,
}

impl FieldSpec {
    pub const fn new(attr_hash: u32, label: &'static str, required: bool) -> Self {
        FieldSpec { attr_hash, label, required }
    }
}

/// ILKUM — Sumerian "mandatory service obligation."
/// In EnkiDB every `FieldSpec { required: true }` is an ILKUM field:
/// its absence triggers a data-quality violation.
pub const ILKUM_DOC: &str = "mandatory DAMA field (FieldSpec required=true)";

/// SHU-GUR — "returning / extension."
/// Every `FieldSpec { required: false }` is a SHU-GUR field:
/// stakeholder extensions that enrich but do not mandate compliance.
pub const SHU_GUR_DOC: &str = "stakeholder extension field (FieldSpec required=false)";

/// Lifecycle state of a `Template` or governance law within the E-DUB-BA library.
///
/// Tracks the PARZU quad (.akk + .hepta + .way + .tmpl) through its review cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LawStatus {
    /// Template has been authored but not yet peer-reviewed.
    Draft,
    /// Template passed formal review (Lean 4 proof obligation satisfied).
    Verified,
    /// Template is in production — all particle classes are bound to it.
    Active,
    /// Template has been superseded or withdrawn; no new particles may reference it.
    Revoked,
}

/// A `Template` that carries a governance lifecycle status.
pub trait VerifiableLaw {
    fn law_status(&self) -> LawStatus;
    fn is_enforceable(&self) -> bool {
        self.law_status() == LawStatus::Active || self.law_status() == LawStatus::Verified
    }
}

/// Origin of a template — governs who may modify it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateOrigin {
    /// Compiled into the engine — immutable by design.
    Default,
    /// Created by a tribe operator — stored as a particle in tribe.templates.
    Stakeholder,
}

/// A named EAV schema describing the expected fields for a particle class.
#[derive(Debug, Clone)]
pub struct Template {
    /// Unique short name (ASCII, ≤64 chars).
    pub name:    &'static str,
    /// Free-text description of the particle class this template covers.
    pub desc:    &'static str,
    pub origin:  TemplateOrigin,
    pub fields:  Vec<FieldSpec>,
}

impl Template {
    /// Construct a Default template from a static field slice.
    pub fn default_template(
        name:   &'static str,
        desc:   &'static str,
        fields: &[FieldSpec],
    ) -> Self {
        Template {
            name,
            desc,
            origin: TemplateOrigin::Default,
            fields: fields.to_vec(),
        }
    }

    /// Construct a Stakeholder template at runtime.
    pub fn stakeholder(name: &'static str, desc: &'static str, fields: Vec<FieldSpec>) -> Self {
        Template { name, desc, origin: TemplateOrigin::Stakeholder, fields }
    }

    /// Returns every FieldSpec whose `required` flag is set.
    pub fn required_fields(&self) -> impl Iterator<Item = &FieldSpec> {
        self.fields.iter().filter(|f| f.required)
    }

    /// Returns `true` if the given attr_hash is declared in this template.
    pub fn contains_attr(&self, attr_hash: u32) -> bool {
        self.fields.iter().any(|f| f.attr_hash == attr_hash)
    }
}

/// Validates that a set of present attr_hashes satisfies all required fields.
///
/// Returns the list of missing required attr_hashes (empty = valid).
pub fn validate_required(template: &Template, present: &[u32]) -> Vec<u32> {
    template.required_fields()
        .filter(|f| !present.contains(&f.attr_hash))
        .map(|f| f.attr_hash)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const F_STATE:   FieldSpec = FieldSpec::new(0x1A4B, "state",   true);
    const F_QUALITY: FieldSpec = FieldSpec::new(0x2C5E, "quality", false);

    #[test]
    fn required_fields_filter() {
        let t = Template::default_template(
            "particle.basic", "Basic particle class",
            &[F_STATE, F_QUALITY],
        );
        let req: Vec<_> = t.required_fields().collect();
        assert_eq!(req.len(), 1);
        assert_eq!(req[0].attr_hash, 0x1A4B);
    }

    #[test]
    fn validate_missing_required() {
        let t = Template::default_template(
            "particle.basic", "Basic",
            &[F_STATE, F_QUALITY],
        );
        // state is missing — should be flagged
        let missing = validate_required(&t, &[0x2C5E]);
        assert_eq!(missing, vec![0x1A4B]);
    }

    #[test]
    fn validate_all_present() {
        let t = Template::default_template(
            "particle.basic", "Basic",
            &[F_STATE, F_QUALITY],
        );
        let missing = validate_required(&t, &[0x1A4B, 0x2C5E]);
        assert!(missing.is_empty());
    }

    #[test]
    fn contains_attr() {
        let t = Template::default_template(
            "particle.basic", "Basic",
            &[F_STATE],
        );
        assert!(t.contains_attr(0x1A4B));
        assert!(!t.contains_attr(0x9999));
    }

    #[test]
    fn stakeholder_origin() {
        let t = Template::stakeholder(
            "custom.class",
            "Custom",
            vec![FieldSpec::new(0xAAAA, "custom_field", true)],
        );
        assert_eq!(t.origin, TemplateOrigin::Stakeholder);
    }
}
