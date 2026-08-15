//! decision.rs — KAKI-stamped EaAgent decisions.
#![forbid(unsafe_code)]

use enkidb_kaki::{Kaki, KakiMinter, KakiRole};
use bahyway_core::TribeId;
use crate::agent_kaki::AGENT_TRIBE_ID;

/// Kind of decision EaAgent makes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionKind {
    /// Pauli Exclusion violation detected between two particles.
    PauliViolation,
    /// Jordan Normal Form stability evaluation.
    JordanStability,
    /// Spectral radius collapse warning.
    CollapseWarning,
    /// Particle promoted to GEM lane.
    GemPromotion,
    /// Particle demoted to DEAD state.
    DeathDecree,
    /// Tribe harmony coefficient computed.
    HarmonyReport,
    /// Algebraic equation solved.
    AlgebraSolved,
}

impl DecisionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PauliViolation  => "PAULI_VIOLATION",
            Self::JordanStability => "JORDAN_STABILITY",
            Self::CollapseWarning => "COLLAPSE_WARNING",
            Self::GemPromotion    => "GEM_PROMOTION",
            Self::DeathDecree     => "DEATH_DECREE",
            Self::HarmonyReport   => "HARMONY_REPORT",
            Self::AlgebraSolved   => "ALGEBRA_SOLVED",
        }
    }
    pub fn severity(&self) -> u8 {
        match self {
            Self::PauliViolation  => 9,
            Self::CollapseWarning => 8,
            Self::DeathDecree     => 7,
            Self::JordanStability => 5,
            Self::HarmonyReport   => 3,
            Self::GemPromotion    => 2,
            Self::AlgebraSolved   => 1,
        }
    }
}

/// The outcome of an EaAgent decision.
#[derive(Debug, Clone, PartialEq)]
pub enum DecisionResult {
    /// Decision passed — system is sovereign.
    Sovereign(String),
    /// Decision flagged — requires attention.
    Warning(String),
    /// Decision failed — immediate action needed.
    Critical(String),
    /// Algebraic result with value.
    Value(f64, String),
}

impl DecisionResult {
    pub fn is_critical(&self) -> bool { matches!(self, Self::Critical(_)) }
    pub fn is_sovereign(&self) -> bool { matches!(self, Self::Sovereign(_)) }

    pub fn message(&self) -> &str {
        match self {
            Self::Sovereign(m) | Self::Warning(m) | Self::Critical(m) => m,
            Self::Value(_, m) => m,
        }
    }
}

/// A single KAKI-stamped EaAgent decision — immutable audit record.
#[derive(Debug, Clone)]
pub struct AgentDecision {
    /// Sovereign KAKI identity of this decision.
    pub kaki:    Kaki,
    /// What kind of decision.
    pub kind:    DecisionKind,
    /// The outcome.
    pub result:  DecisionResult,
    /// Epoch when decision was made.
    pub epoch:   u64,
    /// Particles involved (by name).
    pub parties: Vec<String>,
}

impl AgentDecision {
    /// Mint a new KAKI-stamped decision.
    pub fn new(
        kind:    DecisionKind,
        result:  DecisionResult,
        epoch:   u64,
        parties: Vec<String>,
    ) -> Self {
        let minter  = KakiMinter::new(TribeId::from_u16(AGENT_TRIBE_ID));
        let content = format!("{}:{}:{}", kind.as_str(), epoch,
            parties.join(","));
        let hash    = fnv1a_32(content.as_bytes());
        let kaki    = minter.mint_event(hash, KakiRole::Zikru);
        Self { kaki, kind, result, epoch, parties }
    }

    pub fn is_critical(&self) -> bool { self.result.is_critical() }
    pub fn severity(&self) -> u8 { self.kind.severity() }

    pub fn summary(&self) -> String {
        format!("[{}|ep:{}|sev:{}] {} — {}",
            self.kind.as_str(),
            self.epoch,
            self.severity(),
            self.parties.join("+"),
            self.result.message())
    }
}

fn fnv1a_32(data: &[u8]) -> u32 {
    const OFFSET: u32 = 2166136261;
    const PRIME: u32  = 16777619;
    let mut h = OFFSET;
    for &b in data { h ^= b as u32; h = h.wrapping_mul(PRIME); }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_kaki_valid_checksum() {
        let d = AgentDecision::new(
            DecisionKind::PauliViolation,
            DecisionResult::Critical("Collision detected".into()),
            1000,
            vec!["P1".into(), "P2".into()],
        );
        assert!(d.kaki.verify_checksum());
    }

    #[test]
    fn pauli_violation_is_critical() {
        let d = AgentDecision::new(
            DecisionKind::PauliViolation,
            DecisionResult::Critical("test".into()),
            1, vec![],
        );
        assert!(d.is_critical());
        assert_eq!(d.severity(), 9);
    }

    #[test]
    fn gem_promotion_is_not_critical() {
        let d = AgentDecision::new(
            DecisionKind::GemPromotion,
            DecisionResult::Sovereign("Promoted".into()),
            1, vec!["P1".into()],
        );
        assert!(!d.is_critical());
        assert_eq!(d.severity(), 2);
    }

    #[test]
    fn decision_summary_contains_kind() {
        let d = AgentDecision::new(
            DecisionKind::JordanStability,
            DecisionResult::Warning("Marginal stability".into()),
            42, vec!["Tribe-A".into()],
        );
        let s = d.summary();
        assert!(s.contains("JORDAN_STABILITY"));
        assert!(s.contains("Tribe-A"));
    }

    #[test]
    fn two_decisions_have_distinct_kakis() {
        let d1 = AgentDecision::new(DecisionKind::AlgebraSolved,
            DecisionResult::Value(2.0, "x=2".into()), 1, vec![]);
        let d2 = AgentDecision::new(DecisionKind::AlgebraSolved,
            DecisionResult::Value(3.0, "x=3".into()), 2, vec![]);
        assert_ne!(d1.kaki.bytes(), d2.kaki.bytes());
    }
}
