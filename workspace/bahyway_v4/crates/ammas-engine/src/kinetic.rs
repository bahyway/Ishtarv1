//! AMMAS master kinetic equation: ∂f/∂t = J_phys[f] + J_mem[f] + J_learn[f]
//!
//! Three kinetic terms:
//!   J_phys  → orbital motion + Rule 7 density sink       (tribe-orbit-engine)
//!   J_mem   → BehaviorPolicy tribal law transitions       (behavior.rs)
//!   J_learn → quality drift from H(P) scoring updates    (score-engine, stub here)
//!
//! The `KineticState` holds all per-particle kinetic quantities for one timestep.
//! `kinetic_step()` advances the system by one discrete step.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::agent::{AgentSnapshot, AgentState};
use crate::behavior::{apply_transition, BehaviorPolicy, TransitionOutcome};

/// Kinetic quantities for a single particle at one timestep.
#[derive(Debug, Clone)]
pub struct KineticState {
    /// Original snapshot (immutable this step).
    pub snapshot: AgentSnapshot,
    /// Neighbourhood density count (neighbours within resonance_radius).
    pub density_count: usize,
    /// J_phys contribution: orbital velocity magnitude [m/s analogue].
    pub j_phys: f32,
    /// J_mem contribution: net lane-transition direction (−1, 0, +1).
    pub j_mem: i8,
    /// J_learn contribution: quality drift [−1.0 .. +1.0].
    pub j_learn: f32,
    /// Resulting agent state after this step.
    pub next_state: AgentState,
    /// Behaviour outcome this step.
    pub outcome: TransitionOutcome,
}

/// Compute J_phys for one particle — orbital kinetic energy proxy.
///
/// J_phys = SPH density weight × orbital speed factor.
/// Here we use a simplified closed-form: velocity decays with radius (inner
/// particles orbit faster) and increases with density pressure.
pub fn j_phys(snapshot: &AgentSnapshot, density_count: usize) -> f32 {
    let b11_norm = snapshot.b11 as f32 / 240.0;
    let speed = b11_norm * 2.0; // higher quality → faster inner orbit
    let pressure = density_count as f32 * 0.05; // density pushes particles outward
    (speed + pressure).min(4.0)
}

/// Compute J_mem for one particle — net law-transition direction.
///
/// +1 → net ascend, 0 → stable, −1 → net descend, −99 → Rule 7 sink.
pub fn j_mem(snapshot: &AgentSnapshot, policy: &BehaviorPolicy) -> i8 {
    match policy.resolve(snapshot.state) {
        TransitionOutcome::Ascend => 1,
        TransitionOutcome::Descend => -1,
        TransitionOutcome::Rule7Sink => -99, // sentinel for Rule 7
        TransitionOutcome::Stable => 0,
    }
}

/// Compute J_learn — quality drift from external H(P) scoring.
///
/// Positive drift → quality improving, negative → degrading.
/// In this stub the caller supplies `delta_h` (Δ of normalised H(P)).
/// The kinetic coefficient ζ = 0.1 — slow, stable learning.
pub fn j_learn(delta_h: f32) -> f32 {
    const ZETA: f32 = 0.1;
    (delta_h * ZETA).clamp(-1.0, 1.0)
}

/// Assemble a `BehaviorPolicy` for a particle given density context.
///
/// Rule 7 is highest-impact: if density ≥ 11 and state is Fuzzy/Dead → overflow sink.
/// GEM particles get the sovereign lock policy.
/// Otherwise construct from the agent's current state.
pub fn policy_for(snapshot: &AgentSnapshot, density_count: usize) -> BehaviorPolicy {
    const TAU_R7: usize = 11;

    if density_count >= TAU_R7 && matches!(snapshot.state, AgentState::Fuzzy | AgentState::Dead) {
        return BehaviorPolicy::rule7_overflow();
    }

    match snapshot.state {
        AgentState::Gem => BehaviorPolicy::gem_locked(),
        AgentState::Tribe => BehaviorPolicy {
            l4_ascend: true,
            l2_promote: true,
            ..BehaviorPolicy::null()
        },
        AgentState::Active => BehaviorPolicy {
            l2_promote: true,
            l1_identity: true,
            ..BehaviorPolicy::null()
        },
        AgentState::Fuzzy => BehaviorPolicy {
            l5_stabilise: true,
            l3_degrade: true,
            ..BehaviorPolicy::null()
        },
        AgentState::Dead => BehaviorPolicy {
            l3_degrade: true,
            ..BehaviorPolicy::null()
        },
    }
}

