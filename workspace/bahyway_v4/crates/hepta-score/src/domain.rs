//! 𒁾 Hepta Domain Types
//!
//! Core data structures for 7D sovereign quality scoring:
//!   HeptaDimension — the 7 Ibn Wahshiyya quality dimensions
//!   HeptaVector    — a 7D quality score vector for one data particle
//!   TribeIdealPoint — the tribe ideal T + weight vector
//!   HeptaScore     — full scoring result (H(P), B11, lane, contributions)
//!   QualityLane    — orbital lane derived from B11
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::equation::hepta_health_score;
use crate::errors::HeptaError;
use crate::weights::SOVEREIGN_ARABIC_MDM_WEIGHTS;
use crate::{ACTIVE_B11, FUZZY_DEAD_BOUNDARY, GEM_B11, QUALITY_DIVISOR, TRIBE_B11};

// ── HeptaDimension ────────────────────────────────────────────────────────────

/// The 7 Ibn Wahshiyya data quality dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeptaDimension {
    /// D1 — Arabic name/text correctness (CleansingWay S3)
    Accuracy = 0,
    /// D2 — Missing field rate (KAKI Issuance S1)
    Completeness = 1,
    /// D3 — Cross-record contradiction score (CompareWay S2)
    Consistency = 2,
    /// D4 — Format compliance: NID, phone, coordinates (KAKI S1)
    Validity = 3,
    /// D5 — Deduplication confidence (CompareWay S2)
    Uniqueness = 4,
    /// D6 — Record/file freshness (VaultGate S0)
    Timeliness = 5,
    /// D7 — EAV referential completeness (EnkiWay S7)
    Integrity = 6,
}

impl HeptaDimension {
    pub const ALL: [Self; 7] = [
        Self::Accuracy,
        Self::Completeness,
        Self::Consistency,
        Self::Validity,
        Self::Uniqueness,
        Self::Timeliness,
        Self::Integrity,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Accuracy => "Accuracy",
            Self::Completeness => "Completeness",
            Self::Consistency => "Consistency",
            Self::Validity => "Validity",
            Self::Uniqueness => "Uniqueness",
            Self::Timeliness => "Timeliness",
            Self::Integrity => "Integrity",
        }
    }

    /// Ibn Wahshiyya planetary symbol mapping.
    pub fn planet_symbol(self) -> &'static str {
        match self {
            Self::Accuracy => "ME-Saturn",
            Self::Completeness => "GU-Jupiter",
            Self::Consistency => "SAG-Mars",
            Self::Validity => "IZ-Sun",
            Self::Uniqueness => "A-Venus",
            Self::Timeliness => "UD-Mercury",
            Self::Integrity => "URU-Moon",
        }
    }

    /// BeeMDM pipeline station that produces this dimension's score.
    pub fn pipeline_station(self) -> &'static str {
        match self {
            Self::Accuracy => "S3 CleansingWay",
            Self::Completeness => "S1 KAKI Issuance",
            Self::Consistency => "S2 CompareWay",
            Self::Validity => "S1 KAKI Issuance",
            Self::Uniqueness => "S2 CompareWay",
            Self::Timeliness => "S0 VaultGate",
            Self::Integrity => "S7 EnkiWay",
        }
    }
}

// ── HeptaVector ───────────────────────────────────────────────────────────────

/// A data particle represented as a 7D quality vector.
///
/// Each dimension is a normalised float in `[0.0, 1.0]` where `1.0` is
/// perfect quality. Values are always produced by BeeMDM pipeline stations —
/// never set manually at intake.
#[derive(Debug, Clone, Copy)]
pub struct HeptaVector {
    dims: [f32; 7],
}

impl HeptaVector {
    /// Construct from 7 dimension scores.
    ///
    /// Returns `HeptaError::DimensionOutOfRange` if any value is outside `[0.0, 1.0]`.
    pub fn try_new(
        accuracy: f32,
        completeness: f32,
        consistency: f32,
        validity: f32,
        uniqueness: f32,
        timeliness: f32,
        integrity: f32,
    ) -> Result<Self, HeptaError> {
        let dims = [
            accuracy,
            completeness,
            consistency,
            validity,
            uniqueness,
            timeliness,
            integrity,
        ];
        for (i, &v) in dims.iter().enumerate() {
            if !(0.0..=1.0).contains(&v) {
                return Err(HeptaError::DimensionOutOfRange { dim: i, value: v });
            }
        }
        Ok(Self { dims })
    }

