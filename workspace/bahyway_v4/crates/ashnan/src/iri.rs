#![forbid(unsafe_code)]
//! TIAMAT Engine 5 — Pest Incursion Risk Index (IRI).
//!
//! IRI = (W_p × F_degreeday × F_trap × F_host)^(1/3)
//! Calibrated for Sunn Pest (Eurygaster integriceps) and Iraqi wheat (BC-AGRI-001 §7).

use crate::error::AshnanError;

/// EAV state classification for pest incursion risk.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IriState {
    /// 0.00–0.30 — pre-emergence, normal seasonal background.
    Golden,
    /// 0.30–0.60 — early emergence predicted, pre-emptive scouting.
    Fuzzy,
    /// 0.60–0.85 — active incursion confirmed by trap data.
    Dead,
    /// 0.85–1.00 — regional outbreak, cross-governorate spread risk.
    DeadCritical,
}

/// Computed Pest Incursion Risk Index.
#[derive(Debug, Clone)]
pub struct Iri {
    /// Raw IRI score in [0.0, 1.0].
    pub score: f32,
    /// Derived EAV state.
    pub state: IriState,
    /// ColourID B11 hue: round(score × 240), never 255 (ADR-001).
    pub colour_id: u8,
}

impl Iri {
    /// Compute IRI from three sovereign incursion-risk factors.
    ///
    /// - `f_degreeday` — cumulative GDD vs species emergence threshold
    /// - `f_trap`      — pheromone trap catch rate and week-over-week acceleration
    /// - `f_host`      — host plant vulnerability (CMI × planted area density)
    ///
    /// All factors and weight in [0.0, 1.0].
    pub fn compute(
        weight: f32,
        f_degreeday: f32,
        f_trap: f32,
        f_host: f32,
    ) -> Result<Self, AshnanError> {
        for (name, val) in [
            ("f_degreeday", f_degreeday),
            ("f_trap", f_trap),
            ("f_host", f_host),
        ] {
            if !(0.0..=1.0).contains(&val) {
                return Err(AshnanError::InvalidFactor {
                    factor: name,
                    value: val,
                });
            }
        }
        let product = weight * f_degreeday * f_trap * f_host;
        let score = product.powf(1.0 / 3.0).clamp(0.0_f32, 1.0_f32);
        let state = match score {
            s if s < 0.30 => IriState::Golden,
            s if s < 0.60 => IriState::Fuzzy,
            s if s < 0.85 => IriState::Dead,
            _ => IriState::DeadCritical,
        };
        let colour_id = (score * 240.0).round().min(240.0) as u8;
        Ok(Self {
            score,
            state,
            colour_id,
        })
    }
}