/// Advance one particle by one discrete kinetic timestep.
///
/// `delta_h` — signed change in H(P) score this step (from score-engine).
pub fn kinetic_step(snapshot: AgentSnapshot, density_count: usize, delta_h: f32) -> KineticState {
    let policy = policy_for(&snapshot, density_count);
    let outcome = policy.resolve(snapshot.state);
    let j_m = j_mem(&snapshot, &policy);
    let j_p = j_phys(&snapshot, density_count);
    let j_l = j_learn(delta_h);
    let next = apply_transition(snapshot.state, outcome);

    KineticState {
        j_phys: j_p,
        j_mem: j_m,
        j_learn: j_l,
        next_state: next,
        outcome,
        density_count,
        snapshot,
    }
}

/// Batch kinetic step for all agents in one tribe.
///
/// `density_counts[i]` = neighbourhood count for agent i (from tribe-orbit-engine).
/// `delta_hs[i]`       = H(P) drift for agent i (from score-engine).
pub fn kinetic_step_batch(
    snapshots: &[AgentSnapshot],
    density_counts: &[usize],
    delta_hs: &[f32],
) -> Vec<KineticState> {
    assert_eq!(snapshots.len(), density_counts.len());
    assert_eq!(snapshots.len(), delta_hs.len());

    snapshots
        .iter()
        .enumerate()
        .map(|(i, snap)| kinetic_step(snap.clone(), density_counts[i], delta_hs[i]))
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentState;

    fn make_snap(b11: u8, kaki_type: u8, state_override: Option<AgentState>) -> AgentSnapshot {
        let mut kaki = [0u8; 16];
        kaki[6] = kaki_type;
        let mut snap = AgentSnapshot::new(kaki, b11, [0.0, 0.0, 0.0]);
        if let Some(s) = state_override {
            snap.state = s;
        }
        snap
    }

    #[test]
    fn j_phys_gem_high_velocity() {
        let snap = make_snap(240, 1, None);
        let v = j_phys(&snap, 0);
        assert!(
            v > 1.5,
            "GEM particle should have high orbital velocity, got {v}"
        );
    }

    #[test]
    fn j_phys_density_increases_velocity() {
        let snap = make_snap(100, 2, None);
        let v_sparse = j_phys(&snap, 0);
        let v_dense = j_phys(&snap, 10);
        assert!(v_dense > v_sparse);
    }

    #[test]
    fn j_phys_capped_at_4() {
        let snap = make_snap(240, 1, None);
        assert!(j_phys(&snap, 100) <= 4.0);
    }

    #[test]
    fn j_mem_gem_stable() {
        let snap = make_snap(240, 1, None);
        let policy = policy_for(&snap, 0);
        assert_eq!(j_mem(&snap, &policy), 0);
    }

    #[test]
    fn j_mem_rule7_sentinel() {
        let snap = make_snap(40, 2, Some(AgentState::Dead));
        let policy = policy_for(&snap, 11);
        assert_eq!(j_mem(&snap, &policy), -99);
    }

    #[test]
    fn j_learn_positive_drift() {
        assert!(j_learn(0.5) > 0.0);
    }

    #[test]
    fn j_learn_clamped() {
        assert_eq!(j_learn(100.0), 1.0);
        assert_eq!(j_learn(-100.0), -1.0);
    }

    #[test]
    fn kinetic_step_gem_stays_gem() {
        let snap = make_snap(240, 1, None);
        let ks = kinetic_step(snap, 0, 0.0);
        assert_eq!(ks.next_state, AgentState::Gem);
        assert_eq!(ks.outcome, TransitionOutcome::Stable);
    }

    #[test]
    fn kinetic_step_dead_rule7_sinks() {
        let snap = make_snap(30, 2, Some(AgentState::Dead));
        let ks = kinetic_step(snap, 11, 0.0);
        assert_eq!(ks.outcome, TransitionOutcome::Rule7Sink);
        assert_eq!(ks.next_state, AgentState::Dead);
    }

    #[test]
    fn kinetic_step_batch_same_length() {
        let snaps: Vec<_> = (0..7).map(|_| make_snap(150, 2, None)).collect();
        let counts = vec![0usize; 7];
        let dhs = vec![0.0f32; 7];
        let result = kinetic_step_batch(&snaps, &counts, &dhs);
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn policy_for_fuzzy_overflow() {
        let snap = make_snap(50, 2, Some(AgentState::Fuzzy));
        let p = policy_for(&snap, 11);
        assert!(p.l7_overflow);
    }

    #[test]
    fn policy_for_tribe_has_l4_and_l2() {
        let snap = make_snap(160, 2, Some(AgentState::Tribe));
        let p = policy_for(&snap, 0);
        assert!(p.l4_ascend);
        assert!(p.l2_promote);
    }
}
