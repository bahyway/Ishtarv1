#![forbid(unsafe_code)]

// ============================================================================
// hepta/src/ast.rs  — Typed AST for the Hepta Spatial DSL v4.0
//
// Stripped of all serde / external deps.  Pure sovereign Rust.
// ============================================================================

use crate::kaki::{ChordType, CoordSystem, PlanetNode};

// ─── Header ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct HeptaHeader {
    pub version: String,
    pub coord_system: CoordSystem,
    pub project_id: Option<String>,
}

impl Default for HeptaHeader {
    fn default() -> Self {
        Self {
            version: "4.0".into(),
            coord_system: CoordSystem::Kaki7d,
            project_id: None,
        }
    }
}

// ─── Coordinates ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Coord {
    LatLon { lat: f64, lon: f64 },
    Xy { x: f64, y: f64 },
}

// ─── Distance & Angle ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum DistanceUnit {
    Km,
    M,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Distance {
    pub value: f64,
    pub unit: DistanceUnit,
}

impl Distance {
    pub fn to_metres(&self) -> f64 {
        match self.unit {
            DistanceUnit::Km => self.value * 1_000.0,
            DistanceUnit::M => self.value,
        }
    }
}

// ─── Node Reference ──────────────────────────────────────────────────────────

/// Parsed representation of `[H7-01:SADR-CITY]` or similar node refs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRef {
    pub sector_index: u8,
    pub label: String,
    pub planet: PlanetNode,
}

impl NodeRef {
    pub fn new(sector_index: u8, label: String) -> Self {
        let planet = PlanetNode::from_label(&label);
        Self {
            sector_index,
            label,
            planet,
        }
    }
}

// ─── ATTRIB Value ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AttribValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    HexLiteral(u128),
    Text(String),
    Symbol(String),
    Percent(f64), // 85% → 0.85
}

// ─── ATTRIB Statement ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct AttribStmt {
    pub domain: String,
    pub key: String,
    pub value: AttribValue,
}

// ─── BEAM / PATH Statement ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct BeamStmt {
    pub is_path: bool,
    pub chord: ChordType,
    pub target: NodeRef,
    pub properties: BeamProperties,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeamProperties {
    pub latency_ms: Option<f64>,
    pub risk_weight: Option<f64>,
    pub capacity: Option<String>,
    pub security: Option<String>,
    pub status: Option<String>,
    pub hidden_path_probability: Option<f64>,
    pub beam_type: Option<String>,
    pub passability: Option<String>,
    pub historical_status: Option<String>,
}

// ─── ANCHOR Block ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct AnchorBlock {
    pub node: NodeRef,
    pub center: Option<Coord>,
    pub radius: Option<Distance>,
    pub rotation_deg: Option<f64>,
    pub kaki_pk_base: Option<u128>,
    pub identity: Option<String>,
    pub sacred_weight: Option<f64>,
}

// ─── SECTOR Block ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct SectorBlock {
    pub node: NodeRef,
    pub kaki_pk: Option<u128>,
    pub identity: Option<String>,
    pub geometry: Option<SectorGeometry>,
    pub relative_to: Option<NodeRef>,
    pub chord_link: Option<ChordLink>,
    pub condition: Option<String>,
    pub beams: Vec<BeamStmt>,
    pub attribs: Vec<AttribStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectorGeometry {
    pub kind: String, // e.g. "RADIAL_SECTOR"
    pub start_angle: f64,
    pub end_angle: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChordLink {
    pub target: NodeRef,
    pub link_type: String,
}

// ─── NODE Block ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct NodeBlock {
    pub node: NodeRef,
    pub kaki_pk: Option<u128>,
    pub identity: Option<String>,
    pub relative_to: Option<NodeRef>,
    pub condition: Option<String>,
    pub beams: Vec<BeamStmt>,
    pub attribs: Vec<AttribStmt>,
}

// ─── Top-Level File ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct HeptaFile {
    pub header: HeptaHeader,
    pub anchors: Vec<AnchorBlock>,
    pub sectors: Vec<SectorBlock>,
    pub nodes: Vec<NodeBlock>,
}

impl HeptaFile {
    pub fn beam_count(&self) -> usize {
        self.sectors.iter().map(|s| s.beams.len()).sum::<usize>()
            + self.nodes.iter().map(|n| n.beams.len()).sum::<usize>()
    }

    pub fn attrib_count(&self) -> usize {
        self.sectors.iter().map(|s| s.attribs.len()).sum::<usize>()
            + self.nodes.iter().map(|n| n.attribs.len()).sum::<usize>()
    }

    pub fn sector_by_label(&self, label: &str) -> Option<&SectorBlock> {
        self.sectors.iter().find(|s| s.node.label == label)
    }

    pub fn anchor_by_label(&self, label: &str) -> Option<&AnchorBlock> {
        self.anchors.iter().find(|a| a.node.label == label)
    }
}
