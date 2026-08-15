//! ereskigal-kaki — EREŠKIGAL Informal Settlement Intelligence
//! BC-ERESKIGAL-001 | Status: SCAFFOLD
//!
//! EREŠKIGAL = Queen of the Great Below — sovereign of the forgotten.
//! Domain: informal settlement mapping, service access, humanitarian needs.
#![forbid(unsafe_code)]

pub const TRIBE_ID: &str = "ereskigal_iraq";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EreskigalParticleClass {
    SettlementSite, // 0x01 / KISHIB
    ServiceEvent,   // 0x02 / ZIKRU
    NeedsSignal,    // 0x02 / PARZU
    ExternalReport, // 0x03 / CrossTribe (UNHCR/OCHA — ProvenanceEav mandatory)
}

impl EreskigalParticleClass {
    pub fn kaki_type(&self) -> u8 {
        match self {
            EreskigalParticleClass::SettlementSite => 0x01,
            EreskigalParticleClass::ServiceEvent => 0x02,
            EreskigalParticleClass::NeedsSignal => 0x02,
            EreskigalParticleClass::ExternalReport => 0x03,
        }
    }
    pub fn kaki_role(&self) -> u8 {
        match self {
            EreskigalParticleClass::SettlementSite => 0x01,  // KISHIB
            EreskigalParticleClass::ServiceEvent => 0x02,    // ZIKRU
            EreskigalParticleClass::NeedsSignal => 0x03,     // PARZU
            EreskigalParticleClass::ExternalReport => 0x03,  // PARZU
        }
    }
}

/// SVI = Settlement Vulnerability Index (stub)
pub fn compute_svi(
    access_water: f64,
    access_sanitation: f64,
    population_density: f64,
    service_gap_days: f64,
) -> f64 {
    let water_gap = 1.0 - access_water.clamp(0.0, 1.0);
    let sanit_gap = 1.0 - access_sanitation.clamp(0.0, 1.0);
    let density_risk = (population_density / 10000.0).clamp(0.0, 1.0);
    let gap_risk = (service_gap_days / 30.0).clamp(0.0, 1.0);
    ((water_gap * 0.35) + (sanit_gap * 0.30) + (density_risk * 0.15) + (gap_risk * 0.20))
        .clamp(0.0, 1.0)
}

pub fn eav_state(svi: f64) -> &'static str {
    if svi <= 0.3 {
        "GOLDEN"
    } else if svi <= 0.7 {
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
        assert_eq!(EreskigalParticleClass::SettlementSite.kaki_type(), 0x01);
        assert_eq!(EreskigalParticleClass::ExternalReport.kaki_type(), 0x03);
    }

    #[test]
    fn svi_full_access_is_golden() {
        let svi = compute_svi(1.0, 1.0, 0.0, 0.0);
        assert_eq!(svi, 0.0);
        assert_eq!(eav_state(svi), "GOLDEN");
    }

    #[test]
    fn svi_no_access_is_dead() {
        let svi = compute_svi(0.0, 0.0, 10000.0, 30.0);
        assert_eq!(svi, 1.0);
        assert_eq!(eav_state(svi), "DEAD");
    }

    #[test]
    fn svi_clamped_to_0_1_range() {
        assert_eq!(compute_svi(-1.0, -1.0, 999999.0, 999.0), 1.0);
    }
}
