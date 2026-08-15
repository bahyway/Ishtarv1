//! High-Sovereignty Failure Model — Quantum Freeze detection (doc: Failure_Modeling).
//!
//! Implements the OOO Quantum-Resilient Observation Manifold:
//! When a node's D7 Integrity drops below `QUANTUM_FREEZE_D7_THRESHOLD`, the
//! Shedu runtime transitions from live streaming to deterministic simulation mode,
//! preserving the last sovereign snapshot for offline replay.
//!
//! # Cluster Topology
//! 1 VDI Node + 2 Stream Nodes.  When the last stream node approaches failure,
//! the system emits a `QuantumFreeze` signal so the stakeholder can record the
//! final visualization state as a `.akk-sim` sovereign scroll.
//!
//! # Thermal States
//! `Healthy` → `Warning` → `QuantumFreeze`
//! Transitions are monotone (once frozen, only a steward reincarnation restores health).

#![forbid(unsafe_code)]

/// D7 Integrity threshold below which a Warning is raised.
///
/// D7 carries the 8-bit checksum of a particle's structural self-consistency.
/// Values below this threshold indicate hardware pressure or replication lag.
pub const QUANTUM_FREEZE_WARNING_THRESHOLD: u8 = 100;

/// D7 Integrity threshold below which a full Quantum Freeze is triggered.
///
/// The stakeholder receives an "imminent collapse" notification and should
/// immediately trigger `RECORD SIMULATION` in the DubSar IDE.
pub const QUANTUM_FREEZE_CRITICAL_THRESHOLD: u8 = 40;

/// A node's health classification as observed by the Shedu monitoring engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverState {
    /// D7 Integrity is nominal — live streaming continues uninterrupted.
    Healthy,

    /// D7 Integrity has dropped into the warning band — replication lag detected.
    /// The node is still operational but degrading.  Stakeholder is notified.
    Warning {
        /// Current D7 integrity byte (100 > d7 ≥ 40).
        d7: u8,
    },

    /// D7 Integrity is critically low — imminent node collapse.
    /// The Shedu engine converts the live stream to a Deterministic Simulation.
    /// StoryWay records this event as: "Real-time Stream Unstable.
    /// Materializing Final Sovereign Snapshot."
    QuantumFreeze,
}

impl FailoverState {
    /// Returns `true` if the node can still serve live data.
    pub fn is_live(&self) -> bool {
        matches!(self, FailoverState::Healthy | FailoverState::Warning { .. })
    }

    /// Returns `true` if an immediate action (record simulation) is required.
    pub fn requires_freeze_action(&self) -> bool {
        matches!(self, FailoverState::QuantumFreeze)
    }

    /// Human-readable status message for the DubSar IDE notification bar.
    pub fn status_message(&self) -> &'static str {
        match self {
            FailoverState::Healthy         => "Stream Nominal — All Nodes Operational",
            FailoverState::Warning { .. }  => "Stream Degraded — Replication Lag Detected",
            FailoverState::QuantumFreeze   => "Real-time Stream Unstable — Materializing Final Sovereign Snapshot",
        }
    }
}

/// Classify a node's current D7 Integrity byte into a `FailoverState`.
///
/// Called by the Shedu monitoring engine on every heartbeat cycle.
///
/// # Arguments
/// * `d7_integrity` — current D7 Integrity byte from the node's health metric
///
/// # Returns
/// `FailoverState::Healthy`       if d7_integrity ≥ `QUANTUM_FREEZE_WARNING_THRESHOLD`
/// `FailoverState::Warning`       if d7_integrity in `[CRITICAL, WARNING)`
/// `FailoverState::QuantumFreeze` if d7_integrity < `QUANTUM_FREEZE_CRITICAL_THRESHOLD`
pub fn check_failover(d7_integrity: u8) -> FailoverState {
    if d7_integrity >= QUANTUM_FREEZE_WARNING_THRESHOLD {
        FailoverState::Healthy
    } else if d7_integrity >= QUANTUM_FREEZE_CRITICAL_THRESHOLD {
        FailoverState::Warning { d7: d7_integrity }
    } else {
        FailoverState::QuantumFreeze
    }
}

/// Aggregate health over a cluster of nodes.
///
/// Returns the *worst* state observed across all nodes — a single failing node
/// can drive the cluster to QuantumFreeze even if others are healthy.
pub fn cluster_failover_state(d7_values: &[u8]) -> FailoverState {
    let mut worst = FailoverState::Healthy;
    for &d7 in d7_values {
        let state = check_failover(d7);
        // ordering: Healthy < Warning < QuantumFreeze
        match (&state, &worst) {
            (FailoverState::QuantumFreeze, _) => { worst = state; break; }
            (FailoverState::Warning { .. }, FailoverState::Healthy) => { worst = state; }
            _ => {}
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_above_warning_threshold() {
        let state = check_failover(QUANTUM_FREEZE_WARNING_THRESHOLD);
        assert_eq!(state, FailoverState::Healthy);
    }

    #[test]
    fn test_healthy_max_d7() {
        assert_eq!(check_failover(255), FailoverState::Healthy);
        assert!(check_failover(255).is_live());
    }

    #[test]
    fn test_warning_in_band() {
        let state = check_failover(70);
        assert_eq!(state, FailoverState::Warning { d7: 70 });
        assert!(state.is_live());
        assert!(!state.requires_freeze_action());
    }

    #[test]
    fn test_warning_at_lower_bound() {
        let state = check_failover(QUANTUM_FREEZE_CRITICAL_THRESHOLD);
        assert_eq!(state, FailoverState::Warning { d7: QUANTUM_FREEZE_CRITICAL_THRESHOLD });
    }

    #[test]
    fn test_quantum_freeze_below_critical() {
        let state = check_failover(QUANTUM_FREEZE_CRITICAL_THRESHOLD - 1);
        assert_eq!(state, FailoverState::QuantumFreeze);
        assert!(!state.is_live());
        assert!(state.requires_freeze_action());
    }

    #[test]
    fn test_quantum_freeze_at_zero() {
        assert_eq!(check_failover(0), FailoverState::QuantumFreeze);
    }

    #[test]
    fn test_status_messages_non_empty() {
        assert!(!FailoverState::Healthy.status_message().is_empty());
        assert!(!FailoverState::Warning { d7: 60 }.status_message().is_empty());
        assert!(!FailoverState::QuantumFreeze.status_message().is_empty());
    }

    #[test]
    fn test_cluster_healthy_when_all_nominal() {
        let state = cluster_failover_state(&[255, 200, 150, 100]);
        assert_eq!(state, FailoverState::Healthy);
    }

    #[test]
    fn test_cluster_warning_when_one_degraded() {
        let state = cluster_failover_state(&[255, 70, 200]);
        assert!(matches!(state, FailoverState::Warning { .. }));
    }

    #[test]
    fn test_cluster_freeze_when_one_critical() {
        let state = cluster_failover_state(&[255, 200, 10]);
        assert_eq!(state, FailoverState::QuantumFreeze);
    }

    #[test]
    fn test_cluster_empty_is_healthy() {
        let state = cluster_failover_state(&[]);
        assert_eq!(state, FailoverState::Healthy);
    }
}
