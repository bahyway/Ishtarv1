//! Centroid defuzzification — Centre of Gravity method.
//!
//! Samples the clipped output MFs across 100 points and computes CoG.
//! Output range is [0, 1] matching the sovereign state output variable.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::membership::LinguisticVar;
use std::collections::HashMap;

/// Centroid (CoG) defuzzification over 100 samples.
///
/// Returns a crisp score in `output_var.range`.
/// Falls back to the midpoint of the range when no rules fire.
pub fn centroid_defuzz(clips: &HashMap<String, f32>, output_var: &LinguisticVar) -> f32 {
    let samples = 100usize;
    let (lo, hi) = output_var.range;
    let step = (hi - lo) / samples as f32;

    let mut numerator = 0.0f32;
    let mut denominator = 0.0f32;

    for i in 0..=samples {
        let x = lo + i as f32 * step;
        // Maximum of all clipped membership values at x (union of output MFs)
        let mut mu_max = 0.0f32;
        for (term_name, mf) in &output_var.terms {
            if let Some(&clip) = clips.get(term_name) {
                let mu = mf.eval(x).min(clip); // Mamdani clipping
                mu_max = mu_max.max(mu);
            }
        }
        numerator += x * mu_max;
        denominator += mu_max;
    }

    if denominator < 1e-8 {
        (lo + hi) / 2.0
    } else {
        (numerator / denominator).clamp(lo, hi)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::membership::var_state_output;

    fn output_var() -> LinguisticVar {
        var_state_output()
    }

    #[test]
    fn empty_clips_returns_midpoint() {
        let score = centroid_defuzz(&HashMap::new(), &output_var());
        assert!((score - 0.5).abs() < 1e-4, "midpoint={score}");
    }

    #[test]
    fn security_threat_dominant_returns_high_score() {
        let mut clips = HashMap::new();
        clips.insert("security_threat".into(), 0.95f32);
        let score = centroid_defuzz(&clips, &output_var());
        // security_threat term spans [0.70, 1.0] → CoG should be > 0.70
        assert!(score > 0.70, "security_threat CoG={score}");
    }

    #[test]
    fn clear_dominant_returns_low_score() {
        let mut clips = HashMap::new();
        clips.insert("clear".into(), 0.95f32);
        let score = centroid_defuzz(&clips, &output_var());
        // clear term spans [0.0, 0.35] → CoG should be < 0.35
        assert!(score < 0.35, "clear CoG={score}");
    }

    #[test]
    fn fever_clip_returns_mid_range() {
        let mut clips = HashMap::new();
        clips.insert("fever".into(), 0.80f32);
        let score = centroid_defuzz(&clips, &output_var());
        // fever term centered at 0.40 → CoG ≈ 0.25–0.55
        assert!(score > 0.25 && score < 0.55, "fever CoG={score}");
    }

    #[test]
    fn output_always_in_range() {
        let var = output_var();
        for &clip in &[0.0f32, 0.1, 0.5, 0.9, 1.0] {
            for term in ["clear", "fever", "stress", "security_threat"] {
                let mut clips = HashMap::new();
                clips.insert(term.into(), clip);
                let score = centroid_defuzz(&clips, &var);
                assert!(
                    score >= var.range.0 && score <= var.range.1,
                    "{term}@{clip} → score={score} out of range"
                );
            }
        }
    }
}
