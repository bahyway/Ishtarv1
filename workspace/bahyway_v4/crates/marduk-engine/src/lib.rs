//! MardukEngine — Data Particles Modelling Visual Analysis Specialist
//! (BC-MRD-001). "He fixed the stations of the stars and assigned the
//! orbits — and his son reads them."
//!
//! NAMING NOTE: "Nabu Calculus" (BC-MRD-001 §3, the covariant-geometry
//! mathematics named for the god Nabu) is unrelated to `nabu`, the real
//! BahyWay CLI (`docs/11_tooling/nabu_cli.md`). Not a Rust-identifier
//! collision -- this crate doesn't claim the `nabu` binary name -- but
//! worth stating once in code, since both are legitimately "Nabu" in
//! conversation and playbook prose.
//!
//! SCOPE: of BC-MRD-001's five verbs (Position, Motion, Curvature,
//! Topology, Horizon), this crate implements three: `position`,
//! `horizon`, and `topology` (Betti β0/β1 of relation graphs, scoped
//! exactly as BC-MRD-001/GL-MRD-002 define it -- not full simplicial
//! homology). Motion (covariant ∇r/dt across a recalibrated
//! metric/template, §3.2-§3.3) and Curvature (geodesic deviation as
//! systemic cause, §3.4) are NOT implemented yet, though the
//! differential-geometry primitives they need (Christoffel symbols,
//! covariant derivative, the geodesic ODE, Ricci curvature) already
//! exist, independently built and validated, in
//! `vgca_engine::riemannian` -- they were not duplicated here. None of
//! the four domain calculi (Sazu/Addu/Suhrim/Namtila, BC-MRD-001 §4)
//! are implemented -- this crate is domain-agnostic core only.
//!
//! Every domain calculus MUST declare its radial sign convention
//! (§3.6); `SignConvention` below is the domain-neutral declaration
//! point.

pub mod horizon;
pub mod position;
pub mod topology;

pub use horizon::{golden_horizon, GEM_FLOOR_B11};
pub use position::{radial_coordinate, template_score};
pub use topology::{betti_0, betti_1, RelationGraph};

/// BC-MRD-001 §3.6 — the Sign-Convention Law. Every domain calculus
/// must declare which reading of "inward" is healthy; the invariant
/// DVA verbs watch is always the sign of the drift relative to this
/// declared attractor/boundary, never a hardcoded direction.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SignConvention {
    /// MDM/Marduk domains: the golden core is the attractor (falling
    /// inward = healthy consolidation).
    GoldenCoreAttracts,
    /// Enbilulu (WPDEngine): the ERRA core is the fall (falling inward
    /// = defect). Recorded here for cross-domain reference only --
    /// Enbilulu itself is sealed source and untouched by this crate.
    ErraCoreRepels,
}
