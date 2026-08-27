//! homt-engine — Hepta-Orbit Manifold Theory (HOMT) v4.0
//!
//! Formal system: H = (P, G, Φ, Ψ, O)
//!   P  — KAKI particles (IdentityKaki)
//!   G  — Hepta 7-sector manifold (7D index space)
//!   Φ  — fuzzy/score interpretation field  (read-only projection, the F functor)
//!   Ψ  — routing potential field (gradient flow, drives BEAM/PATH)
//!   O  — orbit evolution operator (deterministic, the O functor)
//!
//! Core PDE (data density field):
//!   ∂φ/∂t = −∇·(φV) + G(x,t) − S(x,t) − Δφ
//!
//!   where:
//!     φ         = particle data density at position x, time t
//!     V = -∇Ψ  = routing velocity (gradient flow on quality potential)
//!     G(x,t)   = source term  (REGISTERED / INITIAL_SCAN events)
//!     S(x,t)   = sink term    (QUARANTINED / DEAD events)
//!     Δφ       = Laplacian smoothing (VGCA-Δ / SCORED events)
//!
//! Convergence theorem (PA-∞):
//!   If Ψ ∈ C¹, Φ bounded, step η < 2/L  (L = Lipschitz constant of Ψ+Φ coupling)
//!   then lim(t→∞) K_t → K*  where ∇Ψ(K*) = 0.
//!
//! Quality thresholds (mathematically grounded convergence boundaries):
//!   DEAD   boundary: 0.2
//!   FUZZY / GOLDEN boundary: 0.6
//!
//! Permanent constraint: zero third-party dependencies.

pub mod convergence;
pub mod field;
pub mod render_modes;
pub mod sector;

pub use convergence::{
    is_converged, lipschitz_bound, state_evolution, trajectory, Matrix7, StateVec7, IDENTITY_7,
    MAX_STABLE_STEP, QUALITY_DEAD_BOUNDARY, QUALITY_GOLDEN_BOUNDARY, ZERO_7,
};
pub use field::OrbitField;
pub use render_modes::{
    rendering_mode, tribe_rendering_mode, RenderingMode, INSTANCED_THRESHOLD,
    POINT_SPRITE_THRESHOLD,
};
pub use sector::HeptaSectorMap;

pub const HOMT_VERSION: &str = "4.0.0";
pub const HEPTA_DIMENSIONS: usize = 7;

pub mod prelude {
    pub use super::convergence::{QUALITY_DEAD_BOUNDARY, QUALITY_GOLDEN_BOUNDARY};
    pub use super::field::OrbitField;
    pub use super::render_modes::{rendering_mode, RenderingMode, POINT_SPRITE_THRESHOLD};
    pub use super::sector::HeptaSectorMap;
}
