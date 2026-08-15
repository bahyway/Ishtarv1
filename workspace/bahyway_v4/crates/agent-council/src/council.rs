#![forbid(unsafe_code)]
//! AiCouncil — orchestrates the 2-phase evaluation and deliberation protocol.

use enkidb_kaki::Kaki;
use crate::agent::{AgentId, AgentEvaluation};
use crate::vote::{ConsensusResult, Vote, VoteRecord};

/// The outcome of a full council session on one pattern-kaki.
#[derive(Clone, Debug)]
pub struct CouncilDecision {
    /// The pattern-kaki that was evaluated.
    pub pattern_kaki: Kaki,
    /// Phase 1 evaluations from all three agents.
    pub evaluations: [AgentEvaluation; 3],
    /// Phase 2 vote records.
    pub votes: Vec<VoteRecord>,
    /// Final consensus result.
    pub result: ConsensusResult,
    /// BASE orbital counter when the decision was finalized.
    pub decided_at_orbital: u64,
}

impl CouncilDecision {
    pub fn is_approved(&self) -> bool { self.result.is_approved() }
}

/// Orchestrates the 2-phase AICouncil protocol.
///
/// In v4.0 the AI scoring logic is provided by caller-supplied closures (the
/// actual model inference happens in the `enkidullm-*` crates; agent-council
/// defines the governance protocol, not the model).
pub struct AiCouncil;

impl AiCouncil {
    /// Run a full 2-phase session for `pattern_kaki`.
    ///
    /// `evaluator` is called once per agent in Phase 1; it returns a score \[0.0, 1.0\]
    /// and a static reason string.  In production this closure calls the LLM inference
    /// layer; in tests it returns deterministic values.
    pub fn evaluate<F>(
        pattern_kaki: Kaki,
        decided_at_orbital: u64,
        mut evaluator: F,
    ) -> CouncilDecision
    where
        F: FnMut(AgentId, Kaki) -> (f64, &'static str),
    {
        // ── Phase 1: Independent evaluation ─────────────────────────────
        let evals: [AgentEvaluation; 3] = AgentId::all().map(|agent| {
            let (score, reason) = evaluator(agent, pattern_kaki);
            AgentEvaluation::new(agent, pattern_kaki, score, reason, decided_at_orbital)
        });

        // ── Phase 2: Deliberation — each agent sees peer scores ──────────
        let scores: [f64; 3] = evals.each_ref().map(|e| e.score);

        let votes: Vec<VoteRecord> = evals.iter().enumerate().map(|(i, ev)| {
            let peer_scores = [scores[(i + 1) % 3], scores[(i + 2) % 3]];
            VoteRecord {
                agent:        ev.agent,
                phase1_score: ev.score,
                phase2_vote:  Vote::deliberate(ev, peer_scores),
                reason:       ev.reason,
            }
        }).collect();

        let result = ConsensusResult::from_votes(&votes);

        CouncilDecision { pattern_kaki, evaluations: evals, votes, result, decided_at_orbital }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{derive_pattern_kaki, FixedCoord7D, PatternType};

    fn sample_kaki() -> Kaki {
        derive_pattern_kaki(
            PatternType::IndoorHallway,
            FixedCoord7D { d: [500, 500, 100, 1, 2, 3, 4] },
            [0xABu8; 32],
            9_000,
            55,
        )
    }

    #[test]
    fn all_high_scores_approve() {
        let k = sample_kaki();
        let decision = AiCouncil::evaluate(k, 100, |agent, _| match agent {
            AgentId::TamuzAI => (0.92, "high quality"),
            AgentId::Ninsun  => (0.89, "consistent"),
            AgentId::Pazuzu  => (0.91, "no anomaly"),
        });
        assert!(decision.is_approved());
        assert_eq!(decision.votes.len(), 3);
    }

    #[test]
    fn pazuzu_flags_synthetic_flood_rejects() {
        let k = sample_kaki();
        let decision = AiCouncil::evaluate(k, 200, |agent, _| match agent {
            AgentId::TamuzAI => (0.88, "quality ok"),
            AgentId::Ninsun  => (0.85, "consistent"),
            AgentId::Pazuzu  => (0.30, "synthetic flood detected"),
        });
        assert!(!decision.is_approved());
        assert!(matches!(
            decision.result,
            ConsensusResult::Rejected { approve_count: 2 }
        ));
    }

    #[test]
    fn decision_records_correct_kaki() {
        let k = sample_kaki();
        let decision = AiCouncil::evaluate(k, 300, |_, _| (0.80, "ok"));
        assert_eq!(decision.pattern_kaki, k);
        assert_eq!(decision.decided_at_orbital, 300);
    }
}
