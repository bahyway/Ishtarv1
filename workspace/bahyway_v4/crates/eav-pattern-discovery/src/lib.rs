//! eav-pattern-discovery — the two-track pattern-discovery engine for
//! "which EAV/quality attributes predict a particle's ETL station
//! outcome," as the Architect specified:
//!
//!   Track A (mandatory_track): patterns in the 7 mandatory attributes'
//!   OWN VALUES (State/ColorRgb/Username/Freshness/Velocity/SystemUser/
//!   UserGroup — `bahyway_core::mandatory_attrs`), via `fuzzy-engine`'s
//!   real Mamdani membership functions.
//!
//!   Track B (smd_track): SMD (Semantic Modeling for Data) — attribute-
//!   NAME equivalence across an organization's different departments
//!   (the Architect's own example: `employee_name` <=> `Empl_firstname`/
//!   `Empl_surname` <=> `Employee_Full_name`). Distinct from KAKI's own
//!   identity resolution (which stays untouched/immutable) — this
//!   operates on schema vocabulary, not entity identity.
//!
//! Both tracks are NOT spatial/trajectory pattern discovery — that
//! mechanism (`nisaba::cluster::GaCluster`, 7D centroids over particle
//! trajectories) stays exactly as it is, untouched, per the Architect's
//! own instruction ("leave GaCluster and the CrowdFlow... as you
//! described it"). Both tracks here construct `nisaba::discovery::
//! DiscoveredPattern` values directly (its fields are public) rather
//! than forcing non-spatial data through `GaCluster`'s spatial-specific
//! API — a `FixedCoord7D` is still fed to `derive_pattern_kaki` (it's
//! the KAKI-minting function's only input shape), but it carries a
//! SYMBOLIC hash encoding here, never claimed as a real spatial
//! centroid.
//!
//! Discovered patterns mint as real EnkiMDB particles via
//! `enkimdb::PatternEmitter` (see `bahyway_core`'s "no particle outside
//! the 7 Types EnkiDB" law) — this crate has no store of its own.
#![forbid(unsafe_code)]

pub mod mandatory_track;
pub mod smd_track;

/// Same Emerging-lifecycle confidence floor Nisaba's own spatial
/// discovery uses (`nisaba::cluster::ClusterMetrics::is_emerging`) —
/// kept identical for a consistent "is this real enough to mint"
/// threshold across every discovery track in the ecosystem.
pub const EMERGING_CONFIDENCE: f64 = 0.60;
/// Same Stable-lifecycle floor Nisaba's own spatial discovery uses.
pub const STABLE_CONFIDENCE: f64 = 0.85;

use enkidb_kaki::{derive_pattern_kaki, FixedCoord7D, PatternLifecycle, PatternType};
use mandatory_track::MandatoryAttrSample;
use nisaba::discovery::DiscoveredPattern;
use smd_track::SmdEquivalenceCluster;

/// Track A end to end: score a batch's mandatory-attribute-value
/// coherence, then mint it as a `DiscoveredPattern` (via
/// `PatternType::Custom` — patterns are entirely new here, not one of
/// the existing spatial `PatternType` variants) if confidence clears
/// `EMERGING_CONFIDENCE`. `orbital_formed` is the caller's own current
/// tick/epoch counter.
pub fn discover_mandatory_pattern(
    samples: &[MandatoryAttrSample],
    orbital_formed: u64,
) -> Option<DiscoveredPattern> {
    let result = mandatory_track::score_mandatory_value_pattern(samples)?;
    let bytes: Vec<u8> = samples
        .iter()
        .flat_map(|s| {
            s.freshness
                .to_le_bytes()
                .into_iter()
                .chain(s.velocity.to_le_bytes())
        })
        .collect();
    let symbolic = symbolic_hash_of(&[&bytes]);
    build_discovered_pattern(
        PatternType::Custom,
        symbolic,
        result.constituent_count,
        merkle_placeholder(&bytes),
        result.confidence,
        orbital_formed,
    )
}

