//! An-Šar Universal Algebra — The "Algebra of Algebras" for UrOS.
//!
//! In Akkadian cosmology, An-Šar means "Whole Heaven" — the totality of all structures.
//! In BahyWay v4.0, the **An-Šar Universal Algebra** defines the **Master Signature Σ_BW**:
//! a set of five fundamental operations that *every* particle in the ecosystem must support.
//!
//! # Σ_BW (BahyWay Universal Signature)
//! ```text
//! Σ_BW = { Birth(η), Pulse(δ), Sift(σ), Wash(ω), Proclaim(π) }
//! ```
//!
//! This follows the framework of Universal Algebra (Birkhoff 1935):
//! - The **Signature Σ** defines which operations exist and their arities.
//! - A **Variety** is a class of Σ-algebras satisfying a fixed set of equations.
//! - **Tribes** are quotient algebras: tribe-membership is a congruence on the particle set.
//! - **CrossTribe-Kakis** are homomorphisms: structure-preserving maps between Tribes.
//! - **Templates** are terms in the free Σ-algebra: the most general structure of that shape.
//!
//! # The Decrees of the Me (Universal Laws)
//! The "Me" (Sumerian divine decrees) are encoded as the five operations of Σ_BW:
//! 1. **Birth (η):** No entity exists without a KAKI Nucleus — identity precedes data.
//! 2. **Pulse (δ):** Every state evolves under the Jordan transformation — there is no frozen truth.
//! 3. **Sift (σ):** Every particle can be filtered by its identity bits — precision is sovereign.
//! 4. **Wash (ω):** Every particle can be cleansed relative to its neighbors — quality is relational.
//! 5. **Proclaim (π):** Every particle can produce a human-readable artifact — silence is not allowed.
//!
//! # Birkhoff's HSP Theorem (Sovereignty Guarantee)
//! Because Tribes are defined equationally over Σ_BW, the class of valid Tribes is closed under:
//! - **H** (Homomorphic images) — CrossTribe-Kaki projections remain valid Tribes.
//! - **S** (Subalgebras) — Sub-tribes inherit all governance laws.
//! - **P** (Products) — Fused Tribes remain valid Tribes.
//!
//! This means a stakeholder's custom Sub-Tribe can never violate the system's fundamental laws.

#![forbid(unsafe_code)]

/// The BahyWay Universal Signature — five operation names and arities.
/// Every Σ_BW-algebra must implement all five operations.
pub const SIGMA_BW: &[(&str, usize)] = &[
    ("Birth", 1),    // η : raw_bytes → KAKI Nucleus
    ("Pulse", 1),    // δ : state → next_state (Jordan evolution)
    ("Sift", 2),     // σ : state × mask → bool (spectral isolation)
    ("Wash", 2),     // ω : state × neighborhood_size → state (VGCA smoothing)
    ("Proclaim", 1), // π : state → text artifact
];

/// AN-ŠAR: The Universal Master Signature for all UrOS particles.
pub const ANSHAR_ALGEBRA: &str =
    "An-Šar Universal Algebra: Σ_BW = {Birth(η,1), Pulse(δ,1), Sift(σ,2), Wash(ω,2), Proclaim(π,1)}";

/// The five operation symbols of Σ_BW.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SigmaOp {
    /// η — assigns a KAKI Nucleus to raw entropy.
    Birth,
    /// δ — applies the Jordan state evolution X(t+1) = J·X(t).
    Pulse,
    /// σ — applies a bit-mask for spectral isolation.
    Sift,
    /// ω — applies VGCA-Delta smoothing relative to neighbors.
    Wash,
    /// π — produces a human-readable artifact (Markdown/Voice).
    Proclaim,
}

impl SigmaOp {
    pub fn arity(self) -> usize {
        match self {
            SigmaOp::Birth => 1,
            SigmaOp::Pulse => 1,
            SigmaOp::Sift => 2,
            SigmaOp::Wash => 2,
            SigmaOp::Proclaim => 1,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            SigmaOp::Birth => "η",
            SigmaOp::Pulse => "δ",
            SigmaOp::Sift => "σ",
            SigmaOp::Wash => "ω",
            SigmaOp::Proclaim => "π",
        }
    }
}

/// A Σ_BW-algebra: any data object that implements all five universal operations.
///
/// Every `AnsharParticle` implementor is a valid member of the UrOS manifold.
/// The MUMMU Brain can govern, cleanse, and visualize *any* implementor without
/// knowing its business domain — only the Signature matters.
pub trait AnsharParticle {
    /// η (Birth) — Return the 16-byte KAKI Nucleus of this particle.
    /// This is the unique invariant identity. Must be deterministic.
    fn birth(&self) -> [u8; 16];

    /// δ (Pulse) — Apply one Jordan evolution step.
    /// `dt` is the discrete time increment (typically 1.0 per event cycle).
    /// Returns the new spectral radius |λ| after the evolution.
    fn pulse(&mut self, dt: f64) -> f64;

    /// σ (Sift) — Test whether this particle passes a spectral bit-mask.
    /// `mask` is applied to B11 health byte; returns `true` if the particle
    /// should remain in the active observation window.
    fn sift(&self, mask: u8) -> bool;

    /// ω (Wash) — Apply VGCA-Delta smoothing.
    /// `neighborhood_size` is the count of neighboring particles used to
    /// compute the Laplacian smoothing term. Returns the new health score.
    fn wash(&mut self, neighborhood_size: usize) -> f64;

