#![forbid(unsafe_code)]
//! DFG directed edges — Chord72, Chord73, and Path routing.

use super::node::NodeId;

/// Routing class of a directed DFG edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    /// [CHORD-7/2] — skip-2 planetary chord (e.g. Sun→Mercury).
    Chord72,
    /// [CHORD-7/3] — skip-3 planetary chord (e.g. Sun→Venus).
    Chord73,
    /// PATH beam — navigational / physical route (lower risk, higher latency).
    Path,
}

/// Rich metadata attached to an edge, sourced from Hepta `BEAM`/`PATH` properties.
#[derive(Debug, Clone, Default)]
pub struct EdgeMeta {
    pub latency_ms:              Option<f64>,
    pub risk_weight:             Option<f64>,
    pub capacity:                Option<String>,
    pub security:                Option<String>,
    pub status:                  Option<String>,
    pub hidden_path_probability: Option<f64>,
    pub beam_type:               Option<String>,
}

/// One directed edge in the DFG execution graph.
#[derive(Debug, Clone)]
pub struct DfgEdge {
    pub from:  NodeId,
    pub to:    NodeId,
    pub kind:  EdgeKind,
    pub meta:  EdgeMeta,
    /// Target node label (resolved to `to` during translation; kept for
    /// diagnostics in case the target does not yet exist in the plan).
    pub target_label: String,
}

impl DfgEdge {
    /// Normalised edge weight in [0, 1].  Lower = safer / faster.
    pub fn weight(&self) -> f64 {
        if let Some(r) = self.meta.risk_weight { return r.clamp(0.0, 1.0); }
        if let Some(l) = self.meta.latency_ms  { return (l / 1_000.0).clamp(0.0, 1.0); }
        1.0
    }
}
