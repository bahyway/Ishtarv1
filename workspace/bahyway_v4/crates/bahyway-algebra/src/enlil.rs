//! The Enlil Algebra (TOP-JNF) — Tribes, Orbits, Particles in Jordan Normal Form.
//!
//! Renamed from Ĝeš-ḫur / GhishKhur for international pronounceability.
//! Enlil is the "Lord of the Wind" — the supreme governor of the algebraic framework.
//!
//! # Ĝeš-ḫur Blueprint
//! The entire data universe is a Block-Diagonal Matrix. Each Tribe is a Jordan Block;
//! each Jordan Block is mathematically independent. There is zero algebraic cross-talk
//! between Tribes, enabling infinite horizontal scaling.
//!
//! # Pauli Exclusion Principle
//! No two conflicting truths may occupy the same state for a single Identity-KAKI.
//! The four Gates enforce this at every entry point.

#![forbid(unsafe_code)]

/// Formal name of the unified algebra underpinning EnkiDB.
pub const ENLIL_ALGEBRA: &str =
    "Enlil Algebra (TOP-JNF): Tribes, Orbits, Particles in Jordan Normal Form";

/// The four High Council gates that every datum must pass in sequence.
///
/// | Gate    | Governs         | Logic                   | Purpose                                   |
/// |---------|-----------------|-------------------------|-------------------------------------------|
/// | Adad    | Events-KAKI     | Temporal Exclusion      | De-duplicate high-velocity signal bursts  |
/// | Anu     | CrossTribe-KAKI | Authority Exclusion     | Rank sources; resolve cross-domain links  |
/// | Marduk  | Transformation  | Structural Exclusion    | Lock Jordan Blocks during VGCA cleansing  |
/// | Shamash | Identity-KAKI   | State Exclusion         | Judge Active / Stewardship-Gap / Dead     |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnlilGate {
    /// Storm God — Temporal Signal Input.
    /// Catches high-velocity data bursts; prevents collision at entry.
    /// Maps to `Events-KAKI` / orbit-tail appends.
    Adad,

    /// Sky Father — Supreme Source Authority.
    /// Establishes hierarchy of truth; resolves CrossTribe-KAKI tensor links.
    /// Change-of-basis matrix P and P⁻¹ live here.
    Anu,

    /// The Architect — Structural Transformation.
    /// Locks Jordan Blocks during VGCA Delta cleansing; forges meaning from chaos.
    /// No two structural transforms may overlap (Marduk holds the write-lock).
    Marduk,

    /// The Judge — Life/Death State Judgment.
    /// Monitors `|λ|` (Spectral Radius) to classify Active / Stewardship-Gap / Dead.
    /// Maps to `Identity-KAKI` primary index (O(1) Shamash gate).
    Shamash,
}

impl EnlilGate {
    /// Returns the Kaki type this gate primarily governs (as a static label).
    pub fn governs(&self) -> &'static str {
        match self {
            EnlilGate::Adad => "Events-KAKI",
            EnlilGate::Anu => "CrossTribe-KAKI",
            EnlilGate::Marduk => "Transformation",
            EnlilGate::Shamash => "Identity-KAKI",
        }
    }

    /// The exclusion principle this gate enforces.
    pub fn exclusion_kind(&self) -> &'static str {
        match self {
            EnlilGate::Adad => "Temporal Exclusion",
            EnlilGate::Anu => "Authority Exclusion",
            EnlilGate::Marduk => "Structural Exclusion",
            EnlilGate::Shamash => "State Exclusion",
        }
    }
}

/// The three JNF shapes a Particle's trajectory can take (from Ĝeš-ḫur dual-layer analysis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JnfShape {
    /// Nilpotent tail — particle state is destined to vanish (N^k = 0).
    /// Signals temporary/transient data that should expire naturally.
    NilpotentTail,

    /// Convergent spiral — orbit is converging toward the Golden Particle (|λ| < 1).
    /// Healthy mastering trajectory.
    ConvergentSpiral,

    /// Divergent wing — Jordan Block is expanding without bound (|λ| > 1).
    /// Signals data bloat or unresolved conflict.
    DivergentWing,
}

