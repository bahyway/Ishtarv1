#![forbid(unsafe_code)]
//! TIAMAT Engine 5 — Structural Mortality Index (SMI).
//!
//! SMI = (W_s × F_acc × F_tilt × F_crack × F_aftershock)^(1/4)
//! as specified in BC-QUAKE-001 §6.

use crate::error::EsarhaddonError;

/// EAV state classification for a structural cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmiState {
    /// 0.00–0.25 — structure stable, rescue entry cleared.
    Golden,
    /// 0.25–0.60 — elevated risk, timed entry only.
    Fuzzy,
    /// 0.60–0.85 — high collapse probability, remote operations only.
    Dead,
    /// 0.85–1.00 — imminent collapse, evacuate 50 m radius.
    DeadCritical,
}

/// Computed Structural Mortality Index for a single HeptaMap cell.
#[derive(Debug, Clone)]
pub struct Smi {
    /// Raw SMI score in [0.0, 1.0].
    pub score: f32,
    /// Derived EAV state.
    pub state: SmiState,
    /// ColourID B11 hue: round(score × 240), clamped to [0, 240].
    pub colour_id: u8,
}

impl Smi {
    /// Compute SMI from four sovereign failure-vector factors.
    ///
    /// All factor arguments are probabilities in `[0.0, 1.0]`.
    /// `weight` is a sovereign tuning coefficient (defaults to 1.0).
    pub fn compute(
        weight: f32,
        f_acc: f32,
        f_tilt: f32,
        f_crack: f32,
        f_aftershock: f32,
    ) -> Result<Self, EsarhaddonError> {
        if f_acc < 0.0 || f_tilt < 0.0 || f_crack < 0.0 || f_aftershock < 0.0 {
            return Err(EsarhaddonError::InsufficientSensorData);
        }
        let product = weight * f_acc * f_tilt * f_crack * f_aftershock;
        let score = product.powf(0.25).min(1.0_f32).max(0.0_f32);
        let state = match score {
            s if s < 0.25 => SmiState::Golden,
            s if s < 0.60 => SmiState::Fuzzy,
            s if s < 0.85 => SmiState::Dead,
            _             => SmiState::DeadCritical,
        };
        // ColourID: round(score × 240), never 255 (ADR-001)
        let colour_id = (score * 240.0).round().min(240.0) as u8;
        Ok(Self { score, state, colour_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_state_below_025() {
        let smi = Smi::compute(1.0, 0.1, 0.1, 0.1, 0.1).unwrap();
        assert_eq!(smi.state, SmiState::Golden);
        assert!(smi.score < 0.25);
    }

    #[test]
    fn dead_critical_near_one() {
        let smi = Smi::compute(1.0, 1.0, 1.0, 1.0, 1.0).unwrap();
        assert_eq!(smi.state, SmiState::DeadCritical);
        assert_eq!(smi.colour_id, 240);
    }
}
