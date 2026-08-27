//! # kinetic-engine — The Mathematical Substrate of BahyWay.Ecosystem
//!
//! Implements the sovereign 7D vector space, discrete kinetic force
//! integration, fuzzy membership functions, and health scoring —
//! all expressed in BahyWay's own Heptagon dimensional vocabulary.
//!
//! ## The Kinetic Equation
//!
//! ```text
//! df/dt = I_phys + I_mem + I_learn
//! ```
//!
//! - `I_phys`  — Physical force: vault-engine Gray-Rot, IO friction, storage health
//! - `I_mem`   — Memory force:   EnkiDB EAV history, cardinality staleness
//! - `I_learn` — Learning force: Oracle Egg rewrites, AkkadianAOL rule activation
//!
//! This equation is implemented here as executable, testable, pure Rust.
//! Every particle in every BahyWay crate shares this mathematical substrate.
//!
//! ## Modules
//!
//! - [`vec7d`]       — The 7D sovereign vector type and all algebraic operations
//! - [`force`]       — Force generators: I_phys, I_mem, I_learn
//! - [`accumulator`] — KineticParticle + SovereignAccumulator (Euler integrator)
//! - [`fuzzy`]       — Fuzzy membership functions + ScoringEngine
//! - [`plimpton`]    — Plimpton 322 boundary triggers for sovereign tier transitions
//!
//! ## Architectural Position
//!
//! kinetic-engine sits at Layer 5 in the dependency graph.
//! Zero dependencies. Pure `std`. Every BahyWay crate may import it
//! without introducing circular dependencies.
//!
//! DUB.SAR 𒁾

#![forbid(unsafe_code)]

pub mod accumulator;
pub mod force;
pub mod fuzzy;
pub mod plimpton;
pub mod vec7d;

// Flat re-exports — the sovereign public surface
pub use accumulator::{KineticParticle, SovereignAccumulator};
pub use force::{
    ConstantForce, ForceGenerator, HeptaDimension, LearningForce, MemoryForce, PhysicalForce,
};
pub use fuzzy::{FuzzyRule, HealthClassification, MembershipFunction, ScoringEngine};
pub use plimpton::{PlimptonAnchors, PlimptonTrigger, TriggerAction};
pub use vec7d::Vec7D;
