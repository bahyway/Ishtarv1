//! ea-agent-algebra — Sovereign pure Rust algebraic engine for EaAgent.
//!
//! Zero external dependencies. All mathematics implemented from first principles.
//!
//! Implements:
//!   - Pauli Exclusion Principle (particle identity collision detection)
//!   - Jordan Normal Form (tribe stability analysis)
//!   - Spectral Radius (collapse prediction)
//!   - Harmony Coefficient (tribe quality aggregate)
//!   - BIGRING TOP: Tribe Algebra + Orbits Calculus + Particles Algebra
//!   - Quadratic & polynomial solver (exact sovereign arithmetic)
#![forbid(unsafe_code)]

pub mod harmony;
pub mod jnf;
pub mod jordan;
pub mod matrix;
pub mod pauli;
pub mod solver;

pub use harmony::{HarmonyEngine, HarmonyResult};
pub use jordan::{JordanAnalyzer, JordanResult, StabilityState};
pub use matrix::{Eigenvalue, SovereignMatrix};
pub use pauli::{PauliChecker, PauliResult, PauliViolation};
pub use solver::{AlgebraSolver, Equation, SolverResult};
