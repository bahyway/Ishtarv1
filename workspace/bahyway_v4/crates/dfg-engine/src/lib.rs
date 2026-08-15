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

pub mod node;
pub mod edge;
pub mod governance;
pub mod plan;
pub mod kernel;

pub use node::{DfgNode, NodeKind, NodeId};
pub use edge::{DfgEdge, EdgeKind, EdgeMeta};
pub use governance::{GovernanceDecision, ParticleState, DataSteward, DefaultSteward};
pub use plan::{DfgPlan, FeatureAttrib};
pub use kernel::{UekKernel, KakiStalk, UekEvent, ExecuteOutcome, FlaggedJournal};
