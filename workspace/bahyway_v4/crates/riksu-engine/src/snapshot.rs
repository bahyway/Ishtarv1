//! FrozenTopology — the crystallized arc state at Quantum Freeze time.
//!
//! A FrozenTopology is the "Simulated Eternity" of the cross-tribe connections.
//! It is sealed by a FreezeResult KAKI and can be serialized to .akk-sim format
//! for offline replay in the Dubsar IDE.

use crate::arc::{RiksuArc, ArcState};
use crate::topology::RiksuTopology;
use shedu_engine::FreezeResult;

/// Crystallized topology snapshot — produced at Quantum Freeze time.
#[derive(Debug, Clone)]
pub struct FrozenTopology {
    /// All arcs at the moment of freeze (including severed ones for history).
    pub arcs: Vec<RiksuArc>,
    /// The freeze event that produced this snapshot.
    pub freeze_result: FreezeResult,
    /// Harmony score at the moment of freeze.
    pub harmony_at_freeze: f32,
    /// Number of active arcs at freeze time.
    pub active_arcs_at_freeze: usize,
}

impl FrozenTopology {
    /// Crystallize a live topology using a freeze result.
    pub fn from_topology(topo: &RiksuTopology, freeze_result: FreezeResult) -> Self {
        let harmony_at_freeze     = topo.harmony_score();
        let active_arcs_at_freeze = topo.active_arc_count();
        Self {
            arcs:                 topo.arcs.clone(),
            freeze_result,
            harmony_at_freeze,
            active_arcs_at_freeze,
        }
    }

    /// Serialize to .akk-sim source format for offline Dubsar replay.
    ///
    /// .akk-sim is a human-readable crystallized simulation file:
    ///   simulation <name> { frozen_at: <ms>  reason: "<reason>"  arc ... }
    pub fn to_akk_sim_source(&self) -> String {
        let mut out = String::with_capacity(512);
        let name = format!("freeze_node{}_{}", self.freeze_result.node_id,
                           self.freeze_result.timestamp_ms);
        out.push_str(&format!("simulation {} {{\n", name));
        out.push_str(&format!("  frozen_at_ms: {}\n", self.freeze_result.timestamp_ms));
        out.push_str(&format!("  reason: \"{}\"\n", self.freeze_result.reason.label()));
        out.push_str(&format!("  node_id: {}\n", self.freeze_result.node_id));
        out.push_str(&format!("  b11_at_freeze: {}\n", self.freeze_result.b11_at_freeze));
        out.push_str(&format!("  harmony_at_freeze: {:.4}\n", self.harmony_at_freeze));
        out.push_str(&format!("  active_arcs: {}\n", self.active_arcs_at_freeze));
        out.push_str(&format!("  total_arcs: {}\n\n", self.arcs.len()));
        for arc in &self.arcs {
            out.push_str(&format!(
                "  arc tribe_{} <-> tribe_{} strength={:.4} state={} opacity={:.4}\n",
                arc.tribe_a, arc.tribe_b, arc.strength, arc.state.label(), arc.opacity()
            ));
        }
        out.push_str("}\n");
        out
    }

    /// Active arcs in the frozen snapshot.
    pub fn visible_arcs(&self) -> impl Iterator<Item = &RiksuArc> {
        self.arcs.iter().filter(|a| a.state != ArcState::Severed)
    }

    /// Message shown to the stakeholder when the freeze fires.
    pub fn stakeholder_message(&self) -> &'static str {
        self.freeze_result.stakeholder_message
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::RiksuTopology;
    use shedu_engine::{HardwareMetrics, NodeHealth, FreezeReason, trigger_quantum_freeze};

    fn make_freeze() -> FreezeResult {
        let h = NodeHealth::from_metrics(0, HardwareMetrics::simulated_critical(5_000));
        trigger_quantum_freeze(&h, FreezeReason::LastNodeFailing, 5_000)
    }

    fn make_kaki(tribe_a: u16, tribe_b: u16) -> [u8; 16] {
        let mut k = [0u8; 16];
        k[4..6].copy_from_slice(&tribe_a.to_le_bytes());
        k[8..10].copy_from_slice(&tribe_b.to_le_bytes());
        k[6] = 0x03;
        k
    }

    #[test]
    fn from_topology_preserves_harmony() {
        let kakis = vec![make_kaki(1, 2), make_kaki(2, 3)];
        let topo  = RiksuTopology::from_cross_tribe_kakis(&kakis);
        let h     = topo.harmony_score();
        let frozen = FrozenTopology::from_topology(&topo, make_freeze());
        assert!((frozen.harmony_at_freeze - h).abs() < 1e-5);
    }

    #[test]
    fn akk_sim_contains_required_fields() {
        let kakis = vec![make_kaki(1, 2)];
        let topo  = RiksuTopology::from_cross_tribe_kakis(&kakis);
        let frozen = FrozenTopology::from_topology(&topo, make_freeze());
        let src   = frozen.to_akk_sim_source();
        assert!(src.contains("simulation freeze_node"));
        assert!(src.contains("frozen_at_ms:"));
        assert!(src.contains("reason:"));
        assert!(src.contains("tribe_1 <-> tribe_2"));
    }

    #[test]
    fn akk_sim_is_valid_text() {
        let kakis = vec![make_kaki(3, 7), make_kaki(1, 5)];
        let topo  = RiksuTopology::from_cross_tribe_kakis(&kakis);
        let frozen = FrozenTopology::from_topology(&topo, make_freeze());
        let src   = frozen.to_akk_sim_source();
        assert!(src.starts_with("simulation "));
        assert!(src.ends_with("}\n"));
    }

    #[test]
    fn visible_arcs_excludes_severed() {
        let kakis = vec![make_kaki(1, 2), make_kaki(2, 3)];
        let mut topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        topo.apply_node_dead(2);
        let frozen = FrozenTopology::from_topology(&topo, make_freeze());
        assert_eq!(frozen.visible_arcs().count(), 0, "all arcs severed after node 2 dies");
    }

    #[test]
    fn stakeholder_message_non_empty() {
        let topo   = RiksuTopology::new();
        let frozen = FrozenTopology::from_topology(&topo, make_freeze());
        assert!(!frozen.stakeholder_message().is_empty());
    }
}
