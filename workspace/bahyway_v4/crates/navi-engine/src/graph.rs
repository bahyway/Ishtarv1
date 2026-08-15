//! NaviGraph — 7-sector heptagram graph structure.
//!
//! Sector layout (Centre=0, outer clockwise):
//!   0=Centre  1=North  2=NorthEast  3=SouthEast  4=South  5=SouthWest  6=NorthWest
//!
//! Edge chord types: Spoke(0.80×) | Rim(1.00×) | Local(1.10×) | Diagonal(1.40×)

use std::collections::HashMap;
use crate::error::{NaviError, NaviResult};
use crate::navimap::NaviMap;
use crate::particle::{NaviCoord, NaviNodeId, NaviParticle};
use bahyway_core::TribeId;

// ── Heptagram sector ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeptaSector {
    Centre,
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

impl HeptaSector {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => HeptaSector::Centre,
            1 => HeptaSector::North,
            2 => HeptaSector::NorthEast,
            3 => HeptaSector::SouthEast,
            4 => HeptaSector::South,
            5 => HeptaSector::SouthWest,
            6 => HeptaSector::NorthWest,
            _ => HeptaSector::Centre,
        }
    }

    pub fn index(self) -> u8 {
        match self {
            HeptaSector::Centre    => 0,
            HeptaSector::North     => 1,
            HeptaSector::NorthEast => 2,
            HeptaSector::SouthEast => 3,
            HeptaSector::South     => 4,
            HeptaSector::SouthWest => 5,
            HeptaSector::NorthWest => 6,
        }
    }

    /// Adjacency topology: Centre touches all; outer sectors touch their two rim neighbours.
    pub fn is_adjacent(self, other: HeptaSector) -> bool {
        if self == other { return false; }
        if matches!(self, HeptaSector::Centre) || matches!(other, HeptaSector::Centre) {
            return true;
        }
        use HeptaSector::*;
        matches!((self, other),
            (North, NorthEast) | (NorthEast, North)     |
            (NorthEast, SouthEast) | (SouthEast, NorthEast) |
            (SouthEast, South) | (South, SouthEast)     |
            (South, SouthWest) | (SouthWest, South)     |
            (SouthWest, NorthWest) | (NorthWest, SouthWest) |
            (NorthWest, North) | (North, NorthWest)
        )
    }
}

// ── Chord types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeptaChordType {
    /// Centre ↔ outer sector — fast arterial.
    Spoke,
    /// Adjacent outer sectors — neutral ring path.
    Rim,
    /// Two nodes within the same sector — local detail path.
    Local,
    /// Non-adjacent outer sectors — expensive cross-sector jump.
    Diagonal,
}

impl HeptaChordType {
    pub fn base_multiplier(self) -> f32 {
        match self {
            HeptaChordType::Spoke    => 0.80,
            HeptaChordType::Rim      => 1.00,
            HeptaChordType::Local    => 1.10,
            HeptaChordType::Diagonal => 1.40,
        }
    }

    pub fn infer(from: HeptaSector, to: HeptaSector) -> Self {
        if from == to                          { return HeptaChordType::Local; }
        if matches!(from, HeptaSector::Centre) || matches!(to, HeptaSector::Centre) {
            return HeptaChordType::Spoke;
        }
        if from.is_adjacent(to) { HeptaChordType::Rim }
        else                    { HeptaChordType::Diagonal }
    }
}

// ── NaviNode ──────────────────────────────────────────────────────────────────

pub struct NaviNode {
    pub id:       NaviNodeId,
    pub particle: NaviParticle,
    pub sector:   HeptaSector,
}

impl NaviNode {
    pub fn new(id: NaviNodeId, particle: NaviParticle, sector: HeptaSector) -> Self {
        NaviNode { id, particle, sector }
    }

    pub fn is_passable(&self)    -> bool  { self.particle.is_passable() }
    pub fn intrinsic_cost(&self) -> f32   { self.particle.intrinsic_cost() }
}

// ── NaviEdge ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NaviEdge {
    pub from:      NaviNodeId,
    pub to:        NaviNodeId,
    /// Raw geometric distance (metres) — the sovereign ground truth from the map.
    pub base_cost: f32,
    pub chord:     HeptaChordType,
}

