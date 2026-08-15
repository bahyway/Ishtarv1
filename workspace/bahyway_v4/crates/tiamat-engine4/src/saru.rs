//! SARU accumulator — transparency deficit accounting.
//! 1 SARU = 1 deficit-day (tau_full below threshold for one full day).
//! SAHU = bilateral negotiated uncertainty modifier from the SLA KAKI.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};

use crate::tau::eav_state_from_tau;

/// SARU threshold: tau_full below this accumulates deficit.
pub const SARU_THRESHOLD: f64 = 0.5;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaruAccumulator {
    pub tribe_id: String,
    pub accumulated_saru: f64,
    /// SAHU: uncertainty modifier [0.0, 1.0]; 0 = no uncertainty
    pub sahu: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaruReport {
    pub tribe_id: String,
    pub tau_full: f64,
    pub eav_state: String,
    pub saru_accumulated: f64,
    pub sahu: f64,
    /// Effective deficit = accumulated_saru x (1 + sahu)
    pub effective_deficit: f64,
}

impl SaruAccumulator {
    pub fn new(tribe_id: &str, sahu: f64) -> Self {
        Self {
            tribe_id: tribe_id.to_string(),
            accumulated_saru: 0.0,
            sahu,
        }
    }

    /// Call once per day-tick. If tau_full < threshold, accumulate 1 SARU.
    pub fn tick_day(&mut self, tau_full: f64) {
        if tau_full < SARU_THRESHOLD {
            self.accumulated_saru += 1.0;
        }
    }

    pub fn report(&self, tau_full: f64) -> SaruReport {
        SaruReport {
            tribe_id: self.tribe_id.clone(),
            tau_full,
            eav_state: eav_state_from_tau(tau_full).to_string(),
            saru_accumulated: self.accumulated_saru,
            sahu: self.sahu,
            effective_deficit: self.accumulated_saru * (1.0 + self.sahu),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_deficit_above_threshold() {
        let mut acc = SaruAccumulator::new("tribe-a", 0.0);
        acc.tick_day(0.9);
        assert_eq!(acc.accumulated_saru, 0.0);
    }

    #[test]
    fn deficit_accumulates_below_threshold() {
        let mut acc = SaruAccumulator::new("tribe-b", 0.0);
        acc.tick_day(0.3);
        acc.tick_day(0.2);
        assert_eq!(acc.accumulated_saru, 2.0);
    }

    #[test]
    fn sahu_scales_effective_deficit() {
        let mut acc = SaruAccumulator::new("tribe-c", 0.5);
        acc.tick_day(0.1);
        let report = acc.report(0.1);
        assert_eq!(report.saru_accumulated, 1.0);
        assert!((report.effective_deficit - 1.5).abs() < 1e-9);
        assert_eq!(report.eav_state, "DEAD");
    }
}
