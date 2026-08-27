//! AlertEngine — scans the ColorID Spatial Index and emits Alerts.

use crate::alert::{Alert, AlertSeverity, DriftCause};
use enkidb_indexes::colorid::ColorIdIndex;

pub struct AlertEngine {
    /// Threshold above which a particle is considered a Watch-level drifter.
    pub watch_threshold: f32,
    /// Threshold above which a particle is considered a Warning-level drifter.
    pub warning_threshold: f32,
    /// Threshold above which a particle is considered Critical.
    pub critical_threshold: f32,
}

impl Default for AlertEngine {
    fn default() -> Self {
        AlertEngine {
            watch_threshold: 50.0,
            warning_threshold: 150.0,
            critical_threshold: 300.0,
        }
    }
}

impl AlertEngine {
    pub fn new(watch: f32, warning: f32, critical: f32) -> Self {
        AlertEngine {
            watch_threshold: watch,
            warning_threshold: warning,
            critical_threshold: critical,
        }
    }

    /// Scan the ColorID index and return all drifting particles as Alerts.
    pub fn scan(&self, index: &ColorIdIndex) -> Vec<Alert> {
        let mut alerts = index
            .query_drift_exceeding(self.watch_threshold)
            .into_iter()
            .map(|(uuid_hash, drift)| {
                let severity = if drift >= self.critical_threshold {
                    AlertSeverity::Critical
                } else if drift >= self.warning_threshold {
                    AlertSeverity::Warning
                } else {
                    AlertSeverity::Watch
                };
                // Cause classification — simple heuristic; full Diagnosis Templates in v4.1
                let cause = DriftCause::Unknown;
                Alert::new(uuid_hash, severity, cause, drift)
            })
            .collect::<Vec<_>>();

        // Sort by drift_distance descending (most critical first)
        alerts.sort_by(|a, b| {
            b.drift_distance
                .partial_cmp(&a.drift_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        alerts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_indexes::colorid::{ColorIdIndex, ColorIdPoint};

    #[test]
    fn no_alerts_for_healthy_particles() {
        let mut idx = ColorIdIndex::new();
        idx.upsert(0x0001, ColorIdPoint::new(255, 255, 255, 0.0));
        let engine = AlertEngine::default();
        let alerts = engine.scan(&idx);
        assert!(alerts.is_empty());
    }

    #[test]
    fn critical_alert_for_extreme_drift() {
        let mut idx = ColorIdIndex::new();
        // Fully gray = maximally drifted
        idx.upsert(0x0001, ColorIdPoint::new(0, 0, 0, 500.0));
        let engine = AlertEngine::default();
        let alerts = engine.scan(&idx);
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
        assert_eq!(alerts[0].uuid_hash, 0x0001);
    }

    #[test]
    fn sorted_by_drift_descending() {
        let mut idx = ColorIdIndex::new();
        idx.upsert(0x0001, ColorIdPoint::new(100, 100, 100, 200.0)); // moderate
        idx.upsert(0x0002, ColorIdPoint::new(0, 0, 0, 400.0)); // severe
        let engine = AlertEngine::default();
        let alerts = engine.scan(&idx);
        assert!(alerts.len() >= 2);
        assert!(alerts[0].drift_distance >= alerts[1].drift_distance);
    }
}
