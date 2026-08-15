//! DiagnosisTemplate — structured findings emitted by the Alert Engine (§6.4).
//!
//! Each DiagnosisKind maps to a named template describing the data quality
//! condition and the recommended stewardship action.

use alert_engine::alert::AlertSeverity;

/// The nature of the data quality issue detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosisKind {
    /// Quality score (G channel) has drifted past the watch threshold.
    QualityDegraded,
    /// Freshness (B channel) has decayed past the warning threshold.
    StaleData,
    /// Particle state = Dead — record is flagged as inactive.
    DeadParticle,
    /// All three channels degraded simultaneously — critical.
    Critical,
}

/// A diagnosis record linking an alert to the template that describes it.
#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub uuid_hash:     u32,
    pub kind:          DiagnosisKind,
    pub severity:      AlertSeverity,
    pub drift:         f32,
    /// Name of the template describing this diagnosis class.
    pub template_name: &'static str,
    /// Short human-readable description of the issue.
    pub description:   &'static str,
    /// Recommended stewardship action per DAMA-DMBOK.
    pub action:        &'static str,
}

impl DiagnosisKind {
    pub fn template_name(self) -> &'static str {
        match self {
            DiagnosisKind::QualityDegraded => "diagnosis.quality_degraded",
            DiagnosisKind::StaleData       => "diagnosis.stale_data",
            DiagnosisKind::DeadParticle    => "diagnosis.dead_particle",
            DiagnosisKind::Critical        => "diagnosis.critical",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            DiagnosisKind::QualityDegraded =>
                "Quality score (HPS G-channel) has dropped below threshold.",
            DiagnosisKind::StaleData =>
                "Freshness decay (B-channel) indicates the record is outdated.",
            DiagnosisKind::DeadParticle =>
                "Particle state is Dead — record is inactive.",
            DiagnosisKind::Critical =>
                "All ColorID channels degraded — immediate stewardship required.",
        }
    }

    pub fn action(self) -> &'static str {
        match self {
            DiagnosisKind::QualityDegraded =>
                "Schedule a re-validation event and update quality score.",
            DiagnosisKind::StaleData =>
                "Trigger a freshness refresh or mark record as archived.",
            DiagnosisKind::DeadParticle =>
                "Review record history via StoryEngine and confirm deactivation.",
            DiagnosisKind::Critical =>
                "Escalate to Data Steward Station for immediate triage.",
        }
    }
}

/// Derive the DiagnosisKind from an alert's severity and drift magnitude.
pub fn classify(severity: AlertSeverity, drift: f32) -> DiagnosisKind {
    match severity {
        AlertSeverity::Critical => DiagnosisKind::Critical,
        AlertSeverity::Warning  => {
            if drift > 200.0 {
                DiagnosisKind::StaleData
            } else {
                DiagnosisKind::QualityDegraded
            }
        }
        AlertSeverity::Watch    => DiagnosisKind::QualityDegraded,
    }
}

/// Build a Diagnosis from an Alert's uuid_hash + severity + drift.
pub fn diagnose(uuid_hash: u32, severity: AlertSeverity, drift: f32) -> Diagnosis {
    let kind = classify(severity, drift);
    Diagnosis {
        uuid_hash,
        kind,
        severity,
        drift,
        template_name: kind.template_name(),
        description:   kind.description(),
        action:        kind.action(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alert_engine::alert::AlertSeverity;

    #[test]
    fn classify_critical() {
        assert_eq!(classify(AlertSeverity::Critical, 400.0), DiagnosisKind::Critical);
    }

    #[test]
    fn classify_stale_vs_quality() {
        assert_eq!(classify(AlertSeverity::Warning, 250.0), DiagnosisKind::StaleData);
        assert_eq!(classify(AlertSeverity::Warning, 100.0), DiagnosisKind::QualityDegraded);
    }

    #[test]
    fn diagnose_builds_full_record() {
        let d = diagnose(0xABCD1234, AlertSeverity::Critical, 350.0);
        assert_eq!(d.kind, DiagnosisKind::Critical);
        assert_eq!(d.template_name, "diagnosis.critical");
        assert!(!d.description.is_empty());
        assert!(!d.action.is_empty());
        assert_eq!(d.uuid_hash, 0xABCD1234);
    }

    #[test]
    fn dead_particle_kind_has_action() {
        assert!(!DiagnosisKind::DeadParticle.action().is_empty());
    }
}
