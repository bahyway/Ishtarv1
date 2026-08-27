//! ea-agent-core — Sovereign EaAgent types and KAKI identity.
//!
//! EaAgent (𒂗𒆠) — named after Ea/Enki, the Akkadian God of Wisdom,
//! Mathematics, and Crafts. EaAgent is the mathematical deity of
//! BahyWay.Ecosystem — the algebraic truth engine beneath DubSar IDE.
//!
//! EaAgent's domain:
//!   - Particles Algebra (PA-1 through PA-16)
//!   - Jordan Normal Form (Enlil Algebra)
//!   - Pauli Exclusion Principle (particle identity collision detection)
//!   - Spectral Radius analysis (stability prediction)
//!   - BIGRING TOP: Tribe Algebra + Orbits Calculus + Particles Algebra
//!
//! Tribe ID namespace: 0x1300 (EaAgent particles)
//! All decisions are KAKI-stamped and immutable.
#![forbid(unsafe_code)]

pub mod agent_kaki;
pub mod constants;
pub mod decision;
pub mod particle_state;

pub use agent_kaki::{AgentKaki, AgentTribe, AGENT_TRIBE_ID};
pub use constants::*;
pub use decision::{AgentDecision, DecisionKind, DecisionResult};
pub use particle_state::{HeptaVector, ParticleSnapshot, QuantumState};
