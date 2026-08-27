#![forbid(unsafe_code)]
//! dfg-engine — Data Flow Graph execution model for the BahyWay UEK.
//!
//! Provides the graph types, governance decisions, and execution kernel
//! that implement the UEK sheaf execution pipeline (docs §20, §22, §23).
//!
//! # Layers
//! - [`node`] / [`edge`] / [`plan`] — DFG graph data model
//! - [`governance`]                 — GovernanceDecision, ParticleState, DataSteward
//! - [`kernel`]                     — UekKernel: event execution loop

pub mod edge;
pub mod governance;
pub mod kernel;
pub mod node;
pub mod plan;

pub use edge::{DfgEdge, EdgeKind, EdgeMeta};
pub use governance::{DataSteward, DefaultSteward, GovernanceDecision, ParticleState};
pub use kernel::{ExecuteOutcome, FlaggedJournal, KakiStalk, UekEvent, UekKernel};
pub use node::{DfgNode, NodeId, NodeKind};
pub use plan::{DfgPlan, FeatureAttrib};
