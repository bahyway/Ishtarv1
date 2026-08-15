//! PatternEmitter — mints one Identity-Kaki per discovered pattern and
//! emits its profile as real `enkidb_particles::Particle` EAV rows,
//! namespaced under `pattern.*`. Same shape as `PbEmitter`/
//! `DocumentEmitter` (ADR-014's mint-at-authoring-time pattern).
//!
//! A pattern already carries its OWN deterministic Pattern-KAKI
//! (`enkidb_kaki::derive_pattern_kaki`, `KakiType::Pattern`) — this
//! emitter does not discard that determinism, it preserves the pattern's
//! own KAKI as `pattern.pattern_kaki_hex` (a queryable EAV attribute),
//! while minting a FRESH Identity-Kaki (role=Parzu — "logic, a template,
//! an axiom, or a rule," the same category playbooks mint under) to
//! serve as the EnkiMDB particle's own entity. Two identities, two
//! purposes: the Pattern-KAKI is what makes pattern discovery
//! deterministic and comparable across a re-run; the Identity-Kaki is
//! what makes the pattern a real, queryable EnkiMDB particle.

use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};
use enkidb_particles::Particle;

use crate::pattern::PatternProfile;

pub struct PatternEmitter<'a> {
    minter: &'a KakiMinter,
}

impl<'a> PatternEmitter<'a> {
    pub fn new(minter: &'a KakiMinter) -> Self {
        Self { minter }
    }

    /// Mint a fresh Identity-Kaki (role=Parzu) for this pattern and emit
    /// its `pattern.*` EAV rows, including the pattern's own Pattern-KAKI
    /// (hex-encoded) and its age-derived storage tier.
    pub fn emit(&self, profile: &PatternProfile) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Parzu))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();
        let p = &profile.pattern;

        let particles = vec![
            Particle::base(kaki, "pattern.pattern_kaki_hex", AkkValue::Text(hex::encode(p.pattern_kaki.bytes())), now),
            Particle::base(kaki, "pattern.type", AkkValue::Text(format!("{:?}", p.pattern_type)), now),
            Particle::base(kaki, "pattern.lifecycle", AkkValue::Text(format!("{:?}", p.lifecycle)), now),
            Particle::base(kaki, "pattern.confidence_millipercent", AkkValue::Int(p.confidence_millipercent as i64), now),
            Particle::base(kaki, "pattern.constituent_count", AkkValue::Int(p.constituent_count as i64), now),
            Particle::base(kaki, "pattern.constituent_merkle_root_hex", AkkValue::Text(hex::encode(p.constituent_merkle_root)), now),
            Particle::base(kaki, "pattern.orbital_formed", AkkValue::Int(p.orbital_formed as i64), now),
            Particle::base(kaki, "pattern.tier", AkkValue::Text(profile.tier().as_str().to_string()), now),
        ];

        (kaki, particles)
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{FixedCoord7D, PatternType};
    use nisaba::cluster::GaCluster;
    use nisaba::discovery::NisabaDiscovery;

    fn make_minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0001))
    }

    fn make_pattern() -> nisaba::discovery::DiscoveredPattern {
        let cluster = GaCluster::new(
            FixedCoord7D { d: [1_000, 2_000, 500, 1, 2, 3, 0] },
            vec![],
            0.9,
            0.9,
            42,
        );
        NisabaDiscovery::discover(&cluster, PatternType::CrowdFlow).unwrap()
    }

    #[test]
    fn emit_mints_a_real_identity_kaki_distinct_from_the_pattern_kaki() {
        let minter = make_minter();
        let profile = PatternProfile::new(make_pattern(), 0);
        let emitter = PatternEmitter::new(&minter);

        let (identity, particles) = emitter.emit(&profile);
        assert_eq!(identity.kaki_type(), enkidb_kaki::KakiType::Identity);
        assert!(!particles.is_empty());
        for p in &particles {
            assert_eq!(p.entity, identity);
        }
    }

    #[test]
    fn emit_preserves_the_patterns_own_deterministic_kaki_as_an_eav_attribute() {
        let minter = make_minter();
        let pattern = make_pattern();
        let expected_hex = hex::encode(pattern.pattern_kaki.bytes());
        let profile = PatternProfile::new(pattern, 0);
        let emitter = PatternEmitter::new(&minter);

        let (_, particles) = emitter.emit(&profile);
        let found = particles.iter().find(|p| p.attribute == "pattern.pattern_kaki_hex").unwrap();
        assert_eq!(found.value, AkkValue::Text(expected_hex));
    }

    #[test]
    fn emit_computes_tier_from_age() {
        let minter = make_minter();
        // 400 orbitals old -> Crystallized (>= 365)
        let profile = PatternProfile::new(make_pattern(), 400);
        let emitter = PatternEmitter::new(&minter);

        let (_, particles) = emitter.emit(&profile);
        let found = particles.iter().find(|p| p.attribute == "pattern.tier").unwrap();
        assert_eq!(found.value, AkkValue::Text("Crystallized".to_string()));
    }

    #[test]
    fn two_emits_of_the_same_pattern_share_the_pattern_kaki_but_get_distinct_identities() {
        // Determinism where it matters (the pattern's own KAKI), fresh
        // identity where it doesn't (mint-at-authoring-time, ADR-014) --
        // two separate mint calls are two separate EnkiMDB particles,
        // even for the "same" discovered pattern.
        let minter = make_minter();
        let pattern1 = make_pattern();
        let pattern2 = make_pattern();
        assert_eq!(pattern1.pattern_kaki, pattern2.pattern_kaki, "same cluster inputs must derive the same Pattern-KAKI");

        let emitter = PatternEmitter::new(&minter);
        let (id1, _) = emitter.emit(&PatternProfile::new(pattern1, 0));
        let (id2, _) = emitter.emit(&PatternProfile::new(pattern2, 0));
        assert_ne!(id1, id2, "each mint call gets its own fresh Identity-Kaki");
    }
}
