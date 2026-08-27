//! particle_state.rs — Hepta 7D particle state for EaAgent evaluation.
#![forbid(unsafe_code)]

use crate::constants::{ABZU_PRECISION, GEM_B11, HEPTA_WEIGHTS, QUALITY_DIVISOR};

/// The 7-dimensional Hepta vector — the mathematical position of a particle.
/// Maps directly to KAKI dimensions D1..D7.
#[derive(Debug, Clone, PartialEq)]
pub struct HeptaVector {
    pub d: [f64; 7],
}

impl HeptaVector {
    pub fn new(d: [f64; 7]) -> Self {
        Self { d }
    }

    /// Zero vector — the void particle (Axiom PA-5).
    pub fn zero() -> Self {
        Self { d: [0.0; 7] }
    }

    /// Unit vector — all dimensions at maximum quality.
    pub fn unit() -> Self {
        Self { d: [1.0; 7] }
    }

    /// Weighted Euclidean distance to target vector T.
    /// Used in H(P) = 1 / (1 + √ Σᵢwᵢ(Pᵢ−Tᵢ)²)
    pub fn weighted_distance(&self, target: &HeptaVector) -> f64 {
        let sum: f64 = HEPTA_WEIGHTS
            .iter()
            .enumerate()
            .map(|(i, w)| w * (self.d[i] - target.d[i]).powi(2))
            .sum();
        sum.sqrt()
    }

    /// H(P) quality score ∈ [0, 1].
    pub fn quality_score(&self) -> f64 {
        let target = HeptaVector::unit();
        let dist = self.weighted_distance(&target);
        1.0 / (1.0 + dist)
    }

    /// B11 = round(H(P) × 240) ∈ [0, 240].
    pub fn b11(&self) -> u8 {
        (self.quality_score() * QUALITY_DIVISOR).round() as u8
    }

    /// Manhattan distance to another vector (for Pauli Exclusion).
    pub fn manhattan_distance(&self, other: &HeptaVector) -> f64 {
        self.d
            .iter()
            .zip(other.d.iter())
            .map(|(a, b)| (a - b).abs())
            .sum()
    }

    /// Cosine similarity to another vector ∈ [-1, 1].
    pub fn cosine_similarity(&self, other: &HeptaVector) -> f64 {
        let dot: f64 = self.d.iter().zip(other.d.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f64 = self.d.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = other.d.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm_a < ABZU_PRECISION || norm_b < ABZU_PRECISION {
            return 0.0;
        }
        (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
    }

    /// Weighted centroid of a collection of vectors.
    pub fn centroid(vectors: &[&HeptaVector]) -> HeptaVector {
        if vectors.is_empty() {
            return HeptaVector::zero();
        }
        let n = vectors.len() as f64;
        let mut d = [0.0f64; 7];
        for v in vectors {
            for (di, vi) in d.iter_mut().zip(v.d.iter()) {
                *di += vi;
            }
        }
        for x in d.iter_mut() {
            *x /= n;
        }
        HeptaVector::new(d)
    }
}

/// Quantum state of a particle — maps to Pauli energy shells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantumState {
    /// s = +1 — Conduction Band — Golden Record candidate.
    Active,
    /// s = 0 — Forbidden Gap — awaiting Data Steward.
    Stewardship,
    /// s = -1 — Valence Band — archived, mathematical barrier.
    Dead,
}

impl QuantumState {
    pub fn spin(&self) -> i8 {
        match self {
            Self::Active => 1,
            Self::Stewardship => 0,
            Self::Dead => -1,
        }
    }

    pub fn from_b11(b11: u8) -> Self {
        match b11 {
            200..=255 => Self::Active,
            60..=199 => Self::Stewardship,
            _ => Self::Dead,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active (Golden)",
            Self::Stewardship => "Stewardship (Fuzzy)",
            Self::Dead => "Dead (Archived)",
        }
    }
}

/// Complete particle snapshot for EaAgent evaluation.
#[derive(Debug, Clone)]
pub struct ParticleSnapshot {
    /// Particle name or ID for display.
    pub name: String,
    /// KAKI uuid_hash (D1 identity).
    pub kaki_hash: u32,
    /// Tribe ID (D2 belonging).
    pub tribe_id: u16,
    /// The 7D Hepta vector.
    pub hepta: HeptaVector,
    /// Current quantum state.
    pub state: QuantumState,
    /// Principal number n — Jordan Chain depth (orbit depth).
    pub n: u32,
}

impl ParticleSnapshot {
    pub fn new(name: &str, kaki_hash: u32, tribe_id: u16, hepta: HeptaVector) -> Self {
        let b11 = hepta.b11();
        let state = QuantumState::from_b11(b11);
        Self {
            name: name.to_string(),
            kaki_hash,
            tribe_id,
            hepta,
            state,
            n: 0,
        }
    }

    /// B11 quality score.
    pub fn b11(&self) -> u8 {
        self.hepta.b11()
    }

    /// Is this particle in the GEM lane?
    pub fn is_gem(&self) -> bool {
        self.b11() >= GEM_B11
    }

    /// Quantum exclusion coordinate: (kaki_hash, n, spin).
    pub fn exclusion_coordinate(&self) -> (u32, u32, i8) {
        (self.kaki_hash, self.n, self.state.spin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_vector_quality_is_one() {
        let v = HeptaVector::unit();
        assert!((v.quality_score() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn zero_vector_quality_is_low() {
        let v = HeptaVector::zero();
        let q = v.quality_score();
        assert!(q <= 0.5, "zero vector quality should be <= 0.5, got {q}");
    }

    #[test]
    fn b11_unit_is_240() {
        assert_eq!(HeptaVector::unit().b11(), 240);
    }

    #[test]
    fn b11_zero_is_low() {
        assert!(HeptaVector::zero().b11() <= 120);
    }

    #[test]
    fn manhattan_distance_zero_to_self() {
        let v = HeptaVector::new([0.5; 7]);
        assert!((v.manhattan_distance(&v)).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_unit_to_self() {
        let v = HeptaVector::unit();
        assert!((v.cosine_similarity(&v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn quantum_state_from_b11() {
        assert_eq!(QuantumState::from_b11(210), QuantumState::Active);
        assert_eq!(QuantumState::from_b11(120), QuantumState::Stewardship);
        assert_eq!(QuantumState::from_b11(30), QuantumState::Dead);
    }

    #[test]
    fn spin_values_correct() {
        assert_eq!(QuantumState::Active.spin(), 1);
        assert_eq!(QuantumState::Stewardship.spin(), 0);
        assert_eq!(QuantumState::Dead.spin(), -1);
    }

    #[test]
    fn centroid_of_two_vectors() {
        let a = HeptaVector::new([0.0; 7]);
        let b = HeptaVector::new([1.0; 7]);
        let c = HeptaVector::centroid(&[&a, &b]);
        for d in c.d {
            assert!((d - 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn particle_snapshot_gem_detection() {
        let v = HeptaVector::unit();
        let p = ParticleSnapshot::new("GEM-001", 0xDEADBEEF, 0x0001, v);
        assert!(p.is_gem());
        assert_eq!(p.state, QuantumState::Active);
    }
}
