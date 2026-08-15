// ============================================================================
// fuzzy-engine/src/defuzz.rs
// Centroid defuzzification → quality byte B11
//
// Output universe: [0, 255]
// 4 output membership functions mapped to quality tiers:
//   NonActive : [0,   0,   60,  100]
//   Active    : [60,  100, 140, 180]
//   Tribe     : [140, 180, 200, 220]
//   Gem       : [200, 220, 255, 255]
// ============================================================================
#![allow(missing_docs)]

use crate::membership::{f_left, f_right, f_trap};
use crate::rules::AggregatedOutput;

const RESOLUTION: usize = 256; // sample points across [0, 255]

/// Defuzzify the aggregated output using centroid method.
/// Returns quality byte B11 (0–255).
pub fn centroid_defuzz(agg: &AggregatedOutput) -> u8 {
    let mut numerator   = 0.0f32;
    let mut denominator = 0.0f32;

    for i in 0..RESOLUTION {
        let x = i as f32;
        // Clip each tier's output membership to its activation strength (MIN)
        let mu_non_active = f_left(x, 60.0, 100.0).min(agg.non_active);
        let mu_active     = f_trap(x, 60.0, 100.0, 140.0, 180.0).min(agg.active);
        let mu_tribe      = f_trap(x, 140.0, 180.0, 200.0, 220.0).min(agg.tribe);
        let mu_gem        = f_right(x, 200.0, 220.0).min(agg.gem);

        // MAX aggregation across all four tiers at this point
        let mu = mu_non_active.max(mu_active).max(mu_tribe).max(mu_gem);

        numerator   += mu * x;
        denominator += mu;
    }

    if denominator < 1e-6 {
        // No rules fired — default to Active floor
        return 100;
    }

    (numerator / denominator).round().clamp(0.0, 255.0) as u8
}

/// Map a raw quality byte to its sovereign tier label.
pub fn quality_tier_label(b11: u8) -> &'static str {
    match b11 {
        200..=255 => "Gem",
        140..=199 => "Tribe",
        100..=139 => "Active",
        _         => "Dead",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::AggregatedOutput;

    #[test]
    fn full_gem_activation_scores_above_200() {
        let agg = AggregatedOutput { gem: 1.0, tribe: 0.0, active: 0.0, non_active: 0.0 };
        let b11 = centroid_defuzz(&agg);
        assert!(b11 >= 200, "full Gem must score >= 200, got {b11}");
    }

    #[test]
    fn full_non_active_scores_below_100() {
        let agg = AggregatedOutput { gem: 0.0, tribe: 0.0, active: 0.0, non_active: 1.0 };
        let b11 = centroid_defuzz(&agg);
        assert!(b11 < 100, "full NonActive must score < 100, got {b11}");
    }

    #[test]
    fn full_tribe_scores_140_to_199() {
        let agg = AggregatedOutput { gem: 0.0, tribe: 1.0, active: 0.0, non_active: 0.0 };
        let b11 = centroid_defuzz(&agg);
        assert!(b11 >= 140 && b11 <= 199, "full Tribe must score 140–199, got {b11}");
    }

    #[test]
    fn full_active_scores_100_to_139() {
        let agg = AggregatedOutput { gem: 0.0, tribe: 0.0, active: 1.0, non_active: 0.0 };
        let b11 = centroid_defuzz(&agg);
        assert!(b11 >= 100 && b11 <= 139, "full Active must score 100–139, got {b11}");
    }

    #[test]
    fn no_activation_returns_active_floor() {
        let agg = AggregatedOutput::default();
        let b11 = centroid_defuzz(&agg);
        assert_eq!(b11, 100, "no activation must return 100 (Active floor)");
    }

    #[test]
    fn quality_tier_label_gem()    { assert_eq!(quality_tier_label(200), "Gem");    assert_eq!(quality_tier_label(255), "Gem"); }
    #[test]
    fn quality_tier_label_tribe()  { assert_eq!(quality_tier_label(140), "Tribe");  assert_eq!(quality_tier_label(199), "Tribe"); }
    #[test]
    fn quality_tier_label_active() { assert_eq!(quality_tier_label(100), "Active"); assert_eq!(quality_tier_label(139), "Active"); }
    #[test]
    fn quality_tier_label_dead()   { assert_eq!(quality_tier_label(0),   "Dead");   assert_eq!(quality_tier_label(99),  "Dead"); }
}
