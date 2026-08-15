//! tau_full computation — four-dimensional transparency score.
#![forbid(unsafe_code)]

#[derive(Debug, Clone)]
pub struct TauInput {
    /// D1: fraction of expected particles received [0,1]
    pub data_completeness: f64,
    /// D2: distinct sovereign source count, normalised to [0,1]
    pub source_diversity: f64,
    /// D3: time freshness — 1.0 = fresh, decays exponentially with staleness hours
    pub time_freshness: f64,
    /// D4: schema consistency — fraction of particles without NRM_SCHEMA_DRIFT [0,1]
    pub schema_consistency: f64,
}

impl TauInput {
    /// All-GOLDEN baseline (fully transparent, freshly updated)
    pub fn perfect() -> Self {
        Self {
            data_completeness: 1.0,
            source_diversity: 1.0,
            time_freshness: 1.0,
            schema_consistency: 1.0,
        }
    }

    /// Compute time freshness from staleness in hours (exponential decay, lambda=0.1)
    pub fn freshness_from_hours(staleness_hours: f64) -> f64 {
        (-0.1 * staleness_hours).exp()
    }
}

/// tau_full = geometric mean of the four transparency dimensions.
pub fn compute_tau_full(input: &TauInput) -> f64 {
    let product = input.data_completeness
        * input.source_diversity
        * input.time_freshness
        * input.schema_consistency;
    product.max(0.0).powf(0.25)
}

pub fn eav_state_from_tau(tau: f64) -> &'static str {
    if tau >= 0.7 {
        "GOLDEN"
    } else if tau >= 0.4 {
        "FUZZY"
    } else {
        "DEAD"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_tau_is_one() {
        let t = compute_tau_full(&TauInput::perfect());
        assert!((t - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_completeness_kills_tau() {
        let mut i = TauInput::perfect();
        i.data_completeness = 0.0;
        assert_eq!(compute_tau_full(&i), 0.0);
    }

    #[test]
    fn freshness_decays_with_staleness() {
        let fresh = TauInput::freshness_from_hours(0.0);
        let stale = TauInput::freshness_from_hours(24.0);
        assert!((fresh - 1.0).abs() < 1e-9);
        assert!(stale < fresh);
        assert!(stale > 0.0);
    }

    #[test]
    fn eav_state_thresholds() {
        assert_eq!(eav_state_from_tau(1.0), "GOLDEN");
        assert_eq!(eav_state_from_tau(0.7), "GOLDEN");
        assert_eq!(eav_state_from_tau(0.5), "FUZZY");
        assert_eq!(eav_state_from_tau(0.4), "FUZZY");
        assert_eq!(eav_state_from_tau(0.1), "DEAD");
    }
}