impl NaviEdge {
    pub fn new(from: NaviNodeId, to: NaviNodeId, base_cost: f32, chord: HeptaChordType) -> Self {
        NaviEdge { from, to, base_cost, chord }
    }

    /// Effective cost = base_cost × chord multiplier.
    pub fn effective_cost(&self) -> f32 {
        self.base_cost * self.chord.base_multiplier()
    }
}

// ── NaviGraph ─────────────────────────────────────────────────────────────────

pub struct NaviGraph {
    nodes:     HashMap<NaviNodeId, NaviNode>,
    edges:     Vec<NaviEdge>,
    /// Pre-built adjacency index: id → [(neighbour_id, effective_cost)].
    /// Paid once at add_edge() time; O(1) lookup during Dijkstra inner loop.
    adjacency: HashMap<NaviNodeId, Vec<(NaviNodeId, f32)>>,
}

impl NaviGraph {
    pub fn new() -> Self {
        NaviGraph {
            nodes:     HashMap::new(),
            edges:     Vec::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: NaviNode) {
        self.adjacency.entry(node.id).or_default();
        self.nodes.insert(node.id, node);
    }

    pub fn add_edge(&mut self, edge: NaviEdge) {
        let ec = edge.effective_cost();
        self.adjacency.entry(edge.from).or_default().push((edge.to, ec));
        self.edges.push(edge);
    }

    /// Build a NaviGraph from a NaviMap.
    /// Bidirectional BEAMs produce two directed edges.
    pub fn from_navimap(map: &NaviMap) -> NaviResult<Self> {
        if map.node_count() == 0 {
            return Err(NaviError::EmptyGraph);
        }
        let mut g = NaviGraph::new();
        let tribe = TribeId::from_u16(0x0001); // default; overridden per node below

        for mn in &map.nodes {
            let coord   = NaviCoord::new(mn.lat, mn.lon, mn.alt);
            let t       = TribeId::from_u16(mn.tribe);
            let p   = NaviParticle::new(coord, t);
            // Inherit state defaults — all nodes start Active
            let _ = p.state; // already Active from NaviParticle::new
            let sector  = HeptaSector::from_u8(mn.sector);
            g.add_node(NaviNode::new(mn.id, p, sector));
        }
        let _ = tribe; // used only for clarity above

        for mb in &map.beams {
            let from_sector = g.nodes.get(&mb.from)
                .map(|n| n.sector)
                .unwrap_or(HeptaSector::Centre);
            let to_sector   = g.nodes.get(&mb.to)
                .map(|n| n.sector)
                .unwrap_or(HeptaSector::Centre);
            let chord = HeptaChordType::infer(from_sector, to_sector);

            g.add_edge(NaviEdge::new(mb.from, mb.to, mb.distance, chord));
            if !mb.one_way {
                let chord_rev = HeptaChordType::infer(to_sector, from_sector);
                g.add_edge(NaviEdge::new(mb.to, mb.from, mb.distance, chord_rev));
            }
        }
        Ok(g)
    }

    pub fn node(&self, id: NaviNodeId)      -> Option<&NaviNode>   { self.nodes.get(&id) }
    pub fn node_mut(&mut self, id: NaviNodeId) -> Option<&mut NaviNode> { self.nodes.get_mut(&id) }
    pub fn node_count(&self)                -> usize { self.nodes.len() }
    pub fn edge_count(&self)                -> usize { self.edges.len() }

    /// O(1) — pre-built adjacency index.
    pub fn neighbours(&self, id: NaviNodeId) -> &[(NaviNodeId, f32)] {
        self.adjacency.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn has_edge(&self, from: NaviNodeId, to: NaviNodeId) -> bool {
        self.edges.iter().any(|e| e.from == from && e.to == to)
    }

    pub fn nodes_in_sector(&self, sector: HeptaSector) -> Vec<&NaviNode> {
        self.nodes.values().filter(|n| n.sector == sector).collect()
    }

    pub fn all_node_ids(&self) -> Vec<NaviNodeId> {
        self.nodes.keys().copied().collect()
    }

    pub fn edges(&self) -> &[NaviEdge] { &self.edges }
}

impl Default for NaviGraph {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navimap::seven_node_map;

    fn build_seven() -> NaviGraph {
        NaviGraph::from_navimap(&seven_node_map()).expect("seven_node_map build failed")
    }

    #[test]
    fn seven_node_graph_counts() {
        let g = build_seven();
        assert_eq!(g.node_count(), 7);
        // 6 bidirectional spokes → 12 directed; 6 bidirectional rim → 12 directed
        assert_eq!(g.edge_count(), 24);
    }

    #[test]
    fn centre_node_exists_and_is_passable() {
        let g = build_seven();
        let c = g.node(1).expect("centre must exist");
        assert!(c.is_passable());
        assert_eq!(c.sector, HeptaSector::Centre);
    }

    #[test]
    fn neighbours_returns_correct_count_for_centre() {
        let g = build_seven();
        // Centre has spokes to 6 outer nodes
        assert_eq!(g.neighbours(1).len(), 6);
    }

    #[test]
    fn spoke_chord_centre_to_outer() {
        let chord = HeptaChordType::infer(HeptaSector::Centre, HeptaSector::North);
        assert_eq!(chord, HeptaChordType::Spoke);
    }

    #[test]
    fn rim_chord_adjacent_outer() {
        let chord = HeptaChordType::infer(HeptaSector::North, HeptaSector::NorthEast);
        assert_eq!(chord, HeptaChordType::Rim);
    }

    #[test]
    fn diagonal_chord_non_adjacent_outer() {
        let chord = HeptaChordType::infer(HeptaSector::North, HeptaSector::South);
        assert_eq!(chord, HeptaChordType::Diagonal);
    }

    #[test]
    fn local_chord_same_sector() {
        let chord = HeptaChordType::infer(HeptaSector::North, HeptaSector::North);
        assert_eq!(chord, HeptaChordType::Local);
    }

    #[test]
    fn spoke_effective_cost_less_than_base() {
        let e = NaviEdge::new(1, 2, 500.0, HeptaChordType::Spoke);
        assert!(e.effective_cost() < 500.0);
    }

    #[test]
    fn diagonal_effective_cost_greater_than_base() {
        let e = NaviEdge::new(1, 3, 500.0, HeptaChordType::Diagonal);
        assert!(e.effective_cost() > 500.0);
    }

    #[test]
    fn sector_adjacency_centre_touches_all() {
        for s in [HeptaSector::North, HeptaSector::NorthEast, HeptaSector::SouthEast,
                  HeptaSector::South, HeptaSector::SouthWest, HeptaSector::NorthWest] {
            assert!(HeptaSector::Centre.is_adjacent(s));
        }
    }

    #[test]
    fn sector_adjacency_north_northeast_south() {
        assert!(HeptaSector::North.is_adjacent(HeptaSector::NorthEast));
        assert!(!HeptaSector::North.is_adjacent(HeptaSector::South));
    }

    #[test]
    fn sector_not_adjacent_to_itself() {
        assert!(!HeptaSector::North.is_adjacent(HeptaSector::North));
    }

    #[test]
    fn sector_from_u8_roundtrip() {
        for i in 0..=6u8 {
            let s = HeptaSector::from_u8(i);
            assert_eq!(s.index(), i);
        }
    }

    #[test]
    fn chord_multiplier_order() {
        assert!(HeptaChordType::Spoke.base_multiplier()    < HeptaChordType::Rim.base_multiplier());
        assert!(HeptaChordType::Rim.base_multiplier()      < HeptaChordType::Local.base_multiplier());
        assert!(HeptaChordType::Local.base_multiplier()    < HeptaChordType::Diagonal.base_multiplier());
    }

    #[test]
    fn has_edge_true_and_false() {
        let g = build_seven();
        assert!(g.has_edge(1, 2)); // Centre → North (spoke)
        assert!(!g.has_edge(2, 4)); // North → SouthEast (non-adjacent, no beam)
    }

    #[test]
    fn nodes_in_sector_centre() {
        let g = build_seven();
        let centres = g.nodes_in_sector(HeptaSector::Centre);
        assert_eq!(centres.len(), 1);
    }

    #[test]
    fn empty_map_errors() {
        let m = NaviMap::new();
        assert!(NaviGraph::from_navimap(&m).is_err());
    }

    #[test]
    fn add_and_kill_node() {
        let mut g = build_seven();
        {
            let n = g.node_mut(2).unwrap();
            n.particle.kill();
        }
        assert!(!g.node(2).unwrap().is_passable());
    }
}
