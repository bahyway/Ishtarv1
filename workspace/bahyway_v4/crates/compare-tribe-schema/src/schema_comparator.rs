// ============================================================================
// compare-tribe-schema/src/schema_comparator.rs
// Core engine: compare V1 → V2 and produce a CompareReport.
//
// Rules:
//   - Any change at all → SchemaMismatch (strict enforcement)
//   - SchemaMatch       → staging_tribe_name = "{file_name}_{arrival_epoch}"
//   - SchemaMismatch    → staging_tribe_name = None; caller marks particles Dead
// ============================================================================
#![allow(missing_docs)]

use crate::field_diff::{FieldChange, FieldDiff};
use crate::schema_version::{FieldMeta, SchemaVersion};

/// Whether the two schema versions are compatible for data loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareVerdict {
    /// No field changes detected — safe to load into staging tribe.
    SchemaMatch,
    /// At least one field change found — data must NOT be loaded; all particles → Dead.
    SchemaMismatch,
}

/// Full comparison report produced by `compare_versions`.
#[derive(Debug, Clone)]
pub struct CompareReport {
    pub verdict: CompareVerdict,
    pub v1_version: u32,
    pub v2_version: u32,
    pub file_name: String,
    pub arrival_epoch: u32,
    /// Per-field diffs, sorted by field name for determinism.
    pub field_diffs: Vec<FieldDiff>,
    /// Breaking change count (Removed, TypeChanged, Mandatory false→true).
    pub breaking_count: usize,
    /// Soft change count (position, length, nullable, Added).
    pub soft_count: usize,
    /// Populated only when verdict == SchemaMatch.
    pub staging_tribe_name: Option<String>,
}

impl CompareReport {
    pub fn is_match(&self) -> bool {
        self.verdict == CompareVerdict::SchemaMatch
    }

    pub fn has_breaking_changes(&self) -> bool {
        self.breaking_count > 0
    }

    /// One-line human-readable summary for logs and alerts.
    pub fn summary(&self) -> String {
        match self.verdict {
            CompareVerdict::SchemaMatch => format!(
                "[MATCH] '{}' V{}→V{} — schemas identical → staging: '{}'",
                self.file_name,
                self.v1_version,
                self.v2_version,
                self.staging_tribe_name.as_deref().unwrap_or("N/A")
            ),
            CompareVerdict::SchemaMismatch => format!(
                "[MISMATCH] '{}' V{}→V{} — {} breaking + {} soft changes in {} field(s)",
                self.file_name,
                self.v1_version,
                self.v2_version,
                self.breaking_count,
                self.soft_count,
                self.field_diffs.len()
            ),
        }
    }
}

/// Compare `v1` against `v2`.
///
/// - `file_name`: base name of the file being ingested (no extension, no path).
/// - `arrival_epoch`: journal epoch of the file's arrival in the LandingZone.
///
/// Returns a `CompareReport`.  On `SchemaMismatch` the caller must call
/// `DeadVerdict::new(...)` to build the alert and mark particles Dead.
pub fn compare_versions(
    v1: &SchemaVersion,
    v2: &SchemaVersion,
    file_name: &str,
    arrival_epoch: u32,
) -> CompareReport {
    let mut diffs: Vec<FieldDiff> = Vec::new();

    // Pass 1: everything in V1 — detect Removed and in-place changes.
    for f1 in &v1.fields {
        match v2.field_by_name(&f1.name) {
            None => diffs.push(FieldDiff {
                field_name: f1.name.clone(),
                attr_hash: f1.attr_hash,
                changes: vec![FieldChange::Removed],
            }),
            Some(f2) => {
                let changes = diff_field(f1, f2);
                if !changes.is_empty() {
                    diffs.push(FieldDiff {
                        field_name: f1.name.clone(),
                        attr_hash: f1.attr_hash,
                        changes,
                    });
                }
            }
        }
    }

    // Pass 2: fields in V2 that didn't exist in V1 → Added.
    for f2 in &v2.fields {
        if v1.field_by_name(&f2.name).is_none() {
            diffs.push(FieldDiff {
                field_name: f2.name.clone(),
                attr_hash: f2.attr_hash,
                changes: vec![FieldChange::Added],
            });
        }
    }

    // Deterministic output order.
    diffs.sort_by(|a, b| a.field_name.cmp(&b.field_name));

    let breaking_count = diffs.iter().filter(|d| d.has_breaking_change()).count();
    let soft_count = diffs.len() - breaking_count;

    let verdict = if diffs.is_empty() {
        CompareVerdict::SchemaMatch
    } else {
        CompareVerdict::SchemaMismatch
    };

    let staging_tribe_name = match verdict {
        CompareVerdict::SchemaMatch => Some(format!("{}_{}", file_name, arrival_epoch)),
        CompareVerdict::SchemaMismatch => None,
    };

    CompareReport {
        verdict,
        v1_version: v1.version,
        v2_version: v2.version,
        file_name: file_name.to_string(),
        arrival_epoch,
        field_diffs: diffs,
        breaking_count,
        soft_count,
        staging_tribe_name,
    }
}

