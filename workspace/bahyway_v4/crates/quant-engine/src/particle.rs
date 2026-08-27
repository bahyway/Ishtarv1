//! AssetParticle — an asset as a first-class BahyWay sovereign particle.
//!
//! Every asset is a particle with:
//!   KAKI nucleus    — immutable identity (asset_hash = lower 32 bits)
//!   EAV orbit       — mutable state (price, returns, factor scores, B11)
//!   Quality lane    — GEM/TRIBE/ACTIVE/FUZZY/DEAD from composite B11
//!   Centroid drift  — asset's factor vector relative to universe centroid
//!
//! The H(P) quality equation governs B11:
//!   H(P) = 1 / (1 + √Σ wᵢ(Pᵢ − Tᵢ)²)
//!   B11  = round(H(P) × 240)   ← ADR-001, always 240, NEVER 255

use crate::factor::FactorScores;
use crate::risk::RiskReport;

// ── Sovereign lane constants (ADR-001 + ADR-008) ─────────────────────────────
pub const GEM_B11: u8 = 200;
pub const TRIBE_B11: u8 = 140;
pub const ACTIVE_B11: u8 = 100;
pub const FUZZY_B11: u8 = 60;

/// Sovereign quality lane for an asset particle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetQualityLane {
    /// B11 ≥ 200 — strong buy signal, eligible to update universe centroid.
    Gem,
    /// B11 140–199 — quality asset, holds position.
    Tribe,
    /// B11 100–139 — below tribe, watch closely.
    Active,
    /// B11 60–99 — underperforming, consider reducing.
    Fuzzy,
    /// B11 < 60 — avoid or liquidate.
    Dead,
}

impl AssetQualityLane {
    pub fn from_b11(b11: u8) -> Self {
        match b11 {
            b if b >= GEM_B11 => Self::Gem,
            b if b >= TRIBE_B11 => Self::Tribe,
            b if b >= ACTIVE_B11 => Self::Active,
            b if b >= FUZZY_B11 => Self::Fuzzy,
            _ => Self::Dead,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Gem => "GEM",
            Self::Tribe => "TRIBE",
            Self::Active => "ACTIVE",
            Self::Fuzzy => "FUZZY",
            Self::Dead => "DEAD",
        }
    }

    pub fn is_investable(self) -> bool {
        matches!(self, Self::Gem | Self::Tribe | Self::Active)
    }

    pub fn is_gem(self) -> bool {
        matches!(self, Self::Gem)
    }
}

/// An asset as a sovereign BahyWay particle.
///
/// Holds the latest factor scores, risk metrics, B11 quality score,
/// and the resulting quality lane.  Updated each epoch from fresh data.
#[derive(Debug, Clone)]
pub struct AssetParticle {
    /// Sovereign 32-bit asset identity (lower 32 bits of KAKI uuid_hash).
    pub asset_hash: u32,
    /// Current epoch.
    pub epoch: u32,
    /// Latest close price (fixed-point).
    pub latest_price: i64,
    /// Factor scores (5-factor model).
    pub factors: FactorScores,
    /// Risk report (Sharpe, VaR, drawdown, etc.).
    pub risk: RiskReport,
    /// Composite B11 quality score (ADR-001: × 240, never × 255).
    pub b11: u8,
    /// Quality lane derived from B11.
    pub lane: AssetQualityLane,
    /// H(P) quality score [0.0–1.0] = b11 / 240.0.
    pub hp: f64,
}

impl AssetParticle {
    /// Construct an asset particle from factor scores and risk report.
    pub fn new(
        asset_hash: u32,
        epoch: u32,
        latest_price: i64,
        factors: FactorScores,
        risk: RiskReport,
    ) -> Self {
        let b11 = factors.b11();
        let lane = AssetQualityLane::from_b11(b11);
        let hp = b11 as f64 / 240.0;
        Self {
            asset_hash,
            epoch,
            latest_price,
            factors,
            risk,
            b11,
            lane,
            hp,
        }
    }

    /// One-line sovereign summary for Dashboard and lineage annotation.
    pub fn summary(&self) -> String {
        format!(
            "[ASSET {:08X}] lane={} B11={} H(P)={:.3} Sharpe={:.3} MaxDD={:.2}%",
            self.asset_hash,
            self.lane.label(),
            self.b11,
            self.hp,
            self.risk.sharpe,
            self.risk.max_drawdown * 100.0,
        )
    }

    /// True if this asset should update the universe centroid (GEM lane).
    pub fn updates_centroid(&self) -> bool {
        self.lane.is_gem()
    }
}

/// Universe centroid — the quality ideal point for the asset universe.
///
/// Only GEM-lane assets update this centroid (same self-calibrating rule
/// as the VGCA DomainCentroid in data-cleansing-station).
#[derive(Debug, Clone)]
pub struct UniverseCentroid {
    /// Running mean of 5 factor scores across GEM-lane assets.
    dims: [f64; 5],
    gem_count: u64,
    weight: u64,
}