impl JnfShape {
    /// Classify a shape from the spectral radius |λ| and nilpotency flag.
    pub fn classify(spectral_radius: f64, is_nilpotent: bool) -> Self {
        if is_nilpotent {
            return JnfShape::NilpotentTail;
        }
        if spectral_radius < 1.0 {
            JnfShape::ConvergentSpiral
        } else {
            JnfShape::DivergentWing
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pauli Exclusion Principle — Three Energy Shells of the Tribal Orbit
// ─────────────────────────────────────────────────────────────────────────────

/// The quantum-analogue state of a particle in its Tribal orbit.
///
/// Based on the Pauli Exclusion Principle: no two particles can occupy the same
/// (Identity-KAKI, orbit_position, state) triple simultaneously.
///
/// | State       | Physics Analogue   | EnkiDB Meaning                        |
/// |-------------|-------------------|---------------------------------------|
/// | Active      | Conduction Band   | λ ≈ 1.0 — data is live and moving     |
/// | Stewardship | Forbidden Gap     | Data awaiting correction by Steward   |
/// | Dead        | Valence Band      | Archived; cannot re-enter without auth|
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuantumState {
    /// Conduction Band — λ ≈ 1.0. Particle is moving through the Active Orbit.
    /// Only one Active value may occupy the head of the Jordan Chain (n=0).
    Active,

    /// Forbidden Gap — particle is locked by a Data Steward for correction.
    /// "Fermi Pressure" prevents a second particle from entering this state
    /// for the same Identity-KAKI while the first is in correction.
    Stewardship,

    /// Valence Band / Underworld — λ → 0. Archived and frozen.
    /// Acts as a mathematical barrier: no Active signal may overwrite Dead data
    /// without a Steward-issued "Reincarnation Order" (SHAMASH Gate approval).
    Dead,
}

impl QuantumState {
    /// Returns `true` if the state allows orbit motion.
    pub fn is_mobile(self) -> bool {
        matches!(self, QuantumState::Active)
    }

    /// Canonical energy level (higher = more active).
    pub fn energy_level(self) -> i8 {
        match self {
            QuantumState::Active => 1,
            QuantumState::Stewardship => 0,
            QuantumState::Dead => -1,
        }
    }
}

/// The three-dimensional "quantum coordinate" of a particle at a point in time.
/// The Pauli Exclusion Principle forbids two particles from sharing all three.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuantumCoord {
    /// The 16-byte Identity-KAKI bytes 0..8 (first half — uniquely identifies the entity).
    pub kaki_prefix: [u8; 8],
    /// Position in the Jordan Chain (0 = head / Golden Record, higher = older lineage).
    pub orbit_position: u32,
    /// The energy shell.
    pub state: QuantumState,
}

/// Outcome of a Pauli Exclusion check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExclusionResult {
    /// No collision detected — the particle may enter this state.
    Allowed,
    /// The coordinate is already occupied.
    /// The `gate` indicates which Council member detected the violation.
    Excluded {
        gate: EnlilGate,
        reason: &'static str,
    },
}

impl ExclusionResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, ExclusionResult::Allowed)
    }
}

