//! The 6 sovereign data quality dimensions.
//!
//! Every record scored by the DQM engine receives one score per dimension
//! in the range [0.0, 1.0].  1.0 = perfect.  0.0 = total failure.
//!
//! These 6 dimensions are the industry-standard DAMA-DMBOK quality model,
//! mapped onto BahyWay's sovereign types and scoring conventions.

/// The 6 universal data quality dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DqmDimension {
    /// All required attributes are populated — no null/empty mandatory fields.
    Completeness,
    /// Values conform to declared formats, types, and business rules.
    Validity,
    /// Values match the trusted golden-record source (GEM particle comparison).
    Accuracy,
    /// Values are internally consistent across all fields and systems.
    Consistency,
    /// No duplicate records exist for this logical entity.
    Uniqueness,
    /// Data was available within the SLA freshness window (epoch latency).
    Timeliness,
}

impl DqmDimension {
    pub const ALL: [DqmDimension; 6] = [
        DqmDimension::Completeness,
        DqmDimension::Validity,
        DqmDimension::Accuracy,
        DqmDimension::Consistency,
        DqmDimension::Uniqueness,
        DqmDimension::Timeliness,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DqmDimension::Completeness => "Completeness",
            DqmDimension::Validity     => "Validity",
            DqmDimension::Accuracy     => "Accuracy",
            DqmDimension::Consistency  => "Consistency",
            DqmDimension::Uniqueness   => "Uniqueness",
            DqmDimension::Timeliness   => "Timeliness",
        }
    }

    /// Short 4-character code used in compact reports and dashboard badges.
    pub fn code(self) -> &'static str {
        match self {
            DqmDimension::Completeness => "COMP",
            DqmDimension::Validity     => "VALD",
            DqmDimension::Accuracy     => "ACCY",
            DqmDimension::Consistency  => "CONS",
            DqmDimension::Uniqueness   => "UNIQ",
            DqmDimension::Timeliness   => "TIME",
        }
    }
}

/// Per-dimension score in [0.0, 1.0].
/// Produced by the DQM engine for one record and one dimension.
#[derive(Debug, Clone, Copy)]
pub struct DimensionScore {
    pub dimension: DqmDimension,
    /// Quality score: 1.0 = perfect, 0.0 = total failure.
    pub score:     f32,
    /// B11 equivalent: `(score × 240.0).round() as u8` — matches ADR-001.
    pub b11:       u8,
}

impl DimensionScore {
    pub fn new(dimension: DqmDimension, score: f32) -> Self {
        let clamped = score.clamp(0.0, 1.0);
        let b11 = (clamped * 240.0_f32).round() as u8;
        DimensionScore { dimension, score: clamped, b11 }
    }

    /// True when this dimension score meets its SLA threshold.
    pub fn meets_sla(&self, sla: &DqmSla) -> bool {
        self.score >= sla.threshold_for(self.dimension)
    }
}

/// SLA thresholds — one per dimension.
/// Default thresholds match the DAMA-DMBOK enterprise baseline.
#[derive(Debug, Clone)]
pub struct DqmSla {
    pub completeness: f32,  // default 0.98 — nearly all required fields present
    pub validity:     f32,  // default 0.95 — most values pass rules
    pub accuracy:     f32,  // default 0.90 — high fidelity to golden source
    pub consistency:  f32,  // default 0.95 — cross-field agreement
    pub uniqueness:   f32,  // default 0.99 — near-zero duplicate rate
    pub timeliness:   f32,  // default 0.90 — within freshness window
}

impl DqmSla {
    /// DAMA-DMBOK enterprise baseline thresholds.
    pub fn enterprise() -> Self {
        DqmSla {
            completeness: 0.98,
            validity:     0.95,
            accuracy:     0.90,
            consistency:  0.95,
            uniqueness:   0.99,
            timeliness:   0.90,
        }
    }

    /// Relaxed thresholds for legacy / exploratory sources.
    pub fn exploratory() -> Self {
        DqmSla {
            completeness: 0.80,
            validity:     0.75,
            accuracy:     0.70,
            consistency:  0.75,
            uniqueness:   0.90,
            timeliness:   0.70,
        }
    }

    /// Strict thresholds for master data / golden records.
    pub fn master_data() -> Self {
        DqmSla {
            completeness: 1.00,
            validity:     0.99,
            accuracy:     0.99,
            consistency:  0.99,
            uniqueness:   1.00,
            timeliness:   0.95,
        }
    }

    pub fn threshold_for(&self, dim: DqmDimension) -> f32 {
        match dim {
            DqmDimension::Completeness => self.completeness,
            DqmDimension::Validity     => self.validity,
            DqmDimension::Accuracy     => self.accuracy,
            DqmDimension::Consistency  => self.consistency,
            DqmDimension::Uniqueness   => self.uniqueness,
            DqmDimension::Timeliness   => self.timeliness,
        }
    }

    pub fn set(&mut self, dim: DqmDimension, threshold: f32) {
        let t = threshold.clamp(0.0, 1.0);
        match dim {
            DqmDimension::Completeness => self.completeness = t,
            DqmDimension::Validity     => self.validity     = t,
            DqmDimension::Accuracy     => self.accuracy     = t,
            DqmDimension::Consistency  => self.consistency  = t,
            DqmDimension::Uniqueness   => self.uniqueness   = t,
            DqmDimension::Timeliness   => self.timeliness   = t,
        }
    }
}

impl Default for DqmSla {
    fn default() -> Self { Self::enterprise() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_dimensions_have_unique_labels() {
        let labels: Vec<_> = DqmDimension::ALL.iter().map(|d| d.label()).collect();
        let unique: std::collections::HashSet<_> = labels.iter().collect();
        assert_eq!(unique.len(), 6);
    }

    #[test]
    fn dimension_score_b11_matches_adr001() {
        let s = DimensionScore::new(DqmDimension::Completeness, 1.0);
        assert_eq!(s.b11, 240);
        let s = DimensionScore::new(DqmDimension::Validity, 0.0);
        assert_eq!(s.b11, 0);
    }

    #[test]
    fn dimension_score_clamps_above_1() {
        let s = DimensionScore::new(DqmDimension::Accuracy, 1.5);
        assert_eq!(s.score, 1.0);
        assert_eq!(s.b11, 240);
    }

    #[test]
    fn sla_meets_threshold() {
        let sla = DqmSla::enterprise();
        let pass = DimensionScore::new(DqmDimension::Completeness, 0.99);
        let fail = DimensionScore::new(DqmDimension::Completeness, 0.97);
        assert!(pass.meets_sla(&sla));
        assert!(!fail.meets_sla(&sla));
    }

    #[test]
    fn sla_set_updates_threshold() {
        let mut sla = DqmSla::enterprise();
        sla.set(DqmDimension::Timeliness, 0.50);
        assert!((sla.timeliness - 0.50).abs() < f32::EPSILON);
    }

    #[test]
    fn master_data_sla_stricter_than_enterprise() {
        let e = DqmSla::enterprise();
        let m = DqmSla::master_data();
        assert!(m.completeness >= e.completeness);
        assert!(m.uniqueness   >= e.uniqueness);
    }
}
