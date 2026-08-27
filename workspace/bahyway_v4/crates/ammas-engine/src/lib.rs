//! 𒁾 ammas-engine — Active Matter Multi-Agent Sovereign System v4.0
//!
//! Implements the J_mem term of the AMMAS master kinetic equation:
//!
//!   ∂f/∂t = J_phys[f] + J_mem[f] + J_learn[f]
//!
//! J_mem drives sovereign state transitions under the 7 Tribal Laws.
//! J_phys (orbital SPH) is computed by tribe-orbit-engine.
//! J_learn (quality drift) bridges score-engine H(P) updates.
//!
//! Entry points:
//!   `step_tribe()`  — advance one tribe (≤7 agents) by one timestep
//!   `step_batch()`  — process multiple tribes
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

#![forbid(unsafe_code)]

pub mod agent;
pub mod behavior;
pub mod kinetic;
pub mod markov;

pub use agent::{AgentClass, AgentSnapshot, AgentState};
pub use behavior::{apply_transition, BehaviorPolicy, TransitionOutcome};
pub use kinetic::{
    j_learn, j_mem, j_phys, kinetic_step, kinetic_step_batch, policy_for, KineticState,
};

/// Advance one sovereign tribe (up to 7 agents) by one kinetic timestep.
///
/// `snapshots`      — agents in this tribe this step
/// `density_counts` — neighbourhood counts from tribe-orbit-engine
/// `delta_hs`       — H(P) quality drift per agent from score-engine
///
/// Returns a `Vec<KineticState>` in the same order as `snapshots`.
pub fn step_tribe(
    snapshots: &[AgentSnapshot],
    density_counts: &[usize],
    delta_hs: &[f32],
) -> Vec<KineticState> {
    kinetic_step_batch(snapshots, density_counts, delta_hs)
}

/// Count how many agents ascended, descended, and sank this step.
#[derive(Debug, Default, Clone, Copy)]
pub struct StepStats {
    pub ascended: usize,
    pub descended: usize,
    pub rule7_sink: usize,
    pub stable: usize,
}

impl StepStats {
    pub fn from_states(states: &[KineticState]) -> Self {
        let mut s = Self::default();
        for ks in states {
            match ks.outcome {
                TransitionOutcome::Ascend => s.ascended += 1,
                TransitionOutcome::Descend => s.descended += 1,
                TransitionOutcome::Rule7Sink => s.rule7_sink += 1,
                TransitionOutcome::Stable => s.stable += 1,
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snap(b11: u8) -> AgentSnapshot {
        AgentSnapshot::new([0u8; 16], b11, [0.0, 0.0, 0.0])
    }

    #[test]
    fn step_tribe_returns_same_count() {
        let snaps: Vec<_> = (0..7).map(|i| make_snap(30 + i as u8 * 30)).collect();
        let counts = vec![0usize; 7];
        let dhs = vec![0.0f32; 7];
        let result = step_tribe(&snaps, &counts, &dhs);
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn step_stats_counts_correctly() {
        let snaps = vec![make_snap(240), make_snap(30), make_snap(50)];
        let counts = vec![0usize, 11, 11]; // last two in overflow
        let dhs = vec![0.0f32; 3];
        let states = step_tribe(&snaps, &counts, &dhs);
        let stats = StepStats::from_states(&states);
        assert_eq!(stats.stable, 1); // GEM stays stable
        assert_eq!(stats.rule7_sink, 2); // both dead/fuzzy + density≥11 → sink
    }

    #[test]
    fn step_stats_default_zero() {
        let s = StepStats::default();
        assert_eq!(s.ascended + s.descended + s.rule7_sink + s.stable, 0);
    }
}
