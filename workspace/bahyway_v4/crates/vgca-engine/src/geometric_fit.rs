//! VGCA GeometricFit — centroid-based quality classification.
//!
//! The self-calibrating centroid drifts ONLY toward GEM-lane particles.
//! Dead and Suspect particles NEVER update the centroid.
//! This guarantees the centroid always converges to the sovereign data ideal
//! for the domain, not toward degraded data.
//!
//! GeometricFit score = 1 / (1 + distance_to_centroid)
//!
//! Thresholds (ADR-008, sovereign):
//!   Clean   ≥ 0.80
//!   Suspect  0.50–0.79
//!   Outlier < 0.50
//!   Alien   < 0.05  (pure noise — unrecognisable as domain data)
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::fsv::FeatureScoreVector;

// ── Sovereign Thresholds ──────────────────────────────────────────────────────

/// GeometricFit ≥ 0.80 → Clean (close to domain centroid).
pub const CLEAN_THRESHOLD: f64 = 0.80;
/// GeometricFit 0.50–0.79 → Suspect.
pub const SUSPECT_THRESHOLD: f64 = 0.50;
/// GeometricFit < 0.05 → Alien (zero resemblance to domain).
pub const ALIEN_THRESHOLD: f64 = 0.05;

// ── GeometricFit category ────────────────────────────────────────────────────

/// Classification of a record based on its geometric distance to the domain centroid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometricFit {
    /// ≥ 0.80 — record is geometrically close to the domain ideal.
    Clean,
    /// 0.50–0.79 — record is plausible but requires scoring and possible steward review.
    Suspect,
    /// 0.05–0.49 — record is far from the centroid; VGCA flags for steward.
    Outlier,
    /// < 0.05 — record bears no resemblance to the domain; pre-classified as noise.
    Alien,
}

impl GeometricFit {
    /// Classify a raw geometric fit score.
    pub fn from_score(score: f64) -> Self {
        if      score >= CLEAN_THRESHOLD   { Self::Clean   }
        else if score >= SUSPECT_THRESHOLD { Self::Suspect }
        else if score >= ALIEN_THRESHOLD   { Self::Outlier }
        else                               { Self::Alien   }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Clean   => "CLEAN",
            Self::Suspect => "SUSPECT",
            Self::Outlier => "OUTLIER",
            Self::Alien   => "ALIEN",
        }
    }

    /// Returns `true` if this classification allows HeptaScore to proceed.
    ///
    /// Alien records do not enter H(P) scoring — they go directly to the dead grid.
    pub fn permits_scoring(self) -> bool {
        !matches!(self, Self::Alien)
    }
}

// ── Self-Calibrating Centroid ─────────────────────────────────────────────────

/// The domain centroid — the running mean of GEM-only particles.
///
/// **Rule**: only GEM-lane particles (B11 ≥ 200) may update this centroid.
/// Dead, Fuzzy, and Suspect particles are excluded.
///
/// This makes the centroid self-calibrating: it always converges toward
/// the sovereign data ideal for the domain, never toward degraded data.
#[derive(Debug, Clone)]
pub struct DomainCentroid {
    dims:      [f64; 7],
    /// Total weight used in the running average (seed counts as 1 prior).
    avg_weight: u64,
    /// Number of actual GEM-lane particles that have updated this centroid.
    gem_count:  u64,
}

impl DomainCentroid {
    /// Start from the all-ones ideal point (perfect quality in every dimension).
    pub fn new() -> Self {
        Self { dims: [1.0; 7], avg_weight: 0, gem_count: 0 }
    }

    /// Start from an explicit seed centroid (domain-specific prior).
    ///
    /// The seed contributes weight=1 to the running average but is NOT counted
    /// as a GEM calibration — `gem_count()` only tracks real GEM updates.
    pub fn from_seed(seed: [f64; 7]) -> Self {
        Self { dims: seed, avg_weight: 1, gem_count: 0 }
    }

    /// Update the centroid using a GEM-lane particle's FSV.
    ///
    /// **Must only be called when B11 ≥ 200.**
    /// Calling with non-GEM particles violates the self-calibrating invariant.
    pub fn update_with_gem(&mut self, fsv: &FeatureScoreVector) {
        let n = self.avg_weight as f64;
        let new_dims = fsv.as_array();
        for i in 0..7 {
            self.dims[i] = (self.dims[i] * n + new_dims[i]) / (n + 1.0);
        }
        self.avg_weight += 1;
        self.gem_count  += 1;
    }

    /// Compute the geometric fit score of an FSV against this centroid.
    ///
    /// `score = 1 / (1 + ||fsv − centroid||₂)`
    ///
    /// Returns 1.0 if no GEM particles have updated the centroid yet
    /// (no calibration → accept all, conservative).
    pub fn geometric_fit_score(&self, fsv: &FeatureScoreVector) -> f64 {
        if self.gem_count == 0 {
            return 1.0; // uncalibrated → accept all
        }
        let fsv_dims = fsv.as_array();
        let dist: f64 = self.dims.iter()
            .zip(fsv_dims.iter())
            .map(|(c, f)| (c - f).powi(2))
            .sum::<f64>()
            .sqrt();
        1.0 / (1.0 + dist)
    }

