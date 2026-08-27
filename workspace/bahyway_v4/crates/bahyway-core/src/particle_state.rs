//! ParticleState — the three lifecycle phases of any BahyWay particle.
//!
//! Current state is NEVER stored in the KAKI (Structural-Facts-Only Rule §2.4).
//! It lives in the Orbit as a mandatory EAV attribute, projected by the StoryEngine.

/// The three particle lifecycle states.  Decay is never deletion — it is a
/// Phase Transition to the Ground State (DEAD/Gray) per Axiom 8.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ParticleState {
    /// Fully validated, current, healthy.
    Golden = 0x01,
    /// Partially validated, ambiguous, or stale — awaiting steward review.
    Fuzzy = 0x02,
    /// Decayed / end-of-life.  KAKI and full Journal remain forever.
    Dead = 0x03,
}

impl ParticleState {
    /// Returns `true` when the state is active (Golden or Fuzzy).
    #[inline]
    pub fn is_active(self) -> bool {
        matches!(self, ParticleState::Golden | ParticleState::Fuzzy)
    }

    /// Returns `true` for Dead — the ground state (Gray) in ColorID space.
    #[inline]
    pub fn is_dead(self) -> bool {
        self == ParticleState::Dead
    }

    /// Display name used in HeptaScript STATE clauses.
    pub fn as_str(self) -> &'static str {
        match self {
            ParticleState::Golden => "GOLDEN",
            ParticleState::Fuzzy => "FUZZY",
            ParticleState::Dead => "DEAD",
        }
    }
}

impl core::fmt::Display for ParticleState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// IDU Probing Rule composition table (§8.3).
/// Returns the effective link state for a CrossTribe-Kaki based on anchor states.
pub fn compose_link_state(a: ParticleState, b: ParticleState) -> LinkState {
    use ParticleState::*;
    match (a, b) {
        (Dead, _) | (_, Dead) => LinkState::Gray,
        (Golden, Golden) => LinkState::Gold,
        _ => LinkState::Orange,
    }
}

/// Effective state of a CrossTribe-Kaki — computed at query time, never stored.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LinkState {
    Gold,
    Orange,
    Gray,
}

impl core::fmt::Display for LinkState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            LinkState::Gold => "GOLD",
            LinkState::Orange => "ORANGE",
            LinkState::Gray => "GRAY",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_table() {
        use ParticleState::*;
        assert_eq!(compose_link_state(Golden, Golden), LinkState::Gold);
        assert_eq!(compose_link_state(Golden, Fuzzy), LinkState::Orange);
        assert_eq!(compose_link_state(Fuzzy, Fuzzy), LinkState::Orange);
        assert_eq!(compose_link_state(Dead, Golden), LinkState::Gray);
        assert_eq!(compose_link_state(Golden, Dead), LinkState::Gray);
        assert_eq!(compose_link_state(Dead, Dead), LinkState::Gray);
    }

    #[test]
    fn active_vs_dead() {
        assert!(ParticleState::Golden.is_active());
        assert!(ParticleState::Fuzzy.is_active());
        assert!(!ParticleState::Dead.is_active());
        assert!(ParticleState::Dead.is_dead());
    }
}
