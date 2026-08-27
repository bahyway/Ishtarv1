//! ColorId — KAKI-compatible RGB health fingerprint for a DMW query particle.
//!
//! Three channels encode the three performance axes of query health:
//!
//!   Red   (0–255): cost intensity      — 255 = catastrophic cost,  0 = negligible
//!   Green (0–255): cardinality fidelity — 255 = estimate is exact, 0 = extreme mismatch
//!   Blue  (0–255): index coverage       — 255 = fully covered,     0 = all full scans
//!
//! `level_bitmask` encodes which of the 7 Journey gates PASSED (only set when
//! derived from a `SevenLevelJourney` via `from_journey()`; zero otherwise):
//!   bit 0 = L1 ME, bit 1 = L2 GU, …, bit 6 = L7 URU
//!
//! The sovereign healthy fingerprint is RGB(10, 220, 200) — low cost, accurate
//! cardinality, well-indexed — the teal-gold of BahyWay.
//! A sick particle is RGB(220, 30, 10) — the warning red of Gray-Rot.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0

use crate::journey::{JourneyLevelId, SevenLevelJourney};
use crate::plan::QueryPlan;

/// RGB health fingerprint for a single query plan analysis.
///
/// Compact (4 bytes), deterministic, and human-readable in hex — every query
/// becomes a visually distinguishable health particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorId {
    /// Cost channel: 0 = minimal cost, 255 = catastrophic cost.
    pub red: u8,
    /// Cardinality fidelity: 255 = estimate matches actual, 0 = extreme mismatch.
    pub green: u8,
    /// Index coverage: 255 = fully covered, 0 = full scans everywhere.
    pub blue: u8,
    /// Bitmask of Journey levels that PASSED (bit n = level n, 0-indexed).
    /// Zero when created via `new()` — only meaningful from `from_journey()`.
    pub level_bitmask: u8,
}

