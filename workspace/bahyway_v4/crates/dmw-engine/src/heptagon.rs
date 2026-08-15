//! heptagon.rs — 7D Heptagon Physics Weights
//!
//! The Heptagon maps the 7 Journey levels onto physical force dimensions:
//!
//!   ME  → intent     — query purpose weight (0=background, 1=mission-critical)
//!   GU  → mass       — row volume / data gravity
//!   SAG → gravity    — selectivity / filter pull toward the tribe centre
//!   A   → viscosity  — IO friction from join complexity   (INVERTED: high=bad)
//!   IZI → heat       — CPU row-by-row work pressure       (INVERTED: high=bad)
//!   UD  → volatility — cache decay / plan staleness rate  (INVERTED: high=bad)
//!   URU → strength   — index alignment / tribe cohesion
//!
//! `viscosity`, `heat`, and `volatility` are inverted from their Journey scores
//! so that zero always means "optimal" and one means "maximum degradation".
//!
//! `alignment_score()` uses domain-specific weights identical to v3.5.1:
//!   gravity ×2, viscosity-complement ×2, strength ×2 dominate performance.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

use crate::journey::{JourneyLevelId, SevenLevelJourney};

// ── HeptaWeights ──────────────────────────────────────────────────────────────

/// The 7 Heptagon dimension weights for one query analysis.
///
/// Derived from a `SevenLevelJourney` via `from_journey()`.
/// `viscosity`, `heat`, and `volatility` are INVERTED Journey scores:
/// a value of 0.0 means optimal; 1.0 means maximum degradation.
#[derive(Debug, Clone, Copy)]
pub struct HeptaWeights {
    /// ME  — Intent: business priority (0=background, 1=mission-critical).
    pub intent:     f32,
    /// GU  — Mass: normalised row volume (0=tiny, 1=large set).
    pub mass:       f32,
    /// SAG — Gravity: predicate selectivity (0=full scan, 1=point seek).
    pub gravity:    f32,
    /// A   — Viscosity: IO friction from join complexity (0=smooth, 1=stuck).
    pub viscosity:  f32,
    /// IZI — Heat: CPU row-by-row pressure (0=idle, 1=row-by-row overload).
    pub heat:       f32,
    /// UD  — Volatility: plan cache decay rate (0=stable, 1=fully volatile).
    pub volatility: f32,
    /// URU — Strength: index alignment / tribe cohesion (0=no index, 1=perfect).
    pub strength:   f32,
}

impl HeptaWeights {
    /// Derive 7D weights from a completed `SevenLevelJourney`.
    ///
    /// Friction dimensions (A, IZI, UD) are inverted so 0 = optimal throughout.
    pub fn from_journey(journey: &SevenLevelJourney) -> Self {
        HeptaWeights {
            intent:     journey.level_score(JourneyLevelId::ME),
            mass:       journey.level_score(JourneyLevelId::GU),
            gravity:    journey.level_score(JourneyLevelId::SAG),
            viscosity:  1.0 - journey.level_score(JourneyLevelId::A),
            heat:       1.0 - journey.level_score(JourneyLevelId::IZI),
            volatility: 1.0 - journey.level_score(JourneyLevelId::UD),
            strength:   journey.level_score(JourneyLevelId::URU),
        }
    }

    /// Sovereign Alignment Score — weighted mean of all 7 dimensions.
    ///
    /// High alignment → healthy, laminar query.
    /// Low alignment  → vortex, needs Oracle intervention.
    ///
    /// Gravity, viscosity-complement, and strength are double-weighted
    /// because they dominate real-world query execution time.
    pub fn alignment_score(&self) -> f32 {
        let weighted = self.intent               * 1.0
            + self.mass                          * 0.8
            + self.gravity                       * 2.0
            + (1.0 - self.viscosity).max(0.0)   * 2.0
            + (1.0 - self.heat).max(0.0)         * 1.0
            + (1.0 - self.volatility).max(0.0)   * 0.8
            + self.strength                      * 2.0;
        (weighted / 9.6).clamp(0.0, 1.0)
    }

    /// Returns `true` when the Gray-Rot threshold is breached (alignment < 20%).
    pub fn is_gray_rot(&self) -> bool {
        self.alignment_score() < 0.20
    }

    /// Returns the dominant performance leak dimension — the worst scorer.
    pub fn dominant_leak(&self) -> HeptaDimension {
        let scores = [
            (self.intent,                   HeptaDimension::Intent),
            (self.mass,                     HeptaDimension::Mass),
            (self.gravity,                  HeptaDimension::Gravity),
            (1.0 - self.viscosity,          HeptaDimension::Viscosity),
            (1.0 - self.heat,               HeptaDimension::Heat),
            (1.0 - self.volatility,         HeptaDimension::Volatility),
            (self.strength,                 HeptaDimension::Strength),
        ];
        scores.iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, d)| *d)
            .unwrap_or(HeptaDimension::Viscosity)
    }

    /// Returns all 7 (dimension, score) pairs ordered by the 7-gate sequence.
    pub fn all_scores(&self) -> [(HeptaDimension, f32); 7] {
        [
            (HeptaDimension::Intent,    self.intent),
            (HeptaDimension::Mass,      self.mass),
            (HeptaDimension::Gravity,   self.gravity),
            (HeptaDimension::Viscosity, 1.0 - self.viscosity),
            (HeptaDimension::Heat,      1.0 - self.heat),
            (HeptaDimension::Volatility,1.0 - self.volatility),
            (HeptaDimension::Strength,  self.strength),
        ]
    }
}