/// Track B end to end: mint a detected SMD equivalence cluster as a
/// `DiscoveredPattern` if its confidence clears `EMERGING_CONFIDENCE`.
pub fn discover_smd_pattern(
    cluster: &SmdEquivalenceCluster,
    orbital_formed: u64,
) -> Option<DiscoveredPattern> {
    let names: Vec<u8> = cluster
        .members
        .iter()
        .flat_map(|m| m.attr_name.bytes())
        .collect();
    let symbolic = symbolic_hash_of(&[&names]);
    build_discovered_pattern(
        PatternType::Custom,
        symbolic,
        cluster.members.len() as u32,
        merkle_placeholder(&names),
        cluster.confidence,
        orbital_formed,
    )
}

/// A real (not cryptographically-strong) content digest over
/// constituent bytes — the same role `nisaba::cluster::GaCluster`'s own
/// `merkle_root` plays for spatial clusters (a fixed-size integrity
/// summary of what went into the pattern), computed here via the same
/// FNV mixing this module already uses, expanded to 32 bytes.
fn merkle_placeholder(bytes: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, chunk) in bytes.chunks(4).enumerate() {
        let lane = symbolic_hash_of(&[chunk])[0];
        let b = lane.to_le_bytes();
        for (k, byte) in b.iter().enumerate() {
            out[(i * 4 + k) % 32] ^= byte;
        }
    }
    out
}

/// Build a `DiscoveredPattern` from a confidence score plus a symbolic
/// (non-spatial) hash of whatever the pattern is actually about — shared
/// by both tracks so their output is uniformly mintable via
/// `enkimdb::PatternEmitter`. Returns `None` below `EMERGING_CONFIDENCE`,
/// matching Nisaba's own spatial-discovery threshold exactly.
pub(crate) fn build_discovered_pattern(
    pattern_type: PatternType,
    symbolic_hash: [i32; 7],
    constituent_count: u32,
    constituent_merkle_root: [u8; 32],
    confidence: f64,
    orbital_formed: u64,
) -> Option<DiscoveredPattern> {
    if confidence < EMERGING_CONFIDENCE {
        return None;
    }
    let confidence_millipercent = (confidence.clamp(0.0, 1.0) * 10_000.0).round() as u16;
    let centroid = FixedCoord7D { d: symbolic_hash };
    let pattern_kaki = derive_pattern_kaki(
        pattern_type,
        centroid,
        constituent_merkle_root,
        confidence_millipercent,
        orbital_formed,
    );
    let lifecycle = if confidence >= STABLE_CONFIDENCE {
        PatternLifecycle::Stable
    } else {
        PatternLifecycle::Emerging
    };
    Some(DiscoveredPattern {
        pattern_kaki,
        pattern_type,
        lifecycle,
        confidence_millipercent,
        constituent_count,
        constituent_merkle_root,
        orbital_formed,
    })
}