impl UniverseCentroid {
    pub fn new() -> Self {
        Self {
            dims: [1.0; 5],
            gem_count: 0,
            weight: 0,
        }
    }

    /// Update with a GEM-lane asset's factor scores.
    /// **Must only be called when `asset.lane == AssetQualityLane::Gem`.**
    pub fn update_with_gem(&mut self, factors: &FactorScores) {
        let n = self.weight as f64;
        for i in 0..5 {
            self.dims[i] = (self.dims[i] * n + factors.scores[i]) / (n + 1.0);
        }
        self.weight += 1;
        self.gem_count += 1;
    }

    /// Euclidean distance from an asset's factor scores to this centroid.
    pub fn distance(&self, factors: &FactorScores) -> f64 {
        factors
            .scores
            .iter()
            .zip(self.dims.iter())
            .map(|(s, c)| (s - c).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Geometric fit score = 1 / (1 + distance).  Same formula as VGCA.
    pub fn geometric_fit(&self, factors: &FactorScores) -> f64 {
        1.0 / (1.0 + self.distance(factors))
    }

    pub fn gem_count(&self) -> u64 {
        self.gem_count
    }
    pub fn dims(&self) -> &[f64; 5] {
        &self.dims
    }
}

impl Default for UniverseCentroid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::risk::RiskReport;

    fn make_particle(b11_target: u8) -> AssetParticle {
        let target = b11_target as f64 / 240.0;
        let fsv = [target; 7];
        let factors = FactorScores::from_fsv(0x1234, 1, &fsv);
        let risk = RiskReport::from_returns(&[0.01, -0.01, 0.02], 0.95);
        AssetParticle::new(0x1234, 1, 10000, factors, risk)
    }

    #[test]
    fn lane_thresholds_correct() {
        assert_eq!(AssetQualityLane::from_b11(240), AssetQualityLane::Gem);
        assert_eq!(AssetQualityLane::from_b11(200), AssetQualityLane::Gem);
        assert_eq!(AssetQualityLane::from_b11(199), AssetQualityLane::Tribe);
        assert_eq!(AssetQualityLane::from_b11(139), AssetQualityLane::Active);
        assert_eq!(AssetQualityLane::from_b11(99), AssetQualityLane::Fuzzy);
        assert_eq!(AssetQualityLane::from_b11(59), AssetQualityLane::Dead);
    }

    #[test]
    fn gem_asset_updates_centroid() {
        let p = make_particle(210);
        assert!(p.updates_centroid());
    }

    #[test]
    fn dead_asset_does_not_update_centroid() {
        let p = make_particle(40);
        assert!(!p.updates_centroid());
    }

    #[test]
    fn hp_equals_b11_over_240() {
        let p = make_particle(120);
        assert!((p.hp - p.b11 as f64 / 240.0).abs() < 1e-9);
    }

    #[test]
    fn summary_contains_lane_and_b11() {
        let p = make_particle(200);
        let s = p.summary();
        assert!(s.contains("GEM"), "summary={s}");
        assert!(s.contains("B11="), "summary={s}");
    }

    #[test]
    fn investable_lanes_correct() {
        assert!(AssetQualityLane::Gem.is_investable());
        assert!(AssetQualityLane::Tribe.is_investable());
        assert!(AssetQualityLane::Active.is_investable());
        assert!(!AssetQualityLane::Fuzzy.is_investable());
        assert!(!AssetQualityLane::Dead.is_investable());
    }

    #[test]
    fn centroid_updates_from_gem() {
        let mut centroid = UniverseCentroid::new();
        let fsv = [0.9f64; 7];
        let scores = FactorScores::from_fsv(0x01, 1, &fsv);
        centroid.update_with_gem(&scores);
        assert_eq!(centroid.gem_count(), 1);
        let dist = centroid.distance(&scores);
        assert!(dist < 0.1, "after update, distance should be small: {dist}");
    }

    #[test]
    fn geometric_fit_perfect_gem_close_to_one() {
        let mut centroid = UniverseCentroid::new();
        let fsv = [1.0f64; 7];
        let scores = FactorScores::from_fsv(0x01, 1, &fsv);
        for _ in 0..10 {
            centroid.update_with_gem(&scores);
        }
        let fit = centroid.geometric_fit(&scores);
        assert!(fit > 0.99, "fit={fit}");
    }

    #[test]
    fn all_lane_labels_non_empty() {
        for lane in [
            AssetQualityLane::Gem,
            AssetQualityLane::Tribe,
            AssetQualityLane::Active,
            AssetQualityLane::Fuzzy,
            AssetQualityLane::Dead,
        ] {
            assert!(!lane.label().is_empty());
        }
    }
}
