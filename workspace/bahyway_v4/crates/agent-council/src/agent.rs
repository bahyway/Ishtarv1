#![forbid(unsafe_code)]
//! AI agent identities and per-agent Phase 1 evaluation output.

use enkidb_kaki::Kaki;

/// The three sovereign AI agents on the pattern governance council.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AgentId {
    /// Quality assessor — confidence, density, constituent count, coherence.
    TamuzAI = 0x01,
    /// Cross-domain consistency — checks against existing canonical patterns.
    Ninsun = 0x02,
    /// Anomaly detector — identifies adversarial, synthetic-flood, or deceptive patterns.
    Pazuzu = 0x03,
}

impl AgentId {
    pub fn all() -> [AgentId; 3] {
        [AgentId::TamuzAI, AgentId::Ninsun, AgentId::Pazuzu]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            AgentId::TamuzAI => "TamuzAI",
            AgentId::Ninsun => "NINSUN",
            AgentId::Pazuzu => "PAZUZU",
        }
    }
}

impl core::fmt::Display for AgentId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Phase 1 evaluation output from a single agent.
///
/// Score is in range [0.0, 1.0].  Scores below 0.60 will almost certainly
/// lead the agent to REJECT in Phase 2.
#[derive(Clone, Debug)]
pub struct AgentEvaluation {
    pub agent: AgentId,
    /// Pattern-KAKI under review.
    pub pattern_kaki: Kaki,
    /// Phase 1 quality score [0.0, 1.0].
    pub score: f64,
    /// Reasoning tag (ASCII, ≤ 64 bytes) for audit trail.
    pub reason: &'static str,
    /// BASE orbital counter when this evaluation was produced.
    pub evaluated_at_orbital: u64,
}

impl AgentEvaluation {
    pub fn new(
        agent: AgentId,
        pattern_kaki: Kaki,
        score: f64,
        reason: &'static str,
        evaluated_at_orbital: u64,
    ) -> Self {
        // Score is clamped to [0.0, 1.0] — values outside this range are
        // a caller bug, not a recoverable error.
        let score = score.clamp(0.0, 1.0);
        Self {
            agent,
            pattern_kaki,
            score,
            reason,
            evaluated_at_orbital,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{derive_pattern_kaki, FixedCoord7D, PatternType};

    fn sample_kaki() -> Kaki {
        derive_pattern_kaki(
            PatternType::CrowdFlow,
            FixedCoord7D::zero(),
            [0u8; 32],
            8_500,
            1,
        )
    }

    #[test]
    fn all_returns_three() {
        assert_eq!(AgentId::all().len(), 3);
    }

    #[test]
    fn score_clamped() {
        let k = sample_kaki();
        let ev = AgentEvaluation::new(AgentId::TamuzAI, k, 1.5, "over", 1);
        assert_eq!(ev.score, 1.0);
        let ev2 = AgentEvaluation::new(AgentId::Ninsun, k, -0.3, "under", 1);
        assert_eq!(ev2.score, 0.0);
    }
}
