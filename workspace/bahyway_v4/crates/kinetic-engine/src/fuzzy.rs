//! # Fuzzy — Sovereign Fuzzy Logic and Scoring Engine
//!
//! Implements the fuzzy membership functions and weighted scoring engine
//! that map particle positions in 7D Heptagon space to health scores
//! and sovereignty classifications.
//!
//! ## The Health Orbit Threshold
//!
//! The configurable threshold `τ` in `[0.0, 1.0]` determines when a particle
//! is considered sovereign:
//!
//! - `τ = 0.50` — Relaxed (exploratory analytics)
//! - `τ = 0.70` — Standard (DMW default SQL optimization)
//! - `τ = 0.90` — Strict (financial / medical data quality)
//! - `τ = 0.99` — Sovereign (najaf-engine grave-level precision)
//!
//! ## The Three Classifications
//!
//! Every particle at every point in its 7D journey is classified as:
//! - [`HealthClassification::Sovereign`]  — above threshold, healthy orbit
//! - [`HealthClassification::Warning`]    — between 60%-100% of threshold
//! - [`HealthClassification::Critical`]   — below 60% of threshold, dead zone candidate

use crate::force::HeptaDimension;
use crate::vec7d::Vec7D;

// ── HealthClassification ────────────────────────────────────────────────────

/// The three sovereign health states of a particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthClassification {
    /// Score ≥ threshold — healthy orbit, σ collapsing toward sovereign
    Sovereign,
    /// Score in [60%, 100%) of threshold — approaching threshold
    Warning,
    /// Score < 60% of threshold — dead zone candidate
    Critical,
}

impl HealthClassification {
    pub fn is_sovereign(self) -> bool { self == Self::Sovereign }
    pub fn is_warning(self)  -> bool { self == Self::Warning }
    pub fn is_critical(self) -> bool { self == Self::Critical }
}

// ── MembershipFunction ──────────────────────────────────────────────────────

/// A triangular fuzzy membership function.
///
/// Maps a scalar value to a membership degree in `[0.0, 1.0]`:
/// - Below `lower_bound` → 0.0 (no membership)
/// - At `peak`           → 1.0 (full membership)
/// - Above `upper_bound` → 0.0 (no membership)
#[derive(Debug, Clone)]
pub struct MembershipFunction {
    pub lower_bound: f64,
    pub peak:        f64,
    pub upper_bound: f64,
}

impl MembershipFunction {
    pub fn new(lower_bound: f64, peak: f64, upper_bound: f64) -> Self {
        assert!(lower_bound <= peak,   "lower_bound must be <= peak");
        assert!(peak <= upper_bound,   "peak must be <= upper_bound");
        Self { lower_bound, peak, upper_bound }
    }

    /// Evaluate membership for a scalar value. Returns a degree in `[0.0, 1.0]`.
    pub fn evaluate(&self, value: f64) -> f64 {
        if value <= self.lower_bound || value >= self.upper_bound {
            return 0.0;
        }
        if (value - self.peak).abs() < f64::EPSILON {
            return 1.0;
        }
        if value < self.peak {
            (value - self.lower_bound) / (self.peak - self.lower_bound)
        } else {
            (self.upper_bound - value) / (self.upper_bound - self.peak)
        }
    }

    /// Full health membership — peaks at 1.0, nearly zero below 0.5
    pub fn sovereign_health() -> Self {
        Self::new(0.5, 1.0, 1.001)
    }

    /// Warning zone — peaks at the threshold boundary
    pub fn warning_zone(threshold: f64) -> Self {
        let lo = threshold * 0.4;
        let hi = threshold * 1.1;
        Self::new(lo, threshold, hi)
    }

    /// Critical zone — peaks at 0.0, decays toward 0.4
    pub fn critical_zone() -> Self {
        Self::new(-0.001, 0.0, 0.4)
    }

    /// Gray-Rot onset — peaks at 20% fragmentation threshold
    pub fn gray_rot_onset() -> Self {
        Self::new(0.0, 0.20, 0.50)
    }

    /// Cardinality error — peaks at 10x estimation error
    pub fn cardinality_error() -> Self {
        Self::new(1.0, 10.0, 1000.0)
    }
}

