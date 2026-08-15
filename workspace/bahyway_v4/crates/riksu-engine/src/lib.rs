//! 𒊑𒆪𒍪 riksu-engine — Sovereign Cross-Tribe Arc Topology v4.0.1
//!
//! Riksu (𒊑𒆪𒍪, Akkadian: "bond, knot, treaty") governs the visible arcs
//! connecting tribes in the Big Ring 3D visualization.
//!
//! Architecture:
//!   arc      — RiksuArc: a single cross-tribe bond (strength, state, opacity)
//!   topology — RiksuTopology: the full cluster arc graph
//!   snapshot — FrozenTopology: crystallized at Quantum Freeze time → .akk-sim
//!
//! Depends on shedu-engine for FreezeResult (KAKI-stamped freeze event).
//!
//! "As nodes fail, the Riksu Arcs remain visible as long as the last node
//!  is breathing. At the Quantum Freeze, they crystallize into Simulated Eternity."
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0.1 | Pure Rust

#![forbid(unsafe_code)]

pub mod arc;
pub mod topology;
pub mod snapshot;

pub use arc::{RiksuArc, ArcState, arc_id_from_tribes, ARC_STRENGTH_MIN, ARC_DIM_PER_FAILURE};
pub use topology::RiksuTopology;
pub use snapshot::FrozenTopology;
pub use shedu_engine::{FreezeReason, FreezeResult, trigger_quantum_freeze};

/// Full Quantum Freeze protocol for the Riksu topology.
///
/// 1. Freeze all surviving arcs in place
/// 2. Crystallize into FrozenTopology
/// 3. Serialize to .akk-sim for offline Dubsar replay
///
/// Returns the FrozenTopology and the .akk-sim source string.
pub fn quantum_freeze_topology(
    topo:   &mut RiksuTopology,
    result: FreezeResult,
) -> (FrozenTopology, String) {
    topo.freeze();
    let frozen  = FrozenTopology::from_topology(topo, result);
    let akk_sim = frozen.to_akk_sim_source();
    (frozen, akk_sim)
}

// ── Integration Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use shedu_engine::{HardwareMetrics, NodeHealth};

    fn cross_kaki(a: u16, b: u16) -> [u8; 16] {
        let mut k = [0u8; 16];
        k[4..6].copy_from_slice(&a.to_le_bytes());
        k[8..10].copy_from_slice(&b.to_le_bytes());
        k[6] = 0x03;
        k
    }

    fn make_freeze(node_id: u8) -> FreezeResult {
        let h = NodeHealth::from_metrics(
            node_id,
            HardwareMetrics::simulated_critical(1_000)
        );
        trigger_quantum_freeze(&h, FreezeReason::LastNodeFailing, 1_000)
    }

    #[test]
    fn full_freeze_pipeline() {
        let kakis = vec![cross_kaki(1, 2), cross_kaki(2, 3), cross_kaki(1, 3)];
        let mut topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        assert_eq!(topo.total_arc_count(), 3);

        // Simulate tribe 2 node dying
        topo.apply_node_dead(2);
        assert_eq!(topo.active_arc_count(), 1);  // only 1-3 survives

        // Quantum Freeze
        let (frozen, akk_sim) = quantum_freeze_topology(&mut topo, make_freeze(0));
        assert!(akk_sim.contains("simulation freeze_node0_1000"));
        assert_eq!(frozen.active_arcs_at_freeze, 1);
    }

    #[test]
    fn akk_sim_parseable_as_text() {
        let kakis = vec![cross_kaki(1, 2)];
        let mut topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        let (_, src) = quantum_freeze_topology(&mut topo, make_freeze(1));
        assert!(src.contains("FROZEN") || src.contains("SEVERED") || src.contains("ACTIVE"));
    }

    #[test]
    fn empty_topology_freeze_produces_valid_sim() {
        let mut topo = RiksuTopology::new();
        let (frozen, src) = quantum_freeze_topology(&mut topo, make_freeze(0));
        assert_eq!(frozen.harmony_at_freeze, 0.0);
        assert!(src.starts_with("simulation "));
    }

    #[test]
    fn arc_id_in_frozen_topology_is_cross_tribe_type() {
        let kakis = vec![cross_kaki(4, 5)];
        let topo  = RiksuTopology::from_cross_tribe_kakis(&kakis);
        let arc   = topo.find_arc(4, 5).unwrap();
        assert_eq!(arc.arc_id[6], 0x03);
    }
}
