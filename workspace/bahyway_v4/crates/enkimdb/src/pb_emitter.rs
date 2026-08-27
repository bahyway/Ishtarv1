//! PbEmitter — mints one Identity-Kaki per real, numbered playbook and
//! emits its profile as real `enkidb_particles::Particle` EAV rows,
//! namespaced under `pb.*`. Same shape as `RegistryEmitter`
//! (`error_type.*`/`attr_type.*`) — a playbook literally is "logic, a
//! template, an axiom, or a rule" (Akkadian *parṣu*), the same category
//! `RegistryEmitter::emit_error_type` mints registered ErrorTypes under.

use std::time::{SystemTime, UNIX_EPOCH};

use akkvalue::AkkValue;
use enkidb_kaki::{IdentityKaki, KakiMinter, KakiRole};
use enkidb_particles::Particle;

use crate::pb::PbProfile;

pub struct PbEmitter<'a> {
    minter: &'a KakiMinter,
}

impl<'a> PbEmitter<'a> {
    pub fn new(minter: &'a KakiMinter) -> Self {
        Self { minter }
    }

    /// Mint an Identity-Kaki (role=Parzu) for this playbook and emit its
    /// EAV rows. Called once per real, numbered PB file — ADR-014's "mint
    /// at authoring time," not deferred to first execution.
    pub fn emit(&self, profile: &PbProfile) -> (IdentityKaki, Vec<Particle>) {
        let kaki = IdentityKaki::try_from_kaki(self.minter.identity(KakiRole::Parzu))
            .expect("KakiMinter::identity always mints kaki_type=Identity");
        let now = now_secs();

        let mut particles = vec![
            Particle::base(kaki, "pb.number", AkkValue::Int(profile.number as i64), now),
            Particle::base(kaki, "pb.file", AkkValue::Text(profile.file.clone()), now),
            Particle::base(kaki, "pb.path", AkkValue::Text(profile.path.clone()), now),
        ];
        if let Some(status) = &profile.status {
            particles.push(Particle::base(
                kaki,
                "pb.status",
                AkkValue::Text(status.clone()),
                now,
            ));
        }
        if let Some(summary) = &profile.summary {
            particles.push(Particle::base(
                kaki,
                "pb.summary",
                AkkValue::Text(summary.clone()),
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
    use bahyway_core::TribeId;

    fn minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0001))
    }

    fn profile() -> PbProfile {
        PbProfile {
            number: 269,
            file: "playbook_269_x.yml".to_string(),
            path: "playbooks/playbook_269_x.yml".to_string(),
            status: Some("[~]".to_string()),
            summary: Some("authored 2026-07-30".to_string()),
        }
    }

    #[test]
    fn emits_identity_kaki_with_parzu_role() {
        let m = minter();
        let (kaki, _particles) = PbEmitter::new(&m).emit(&profile());
        assert_eq!(kaki.role(), KakiRole::Parzu);
    }

    #[test]
    fn emits_status_and_summary_only_when_known() {
        let m = minter();
        let e = PbEmitter::new(&m);
        let (_, with_particles) = e.emit(&profile());
        let mut bare = profile();
        bare.status = None;
        bare.summary = None;
        let (_, without_particles) = e.emit(&bare);

        assert!(with_particles.iter().any(|p| p.attribute == "pb.status"));
        assert!(with_particles.iter().any(|p| p.attribute == "pb.summary"));
        assert!(!without_particles.iter().any(|p| p.attribute == "pb.status"));
        assert!(!without_particles
            .iter()
            .any(|p| p.attribute == "pb.summary"));
    }

    #[test]
    fn always_emits_number_file_and_path() {
        let m = minter();
        let (_, particles) = PbEmitter::new(&m).emit(&profile());
        assert!(particles.iter().any(|p| p.attribute == "pb.number"));
        assert!(particles.iter().any(|p| p.attribute == "pb.file"));
        assert!(particles.iter().any(|p| p.attribute == "pb.path"));
    }

    #[test]
    fn all_particles_share_the_pb_entity() {
        let m = minter();
        let (kaki, particles) = PbEmitter::new(&m).emit(&profile());
        for p in &particles {
            assert_eq!(p.entity, kaki);
        }
    }
}
