// ============================================================================
// compare-tribe-schema/src/dead_verdict.rs
// DeadVerdict — outcome when CompareEngine finds a SchemaMismatch.
//
// All particles in the mismatched file are declared Dead.
// A SchemaMismatchAlert is routed to DataStewardEngine for SLA scheduling.
// ============================================================================
#![allow(missing_docs)]

use crate::schema_comparator::CompareReport;

/// Severity classification for the DataSteward routing decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertPriority {
    /// Breaking changes (type removed, type changed, mandatory upgraded) — immediate review.
    High,
    /// Soft changes only (position, length, added fields) — scheduled review.
    Normal,
}

/// Alert raised by CompareEngine and routed to DataStewardEngine.
#[derive(Debug, Clone)]
pub struct SchemaMismatchAlert {
    pub file_name: String,
    pub v1_version: u32,
    pub v2_version: u32,
    pub breaking_count: usize,
    pub soft_count: usize,
    pub total_fields_affected: usize,
    pub priority: AlertPriority,
    /// Human-readable message for the DataSteward dashboard.
    pub message: String,
}

impl SchemaMismatchAlert {
    pub fn from_report(report: &CompareReport) -> Self {
        let priority = if report.breaking_count > 0 {
            AlertPriority::High
        } else {
            AlertPriority::Normal
        };
        let message = format!(
            "[{}] Schema mismatch in '{}' (V{} → V{}): \
             {} breaking + {} soft changes across {} field(s). \
             {} particles marked Dead. DataSteward review required.",
            if priority == AlertPriority::High {
                "HIGH"
            } else {
                "NORMAL"
            },
            report.file_name,
            report.v1_version,
            report.v2_version,
            report.breaking_count,
            report.soft_count,
            report.field_diffs.len(),
            // particle_count filled in by DeadVerdict
            0,
        );
        SchemaMismatchAlert {
            file_name: report.file_name.clone(),
            v1_version: report.v1_version,
            v2_version: report.v2_version,
            breaking_count: report.breaking_count,
            soft_count: report.soft_count,
            total_fields_affected: report.field_diffs.len(),
            priority,
            message,
        }
    }

    pub fn is_high_priority(&self) -> bool {
        self.priority == AlertPriority::High
    }
}

/// The complete verdict when a file fails schema validation.
///
/// The caller (EtlPipeline / CompareStation) must:
///   1. Mark all `particle_count` particles with state = Dead.
///   2. Surface `alert` to the DataStewardEngine dashboard.
///   3. Skip loading the file into any staging tribe.
#[derive(Debug, Clone)]
pub struct DeadVerdict {
    pub file_name: String,
    /// Number of data rows (particles) in the rejected file.
    pub particle_count: usize,
    pub report: CompareReport,
    pub alert: SchemaMismatchAlert,
}

impl DeadVerdict {
    pub fn new(particle_count: usize, report: CompareReport) -> Self {
        let mut alert = SchemaMismatchAlert::from_report(&report);
        // Patch the particle count into the message now that we know it.
        alert.message = alert
            .message
            .replace(" 0 particles", &format!(" {particle_count} particles"));
        DeadVerdict {
            file_name: report.file_name.clone(),
            particle_count,
            report,
            alert,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_comparator::compare_versions;
    use crate::schema_version::{FieldMeta, FieldType, SchemaVersion};

    fn mismatch_report() -> CompareReport {
        let v1 = SchemaVersion::new(
            1,
            "civil.registry",
            1000,
            vec![
                FieldMeta::new("national_id", FieldType::Integer, Some(12), 1, true, false),
                FieldMeta::new("full_name", FieldType::Text, Some(200), 2, true, false),
            ],
        );
        // V2 removes "full_name" — breaking
        let v2 = SchemaVersion::new(
            2,
            "civil.registry",
            2000,
            vec![FieldMeta::new(
                "national_id",
                FieldType::Integer,
                Some(12),
                1,
                true,
                false,
            )],
        );
        compare_versions(&v1, &v2, "najaf_2020", 5000)
    }

    #[test]
    fn dead_verdict_is_high_priority_on_breaking() {
        let report = mismatch_report();
        let verdict = DeadVerdict::new(42, report);
        assert!(verdict.alert.is_high_priority());
        assert_eq!(verdict.particle_count, 42);
    }

    #[test]
    fn dead_verdict_message_contains_particle_count() {
        let report = mismatch_report();
        let verdict = DeadVerdict::new(99, report);
        assert!(
            verdict.alert.message.contains("99 particles"),
            "{}",
            verdict.alert.message
        );
    }

    #[test]
    fn normal_priority_for_soft_only() {
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
                FieldMeta::new("col", FieldType::Text, None, 2, false, true), // position only
            ],
        );
        let report = compare_versions(&v1, &v2, "file", 1);
        let verdict = DeadVerdict::new(10, report);
        assert!(!verdict.alert.is_high_priority());
        assert_eq!(verdict.alert.priority, AlertPriority::Normal);
    }

    #[test]
    fn alert_breaking_count_matches_report() {
        let report = mismatch_report();
        let expected_breaks = report.breaking_count;
        let verdict = DeadVerdict::new(5, report);
        assert_eq!(verdict.alert.breaking_count, expected_breaks);
    }
}