// ── FuzzyRule ───────────────────────────────────────────────────────────────

/// A fuzzy rule evaluating one dimension of the particle's 7D position.
///
/// ## DMW Default Weight Distribution
///
/// | Dimension | Weight | Rationale |
/// |-----------|--------|-----------|
/// | URU (Index) | 0.20 | Index health directly impacts all query performance |
/// | A (IO)      | 0.20 | IO friction is the primary bottleneck in most SQL queries |
/// | GU (Card)   | 0.15 | Wrong cardinality estimates cause wrong plan choices |
/// | UD (Cache)  | 0.15 | Parameter sniffing causes correct plans to be reused wrongly |
/// | ME (Intent) | 0.15 | Cartesian products are catastrophic but less frequent |
/// | SAG (Sel)   | 0.10 | Non-sargable predicates are common and fixable |
/// | IZI (CPU)   | 0.05 | CPU issues appear after IO is resolved |
#[derive(Debug, Clone)]
pub struct FuzzyRule {
    pub dimension:  HeptaDimension,
    pub membership: MembershipFunction,
    pub weight:     f64,
}

impl FuzzyRule {
    pub fn new(
        dimension:  HeptaDimension,
        membership: MembershipFunction,
        weight:     f64,
    ) -> Self {
        Self {
            dimension,
            membership,
            weight: weight.clamp(0.0, 1.0),
        }
    }

    /// Evaluate this rule for the given particle position.
    /// Returns the weighted membership degree in `[0.0, weight]`.
    pub fn evaluate(&self, particle: &Vec7D) -> f64 {
        let value = self.dimension_value(particle);
        self.membership.evaluate(value) * self.weight
    }

    fn dimension_value(&self, particle: &Vec7D) -> f64 {
        match self.dimension {
            HeptaDimension::ME  => particle.me,
            HeptaDimension::GU  => particle.gu,
            HeptaDimension::SAG => particle.sag,
            HeptaDimension::A   => particle.a,
            HeptaDimension::IZI => particle.izi,
            HeptaDimension::UD  => particle.ud,
            HeptaDimension::URU => particle.uru,
        }
    }
}

// ── ScoringEngine ───────────────────────────────────────────────────────────

/// The Sovereign Scoring Engine.
///
/// Aggregates fuzzy rule activations into a single health score in `[0.0, 1.0]`,
/// then classifies the particle against the configurable Health Orbit Threshold.
pub struct ScoringEngine {
    pub rules:     Vec<FuzzyRule>,
    pub threshold: f64,
}