    /// π (Proclaim) — Produce a short human-readable status line.
    /// Used by the DubSar IDE and the Story Engine. Max 80 chars.
    fn proclaim(&self) -> &'static str;
}

/// Result of a sub-algebra integrity verification (Birkhoff HSP check).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    /// The algebra satisfies all Σ_BW equations — it is a valid UrOS Tribe.
    Valid,
    /// The algebra violates the named operation's equation.
    Violated {
        operation: SigmaOp,
        reason: &'static str,
    },
}

impl VerificationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, VerificationResult::Valid)
    }
}

/// AN-ŠAR Engine — verifies that a data domain is a valid Σ_BW sub-algebra.
///
/// In practice the full Lean 4 proof obligation runs in MUMMU; this struct holds
/// the runtime evidence collected by the Adad Gate during Tribe registration.
pub struct AnsharEngine {
    /// Operations the subject algebra has declared it implements.
    pub declared_ops: u8, // bitmask over SigmaOp (5 bits)
}

impl AnsharEngine {
    /// Create an engine for a new Tribe registration check.
    pub fn new() -> Self {
        AnsharEngine { declared_ops: 0 }
    }

    /// Mark an operation as declared-implemented by the stakeholder.
    pub fn declare(&mut self, op: SigmaOp) {
        let bit = match op {
            SigmaOp::Birth => 0b00001,
            SigmaOp::Pulse => 0b00010,
            SigmaOp::Sift => 0b00100,
            SigmaOp::Wash => 0b01000,
            SigmaOp::Proclaim => 0b10000,
        };
        self.declared_ops |= bit;
    }

    /// Verify all five Σ_BW operations are declared (completeness check).
    /// A missing operation means the Tribe is not a Σ_BW-algebra and is rejected.
    pub fn verify_completeness(&self) -> VerificationResult {
        const ALL: u8 = 0b11111;
        if self.declared_ops == ALL {
            return VerificationResult::Valid;
        }
        let ops = [
            SigmaOp::Birth,
            SigmaOp::Pulse,
            SigmaOp::Sift,
            SigmaOp::Wash,
            SigmaOp::Proclaim,
        ];
        let bits = [0b00001u8, 0b00010, 0b00100, 0b01000, 0b10000];
        let reasons = [
            "Birth: no KAKI Nucleus assigned",
            "Pulse: no Jordan evolution defined",
            "Sift: no spectral mask filter",
            "Wash: no VGCA smoothing",
            "Proclaim: no artifact output",
        ];
        for ((&op, &bit), &reason) in ops.iter().zip(bits.iter()).zip(reasons.iter()) {
            if self.declared_ops & bit == 0 {
                return VerificationResult::Violated {
                    operation: op,
                    reason,
                };
            }
        }
        VerificationResult::Valid
    }
}

impl Default for AnsharEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_bw_has_five_operations() {
        assert_eq!(SIGMA_BW.len(), 5);
    }

    #[test]
    fn test_sigma_op_arities() {
        assert_eq!(SigmaOp::Birth.arity(), 1);
        assert_eq!(SigmaOp::Pulse.arity(), 1);
        assert_eq!(SigmaOp::Sift.arity(), 2);
        assert_eq!(SigmaOp::Wash.arity(), 2);
        assert_eq!(SigmaOp::Proclaim.arity(), 1);
    }

    #[test]
    fn test_sigma_op_symbols() {
        assert_eq!(SigmaOp::Birth.symbol(), "η");
        assert_eq!(SigmaOp::Pulse.symbol(), "δ");
        assert_eq!(SigmaOp::Sift.symbol(), "σ");
        assert_eq!(SigmaOp::Wash.symbol(), "ω");
        assert_eq!(SigmaOp::Proclaim.symbol(), "π");
    }

    #[test]
    fn test_engine_rejected_when_incomplete() {
        let mut engine = AnsharEngine::new();
        engine.declare(SigmaOp::Birth);
        engine.declare(SigmaOp::Pulse);
        // Missing: Sift, Wash, Proclaim
        let result = engine.verify_completeness();
        assert!(!result.is_valid());
        assert!(matches!(
            result,
            VerificationResult::Violated {
                operation: SigmaOp::Sift,
                ..
            }
        ));
    }

    #[test]
    fn test_engine_valid_when_all_declared() {
        let mut engine = AnsharEngine::new();
        for op in [
            SigmaOp::Birth,
            SigmaOp::Pulse,
            SigmaOp::Sift,
            SigmaOp::Wash,
            SigmaOp::Proclaim,
        ] {
            engine.declare(op);
        }
        assert_eq!(engine.verify_completeness(), VerificationResult::Valid);
    }

    #[test]
    fn test_anshar_algebra_constant_contains_sigma() {
        assert!(ANSHAR_ALGEBRA.contains("Σ_BW"));
        assert!(ANSHAR_ALGEBRA.contains("Birth"));
        assert!(ANSHAR_ALGEBRA.contains("Proclaim"));
    }

    #[test]
    fn test_missing_proclaim_reports_correct_violation() {
        let mut engine = AnsharEngine::new();
        for op in [SigmaOp::Birth, SigmaOp::Pulse, SigmaOp::Sift, SigmaOp::Wash] {
            engine.declare(op);
        }
        let result = engine.verify_completeness();
        assert!(matches!(
            result,
            VerificationResult::Violated {
                operation: SigmaOp::Proclaim,
                ..
            }
        ));
    }
}
