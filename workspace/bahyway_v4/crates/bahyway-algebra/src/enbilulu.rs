//! enbilulu.rs — Enbilulu Calculus 𒀭𒂗𒉈𒇻, the divine canal inspector.
//!
//! HONEST SOURCE NOTE (2026-07-10): the authoritative source documents
//! (`BCENV001_Enbilulu_Calculus.md`, `BCENV001_Enbilulu_WPDEngine_Rev2.md`)
//! were verified as real, non-duplicate content by an earlier session
//! (2026-07-07) but are not present in this repository checkout — only
//! their verification summary is. This module is built from that summary
//! plus the Architect's own restated spec (2026-07-10): the five Phi_Enbi
//! weights, the baru King-plot residual as the heaviest factor, the four
//! TIAMAT band thresholds, and the closing narration sentence are all
//! confirmed exact. The identities of factors 1/2/3/5 (everything except
//! the baru residual) are NOT confirmed — placeholder names are used
//! below and marked `TODO(ARCHITECT)`. Replace them from the source .md
//! when available; do not treat the placeholder names as sealed.
//!
//! PB-150 does not exist in this repository (checked directly) despite
//! being cited as already covering this work — this module is the first
//! real implementation, not a port of an existing sealed playbook.
//!
//! Lives in `bahyway-algebra` — the crate the sealed concept documents
//! call "GeoEngine" (corrected 2026-07-10; an earlier pass this session
//! built a separate, now-removed `geo-engine` crate instead of finding
//! this one first).
#![forbid(unsafe_code)]

use alert_engine::AlertSeverity;

/// Weights for the five Phi_Enbi factors, confirmed exact by the
/// Architect (2026-07-10). Sum to 1.0 exactly, so Phi_Enbi stays on the
/// same 0-100 scale as its inputs.
pub const PHI_ENBI_WEIGHTS: [f32; 5] = [0.20, 0.20, 0.15, 0.30, 0.15];

/// The five junction defect-potential factors, each in [0.0, 100.0].
/// Only `baru_residual` (index 3, weight 0.30 -- the heaviest) is
/// confirmed by name. The others are placeholders pending the source
/// document.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EnbiFactors {
    /// TODO(ARCHITECT): real factor 1 name/definition pending source .md.
    pub factor_1: f32,
    /// TODO(ARCHITECT): real factor 2 name/definition pending source .md.
    pub factor_2: f32,
    /// TODO(ARCHITECT): real factor 3 name/definition pending source .md.
    pub factor_3: f32,
    /// baru King-plot residual (kappa) -- CONFIRMED: the heaviest-
    /// weighted factor (0.30). Named for the Mesopotamian diviner
    /// (baru) who read omens from a residual pattern against an
    /// expected baseline ("King-plot" = the reference trajectory a
    /// junction's real behaviour is compared against).
    pub baru_residual: f32,
    /// TODO(ARCHITECT): real factor 5 name/definition pending source .md.
    pub factor_5: f32,
}

impl EnbiFactors {
    fn as_weighted_array(&self) -> [f32; 5] {
        [
            self.factor_1,
            self.factor_2,
            self.factor_3,
            self.baru_residual,
            self.factor_5,
        ]
    }
}

/// Phi_Enbi junction defect potential, 0-100 scale.
/// weight order: [factor_1, factor_2, factor_3, baru_residual, factor_5]
///             = [0.20,     0.20,     0.15,     0.30,          0.15]
pub fn phi_enbi(f: &EnbiFactors) -> f32 {
    PHI_ENBI_WEIGHTS
        .iter()
        .zip(f.as_weighted_array().iter())
        .map(|(w, v)| w * v)
        .sum()
}

/// TIAMAT band for a Phi_Enbi score. Thresholds confirmed exact by the
/// Architect: Stable <40, Watch 40-60, Serious 60-80, ERRA >=80.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TiamatBand {
    Stable,
    Watch,
    Serious,
    Erra,
}

