//! 𒁾 Hepta Batch Scorer
//!
//! `HeptaScorer` scores individual particles and whole batches.
//! `BatchScoringResult` aggregates lane counts, mean health, and SLA flags.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::domain::{HeptaVector, TribeIdealPoint, HeptaScore, QualityLane};
use crate::{GEM_RATE_TARGET, STEWARD_RATE};

/// A particle with its 16-byte KAKI sovereign PK and computed HeptaScore.
#[derive(Debug, Clone)]
pub struct ScoredParticle {
    /// 16-byte KAKI sovereign PK
    pub kaki:  [u8; 16],
    /// The 7D quality vector
    pub hepta: HeptaVector,
    /// Full scoring result: B11, lane, dim contributions
    pub score: HeptaScore,
}

impl ScoredParticle {
    pub fn new(kaki: [u8; 16], hepta: HeptaVector, tribe: &TribeIdealPoint) -> Self {
        let score = hepta.score(tribe);
        Self { kaki, hepta, score }
    }

    pub fn is_gem(&self) -> bool { self.score.is_gem() }
    pub fn b11(&self)    -> u8   { self.score.b11 }
}

/// Aggregated statistics for a scored batch.
#[derive(Debug, Clone)]
pub struct BatchScoringResult {
    pub total:          usize,
    pub gem_count:      usize,
    pub tribe_count:    usize,
    pub active_count:   usize,
    pub fuzzy_count:    usize,
    pub dead_count:     usize,
    pub mean_health:    f32,
    pub mean_b11:       f32,
    pub gem_rate:       f32,
    /// Per-dimension mean scores across all particles
    pub dim_means:      [f32; 7],
    /// Index of the weakest dimension across the whole batch
    pub batch_weakest:  usize,
    /// ADR-004: GEM rate fell below 35.4% threshold
    pub sla_breach:     bool,
    /// DataSteward queue SLA breach
    pub steward_breach: bool,
}

impl BatchScoringResult {
    pub fn summary(&self) -> String {
        format!(
            "Batch: {} total | GEM {:.1}% ({}) | Dead {} | {}",
            self.total,
            self.gem_rate * 100.0,
            self.gem_count,
            self.dead_count,
            if self.sla_breach { "ADR-004 BREACH" } else { "SLA OK" }
        )
    }
}

/// The sovereign batch scorer.
pub struct HeptaScorer {
    tribe: TribeIdealPoint,
}

impl HeptaScorer {
    pub fn new(tribe: TribeIdealPoint) -> Self { Self { tribe } }

    /// Create with sovereign Arabic MDM weights (BahyWay default).
    pub fn sovereign() -> Self { Self::new(TribeIdealPoint::sovereign_arabic_mdm()) }

    pub fn score_one(&self, kaki: [u8; 16], hepta: HeptaVector) -> ScoredParticle {
        ScoredParticle::new(kaki, hepta, &self.tribe)
    }

    /// Score a batch of (KAKI, HeptaVector) pairs.
    pub fn score_batch(
        &self,
        particles: Vec<([u8; 16], HeptaVector)>,
    ) -> (Vec<ScoredParticle>, BatchScoringResult) {
        let scored: Vec<ScoredParticle> = particles
            .into_iter()
            .map(|(kaki, hepta)| ScoredParticle::new(kaki, hepta, &self.tribe))
            .collect();
        let stats = self.aggregate(&scored);
        (scored, stats)
    }