/// Check whether an incoming `QuantumCoord` collides with any occupied coordinate.
///
/// This is the pure logic of the Pauli Exclusion Principle — no I/O needed.
/// The caller supplies the set of currently occupied coordinates.
pub fn pauli_check(incoming: &QuantumCoord, occupied: &[QuantumCoord]) -> ExclusionResult {
    for existing in occupied {
        if existing.kaki_prefix != incoming.kaki_prefix {
            continue;
        }
        if existing.orbit_position != incoming.orbit_position {
            continue;
        }
        if existing.state != incoming.state {
            continue;
        }
        // All three quantum numbers match → Pauli Violation
        let gate = match incoming.state {
            QuantumState::Active => EnlilGate::Shamash, // State/Life judgment
            QuantumState::Stewardship => EnlilGate::Marduk, // Transformation lock
            QuantumState::Dead => EnlilGate::Shamash,   // Underworld guard
        };
        let reason = match incoming.state {
            QuantumState::Active => "Active head already occupied — Golden Record unique",
            QuantumState::Stewardship => "Stewardship locked — Marduk holds the transformation",
            QuantumState::Dead => "Dead particle blocks reincarnation — SHAMASH Gate required",
        };
        return ExclusionResult::Excluded { gate, reason };
    }
    ExclusionResult::Allowed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enlil_algebra_constant_non_empty() {
        assert!(!ENLIL_ALGEBRA.is_empty());
        assert!(ENLIL_ALGEBRA.contains("TOP-JNF"));
    }

    #[test]
    fn test_gate_governs() {
        assert_eq!(EnlilGate::Adad.governs(), "Events-KAKI");
        assert_eq!(EnlilGate::Anu.governs(), "CrossTribe-KAKI");
        assert_eq!(EnlilGate::Marduk.governs(), "Transformation");
        assert_eq!(EnlilGate::Shamash.governs(), "Identity-KAKI");
    }

    #[test]
    fn test_gate_exclusion_kind() {
        assert_eq!(EnlilGate::Adad.exclusion_kind(), "Temporal Exclusion");
        assert_eq!(EnlilGate::Shamash.exclusion_kind(), "State Exclusion");
    }

    #[test]
    fn test_jnf_shape_nilpotent_flag_takes_priority() {
        let shape = JnfShape::classify(2.0, true);
        assert_eq!(shape, JnfShape::NilpotentTail);
    }

    #[test]
    fn test_jnf_shape_convergent() {
        assert_eq!(JnfShape::classify(0.5, false), JnfShape::ConvergentSpiral);
    }

    #[test]
    fn test_jnf_shape_divergent() {
        assert_eq!(JnfShape::classify(1.5, false), JnfShape::DivergentWing);
    }

    #[test]
    fn test_jnf_shape_exactly_one_is_divergent() {
        assert_eq!(JnfShape::classify(1.0, false), JnfShape::DivergentWing);
    }

    // ── Pauli Exclusion tests ─────────────────────────────────────────────────

    fn coord(pos: u32, state: QuantumState) -> QuantumCoord {
        QuantumCoord {
            kaki_prefix: [1, 2, 3, 4, 5, 6, 7, 8],
            orbit_position: pos,
            state,
        }
    }

    #[test]
    fn test_pauli_allows_empty_field() {
        let c = coord(0, QuantumState::Active);
        assert_eq!(pauli_check(&c, &[]), ExclusionResult::Allowed);
    }

    #[test]
    fn test_pauli_excludes_identical_coord() {
        let c = coord(0, QuantumState::Active);
        let occupied = [c.clone()];
        let result = pauli_check(&c, &occupied);
        assert!(!result.is_allowed());
        assert!(matches!(
            result,
            ExclusionResult::Excluded {
                gate: EnlilGate::Shamash,
                ..
            }
        ));
    }

    #[test]
    fn test_pauli_allows_different_orbit_position() {
        let c0 = coord(0, QuantumState::Active);
        let c1 = coord(1, QuantumState::Active);
        assert_eq!(pauli_check(&c1, &[c0]), ExclusionResult::Allowed);
    }

    #[test]
    fn test_pauli_allows_different_state_same_position() {
        let active = coord(0, QuantumState::Active);
        let dead = coord(0, QuantumState::Dead);
        // Active and Dead can coexist at position 0 — different shells
        assert_eq!(pauli_check(&dead, &[active]), ExclusionResult::Allowed);
    }

    #[test]
    fn test_pauli_stewardship_lock_detected_by_marduk() {
        let s = coord(0, QuantumState::Stewardship);
        let result = pauli_check(&s, &[s.clone()]);
        assert!(matches!(
            result,
            ExclusionResult::Excluded {
                gate: EnlilGate::Marduk,
                ..
            }
        ));
    }

    #[test]
    fn test_quantum_state_energy_levels() {
        assert_eq!(QuantumState::Active.energy_level(), 1);
        assert_eq!(QuantumState::Stewardship.energy_level(), 0);
        assert_eq!(QuantumState::Dead.energy_level(), -1);
    }

    #[test]
    fn test_active_is_mobile_others_not() {
        assert!(QuantumState::Active.is_mobile());
        assert!(!QuantumState::Stewardship.is_mobile());
        assert!(!QuantumState::Dead.is_mobile());
    }
}
