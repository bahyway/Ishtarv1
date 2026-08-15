//! akk-loader — compile `.akk` PARTICLE declarations into `Template` objects.
//!
//! # Mapping rules
//!
//! | `.akk` field type              | `FieldSpec.required` |
//! |--------------------------------|----------------------|
//! | `KAKI`                         | `true`  — identity field, always mandatory |
//! | `TEXT(completeness=1.0)`       | `true`  — fully complete = no nulls allowed |
//! | `TEXT(…)` (any other threshold)| `false` — conditional quality, nullable |
//! | `Custom(_)`                    | `false` — domain-specific, treated as optional |
//!
//! `ATTR_STATE` (sovereign constant `0x1A4B`) is always injected as the first
//! required field regardless of what the PARTICLE declares.  Any `state` field
//! in the PARTICLE body is skipped to prevent duplicates.
//!
//! # String lifetime
//!
//! `Template::name`, `Template::desc`, and `FieldSpec::label` are `&'static str`
//! by design (compile-time templates).  Templates loaded at startup via this
//! crate use `Box::leak()` to promote heap `String`s to `&'static str`.
//! Leaked memory is bounded: one allocation per field label per template load.
//!
//! # Usage
//!
//! ```text
//! // Compile-time
//! use akk_loader::load_first_particle;
//! let tmpl = load_first_particle(include_str!("automation_task.akk")).unwrap();
//!
//! // Runtime file
//! use akk_loader::load_akk_file;
//! let tmpl = load_akk_file(Path::new("shard/profiles/my_batch.template.akk")).unwrap();
//! ```

#![forbid(unsafe_code)]

use aaol::{AkkNodeKind, FieldType, PmpvdParser, QualityConstraint};
use template_engine::{FieldSpec, Template};

/// Sovereign ATTR_STATE — matches `story_engine::projection::ATTR_STATE`.
/// Value is `crc16("state")` per ADR-sovereign-constants.
const ATTR_STATE: u32 = 0x1A4B;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AkkLoadError {
    ParseError(String),
    ParticleNotFound(String),
    IoError(String),
}

impl core::fmt::Display for AkkLoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ParseError(msg)     => write!(f, "AKK parse error: {msg}"),
            Self::ParticleNotFound(n) => write!(f, "PARTICLE '{n}' not found in .akk source"),
            Self::IoError(msg)        => write!(f, "IO error: {msg}"),
        }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse the **first** `PARTICLE` declaration in `source` into a `Template`.
///
/// ATTR_STATE is always the first required field. Any explicit `state` field in
/// the PARTICLE body is silently skipped to avoid duplication.
pub fn load_first_particle(source: &str) -> Result<Template, AkkLoadError> {
    let prog = PmpvdParser::parse_source(source)
        .map_err(AkkLoadError::ParseError)?;

    for node in &prog.nodes {
        if let AkkNodeKind::Particle(p) = &node.kind {
            return Ok(particle_to_template(p));
        }
    }
    Err(AkkLoadError::ParticleNotFound("(first)".into()))
}

/// Parse the `PARTICLE` named `particle_name` in `source` into a `Template`.
pub fn load_particle(source: &str, particle_name: &str) -> Result<Template, AkkLoadError> {
    let prog = PmpvdParser::parse_source(source)
        .map_err(AkkLoadError::ParseError)?;

    for node in &prog.nodes {
        if let AkkNodeKind::Particle(p) = &node.kind {
            if p.name == particle_name {
                return Ok(particle_to_template(p));
            }
        }
    }
    Err(AkkLoadError::ParticleNotFound(particle_name.into()))
}

/// Read a `.akk` file from disk and compile its first `PARTICLE` into a `Template`.
///
/// bee-watchdog looks for `shard/profiles/{batch_name}.template.akk` — if present
/// this function replaces `build_template_from_schema()` for that batch.
pub fn load_akk_file(path: &std::path::Path) -> Result<Template, AkkLoadError> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        AkkLoadError::IoError(format!("cannot read {}: {e}", path.display()))
    })?;
    load_first_particle(&source)
}

// ── Pipeline support ──────────────────────────────────────────────────────────

