#![forbid(unsafe_code)]
//! Nash Equilibrium state for pattern populations.
//!
//! The Nash state tracks whether the collective behaviour of particles
//! following a pattern is stable (Nash-optimal) or breaking (anarchy).

/// Nash Equilibrium state for one pattern.
#[derive(Clone, Debug)]
pub struct NashState {
    /// Overall equilibrium score: 0.0 = fully stable, 1.0 = fully breaking.
    pub equilibrium_score: f64,
    /// Price of anarchy — ratio of system cost under anarchy to Nash-optimal cost.
    pub anarchy_score: f64,
    /// Number of constituent particles currently deviating from Nash-optimal paths.
    pub deviation_count: u32,
    /// BASE orbital when this Nash state was last computed.
    pub computed_at_orbital: u64,
}

impl NashState {
    pub fn stable(orbital: u64) -> Self {
        Self { equilibrium_score: 0.0, anarchy_score: 0.0, deviation_count: 0, computed_at_orbital: orbital }
    }

    /// True if equilibrium is breaking (score > 0.85 triggers SA-001 anomaly).
    pub fn is_breaking(&self) -> bool { self.equilibrium_score > 0.85 }

    /// True if the pattern is fully stable (score < 0.05).
    pub fn is_stable(&self) -> bool { self.equilibrium_score < 0.05 }

    /// TiamatLevel as u8: 0=GREEN, 1=DILBAT, 2=KAKKAB, 3=NERGAL, 4=MAROON.
    pub fn tiamat_level(&self) -> u8 {
        match self.equilibrium_score {
            s if s < 0.20 => 0, // GREEN
            s if s < 0.40 => 1, // DILBAT
            s if s < 0.60 => 2, // KAKKAB
            s if s < 0.85 => 3, // NERGAL
            _             => 4, // MAROON
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_state() {
        let s = NashState::stable(100);
        assert!(s.is_stable());
        assert!(!s.is_breaking());
        assert_eq!(s.tiamat_level(), 0);
    }

    #[test]
    fn breaking_state() {
        let s = NashState {
            equilibrium_score: 0.90,
            anarchy_score: 1.4,
            deviation_count: 30,
            computed_at_orbital: 200,
        };
        assert!(s.is_breaking());
        assert_eq!(s.tiamat_level(), 4);
    }
}
