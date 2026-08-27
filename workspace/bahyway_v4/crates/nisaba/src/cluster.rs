#![forbid(unsafe_code)]
//! GA cluster — the intermediate result of grouping trajectory segments by 7D centroid proximity.

use enkidb_kaki::{FixedCoord7D, Kaki};

/// Hard fanout limit from E7 Viazovska kissing number constraint.
pub const MAX_CONSTITUENTS: usize = 126;

/// Quality metrics for a GA cluster, used to determine PatternLifecycle.
#[derive(Clone, Debug)]
pub struct ClusterMetrics {
    /// Number of distinct particle KAKIs in this cluster.
    pub constituent_count: u32,
    /// Mean intra-cluster distance in 7D millimetre space (lower = tighter).
    pub mean_intra_distance_mm: i64,
    /// Confidence: proportion of constituent trajectory windows that are consistent
    /// with the centroid direction (0.0..=1.0).
    pub coherence: f64,
    /// Combined confidence score = sqrt(density × coherence), range [0.0, 1.0].
    pub confidence: f64,
}

impl ClusterMetrics {
    /// Compute confidence from raw inputs.
    /// `density` = constituent_count / expected_capacity (caller provides).
    pub fn compute(constituent_count: u32, density: f64, coherence: f64) -> Self {
        let coherence = coherence.clamp(0.0, 1.0);
        let density = density.clamp(0.0, 1.0);
        let confidence = (density * coherence).sqrt().clamp(0.0, 1.0);
        Self {
            constituent_count,
            mean_intra_distance_mm: 0, // filled by caller
            coherence,
            confidence,
        }
    }

    /// True if this cluster has reached Emerging lifecycle status.
    pub fn is_emerging(&self) -> bool {
        self.confidence >= 0.60
    }

    /// True if this cluster has reached Stable lifecycle status.
    pub fn is_stable(&self) -> bool {
        self.confidence >= 0.85
    }
}

/// A GA-produced cluster ready for Pattern-KAKI derivation.
#[derive(Clone, Debug)]
pub struct GaCluster {
    /// 7D centroid of this cluster in fixed-point millimetres.
    pub centroid: FixedCoord7D,
    /// Constituent particle KAKIs — capped at MAX_CONSTITUENTS (126).
    pub constituents: Vec<Kaki>,
    /// Merkle root over the sorted constituent KAKI bytes.
    pub merkle_root: [u8; 32],
    /// Quality metrics.
    pub metrics: ClusterMetrics,
    /// BASE orbital at which this cluster was first identified.
    pub orbital_formed: u64,
}

impl GaCluster {
    /// Build a GaCluster, capping constituents at MAX_CONSTITUENTS.
    pub fn new(
        centroid: FixedCoord7D,
        mut constituents: Vec<Kaki>,
        density: f64,
        coherence: f64,
        orbital_formed: u64,
    ) -> Self {
        constituents.truncate(MAX_CONSTITUENTS);
        let metrics = ClusterMetrics::compute(constituents.len() as u32, density, coherence);
        let merkle_root = Self::compute_merkle(&constituents);
        Self {
            centroid,
            constituents,
            merkle_root,
            metrics,
            orbital_formed,
        }
    }

    /// Deterministic Merkle root over sorted constituent KAKI bytes (FNV-1a chain).
    fn compute_merkle(constituents: &[Kaki]) -> [u8; 32] {
        // Sort constituent bytes for determinism (order of discovery is irrelevant).
        let mut sorted: Vec<[u8; 16]> = constituents.iter().map(|k| *k.bytes()).collect();
        sorted.sort_unstable();

        // FNV-1a chain over concatenated sorted bytes, folded into 32 bytes
        // by XOR-ing into alternating halves.
        const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
        const FNV_PRIME: u64 = 1_099_511_628_211;
        let mut h = FNV_OFFSET;
        let mut root = [0u8; 32];
        for (idx, raw) in sorted.iter().enumerate() {
            for byte in raw {
                h ^= *byte as u64;
                h = h.wrapping_mul(FNV_PRIME);
            }
            let slot = idx % 4;
            let h_bytes = h.to_be_bytes();
            root[slot * 8..(slot + 1) * 8]
                .iter_mut()
                .zip(h_bytes.iter())
                .for_each(|(r, &b)| *r ^= b);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{derive_pattern_kaki, PatternType};

    fn kaki(n: u16) -> Kaki {
        derive_pattern_kaki(
            PatternType::CrowdFlow,
            FixedCoord7D {
                d: [n as i32, 0, 0, 0, 0, 0, 0],
            },
            [0u8; 32],
            8_000,
            n as u64,
        )
    }

    #[test]
    fn constituents_capped_at_126() {
        let kakis: Vec<Kaki> = (0..200).map(kaki).collect();
        let cluster = GaCluster::new(FixedCoord7D::zero(), kakis, 0.9, 0.9, 1);
        assert_eq!(cluster.constituents.len(), 126);
    }

    #[test]
    fn merkle_is_deterministic() {
        let kakis: Vec<Kaki> = (0..10).map(kaki).collect();
        let c1 = GaCluster::new(FixedCoord7D::zero(), kakis.clone(), 0.8, 0.9, 1);
        let mut shuffled = kakis.clone();
        shuffled.swap(0, 5);
        let c2 = GaCluster::new(FixedCoord7D::zero(), shuffled, 0.8, 0.9, 1);
        assert_eq!(
            c1.merkle_root, c2.merkle_root,
            "merkle root must be order-independent"
        );
    }

    #[test]
    fn confidence_thresholds() {
        let m1 = ClusterMetrics::compute(50, 0.50, 0.72);
        assert!(m1.is_emerging());
        assert!(!m1.is_stable());

        let m2 = ClusterMetrics::compute(100, 0.90, 0.94);
        assert!(m2.is_stable());
    }
}