/// A compiled AKK PIPELINE declaration for bee-watchdog entry routing.
///
/// Each stage name maps to a `.template.akk` file in the profiles directory.
/// bee-watchdog matches stage names against archive entry filenames (case-insensitive
/// `contains`) to select the right template per entry type.
#[derive(Debug, Clone)]
pub struct AkkPipeline {
    pub name:    String,
    pub version: Option<String>,
    /// Ordered stage names — each maps to `{stage}.template.akk`.
    pub stages:  Vec<String>,
}

/// Parse the first `PIPELINE` declaration from `.akk` source.
pub fn load_pipeline(source: &str) -> Result<AkkPipeline, AkkLoadError> {
    let prog = PmpvdParser::parse_source(source)
        .map_err(AkkLoadError::ParseError)?;
    for node in &prog.nodes {
        if let AkkNodeKind::Pipeline(p) = &node.kind {
            return Ok(AkkPipeline {
                name:    p.name.clone(),
                version: p.version.clone(),
                stages:  p.stages.clone(),
            });
        }
    }
    Err(AkkLoadError::ParticleNotFound("PIPELINE".into()))
}

/// Read a `.akk` file and parse its first `PIPELINE` declaration.
pub fn load_pipeline_file(path: &std::path::Path) -> Result<AkkPipeline, AkkLoadError> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        AkkLoadError::IoError(format!("cannot read {}: {e}", path.display()))
    })?;
    load_pipeline(&source)
}

// ── Internals ─────────────────────────────────────────────────────────────────

fn particle_to_template(p: &aaol::ParticleDecl) -> Template {
    let mut fields = Vec::with_capacity(p.fields.len() + 1);

    // Sovereign first field: ATTR_STATE is always required.
    fields.push(FieldSpec::new(ATTR_STATE, "state", true));

    for f in &p.fields {
        if f.name.to_lowercase() == "state" {
            continue; // already injected as ATTR_STATE above
        }
        let attr_hash = fnv1a_str(&f.name);
        let required  = field_is_required(&f.field_type, &f.constraints);
        let label     = leak_string(f.name.clone());
        fields.push(FieldSpec { attr_hash, label, required });
    }

    let name = leak_string(p.name.clone());
    let desc = leak_string(format!("AKK particle template: {}", p.name));
    Template::stakeholder(name, desc, fields)
}

/// Determine if a PARTICLE field is required from its type and constraints.
///
/// - `KAKI` → always required (sovereign identity field)
/// - `TEXT(completeness=1.0)` → required (no nulls)
/// - Everything else → optional
fn field_is_required(ft: &FieldType, constraints: &[QualityConstraint]) -> bool {
    match ft {
        FieldType::Kaki   => true,
        FieldType::Text   => constraints.iter()
            .any(|c| c.dimension == "completeness" && c.threshold >= 1.0),
        FieldType::Custom(_) => false,
    }
}

