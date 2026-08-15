//! aaol — Akkadian Actor Orchestration Language (`.akk`).
//!
//! Four layers:
//! - Orchestration (`ast`) — tribe/actor/event/route declarations
//! - PMPVD (`pmpvd`) — full 9-node sovereign IR (Particle Meta-Programming)
//! - Simulation (`sim`) — .akk-sim crystalline offline replay format (v4.0.1)
//! - Compiler (`compiler`) — AkkadianAOL compiler pipeline:
//!     `.akk` → Lexer → Parser → Semantic → CodeGen (Rust/Python/JSON/PS/XML)

pub mod token;
pub mod ast;
pub mod pmpvd;
pub mod sim;
pub mod compiler;

pub use token::{Token, tokenize};
pub use ast::{Program, Statement, Parser};
pub use pmpvd::{
    AkkNode, AkkNodeKind, NodeId, PmpvdProgram, PmpvdParser,
    ParticleDecl, FieldDecl, FieldType, QualityConstraint,
    PmpvdTribeDecl,
    RuleDecl, Condition, CompareOp, ConditionValue, RuleAction,
    EquationDecl,
    GuardDecl,
    FlowDecl, FlowEndpoint, PipeStep,
    ObserveDecl, SortSpec, SortOrder,
    EmitDecl,
    PipelineDecl,
};
pub use sim::{SimulationProgram, SimParticle, SimArc, SimRing};