impl ScoringEngine {
    pub fn new(threshold: f64) -> Self {
        Self {
            rules:     Vec::new(),
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    pub fn add_rule(&mut self, rule: FuzzyRule) {
        self.rules.push(rule);
    }

    /// Compute the weighted health score for a particle in `[0.0, 1.0]`.
    /// Uses normalized weighted average: `score = Σ(rule.evaluate(p)) / Σ(rule.weight)`
    pub fn score(&self, particle: &Vec7D) -> f64 {
        if self.rules.is_empty() {
            return 0.0;
        }
        let total_weight: f64 = self.rules.iter().map(|r| r.weight).sum();
        if total_weight < f64::EPSILON {
            return 0.0;
        }
        let weighted_sum: f64 = self.rules.iter().map(|r| r.evaluate(particle)).sum();
        (weighted_sum / total_weight).clamp(0.0, 1.0)
    }

    pub fn is_sovereign(&self, particle: &Vec7D) -> bool {
        self.score(particle) >= self.threshold
    }

    /// Sigma (σ) — `1 - score`
    pub fn sigma(&self, particle: &Vec7D) -> f64 {
        1.0 - self.score(particle)
    }

    pub fn classify(&self, particle: &Vec7D) -> HealthClassification {
        let s = self.score(particle);
        if s >= self.threshold {
            HealthClassification::Sovereign
        } else if s >= self.threshold * 0.60 {
            HealthClassification::Warning
        } else {
            HealthClassification::Critical
        }
    }

    /// The dominant failing dimension — routes Critical particles to the correct repair service.
    pub fn dominant_failure(&self, particle: &Vec7D) -> Option<HeptaDimension> {
        self.rules
            .iter()
            .min_by(|a, b| {
                let sa = a.evaluate(particle) / a.weight.max(f64::EPSILON);
                let sb = b.evaluate(particle) / b.weight.max(f64::EPSILON);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.dimension)
    }

    /// **DMW Default Engine** — standard SQL optimization thresholds (τ = 0.70).
    pub fn dmw_default() -> Self {
        let mut engine = Self::new(0.70);
        engine.add_rule(FuzzyRule::new(HeptaDimension::URU, MembershipFunction::sovereign_health(), 0.20));
        engine.add_rule(FuzzyRule::new(HeptaDimension::A,   MembershipFunction::sovereign_health(), 0.20));
        engine.add_rule(FuzzyRule::new(HeptaDimension::GU,  MembershipFunction::sovereign_health(), 0.15));
        engine.add_rule(FuzzyRule::new(HeptaDimension::UD,  MembershipFunction::sovereign_health(), 0.15));
        engine.add_rule(FuzzyRule::new(HeptaDimension::ME,  MembershipFunction::sovereign_health(), 0.15));
        engine.add_rule(FuzzyRule::new(HeptaDimension::SAG, MembershipFunction::sovereign_health(), 0.10));
        engine.add_rule(FuzzyRule::new(HeptaDimension::IZI, MembershipFunction::sovereign_health(), 0.05));
        engine
    }

    /// **Strict Engine** — financial / medical data quality (τ = 0.90)
    pub fn strict() -> Self {
        let mut engine = Self::new(0.90);
        for rule in Self::dmw_default().rules {
            engine.add_rule(rule);
        }
        engine
    }

    /// **Sovereign Engine** — najaf-engine grave-level precision (τ = 0.99)
    pub fn sovereign() -> Self {
        let mut engine = Self::new(0.99);
        for rule in Self::dmw_default().rules {
            engine.add_rule(rule);
        }
        engine
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn membership_at_peak_is_one() {
        let mf = MembershipFunction::new(0.0, 0.5, 1.0);
        assert!(approx(mf.evaluate(0.5), 1.0));
    }

    #[test]
    fn membership_below_lower_bound_is_zero() {
        let mf = MembershipFunction::new(0.3, 0.6, 0.9);
        assert!(approx(mf.evaluate(0.0), 0.0));
        assert!(approx(mf.evaluate(0.3), 0.0));
    }

    #[test]
    fn membership_above_upper_bound_is_zero() {
        let mf = MembershipFunction::new(0.3, 0.6, 0.9);
        assert!(approx(mf.evaluate(1.0), 0.0));
        assert!(approx(mf.evaluate(0.9), 0.0));
    }

    #[test]
    fn membership_midpoint_rising_slope_is_half() {
        let mf = MembershipFunction::new(0.0, 1.0, 2.0);
        assert!(approx(mf.evaluate(0.5), 0.5));
    }

    #[test]
    fn membership_midpoint_falling_slope_is_half() {
        let mf = MembershipFunction::new(0.0, 1.0, 2.0);
        assert!(approx(mf.evaluate(1.5), 0.5));
    }

    #[test]
    fn sovereign_health_mf_peaks_at_one() {
        let mf = MembershipFunction::sovereign_health();
        assert!(approx(mf.evaluate(1.0), 1.0));
    }

    #[test]
    fn sovereign_health_mf_is_zero_at_zero() {
        let mf = MembershipFunction::sovereign_health();
        assert!(approx(mf.evaluate(0.0), 0.0));
    }

    #[test]
    fn fuzzy_rule_evaluates_correct_dimension() {
        let rule = FuzzyRule::new(
            HeptaDimension::URU,
            MembershipFunction::new(0.0, 1.0, 1.001),
            1.0,
        );
        let particle = Vec7D::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        assert!(rule.evaluate(&particle) > 0.9,
            "should evaluate URU=1.0 as high: {}", rule.evaluate(&particle));
    }

    #[test]
    fn fuzzy_rule_weight_scales_output() {
        let rule_full = FuzzyRule::new(HeptaDimension::ME, MembershipFunction::new(0.0, 1.0, 1.001), 1.0);
        let rule_half = FuzzyRule::new(HeptaDimension::ME, MembershipFunction::new(0.0, 1.0, 1.001), 0.5);
        let particle  = Vec7D::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let v_full    = rule_full.evaluate(&particle);
        let v_half    = rule_half.evaluate(&particle);
        assert!(approx(v_full, v_half * 2.0),
            "weight=1.0 should be 2× weight=0.5: full={v_full}, half={v_half}");
    }

    #[test]
    fn scoring_engine_empty_rules_returns_zero() {
        let engine = ScoringEngine::new(0.70);
        assert!(approx(engine.score(&Vec7D::HEALTHY_ORBIT), 0.0));
    }

    #[test]
    fn scoring_engine_perfect_particle_scores_near_one() {
        let engine = ScoringEngine::dmw_default();
        let score  = engine.score(&Vec7D::HEALTHY_ORBIT);
        assert!(score > 0.95, "perfect particle should score near 1.0: {score}");
    }

    #[test]
    fn scoring_engine_zero_particle_scores_near_zero() {
        let engine = ScoringEngine::dmw_default();
        let score  = engine.score(&Vec7D::ZERO);
        assert!(score < 0.05, "zero particle should score near 0.0: {score}");
    }

    #[test]
    fn is_sovereign_true_above_threshold() {
        let engine = ScoringEngine::dmw_default();
        assert!(engine.is_sovereign(&Vec7D::HEALTHY_ORBIT));
    }

    #[test]
    fn is_sovereign_false_below_threshold() {
        let engine = ScoringEngine::dmw_default();
        assert!(!engine.is_sovereign(&Vec7D::ZERO));
    }

    #[test]
    fn sigma_is_complement_of_score() {
        let engine   = ScoringEngine::dmw_default();
        let particle = Vec7D::HEALTHY_ORBIT;
        let score    = engine.score(&particle);
        let sigma    = engine.sigma(&particle);
        assert!(approx(score + sigma, 1.0),
            "score + sigma must equal 1.0: score={score}, sigma={sigma}");
    }

    #[test]
    fn classify_sovereign_for_healthy_particle() {
        let engine = ScoringEngine::dmw_default();
        assert_eq!(engine.classify(&Vec7D::HEALTHY_ORBIT), HealthClassification::Sovereign);
    }

    #[test]
    fn classify_critical_for_zero_particle() {
        let engine = ScoringEngine::dmw_default();
        assert_eq!(engine.classify(&Vec7D::ZERO), HealthClassification::Critical);
    }

    #[test]
    fn dmw_default_has_seven_rules() {
        let engine = ScoringEngine::dmw_default();
        assert_eq!(engine.rules.len(), 7);
    }

    #[test]
    fn dmw_default_threshold_is_0_70() {
        let engine = ScoringEngine::dmw_default();
        assert!(approx(engine.threshold, 0.70));
    }

    #[test]
    fn strict_engine_threshold_is_0_90() {
        let engine = ScoringEngine::strict();
        assert!(approx(engine.threshold, 0.90));
    }

    #[test]
    fn sovereign_engine_threshold_is_0_99() {
        let engine = ScoringEngine::sovereign();
        assert!(approx(engine.threshold, 0.99));
    }

    #[test]
    fn dmw_default_weights_sum_to_one() {
        let engine = ScoringEngine::dmw_default();
        let total: f64 = engine.rules.iter().map(|r| r.weight).sum();
        assert!(approx(total, 1.0),
            "DMW default rule weights must sum to 1.0: sum={total}");
    }

    #[test]
    fn dominant_failure_identifies_weakest_dimension() {
        let engine   = ScoringEngine::dmw_default();
        let particle = Vec7D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0);
        let dominant = engine.dominant_failure(&particle);
        assert!(dominant.is_some());
        assert_eq!(dominant.unwrap(), HeptaDimension::URU,
            "URU should be dominant failure when URU=0.0 and all others=1.0");
    }

    #[test]
    fn health_classification_methods_are_exclusive() {
        let engine = ScoringEngine::dmw_default();
        let c      = engine.classify(&Vec7D::HEALTHY_ORBIT);
        let exclusive = [c.is_sovereign(), c.is_warning(), c.is_critical()]
            .iter()
            .filter(|&&b| b)
            .count();
        assert_eq!(exclusive, 1, "exactly one classification must be true");
    }
}
