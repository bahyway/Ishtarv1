//! Enlil Spectral Defender — Shamash Gate health-spectrum anomaly detection.
//!
//! The Shamash Gate monitors the Spectral Radius (|λ|) of each Tribe's B11
//! health distribution. When the observed spectrum deviates from the golden
//! baseline by more than `GAP_SIGMA_THRESHOLD`, a stewardship gap is raised.
//!
//! This is the implementation stub for the "detect_gap" responsibility documented
//! in the Enlil Algebra (TOP-JNF) Shamash Gate (State Exclusion).

#![forbid(unsafe_code)]

/// How many standard deviations of drift trigger a `SpectralGap`.
pub const GAP_SIGMA_THRESHOLD: f64 = 2.0;

/// Quality divisor (ADR-001) — must match `bahyway_core::grid::QUALITY_DIVISOR`.
const QUALITY_DIVISOR: f64 = 240.0;

/// Result of a spectral gap analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum GapDetectionResult {
    /// The observed spectrum is within acceptable range of the baseline.
    Healthy,
    /// The observed mean health has drifted beyond `GAP_SIGMA_THRESHOLD` sigma
    /// below the golden baseline.
    SpectralGap {
        /// Absolute drop in mean health score (0.0–1.0 units).
        delta: f64,
        /// Number of σ from the baseline mean.
        sigma: f64,
    },
    /// No particles in the observed slice — Shamash cannot judge an empty cell.
    InsufficientData,
}

impl GapDetectionResult {
    pub fn is_healthy(&self) -> bool {
        matches!(self, GapDetectionResult::Healthy)
    }
}

/// Compare the observed B11 health spectrum of a cell against a golden baseline.
///
/// Both slices contain raw B11 bytes (0–240 range; ADR-001).
/// Returns `InsufficientData` if either slice is empty.
pub fn detect_gap(observed: &[u8], baseline: &[u8]) -> GapDetectionResult {
    if observed.is_empty() || baseline.is_empty() {
        return GapDetectionResult::InsufficientData;
    }
    let obs_mean = mean_health(observed);
    let base_mean = mean_health(baseline);
    let base_sigma = std_dev(baseline, base_mean);

    // Avoid division by zero when baseline has zero variance (all identical).
    let effective_sigma = if base_sigma < f64::EPSILON {
        1.0 / QUALITY_DIVISOR
    } else {
        base_sigma
    };

    let delta = base_mean - obs_mean; // positive = observed is lower
    let sigma_distance = delta / effective_sigma;

    if sigma_distance > GAP_SIGMA_THRESHOLD {
        GapDetectionResult::SpectralGap {
            delta,
            sigma: sigma_distance,
        }
    } else {
        GapDetectionResult::Healthy
    }
}

fn mean_health(b11: &[u8]) -> f64 {
    b11.iter().map(|&v| v as f64 / QUALITY_DIVISOR).sum::<f64>() / b11.len() as f64
}

fn std_dev(b11: &[u8], mean: f64) -> f64 {
    let variance = b11
        .iter()
        .map(|&v| {
            let diff = (v as f64 / QUALITY_DIVISOR) - mean;
            diff * diff
        })
        .sum::<f64>()
        / b11.len() as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_when_spectra_match() {
        let baseline = [200u8, 210, 195, 205];
        let observed = [198u8, 212, 193, 207];
        assert_eq!(
            detect_gap(&observed, &baseline),
            GapDetectionResult::Healthy
        );
    }

    #[test]
    fn test_gap_when_observed_much_lower() {
        // baseline: all 200 → mean = 200/240 ≈ 0.833, σ ≈ 0
        // observed: all 60  → mean = 60/240 = 0.25
        // delta = 0.583, sigma >> 2.0
        let baseline = [200u8; 10];
        let observed = [60u8; 10];
        let result = detect_gap(&observed, &baseline);
        assert!(matches!(result, GapDetectionResult::SpectralGap { .. }));
        if let GapDetectionResult::SpectralGap { delta, sigma } = result {
            assert!(delta > 0.5);
            assert!(sigma > GAP_SIGMA_THRESHOLD);
        }
    }

    #[test]
    fn test_insufficient_data_on_empty_observed() {
        let baseline = [200u8; 5];
        assert_eq!(
            detect_gap(&[], &baseline),
            GapDetectionResult::InsufficientData
        );
    }

    #[test]
    fn test_insufficient_data_on_empty_baseline() {
        let observed = [180u8; 5];
        assert_eq!(
            detect_gap(&observed, &[]),
            GapDetectionResult::InsufficientData
        );
    }

    #[test]
    fn test_healthy_when_observed_above_baseline() {
        // Observed is better than baseline — not a gap
        let baseline = [100u8; 5];
        let observed = [200u8; 5];
        assert_eq!(
            detect_gap(&observed, &baseline),
            GapDetectionResult::Healthy
        );
    }
}
