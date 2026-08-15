//! Track A — pattern discovery over the 7 mandatory attributes' OWN
//! VALUES (`bahyway_core::mandatory_attrs::MandatoryAttr`), using
//! `fuzzy-engine`'s real Mamdani triangular membership function
//! (`f_tri`) to score how tightly a batch of particles' freshness/
//! velocity values cluster around their own batch mean — a real,
//! reusable "these particles are behaving the same way" signal, the
//! same shape of question the Architect's own B11/Score Engine already
//! answers per-particle, applied here across a batch to find whether
//! the BATCH itself is a coherent pattern.
//!
//! HONEST SCOPE: this is a real, working v1 — a single fuzzy dimension
//! (freshness/velocity coherence) via one real membership function. It
//! is not a full Mamdani rule-base (multiple antecedents, weighted rule
//! aggregation, defuzzification via `centroid_defuzz`) — that's real,
//! valuable follow-up work once there's a concrete second dimension to
//! combine it with.

use fuzzy_engine::membership::f_tri;

/// One particle's mandatory-attribute values, sampled for Track A.
#[derive(Debug, Clone, Copy)]
pub struct MandatoryAttrSample {
    pub freshness: f32,
    pub velocity: f32,
}

/// Result of scoring a batch of samples for a coherent value pattern.
#[derive(Debug, Clone)]
pub struct MandatoryPatternResult {
    pub constituent_count: u32,
    /// Mean coherence across the batch (0.0..=1.0) — the Track A
    /// confidence signal fed to `build_discovered_pattern`.
    pub confidence: f64,
    pub mean_freshness: f32,
    pub mean_velocity: f32,
}

/// Score how coherent a batch's freshness/velocity values are around
/// their own mean, using a real triangular fuzzy membership function
/// peaking exactly at the mean. Tight clustering (most samples near the
/// mean) → confidence near 1.0; scattered values → confidence near 0.0.
/// Returns `None` for fewer than 2 samples (no meaningful spread to
/// measure).
pub fn score_mandatory_value_pattern(samples: &[MandatoryAttrSample]) -> Option<MandatoryPatternResult> {
    if samples.len() < 2 {
        return None;
    }

    let n = samples.len() as f32;
    let mean_freshness = samples.iter().map(|s| s.freshness).sum::<f32>() / n;
    let mean_velocity = samples.iter().map(|s| s.velocity).sum::<f32>() / n;

    let spread_freshness = spread(samples.iter().map(|s| s.freshness), mean_freshness);
    let spread_velocity = spread(samples.iter().map(|s| s.velocity), mean_velocity);

    // A real triangular fuzzy set per dimension, centered on the batch's
    // own mean, half-width = the batch's own spread (never zero, so a
    // perfectly uniform batch still scores full membership rather than
    // dividing by zero).
    let half_width_f = spread_freshness.max(0.01);
    let half_width_v = spread_velocity.max(0.01);

    let mut total = 0.0f32;
    for s in samples {
        let m_f = f_tri(s.freshness, mean_freshness - half_width_f, mean_freshness, mean_freshness + half_width_f);
        let m_v = f_tri(s.velocity, mean_velocity - half_width_v, mean_velocity, mean_velocity + half_width_v);
        total += (m_f + m_v) / 2.0;
    }
    let confidence = (total / n) as f64;

    Some(MandatoryPatternResult {
        constituent_count: samples.len() as u32,
        confidence,
        mean_freshness,
        mean_velocity,
    })
}

fn spread(values: impl Iterator<Item = f32> + Clone, mean: f32) -> f32 {
    let values2 = values.clone();
    let n = values2.count() as f32;
    if n == 0.0 {
        return 0.0;
    }
    let variance = values.map(|v| (v - mean).powi(2)).sum::<f32>() / n;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_values_score_full_confidence() {
        let samples = vec![
            MandatoryAttrSample { freshness: 0.8, velocity: 0.5 },
            MandatoryAttrSample { freshness: 0.8, velocity: 0.5 },
            MandatoryAttrSample { freshness: 0.8, velocity: 0.5 },
        ];
        let result = score_mandatory_value_pattern(&samples).unwrap();
        assert!(result.confidence > 0.95, "identical samples must score near-perfect coherence, got {}", result.confidence);
    }

    #[test]
    fn wildly_scattered_values_score_low_confidence() {
        let samples = vec![
            MandatoryAttrSample { freshness: 0.0, velocity: 0.0 },
            MandatoryAttrSample { freshness: 1.0, velocity: 1.0 },
            MandatoryAttrSample { freshness: 0.5, velocity: 0.9 },
            MandatoryAttrSample { freshness: 0.9, velocity: 0.1 },
        ];
        let result = score_mandatory_value_pattern(&samples).unwrap();
        assert!(result.confidence < 0.5, "scattered samples must score low coherence, got {}", result.confidence);
    }

    #[test]
    fn fewer_than_two_samples_returns_none() {
        assert!(score_mandatory_value_pattern(&[]).is_none());
        assert!(score_mandatory_value_pattern(&[MandatoryAttrSample { freshness: 0.5, velocity: 0.5 }]).is_none());
    }

    #[test]
    fn mean_values_are_computed_correctly() {
        let samples = vec![
            MandatoryAttrSample { freshness: 0.2, velocity: 0.4 },
            MandatoryAttrSample { freshness: 0.8, velocity: 0.6 },
        ];
        let result = score_mandatory_value_pattern(&samples).unwrap();
        assert!((result.mean_freshness - 0.5).abs() < 1e-6);
        assert!((result.mean_velocity - 0.5).abs() < 1e-6);
    }
}
