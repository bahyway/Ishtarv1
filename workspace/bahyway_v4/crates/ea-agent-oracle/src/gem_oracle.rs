//! gem_oracle.rs — GEM rate tracking vs ADR-004 target (35.4%).
#![forbid(unsafe_code)]

use ea_agent_core::{ParticleSnapshot, constants::GEM_RATE_TARGET};

/// GEM rate report for a tribe.
#[derive(Debug)]
pub struct GemReport {
    pub tribe_id:      u16,
    pub epoch:         u64,
    pub total:         usize,
    pub gem_count:     usize,
    pub gem_rate:      f64,
    pub target:        f64,
    pub delta:         f64,   // gem_rate - target (positive = above target)
    pub assessment:    String,
}

impl GemReport {
    pub fn meets_target(&self) -> bool { self.gem_rate >= self.target }
    pub fn summary(&self) -> String {
        let icon = if self.meets_target() { "✅" } else { "⚠" };
        format!("{icon} Tribe 0x{:04X} | GEM: {}/{} ({:.1}%) | Target: {:.1}% | Δ={:+.1}%",
            self.tribe_id, self.gem_count, self.total,
            self.gem_rate * 100.0, self.target * 100.0, self.delta * 100.0)
    }
}

/// GEM rate oracle — tracks ADR-004 compliance.
pub struct GemOracle {
    pub history: Vec<GemReport>,
}

impl GemOracle {
    pub fn new() -> Self { Self { history: Vec::new() } }

    pub fn evaluate(&mut self, tribe_id: u16, epoch: u64, particles: &[ParticleSnapshot]) -> &GemReport {
        let total     = particles.len();
        let gem_count = particles.iter().filter(|p| p.is_gem()).count();
        let gem_rate  = if total > 0 { gem_count as f64 / total as f64 } else { 0.0 };
        let delta     = gem_rate - GEM_RATE_TARGET;

        let assessment = if total == 0 {
            "Empty tribe.".into()
        } else if gem_rate >= GEM_RATE_TARGET {
            format!("ADR-004 met. {:.1}% GEM particles (target {:.1}%).",
                gem_rate * 100.0, GEM_RATE_TARGET * 100.0)
        } else {
            let deficit = (GEM_RATE_TARGET - gem_rate) * total as f64;
            format!("ADR-004 deficit: need {:.0} more GEM particles. Apply DQM remediation.",
                deficit.ceil())
        };

        self.history.push(GemReport { tribe_id, epoch, total, gem_count, gem_rate,
            target: GEM_RATE_TARGET, delta, assessment });
        self.history.last().unwrap()
    }

    /// Trend: is GEM rate improving?
    pub fn is_improving(&self) -> bool {
        if self.history.len() < 2 { return false; }
        let last = self.history.last().unwrap().gem_rate;
        let prev = self.history[self.history.len()-2].gem_rate;
        last > prev
    }

    /// Epochs above target consecutively.
    pub fn compliance_streak(&self) -> usize {
        self.history.iter().rev().take_while(|r| r.meets_target()).count()
    }
}

impl Default for GemOracle { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use ea_agent_core::HeptaVector;

    fn gem_particle(name: &str) -> ParticleSnapshot {
        ParticleSnapshot::new(name, 0x1234, 0x0001, HeptaVector::unit())
    }
    fn dead_particle(name: &str) -> ParticleSnapshot {
        ParticleSnapshot::new(name, 0x5678, 0x0001, HeptaVector::zero())
    }

    #[test]
    fn all_gem_meets_target() {
        let mut oracle = GemOracle::new();
        let particles: Vec<_> = (0..5).map(|i| gem_particle(&format!("G{i}"))).collect();
        let r = oracle.evaluate(0x0001, 1, &particles);
        assert!(r.meets_target());
        assert!((r.gem_rate - 1.0).abs() < 1e-10);
    }

    #[test]
    fn no_gem_fails_target() {
        let mut oracle = GemOracle::new();
        let particles: Vec<_> = (0..5).map(|i| dead_particle(&format!("D{i}"))).collect();
        let r = oracle.evaluate(0x0001, 1, &particles);
        assert!(!r.meets_target());
    }

    #[test]
    fn gem_rate_computed_correctly() {
        let mut oracle = GemOracle::new();
        let mut particles = vec![gem_particle("G1"), gem_particle("G2")];
        particles.extend((0..8).map(|i| dead_particle(&format!("D{i}"))));
        let r = oracle.evaluate(0x0001, 1, &particles);
        assert!((r.gem_rate - 0.2).abs() < 0.01);
    }

    #[test]
    fn compliance_streak_counted() {
        let mut oracle = GemOracle::new();
        let gems: Vec<_> = (0..5).map(|i| gem_particle(&format!("G{i}"))).collect();
        for ep in 1..=3 { oracle.evaluate(0x0001, ep, &gems); }
        assert_eq!(oracle.compliance_streak(), 3);
    }

    #[test]
    fn improvement_detected() {
        let mut oracle = GemOracle::new();
        let few_gems: Vec<_> = vec![gem_particle("G1"), dead_particle("D1"), dead_particle("D2")];
        let more_gems: Vec<_> = vec![gem_particle("G1"), gem_particle("G2"), dead_particle("D1")];
        oracle.evaluate(0x0001, 1, &few_gems);
        oracle.evaluate(0x0001, 2, &more_gems);
        assert!(oracle.is_improving());
    }
}