impl TiamatBand {
    pub fn from_phi(phi: f32) -> Self {
        if phi < 40.0 {
            TiamatBand::Stable
        } else if phi < 60.0 {
            TiamatBand::Watch
        } else if phi < 80.0 {
            TiamatBand::Serious
        } else {
            TiamatBand::Erra
        }
    }

    /// Bridges into the real alert-engine's AlertSeverity so Milu
    /// alerts flow through the same machinery as ColorID drift alerts,
    /// rather than a parallel alerting system. Stable produces no
    /// alert at all.
    pub fn to_alert_severity(self) -> Option<AlertSeverity> {
        match self {
            TiamatBand::Stable => None,
            TiamatBand::Watch => Some(AlertSeverity::Watch),
            TiamatBand::Serious => Some(AlertSeverity::Warning),
            TiamatBand::Erra => Some(AlertSeverity::Critical),
        }
    }
}

/// Enbi horizon: weeks until Phi_Enbi crosses ERRA (>=80), linearly
/// extrapolated from the current score and its weekly rate of change.
/// Returns `Some(0.0)` if already at or past ERRA, `None` if the rate is
/// zero or negative (never crosses under current conditions -- a loud
/// "no horizon" rather than a misleading infinite number).
pub fn enbi_horizon_weeks(phi_now: f32, phi_rate_per_week: f32) -> Option<f32> {
    if phi_now >= 80.0 {
        return Some(0.0);
    }
    if phi_rate_per_week <= 0.0 {
        return None;
    }
    Some(((80.0 - phi_now) / phi_rate_per_week).max(0.0))
}

/// Recomputes the horizon (T') under a hypothetical capacity expansion
/// that changes the weekly rate of Phi_Enbi increase -- the "T shrinks
/// to T'" half of the Architect's compass sentence. A well-designed
/// expansion should reduce the rate (larger denominator, longer T'); a
/// poorly-targeted one could worsen it (this function does not assume
/// expansions always help -- it just recomputes honestly).
pub fn expansion_horizon_weeks(phi_now: f32, new_rate_per_week: f32) -> Option<f32> {
    enbi_horizon_weeks(phi_now, new_rate_per_week)
}

/// Likely failure mechanism, per the Terru diagnosis layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureMechanism {
    Corrosion,
    JointFailure,
    Subsidence,
    Overpressure,
    Unknown,
}

/// Likely root cause, per the Terru diagnosis layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootCause {
    MaterialAge,
    SoilMovement,
    HydraulicSurge,
    ExternalLoad,
    Unknown,
}

/// One fingerprint-table entry: which factor dominating the Phi_Enbi
/// score implies which mechanism+cause pair.
///
/// STARTER TABLE ONLY. The Architect's real Terru fingerprint table
/// (from the source .md, not present in this checkout) should replace
/// these five illustrative entries -- they encode plausible domain
/// logic (which factor index tends to co-occur with which failure mode)
/// but are not sourced from the sealed document.
pub struct TertuFingerprint {
    pub dominant_factor_index: usize,
    pub mechanism: FailureMechanism,
    pub cause: RootCause,
}

pub const TERTU_TABLE: &[TertuFingerprint] = &[
    TertuFingerprint {
        dominant_factor_index: 0,
        mechanism: FailureMechanism::Corrosion,
        cause: RootCause::MaterialAge,
    },
    TertuFingerprint {
        dominant_factor_index: 1,
        mechanism: FailureMechanism::JointFailure,
        cause: RootCause::SoilMovement,
    },
    TertuFingerprint {
        dominant_factor_index: 2,
        mechanism: FailureMechanism::Overpressure,
        cause: RootCause::HydraulicSurge,
    },
    TertuFingerprint {
        dominant_factor_index: 3,
        mechanism: FailureMechanism::Subsidence,
        cause: RootCause::SoilMovement,
    },
    TertuFingerprint {
        dominant_factor_index: 4,
        mechanism: FailureMechanism::JointFailure,
        cause: RootCause::ExternalLoad,
    },
];