    /// Construct without bounds checking. Panics in debug if any value is out of range.
    pub fn new(
        accuracy: f32,
        completeness: f32,
        consistency: f32,
        validity: f32,
        uniqueness: f32,
        timeliness: f32,
        integrity: f32,
    ) -> Self {
        let dims = [
            accuracy,
            completeness,
            consistency,
            validity,
            uniqueness,
            timeliness,
            integrity,
        ];
        debug_assert!(
            dims.iter().all(|&v| (0.0..=1.0).contains(&v)),
            "HeptaVector dimensions must be in [0.0, 1.0]"
        );
        Self { dims }
    }

    /// Perfect particle — all dimensions = 1.0.
    pub fn perfect() -> Self {
        Self { dims: [1.0; 7] }
    }

    /// Zero particle — all dimensions = 0.0 (worst possible quality).
    pub fn zero() -> Self {
        Self { dims: [0.0; 7] }
    }

    /// Construct from a raw array.
    pub fn from_array(dims: [f32; 7]) -> Result<Self, HeptaError> {
        for (i, &v) in dims.iter().enumerate() {
            if !(0.0..=1.0).contains(&v) {
                return Err(HeptaError::DimensionOutOfRange { dim: i, value: v });
            }
        }
        Ok(Self { dims })
    }

    pub fn dimensions(&self) -> &[f32; 7] {
        &self.dims
    }

    pub fn get(&self, dim: HeptaDimension) -> f32 {
        self.dims[dim as usize]
    }

