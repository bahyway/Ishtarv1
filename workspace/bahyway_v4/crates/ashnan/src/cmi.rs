#![forbid(unsafe_code)]
//! TIAMAT Engine 5 — Crop Mortality Index (CMI).
//!
//! CMI = (W_c × F_soil × F_heat × F_water × F_phenology)^(1/4)
//! Calibrated for Iraqi staple cereals: wheat and barley (BC-AGRI-001 §6).

use crate::error::AshnanError;

/// EAV state classification for a crop zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmiState {
    /// 0.00–0.25 — healthy growth, on-track yield.
    Golden,
    /// 0.25–0.55 — moderate stress, irrigation intervention window open.
    Fuzzy,
    /// 0.55–0.80 — severe stress, yield loss likely without intervention.
    Dead,
    /// 0.80–1.00 — crop failure imminent or occurring.
    DeadCritical,
}

/// Computed Crop Mortality Index for a field zone.
#[derive(Debug, Clone)]
pub struct Cmi {
    /// Raw CMI score in [0.0, 1.0].
    pub score: f32,
    /// Derived EAV state.
    pub state: CmiState,
    /// ColourID B11 hue: round(score × 240), never 255 (ADR-001).
    pub colour_id: u8,
}

impl Cmi {
    /// Compute CMI from four sovereign failure-vector factors.
    ///
    /// - `f_soil`      — soil moisture deficit relative to crop wilting point, penalised by salinity
    /// - `f_heat`      — canopy temperature differential (stomatal closure indicator)
    /// - `f_water`     — river/irrigation flow vs crop water requirement for current stage
    /// - `f_phenology` — growth-stage-weighted vulnerability (grain fill >> vegetative)
    ///
    /// All factors and weight in [0.0, 1.0].
    pub fn compute(
        weight: f32,
        f_soil: f32,
        f_heat: f32,
        f_water: f32,
        f_phenology: f32,
    ) -> Result<Self, AshnanError> {
        for (name, val) in [("f_soil", f_soil), ("f_heat", f_heat),
                             ("f_water", f_water), ("f_phenology", f_phenology)] {
            if !(0.0..=1.0).contains(&val) {
                return Err(AshnanError::InvalidFactor { factor: name, value: val });
            }
        }
        let product = weight * f_soil * f_heat * f_water * f_phenology;
        let score = product.powf(0.25).min(1.0_f32).max(0.0_f32);
        let state = match score {
            s if s < 0.25 => CmiState::Golden,
            s if s < 0.55 => CmiState::Fuzzy,
            s if s < 0.80 => CmiState::Dead,
            _             => CmiState::DeadCritical,
        };
        let colour_id = (score * 240.0).round().min(240.0) as u8;
        Ok(Self { score, state, colour_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn healthy_crop_golden() {
        let cmi = Cmi::compute(1.0, 0.1, 0.1, 0.1, 0.1).unwrap();
        assert_eq!(cmi.state, CmiState::Golden);
    }

    #[test]
    fn full_failure_dead_critical() {
        let cmi = Cmi::compute(1.0, 1.0, 1.0, 1.0, 1.0).unwrap();
        assert_eq!(cmi.state, CmiState::DeadCritical);
        assert_eq!(cmi.colour_id, 240);
    }

    #[test]
    fn invalid_factor_rejected() {
        assert!(Cmi::compute(1.0, -0.1, 0.5, 0.5, 0.5).is_err());
    }
}
