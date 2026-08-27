//! GULA — the grammar-minting agent (candidate name, pending CSR-08 seal).
//! Governed by GL-LSN-001 (Lišānu Law) under GL-ALG-002 (Unified Algebra).
//!
//! The arc: WITNESS → JUDGE → MOVE → STAGE (Nimrud discipline, GL-STK-001).
//! The agent heals the corpus from its own flesh: wounds (three-gloss
//! collisions) become grafts (dialect candidates), staged in URUK only.
//! Sealing to KISH is Bahaa's act alone — no code path here writes to kish/.
//!
//! Zero external dependencies. std only. No Z3, no solvers, no ML.

pub mod apsu;
pub mod arc;
pub mod court;
pub mod graft;
pub mod harvest;
pub mod field;
pub mod naru;
pub mod noise;
pub mod residual;
pub mod tissue;
pub mod uruk;
pub mod usurtu;
pub mod wound;
pub mod zaqiqu;

/// The canonical structural signature of a pattern family, decomposed on the
/// three faces of the GL-ALG-002 triality.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Signature {
    pub particle_face: String,
    pub orbit_face: String,
    pub tribe_face: String,
}

impl Signature {
    pub fn new(particle: &str, orbit: &str, tribe: &str) -> Self {
        Self {
            particle_face: normalize(particle),
            orbit_face: normalize(orbit),
            tribe_face: normalize(tribe),
        }
    }

    /// Canonical key: the shape as one string. Two patterns share a key
    /// iff they share the same deep structure on all three faces.
    pub fn key(&self) -> String {
        format!(
            "P[{}]|O[{}]|T[{}]",
            self.particle_face, self.orbit_face, self.tribe_face
        )
    }
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join("-")
}

/// One engine's testimony: "I hold this pattern shape, and I gloss it thus."
#[derive(Debug, Clone)]
pub struct PatternWitness {
    pub engine: String,
    pub signature: Signature,
    pub hubullu_gloss: String,
}

/// Verdict vocabulary of GL-LSN-001 §6.2. SEALED-KISH is intentionally
/// absent: the agent cannot pronounce it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Observed,
    Triggered,
    Minted,
    RejectedForeign(String),
    RejectedTumor(String),
    StagedUruk(String),
    /// A phantom pattern predicted from the lattice's negative space
    /// (zaqiqu). Prune-exempt; a prediction, never a dialect.
    PhantomPredicted(String),
    /// A live testimony matched a phantom's signature — prophecy fulfilled.
    PhantomRecovered(String),
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verdict::Observed => "OBSERVED",
            Verdict::Triggered => "TRIGGERED",
            Verdict::Minted => "MINTED",
            Verdict::RejectedForeign(_) => "REJECTED-FOREIGN",
            Verdict::RejectedTumor(_) => "REJECTED-TUMOR",
            Verdict::StagedUruk(_) => "STAGED-URUK",
            Verdict::PhantomPredicted(_) => "PHANTOM-PREDICTED",
            Verdict::PhantomRecovered(_) => "PHANTOM-RECOVERED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_key_is_canonical() {
        let a = Signature::new("  Leak  Event ", "decay ORBIT", "pipe tribe");
        let b = Signature::new("leak event", "Decay Orbit", "Pipe   Tribe");
        assert_eq!(a.key(), b.key());
    }
}
