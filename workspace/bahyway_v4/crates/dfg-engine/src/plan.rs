#![forbid(unsafe_code)]
//! DfgPlan — the complete, translated DFG execution plan for one Hepta file.

use super::node::{DfgNode, NodeId, NodeKind};
use super::edge::DfgEdge;

/// A single EAV-style feature derived from a Hepta ATTRIB statement.
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureAttrib {
    pub domain: String,
    pub key:    String,
    pub value:  String,
}

/// Complete DFG plan produced by translating one `HeptaFile`.
///
/// The plan is directed acyclic in the governance evaluation direction
/// (RootGovernance → ExecutionCluster → ParticleContainer), but edges from
/// BEAM/PATH statements may cross cluster boundaries forming the actual
/// execution flow topology.
#[derive(Debug, Clone)]
pub struct DfgPlan {
    pub project_id:   Option<String>,
    pub coord_system: String,
    pub nodes:        Vec<DfgNode>,
    pub edges:        Vec<DfgEdge>,
}

impl DfgPlan {
    pub fn node_by_id(&self, id: NodeId) -> Option<&DfgNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_by_label(&self, label: &str) -> Option<&DfgNode> {
        self.nodes.iter().find(|n| n.label == label)
    }

    pub fn edges_from(&self, id: NodeId) -> impl Iterator<Item = &DfgEdge> {
        self.edges.iter().filter(move |e| e.from == id)
    }

    pub fn edges_to(&self, id: NodeId) -> impl Iterator<Item = &DfgEdge> {
        self.edges.iter().filter(move |e| e.to == id)
    }

    pub fn root_nodes(&self) -> impl Iterator<Item = &DfgNode> {
        self.nodes.iter().filter(|n| n.kind == NodeKind::RootGovernance)
    }

    pub fn cluster_nodes(&self) -> impl Iterator<Item = &DfgNode> {
        self.nodes.iter().filter(|n| n.kind == NodeKind::ExecutionCluster)
    }

    pub fn container_nodes(&self) -> impl Iterator<Item = &DfgNode> {
        self.nodes.iter().filter(|n| n.kind == NodeKind::ParticleContainer)
    }

    /// Total nodes.
    pub fn node_count(&self) -> usize { self.nodes.len() }

    /// Total directed edges.
    pub fn edge_count(&self) -> usize { self.edges.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::{DfgEdge, EdgeKind, EdgeMeta};
    use crate::node::{DfgNode, NodeKind};

    fn make_plan() -> DfgPlan {
        let nodes = vec![
            DfgNode { id: 0, kind: NodeKind::RootGovernance,  label: "ANCHOR".into(), sector_index: 0, condition: None, attribs: vec![] },
            DfgNode { id: 1, kind: NodeKind::ExecutionCluster, label: "MOON".into(),  sector_index: 1, condition: None, attribs: vec![] },
            DfgNode { id: 2, kind: NodeKind::ExecutionCluster, label: "MARS".into(),  sector_index: 4, condition: Some("ELEVATED_RISK".into()), attribs: vec![] },
        ];
        let edges = vec![
            DfgEdge { from: 1, to: 2, kind: EdgeKind::Chord72, meta: EdgeMeta::default(), target_label: "MARS".into() },
        ];
        DfgPlan { project_id: Some("test".into()), coord_system: "Kaki7d".into(), nodes, edges }
    }

    #[test]
    fn node_lookup_by_label() {
        let p = make_plan();
        assert_eq!(p.node_by_label("MOON").unwrap().sector_index, 1);
        assert!(p.node_by_label("VENUS").is_none());
    }

    #[test]
    fn edges_from_moon() {
        let p = make_plan();
        let moon = p.node_by_label("MOON").unwrap();
        let edges: Vec<_> = p.edges_from(moon.id).collect();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].kind, EdgeKind::Chord72);
    }

    #[test]
    fn root_cluster_counts() {
        let p = make_plan();
        assert_eq!(p.root_nodes().count(),    1);
        assert_eq!(p.cluster_nodes().count(), 2);
        assert_eq!(p.container_nodes().count(), 0);
    }
}
