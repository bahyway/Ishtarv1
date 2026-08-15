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

pub mod node;
pub mod node_id;
pub mod ir;
pub mod backend;
pub mod walker;
pub mod quality;
pub mod kinetic;
pub mod span;
pub mod errors;

pub use node::{
    AkkNode,
    ParticleNode, TribeNode, RuleNode, EquationNode,
    FlowNode, ObserveNode, EmitNode, GuardNode, PipelineNode,
    FieldDecl, QualityConstraint, WeightProfile,
    RuleCondition, RuleAction, ConditionRhs, ConditionOp,
    FlowStage, FlowSource, FlowSink,
    ObserveFrom, ObserveFilter, ObserveSort, ObserveProperty,
    EmitTarget, EmitContext,
    KineticTerm, EquationBody,
};
pub use node_id::{NodeId, NodeIdBuilder};
pub use ir::{AkkIr, IrBuilder};
pub use backend::{AkkBackend, DebugBackend, AkkGenBackend, RustHintBackend};
pub use walker::{AkkWalker, WalkAction, CollectNames, ValidateQuality, NodeCounter, FindGenerative};
pub use quality::{QualityLane, HeptaDim, HEPTA_DIMS};
pub use kinetic::{KineticForce, ForceKind};
pub use span::Span;
pub use errors::IrError;

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
