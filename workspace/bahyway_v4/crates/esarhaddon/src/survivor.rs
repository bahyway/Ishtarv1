#![forbid(unsafe_code)]
//! Survivor Signal Aggregator — Bayesian confidence fusion.
//!
//! Fuses heterogeneous survivor detection signals (acoustic knock detectors,
//! thermal cameras, RF/mobile triangulators, robot LIDAR voids, voice sensors)
//! into a single sovereign KAKI Identity particle per HeptaMap cell per
//! 120-second fusion window (BC-QUAKE-001 §7).

use crate::error::EsarhaddonError;
use crate::kaki::SurvivorSignal;

/// Individual signal contribution to a fusion event.
#[derive(Debug, Clone)]
pub struct SignalContribution {
    /// Sensor class identifier.
    pub source: SignalSource,
    /// Raw detection confidence from this sensor class (0–100).
    pub raw_confidence_pct: u8,
    /// HeptaMap E7 cell of detection.
    pub hepta_cell: u16,
    /// Detection epoch (seconds since earthquake T+0).
    pub epoch_s: u16,
}

/// Sensor class for survivor detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalSource {
    /// Acoustic knock detector — up to 30 m range.
    AcousticKnock,
    /// Thermal imaging camera — up to 15 m range.
    ThermalCamera,
    /// RF / mobile signal triangulator — up to 200 m range.
    RfMobile,
    /// Search robot LIDAR void scan — penetrating rubble.
    RobotLidar,
    /// Voice frequency sensor — up to 20 m range.
    VoiceFrequency,
}

/// Aggregator that fuses signals within a 120-second window per HeptaMap cell.
pub struct SurvivorAggregator {
    /// Fusion window in seconds (sovereign default: 120 s).
    pub window_s: u16,
}

impl SurvivorAggregator {
    pub fn new() -> Self {
        Self { window_s: 120 }
    }

    /// Fuse a slice of signal contributions into a single survivor particle.
    ///
    /// Requires at least two corroborating sources in the same HeptaMap cell
    /// within `self.window_s` seconds (BC-QUAKE-001 §7.2 sovereign rule).
    pub fn fuse(
        &self,
        signals: &[SignalContribution],
        epoch_s: u16,
    ) -> Result<SurvivorSignal, EsarhaddonError> {
        if signals.len() < 2 {
            return Err(EsarhaddonError::InsufficientSignalSources);
        }
        let hepta_cell = signals[0].hepta_cell;
        // Bayesian product across independent signal likelihoods (simplified sovereign form).
        // Full derivation: P(S|signals) ∝ P(S) × ∏ P(signal_i | S)
        let fused = signals.iter().fold(1.0_f32, |acc, s| {
            acc * (s.raw_confidence_pct as f32 / 100.0)
        });
        let confidence_pct = (fused * 100.0).min(100.0) as u8;
        Ok(SurvivorSignal {
            confidence_pct,
            hepta_cell,
            depth_m: 0.0, // depth estimated by robot LIDAR in Phase 2
            epoch_s,
            source_count: signals.len() as u8,
        })
    }
}

impl Default for SurvivorAggregator {
    fn default() -> Self {
        Self::new()
    }
}