    pub fn aggregate(&self, scored: &[ScoredParticle]) -> BatchScoringResult {
        let total = scored.len();
        if total == 0 {
            return BatchScoringResult {
                total: 0, gem_count: 0, tribe_count: 0, active_count: 0,
                fuzzy_count: 0, dead_count: 0, mean_health: 0.0, mean_b11: 0.0,
                gem_rate: 0.0, dim_means: [0.0; 7], batch_weakest: 0,
                sla_breach: false, steward_breach: false,
            };
        }

        let mut gem_count    = 0usize;
        let mut tribe_count  = 0usize;
        let mut active_count = 0usize;
        let mut fuzzy_count  = 0usize;
        let mut dead_count   = 0usize;
        let mut sum_health   = 0.0f32;
        let mut sum_b11      = 0u32;
        let mut sum_dims     = [0.0f32; 7];
        let mut dim_weakest  = [0.0f32; 7];

        for p in scored {
            match p.score.lane {
                QualityLane::Gem         => gem_count    += 1,
                QualityLane::TribeMember => tribe_count  += 1,
                QualityLane::Active      => active_count += 1,
                QualityLane::Fuzzy       => fuzzy_count  += 1,
                QualityLane::Dead        => dead_count   += 1,
            }
            sum_health += p.score.health;
            sum_b11    += p.score.b11 as u32;
            for i in 0..7 {
                sum_dims[i]    += p.hepta[i];
                dim_weakest[i] += p.score.dim_contributions[i];
            }
        }

        let n = total as f32;
        let gem_rate   = gem_count   as f32 / n;
        let fuzzy_rate = fuzzy_count as f32 / n;

        let batch_weakest = dim_weakest.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        BatchScoringResult {
            total,
            gem_count, tribe_count, active_count, fuzzy_count, dead_count,
            mean_health:    sum_health / n,
            mean_b11:       sum_b11 as f32 / n,
            gem_rate,
            dim_means:      std::array::from_fn(|i| sum_dims[i] / n),
            batch_weakest,
            sla_breach:     gem_rate   < GEM_RATE_TARGET,
            steward_breach: fuzzy_rate > STEWARD_RATE * 2.0,
        }
    }

    /// Filter scored particles by lane.
    pub fn filter_lane<'a>(
        scored: &'a [ScoredParticle],
        lane: QualityLane,
    ) -> impl Iterator<Item = &'a ScoredParticle> {
        scored.iter().filter(move |p| p.score.lane == lane)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_particle(quality: f32) -> ([u8; 16], HeptaVector) {
        let kaki = [0x01u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0, 0, 0, 0, 0];
        let h = HeptaVector::new(quality, quality, quality, quality, quality, quality, quality);
        (kaki, h)
    }

    #[test]
    fn empty_batch_returns_zero_stats() {
        let (_, stats) = HeptaScorer::sovereign().score_batch(vec![]);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.gem_rate, 0.0);
    }

    #[test]
    fn perfect_batch_no_sla_breach() {
        let batch: Vec<_> = (0..100).map(|_| make_particle(1.0)).collect();
        let (_, stats) = HeptaScorer::sovereign().score_batch(batch);
        assert!(!stats.sla_breach, "perfect batch must not breach SLA");
        assert_eq!(stats.gem_count, 100);
    }

    #[test]
    fn poor_batch_triggers_sla_breach() {
        // q=0.2 → B11=133 → Active (Dead unreachable: H_min=0.5 with T=[1;7])
        let batch: Vec<_> = (0..100).map(|_| make_particle(0.2)).collect();
        let (_, stats) = HeptaScorer::sovereign().score_batch(batch);
        assert!(stats.sla_breach, "poor batch must trigger ADR-004 breach");
        assert_eq!(stats.active_count, 100);
    }

    #[test]
    fn scored_particles_preserve_kaki() {
        let kaki = [0x01u8, 0xAB, 0xCD, 0xEF, 0, 0, 0, 0, 0, 0, 0x10, 0xD4, 0, 0, 0, 0];
        let h = HeptaVector::new(0.95, 0.92, 0.93, 0.98, 0.90, 0.88, 1.00);
        let p = HeptaScorer::sovereign().score_one(kaki, h);
        assert_eq!(p.kaki, kaki);
    }

    #[test]
    fn gem_rate_matches_manual_count() {
        let mut batch = Vec::new();
        for _ in 0..40 { batch.push(make_particle(1.0)); }
        for _ in 0..60 { batch.push(make_particle(0.1)); }
        let (scored, stats) = HeptaScorer::sovereign().score_batch(batch);
        let actual_gems = scored.iter().filter(|p| p.is_gem()).count();
        assert_eq!(actual_gems, stats.gem_count);
    }

    #[test]
    fn batch_summary_non_empty() {
        let (_, stats) = HeptaScorer::sovereign().score_batch(vec![make_particle(0.95)]);
        assert!(!stats.summary().is_empty());
    }
}