/// Promote a runtime `String` to `&'static str` via heap leak.
///
/// Used for template names and field labels that must satisfy the `&'static str`
/// bound on `Template` and `FieldSpec`.  One allocation per label; bounded by
/// the number of templates loaded at startup.
fn leak_string(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// FNV-1a 32-bit hash — must match `fn fnv1a()` in bee-watchdog/main.rs.
fn fnv1a_str(s: &str) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for &b in s.as_bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_particle() {
        let src = r#"
            PARTICLE AutomationTask {
                task_id:   KAKI
                task_name: TEXT(accuracy=1.0, completeness=1.0)
                notes:     TEXT
                TRIBE: AutomationMDM
            }
        "#;
        let tmpl = load_first_particle(src).unwrap();
        // ATTR_STATE is always first
        assert_eq!(tmpl.fields[0].attr_hash, ATTR_STATE);
        assert_eq!(tmpl.fields[0].label, "state");
        assert!(tmpl.fields[0].required);
        // task_id (KAKI) → required
        let task_id = tmpl.fields.iter().find(|f| f.label == "task_id").unwrap();
        assert!(task_id.required, "KAKI field must be required");
        // task_name (TEXT completeness=1.0) → required
        let task_name = tmpl.fields.iter().find(|f| f.label == "task_name").unwrap();
        assert!(task_name.required, "completeness=1.0 must be required");
        // notes (TEXT without completeness) → optional
        let notes = tmpl.fields.iter().find(|f| f.label == "notes").unwrap();
        assert!(!notes.required, "plain TEXT must be optional");
        // total fields: state + task_id + task_name + notes = 4
        assert_eq!(tmpl.fields.len(), 4);
    }

    #[test]
    fn load_named_particle() {
        let src = r#"
            PARTICLE Other { name: TEXT TRIBE: X }
            PARTICLE Target { id: KAKI TRIBE: Y }
        "#;
        let tmpl = load_particle(src, "Target").unwrap();
        assert!(tmpl.fields.iter().any(|f| f.label == "id" && f.required));
        assert!(!tmpl.fields.iter().any(|f| f.label == "name"),
            "Other's fields must not appear in Target template");
    }

    #[test]
    fn particle_not_found_error() {
        let src = r#" PARTICLE A { x: TEXT } "#;
        let err = load_particle(src, "NonExistent");
        assert!(matches!(err, Err(AkkLoadError::ParticleNotFound(_))));
    }

    #[test]
    fn explicit_state_field_not_duplicated() {
        let src = r#"
            PARTICLE WithState {
                state: TEXT
                name:  TEXT(completeness=1.0)
            }
        "#;
        let tmpl = load_first_particle(src).unwrap();
        let state_count = tmpl.fields.iter()
            .filter(|f| f.attr_hash == ATTR_STATE)
            .count();
        assert_eq!(state_count, 1, "ATTR_STATE must appear exactly once");
        // fields: state(injected) + name = 2
        assert_eq!(tmpl.fields.len(), 2);
    }

    #[test]
    fn custom_field_type_is_optional() {
        let src = r#" PARTICLE P { code: SovereignCode } "#;
        let tmpl = load_first_particle(src).unwrap();
        let code = tmpl.fields.iter().find(|f| f.label == "code").unwrap();
        assert!(!code.required);
    }

    #[test]
    fn template_name_matches_particle_name() {
        let src = r#" PARTICLE MyEntity { x: KAKI } "#;
        let tmpl = load_first_particle(src).unwrap();
        assert_eq!(tmpl.name, "MyEntity");
        assert!(tmpl.desc.contains("MyEntity"));
    }

    #[test]
    fn parse_pipeline_decl() {
        let src = r#"
            PIPELINE AutomationPillar v4 {
                AutomationTask
                AutomationSchedule
                AutomationLog
            }
        "#;
        let pl = load_pipeline(src).unwrap();
        assert_eq!(pl.name, "AutomationPillar");
        assert_eq!(pl.version.as_deref(), Some("v4"));
        assert_eq!(pl.stages, ["AutomationTask", "AutomationSchedule", "AutomationLog"]);
    }

    #[test]
    fn pipeline_not_found_error() {
        let src = r#" PARTICLE A { x: TEXT } "#;
        assert!(matches!(load_pipeline(src), Err(AkkLoadError::ParticleNotFound(_))));
    }

    #[test]
    fn fnv1a_is_deterministic() {
        assert_eq!(fnv1a_str("task_name"), fnv1a_str("task_name"));
        assert_ne!(fnv1a_str("task_name"), fnv1a_str("task_id"));
    }

    #[test]
    fn full_automation_task_particle() {
        let src = r#"
            PARTICLE AutomationTask {
                task_id:     KAKI
                task_name:   TEXT(accuracy=1.0, completeness=1.0)
                description: TEXT
                schedule:    TEXT(completeness=1.0)
                status:      TEXT(completeness=1.0)
                owner_kaki:  KAKI
                domain:      TEXT
                TRIBE: AutomationMDM
            }
        "#;
        let tmpl = load_first_particle(src).unwrap();
        // Required: state, task_id, task_name, schedule, status, owner_kaki
        let req: Vec<_> = tmpl.required_fields().collect();
        assert_eq!(req.len(), 6, "expected 6 required fields");
        // Optional: description, domain
        let opt: Vec<_> = tmpl.fields.iter().filter(|f| !f.required).collect();
        assert_eq!(opt.len(), 2, "expected 2 optional fields");
    }
}
