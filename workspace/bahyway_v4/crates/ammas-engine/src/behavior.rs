//! J_mem — Memory kinetic term: BehaviorPolicy and tribal law transitions.
//!
//! J_mem[f] drives state transitions under the 7 Tribal Laws:
//!   GEM    → stays unless Rule 7 fires (L7)
//!   TRIBE  → can ascend to GEM (L4: Orbit Law) or descend (L3: Scope)
//!   ACTIVE → promoted to TRIBE via L2 (Quality), degraded by L3
//!   FUZZY  → stabilised by L5 (Transition) or sunk by L7
//!   DEAD   → candidate for removal by L7 (Overflow sink)
//!
//! 7 Tribal Laws priority (ascending):
//!   P(L3) < P(L7) < P(L1) < P(L2) < P(L5) < P(L4) < P(L6)
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

use crate::agent::AgentState;

/// Outcome of a single J_mem evaluation step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionOutcome {
    /// State unchanged — no law fired or laws cancelled out.
    Stable,
    /// Ascend one lane — quality improved.
    Ascend,
    /// Descend one lane — quality degraded.
    Descend,
    /// Rule 7 overflow sink fired — particle exits the tribe.
    Rule7Sink,
}

/// Policy governing which laws can fire given the current agent state.
///
/// Each entry is (law_label, fires) — evaluated in sovereign priority order.
#[derive(Debug, Clone)]
pub struct BehaviorPolicy {
    /// Whether L6 (Sovereignty) is active — vetoes all other law outcomes.
    pub l6_sovereign: bool,
    /// Whether L4 (Orbit Law) ascension is permitted.
    pub l4_ascend: bool,
    /// Whether L5 (Transition) stabilisation is permitted.
    pub l5_stabilise: bool,
    /// Whether L2 (Quality) quality promotion fires.
    pub l2_promote: bool,
    /// Whether L1 (Identity) identity lock is active.
    pub l1_identity: bool,
    /// Whether L7 (Overflow) sink is triggered (density ≥ τ_R7).
    pub l7_overflow: bool,
    /// Whether L3 (Scope) degradation fires.
    pub l3_degrade: bool,
}

impl BehaviorPolicy {
    /// Null policy — nothing fires, particle is stable.
    pub fn null() -> Self {
        Self {
            l6_sovereign: false,
            l4_ascend: false,
            l5_stabilise: false,
            l2_promote: false,
            l1_identity: false,
            l7_overflow: false,
            l3_degrade: false,
        }
    }

    /// Policy for a particle in Rule 7 overflow (density ≥ 11).
    pub fn rule7_overflow() -> Self {
        Self {
            l7_overflow: true,
            ..Self::null()
        }
    }

    /// Policy for a GEM particle — identity locked, orbit law active.
    pub fn gem_locked() -> Self {
        Self {
            l1_identity: true,
            l4_ascend: true,
            l6_sovereign: true,
            ..Self::null()
        }
    }

    /// Resolve the net transition outcome under sovereign law priority.
    ///
    /// Priority order (lowest → highest): L3 < L7 < L1 < L2 < L5 < L4 < L6
    /// The highest-priority firing law determines the outcome.
    pub fn resolve(&self, state: AgentState) -> TransitionOutcome {
        // L6 veto: if sovereign law active, state is locked
        if self.l6_sovereign && state == AgentState::Gem {
            return TransitionOutcome::Stable;
        }

        // L4 (Orbit): highest non-sovereign — ascend if possible
        if self.l4_ascend && can_ascend(state) {
            return TransitionOutcome::Ascend;
        }

        // L5 (Transition): stabilise fuzzy particles
        if self.l5_stabilise && state == AgentState::Fuzzy {
            return TransitionOutcome::Stable;
        }

        // L2 (Quality): promote active/tribe
        if self.l2_promote && matches!(state, AgentState::Active | AgentState::Tribe) {
            return TransitionOutcome::Ascend;
        }

        // L1 (Identity): identity lock — no change
        if self.l1_identity {
            return TransitionOutcome::Stable;
        }

        // L7 (Overflow): rule 7 sink
        if self.l7_overflow {
            return TransitionOutcome::Rule7Sink;
        }

        // L3 (Scope): lowest — degrade
        if self.l3_degrade {
            return TransitionOutcome::Descend;
        }

        TransitionOutcome::Stable
    }
}

/// Whether `state` can ascend to a higher lane.
fn can_ascend(state: AgentState) -> bool {
    !matches!(state, AgentState::Gem)
}

