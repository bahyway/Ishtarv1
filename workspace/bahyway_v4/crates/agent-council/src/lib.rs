#![forbid(unsafe_code)]
//! # AICouncil — Sovereign Pattern Governance Council
//!
//! Three AI agents vote on pattern canonicalization in a 2-phase protocol:
//!
//! ## Phase 1 — Evaluate
//! Each agent independently evaluates the pattern-kaki and emits a preliminary
//! score in range \[0.0, 1.0\].  No inter-agent communication in Phase 1.
//!
//! ## Phase 2 — Deliberate
//! Agents see each other's Phase 1 scores and issue a final APPROVE/REJECT vote.
//! Consensus threshold: 75% → at least 3/3 votes (unanimous) because ⌈3 × 0.75⌉ = 3.
//! With 3 agents, 75% means all three must approve.
//!
//! ## Agents
//! | Agent   | Role                                       |
//! |---------|--------------------------------------------|
//! | TamuzAI | Pattern quality — confidence, density, size |
//! | NINSUN  | Cross-domain consistency with existing patterns |
//! | PAZUZU  | Anomaly detection — adversarial / synthetic flood guard |

pub mod agent;
pub mod council;
pub mod vote;

pub use agent::{AgentId, AgentEvaluation};
pub use council::{AiCouncil, CouncilDecision};
pub use vote::{Vote, VoteRecord, ConsensusResult};