fn diff_field(f1: &FieldMeta, f2: &FieldMeta) -> Vec<FieldChange> {
    let mut changes = Vec::new();
    if f1.data_type != f2.data_type {
        changes.push(FieldChange::TypeChanged {
            from: f1.data_type,
            to: f2.data_type,
        });
    }
    if f1.max_length != f2.max_length {
        changes.push(FieldChange::LengthChanged {
            from: f1.max_length,
            to: f2.max_length,
        });
    }
    if f1.position != f2.position {
        changes.push(FieldChange::PositionChanged {
            from: f1.position,
            to: f2.position,
        });
    }
    if f1.mandatory != f2.mandatory {
        changes.push(FieldChange::MandatoryChanged {
            was: f1.mandatory,
            now: f2.mandatory,
        });
    }
    if f1.nullable != f2.nullable {
        changes.push(FieldChange::NullableChanged {
            was: f1.nullable,
            now: f2.nullable,
        });
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_version::{FieldMeta, FieldType, SchemaVersion};

    fn make_v1() -> SchemaVersion {
        SchemaVersion::new(
            1,
            "civil.registry",
            1000,
            vec![
                FieldMeta::new("national_id", FieldType::Integer, Some(12), 1, true, false),
                FieldMeta::new("full_name", FieldType::Text, Some(200), 2, true, false),
                FieldMeta::new("birth_date", FieldType::Date, None, 3, true, false),
                FieldMeta::new("governorate", FieldType::Text, Some(64), 4, false, true),
            ],
        )
    }

    #[test]
    fn identical_schemas_produce_match() {
        let v1 = make_v1();
        let v2 = make_v1();
        let r = compare_versions(&v1, &v2, "najaf_2020", 5000);
        assert_eq!(r.verdict, CompareVerdict::SchemaMatch);
        assert!(r.field_diffs.is_empty());
        assert_eq!(r.staging_tribe_name.as_deref(), Some("najaf_2020_5000"));
    }

    #[test]
    fn removed_field_produces_mismatch() {
        let v1 = make_v1();
        // V2 is missing "birth_date"
        let v2 = SchemaVersion::new(
            2,
            "civil.registry",
            2000,
            vec![
                FieldMeta::new("national_id", FieldType::Integer, Some(12), 1, true, false),
                FieldMeta::new("full_name", FieldType::Text, Some(200), 2, true, false),
                FieldMeta::new("governorate", FieldType::Text, Some(64), 3, false, true),
            ],
        );
        let r = compare_versions(&v1, &v2, "test", 1);
        assert_eq!(r.verdict, CompareVerdict::SchemaMismatch);
        assert!(r.breaking_count > 0);
        assert!(r.staging_tribe_name.is_none());
        let birth_diff = r
            .field_diffs
            .iter()
            .find(|d| d.field_name == "birth_date")
            .unwrap();
        assert!(birth_diff.changes.contains(&FieldChange::Removed));
    }

    #[test]
    fn type_change_is_breaking_mismatch() {
        let v1 = make_v1();
        let v2 = SchemaVersion::new(
            2,
            "civil.registry",
            2000,
            vec![
                FieldMeta::new("national_id", FieldType::Text, Some(12), 1, true, false), // was Integer
                FieldMeta::new("full_name", FieldType::Text, Some(200), 2, true, false),
                FieldMeta::new("birth_date", FieldType::Date, None, 3, true, false),
                FieldMeta::new("governorate", FieldType::Text, Some(64), 4, false, true),
            ],
        );
        let r = compare_versions(&v1, &v2, "file", 1);
        assert_eq!(r.verdict, CompareVerdict::SchemaMismatch);
        assert!(r.has_breaking_changes());
    }

    #[test]
    fn position_change_is_soft_mismatch() {
        let v1 = make_v1();
        let v2 = SchemaVersion::new(
            2,
            "civil.registry",
            2000,
            vec![
                FieldMeta::new("national_id", FieldType::Integer, Some(12), 2, true, false), // pos 1→2
                FieldMeta::new("full_name", FieldType::Text, Some(200), 1, true, false), // pos 2→1
                FieldMeta::new("birth_date", FieldType::Date, None, 3, true, false),
                FieldMeta::new("governorate", FieldType::Text, Some(64), 4, false, true),
            ],
        );
        let r = compare_versions(&v1, &v2, "file", 1);
        assert_eq!(r.verdict, CompareVerdict::SchemaMismatch);
        assert!(
            !r.has_breaking_changes(),
            "position-only changes must be soft"
        );
        assert!(r.soft_count > 0);
    }

    #[test]
    fn added_field_is_soft_mismatch() {
        let mut v2 = make_v1();
        v2.version = 2;
        v2.fields.push(FieldMeta::new(
            "religion",
            FieldType::Text,
            Some(30),
            5,
            false,
            true,
        ));
        let r = compare_versions(&make_v1(), &v2, "file", 1);
        assert_eq!(r.verdict, CompareVerdict::SchemaMismatch);
        assert!(!r.has_breaking_changes());
        assert!(r.field_diffs.iter().any(|d| d.field_name == "religion"));
    }

    #[test]
    fn mandatory_upgrade_false_to_true_is_breaking() {
        let v1 = SchemaVersion::new(
            1,
            "t",
            1000,
            vec![FieldMeta::new("col", FieldType::Text, None, 1, false, true)],
        );
        let v2 = SchemaVersion::new(
            2,
            "t",
            2000,
            vec![
                FieldMeta::new("col", FieldType::Text, None, 1, true, false), // mandatory now
            ],
        );
        let r = compare_versions(&v1, &v2, "f", 1);
        assert!(r.has_breaking_changes());
    }

    #[test]
    fn staging_name_format_is_correct() {
        let v1 = make_v1();
        let r = compare_versions(&v1, &v1.clone(), "najaf_decades", 9999);
        assert_eq!(r.staging_tribe_name.as_deref(), Some("najaf_decades_9999"));
    }

    #[test]
    fn summary_contains_verdict_keyword() {
        let v1 = make_v1();
        let match_r = compare_versions(&v1, &v1.clone(), "f", 1);
        let mismatch_r = compare_versions(&v1, &SchemaVersion::new(2, "t", 1, vec![]), "f", 1);
        assert!(match_r.summary().contains("MATCH"));
        assert!(mismatch_r.summary().contains("MISMATCH"));
    }

    #[test]
    fn diff_output_is_sorted_by_field_name() {
        let v1 = SchemaVersion::new(
            1,
            "t",
            1,
            vec![
                FieldMeta::new("zzz", FieldType::Text, None, 1, true, false),
                FieldMeta::new("aaa", FieldType::Text, None, 2, true, false),
            ],
        );
        let v2 = SchemaVersion::new(
            2,
            "t",
            2,
            vec![
                FieldMeta::new("zzz", FieldType::Integer, None, 1, true, false), // changed
            ],
        ); // "aaa" removed
        let r = compare_versions(&v1, &v2, "f", 1);
        assert_eq!(r.field_diffs[0].field_name, "aaa");
        assert_eq!(r.field_diffs[1].field_name, "zzz");
    }
}
