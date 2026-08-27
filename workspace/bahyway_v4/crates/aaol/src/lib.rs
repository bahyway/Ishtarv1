//! aaol — Akkadian Actor Orchestration Language (`.akk`).
//!
//! Four layers:
//! - Orchestration (`ast`) — tribe/actor/event/route declarations
//! - PMPVD (`pmpvd`) — full 9-node sovereign IR (Particle Meta-Programming)
//! - Simulation (`sim`) — .akk-sim crystalline offline replay format (v4.0.1)
//! - Compiler (`compiler`) — AkkadianAOL compiler pipeline:
//!   `.akk` → Lexer → Parser → Semantic → CodeGen (Rust/Python/JSON/PS/XML)

pub mod ast;
pub mod compiler;
pub mod pmpvd;
pub mod sim;
pub mod token;

pub use ast::{Parser, Program, Statement};
pub use pmpvd::{
    AkkNode, AkkNodeKind, CompareOp, Condition, ConditionValue, EmitDecl, EquationDecl, FieldDecl,
    FieldType, FlowDecl, FlowEndpoint, GuardDecl, NodeId, ObserveDecl, ParticleDecl, PipeStep,
    PipelineDecl, PmpvdParser, PmpvdProgram, PmpvdTribeDecl, QualityConstraint, RuleAction,
    RuleDecl, SortOrder, SortSpec,
};
pub use sim::{SimArc, SimParticle, SimRing, SimulationProgram};
pub use token::{tokenize, Token};
