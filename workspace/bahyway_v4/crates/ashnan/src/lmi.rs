#![forbid(unsafe_code)]
//! TIAMAT Engine 5 — Livestock Mortality Index (LMI).
//!
//! LMI = (W_l × F_thi × F_forage × F_disease)^(1/3)
//! Uses the standard veterinary Temperature-Humidity Index (THI) as
//! its physiological core (BC-AGRI-001 §8).

use crate::error::AshnanError;

/// EAV state classification for herd health.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LmiState {
    /// 0.00–0.25 — normal activity, rumination, and temperature.
    Golden,
    /// 0.25–0.55 — elevated body temp, reduced rumination — early heat stress.
    Fuzzy,
    /// 0.55–0.80 — severe heat stress, reduced intake, disease vulnerability rising.
    Dead,
    /// 0.80–1.00 — active mortality event or imminent collapse.
    DeadCritical,
}

/// Computed Livestock Mortality Index for a herd unit.
#[derive(Debug, Clone)]
pub struct Lmi {
    /// Raw LMI score in [0.0, 1.0].
    pub score: f32,
    /// Derived EAV state.
    pub state: LmiState,
    /// ColourID B11 hue: round(score × 240), never 255 (ADR-001).
    pub colour_id: u8,
}

impl Lmi {
    /// Compute LMI from three sovereign livestock-health factors.
    ///
    /// - `f_thi`     — THI weighted by cumulative daily exposure hours above species threshold
    /// - `f_forage`  — forage moisture and biomass vs required intake (shared with CMI stream)
    /// - `f_disease` — vector-borne disease range expansion risk weighted by herd density
    ///
    /// All factors and weight in [0.0, 1.0].
    pub fn compute(
        weight: f32,
        f_thi: f32,
        f_forage: f32,
        f_disease: f32,
    ) -> Result<Self, AshnanError> {
        for (name, val) in [("f_thi", f_thi), ("f_forage", f_forage), ("f_disease", f_disease)] {
            if !(0.0..=1.0).contains(&val) {
                return Err(AshnanError::InvalidFactor { factor: name, value: val });
            }
        }
        let product = weight * f_thi * f_forage * f_disease;
        let score = product.powf(1.0 / 3.0).min(1.0_f32).max(0.0_f32);
        let state = match score {
            s if s < 0.25 => LmiState::Golden,
            s if s < 0.55 => LmiState::Fuzzy,
            s if s < 0.80 => LmiState::Dead,
            _             => LmiState::DeadCritical,
        };
        let colour_id = (score * 240.0).round().min(240.0) as u8;
        Ok(Self { score, state, colour_id })
    }

    /// Compute the standard veterinary Temperature-Humidity Index (THI).
    ///
    /// THI = T_dry + 0.36 × T_dew + 41.2  (Kibler, 1964 formula)
    /// where T_dry = dry-bulb temperature (°C), T_dew = dew-point (°C).
    pub fn thi(t_dry_c: f32, t_dew_c: f32) -> f32 {
        t_dry_c + 0.36 * t_dew_c + 41.2
    }
}
