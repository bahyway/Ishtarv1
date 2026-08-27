//! DqmReport — the per-record quality verdict with all 6 dimension scores,
//! SLA compliance flags, and the composite DQM quality score.

use crate::dimensions::{DimensionScore, DqmDimension, DqmSla};

/// Full data quality assessment for one record.
#[derive(Debug, Clone)]
pub struct DqmReport {
    /// Scores for all 6 dimensions.
    pub scores: [DimensionScore; 6],
    /// SLA configuration used for this assessment.
    pub sla: DqmSla,
    /// Composite DQM score = weighted mean of all 6 dimension scores.
    pub composite: f32,
    /// B11 equivalent of composite: `(composite × 240).round() as u8`.
    pub composite_b11: u8,
    /// True when ALL 6 dimensions meet their SLA thresholds.
    pub sla_compliant: bool,
}

impl DqmReport {
    /// Build a report from 6 raw dimension scores and an SLA config.
    pub fn new(scores: [DimensionScore; 6], sla: DqmSla) -> Self {
        let composite = scores.iter().map(|s| s.score).sum::<f32>() / 6.0;
        let composite_b11 = (composite * 240.0).round() as u8;
        let sla_compliant = scores.iter().all(|s| s.meets_sla(&sla));
        DqmReport {
            scores,
            sla,
            composite,
            composite_b11,
            sla_compliant,
        }
    }

    /// Score for a specific dimension.
    pub fn score_for(&self, dim: DqmDimension) -> f32 {
        self.scores
            .iter()
            .find(|s| s.dimension == dim)
            .map(|s| s.score)
            .unwrap_or(0.0)
    }

    /// Dimensions that failed their SLA threshold.
    pub fn failing_dimensions(&self) -> Vec<DqmDimension> {
        self.scores
            .iter()
            .filter(|s| !s.meets_sla(&self.sla))
            .map(|s| s.dimension)
            .collect()
    }

    /// Dimensions that passed their SLA threshold.
    pub fn passing_dimensions(&self) -> Vec<DqmDimension> {
        self.scores
            .iter()
            .filter(|s| s.meets_sla(&self.sla))
            .map(|s| s.dimension)
            .collect()
    }

    /// One-line summary for logging and dashboard tooltips.
    pub fn summary(&self) -> String {
        let status = if self.sla_compliant { "PASS" } else { "FAIL" };
        let failing = self.failing_dimensions();
        if failing.is_empty() {
            format!(
                "[DQM {status}] composite={:.2} B11={}",
                self.composite, self.composite_b11
            )
        } else {
            let names: Vec<&str> = failing.iter().map(|d| d.label()).collect();
            format!(
                "[DQM {status}] composite={:.2} B11={} | failing: {}",
                self.composite,
                self.composite_b11,
                names.join(", ")
            )
        }
    }
}

/// Aggregated DQM report over a batch of records.
#[derive(Debug, Clone, Default)]
pub struct DqmBatchReport {
    pub total_records: usize,
    pub sla_compliant: usize,
    pub sla_violations: usize,
    /// Per-dimension mean score across all records.
    pub mean_scores: [f32; 6],
    /// Per-dimension SLA pass rate (0.0–1.0).
    pub sla_pass_rates: [f32; 6],
    pub mean_composite: f32,
}

impl DqmBatchReport {
    pub fn from_reports(reports: &[DqmReport]) -> Self {
        if reports.is_empty() {
            return DqmBatchReport::default();
        }

        let n = reports.len();
        let compliant = reports.iter().filter(|r| r.sla_compliant).count();
        let mut dim_sums = [0.0f32; 6];
        let mut dim_pass = [0usize; 6];

        for report in reports {
            for (i, score) in report.scores.iter().enumerate() {
                dim_sums[i] += score.score;
                if score.meets_sla(&report.sla) {
                    dim_pass[i] += 1;
                }
            }
        }

        let mut mean_scores = [0.0f32; 6];
        let mut sla_pass_rates = [0.0f32; 6];
        for i in 0..6 {
            mean_scores[i] = dim_sums[i] / n as f32;
            sla_pass_rates[i] = dim_pass[i] as f32 / n as f32;
        }
        let mean_composite = mean_scores.iter().sum::<f32>() / 6.0;

        DqmBatchReport {
            total_records: n,
            sla_compliant: compliant,
            sla_violations: n - compliant,
            mean_scores,
            sla_pass_rates,
            mean_composite,
        }
    }

    /// SLA compliance rate for the batch (0.0–1.0).
    pub fn compliance_rate(&self) -> f32 {
        if self.total_records == 0 {
            return 1.0;
        }
        self.sla_compliant as f32 / self.total_records as f32
    }

    /// Label for dimension at index `i` (index = DqmDimension::ALL order).
    pub fn dimension_label(i: usize) -> &'static str {
        DqmDimension::ALL
            .get(i)
            .map(|d| d.label())
            .unwrap_or("Unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dimensions::DqmSla;

    fn score(dim: DqmDimension, v: f32) -> DimensionScore {
        DimensionScore::new(dim, v)
    }

    fn perfect_report() -> DqmReport {
        DqmReport::new(
            [
                score(DqmDimension::Completeness, 1.0),
                score(DqmDimension::Validity, 1.0),
                score(DqmDimension::Accuracy, 1.0),
                score(DqmDimension::Consistency, 1.0),
                score(DqmDimension::Uniqueness, 1.0),
                score(DqmDimension::Timeliness, 1.0),
            ],
            DqmSla::enterprise(),
        )
    }

    #[test]
    fn perfect_report_is_sla_compliant() {
        assert!(perfect_report().sla_compliant);
    }

    #[test]
    fn perfect_composite_is_one() {
        let r = perfect_report();
        assert!((r.composite - 1.0).abs() < f32::EPSILON);
        assert_eq!(r.composite_b11, 240);
    }

    #[test]
    fn failing_dimension_detected() {
        let mut r = perfect_report();
        // Override completeness to fail SLA (< 0.98)
        r.scores[0] = score(DqmDimension::Completeness, 0.90);
        r.sla_compliant = r.scores.iter().all(|s| s.meets_sla(&r.sla));
        let failing = r.failing_dimensions();
        assert!(failing.contains(&DqmDimension::Completeness));
    }

    #[test]
    fn summary_contains_pass_for_perfect() {
        assert!(perfect_report().summary().contains("PASS"));
    }

    #[test]
    fn batch_report_compliance_rate_one_for_all_pass() {
        let reports = vec![perfect_report(), perfect_report(), perfect_report()];
        let batch = DqmBatchReport::from_reports(&reports);
        assert!((batch.compliance_rate() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn batch_report_totals_correct() {
        let r1 = perfect_report();
        let mut r2 = perfect_report();
        r2.scores[0] = score(DqmDimension::Completeness, 0.50);
        r2.sla_compliant = false;
        let batch = DqmBatchReport::from_reports(&[r1, r2]);
        assert_eq!(batch.total_records, 2);
        assert_eq!(batch.sla_compliant, 1);
        assert_eq!(batch.sla_violations, 1);
    }

    #[test]
    fn empty_batch_compliance_rate_one() {
        let batch = DqmBatchReport::from_reports(&[]);
        assert!((batch.compliance_rate() - 1.0).abs() < f32::EPSILON);
    }
}
