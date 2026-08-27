use crate::kaki::{derive_kaki, kaki_d1_hash};
use crate::types::{BodyType, Hepta, KakiPK, ZoneId};

// ── Timing helper ─────────────────────────────────────────────────
fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn temporal_norm(timestamp_ms: u64) -> f32 {
    const DAY_MS: u64 = 86_400_000;
    (timestamp_ms % DAY_MS) as f32 / DAY_MS as f32
}

// ── ParticleSignal ────────────────────────────────────────────────

/// Sovereign inter-crate thermal particle — one zone reading with full Hepta.
#[derive(Debug, Clone)]
pub struct ParticleSignal {
    pub kaki: KakiPK,
    pub zone: ZoneId,
    pub body_type: BodyType,
    pub temperature: f32, // °C measured
    pub baseline: f32,    // °C expected for this zone/body-type
    pub deviation: f32,   // temperature − baseline
    pub hepta: Hepta,
    pub hps: f32,          // cached HPS
    pub timestamp_ms: u64, // Unix epoch ms
    pub frame_id: u64,
}

impl ParticleSignal {
    pub fn new(zone: ZoneId, body_type: BodyType, temperature: f32, frame_id: u64) -> Self {
        Self::with_timestamp(zone, body_type, temperature, frame_id, now_ms())
    }

    pub(crate) fn with_timestamp(
        zone: ZoneId,
        body_type: BodyType,
        temperature: f32,
        frame_id: u64,
        timestamp_ms: u64,
    ) -> Self {
        let baseline = zone.baseline(body_type);
        let deviation = temperature - baseline;
        let abs_dev = deviation.abs();

        let hepta = Hepta {
            d1_identity: kaki_d1_hash(zone as u8, frame_id),
            d2_belonging: zone.tribe().norm(),
            d3_domain: (abs_dev / 2.0).min(1.0),
            d4_quality: (1.0 - abs_dev / 2.5).max(0.0),
            d5_freshness: 0.95,
            d6_temporal: temporal_norm(timestamp_ms),
            d7_integrity: (1.0 - abs_dev / 3.5).max(0.0),
        };

        let hps = hepta.hps();
        let kaki = derive_kaki(&hepta, zone as u8, frame_id);

        ParticleSignal {
            kaki,
            zone,
            body_type,
            temperature,
            baseline,
            deviation,
            hepta,
            hps,
            timestamp_ms,
            frame_id,
        }
    }
}

// ── BodyScan ──────────────────────────────────────────────────────

/// Full 16-zone sovereign body scan — one complete thermal frame.
#[derive(Debug, Clone)]
pub struct BodyScan {
    pub scan_id: KakiPK,
    pub body_type: BodyType,
    pub frame_id: u64,
    pub zones: [ParticleSignal; 16],
    pub timestamp_ms: u64,
}

impl BodyScan {
    /// Build from a 16-element temperature array (one per ZoneId index).
    pub fn from_temperatures(temperatures: &[f32; 16], body_type: BodyType, frame_id: u64) -> Self {
        let timestamp_ms = now_ms();
        let zones = ZoneId::ALL.map(|z| {
            ParticleSignal::with_timestamp(
                z,
                body_type,
                temperatures[z as usize],
                frame_id,
                timestamp_ms,
            )
        });
        let scan_id = derive_kaki(&zones[0].hepta, frame_id as u8, frame_id);
        BodyScan {
            scan_id,
            body_type,
            frame_id,
            zones,
            timestamp_ms,
        }
    }

    /// All zones at their sovereign baseline temperatures (zero deviation).
    ///
    /// Useful for tests — creates a physiologically "normal" scan.
    pub fn uniform(body_type: BodyType, frame_id: u64) -> Self {
        let temps = ZoneId::ALL.map(|z| z.baseline(body_type));
        Self::from_temperatures(&temps, body_type, frame_id)
    }

    /// Build with explicit per-zone deviations from sovereign baselines.
    ///
    /// Non-specified zones get zero deviation (baseline temperature).
    /// Replaces the v354 `with_deviations(baseline, overrides)` helper — uses
    /// zone-specific baselines rather than a single flat baseline.
    pub fn from_deviations(
        body_type: BodyType,
        deviations: &[(ZoneId, f32)],
        frame_id: u64,
    ) -> Self {
        let temps = ZoneId::ALL.map(|z| {
            let baseline = z.baseline(body_type);
            let dev = deviations
                .iter()
                .find(|&&(z2, _)| z2 == z)
                .map(|&(_, d)| d)
                .unwrap_or(0.0);
            baseline + dev
        });
        Self::from_temperatures(&temps, body_type, frame_id)
    }

