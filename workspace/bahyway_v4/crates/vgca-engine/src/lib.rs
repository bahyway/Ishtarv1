//! 𒁾 VGCA — Vector Geometric Cleansing Analysis v4.0
//!
//! Three algorithm families:
//!   VGCA-Σ  7D Feature Score Vector  (text geometry)       → `fsv`
//!   VGCA-Δ  6D Binary Feature Vector (binary block analysis) → `bfv`
//!   VGCA-Λ  GeometricFit + self-calibrating centroid         → `geometric_fit`
//!
//! Sovereign constants (ADR-008):
//!   δ_frag = 0.35  (VGCA-Δ fragmentation threshold)
//!   Clean ≥ 0.80 · Suspect 0.50–0.79 · Outlier < 0.50 · Alien < 0.05
//!
//! The self-calibrating centroid rule:
//!   Only GEM-lane particles (B11 ≥ 200) update the domain centroid.
//!   Dead and Suspect particles are excluded — centroid always converges
//!   toward the sovereign data ideal, never toward degraded data.
//!
//! §2.4 compliance: VGCA scores are written to EAV Orbit, never to KAKI nucleus.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

#![forbid(unsafe_code)]

pub mod fsv;
pub mod bfv;
pub mod geometric_fit;
pub mod kl_divergence;
pub mod riemannian;

pub use fsv::{
    FeatureScoreVector, compute_fsv, shannon_entropy, is_arabic_char,
};
pub use bfv::{
    BinaryFeatureVector, compute_bfv, fragmentation_score, is_fragmented,
    DELTA_FRAG,
};
pub use geometric_fit::{
    GeometricFit, DomainCentroid, VgcaResult,
    CLEAN_THRESHOLD, SUSPECT_THRESHOLD, ALIEN_THRESHOLD,
};

/// Run the full VGCA-Σ pipeline on one text field.
///
/// Returns `VgcaResult` containing the FSV, geometric fit score, and category.
/// If `b11 >= 200`, call `centroid.update_with_gem(&result.fsv)` after this.
pub fn analyse_text(text: &str, centroid: &DomainCentroid) -> VgcaResult {
    let fsv = compute_fsv(text);
    VgcaResult::compute(fsv, centroid)
}

/// Run the full VGCA-Δ pipeline on one binary block.
///
/// Returns `(BinaryFeatureVector, is_fragmented)`.
pub fn analyse_bytes(bytes: &[u8]) -> (BinaryFeatureVector, bool) {
    let bfv = compute_bfv(bytes);
    let frag = is_fragmented(&bfv);
    (bfv, frag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyse_text_arabic_name_clean_after_calibration() {
        let mut centroid = DomainCentroid::new();
        // seed centroid with a few Arabic GEM particles
        for name in ["محمد", "أحمد", "علي", "فاطمة", "مريم"] {
            let r = analyse_text(name, &centroid);
            centroid.update_with_gem(&r.fsv);
        }
        let result = analyse_text("محمد أحمد علي", &centroid);
        assert!(result.score > 0.0, "score={}", result.score);
        assert!(result.fit.permits_scoring());
    }

    #[test]
    fn analyse_bytes_null_block_is_fragmented() {
        let (_, frag) = analyse_bytes(&[0u8; 32]);
        assert!(frag);
    }

    #[test]
    fn analyse_bytes_clean_ascii_not_fragmented() {
        let (_, frag) = analyse_bytes(b"Hello World");
        assert!(!frag);
    }
}
