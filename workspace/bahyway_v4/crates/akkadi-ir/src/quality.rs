//! 𒁾 Quality — Lane, Dimension, Threshold Primitives

/// Orbital lane of a particle — derived from B11 byte (ADR-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityLane {
    /// B11 ≥ 200 — Golden Record, inner orbit, gold glow
    Gem,
    /// B11 140–199 — Tribe member, mid orbit
    TribeMember,
    /// B11 100–139 — Active, outer orbit
    Active,
    /// B11 59–99 — Fuzzy, wide unstable orbit, pink (#FF44AA)
    Fuzzy,
    /// B11 < 59 — Dead, exits orbit → Enki dead grid, gray
    Dead,
}

impl QualityLane {
    pub fn from_b11(b11: u8) -> Self {
        match b11 {
            200..=255 => Self::Gem,
            140..=199 => Self::TribeMember,
            100..=139 => Self::Active,
             59..= 99 => Self::Fuzzy,
              _        => Self::Dead,
        }
    }

    pub fn akk_keyword(&self) -> &'static str {
        match self {
            Self::Gem         => "GEM",
            Self::TribeMember => "TRIBE",
            Self::Active      => "ACTIVE",
            Self::Fuzzy       => "FUZZY",
            Self::Dead        => "DEAD",
        }
    }

    pub fn is_alive(&self) -> bool { *self != Self::Dead }

    pub fn orbital_radius_factor(&self) -> f32 {
        match self {
            Self::Gem         => 1.0,
            Self::TribeMember => 1.8,
            Self::Active      => 2.5,
            Self::Fuzzy       => 3.2,
            Self::Dead        => 4.0,
        }
    }
}

impl std::fmt::Display for QualityLane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.akk_keyword())
    }
}

/// One of the 7 Ibn Wahshiyya quality dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeptaDim {
    Accuracy,     // D1 — Arabic name/text correctness
    Completeness, // D2 — Null/missing field rate
    Consistency,  // D3 — Cross-record contradiction
    Validity,     // D4 — Format compliance (NID, phone, coord)
    Uniqueness,   // D5 — Dedup confidence via MinHash/HNSW
    Timeliness,   // D6 — File/record freshness
    Integrity,    // D7 — EAV referential completeness
}

pub const HEPTA_DIMS: [HeptaDim; 7] = [
    HeptaDim::Accuracy, HeptaDim::Completeness, HeptaDim::Consistency,
    HeptaDim::Validity, HeptaDim::Uniqueness, HeptaDim::Timeliness,
    HeptaDim::Integrity,
];

impl HeptaDim {
    pub fn akk_keyword(&self) -> &'static str {
        match self {
            Self::Accuracy     => "accuracy",
            Self::Completeness => "completeness",
            Self::Consistency  => "consistency",
            Self::Validity     => "validity",
            Self::Uniqueness   => "uniqueness",
            Self::Timeliness   => "timeliness",
            Self::Integrity    => "integrity",
        }
    }

    pub fn sovereign_weight(&self) -> f32 {
        match self {
            Self::Accuracy     => 0.30,
            Self::Completeness => 0.20,
            Self::Consistency  => 0.15,
            Self::Validity     => 0.15,
            Self::Uniqueness   => 0.10,
            Self::Timeliness   => 0.05,
            Self::Integrity    => 0.05,
        }
    }

    pub fn planet(&self) -> &'static str {
        match self {
            Self::Accuracy     => "ME-Saturn",
            Self::Completeness => "GU-Jupiter",
            Self::Consistency  => "SAG-Mars",
            Self::Validity     => "IZ-Sun",
            Self::Uniqueness   => "A-Venus",
            Self::Timeliness   => "UD-Mercury",
            Self::Integrity    => "URU-Moon",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Accuracy     => 0,
            Self::Completeness => 1,
            Self::Consistency  => 2,
            Self::Validity     => 3,
            Self::Uniqueness   => 4,
            Self::Timeliness   => 5,
            Self::Integrity    => 6,
        }
    }
}

impl std::fmt::Display for HeptaDim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.akk_keyword())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn gem_from_b11()    { assert_eq!(QualityLane::from_b11(240), QualityLane::Gem); }
    #[test] fn dead_from_b11()   { assert_eq!(QualityLane::from_b11(0),   QualityLane::Dead); }
    #[test] fn fuzzy_from_b11()  { assert_eq!(QualityLane::from_b11(75),  QualityLane::Fuzzy); }
    #[test] fn active_from_b11() { assert_eq!(QualityLane::from_b11(120), QualityLane::Active); }
    #[test] fn gem_is_alive()    { assert!(QualityLane::Gem.is_alive()); }
    #[test] fn dead_not_alive()  { assert!(!QualityLane::Dead.is_alive()); }

    #[test]
    fn sovereign_weights_sum_to_one() {
        let sum: f32 = HEPTA_DIMS.iter().map(|d| d.sovereign_weight()).sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn all_dims_unique_indices() {
        let mut idx: Vec<usize> = HEPTA_DIMS.iter().map(|d| d.index()).collect();
        idx.sort(); idx.dedup();
        assert_eq!(idx.len(), 7);
    }
}
