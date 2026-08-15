//! RiksuTopology — the complete cross-tribe arc graph for a cluster.
//!
//! The topology is built from CrossTribeKaki (Type 3) 16-byte keys.
//! As nodes fail, the topology degrades gracefully: arcs dim, then sever.
//! At Quantum Freeze, the topology crystallizes into a FrozenTopology.

use crate::arc::{RiksuArc, ArcState};

/// The live cross-tribe arc topology for the cluster.
#[derive(Debug, Default)]
pub struct RiksuTopology {
    pub arcs: Vec<RiksuArc>,
}

impl RiksuTopology {
    pub fn new() -> Self { Self::default() }

    /// Build topology from a slice of CrossTribeKaki 16-byte primary keys.
    ///
    /// Bytes [4..6] = tribe_id of the source, [8..10] = tribe_id of destination
    /// (following the KAKI layout: reserved[4] contains cross-tribe metadata).
    /// For simplicity, tribe_a = kaki[4..6] as u16, tribe_b = kaki[8..10] as u16.
    pub fn from_cross_tribe_kakis(kakis: &[[u8; 16]]) -> Self {
        use std::collections::HashMap;
        // Count events per (tribe_a, tribe_b) canonical pair
        let mut counts: HashMap<(u16, u16), u64> = HashMap::new();
        for kaki in kakis {
            let a = u16::from_le_bytes([kaki[4], kaki[5]]);
            let b = u16::from_le_bytes([kaki[8], kaki[9]]);
            if a != b {
                let key = if a <= b { (a, b) } else { (b, a) };
                *counts.entry(key).or_insert(0) += 1;
            }
        }
        let max_events = counts.values().copied().max().unwrap_or(1);
        let arcs = counts.into_iter()
            .map(|((a, b), count)| RiksuArc::new(a, b, count, max_events))
            .collect();
        Self { arcs }
    }

    /// Apply a node failure — dims all arcs connected to `failed_tribe_id`.
    pub fn apply_node_degraded(&mut self, tribe_id: u16) {
        for arc in self.arcs.iter_mut().filter(|a| a.connects(tribe_id)) {
            arc.apply_dim();
        }
    }

    /// Apply a node death — severs all arcs connected to `dead_tribe_id`.
    pub fn apply_node_dead(&mut self, tribe_id: u16) {
        for arc in self.arcs.iter_mut().filter(|a| a.connects(tribe_id)) {
            arc.sever();
        }
    }

    /// Active arc count (not severed).
    pub fn active_arc_count(&self) -> usize {
        self.arcs.iter().filter(|a| a.state.is_visible()).count()
    }

    /// Total arc count.
    pub fn total_arc_count(&self) -> usize { self.arcs.len() }

    /// Topology harmony score — fraction of arcs still active (0..1).
    pub fn harmony_score(&self) -> f32 {
        if self.arcs.is_empty() { return 0.0; }
        self.active_arc_count() as f32 / self.arcs.len() as f32
    }

    /// Average strength of all visible arcs.
    pub fn average_strength(&self) -> f32 {
        let visible: Vec<f32> = self.arcs.iter()
            .filter(|a| a.state.is_visible())
            .map(|a| a.strength)
            .collect();
        if visible.is_empty() { return 0.0; }
        visible.iter().sum::<f32>() / visible.len() as f32
    }

    /// Find the arc connecting two specific tribes (if it exists).
    pub fn find_arc(&self, tribe_a: u16, tribe_b: u16) -> Option<&RiksuArc> {
        self.arcs.iter().find(|a| {
            (a.tribe_a == tribe_a && a.tribe_b == tribe_b) ||
            (a.tribe_a == tribe_b && a.tribe_b == tribe_a)
        })
    }

    /// Crystallize the topology at freeze time.
    pub fn freeze(&mut self) {
        for arc in self.arcs.iter_mut() {
            if arc.state != ArcState::Severed {
                arc.freeze();
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kaki(tribe_a: u16, tribe_b: u16) -> [u8; 16] {
        let mut kaki = [0u8; 16];
        kaki[4..6].copy_from_slice(&tribe_a.to_le_bytes());
        kaki[8..10].copy_from_slice(&tribe_b.to_le_bytes());
        kaki[6] = 0x03; // CrossTribeKaki type
        kaki
    }

    #[test]
    fn build_from_kakis() {
        let kakis = vec![make_kaki(1, 2), make_kaki(1, 2), make_kaki(2, 3)];
        let topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        assert_eq!(topo.total_arc_count(), 2, "two distinct pairs");
    }

    #[test]
    fn self_loop_ignored() {
        let kakis = vec![make_kaki(1, 1)];
        let topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        assert_eq!(topo.total_arc_count(), 0);
    }

    #[test]
    fn harmony_full_when_all_active() {
        let kakis = vec![make_kaki(1, 2), make_kaki(2, 3)];
        let topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        assert_eq!(topo.harmony_score(), 1.0);
    }

    #[test]
    fn apply_dead_severs_connected_arcs() {
        let kakis = vec![make_kaki(1, 2), make_kaki(2, 3), make_kaki(1, 3)];
        let mut topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        topo.apply_node_dead(2);
        // arcs 1-2 and 2-3 should be severed; 1-3 should remain active
        let arc_12 = topo.find_arc(1, 2).unwrap();
        let arc_23 = topo.find_arc(2, 3).unwrap();
        let arc_13 = topo.find_arc(1, 3).unwrap();
        assert_eq!(arc_12.state, ArcState::Severed);
        assert_eq!(arc_23.state, ArcState::Severed);
        assert_eq!(arc_13.state, ArcState::Active);
        assert_eq!(topo.active_arc_count(), 1);
    }

    #[test]
    fn apply_degraded_dims_arcs() {
        let kakis = vec![make_kaki(1, 2)];
        let mut topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        topo.apply_node_degraded(1);
        let arc = topo.find_arc(1, 2).unwrap();
        assert_eq!(arc.state, ArcState::Dimming);
    }

    #[test]
    fn freeze_preserves_visible_arcs() {
        let kakis = vec![make_kaki(1, 2), make_kaki(2, 3)];
        let mut topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        topo.apply_node_dead(2); // severs both arcs
        // add a separate arc that's still active conceptually
        let kakis2 = vec![make_kaki(1, 3)];
        let mut topo2 = RiksuTopology::from_cross_tribe_kakis(&kakis2);
        topo2.freeze();
        assert_eq!(topo2.find_arc(1, 3).unwrap().state, ArcState::Frozen);
    }

    #[test]
    fn average_strength_empty_topology() {
        let topo = RiksuTopology::new();
        assert_eq!(topo.average_strength(), 0.0);
    }

    #[test]
    fn find_arc_symmetric() {
        let kakis = vec![make_kaki(3, 7)];
        let topo = RiksuTopology::from_cross_tribe_kakis(&kakis);
        assert!(topo.find_arc(3, 7).is_some());
        assert!(topo.find_arc(7, 3).is_some());
    }
}
