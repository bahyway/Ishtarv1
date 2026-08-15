//! The Gibil horizon: predicted date an asset's kappa trajectory
//! crosses the defect threshold. Trend-fitting itself is delegated to
//! `trend-core` (shared with Marduk's T_golden and, eventually, any
//! other domain that needs "time until this metric crosses a line") —
//! this module supplies only the domain meaning: the 90-day
//! maintenance-window law (GL-EGD-001) and the Sound/Horizon/Birqu
//! tiers rendered on the DubSar stage.

pub const MAINTENANCE_WINDOW_DAYS: f64 = 90.0;

/// GL-EGD-001 tier for the stage.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tier {
    Sound,
    Horizon(f64),
    Birqu,
}

/// (t_days, kappa) history for one asset, t relative to now
/// (negative = past).
pub fn tier(history: &[trend_core::Sample], threshold: f64) -> Tier {
    match trend_core::time_to_threshold(history, threshold) {
        Ok(Some(d)) if d <= 0.0 => Tier::Birqu,
        Ok(Some(d)) if d <= MAINTENANCE_WINDOW_DAYS => Tier::Horizon(d),
        _ => Tier::Sound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrading_asset_enters_window() {
        // kappa rising 0.01/day, now at 0.5, threshold 1.0
        let h: Vec<trend_core::Sample> =
            (0..60).map(|i| (-(60 - i) as f64, 0.5 - (60 - i) as f64 * 0.01)).collect();
        match tier(&h, 1.0) {
            Tier::Horizon(d) => assert!((d - 50.0).abs() < 2.0),
            t => panic!("expected Horizon, got {:?}", t),
        }
    }

    #[test]
    fn stable_asset_stays_sound() {
        let h: Vec<trend_core::Sample> = (0..60).map(|i| (-(i as f64), 0.3)).collect();
        assert_eq!(tier(&h, 1.0), Tier::Sound);
    }

    #[test]
    fn asset_over_threshold_now_is_birqu() {
        let h: Vec<trend_core::Sample> = vec![(-10.0, 2.0), (-5.0, 2.0), (0.0, 2.0)];
        assert_eq!(tier(&h, 1.0), Tier::Birqu);
    }
}
