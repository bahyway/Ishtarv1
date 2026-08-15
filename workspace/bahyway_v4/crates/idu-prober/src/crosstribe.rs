//! CrossTribeLinkState — effective state of a CrossTribe-Kaki at probe time (§8.3).
//!
//! This state is COMPUTED at query time, never stored.
//! Storing it would violate forbidden op #8 (§10).

use bahyway_core::{LinkState, ParticleState};

/// The effective state of a CrossTribe-Kaki, computed by IDU probe.
pub type CrossTribeLinkState = LinkState;

/// Compose N anchor states using the meet (weakest wins) rule from §8.3.
/// Dead dominates: any Dead anchor → Gray.
/// Golden+Fuzzy mix → Orange.
/// All Golden → Gold.
pub fn compose_n_anchors(states: &[ParticleState]) -> CrossTribeLinkState {
    if states.is_empty() {
        return CrossTribeLinkState::Gray;
    }
    if states.iter().any(|s| *s == ParticleState::Dead) {
        return CrossTribeLinkState::Gray;
    }
    if states.iter().all(|s| *s == ParticleState::Golden) {
        CrossTribeLinkState::Gold
    } else {
        CrossTribeLinkState::Orange
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_golden_is_gold() {
        let s = compose_n_anchors(&[ParticleState::Golden, ParticleState::Golden]);
        assert_eq!(s, CrossTribeLinkState::Gold);
    }

    #[test]
    fn any_dead_is_gray() {
        let s = compose_n_anchors(&[ParticleState::Golden, ParticleState::Dead]);
        assert_eq!(s, CrossTribeLinkState::Gray);
    }

    #[test]
    fn golden_fuzzy_is_orange() {
        let s = compose_n_anchors(&[ParticleState::Golden, ParticleState::Fuzzy]);
        assert_eq!(s, CrossTribeLinkState::Orange);
    }

    #[test]
    fn empty_anchors_is_gray() {
        let s = compose_n_anchors(&[]);
        assert_eq!(s, CrossTribeLinkState::Gray);
    }

    #[test]
    fn hyperedge_three_anchors() {
        let s = compose_n_anchors(&[
            ParticleState::Golden,
            ParticleState::Golden,
            ParticleState::Fuzzy,
        ]);
        assert_eq!(s, CrossTribeLinkState::Orange);
    }
}
