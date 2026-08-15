//! Sovereign weight profiles for the 7 quality dimensions.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::errors::HeptaError;

/// Arabic MDM weights — BahyWay sovereign default.
/// Accuracy (Arabic name correctness) weighted highest at 0.30.
pub const SOVEREIGN_ARABIC_MDM_WEIGHTS: [f32; 7] =
    [0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05];

/// Equal weights — all 7 dimensions contribute equally (1/7 each).
pub const EQUAL_WEIGHTS: [f32; 7] = [
    1.0/7.0, 1.0/7.0, 1.0/7.0, 1.0/7.0, 1.0/7.0, 1.0/7.0, 1.0/7.0,
];

/// A named weight profile for a specific domain.
#[derive(Debug, Clone)]
pub struct WeightProfile {
    pub name:    String,
    /// The 7 dimension weights — must sum to 1.0.
    pub weights: [f32; 7],
}

impl WeightProfile {
    pub fn new(name: impl Into<String>, weights: [f32; 7]) -> Result<Self, HeptaError> {
        let sum: f32 = weights.iter().sum();
        if (sum - 1.0).abs() > 0.001 {
            return Err(HeptaError::WeightsDoNotSumToOne(sum));
        }
        Ok(Self { name: name.into(), weights })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sovereign_arabic_mdm_sums_to_one() {
        let sum: f32 = SOVEREIGN_ARABIC_MDM_WEIGHTS.iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "sum={sum}");
    }

    #[test]
    fn equal_weights_sums_to_one() {
        let sum: f32 = EQUAL_WEIGHTS.iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "sum={sum}");
    }

    #[test]
    fn invalid_weights_rejected() {
        let bad = [0.5f32; 7]; // sums to 3.5
        assert!(WeightProfile::new("bad", bad).is_err());
    }

    #[test]
    fn valid_profile_accepted() {
        let w = SOVEREIGN_ARABIC_MDM_WEIGHTS;
        assert!(WeightProfile::new("arabic-mdm", w).is_ok());
    }
}