    /// Classify an FSV against the current centroid.
    pub fn classify(&self, fsv: &FeatureScoreVector) -> GeometricFit {
        GeometricFit::from_score(self.geometric_fit_score(fsv))
    }

    /// Number of GEM particles that have calibrated this centroid.
    pub fn gem_count(&self) -> u64 { self.gem_count }

    /// Current centroid coordinates (read-only).
    pub fn dims(&self) -> &[f64; 7] { &self.dims }
}

impl Default for DomainCentroid {
    fn default() -> Self { Self::new() }
}

// ── Full VGCA result ─────────────────────────────────────────────────────────

/// Complete result of running VGCA-Σ on one text field.
#[derive(Debug, Clone)]
pub struct VgcaResult {
    pub fsv:     FeatureScoreVector,
    pub score:   f64,
    pub fit:     GeometricFit,
    /// EAV attributes to write to Orbit (7 float dimensions + 1 fit category).
    pub attrs_count: usize,
}

impl VgcaResult {
    pub fn compute(fsv: FeatureScoreVector, centroid: &DomainCentroid) -> Self {
        let score = centroid.geometric_fit_score(&fsv);
        let fit   = GeometricFit::from_score(score);
        Self { fsv, score, fit, attrs_count: 8 }
    }

    pub fn is_clean(&self) -> bool { self.fit == GeometricFit::Clean }
    pub fn is_alien(&self) -> bool { self.fit == GeometricFit::Alien }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsv::compute_fsv;

    #[test]
    fn uncalibrated_centroid_accepts_all() {
        let c = DomainCentroid::new();
        let fsv = compute_fsv("any text");
        let score = c.geometric_fit_score(&fsv);
        assert_eq!(score, 1.0);
        assert_eq!(c.classify(&fsv), GeometricFit::Clean);
    }

    #[test]
    fn centroid_only_moves_toward_gem() {
        let mut c = DomainCentroid::from_seed([0.5; 7]);
        let arabic_name = compute_fsv("محمد أحمد علي");
        c.update_with_gem(&arabic_name);
        assert_eq!(c.gem_count(), 1);
        // centroid shifted toward arabic_name
        let dims = c.dims();
        assert!(dims[2] > 0.5, "centroid D3 (arabic_density) should increase");
    }

    #[test]
    fn perfect_fsv_scores_one_against_perfect_centroid() {
        let c = DomainCentroid::from_seed([1.0; 7]);
        let fsv = FeatureScoreVector::perfect();
        let score = c.geometric_fit_score(&fsv);
        assert!((score - 1.0).abs() < 1e-9, "score={score}");
    }

    #[test]
    fn zero_fsv_against_perfect_centroid_gives_low_score() {
        let mut c = DomainCentroid::new();
        // calibrate with perfect particles
        for _ in 0..10 {
            c.update_with_gem(&FeatureScoreVector::perfect());
        }
        let score = c.geometric_fit_score(&FeatureScoreVector::zero());
        assert!(score < SUSPECT_THRESHOLD, "zero FSV against perfect centroid: score={score}");
    }

    #[test]
    fn geometric_fit_thresholds_correct() {
        assert_eq!(GeometricFit::from_score(1.00), GeometricFit::Clean);
        assert_eq!(GeometricFit::from_score(0.80), GeometricFit::Clean);
        assert_eq!(GeometricFit::from_score(0.79), GeometricFit::Suspect);
        assert_eq!(GeometricFit::from_score(0.50), GeometricFit::Suspect);
        assert_eq!(GeometricFit::from_score(0.49), GeometricFit::Outlier);
        assert_eq!(GeometricFit::from_score(0.05), GeometricFit::Outlier);
        assert_eq!(GeometricFit::from_score(0.04), GeometricFit::Alien);
        assert_eq!(GeometricFit::from_score(0.00), GeometricFit::Alien);
    }

    #[test]
    fn alien_does_not_permit_scoring() {
        assert!(!GeometricFit::Alien.permits_scoring());
    }

    #[test]
    fn clean_suspect_outlier_permit_scoring() {
        assert!(GeometricFit::Clean.permits_scoring());
        assert!(GeometricFit::Suspect.permits_scoring());
        assert!(GeometricFit::Outlier.permits_scoring());
    }

    #[test]
    fn vgca_result_computes_correctly() {
        let c = DomainCentroid::new(); // uncalibrated
        let fsv = compute_fsv("محمد");
        let r = VgcaResult::compute(fsv, &c);
        assert_eq!(r.attrs_count, 8);
        assert!(r.score > 0.0 && r.score <= 1.0);
    }

    #[test]
    fn centroid_accumulates_multiple_gems() {
        let mut c = DomainCentroid::new();
        for _ in 0..5 {
            c.update_with_gem(&compute_fsv("محمد أحمد"));
        }
        assert_eq!(c.gem_count(), 5);
    }

    #[test]
    fn geometric_fit_labels_non_empty() {
        for fit in [GeometricFit::Clean, GeometricFit::Suspect,
                    GeometricFit::Outlier, GeometricFit::Alien] {
            assert!(!fit.label().is_empty());
        }
    }
}
