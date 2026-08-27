//! Metric registry with sealed threshold laws. Every sample is destined to
//! become a KAKI Event particle (kaki_type 0x02) in EnkiODB (separate,
//! later wiring) -- this in-memory registry is only the live window for
//! the lamp: the observer-independence law says the display never depends
//! on EnkiODB availability.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Band {
    Green,
    Amber,
    Red,
}

#[derive(Clone, Copy, Debug)]
pub struct Thresholds {
    pub amber: f64,
    pub red: f64,
    /// true when HIGHER is worse (lag, anomaly rate); false when LOWER is
    /// worse (throughput, fidelity).
    pub high_is_bad: bool,
}

impl Thresholds {
    pub fn band(&self, v: f64) -> Band {
        if self.high_is_bad {
            if v >= self.red {
                Band::Red
            } else if v >= self.amber {
                Band::Amber
            } else {
                Band::Green
            }
        } else if v <= self.red {
            Band::Red
        } else if v <= self.amber {
            Band::Amber
        } else {
            Band::Green
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub value: f64,
    pub ts_millis: u64,
}

#[derive(Default)]
pub struct Registry {
    pub series: HashMap<String, Vec<Sample>>,
}

pub type Shared = Arc<Mutex<Registry>>;

impl Registry {
    pub fn push(&mut self, name: &str, value: f64, ts_millis: u64) {
        let s = self.series.entry(name.to_string()).or_default();
        s.push(Sample { value, ts_millis });
        let len = s.len();
        if len > 3600 {
            s.drain(0..len - 3600); // keep last hour @1Hz
        }
    }

    pub fn latest(&self, name: &str) -> Option<f64> {
        self.series
            .get(name)
            .and_then(|s| s.last())
            .map(|s| s.value)
    }
}

/// Sealed gauge laws, carried over from the review's ShamashEngine/
/// dagan-engine dashboard design: ingest throughput, CQRS lag, quarantine
/// ratio, projection fidelity, anomaly rate, cluster churn, plus host
/// vitals.
pub fn gauge_laws() -> Vec<(&'static str, &'static str, Thresholds)> {
    vec![
        (
            "ingest_pps",
            "Ingest particles/sec",
            Thresholds {
                amber: 1.0e9,
                red: 1.0e8,
                high_is_bad: false,
            },
        ),
        (
            "cqrs_lag_ms",
            "CQRS lag (ms)",
            Thresholds {
                amber: 100.0,
                red: 1000.0,
                high_is_bad: true,
            },
        ),
        (
            "quarantine_ratio",
            "Quarantine ratio",
            Thresholds {
                amber: 0.02,
                red: 0.10,
                high_is_bad: true,
            },
        ),
        (
            "projection_fidelity",
            "Projection fidelity",
            Thresholds {
                amber: 0.85,
                red: 0.60,
                high_is_bad: false,
            },
        ),
        (
            "anomaly_rate",
            "Anomaly rate (>3sigma)",
            Thresholds {
                amber: 0.005,
                red: 0.03,
                high_is_bad: true,
            },
        ),
        (
            "cluster_churn",
            "Cluster churn",
            Thresholds {
                amber: 0.01,
                red: 0.05,
                high_is_bad: true,
            },
        ),
        (
            "host_cpu",
            "Host CPU load",
            Thresholds {
                amber: 0.75,
                red: 0.90,
                high_is_bad: true,
            },
        ),
        (
            "host_mem",
            "Host memory used",
            Thresholds {
                amber: 0.80,
                red: 0.95,
                high_is_bad: true,
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_for_high_is_bad_metric() {
        let t = Thresholds {
            amber: 100.0,
            red: 1000.0,
            high_is_bad: true,
        };
        assert_eq!(t.band(10.0), Band::Green);
        assert_eq!(t.band(150.0), Band::Amber);
        assert_eq!(t.band(1500.0), Band::Red);
    }

    #[test]
    fn band_for_low_is_bad_metric() {
        let t = Thresholds {
            amber: 0.85,
            red: 0.60,
            high_is_bad: false,
        };
        assert_eq!(t.band(0.99), Band::Green);
        assert_eq!(t.band(0.70), Band::Amber);
        assert_eq!(t.band(0.30), Band::Red);
    }

    #[test]
    fn registry_keeps_only_last_hour_at_1hz() {
        let mut r = Registry::default();
        for i in 0..4000 {
            r.push("m", i as f64, i as u64 * 1000);
        }
        assert_eq!(r.series.get("m").unwrap().len(), 3600);
        // Latest value should be the most recent push, not evicted.
        assert_eq!(r.latest("m"), Some(3999.0));
    }

    #[test]
    fn latest_of_unknown_series_is_none() {
        let r = Registry::default();
        assert_eq!(r.latest("nope"), None);
    }

    #[test]
    fn eight_gauges_sealed() {
        assert_eq!(gauge_laws().len(), 8);
    }
}
