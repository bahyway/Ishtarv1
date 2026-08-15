//! collapse_predictor.rs — Jordan spectral radius trend analysis.
//!
//! Monitors the spectral radius of a tribe over time and predicts
//! when it will drop below the collapse threshold (0.50).
#![forbid(unsafe_code)]

use ea_agent_core::constants::{JORDAN_STABILITY_THRESHOLD, JORDAN_COLLAPSE_THRESHOLD};

/// A single measurement point in the spectral radius trend.
#[derive(Debug, Clone)]
pub struct TrendPoint {
    pub epoch:           u64,
    pub spectral_radius: f64,
    pub particle_count:  usize,
}

/// Collapse risk level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CollapseRisk {
    /// Spectral radius stable and rising — no risk.
    None,
    /// Spectral radius declining slowly — monitor.
    Low,
    /// Spectral radius declining — intervention needed.
    Medium,
    /// Spectral radius approaching collapse threshold — urgent.
    High,
    /// Spectral radius below collapse threshold — collapse imminent.
    Critical,
}

impl CollapseRisk {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None     => "✅ NONE — Tribe is stable",
            Self::Low      => "🟡 LOW — Monitor trend",
            Self::Medium   => "🟠 MEDIUM — Intervention needed",
            Self::High     => "🔴 HIGH — Urgent action required",
            Self::Critical => "💀 CRITICAL — Collapse imminent",
        }
    }
    pub fn severity(self) -> u8 {
        match self {
            Self::None     => 0,
            Self::Low      => 2,
            Self::Medium   => 5,
            Self::High     => 8,
            Self::Critical => 10,
        }
    }
}

/// Collapse prediction result.
#[derive(Debug)]
pub struct CollapsePrediction {
    pub tribe_id:            u16,
    pub current_sr:          f64,
    pub trend_slope:         f64,  // negative = declining
    pub epochs_to_collapse:  Option<u64>,
    pub risk:                CollapseRisk,
    pub recommendation:      String,
}

impl CollapsePrediction {
    pub fn is_urgent(&self) -> bool {
        self.risk >= CollapseRisk::High
    }

    pub fn summary(&self) -> String {
        let eta = match self.epochs_to_collapse {
            Some(e) => format!("ETA collapse: {} epochs", e),
            None    => "No collapse predicted".into(),
        };
        format!("Tribe 0x{:04X} | ρ={:.4} | slope={:+.4} | {} | {}",
            self.tribe_id, self.current_sr, self.trend_slope,
            self.risk.as_str(), eta)
    }
}

/// The collapse predictor — linear regression over spectral radius history.
pub struct CollapsePredictor {
    history: Vec<TrendPoint>,
    max_history: usize,
}

impl CollapsePredictor {
    pub fn new(max_history: usize) -> Self {
        Self { history: Vec::new(), max_history }
    }

