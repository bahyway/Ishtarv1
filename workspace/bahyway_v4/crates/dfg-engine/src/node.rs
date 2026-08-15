#![forbid(unsafe_code)]
//! DFG node types — RootGovernance, ExecutionCluster, ParticleContainer.

/// Stable numeric identity for a node within a `DfgPlan`.
pub type NodeId = u32;

/// The semantic role of a DFG node, derived from its Hepta origin.
///
/// HDF-Bridge mapping (doc §7):
///   ANCHOR  → RootGovernance
///   SECTOR  → ExecutionCluster
///   NODE    → ParticleContainer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// Root governance anchor — policy evaluation entry point.
    RootGovernance,
    /// Execution cluster — spatial sector with routing + attribs.
    ExecutionCluster,
    /// Particle container — nested entity within a cluster.
    ParticleContainer,
}

/// One vertex in the DFG execution graph.
#[derive(Debug, Clone)]
pub struct DfgNode {
    /// Stable identity within this plan (0-indexed, assigned during translation).
    pub id:           NodeId,
    pub kind:         NodeKind,
    /// Human-readable label (e.g. "MOON", "SATURN", "SHRINE").
    pub label:        String,
    /// 0-based sector index from the Hepta file.
    pub sector_index: u8,
    /// Conditional activation expression (`condition:` in Hepta).
    pub condition:    Option<String>,
    /// Feature vector derived from ATTRIB statements.
    pub attribs:      Vec<super::plan::FeatureAttrib>,
}

impl DfgNode {
    pub fn attrib(&self, domain: &str, key: &str) -> Option<&str> {
        self.attribs
            .iter()
            .find(|a| a.domain == domain && a.key == key)
            .map(|a| a.value.as_str())
    }
}
