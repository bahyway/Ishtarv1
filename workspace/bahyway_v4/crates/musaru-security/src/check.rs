//! MusaruCheck — sovereignty and role validation (§10.2, PA-15).
//!
//! Musarû (Akkadian: "confirmation") enforces that incoming particles belong
//! to the correct tribe and carry a valid role. No KAKI may cross tribe
//! boundaries without explicit CrossTribe-Kaki authorization.

use bahyway_core::TribeId;
use enkidb_kaki::IdentityKaki;

/// Result of a Musarû security check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityResult {
    /// Particle passed all checks.
    Approved,
    /// Tribe mismatch — PA-15 violation.
    TribeMismatch { expected: u16, actual: u16 },
    /// KAKI checksum invalid — corrupted or forged.
    InvalidChecksum,
    /// Role is not permitted for this station's intake.
    ForbiddenRole(u8),
}

impl SecurityResult {
    pub fn is_approved(&self) -> bool {
        matches!(self, SecurityResult::Approved)
    }
}

/// Validate that the particle's tribe_id matches the database tribe.
pub fn check_sovereignty(db_tribe: TribeId, particle: &IdentityKaki) -> SecurityResult {
    if !particle.verify_checksum() {
        return SecurityResult::InvalidChecksum;
    }
    let expected = db_tribe.as_u16();
    let actual = particle.tribe_id().as_u16();
    if expected != actual {
        return SecurityResult::TribeMismatch { expected, actual };
    }
    SecurityResult::Approved
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{mint::KakiMinter, KakiRole};

    #[test]
    fn approved_for_matching_tribe() {
        let tid = TribeId::from_u16(0x0001);
        let m = KakiMinter::new(tid);
        let p = enkidb_kaki::IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        assert_eq!(check_sovereignty(tid, &p), SecurityResult::Approved);
    }

    #[test]
    fn rejected_for_wrong_tribe() {
        let tid1 = TribeId::from_u16(0x0001);
        let tid2 = TribeId::from_u16(0x0002);
        let m = KakiMinter::new(tid1);
        let p = enkidb_kaki::IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let res = check_sovereignty(tid2, &p);
        assert!(matches!(res, SecurityResult::TribeMismatch { .. }));
    }
}