impl ColorId {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Creates a ColorId from raw RGB values (level_bitmask = 0).
    /// Used by analysis modules that compute channels independently.
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            level_bitmask: 0,
        }
    }

    /// Healthy sovereign particle — low cost, accurate stats, good index.
    /// RGB(10, 220, 200) — the teal-gold of BahyWay.
    pub fn healthy() -> Self {
        Self::new(10, 220, 200)
    }

    /// Sick particle — high cost, wrong stats, no index.
    /// RGB(220, 30, 10) — the warning red of Gray-Rot.
    pub fn critical() -> Self {
        Self::new(220, 30, 10)
    }

    /// Fuzzy particle — moderate issues across all channels.
    /// RGB(180, 140, 80) — amber caution.
    pub fn warning() -> Self {
        Self::new(180, 140, 80)
    }

    /// Derive a ColorId from the query plan and its Seven-Level Journey result.
    ///
    /// - Red   ← plan `total_cost` normalised against a 500-unit cap
    /// - Green ← L2 GU score (cardinality level)
    /// - Blue  ← L7 URU score (index coverage level)
    /// - Bitmask ← which Journey gates passed
    pub fn from_journey(plan: &QueryPlan, journey: &SevenLevelJourney) -> Self {
        let red = ((plan.total_cost / 500.0).min(1.0) * 255.0) as u8;
        let green = (journey.level_score(JourneyLevelId::GU) * 255.0) as u8;
        let blue = (journey.level_score(JourneyLevelId::URU) * 255.0) as u8;

        let mut bitmask: u8 = 0;
        for level in journey.all_levels() {
            if level.passed {
                bitmask |= 1 << (level.level as u8);
            }
        }
        Self {
            red,
            green,
            blue,
            level_bitmask: bitmask,
        }
    }

    // ── Core health metrics ───────────────────────────────────────────────────

    /// Composite health score (0.0 = critical, 1.0 = healthy).
    ///
    /// Weights: cost 40 %, cardinality 30 %, index coverage 30 %.
    pub fn health_score(self) -> f32 {
        let cost_health = 1.0 - (self.red as f32 / 255.0);
        let card_health = self.green as f32 / 255.0;
        let index_health = self.blue as f32 / 255.0;
        cost_health * 0.40 + card_health * 0.30 + index_health * 0.30
    }

    /// `true` when the fingerprint indicates a healthy query (health_score ≥ 0.75).
    pub fn is_healthy(self) -> bool {
        self.health_score() >= 0.75
    }

    /// Human-readable status label derived from the composite health score.
    pub fn status_label(self) -> &'static str {
        let s = self.health_score();
        if s >= 0.75 {
            "Golden"
        } else if s >= 0.50 {
            "Fuzzy"
        } else if s >= 0.25 {
            "Weak"
        } else {
            "Dead"
        }
    }

    // ── Channel accessors ────────────────────────────────────────────────────

    /// Returns the (red, green, blue) triple.
    pub fn rgb_tuple(self) -> (u8, u8, u8) {
        (self.red, self.green, self.blue)
    }

    /// Returns the three colour bytes for KAKI PK embedding.
    pub fn as_bytes(self) -> [u8; 3] {
        [self.red, self.green, self.blue]
    }

    /// Number of Journey levels that passed (0–7). Only meaningful from `from_journey()`.
    pub fn passed_count(self) -> u32 {
        self.level_bitmask.count_ones()
    }

    /// `true` when all 7 Journey levels passed.
    pub fn all_passed(self) -> bool {
        self.level_bitmask == 0x7F
    }

    // ── Formatting ───────────────────────────────────────────────────────────

    /// CSS / hex colour string, e.g. `"#1aff8c"`.
    pub fn to_hex_string(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }

    // ── Composition ──────────────────────────────────────────────────────────

    /// Merges two ColorIds by taking the worst (most degraded) per channel.
    /// Used when combining analysis results from multiple levels.
    pub fn merge_worst(self, other: ColorId) -> ColorId {
        ColorId::new(
            self.red.max(other.red),     // higher red = worse cost
            self.green.min(other.green), // lower green = worse cardinality
            self.blue.min(other.blue),   // lower blue = worse index
        )
    }

    /// Improves this ColorId toward the target after an Oracle rotation is applied.
    /// `factor` is 0.0 (no change) → 1.0 (fully healed).
    pub fn apply_improvement(self, factor: f32) -> ColorId {
        let f = factor.clamp(0.0, 1.0);
        ColorId::new(
            (self.red as f32 * (1.0 - f)) as u8,
            (self.green as f32 + (255.0 - self.green as f32) * f) as u8,
            (self.blue as f32 + (255.0 - self.blue as f32) * f) as u8,
        )
    }
}

impl std::fmt::Display for ColorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RGB({}, {}, {}) [{}]",
            self.red,
            self.green,
            self.blue,
            self.status_label()
        )
    }
}

// ── Level Contribution ────────────────────────────────────────────────────────

/// How a single analysis level contributes to the composite ColorId.
#[derive(Debug, Clone)]
pub struct LevelColorContribution {
    pub level_name: String,
    pub color_id: ColorId,
    pub description: String,
}

impl LevelColorContribution {
    pub fn new(level_name: &str, color_id: ColorId, description: &str) -> Self {
        Self {
            level_name: level_name.to_string(),
            color_id,
            description: description.to_string(),
        }
    }
}

// ── ColorId Builder ───────────────────────────────────────────────────────────

/// Builds the final ColorId from contributions across all 7 analysis levels.
///
/// Each level calls `add()` with its `LevelColorContribution`; `build()` merges
/// all contributions using `merge_worst()` so the final particle reflects the
/// worst-case channel across the entire 7-gate journey.
#[derive(Debug, Default)]
pub struct ColorIdBuilder {
    contributions: Vec<LevelColorContribution>,
}

impl ColorIdBuilder {
    pub fn new() -> Self {
        Self {
            contributions: Vec::new(),
        }
    }

    pub fn add(&mut self, contribution: LevelColorContribution) -> &mut Self {
        self.contributions.push(contribution);
        self
    }

