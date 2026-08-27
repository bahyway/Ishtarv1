//! junction.rs — WPDEngine's consumer of Enbilulu Calculus.
//!
//! The math itself (Phi_Enbi weighting, TIAMAT bands, Enbi horizon,
//! Terru diagnosis, the compass sentence) lives in
//! `bahyway_algebra::enbilulu` now, not here — the real "GeoEngine"
//! (corrected 2026-07-10; see bahyway-algebra's lib.rs for why). This
//! module stays deliberately thin: it maps a BaghdadSector-scoped
//! junction assessment onto the shared math and back into WPDEngine's
//! own domain types (sector, tribe_id), proving the "single truth
//! source" law by actually depending on it rather than re-deriving the
//! formula locally.

use crate::sector::BaghdadSector;
use alert_engine::AlertSeverity;
use bahyway_algebra::enbilulu::{self, EnbiFactors, FailureMechanism, RootCause, TiamatBand};

/// A single junction's Enbilulu assessment, scoped to its Baghdad sector
/// and KAKI tribe for deployment routing.
#[derive(Debug, Clone)]
pub struct JunctionAssessment {
    pub junction_id: String,
    pub sector: BaghdadSector,
    pub tribe_id: u16,
    pub phi_enbi: f32,
    pub band: TiamatBand,
    pub mechanism: FailureMechanism,
    pub cause: RootCause,
    pub milu_severity: Option<AlertSeverity>,
    pub horizon_weeks: Option<f32>,
}

/// Assesses one junction: computes Phi_Enbi, its TIAMAT band, the
/// Terru-diagnosed mechanism/cause, the Milu alert severity (if any),
/// and the Enbi horizon given a weekly degradation rate. This is the
/// real call site proving WPDEngine consumes geo-engine rather than
/// duplicating its math.
pub fn assess_junction(
    junction_id: impl Into<String>,
    sector: BaghdadSector,
    factors: &EnbiFactors,
    phi_rate_per_week: f32,
) -> JunctionAssessment {
    let phi = enbilulu::phi_enbi(factors);
    let band = TiamatBand::from_phi(phi);
    let (mechanism, cause) = enbilulu::diagnose(factors);
    let milu_severity = band.to_alert_severity();
    let horizon_weeks = enbilulu::enbi_horizon_weeks(phi, phi_rate_per_week);

    JunctionAssessment {
        junction_id: junction_id.into(),
        sector,
        tribe_id: sector.tribe_id(),
        phi_enbi: phi,
        band,
        mechanism,
        cause,
        milu_severity,
        horizon_weeks,
    }
}

impl JunctionAssessment {
    /// The compass sentence for this junction, optionally with a
    /// capacity-expansion "what if" scenario.
    pub fn narrate(&self, expansion: Option<(&str, f32)>) -> String {
        let t = self.horizon_weeks.unwrap_or(f32::INFINITY);
        enbilulu::narrate(&self.junction_id, self.mechanism, self.cause, t, expansion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erra_junction_in_sadr_city_carries_that_tribe_id() {
        let factors = EnbiFactors {
            factor_1: 90.0,
            factor_2: 90.0,
            factor_3: 90.0,
            baru_residual: 90.0,
            factor_5: 90.0,
        };
        let a = assess_junction("J-SadrCity-12", BaghdadSector::SadrCity, &factors, 2.0);
        assert_eq!(a.tribe_id, BaghdadSector::SadrCity.tribe_id());
        assert_eq!(a.band, TiamatBand::Erra);
        assert_eq!(a.milu_severity, Some(AlertSeverity::Critical));
        assert_eq!(a.horizon_weeks, Some(0.0));
    }

    #[test]
    fn stable_junction_has_no_milu_alert() {
        let factors = EnbiFactors {
            factor_1: 5.0,
            factor_2: 5.0,
            factor_3: 5.0,
            baru_residual: 5.0,
            factor_5: 5.0,
        };
        let a = assess_junction("J-GreenZone-1", BaghdadSector::GreenZone, &factors, 1.0);
        assert_eq!(a.band, TiamatBand::Stable);
        assert_eq!(a.milu_severity, None);
    }

    #[test]
    fn narrate_includes_junction_id_and_sector_tribe_is_reachable() {
        let factors = EnbiFactors {
            factor_1: 50.0,
            factor_2: 50.0,
            factor_3: 50.0,
            baru_residual: 70.0,
            factor_5: 50.0,
        };
        let a = assess_junction("J-Karrada-7", BaghdadSector::Karrada, &factors, 4.0);
        let sentence = a.narrate(Some(("Karrada-capacity-X", 20.0)));
        assert!(sentence.starts_with("Element J-Karrada-7"));
        assert!(sentence.contains("Karrada-capacity-X"));
        assert_eq!(a.tribe_id, 0x3003); // Karrada = index 3 -> 0x3000+3
    }
}
