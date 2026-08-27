//! TabletEmitter — mints one Identity-Kaki per real `.akk`/`.way`/`.tmpl`
//! tablet and emits its profile as real `enkidb_particles::Particle` EAV
//! rows, namespaced under `<kind>.*` (e.g. `akk.path`, `way.sha256`).
//! Mirrors `PbEmitter`'s shape exactly: role Parzu ("logic, template,
//! axiom, or rule") — a tablet, like a playbook, is a rule/procedure the
//! ecosystem executes, not a passive record.

use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};
use enkidb_particles::Particle;

use crate::tablet::TabletProfile;

pub struct TabletEmitter<'a> {
    minter: &'a KakiMinter,
}

impl<'a> TabletEmitter<'a> {
    pub fn new(minter: &'a KakiMinter) -> Self {
        Self { minter }
    }

    /// Mint an Identity-Kaki (role=Parzu) for this tablet and emit its
    /// EAV rows: universal `<kind>.path`/`<kind>.bytes`/`<kind>.sha256`,
    /// plus whatever kind-specific `extra` fields `profile_tablet`
    /// extracted (real `.akk` structure when it parses; empty for
    /// `.way`/`.tmpl`, which have no parser yet).
    pub fn emit(&self, profile: &TabletProfile) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Parzu))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();
        let kind = profile.kind.as_str();

        let mut particles = vec![
            Particle::base(
                kaki,
                leaked_attr(kind, "path"),
                AkkValue::Text(profile.path.clone()),
                now,
            ),
            Particle::base(
                kaki,
                leaked_attr(kind, "bytes"),
                AkkValue::Int(profile.bytes as i64),
                now,
            ),
            Particle::base(
                kaki,
                leaked_attr(kind, "sha256"),
                AkkValue::Text(profile.sha256.clone()),
                now,
            ),
        ];
        for (attr, value) in &profile.extra {
            particles.push(Particle::base(
                kaki,
                (*attr).to_string(),
                value.clone(),
                now,
            ));
        }

        (kaki, particles)
    }
}

/// `<kind>.<field>`, e.g. "way.path". Universal fields aren't known at
/// compile time per kind (unlike `extra`'s `&'static str` literals), so
/// this builds the attribute name at call time.
fn leaked_attr(kind: &str, field: &str) -> String {
    format!("{kind}.{field}")
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
    use crate::tablet::TabletKind;
    use bahyway_core::TribeId;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x7163))
    }

    fn way_profile() -> TabletProfile {
        TabletProfile {
            kind: TabletKind::Way,
            path: "policies/example.way".to_string(),
            bytes: 42,
            sha256: "deadbeef".to_string(),
            extra: vec![],
        }
    }

    fn akk_profile_with_structure() -> TabletProfile {
        TabletProfile {
            kind: TabletKind::Akk,
            path: "tablets/example.akk".to_string(),
            bytes: 100,
            sha256: "cafef00d".to_string(),
            extra: vec![
                ("akk.tribe_count", AkkValue::Int(2)),
                ("akk.rule_count", AkkValue::Int(5)),
            ],
        }
    }

    #[test]
    fn emits_identity_kaki_with_parzu_role() {
        let m = minter();
        let (kaki, _) = TabletEmitter::new(&m).emit(&way_profile());
        assert_eq!(kaki.role(), KakiRole::Parzu);
    }

    #[test]
    fn attribute_names_are_namespaced_by_kind() {
        let m = minter();
        let (_, particles) = TabletEmitter::new(&m).emit(&way_profile());
        assert!(particles.iter().any(|p| p.attribute == "way.path"));
        assert!(particles.iter().any(|p| p.attribute == "way.bytes"));
        assert!(particles.iter().any(|p| p.attribute == "way.sha256"));
    }

    #[test]
    fn extra_fields_are_emitted_verbatim_when_present() {
        let m = minter();
        let (_, particles) = TabletEmitter::new(&m).emit(&akk_profile_with_structure());
        assert!(particles.iter().any(|p| p.attribute == "akk.tribe_count"));
        assert!(particles.iter().any(|p| p.attribute == "akk.rule_count"));
        // Universal fields are still present alongside the extras.
        assert!(particles.iter().any(|p| p.attribute == "akk.path"));
    }

    #[test]
    fn no_extra_fields_for_a_kind_with_no_parser() {
        let m = minter();
        let (_, particles) = TabletEmitter::new(&m).emit(&way_profile());
        assert_eq!(
            particles.len(),
            3,
            "way.path/bytes/sha256 only -- no fabricated extras"
        );
    }

    #[test]
    fn all_particles_share_the_tablet_entity() {
        let m = minter();
        let (kaki, particles) = TabletEmitter::new(&m).emit(&akk_profile_with_structure());
        for p in &particles {
            assert_eq!(p.entity, kaki);
        }
    }
}