/// Diagnoses the dominant factor (by raw magnitude, not weighted
/// contribution -- the diagnosis asks "what is most wrong," the Phi_Enbi
/// score asks "how wrong overall") and looks it up in TERTU_TABLE.
pub fn diagnose(f: &EnbiFactors) -> (FailureMechanism, RootCause) {
    let vals = f.as_weighted_array();
    let dominant = vals
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    TERTU_TABLE
        .iter()
        .find(|e| e.dominant_factor_index == dominant)
        .map(|e| (e.mechanism, e.cause))
        .unwrap_or((FailureMechanism::Unknown, RootCause::Unknown))
}

/// Produces the exact compass sentence the system must speak, per the
/// Architect's law (2026-07-10):
/// "Element E, failing by mechanism M, from cause C, crossing ERRA in T
/// weeks -- and if expansion X proceeds, T shrinks to T'."
/// Falls back to a shorter form when no expansion scenario is supplied.
pub fn narrate(
    element: &str,
    mechanism: FailureMechanism,
    cause: RootCause,
    t_weeks: f32,
    expansion: Option<(&str, f32)>, // (expansion_name, t_prime_weeks)
) -> String {
    let base = format!(
        "Element {element}, failing by mechanism {mechanism:?}, from cause {cause:?}, crossing ERRA in {t_weeks:.1} weeks"
    );
    match expansion {
        Some((name, t_prime)) => {
            format!("{base} — and if expansion {name} proceeds, T shrinks to {t_prime:.1} weeks.")
        }
        None => format!("{base}."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stable_factors() -> EnbiFactors {
        EnbiFactors {
            factor_1: 10.0,
            factor_2: 10.0,
            factor_3: 10.0,
            baru_residual: 10.0,
            factor_5: 10.0,
        }
    }

    fn erra_factors() -> EnbiFactors {
        EnbiFactors {
            factor_1: 90.0,
            factor_2: 90.0,
            factor_3: 90.0,
            baru_residual: 90.0,
            factor_5: 90.0,
        }
    }

    #[test]
    fn phi_enbi_weights_sum_to_one() {
        let sum: f32 = PHI_ENBI_WEIGHTS.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn phi_enbi_uniform_inputs_equals_input_value() {
        // When all five factors are equal, the weighted sum (weights
        // summing to 1.0) must equal that shared value exactly.
        let f = EnbiFactors {
            factor_1: 42.0,
            factor_2: 42.0,
            factor_3: 42.0,
            baru_residual: 42.0,
            factor_5: 42.0,
        };
        assert!((phi_enbi(&f) - 42.0).abs() < 1e-4);
    }

    #[test]
    fn baru_residual_carries_the_most_weight() {
        // Two factor sets differing only in one factor's value: moving
        // baru_residual must shift Phi_Enbi more than moving any other
        // single factor by the same amount.
        let base = stable_factors();
        let mut bump_baru = base;
        bump_baru.baru_residual += 10.0;
        let mut bump_f1 = base;
        bump_f1.factor_1 += 10.0;

        let delta_baru = phi_enbi(&bump_baru) - phi_enbi(&base);
        let delta_f1 = phi_enbi(&bump_f1) - phi_enbi(&base);
        assert!(delta_baru > delta_f1);
        assert!((delta_baru - 3.0).abs() < 1e-4); // 0.30 * 10.0
    }

    #[test]
    fn tiamat_bands_match_confirmed_thresholds() {
        assert_eq!(TiamatBand::from_phi(0.0), TiamatBand::Stable);
        assert_eq!(TiamatBand::from_phi(39.9), TiamatBand::Stable);
        assert_eq!(TiamatBand::from_phi(40.0), TiamatBand::Watch);
        assert_eq!(TiamatBand::from_phi(59.9), TiamatBand::Watch);
        assert_eq!(TiamatBand::from_phi(60.0), TiamatBand::Serious);
        assert_eq!(TiamatBand::from_phi(79.9), TiamatBand::Serious);
        assert_eq!(TiamatBand::from_phi(80.0), TiamatBand::Erra);
        assert_eq!(TiamatBand::from_phi(100.0), TiamatBand::Erra);
    }

    #[test]
    fn stable_band_produces_no_alert() {
        assert_eq!(TiamatBand::Stable.to_alert_severity(), None);
    }

    #[test]
    fn erra_band_maps_to_critical_alert() {
        assert_eq!(
            TiamatBand::Erra.to_alert_severity(),
            Some(AlertSeverity::Critical)
        );
    }

    #[test]
    fn enbi_horizon_already_erra_is_zero_weeks() {
        assert_eq!(enbi_horizon_weeks(85.0, 2.0), Some(0.0));
    }

    #[test]
    fn enbi_horizon_zero_or_negative_rate_has_no_horizon() {
        assert_eq!(enbi_horizon_weeks(50.0, 0.0), None);
        assert_eq!(enbi_horizon_weeks(50.0, -1.0), None);
    }

    #[test]
    fn enbi_horizon_computes_correct_weeks() {
        // phi_now=60, rate=5/week -> (80-60)/5 = 4 weeks
        let t = enbi_horizon_weeks(60.0, 5.0).unwrap();
        assert!((t - 4.0).abs() < 1e-4);
    }

    #[test]
    fn expansion_reduces_rate_and_extends_horizon() {
        let t_before = enbi_horizon_weeks(60.0, 5.0).unwrap();
        let t_after = expansion_horizon_weeks(60.0, 2.0).unwrap(); // expansion halves the rate roughly
        assert!(t_after > t_before);
    }

    #[test]
    fn diagnose_picks_dominant_factor() {
        let f = EnbiFactors {
            factor_1: 5.0,
            factor_2: 5.0,
            factor_3: 5.0,
            baru_residual: 95.0,
            factor_5: 5.0,
        };
        let (mechanism, cause) = diagnose(&f);
        assert_eq!(mechanism, FailureMechanism::Subsidence);
        assert_eq!(cause, RootCause::SoilMovement);
    }

    #[test]
    fn narrate_produces_the_compass_sentence_with_expansion() {
        let s = narrate(
            "J-4471",
            FailureMechanism::Corrosion,
            RootCause::MaterialAge,
            6.0,
            Some(("X-Karrada-2", 14.0)),
        );
        assert_eq!(
            s,
            "Element J-4471, failing by mechanism Corrosion, from cause MaterialAge, crossing ERRA in 6.0 weeks — and if expansion X-Karrada-2 proceeds, T shrinks to 14.0 weeks."
        );
    }

    #[test]
    fn narrate_without_expansion_ends_with_period() {
        let s = narrate(
            "J-100",
            FailureMechanism::JointFailure,
            RootCause::ExternalLoad,
            2.0,
            None,
        );
        assert_eq!(
            s,
            "Element J-100, failing by mechanism JointFailure, from cause ExternalLoad, crossing ERRA in 2.0 weeks."
        );
    }

    #[test]
    fn end_to_end_erra_junction_produces_critical_alert_and_narration() {
        let f = erra_factors();
        let phi = phi_enbi(&f);
        let band = TiamatBand::from_phi(phi);
        assert_eq!(band, TiamatBand::Erra);
        assert_eq!(band.to_alert_severity(), Some(AlertSeverity::Critical));
        let (mechanism, cause) = diagnose(&f);
        let t = enbi_horizon_weeks(phi, 3.0).unwrap();
        assert_eq!(t, 0.0); // already past ERRA
        let sentence = narrate("J-Test", mechanism, cause, t, None);
        assert!(sentence.contains("crossing ERRA in 0.0 weeks"));
    }
}