/// FNV-1a over arbitrary bytes, folded into 7 symbolic i64 lanes — the
/// same "mix everything into a fixed-size deterministic signature" idea
/// `derive_pattern_kaki` itself already uses internally, reused here
/// only to build the FUNCTION'S input, not to reimplement KAKI derivation.
pub(crate) fn symbolic_hash_of(parts: &[&[u8]]) -> [i32; 7] {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    let mut lanes = [0i32; 7];
    for (i, part) in parts.iter().enumerate() {
        let mut h = FNV_OFFSET ^ (i as u64);
        for &b in *part {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        lanes[i % 7] ^= h as i32;
    }
    lanes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_confidence_returns_none() {
        let p = build_discovered_pattern(PatternType::Custom, [0; 7], 3, [0; 32], 0.5, 1);
        assert!(p.is_none());
    }

    #[test]
    fn emerging_confidence_produces_emerging_lifecycle() {
        let p = build_discovered_pattern(PatternType::Custom, [1; 7], 3, [0; 32], 0.7, 1).unwrap();
        assert_eq!(p.lifecycle, PatternLifecycle::Emerging);
    }

    #[test]
    fn stable_confidence_produces_stable_lifecycle() {
        let p = build_discovered_pattern(PatternType::Custom, [1; 7], 3, [0; 32], 0.9, 1).unwrap();
        assert_eq!(p.lifecycle, PatternLifecycle::Stable);
    }

    #[test]
    fn symbolic_hash_is_deterministic() {
        let a = symbolic_hash_of(&[b"employee_name", b"finance"]);
        let b = symbolic_hash_of(&[b"employee_name", b"finance"]);
        assert_eq!(a, b);
    }

    #[test]
    fn symbolic_hash_differs_for_different_input() {
        let a = symbolic_hash_of(&[b"employee_name"]);
        let b = symbolic_hash_of(&[b"empl_firstname"]);
        assert_ne!(a, b);
    }

    #[test]
    fn discover_mandatory_pattern_mints_a_real_pattern_for_a_coherent_batch() {
        let samples = vec![
            MandatoryAttrSample {
                freshness: 0.8,
                velocity: 0.5,
            },
            MandatoryAttrSample {
                freshness: 0.8,
                velocity: 0.5,
            },
            MandatoryAttrSample {
                freshness: 0.8,
                velocity: 0.5,
            },
        ];
        let pattern = discover_mandatory_pattern(&samples, 100).unwrap();
        assert_eq!(pattern.pattern_type, PatternType::Custom);
        assert_eq!(pattern.constituent_count, 3);
        assert_eq!(pattern.orbital_formed, 100);
    }

    #[test]
    fn discover_mandatory_pattern_returns_none_for_incoherent_batch() {
        let samples = vec![
            MandatoryAttrSample {
                freshness: 0.0,
                velocity: 0.0,
            },
            MandatoryAttrSample {
                freshness: 1.0,
                velocity: 1.0,
            },
            MandatoryAttrSample {
                freshness: 0.5,
                velocity: 0.9,
            },
        ];
        assert!(discover_mandatory_pattern(&samples, 100).is_none());
    }

    #[test]
    fn discover_smd_pattern_mints_a_real_pattern_for_the_architects_example() {
        let cluster = smd_track::SmdEquivalenceCluster {
            members: vec![
                smd_track::DepartmentAttr {
                    department: "hr".into(),
                    attr_name: "employee_name".into(),
                },
                smd_track::DepartmentAttr {
                    department: "finance".into(),
                    attr_name: "Employee_Full_Name".into(),
                },
            ],
            confidence: 0.75,
        };
        let pattern = discover_smd_pattern(&cluster, 50).unwrap();
        assert_eq!(pattern.pattern_type, PatternType::Custom);
        assert_eq!(pattern.constituent_count, 2);
        assert_eq!(pattern.confidence_millipercent, 7500);
    }

    #[test]
    fn end_to_end_discovered_pattern_mints_a_real_enkimdb_particle() {
        use bahyway_core::TribeId;
        use enkidb_kaki::KakiMinter;
        use enkimdb::{PatternProfile, WriteNode};

        let samples = vec![
            MandatoryAttrSample {
                freshness: 0.8,
                velocity: 0.5,
            },
            MandatoryAttrSample {
                freshness: 0.8,
                velocity: 0.5,
            },
        ];
        let pattern = discover_mandatory_pattern(&samples, 1)
            .expect("coherent batch must discover a pattern");

        let mut wn = WriteNode::new(KakiMinter::new(TribeId::from_u16(0x0001)), 64);
        let profile = PatternProfile::new(pattern, 0);
        let identity = wn.ingest_pattern(&profile, 1);

        let history = wn.journal().read_particle_history(&identity);
        assert_eq!(history.len(), 1);
        assert!(
            history[0].eav.iter().count() > 0,
            "the minted particle must carry real EAV data"
        );
    }
}