    pub fn zone(&self, id: ZoneId) -> &ParticleSignal {
        &self.zones[id as usize]
    }

    /// Aggregate TRS across torso + arm zones (security-focus).
    pub fn trs(&self) -> f32 {
        let focus = [
            ZoneId::Chest,
            ZoneId::Abdomen,
            ZoneId::LShoulderZ,
            ZoneId::RShoulderZ,
            ZoneId::LUpperArm,
            ZoneId::RUpperArm,
        ];
        focus.iter().map(|z| self.zone(*z).hps).sum::<f32>() / focus.len() as f32
    }

    /// Mean deviation across all 16 zones.
    pub fn mean_deviation(&self) -> f32 {
        self.zones.iter().map(|p| p.deviation).sum::<f32>() / 16.0
    }

    /// Left-right bilateral symmetry score (1.0 = perfect symmetry).
    pub fn symmetry_score(&self) -> f32 {
        let pairs = [
            (ZoneId::LShoulderZ, ZoneId::RShoulderZ),
            (ZoneId::LUpperArm, ZoneId::RUpperArm),
            (ZoneId::LForearm, ZoneId::RForearm),
            (ZoneId::LHip, ZoneId::RHip),
            (ZoneId::LThigh, ZoneId::RThigh),
            (ZoneId::LShin, ZoneId::RShin),
        ];
        let mean_diff = pairs
            .iter()
            .map(|(l, r)| (self.zone(*l).temperature - self.zone(*r).temperature).abs())
            .sum::<f32>()
            / pairs.len() as f32;
        (1.0 - (mean_diff / 2.0)).clamp(0.0, 1.0)
    }
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BodyType, ZoneId};

    #[test]
    fn uniform_scan_has_zero_deviation() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        assert!(
            scan.mean_deviation().abs() < 1e-4,
            "mean_dev={}",
            scan.mean_deviation()
        );
        assert!(scan.zone(ZoneId::Head).deviation.abs() < 1e-4);
    }

    #[test]
    fn from_deviations_sets_correct_deviation() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::Chest, 1.0)], 0);
        let r = scan.zone(ZoneId::Chest);
        assert!(
            (r.deviation - 1.0).abs() < 1e-4,
            "deviation={}",
            r.deviation
        );
        // temperature = Chest baseline (36.6) + 1.0 = 37.6
        assert!(
            (r.temperature - 37.6).abs() < 1e-4,
            "temperature={}",
            r.temperature
        );
    }

    #[test]
    fn non_overridden_zone_has_zero_deviation() {
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::Head, 2.0)], 0);
        assert!(scan.zone(ZoneId::Neck).deviation.abs() < 1e-4);
    }

    #[test]
    fn mean_deviation_single_zone_elevated() {
        // Head elevated 1.6°C across 16 zones → mean = 1.6/16 = 0.1
        let scan = BodyScan::from_deviations(BodyType::Man, &[(ZoneId::Head, 1.6)], 0);
        let expected = 1.6 / 16.0;
        assert!(
            (scan.mean_deviation() - expected).abs() < 1e-4,
            "mean_dev={:.4}",
            scan.mean_deviation()
        );
    }

    #[test]
    fn uniform_scan_is_perfectly_symmetric() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        assert!(
            (scan.symmetry_score() - 1.0).abs() < 1e-4,
            "symmetry={}",
            scan.symmetry_score()
        );
    }

    #[test]
    fn particle_hps_in_range() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        for p in &scan.zones {
            assert!(p.hps >= 0.0 && p.hps <= 1.0, "hps={}", p.hps);
        }
    }

    #[test]
    fn particle_baseline_matches_zone_baseline() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        for p in &scan.zones {
            assert!((p.baseline - p.zone.baseline_man()).abs() < 1e-5);
        }
    }

    #[test]
    fn trs_in_range() {
        let scan = BodyScan::uniform(BodyType::Man, 0);
        let trs = scan.trs();
        assert!(trs >= 0.0 && trs <= 1.0, "trs={trs}");
    }
}
