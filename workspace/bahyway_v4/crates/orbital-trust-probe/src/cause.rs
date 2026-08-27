//! DeviationCause — causal attribution for orbital movement.
//!
//! Before a trust penalty is issued the probe must determine WHY a particle
//! moved. Only `Unexplained` deviations become sovereign audit events.
//! All other causes are observability context — logged, never penalised.

/// Causal classification for an orbital deviation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviationCause {
    /// Fuzzy rule set changed between observations — deviation is structurally
    /// expected. Abort the probe; do not log or penalise.
    FuzzyRulesChanged,

    /// StoryEngine replay shows EAV events arrived between T_expected and
    /// T_observed. The particle's data legitimately evolved; orbit followed.
    LegitimateStateEvolution,

    /// SPH neighbourhood density changed (neighbour joined / left the
    /// resonance_radius=0.15 shell). Particle drifted due to field pressure,
    /// not internal corruption.
    NeighborDensityShift,

    /// BLUE byte (freshness) decayed past a ring-boundary threshold. This is
    /// a natural time function — not a trust signal.
    FreshnessDecay,

    /// B11 sits within the hysteresis band around a ring boundary (±HYSTERESIS
    /// units). Discrete ring assignment oscillates; treat as scoring noise.
    ThresholdBoundaryNoise,

    /// No causal explanation found. Orbit changed while rules, score pipeline,
    /// neighbour field, and freshness are all stable. Emit AuditJournal event.
    Unexplained,
}

impl DeviationCause {
    /// Returns true when the probe must halt without issuing a trust penalty.
    pub fn is_abort(self) -> bool {
        self == Self::FuzzyRulesChanged
    }

    /// Returns true when the deviation is genuine and requires journaling.
    pub fn requires_audit(self) -> bool {
        self == Self::Unexplained
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::FuzzyRulesChanged => "FUZZY_RULES_CHANGED",
            Self::LegitimateStateEvolution => "LEGITIMATE_STATE_EVOLUTION",
            Self::NeighborDensityShift => "NEIGHBOR_DENSITY_SHIFT",
            Self::FreshnessDecay => "FRESHNESS_DECAY",
            Self::ThresholdBoundaryNoise => "THRESHOLD_BOUNDARY_NOISE",
            Self::Unexplained => "UNEXPLAINED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_fuzzy_rules_changed_is_abort() {
        assert!(DeviationCause::FuzzyRulesChanged.is_abort());
        assert!(!DeviationCause::Unexplained.is_abort());
        assert!(!DeviationCause::NeighborDensityShift.is_abort());
    }

    #[test]
    fn only_unexplained_requires_audit() {
        assert!(DeviationCause::Unexplained.requires_audit());
        assert!(!DeviationCause::LegitimateStateEvolution.requires_audit());
        assert!(!DeviationCause::FreshnessDecay.requires_audit());
    }

    #[test]
    fn labels_are_non_empty() {
        let causes = [
            DeviationCause::FuzzyRulesChanged,
            DeviationCause::LegitimateStateEvolution,
            DeviationCause::NeighborDensityShift,
            DeviationCause::FreshnessDecay,
            DeviationCause::ThresholdBoundaryNoise,
            DeviationCause::Unexplained,
        ];
        for c in causes {
            assert!(!c.label().is_empty());
        }
    }
}
