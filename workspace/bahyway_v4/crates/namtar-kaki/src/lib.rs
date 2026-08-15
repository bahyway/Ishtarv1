//! namtar-kaki — NAMTAR Sacred Burial Intelligence KAKI Schema
//! BC-NAMTAR-001 | Status: SCAFFOLD
//!
//! NAMTAR = Mesopotamian messenger of fate, guardian between living and dead.
//! Domain: sacred burial site preservation, threat detection, heritage provenance.
#![forbid(unsafe_code)]

pub const TRIBE_ID: &str = "namtar_iraq";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamtarParticleClass {
    SiteRecord,   // 0x01 / KISHIB — site identity
    StatusEvent,  // 0x02 / ZIKRU  — preservation state change
    ThreatSignal, // 0x02 / PARZU  — looting/conflict/construction risk
    ExternalRef,  // 0x03 / CrossTribe — UNESCO/govt/satellite
}

impl NamtarParticleClass {
    pub fn kaki_type(&self) -> u8 {
        match self {
            NamtarParticleClass::SiteRecord => 0x01,
            NamtarParticleClass::StatusEvent => 0x02,
            NamtarParticleClass::ThreatSignal => 0x02,
            NamtarParticleClass::ExternalRef => 0x03,
        }
    }
    pub fn kaki_role(&self) -> u8 {
        match self {
            NamtarParticleClass::SiteRecord => 0x01,   // KISHIB
            NamtarParticleClass::StatusEvent => 0x02,  // ZIKRU
            NamtarParticleClass::ThreatSignal => 0x03, // PARZU
            NamtarParticleClass::ExternalRef => 0x03,  // PARZU
        }
    }
}

/// PHI = Preservation Health Index (stub — full Mamdani FIS in BC-NAMTAR-001 full)
pub fn compute_phi(site_integrity: f64, threat_level: f64, access_status: f64) -> f64 {
    let health = site_integrity.clamp(0.0, 1.0);
    let threat = threat_level.clamp(0.0, 1.0);
    let access = access_status.clamp(0.0, 1.0);
    ((health * 0.5) + ((1.0 - threat) * 0.3) + (access * 0.2)).clamp(0.0, 1.0)
}

pub fn eav_state(phi: f64) -> &'static str {
    if phi >= 0.7 {
        "GOLDEN"
    } else if phi >= 0.3 {
        "FUZZY"
    } else {
        "DEAD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kaki_type_role_mapping() {
        assert_eq!(NamtarParticleClass::SiteRecord.kaki_type(), 0x01);
        assert_eq!(NamtarParticleClass::SiteRecord.kaki_role(), 0x01);
        assert_eq!(NamtarParticleClass::ExternalRef.kaki_type(), 0x03);
    }

    #[test]
    fn phi_perfect_site_is_golden() {
        let phi = compute_phi(1.0, 0.0, 1.0);
        assert_eq!(phi, 1.0);
        assert_eq!(eav_state(phi), "GOLDEN");
    }

    #[test]
    fn phi_destroyed_site_is_dead() {
        let phi = compute_phi(0.0, 1.0, 0.0);
        assert_eq!(phi, 0.0);
        assert_eq!(eav_state(phi), "DEAD");
    }

    #[test]
    fn phi_clamped_to_0_1_range() {
        assert_eq!(compute_phi(2.0, -1.0, 2.0), 1.0);
    }
}