    pub fn set(&mut self, dim: HeptaDimension, value: f32) -> Result<(), HeptaError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(HeptaError::DimensionOutOfRange {
                dim: dim as usize,
                value,
            });
        }
        self.dims[dim as usize] = value;
        Ok(())
    }

    /// Compute H(P) health score against a tribe ideal point.
    pub fn health_score(&self, tribe: &TribeIdealPoint) -> f32 {
        hepta_health_score(&self.dims, tribe.ideal(), tribe.weights())
    }

    /// Derive the sovereign B11 quality byte: `B11 = round(H(P) × 240.0)`.
    pub fn to_b11(&self, tribe: &TribeIdealPoint) -> u8 {
        (self.health_score(tribe) * QUALITY_DIVISOR)
            .round()
            .clamp(0.0, 240.0) as u8
    }

    /// Compute the full `HeptaScore` (health, lane, B11, per-dimension analysis).
    pub fn score(&self, tribe: &TribeIdealPoint) -> HeptaScore {
        HeptaScore::compute(self, tribe)
    }

    /// Returns the weakest dimension (lowest score).
    pub fn weakest_dimension(&self) -> HeptaDimension {
        let (idx, _) = self
            .dims
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap_or((0, &0.0));
        HeptaDimension::ALL[idx]
    }

    /// Euclidean distance from the tribe ideal (un-weighted).
    pub fn raw_distance(&self, tribe: &TribeIdealPoint) -> f32 {
        self.dims
            .iter()
            .zip(tribe.ideal().iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Weighted distance √Σ wᵢ(Pᵢ−Tᵢ)².
    pub fn weighted_distance(&self, tribe: &TribeIdealPoint) -> f32 {
        self.dims
            .iter()
            .zip(tribe.ideal().iter())
            .zip(tribe.weights().iter())
            .map(|((p, t), w)| w * (p - t).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

impl std::ops::Index<usize> for HeptaVector {
    type Output = f32;
    fn index(&self, i: usize) -> &Self::Output {
        &self.dims[i]
    }
}

// ── TribeIdealPoint ───────────────────────────────────────────────────────────

/// The tribe ideal point T — the centroid of perfect quality.
///
/// Ideal is always `[1.0; 7]`. Weights encode domain-specific quality priorities
/// and must sum to 1.0.
#[derive(Debug, Clone)]
pub struct TribeIdealPoint {
    ideal: [f32; 7],
    weights: [f32; 7],
    pub name: String,
}

impl TribeIdealPoint {
    /// Create with explicit ideal values and weights.
    pub fn new(
        ideal: [f32; 7],
        weights: [f32; 7],
        name: impl Into<String>,
    ) -> Result<Self, HeptaError> {
        let sum: f32 = weights.iter().sum();
        if (sum - 1.0).abs() > 0.001 {
            return Err(HeptaError::WeightsDoNotSumToOne(sum));
        }
        for &v in &ideal {
            if !(0.0..=1.0).contains(&v) {
                return Err(HeptaError::InvalidIdealPoint);
            }
        }
        Ok(Self {
            ideal,
            weights,
            name: name.into(),
        })
    }

    /// Sovereign Arabic MDM weight profile — the BahyWay default.
    ///
    /// `[0.30, 0.20, 0.15, 0.15, 0.10, 0.05, 0.05]`
    /// Accuracy (Arabic name correctness) weighted highest.
    pub fn sovereign_arabic_mdm() -> Self {
        Self {
            ideal: [1.0; 7],
            weights: SOVEREIGN_ARABIC_MDM_WEIGHTS,
            name: "Sovereign Arabic MDM".into(),
        }
    }

    /// Equal weights (1/7 each) — for testing and domain-agnostic scoring.
    pub fn equal_weights() -> Self {
        Self {
            ideal: [1.0; 7],
            weights: [1.0 / 7.0; 7],
            name: "Equal Weights".into(),
        }
    }

    /// Environmental MDM — Validity (sensor range) and Timeliness (freshness) prioritized.
    pub fn environmental_mdm() -> Self {
        Self {
            ideal: [1.0; 7],
            weights: [0.20, 0.15, 0.10, 0.25, 0.10, 0.15, 0.05],
            name: "Environmental MDM".into(),
        }
    }

    /// Government citizen identity MDM — Uniqueness and Validity highest (NID dedup).
    pub fn government_identity() -> Self {
        Self {
            ideal: [1.0; 7],
            weights: [0.25, 0.20, 0.15, 0.20, 0.15, 0.03, 0.02],
            name: "Government Identity MDM".into(),
        }
    }

    pub fn ideal(&self) -> &[f32; 7] {
        &self.ideal
    }
    pub fn weights(&self) -> &[f32; 7] {
        &self.weights
    }

    pub fn weight_for(&self, dim: HeptaDimension) -> f32 {
        self.weights[dim as usize]
    }

    pub fn ideal_for(&self, dim: HeptaDimension) -> f32 {
        self.ideal[dim as usize]
    }
}

// ── QualityLane ───────────────────────────────────────────────────────────────

/// Orbital lane derived from the B11 quality byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLane {
    /// B11 ≥ 200 — Golden Record, inner orbit
    Gem,
    /// B11 140–199 — Tribe member, mid orbit
    TribeMember,
    /// B11 100–139 — Active, outer orbit
    Active,
    /// B11 59–99 — Fuzzy, DataSteward queue
    Fuzzy,
    /// B11 < 59 — Dead, exits to EnkiWay dead grid
    Dead,
}

impl QualityLane {
    pub fn from_b11(b11: u8) -> Self {
        if b11 >= GEM_B11 {
            Self::Gem
        } else if b11 >= TRIBE_B11 {
            Self::TribeMember
        } else if b11 >= ACTIVE_B11 {
            Self::Active
        } else if b11 >= FUZZY_DEAD_BOUNDARY {
            Self::Fuzzy
        } else {
            Self::Dead
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Gem => "GEM",
            Self::TribeMember => "TRIBE",
            Self::Active => "ACTIVE",
            Self::Fuzzy => "FUZZY",
            Self::Dead => "DEAD",
        }
    }

    /// Orbital radius factor (Gem = tightest orbit = 1.0).
    pub fn orbital_radius_factor(self) -> f32 {
        match self {
            Self::Gem => 1.0,
            Self::TribeMember => 1.8,
            Self::Active => 2.5,
            Self::Fuzzy => 3.2,
            Self::Dead => 4.0,
        }
    }
}

// ── HeptaScore ────────────────────────────────────────────────────────────────

/// Complete scoring result for one data particle.
#[derive(Debug, Clone)]
pub struct HeptaScore {
    /// H(P) health score in (0.0, 1.0]
    pub health: f32,
    /// Sovereign B11 quality byte (0–240)
    pub b11: u8,
    /// Quality lane derived from B11
    pub lane: QualityLane,
    /// Weighted distance from tribe ideal
    pub weighted_dist: f32,
    /// Per-dimension scores
    pub dimensions: [f32; 7],
    /// Weighted contribution of each dimension to total distance
    pub dim_contributions: [f32; 7],
    /// Weakest dimension (fix this first)
    pub weakest: HeptaDimension,
    /// Strongest dimension (closest to ideal)
    pub strongest: HeptaDimension,
    /// Remediation priority = weakest dimension
    pub fix_priority: HeptaDimension,
}

impl HeptaScore {
    pub fn compute(p: &HeptaVector, tribe: &TribeIdealPoint) -> Self {
        let dims = *p.dimensions();
        let ideal = *tribe.ideal();
        let w = *tribe.weights();

        let dim_contributions: [f32; 7] =
            std::array::from_fn(|i| w[i] * (dims[i] - ideal[i]).powi(2));

        let weighted_dist = dim_contributions.iter().sum::<f32>().sqrt();
        let health = 1.0 / (1.0 + weighted_dist);
        let b11 = (health * QUALITY_DIVISOR).round().clamp(0.0, 240.0) as u8;
        let lane = QualityLane::from_b11(b11);

        let weakest = dim_contributions
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| HeptaDimension::ALL[i])
            .unwrap_or(HeptaDimension::Accuracy);

        let strongest = dim_contributions
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| HeptaDimension::ALL[i])
            .unwrap_or(HeptaDimension::Integrity);

        Self {
            health,
            b11,
            lane,
            weighted_dist,
            dimensions: dims,
            dim_contributions,
            weakest,
            strongest,
            fix_priority: weakest,
        }
    }

    pub fn is_gem(&self) -> bool {
        self.lane == QualityLane::Gem
    }
    pub fn is_dead(&self) -> bool {
        self.lane == QualityLane::Dead
    }

    /// Quality percentage (0–100 %)
    pub fn quality_pct(&self) -> f32 {
        self.b11 as f32 / QUALITY_DIVISOR * 100.0
    }

    /// Orbital radius for visualization: Gem=1.0, Dead=4.0
    pub fn orbital_radius(&self) -> f32 {
        let b11_norm = self.b11 as f32 / QUALITY_DIVISOR;
        4.0 - b11_norm * 3.0
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn tribe() -> TribeIdealPoint {
        TribeIdealPoint::sovereign_arabic_mdm()
    }

    #[test]
    fn perfect_vector_gives_b11_240() {
        let b11 = HeptaVector::perfect().to_b11(&tribe());
        assert_eq!(b11, 240);
    }

    #[test]
    fn perfect_vector_is_gem() {
        let score = HeptaVector::perfect().score(&tribe());
        assert!(score.is_gem());
    }

    #[test]
    fn zero_vector_is_active() {
        // T=[1;7], Σwᵢ=1 → d_max=1 → H_min=0.5 → B11=120 → Active
        let score = HeptaVector::zero().score(&tribe());
        assert_eq!(score.lane, QualityLane::Active, "B11={}", score.b11);
    }

    #[test]
    fn try_new_rejects_out_of_range() {
        assert!(HeptaVector::try_new(1.1, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5).is_err());
        assert!(HeptaVector::try_new(-0.1, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5).is_err());
    }

    #[test]
    fn try_new_accepts_boundary_values() {
        assert!(HeptaVector::try_new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0).is_ok());
        assert!(HeptaVector::try_new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0).is_ok());
    }

    #[test]
    fn sovereign_weights_sum_to_one() {
        let t = TribeIdealPoint::sovereign_arabic_mdm();
        let sum: f32 = t.weights().iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "sum={sum}");
    }

    #[test]
    fn quality_lane_boundaries() {
        assert_eq!(QualityLane::from_b11(240), QualityLane::Gem);
        assert_eq!(QualityLane::from_b11(200), QualityLane::Gem);
        assert_eq!(QualityLane::from_b11(199), QualityLane::TribeMember);
        assert_eq!(QualityLane::from_b11(140), QualityLane::TribeMember);
        assert_eq!(QualityLane::from_b11(139), QualityLane::Active);
        assert_eq!(QualityLane::from_b11(100), QualityLane::Active);
        assert_eq!(QualityLane::from_b11(99), QualityLane::Fuzzy);
        assert_eq!(QualityLane::from_b11(59), QualityLane::Fuzzy);
        assert_eq!(QualityLane::from_b11(58), QualityLane::Dead);
        assert_eq!(QualityLane::from_b11(0), QualityLane::Dead);
    }

    #[test]
    fn lane_orbital_radius_ordering() {
        assert!(
            QualityLane::Gem.orbital_radius_factor()
                < QualityLane::TribeMember.orbital_radius_factor()
        );
        assert!(
            QualityLane::Dead.orbital_radius_factor() > QualityLane::Active.orbital_radius_factor()
        );
    }

    #[test]
    fn weakest_dimension_finds_lowest() {
        let v = HeptaVector::new(1.0, 1.0, 1.0, 1.0, 0.3, 1.0, 1.0);
        assert_eq!(v.weakest_dimension(), HeptaDimension::Uniqueness);
    }

    #[test]
    fn set_dimension_updates_value() {
        let mut v = HeptaVector::perfect();
        v.set(HeptaDimension::Accuracy, 0.5).unwrap();
        assert!((v.get(HeptaDimension::Accuracy) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn dimension_labels_non_empty() {
        for d in HeptaDimension::ALL {
            assert!(!d.label().is_empty());
        }
    }

    #[test]
    fn environmental_mdm_weights_valid() {
        let t = TribeIdealPoint::environmental_mdm();
        let sum: f32 = t.weights().iter().sum();
        assert!((sum - 1.0).abs() < 0.001, "sum={sum}");
    }
}