impl std::fmt::Display for HeptaWeights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "ME:{:.2} GU:{:.2} SAG:{:.2} A:{:.2} IZI:{:.2} UD:{:.2} URU:{:.2}",
            self.intent, self.mass, self.gravity,
            self.viscosity, self.heat, self.volatility, self.strength)
    }
}

// ── HeptaDimension ────────────────────────────────────────────────────────────

/// The 7 named dimensions of the Heptagon — one per Journey level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeptaDimension {
    Intent,
    Mass,
    Gravity,
    Viscosity,
    Heat,
    Volatility,
    Strength,
}

impl HeptaDimension {
    pub fn label(self) -> &'static str {
        match self {
            HeptaDimension::Intent     => "ME (Intent)",
            HeptaDimension::Mass       => "GU (Mass)",
            HeptaDimension::Gravity    => "SAG (Gravity)",
            HeptaDimension::Viscosity  => "A (Viscosity)",
            HeptaDimension::Heat       => "IZI (Heat)",
            HeptaDimension::Volatility => "UD (Volatility)",
            HeptaDimension::Strength   => "URU (Strength)",
        }
    }
}

impl std::fmt::Display for HeptaDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bottleneck::BottleneckDetector;
    use crate::journey::SevenLevelJourney;
    use crate::plan::{sales_order_plan, simple_nested_loop_plan, OpKind, PlanNode, QueryPlan, ScanType};

    fn weights_for(plan: &QueryPlan) -> HeptaWeights {
        let bns = BottleneckDetector::detect(plan);
        let j   = SevenLevelJourney::analyze(plan, &bns);
        HeptaWeights::from_journey(&j)
    }

    #[test]
    fn perfect_plan_high_alignment() {
        let root = PlanNode::new(
            OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered },
            10.0, 100, 50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let w = weights_for(&plan);
        assert!(w.alignment_score() > 0.80,
            "perfect plan alignment={:.2}", w.alignment_score());
    }

    #[test]
    fn nested_loop_plan_lower_alignment() {
        let w = weights_for(&simple_nested_loop_plan());
        assert!(w.alignment_score() < 0.75,
            "nested loop plan alignment={:.2}", w.alignment_score());
    }

    #[test]
    fn alignment_score_in_range() {
        let w = weights_for(&sales_order_plan());
        let s = w.alignment_score();
        assert!(s >= 0.0 && s <= 1.0, "score={s}");
    }

    #[test]
    fn viscosity_inverted_from_join_level() {
        let w_good = weights_for(&sales_order_plan());
        let w_bad  = weights_for(&simple_nested_loop_plan());
        // SalesOrder uses HashMatch → lower viscosity than NestedLoop plan
        assert!(w_bad.viscosity > w_good.viscosity,
            "bad={:.2} good={:.2}", w_bad.viscosity, w_good.viscosity);
    }

    #[test]
    fn dominant_leak_is_worst_dimension() {
        let w = weights_for(&simple_nested_loop_plan());
        let leak = w.dominant_leak();
        // NestedLoop + TableScan degrades Gravity or Viscosity or Heat
        let valid = [
            HeptaDimension::Gravity,
            HeptaDimension::Viscosity,
            HeptaDimension::Heat,
        ];
        assert!(valid.contains(&leak), "leak={:?}", leak);
    }

    #[test]
    fn all_scores_has_seven_entries() {
        let w = weights_for(&sales_order_plan());
        assert_eq!(w.all_scores().len(), 7);
    }

    #[test]
    fn all_scores_values_in_range() {
        let w = weights_for(&simple_nested_loop_plan());
        for (dim, score) in w.all_scores() {
            assert!(score >= 0.0 && score <= 1.0,
                "{:?} score={score}", dim);
        }
    }

    #[test]
    fn display_shows_all_seven_labels() {
        let w = weights_for(&sales_order_plan());
        let s = format!("{w}");
        for label in ["ME:", "GU:", "SAG:", "A:", "IZI:", "UD:", "URU:"] {
            assert!(s.contains(label), "missing '{label}' in '{s}'");
        }
    }

    #[test]
    fn dimension_labels_non_empty() {
        for d in [
            HeptaDimension::Intent, HeptaDimension::Mass, HeptaDimension::Gravity,
            HeptaDimension::Viscosity, HeptaDimension::Heat, HeptaDimension::Volatility,
            HeptaDimension::Strength,
        ] { assert!(!d.label().is_empty()); }
    }

    #[test]
    fn is_gray_rot_false_for_perfect_plan() {
        let root = PlanNode::new(
            OpKind::IndexSeek { name: "PK_T".into(), scan_type: ScanType::Clustered },
            10.0, 100, 50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        assert!(!weights_for(&plan).is_gray_rot());
    }
}
