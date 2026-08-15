//! Horizon verb (BC-MRD-001 §3.1 item 3): T_golden, the predicted time
//! until a diverging particle exits the GEM band. Same trend-to-
//! threshold primitive as Gibil's horizon (`trend-core`), wrapped with
//! Marduk's own sign convention.
//!
//! Sign-convention law (§3.6): MDM/Marduk domains treat the golden
//! core as the attractor -- B11 FALLS as a particle degrades, which is
//! the opposite direction from Gibil/Enbilulu (where kappa RISES as an
//! asset degrades). `trend_core::time_to_threshold` only knows "rising
//! toward a threshold from below"; `golden_horizon` mirrors the series
//! so a falling B11 is expressed in that same shape, rather than
//! duplicating the least-squares fit with the sign hardcoded the other
//! way.

/// The floor below which a particle exits the GEM band (BC-MRD-001 §3.1).
pub const GEM_FLOOR_B11: f64 = 200.0;

/// Predicted days (or whatever time unit the caller's samples use)
/// until a particle's B11 trend crosses below `GEM_FLOOR_B11`.
/// `Ok(Some(0.0))` means already below the floor now.
pub fn golden_horizon(
    b11_history: &[trend_core::Sample],
) -> Result<Option<f64>, trend_core::TrendError> {
    let mirrored: Vec<trend_core::Sample> = b11_history.iter().map(|&(t, v)| (t, -v)).collect();
    trend_core::time_to_threshold(&mirrored, -GEM_FLOOR_B11)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declining_b11_predicts_a_future_crossing() {
        // B11 falling 1.0/day, now at 220, floor 200 -> ~20 days.
        let h: Vec<trend_core::Sample> =
            (0..30).map(|i| (-(30 - i) as f64, 220.0 + (30 - i) as f64 * 1.0)).collect();
        match golden_horizon(&h) {
            Ok(Some(d)) => assert!((d - 20.0).abs() < 2.0, "expected ~20, got {d}"),
            other => panic!("expected Some(~20), got {other:?}"),
        }
    }

    #[test]
    fn stable_high_b11_never_crosses() {
        let h: Vec<trend_core::Sample> = (0..30).map(|i| (-(i as f64), 900.0)).collect();
        assert_eq!(golden_horizon(&h), Ok(None));
    }

    #[test]
    fn b11_already_below_floor_is_zero() {
        let h: Vec<trend_core::Sample> = vec![(-10.0, 150.0), (-5.0, 150.0), (0.0, 150.0)];
        assert_eq!(golden_horizon(&h), Ok(Some(0.0)));
    }
}
