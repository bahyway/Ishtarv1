//! # 𒁾 akkadi-ir
//!
//! **AkkadianAOL Sovereign Intermediate Representation**
//! — BahyWay.Ecosystem v4.0
//!
//! The target-agnostic tree that every `.akk` source file compiles into
//! before any backend touches it. Adding a new backend means writing one
//! `AkkBackend` implementor — no parser changes needed.
//!
//! ## Zero Dependencies
//!
//! Pure `std` Rust — sovereign law for all foundation-layer crates.

#![allow(missing_docs)]
#![forbid(unsafe_code)]

pub mod backend;
pub mod errors;
pub mod ir;
pub mod kinetic;
pub mod node;
pub mod node_id;
pub mod quality;
pub mod span;
pub mod walker;

pub use backend::{AkkBackend, AkkGenBackend, DebugBackend, RustHintBackend};
pub use errors::IrError;
pub use ir::{AkkIr, IrBuilder};
pub use kinetic::{ForceKind, KineticForce};
pub use node::{
    AkkNode, ConditionOp, ConditionRhs, EmitContext, EmitNode, EmitTarget, EquationBody,
    EquationNode, FieldDecl, FlowNode, FlowSink, FlowSource, FlowStage, GuardNode, KineticTerm,
    ObserveFilter, ObserveFrom, ObserveNode, ObserveProperty, ObserveSort, ParticleNode,
    PipelineNode, QualityConstraint, RuleAction, RuleCondition, RuleNode, TribeNode, WeightProfile,
};
pub use node_id::{NodeId, NodeIdBuilder};
pub use quality::{HeptaDim, QualityLane, HEPTA_DIMS};
pub use span::Span;
pub use walker::{
    AkkWalker, CollectNames, FindGenerative, NodeCounter, ValidateQuality, WalkAction,
};

// ── ADR-001 sovereign constants ───────────────────────────────────────────────

/// Quality divisor — always 240.0, never 255 (ADR-001)
pub const QUALITY_DIVISOR: f32 = 240.0;

/// Minimum B11 for Golden Record (Gem lane)
pub const GEM_B11: u8 = 200;

/// Minimum B11 for Tribe membership
pub const TRIBE_B11: u8 = 140;

/// Minimum B11 for Active lane
pub const ACTIVE_B11: u8 = 100;

/// Maximum B11 for Fuzzy lane boundary (59–99 = Fuzzy)
pub const FUZZY_DEAD_BOUNDARY: u8 = 59;

/// Target Golden Record rate (ADR-004)
pub const GEM_RATE_TARGET: f32 = 0.354;

/// Version string for this IR specification
pub const IR_VERSION: &str = "akkadi-ir-v4";
