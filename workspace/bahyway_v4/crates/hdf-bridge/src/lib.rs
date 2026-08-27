#![forbid(unsafe_code)]
//! hdf-bridge — Hepta→DFG translation bridge.
//!
//! Takes a parsed `hepta_spatial::HeptaFile` and produces a `dfg_engine::DfgPlan`
//! that can be executed by the UEK kernel.
//!
//! # Example
//! ```
//! let src = include_str!("../../hepta-spatial/maps/baghdad_primary.hepta");
//! let file   = hepta_spatial::parse(src).unwrap();
//! let plan   = hdf_bridge::translate(&file);
//! assert!(plan.cluster_nodes().count() > 0);
//! ```

pub mod translate;

pub use translate::translate;

// Re-export key dfg-engine types so callers don't need a direct dep.
pub use dfg_engine::{
    DefaultSteward, DfgEdge, DfgNode, DfgPlan, EdgeKind, ExecuteOutcome, FeatureAttrib,
    GovernanceDecision, NodeKind, ParticleState, UekEvent, UekKernel,
};
