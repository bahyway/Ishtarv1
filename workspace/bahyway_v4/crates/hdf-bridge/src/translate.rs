#![forbid(unsafe_code)]
//! HDF Bridge — Hepta AST → DFG execution plan translation.
//!
//! Mapping (Hepta→DFG bridge doc §7):
//!   ANCHOR  → RootGovernanceNode   — policy evaluation root
//!   SECTOR  → ExecutionCluster     — spatial execution region
//!   NODE    → ParticleContainer    — entity within a cluster
//!   BEAM    → DfgEdge (Chord72/73) — execution routing
//!   PATH    → DfgEdge (Path)       — navigational route
//!   ATTRIB  → FeatureAttrib        — feature vector element

use hepta::{
    HeptaFile, AttribValue, ChordType,
    AnchorBlock, SectorBlock, NodeBlock,
};
use dfg_engine::{
    DfgPlan, DfgNode, DfgEdge, NodeKind, NodeId,
    EdgeKind, EdgeMeta, FeatureAttrib,
};

/// Translate a parsed `HeptaFile` into a `DfgPlan`.
///
/// All node IDs are assigned incrementally in declaration order:
/// anchors first, then sectors, then node blocks.
///
/// Edge targets are resolved in a second pass by matching the `target_label`
/// against node labels.  Unresolved edges keep `to = NodeId::MAX`.
pub fn translate(file: &HeptaFile) -> DfgPlan {
    let mut nodes: Vec<DfgNode> = Vec::new();
    let mut edges: Vec<DfgEdge> = Vec::new();
    let mut counter: NodeId     = 0;

    // ── 1. ANCHOR → RootGovernanceNode ───────────────────────────────────────
    for anchor in &file.anchors {
        let id = counter; counter += 1;
        nodes.push(anchor_to_node(anchor, id));
    }

    // ── 2. SECTOR → ExecutionCluster ─────────────────────────────────────────
    for sector in &file.sectors {
        let id = counter; counter += 1;
        let attribs = attribs_from_sector(sector);
        nodes.push(DfgNode {
            id,
            kind:         NodeKind::ExecutionCluster,
            label:        sector.node.label.clone(),
            sector_index: sector.node.sector_index,
            condition:    sector.condition.clone(),
            attribs,
        });
        for beam in &sector.beams {
            edges.push(DfgEdge {
                from:         id,
                to:           NodeId::MAX,  // resolved in pass 2
                kind:         beam_edge_kind(beam.is_path, beam.chord),
                meta:         props_to_meta(&beam.properties),
                target_label: beam.target.label.clone(),
            });
        }
    }

    // ── 3. NODE → ParticleContainer ──────────────────────────────────────────
    for node_block in &file.nodes {
        let id = counter; counter += 1;
        let attribs = attribs_from_node(node_block);
        nodes.push(DfgNode {
            id,
            kind:         NodeKind::ParticleContainer,
            label:        node_block.node.label.clone(),
            sector_index: node_block.node.sector_index,
            condition:    node_block.condition.clone(),
            attribs,
        });
        for beam in &node_block.beams {
            edges.push(DfgEdge {
                from:         id,
                to:           NodeId::MAX,
                kind:         beam_edge_kind(beam.is_path, beam.chord),
                meta:         props_to_meta(&beam.properties),
                target_label: beam.target.label.clone(),
            });
        }
    }

    // ── 4. Resolve edge targets ───────────────────────────────────────────────
    for edge in &mut edges {
        if let Some(target) = nodes.iter().find(|n| n.label == edge.target_label) {
            edge.to = target.id;
        }
    }

    DfgPlan {
        project_id:   file.header.project_id.clone(),
        coord_system: format!("{:?}", file.header.coord_system),
        nodes,
        edges,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn anchor_to_node(anchor: &AnchorBlock, id: NodeId) -> DfgNode {
    let mut attribs = Vec::new();
    if let Some(w) = anchor.sacred_weight {
        attribs.push(FeatureAttrib {
            domain: "anchor".into(), key: "sacred_weight".into(),
            value:  format!("{w}"),
        });
    }
    if let Some(ref identity) = anchor.identity {
        attribs.push(FeatureAttrib {
            domain: "anchor".into(), key: "identity".into(),
            value:  identity.clone(),
        });
    }
    if let Some(ref radius) = anchor.radius {
        attribs.push(FeatureAttrib {
            domain: "anchor".into(), key: "radius_m".into(),
            value:  format!("{}", radius.to_metres()),
        });
    }
    DfgNode {
        id,
        kind:         NodeKind::RootGovernance,
        label:        anchor.node.label.clone(),
        sector_index: anchor.node.sector_index,
        condition:    None,
        attribs,
    }
}

fn attribs_from_sector(sector: &SectorBlock) -> Vec<FeatureAttrib> {
    let mut v: Vec<FeatureAttrib> = sector.attribs.iter().map(attrib_stmt_to_feature).collect();
    if let Some(ref cond) = sector.condition {
        v.push(FeatureAttrib { domain: "governance".into(), key: "condition".into(), value: cond.clone() });
    }
    if let Some(ref cl) = sector.chord_link {
        v.push(FeatureAttrib { domain: "routing".into(), key: "chord_link_target".into(), value: cl.target.label.clone() });
        v.push(FeatureAttrib { domain: "routing".into(), key: "chord_link_type".into(),   value: cl.link_type.clone()    });
    }
    v
}

fn attribs_from_node(node_block: &NodeBlock) -> Vec<FeatureAttrib> {
    let mut v: Vec<FeatureAttrib> = node_block.attribs.iter().map(attrib_stmt_to_feature).collect();
    if let Some(ref cond) = node_block.condition {
        v.push(FeatureAttrib { domain: "governance".into(), key: "condition".into(), value: cond.clone() });
    }
    v
}

fn attrib_stmt_to_feature(a: &hepta::AttribStmt) -> FeatureAttrib {
    FeatureAttrib {
        domain: a.domain.clone(),
        key:    a.key.clone(),
        value:  attrib_value_to_str(&a.value),
    }
}

fn attrib_value_to_str(v: &AttribValue) -> String {
    match v {
        AttribValue::Integer(n)    => n.to_string(),
        AttribValue::Float(f)      => format!("{f}"),
        AttribValue::Bool(b)       => b.to_string(),
        AttribValue::HexLiteral(h) => format!("{h:#x}"),
        AttribValue::Text(s)       => s.clone(),
        AttribValue::Symbol(s)     => s.clone(),
        AttribValue::Percent(p)    => format!("{:.4}", p),
    }
}

fn beam_edge_kind(is_path: bool, chord: ChordType) -> EdgeKind {
    if is_path { return EdgeKind::Path; }
    match chord {
        ChordType::Chord72 => EdgeKind::Chord72,
        ChordType::Chord73 => EdgeKind::Chord73,
    }
}

fn props_to_meta(p: &hepta::BeamProperties) -> EdgeMeta {
    EdgeMeta {
        latency_ms:              p.latency_ms,
        risk_weight:             p.risk_weight,
        capacity:                p.capacity.clone(),
        security:                p.security.clone(),
        status:                  p.status.clone(),
        hidden_path_probability: p.hidden_path_probability,
        beam_type:               p.beam_type.clone(),
    }
}