/// Apply a transition outcome to produce the next AgentState.
///
/// Ascend/Descend move exactly one lane; Rule7Sink → Dead.
pub fn apply_transition(state: AgentState, outcome: TransitionOutcome) -> AgentState {
    match outcome {
        TransitionOutcome::Stable => state,
        TransitionOutcome::Rule7Sink => AgentState::Dead,
        TransitionOutcome::Ascend => ascend_one(state),
        TransitionOutcome::Descend => descend_one(state),
    }
}

fn ascend_one(s: AgentState) -> AgentState {
    match s {
        AgentState::Dead => AgentState::Fuzzy,
        AgentState::Fuzzy => AgentState::Active,
        AgentState::Active => AgentState::Tribe,
        AgentState::Tribe => AgentState::Gem,
        AgentState::Gem => AgentState::Gem, // already at top
    }
}

fn descend_one(s: AgentState) -> AgentState {
    match s {
        AgentState::Gem => AgentState::Tribe,
        AgentState::Tribe => AgentState::Active,
        AgentState::Active => AgentState::Fuzzy,
        AgentState::Fuzzy => AgentState::Dead,
        AgentState::Dead => AgentState::Dead, // floor
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_policy_always_stable() {
        let p = BehaviorPolicy::null();
        for &state in &[
            AgentState::Gem,
            AgentState::Tribe,
            AgentState::Active,
            AgentState::Fuzzy,
            AgentState::Dead,
        ] {
            assert_eq!(p.resolve(state), TransitionOutcome::Stable);
        }
    }

    #[test]
    fn rule7_overflow_sinks_to_dead() {
        let p = BehaviorPolicy::rule7_overflow();
        assert_eq!(p.resolve(AgentState::Fuzzy), TransitionOutcome::Rule7Sink);
        assert_eq!(p.resolve(AgentState::Dead), TransitionOutcome::Rule7Sink);
    }

    #[test]
    fn gem_locked_policy_stabilises_gem() {
        let p = BehaviorPolicy::gem_locked();
        assert_eq!(p.resolve(AgentState::Gem), TransitionOutcome::Stable);
    }

    #[test]
    fn l4_ascend_promotes_tribe_to_gem() {
        let p = BehaviorPolicy {
            l4_ascend: true,
            ..BehaviorPolicy::null()
        };
        assert_eq!(p.resolve(AgentState::Tribe), TransitionOutcome::Ascend);
        let next = apply_transition(AgentState::Tribe, TransitionOutcome::Ascend);
        assert_eq!(next, AgentState::Gem);
    }

    #[test]
    fn l3_degrade_demotes_active() {
        let p = BehaviorPolicy {
            l3_degrade: true,
            ..BehaviorPolicy::null()
        };
        assert_eq!(p.resolve(AgentState::Active), TransitionOutcome::Descend);
        let next = apply_transition(AgentState::Active, TransitionOutcome::Descend);
        assert_eq!(next, AgentState::Fuzzy);
    }

    #[test]
    fn l4_overrides_l3() {
        let p = BehaviorPolicy {
            l4_ascend: true,
            l3_degrade: true,
            ..BehaviorPolicy::null()
        };
        // L4 has higher priority than L3, so ascend wins
        assert_eq!(p.resolve(AgentState::Active), TransitionOutcome::Ascend);
    }

    #[test]
    fn apply_transition_rule7_always_dead() {
        for &s in &[AgentState::Gem, AgentState::Tribe, AgentState::Fuzzy] {
            assert_eq!(
                apply_transition(s, TransitionOutcome::Rule7Sink),
                AgentState::Dead
            );
        }
    }

    #[test]
    fn ascend_one_full_chain() {
        assert_eq!(ascend_one(AgentState::Dead), AgentState::Fuzzy);
        assert_eq!(ascend_one(AgentState::Fuzzy), AgentState::Active);
        assert_eq!(ascend_one(AgentState::Active), AgentState::Tribe);
        assert_eq!(ascend_one(AgentState::Tribe), AgentState::Gem);
        assert_eq!(ascend_one(AgentState::Gem), AgentState::Gem);
    }

    #[test]
    fn descend_one_full_chain() {
        assert_eq!(descend_one(AgentState::Gem), AgentState::Tribe);
        assert_eq!(descend_one(AgentState::Tribe), AgentState::Active);
        assert_eq!(descend_one(AgentState::Active), AgentState::Fuzzy);
        assert_eq!(descend_one(AgentState::Fuzzy), AgentState::Dead);
        assert_eq!(descend_one(AgentState::Dead), AgentState::Dead);
    }

    #[test]
    fn l5_stabilises_fuzzy() {
        let p = BehaviorPolicy {
            l5_stabilise: true,
            l3_degrade: true, // L3 also fires but L5 wins
            ..BehaviorPolicy::null()
        };
        assert_eq!(p.resolve(AgentState::Fuzzy), TransitionOutcome::Stable);
    }
}
