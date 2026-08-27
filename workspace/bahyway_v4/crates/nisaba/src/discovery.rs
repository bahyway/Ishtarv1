#![forbid(unsafe_code)]
//! NisabaDiscovery — the main pattern discovery entry point.
//! Converts a GaCluster into a DiscoveredPattern with a minted Pattern-KAKI.

use crate::cluster::GaCluster;
use enkidb_kaki::{
    derive_pattern_kaki, pattern_kaki_confidence, Kaki, PatternKaki, PatternLifecycle, PatternType,
};

/// A discovered pattern ready for submission to ENKI-PATTERN registry.
#[derive(Clone, Debug)]
pub struct DiscoveredPattern {
    /// The deterministically derived Pattern-KAKI.
    pub pattern_kaki: Kaki,
    /// Pattern domain type.
    pub pattern_type: PatternType,
    /// Initial lifecycle state based on cluster confidence.
    pub lifecycle: PatternLifecycle,
    /// Confidence in millipercent (0..=10_000), readable from KAKI bytes κ[12..14].
    pub confidence_millipercent: u16,
    /// Number of constituent particles in the GA cluster.
    pub constituent_count: u32,
    /// Merkle root over sorted constituent KAKIs (for integrity verification).
    pub constituent_merkle_root: [u8; 32],
    /// BASE orbital when this pattern was discovered.
    pub orbital_formed: u64,
}

impl DiscoveredPattern {
    /// Verify that the confidence encoded in the KAKI bytes matches the struct field.
    pub fn verify_confidence(&self) -> bool {
        pattern_kaki_confidence(&self.pattern_kaki) == Some(self.confidence_millipercent)
    }
}

/// Stateless discovery engine — converts clusters into patterns.
pub struct NisabaDiscovery;

impl NisabaDiscovery {
    /// Convert a `GaCluster` to a `DiscoveredPattern` by deriving a Pattern-KAKI.
    ///
    /// Returns `None` if the cluster's confidence is below the Emerging threshold (0.60).
    pub fn discover(cluster: &GaCluster, pattern_type: PatternType) -> Option<DiscoveredPattern> {
        if !cluster.metrics.is_emerging() {
            return None;
        }

        let confidence_f = cluster.metrics.confidence;
        // Convert to millipercent: clamp then scale
        let confidence_millipercent = (confidence_f.clamp(0.0, 1.0) * 10_000.0).round() as u16;

        let pattern_kaki = derive_pattern_kaki(
            pattern_type,
            cluster.centroid,
            cluster.merkle_root,
            confidence_millipercent,
            cluster.orbital_formed,
        );

        let lifecycle = if cluster.metrics.is_stable() {
            PatternLifecycle::Stable
        } else {
            PatternLifecycle::Emerging
        };

        // Wrap to PatternKaki newtype for type-safety, then extract inner Kaki
        // (the registry stores raw Kaki; PatternKaki is used at call-site boundaries).
        let _typed = PatternKaki::try_from_kaki(pattern_kaki)
            .expect("derive_pattern_kaki always produces kaki_type=Pattern");

        Some(DiscoveredPattern {
            pattern_kaki,
            pattern_type,
            lifecycle,
            confidence_millipercent,
            constituent_count: cluster.metrics.constituent_count,
            constituent_merkle_root: cluster.merkle_root,
            orbital_formed: cluster.orbital_formed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::GaCluster;
    use enkidb_kaki::FixedCoord7D;

    fn make_cluster(confidence: f64) -> GaCluster {
        // density=1.0, coherence=confidence → confidence = sqrt(1 × confidence²)
        // Actually we want the final confidence to equal the parameter.
        // Since confidence = sqrt(density × coherence), set both to confidence:
        // confidence = sqrt(c × c) = c.
        GaCluster::new(
            FixedCoord7D {
                d: [1_000, 2_000, 500, 1, 2, 3, 0],
            },
            vec![],
            confidence,
            confidence,
            42,
        )
    }

    #[test]
    fn below_threshold_returns_none() {
        let cluster = make_cluster(0.50);
        assert!(NisabaDiscovery::discover(&cluster, PatternType::CrowdFlow).is_none());
    }

    #[test]
    fn emerging_pattern_discovered() {
        let cluster = make_cluster(0.65);
        let dp = NisabaDiscovery::discover(&cluster, PatternType::IndoorHallway).unwrap();
        assert_eq!(dp.lifecycle, PatternLifecycle::Emerging);
        assert!(dp.verify_confidence());
    }

    #[test]
    fn stable_pattern_discovered() {
        let cluster = make_cluster(0.90);
        let dp = NisabaDiscovery::discover(&cluster, PatternType::CrowdFlow).unwrap();
        assert_eq!(dp.lifecycle, PatternLifecycle::Stable);
        assert_eq!(dp.pattern_type, PatternType::CrowdFlow);
        assert!(dp.verify_confidence());
    }

    #[test]
    fn pattern_kaki_is_pattern_type() {
        let cluster = make_cluster(0.80);
        let dp = NisabaDiscovery::discover(&cluster, PatternType::WaterFlow).unwrap();
        use enkidb_kaki::KakiType;
        assert_eq!(dp.pattern_kaki.kaki_type(), KakiType::Pattern);
    }

    #[test]
    fn deterministic_discovery() {
        let cluster = make_cluster(0.85);
        let dp1 = NisabaDiscovery::discover(&cluster, PatternType::AviationCorridor).unwrap();
        let dp2 = NisabaDiscovery::discover(&cluster, PatternType::AviationCorridor).unwrap();
        assert_eq!(dp1.pattern_kaki, dp2.pattern_kaki);
    }
}
