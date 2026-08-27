#![forbid(unsafe_code)]
//! Vote types and consensus calculation for the AICouncil.

use crate::agent::{AgentEvaluation, AgentId};

/// A binary approve/reject vote from one agent in Phase 2.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Vote {
    Approve,
    Reject,
}

impl Vote {
    /// An agent approves if its Phase 1 score is ≥ 0.75 after seeing peer scores.
    /// The deliberation boost allows a score of 0.70–0.74 to flip to Approve if
    /// both peers scored ≥ 0.85 (social force alignment).
    pub fn deliberate(own_eval: &AgentEvaluation, peer_scores: [f64; 2]) -> Self {
        let base = own_eval.score;
        let peers_strong = peer_scores.iter().all(|&p| p >= 0.85);
        let effective = if peers_strong && base >= 0.70 {
            base + 0.05
        } else {
            base
        };
        if effective >= 0.75 {
            Vote::Approve
        } else {
            Vote::Reject
        }
    }
}

/// One agent's complete vote record for the audit trail.
#[derive(Clone, Debug)]
pub struct VoteRecord {
    pub agent: AgentId,
    pub phase1_score: f64,
    pub phase2_vote: Vote,
    pub reason: &'static str,
}

/// Result of the full 2-phase council vote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsensusResult {
    /// All 3 agents approved (≥ 75% of 3 = 3 required).
    Approved,
    /// Less than 3 approvals — with the approve_count indicating how many did approve.
    Rejected { approve_count: u8 },
    /// Phase 1 evaluation set was incomplete (< 3 agents submitted evaluations).
    Incomplete,
}

impl ConsensusResult {
    pub fn from_votes(votes: &[VoteRecord]) -> Self {
        if votes.len() < 3 {
            return ConsensusResult::Incomplete;
        }
        let approvals = votes
            .iter()
            .filter(|v| v.phase2_vote == Vote::Approve)
            .count() as u8;
        // 3 agents × 75% threshold → ceiling(3 × 0.75) = 3 required
        if approvals >= 3 {
            ConsensusResult::Approved
        } else {
            ConsensusResult::Rejected {
                approve_count: approvals,
            }
        }
    }

    pub fn is_approved(&self) -> bool {
        *self == ConsensusResult::Approved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{derive_pattern_kaki, FixedCoord7D, PatternType};

    fn dummy_eval(agent: AgentId, score: f64) -> AgentEvaluation {
        let k = derive_pattern_kaki(
            PatternType::CrowdFlow,
            FixedCoord7D::zero(),
            [0u8; 32],
            8_000,
            1,
        );
        AgentEvaluation::new(agent, k, score, "test", 1)
    }

    fn vote_record(agent: AgentId, score: f64, peers: [f64; 2]) -> VoteRecord {
        let ev = dummy_eval(agent, score);
        VoteRecord {
            agent,
            phase1_score: score,
            phase2_vote: Vote::deliberate(&ev, peers),
            reason: "test",
        }
    }

    #[test]
    fn unanimous_approve() {
        let records = vec![
            vote_record(AgentId::TamuzAI, 0.90, [0.88, 0.92]),
            vote_record(AgentId::Ninsun, 0.88, [0.90, 0.92]),
            vote_record(AgentId::Pazuzu, 0.92, [0.90, 0.88]),
        ];
        assert_eq!(
            ConsensusResult::from_votes(&records),
            ConsensusResult::Approved
        );
    }

    #[test]
    fn one_reject_fails_consensus() {
        let records = vec![
            vote_record(AgentId::TamuzAI, 0.90, [0.88, 0.50]),
            vote_record(AgentId::Ninsun, 0.88, [0.90, 0.50]),
            vote_record(AgentId::Pazuzu, 0.50, [0.90, 0.88]),
        ];
        let result = ConsensusResult::from_votes(&records);
        assert!(!result.is_approved());
        assert!(matches!(
            result,
            ConsensusResult::Rejected { approve_count: 2 }
        ));
    }

    #[test]
    fn incomplete_on_two_votes() {
        let records = vec![
            vote_record(AgentId::TamuzAI, 0.90, [0.88, 0.92]),
            vote_record(AgentId::Ninsun, 0.88, [0.90, 0.92]),
        ];
        assert_eq!(
            ConsensusResult::from_votes(&records),
            ConsensusResult::Incomplete
        );
    }

    #[test]
    fn deliberation_boost_applies() {
        // score 0.71 alone would reject, but peers at 0.90/0.88 boost to 0.76 → approve
        let ev = dummy_eval(AgentId::Pazuzu, 0.71);
        assert_eq!(Vote::deliberate(&ev, [0.90, 0.88]), Vote::Approve);
    }
}
