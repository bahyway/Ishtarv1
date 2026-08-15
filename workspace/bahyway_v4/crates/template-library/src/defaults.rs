//! Default template catalog — built-in templates compiled into the engine (§6.3).
//!
//! These templates cover the three canonical particle classes:
//!   civil_registry  — long-lived sovereign identity records
//!   operational     — short-lived operational particles
//!   sensor_stream   — high-frequency sensor / IoT particles
//!
//! All attr_hash values match `story_engine::projection::ATTR_*`.

use template_engine::{FieldSpec, Template, TemplateRegistry};

// ── Hepta attribute hashes (§7.5 — must stay in sync with story-engine) ──────
pub const ATTR_STATE:              u32 = 0x1A4B;
pub const ATTR_QUALITY:            u32 = 0x2C5E;
pub const ATTR_COLOR_RGB:          u32 = 0x3D7F;
pub const ATTR_FRESHNESS:          u32 = 0x4E89;
pub const ATTR_SNAPSHOT_DATE:      u32 = 0x9A1D;
pub const ATTR_SNAPSHOT_STATE:     u32 = 0x7E32;
pub const ATTR_SNAPSHOT_FREQUENCY: u32 = 0x4B0F;

// ── civil.registry ─────────────────────────────────────────────────────────

static CIVIL_FIELDS: &[FieldSpec] = &[
    FieldSpec::new(ATTR_STATE,              "state",              true),
    FieldSpec::new(ATTR_QUALITY,            "quality",            false),
    FieldSpec::new(ATTR_COLOR_RGB,          "color_rgb",          false),
    FieldSpec::new(ATTR_FRESHNESS,          "freshness",          false),
    FieldSpec::new(ATTR_SNAPSHOT_DATE,      "snapshot_date",      false),
    FieldSpec::new(ATTR_SNAPSHOT_STATE,     "snapshot_state",     false),
    FieldSpec::new(ATTR_SNAPSHOT_FREQUENCY, "snapshot_frequency", false),
];

pub fn civil_registry_template() -> Template {
    Template::default_template(
        "civil.registry",
        "Long-lived sovereign identity record — 1-year freshness half-life",
        CIVIL_FIELDS,
    )
}

// ── operational ────────────────────────────────────────────────────────────

static OPERATIONAL_FIELDS: &[FieldSpec] = &[
    FieldSpec::new(ATTR_STATE,   "state",   true),
    FieldSpec::new(ATTR_QUALITY, "quality", true),
    FieldSpec::new(ATTR_FRESHNESS, "freshness", true),
];

pub fn operational_template() -> Template {
    Template::default_template(
        "operational",
        "Short-lived operational particle — 30-day freshness half-life",
        OPERATIONAL_FIELDS,
    )
}

// ── sensor.stream ──────────────────────────────────────────────────────────

static SENSOR_FIELDS: &[FieldSpec] = &[
    FieldSpec::new(ATTR_STATE,     "state",     true),
    FieldSpec::new(ATTR_FRESHNESS, "freshness", true),
    FieldSpec::new(ATTR_COLOR_RGB, "color_rgb", false),
];

pub fn sensor_stream_template() -> Template {
    Template::default_template(
        "sensor.stream",
        "High-frequency sensor / IoT particle — 1-hour freshness half-life",
        SENSOR_FIELDS,
    )
}

/// Populate a TemplateRegistry with all three Default templates.
pub fn load_defaults(reg: &mut TemplateRegistry) {
    reg.register(civil_registry_template());
    reg.register(operational_template());
    reg.register(sensor_stream_template());
}

#[cfg(test)]
mod tests {
    use super::*;
    use template_engine::TemplateRegistry;

    #[test]
    fn load_defaults_registers_three_templates() {
        let mut reg = TemplateRegistry::new();
        load_defaults(&mut reg);
        assert_eq!(reg.len(), 3);
        assert!(reg.get("civil.registry").is_some());
        assert!(reg.get("operational").is_some());
        assert!(reg.get("sensor.stream").is_some());
    }

    #[test]
    fn civil_registry_has_all_hepta_attrs() {
        let t = civil_registry_template();
        for attr in [
            ATTR_STATE, ATTR_QUALITY, ATTR_COLOR_RGB,
            ATTR_FRESHNESS, ATTR_SNAPSHOT_DATE,
            ATTR_SNAPSHOT_STATE, ATTR_SNAPSHOT_FREQUENCY,
        ] {
            assert!(t.contains_attr(attr), "missing attr {attr:#010x}");
        }
    }

    #[test]
    fn operational_requires_freshness() {
        let t = operational_template();
        let req: Vec<_> = t.required_fields().collect();
        let has_freshness = req.iter().any(|f| f.attr_hash == ATTR_FRESHNESS);
        assert!(has_freshness);
    }

    #[test]
    fn sensor_requires_state_and_freshness() {
        let t = sensor_stream_template();
        let req: Vec<_> = t.required_fields().collect();
        assert!(req.iter().any(|f| f.attr_hash == ATTR_STATE));
        assert!(req.iter().any(|f| f.attr_hash == ATTR_FRESHNESS));
    }
}
