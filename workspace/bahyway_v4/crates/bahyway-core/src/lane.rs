//! Lane — evidence classification for BahyWay particles (Ṭupšarrūtu §1, A3).
//!
//! Lane is the **trust axis** — entirely separate from `ParticleState` (lifecycle).
//! A Golden particle can sit in any lane.  A Dead particle can also sit in any lane.
//! The invariant A3 enforces: Lane::Gray requires a prior Evidence event.

/// Evidence classification — the three lanes of the Ṭupšarrūtu algebra (sort 𝓛).
///
/// A3 invariant: `lane(p) = Gray  ⟹  ∃ e ∈ events(p). kind(e) = Evidence`
/// This is enforced at runtime by the `mint` and `journal` paths; this type
/// represents the value, not the enforcement.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Lane {
    /// Cleared, evidence-free — the default trust level for a new particle.
    White = 0x01,
    /// Blocked, revoked, or permanently denied.
    Black = 0x02,
    /// Evidence-required — particle has an attached Evidence event (A3).
    Gray  = 0x03,
}

impl Lane {
    /// Returns `true` for `White` — no evidence required, default entry lane.
    #[inline]
    pub fn is_white(self) -> bool { self == Lane::White }

    /// Returns `true` for `Black` — access blocked regardless of evidence.
    #[inline]
    pub fn is_black(self) -> bool { self == Lane::Black }

    /// Returns `true` for `Gray` — evidence obligation active (A3).
    #[inline]
    pub fn is_gray(self) -> bool { self == Lane::Gray }

    /// Returns `true` if a transition *into* this lane would require
    /// an Evidence event to accompany it (i.e., this is Gray).
    #[inline]
    pub fn requires_evidence(self) -> bool { self == Lane::Gray }

    /// Display name used in HeptaScript LANE clauses and STIR output.
    pub fn as_str(self) -> &'static str {
        match self {
            Lane::White => "WHITE",
            Lane::Black => "BLACK",
            Lane::Gray  => "GRAY",
        }
    }
}

impl Default for Lane {
    /// New particles enter the White lane by default (no obligations).
    fn default() -> Self { Lane::White }
}

impl core::fmt::Display for Lane {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Compute the effective lane when two lanes must be composed (e.g., CrossTribe merge).
///
/// Composition table:
/// - Any Black → Black  (revocation is absolute)
/// - Any Gray  → Gray   (evidence obligation propagates)
/// - Both White → White
pub fn compose_lanes(a: Lane, b: Lane) -> Lane {
    use Lane::*;
    match (a, b) {
        (Black, _) | (_, Black) => Black,
        (Gray, _)  | (_, Gray)  => Gray,
        _                       => White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_white() {
        assert_eq!(Lane::default(), Lane::White);
    }

    #[test]
    fn predicates() {
        assert!(Lane::White.is_white());
        assert!(Lane::Black.is_black());
        assert!(Lane::Gray.is_gray());
        assert!(!Lane::White.is_black());
        assert!(!Lane::Black.is_gray());
    }

    #[test]
    fn gray_requires_evidence() {
        assert!(Lane::Gray.requires_evidence());
        assert!(!Lane::White.requires_evidence());
        assert!(!Lane::Black.requires_evidence());
    }

    #[test]
    fn display() {
        assert_eq!(Lane::White.to_string(), "WHITE");
        assert_eq!(Lane::Black.to_string(), "BLACK");
        assert_eq!(Lane::Gray.to_string(),  "GRAY");
    }

    #[test]
    fn compose_black_dominates() {
        assert_eq!(compose_lanes(Lane::Black, Lane::White), Lane::Black);
        assert_eq!(compose_lanes(Lane::White, Lane::Black), Lane::Black);
        assert_eq!(compose_lanes(Lane::Black, Lane::Gray),  Lane::Black);
        assert_eq!(compose_lanes(Lane::Gray,  Lane::Black), Lane::Black);
        assert_eq!(compose_lanes(Lane::Black, Lane::Black), Lane::Black);
    }

    #[test]
    fn compose_gray_propagates() {
        assert_eq!(compose_lanes(Lane::Gray,  Lane::White), Lane::Gray);
        assert_eq!(compose_lanes(Lane::White, Lane::Gray),  Lane::Gray);
        assert_eq!(compose_lanes(Lane::Gray,  Lane::Gray),  Lane::Gray);
    }

    #[test]
    fn compose_white_identity() {
        assert_eq!(compose_lanes(Lane::White, Lane::White), Lane::White);
    }
}
