//! Body domain types — canonical source is nusku-engine.
//! This module re-exports the shared types so the rest of azuga-engine
//! can use `crate::body::*` without depending on nusku-engine directly.

pub use nusku_engine::{BodyScan, BodyTribe, BodyType, ZoneId};

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use nusku_engine::{BodyScan, BodyType, ZoneId};

    #[test]
    fn from_deviations_sets_correct_deviation() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::Chest, 1.0)], 0);
        let r = scan.zone(ZoneId::Chest);
        // Chest Man baseline = 36.6; temperature = 37.6
        assert!((r.temperature - 37.6).abs() < 1e-4, "temperature={}", r.temperature);
        assert!((r.deviation - 1.0).abs() < 1e-4, "deviation={}", r.deviation);
    }

    #[test]
    fn non_overridden_zone_has_zero_deviation() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::Head, 2.0)], 0);
        assert!(scan.zone(ZoneId::Neck).deviation.abs() < 1e-4);
    }

    #[test]
    fn mean_deviation_uniform_zero() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        assert!(scan.mean_deviation().abs() < 1e-4,
            "mean_dev={}", scan.mean_deviation());
    }

    #[test]
    fn mean_deviation_single_zone_elevated() {
        // Head elevated 1.6°C; mean = 1.6/16 = 0.1
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::Head, 1.6)], 0);
        let expected = 1.6 / 16.0;
        assert!((scan.mean_deviation() - expected).abs() < 1e-4,
            "mean_dev={:.4}", scan.mean_deviation());
    }
}
