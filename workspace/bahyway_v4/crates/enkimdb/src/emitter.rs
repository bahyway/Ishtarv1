//! ArtifactEmitter — mints one Identity-Kaki per artifact and emits its
//! profile as real `enkidb_particles::Particle` EAV rows, namespaced
//! under `artifact.*`.

use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};
use enkidb_particles::Particle;

use crate::artifact::ArtifactProfile;

pub struct ArtifactEmitter<'a> {
    minter: &'a KakiMinter,
}

impl<'a> ArtifactEmitter<'a> {
    pub fn new(minter: &'a KakiMinter) -> Self {
        Self { minter }
    }

    /// Mint an Identity-Kaki for this artifact and emit its EAV rows.
    pub fn emit(&self, profile: &ArtifactProfile) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Kishib))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();

        let mut particles = vec![
            Particle::base(
                kaki,
                "artifact.name",
                AkkValue::Text(profile.name.clone()),
                now,
            ),
            Particle::base(
                kaki,
                "artifact.kind",
                AkkValue::Text(profile.kind.as_str().to_string()),
                now,
            ),
            Particle::base(
                kaki,
                "artifact.path",
                AkkValue::Text(profile.path.clone()),
                now,
            ),
        ];

        if let Some(version) = &profile.version {
            particles.push(Particle::base(
                kaki,
                "artifact.version",
                AkkValue::Text(version.clone()),
                now,
            ));
        }

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
    use crate::artifact::ArtifactKind;
    use bahyway_core::TribeId;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x00E2))
    }

    #[test]
    fn emits_identity_kaki_with_kishib_role() {
        let m = minter();
        let profile = ArtifactProfile {
            name: "enkiddb".to_string(),
            kind: ArtifactKind::Crate,
            path: "crates/enkiddb".to_string(),
            version: Some("4.0.2".to_string()),
        };
        let (kaki, _particles) = ArtifactEmitter::new(&m).emit(&profile);
        assert_eq!(kaki.role(), KakiRole::Kishib);
    }

    #[test]
    fn emits_version_particle_only_when_known() {
        let m = minter();
        let with_version = ArtifactProfile {
            name: "enkiddb".to_string(),
            kind: ArtifactKind::Crate,
            path: "crates/enkiddb".to_string(),
            version: Some("4.0.2".to_string()),
        };
        let without_version = ArtifactProfile {
            name: "playbook_96_enkimdb".to_string(),
            kind: ArtifactKind::Playbook,
            path: "playbooks/playbook_96_enkimdb.yml".to_string(),
            version: None,
        };
        let e = ArtifactEmitter::new(&m);
        let (_, with_particles) = e.emit(&with_version);
        let (_, without_particles) = e.emit(&without_version);

        assert!(with_particles.iter().any(|p| p.attribute == "artifact.version"));
        assert!(!without_particles.iter().any(|p| p.attribute == "artifact.version"));
    }

    #[test]
    fn all_particles_share_artifact_entity() {
        let m = minter();
        let profile = ArtifactProfile {
            name: "kinetic-engine".to_string(),
            kind: ArtifactKind::Crate,
            path: "crates/kinetic-engine".to_string(),
            version: None,
        };
        let (kaki, particles) = ArtifactEmitter::new(&m).emit(&profile);
        for p in &particles {
            assert_eq!(p.entity, kaki);
        }
    }
}