    /// Merges all level contributions into the final particle ColorId.
    /// Returns `ColorId::healthy()` when no contributions have been added.
    pub fn build(&self) -> ColorId {
        if self.contributions.is_empty() {
            return ColorId::healthy();
        }
        let mut result = self.contributions[0].color_id;
        for c in &self.contributions[1..] {
            result = result.merge_worst(c.color_id);
        }
        result
    }

    pub fn contributions(&self) -> &[LevelColorContribution] {
        &self.contributions
    }

    pub fn contribution_count(&self) -> usize {
        self.contributions.len()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bottleneck::BottleneckDetector;
    use crate::plan::{
        sales_order_plan, simple_nested_loop_plan, OpKind, PlanNode, QueryPlan, ScanType,
    };

    fn color_for(plan: &QueryPlan) -> ColorId {
        let bns = BottleneckDetector::detect(plan);
        let j = crate::journey::SevenLevelJourney::analyze(plan, &bns);
        ColorId::from_journey(plan, &j)
    }

    // ── Preset constructors ────────────────────────────────────────────────

    #[test]
    fn healthy_preset_is_healthy() {
        assert!(ColorId::healthy().is_healthy());
    }

    #[test]
    fn critical_preset_is_not_healthy() {
        assert!(!ColorId::critical().is_healthy());
    }

    #[test]
    fn warning_preset_is_not_healthy() {
        assert!(!ColorId::warning().is_healthy());
    }

    #[test]
    fn healthy_health_score_above_70() {
        assert!(ColorId::healthy().health_score() > 0.70);
    }

    #[test]
    fn critical_health_score_below_30() {
        assert!(ColorId::critical().health_score() < 0.30);
    }

    // ── new() constructor ──────────────────────────────────────────────────

    #[test]
    fn new_sets_bitmask_to_zero() {
        let c = ColorId::new(10, 200, 200);
        assert_eq!(c.level_bitmask, 0);
    }

    #[test]
    fn new_fields_match() {
        let c = ColorId::new(42, 100, 180);
        assert_eq!(c.rgb_tuple(), (42, 100, 180));
    }

    // ── from_journey ──────────────────────────────────────────────────────

    #[test]
    fn hex_string_format() {
        let c = color_for(&sales_order_plan());
        let h = c.to_hex_string();
        assert!(h.starts_with('#'), "must start with #");
        assert_eq!(h.len(), 7, "must be 7 chars");
    }

    #[test]
    fn rgb_tuple_matches_fields() {
        let c = color_for(&sales_order_plan());
        assert_eq!(c.rgb_tuple(), (c.red, c.green, c.blue));
    }

    #[test]
    fn as_bytes_matches_rgb() {
        let c = ColorId::new(10, 220, 200);
        assert_eq!(c.as_bytes(), [10, 220, 200]);
    }

    #[test]
    fn passed_count_matches_bitmask() {
        let c = color_for(&sales_order_plan());
        assert_eq!(c.passed_count(), c.level_bitmask.count_ones());
    }

    #[test]
    fn passed_count_in_range() {
        let c = color_for(&simple_nested_loop_plan());
        assert!(c.passed_count() <= 7);
    }

    #[test]
    fn simple_plan_not_all_passed() {
        let c = color_for(&simple_nested_loop_plan());
        assert!(
            !c.all_passed(),
            "simple plan cannot have all 7 gates passing"
        );
    }

    #[test]
    fn perfect_plan_is_healthy() {
        let root = PlanNode::new(
            OpKind::IndexSeek {
                name: "PK_T".into(),
                scan_type: ScanType::Clustered,
            },
            10.0,
            100,
            50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let c = color_for(&plan);
        assert!(
            c.is_healthy(),
            "red={} green={} blue={}",
            c.red,
            c.green,
            c.blue
        );
    }

    #[test]
    fn perfect_plan_all_passed() {
        let root = PlanNode::new(
            OpKind::IndexSeek {
                name: "PK_T".into(),
                scan_type: ScanType::Clustered,
            },
            10.0,
            100,
            50,
        );
        let plan = QueryPlan::new("perfect", root, 10.0).with_actual_rows(100);
        let c = color_for(&plan);
        assert!(c.all_passed());
        assert_eq!(c.passed_count(), 7);
    }

    #[test]
    fn high_cost_plan_has_high_red() {
        let root = PlanNode::new(
            OpKind::IndexSeek {
                name: "PK_T".into(),
                scan_type: ScanType::Clustered,
            },
            100.0,
            100,
            50,
        );
        let plan = QueryPlan::new("costly", root, 1000.0).with_actual_rows(100);
        let c = color_for(&plan);
        assert_eq!(c.red, 255);
    }

    #[test]
    fn sales_order_passes_at_least_five_gates() {
        let c = color_for(&sales_order_plan());
        assert!(c.passed_count() >= 5, "passed={}", c.passed_count());
    }

    // ── merge_worst ───────────────────────────────────────────────────────

    #[test]
    fn merge_worst_takes_max_red() {
        let a = ColorId::new(100, 200, 200);
        let b = ColorId::new(50, 150, 180);
        let m = a.merge_worst(b);
        assert_eq!(m.red, 100);
        assert_eq!(m.green, 150);
        assert_eq!(m.blue, 180);
    }

    #[test]
    fn merge_worst_is_commutative_for_extreme_values() {
        let healthy = ColorId::healthy();
        let critical = ColorId::critical();
        let m1 = healthy.merge_worst(critical);
        let m2 = critical.merge_worst(healthy);
        assert_eq!(m1.red, m2.red);
        assert_eq!(m1.green, m2.green);
        assert_eq!(m1.blue, m2.blue);
    }

    // ── apply_improvement ─────────────────────────────────────────────────

    #[test]
    fn apply_improvement_reduces_red() {
        let sick = ColorId::new(200, 50, 50);
        let better = sick.apply_improvement(0.5);
        assert!(better.red < sick.red);
        assert!(better.green > sick.green);
        assert!(better.blue > sick.blue);
    }

    #[test]
    fn full_improvement_reaches_healthy_range() {
        let sick = ColorId::new(200, 50, 50);
        let fixed = sick.apply_improvement(1.0);
        assert_eq!(fixed.red, 0);
        assert_eq!(fixed.green, 255);
        assert_eq!(fixed.blue, 255);
    }

    #[test]
    fn zero_improvement_is_identity() {
        let c = ColorId::new(100, 100, 100);
        let same = c.apply_improvement(0.0);
        assert_eq!(same.red, 100);
        assert_eq!(same.green, 100);
        assert_eq!(same.blue, 100);
    }

    // ── Display ──────────────────────────────────────────────────────────

    #[test]
    fn display_contains_rgb_values() {
        let c = ColorId::new(10, 220, 200);
        let s = format!("{c}");
        assert!(s.contains("10") && s.contains("220") && s.contains("200"));
    }

    #[test]
    fn status_labels_cover_all_tiers() {
        assert_eq!(ColorId::healthy().status_label(), "Golden");
        assert_eq!(ColorId::critical().status_label(), "Dead");
    }

    // ── ColorIdBuilder ────────────────────────────────────────────────────

    #[test]
    fn builder_empty_returns_healthy() {
        assert_eq!(ColorIdBuilder::new().build(), ColorId::healthy());
    }

    #[test]
    fn builder_merges_worst() {
        let mut b = ColorIdBuilder::new();
        b.add(LevelColorContribution::new(
            "L1",
            ColorId::new(50, 200, 200),
            "ok",
        ));
        b.add(LevelColorContribution::new(
            "L4",
            ColorId::new(220, 80, 30),
            "loop",
        ));
        let r = b.build();
        assert_eq!(r.red, 220);
        assert_eq!(r.green, 80);
        assert_eq!(r.blue, 30);
    }

    #[test]
    fn builder_contribution_count() {
        let mut b = ColorIdBuilder::new();
        b.add(LevelColorContribution::new("L1", ColorId::healthy(), ""));
        b.add(LevelColorContribution::new("L2", ColorId::healthy(), ""));
        assert_eq!(b.contribution_count(), 2);
    }
}