    /// Add a new spectral radius measurement.
    pub fn record(&mut self, epoch: u64, spectral_radius: f64, particle_count: usize) {
        self.history.push(TrendPoint { epoch, spectral_radius, particle_count });
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Predict collapse risk from the current trend.
    pub fn predict(&self, tribe_id: u16) -> CollapsePrediction {
        if self.history.is_empty() {
            return CollapsePrediction {
                tribe_id, current_sr: 0.0, trend_slope: 0.0,
                epochs_to_collapse: None, risk: CollapseRisk::None,
                recommendation: "No data yet.".into(),
            };
        }

        let current_sr = self.history.last().unwrap().spectral_radius;

        if self.history.len() < 2 {
            let risk = self.risk_from_sr(current_sr);
            return CollapsePrediction {
                tribe_id, current_sr, trend_slope: 0.0,
                epochs_to_collapse: None, risk,
                recommendation: self.recommend(risk, 0.0),
            };
        }

        // Linear regression: slope of spectral radius over epochs
        let slope = self.compute_slope();
        let risk  = self.risk_from_trend(current_sr, slope);

        // Predict epochs until collapse threshold
        let epochs_to_collapse = if slope < -1e-10 && current_sr > JORDAN_COLLAPSE_THRESHOLD {
            let epochs = ((current_sr - JORDAN_COLLAPSE_THRESHOLD) / (-slope)) as u64;
            Some(epochs)
        } else {
            None
        };

        let recommendation = self.recommend(risk, slope);

        CollapsePrediction {
            tribe_id, current_sr, trend_slope: slope,
            epochs_to_collapse, risk, recommendation,
        }
    }

    /// Linear regression slope (change in spectral radius per epoch).
    fn compute_slope(&self) -> f64 {
        let n = self.history.len() as f64;
        let sum_x:  f64 = self.history.iter().map(|p| p.epoch as f64).sum();
        let sum_y:  f64 = self.history.iter().map(|p| p.spectral_radius).sum();
        let sum_xy: f64 = self.history.iter().map(|p| p.epoch as f64 * p.spectral_radius).sum();
        let sum_x2: f64 = self.history.iter().map(|p| (p.epoch as f64).powi(2)).sum();
        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-10 { return 0.0; }
        (n * sum_xy - sum_x * sum_y) / denom
    }

    fn risk_from_sr(&self, sr: f64) -> CollapseRisk {
        if sr < JORDAN_COLLAPSE_THRESHOLD  { CollapseRisk::Critical }
        else if sr < JORDAN_STABILITY_THRESHOLD { CollapseRisk::Medium }
        else { CollapseRisk::None }
    }

    fn risk_from_trend(&self, sr: f64, slope: f64) -> CollapseRisk {
        if sr < JORDAN_COLLAPSE_THRESHOLD            { CollapseRisk::Critical }
        else if sr < JORDAN_STABILITY_THRESHOLD {
            if slope < -0.01 { CollapseRisk::High }
            else             { CollapseRisk::Medium }
        }
        else if slope < -0.05 { CollapseRisk::High }
        else if slope < -0.01 { CollapseRisk::Medium }
        else if slope < -0.001{ CollapseRisk::Low }
        else                  { CollapseRisk::None }
    }

    fn recommend(&self, risk: CollapseRisk, slope: f64) -> String {
        match risk {
            CollapseRisk::None     => "Tribe is sovereign. Continue monitoring.".into(),
            CollapseRisk::Low      => format!("Slow decay detected (slope={slope:+.4}). Review D5 Freshness."),
            CollapseRisk::Medium   => "Orbit decay accelerating. Run VGCA cleansing on low-B11 particles.".into(),
            CollapseRisk::High     => "URGENT: Invoke Data Steward station. Apply DQM remediation immediately.".into(),
            CollapseRisk::Critical => "CRITICAL: Invoke Quantum Freeze protocol. Archive tribe to EnkiDW.".into(),
        }
    }

    pub fn history(&self) -> &[TrendPoint] { &self.history }
    pub fn data_points(&self) -> usize { self.history.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_trend_no_risk() {
        let mut p = CollapsePredictor::new(10);
        for i in 0..5 { p.record(i, 0.95, 10); }
        let pred = p.predict(0x0001);
        assert!(pred.risk <= CollapseRisk::Low);
        assert!(pred.epochs_to_collapse.is_none());
    }

    #[test]
    fn declining_trend_predicts_collapse() {
        let mut p = CollapsePredictor::new(10);
        for i in 0..10u64 {
            p.record(i, 0.90 - i as f64 * 0.05, 10);
        }
        let pred = p.predict(0x0001);
        assert!(pred.trend_slope < 0.0, "slope should be negative");
        assert!(pred.risk >= CollapseRisk::Medium);
    }

    #[test]
    fn critical_sr_is_critical_risk() {
        let mut p = CollapsePredictor::new(10);
        p.record(1, 0.30, 5);
        p.record(2, 0.28, 5);
        let pred = p.predict(0x0001);
        assert_eq!(pred.risk, CollapseRisk::Critical);
    }

    #[test]
    fn empty_predictor_returns_none_risk() {
        let p = CollapsePredictor::new(10);
        let pred = p.predict(0x0001);
        assert_eq!(pred.risk, CollapseRisk::None);
    }

    #[test]
    fn slope_computed_correctly() {
        let mut p = CollapsePredictor::new(10);
        // Perfect linear decline: 1.0, 0.9, 0.8, 0.7, 0.6
        for i in 0..5u64 { p.record(i, 1.0 - i as f64 * 0.1, 10); }
        let pred = p.predict(0x0001);
        assert!(pred.trend_slope < -0.09 && pred.trend_slope > -0.11,
            "slope should be ≈ -0.1, got {}", pred.trend_slope);
    }

    #[test]
    fn collapse_eta_computed() {
        let mut p = CollapsePredictor::new(20);
        for i in 0..10u64 { p.record(i, 0.90 - i as f64 * 0.03, 10); }
        let pred = p.predict(0x0001);
        if pred.trend_slope < 0.0 {
            assert!(pred.epochs_to_collapse.is_some());
        }
    }
}
