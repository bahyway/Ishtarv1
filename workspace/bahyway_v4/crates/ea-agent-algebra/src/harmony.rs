//! harmony.rs — Tribe Harmony Coefficient for EaAgent.
#![forbid(unsafe_code)]

use ea_agent_core::constants::GEM_RATE_TARGET;
use ea_agent_core::{HeptaVector, ParticleSnapshot};

#[derive(Debug)]
pub struct HarmonyResult {
    pub tribe_id: u16,
    pub particle_count: usize,
    pub harmony: f64,  // τ ∈ [0, 1]
    pub gem_rate: f64, // fraction of GEM particles
    pub centroid: HeptaVector,
    pub avg_b11: f64,
    pub assessment: String,
}

impl HarmonyResult {
    pub fn is_harmonious(&self) -> bool {
        self.harmony >= 0.75
    }
    pub fn meets_gem_target(&self) -> bool {
        self.gem_rate >= GEM_RATE_TARGET
    }
}

pub struct HarmonyEngine;

impl HarmonyEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute(&self, tribe_id: u16, particles: &[ParticleSnapshot]) -> HarmonyResult {
        let n = particles.len();
        if n == 0 {
            return HarmonyResult {
                tribe_id,
                particle_count: 0,
                harmony: 0.0,
                gem_rate: 0.0,
                centroid: HeptaVector::zero(),
                avg_b11: 0.0,
                assessment: "Empty tribe.".into(),
            };
        }

        let refs: Vec<&HeptaVector> = particles.iter().map(|p| &p.hepta).collect();
        let centroid = HeptaVector::centroid(&refs);
        let avg_b11 = particles.iter().map(|p| p.b11() as f64).sum::<f64>() / n as f64;
        let gem_rate = particles.iter().filter(|p| p.is_gem()).count() as f64 / n as f64;

        // Harmony = geometric mean of (1 - variance_per_dim)
        let harmony = self.compute_harmony(particles, &centroid);

        let assessment = self.assess(harmony, gem_rate, avg_b11);

        HarmonyResult {
            tribe_id,
            particle_count: n,
            harmony,
            gem_rate,
            centroid,
            avg_b11,
            assessment,
        }
    }

    fn compute_harmony(&self, particles: &[ParticleSnapshot], centroid: &HeptaVector) -> f64 {
        let n = particles.len() as f64;
        let mut product = 1.0f64;
        for d in 0..7 {
            let var: f64 = particles
                .iter()
                .map(|p| (p.hepta.d[d] - centroid.d[d]).powi(2))
                .sum::<f64>()
                / n;
            product *= (1.0 - var.min(1.0)).max(0.0);
        }
        product.powf(1.0 / 7.0) // 7th root = geometric mean
    }

    fn assess(&self, harmony: f64, gem_rate: f64, avg_b11: f64) -> String {
        let target_met = if gem_rate >= GEM_RATE_TARGET {
            "✅"
        } else {
            "⚠"
        };
        format!(
            "Harmony: {:.3} | GEM rate: {:.1}% {target_met} (target {:.1}%) | Avg B11: {:.1}/240",
            harmony,
            gem_rate * 100.0,
            GEM_RATE_TARGET * 100.0,
            avg_b11
        )
    }
}

impl Default for HarmonyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_p(d: [f64; 7]) -> ParticleSnapshot {
        ParticleSnapshot::new("P", 0x1234, 0x0001, HeptaVector::new(d))
    }

    #[test]
    fn empty_tribe_harmony_zero() {
        let r = HarmonyEngine::new().compute(0x0001, &[]);
        assert_eq!(r.harmony, 0.0);
    }

    #[test]
    fn identical_particles_max_harmony() {
        let particles: Vec<_> = (0..5).map(|_| make_p([0.9; 7])).collect();
        let r = HarmonyEngine::new().compute(0x0001, &particles);
        assert!(
            r.harmony > 0.95,
            "identical particles → harmony ≈ 1.0, got {}",
            r.harmony
        );
    }

    #[test]
    fn gem_rate_computed_correctly() {
        let gem = make_p([1.0; 7]); // B11=240 → GEM
        let dead = make_p([0.0; 7]); // B11≈low → not GEM
        let r = HarmonyEngine::new().compute(0x0001, &[gem, dead]);
        assert!((r.gem_rate - 0.5).abs() < 0.1);
    }

    #[test]
    fn assessment_contains_gem_rate() {
        let p = make_p([0.9; 7]);
        let r = HarmonyEngine::new().compute(0x0001, &[p]);
        assert!(r.assessment.contains("GEM rate"));
    }
}
