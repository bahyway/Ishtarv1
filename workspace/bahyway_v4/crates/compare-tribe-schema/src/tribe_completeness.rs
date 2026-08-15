// ============================================================================
// compare-tribe-schema/src/tribe_completeness.rs
// Cross-tribe optional attribute completeness ranking.
//
// A tribe with more optional fields filled is "more organized" (per spec).
// Ranking affects which Templates generate optional attributes per SLA.
// ============================================================================
#![allow(missing_docs)]

use core::cmp::Ordering;

/// Optional attribute completeness stats for one tribe.
#[derive(Debug, Clone)]
pub struct TribeCompleteness {
    pub tribe_name:         String,
    pub tribe_id:           u16,
    /// Total particles sampled.
    pub total_particles:    usize,
    /// Total optional EAV slots across all sampled particles (particles × optional_field_count).
    pub optional_slots:     usize,
    /// How many of those optional slots have a non-null value.
    pub optional_filled:    usize,
    /// optional_filled / optional_slots — 0.0 if optional_slots == 0.
    pub completeness_ratio: f32,
}

impl TribeCompleteness {
    pub fn new(
        tribe_name:      impl Into<String>,
        tribe_id:        u16,
        total_particles: usize,
        optional_slots:  usize,
        optional_filled: usize,
    ) -> Self {
        let ratio = if optional_slots == 0 {
            0.0
        } else {
            (optional_filled as f32 / optional_slots as f32).clamp(0.0, 1.0)
        };
        TribeCompleteness {
            tribe_name: tribe_name.into(),
            tribe_id,
            total_particles,
            optional_slots,
            optional_filled,
            completeness_ratio: ratio,
        }
    }

    /// Percentage 0–100.
    pub fn completeness_pct(&self) -> f32 {
        self.completeness_ratio * 100.0
    }
}

/// A tribe with its rank position (1 = most organized).
#[derive(Debug, Clone)]
pub struct TribeRank {
    /// 1-based rank (1 = best).
    pub rank:         usize,
    pub completeness: TribeCompleteness,
}

impl TribeRank {
    pub fn is_top_tier(&self) -> bool {
        self.rank == 1
    }
}

/// Rank tribes by optional attribute completeness, descending.
/// Ties are broken alphabetically by tribe_name for determinism.
pub fn rank_tribes(tribes: &[TribeCompleteness]) -> Vec<TribeRank> {
    let mut indexed: Vec<(usize, &TribeCompleteness)> = tribes.iter().enumerate().collect();
    indexed.sort_by(|(_, a), (_, b)| {
        b.completeness_ratio
            .partial_cmp(&a.completeness_ratio)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.tribe_name.cmp(&b.tribe_name))
    });
    indexed
        .into_iter()
        .enumerate()
        .map(|(rank_idx, (_, t))| TribeRank {
            rank:         rank_idx + 1,
            completeness: t.clone(),
        })
        .collect()
}

/// Return a reference to the most organized tribe, or None if empty.
pub fn most_organized(tribes: &[TribeCompleteness]) -> Option<&TribeCompleteness> {
    tribes.iter().max_by(|a, b| {
        a.completeness_ratio
            .partial_cmp(&b.completeness_ratio)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.tribe_name.cmp(&a.tribe_name))
    })
}

/// Compute the gap between the best and worst tribe's completeness ratio.
/// Useful for SLA scheduling — a large gap means some tribes need template attention.
pub fn completeness_gap(tribes: &[TribeCompleteness]) -> f32 {
    if tribes.is_empty() { return 0.0; }
    let max = tribes.iter().map(|t| t.completeness_ratio).fold(f32::NEG_INFINITY, f32::max);
    let min = tribes.iter().map(|t| t.completeness_ratio).fold(f32::INFINITY,     f32::min);
    (max - min).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tribes() -> Vec<TribeCompleteness> {
        vec![
            TribeCompleteness::new("Baghdad",  0x0001, 1000, 3000, 2700), // 90%
            TribeCompleteness::new("Najaf",    0x0002,  500, 1500,  900), // 60%
            TribeCompleteness::new("Basra",    0x0003,  800, 2400, 2160), // 90% tie
            TribeCompleteness::new("Mosul",    0x0004,  300,  900,  270), // 30%
        ]
    }

    #[test]
    fn completeness_ratio_correct() {
        let t = TribeCompleteness::new("X", 1, 100, 200, 150);
        assert!((t.completeness_ratio - 0.75).abs() < 0.001);
        assert!((t.completeness_pct()  - 75.0).abs() < 0.01);
    }

    #[test]
    fn zero_optional_slots_is_zero_ratio() {
        let t = TribeCompleteness::new("Empty", 1, 0, 0, 0);
        assert_eq!(t.completeness_ratio, 0.0);
    }

    #[test]
    fn rank_tribes_most_organized_is_rank_one() {
        let tribes  = sample_tribes();
        let ranked  = rank_tribes(&tribes);
        assert_eq!(ranked[0].rank, 1);
        // Baghdad and Basra are both 90% — alphabetically Baghdad wins
        assert_eq!(ranked[0].completeness.tribe_name, "Baghdad");
        assert_eq!(ranked[1].completeness.tribe_name, "Basra");
    }

    #[test]
    fn rank_tribes_least_organized_is_last() {
        let tribes = sample_tribes();
        let ranked = rank_tribes(&tribes);
        assert_eq!(ranked.last().unwrap().completeness.tribe_name, "Mosul");
    }

    #[test]
    fn most_organized_returns_best_tribe() {
        let tribes = sample_tribes();
        let best   = most_organized(&tribes).unwrap();
        assert_eq!(best.tribe_name, "Baghdad");
    }

    #[test]
    fn most_organized_on_empty_is_none() {
        assert!(most_organized(&[]).is_none());
    }

    #[test]
    fn completeness_gap_correct() {
        let tribes = sample_tribes();
        let gap    = completeness_gap(&tribes);
        // max=0.9, min=0.3 → gap=0.6
        assert!((gap - 0.6).abs() < 0.01, "gap should be ~0.6, got {gap}");
    }

    #[test]
    fn rank_count_matches_input() {
        let tribes = sample_tribes();
        let ranked = rank_tribes(&tribes);
        assert_eq!(ranked.len(), tribes.len());
    }
}
