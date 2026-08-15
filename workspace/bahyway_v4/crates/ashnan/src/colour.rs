#![forbid(unsafe_code)]
//! ColourID B11 — sovereign colour encoding for ASHNAN particles (BC-AGRI-001 §4).
//!
//! B11 = round(H(orbital_position) x 240)
//!
//! 240 is derived from Plimpton 322 (sexagesimal base 60 x 4 = 240).
//! This is a SOVEREIGN CONSTANT — it is NEVER 255 (RGB byte-max).
//!
//! `orbital_hue` must be in [0.0, 1.0].

/// Sovereign ColourID base — derived from Plimpton 322. NEVER 255.
pub const PLIMPTON_322_BASE: f64 = 240.0;

/// Compute ColourID B11 from orbital position hue component.
///
/// `orbital_hue` — H component of the orbital position, in [0.0, 1.0].
/// Returns u8 in [0, 240].
pub fn colour_id_b11(orbital_hue: f64) -> u8 {
    let clamped = orbital_hue.clamp(0.0, 1.0);
    (clamped * PLIMPTON_322_BASE).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries() {
        assert_eq!(colour_id_b11(0.0), 0);
        assert_eq!(colour_id_b11(1.0), 240);
        assert_eq!(colour_id_b11(0.5), 120);
        assert!(colour_id_b11(1.0) < 255);
    }

    #[test]
    fn clamps_out_of_range_input() {
        assert_eq!(colour_id_b11(-1.0), 0);
        assert_eq!(colour_id_b11(2.0), 240);
    }
}
