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

pub mod pauli;
pub mod jordan;
pub mod harmony;
pub mod solver;
pub mod matrix;
pub mod jnf;

pub use pauli::{PauliChecker, PauliResult, PauliViolation};
pub use jordan::{JordanAnalyzer, JordanResult, StabilityState};
pub use harmony::{HarmonyEngine, HarmonyResult};
pub use solver::{AlgebraSolver, SolverResult, Equation};
pub use matrix::{SovereignMatrix, Eigenvalue};
