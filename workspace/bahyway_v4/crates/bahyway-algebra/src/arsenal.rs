//! The Algebra Arsenal Index — absorbed from the standalone
//! `algebra-arsenal` crate (2026-08-01), at the Architect's direct
//! instruction to put ALL of BahyWay's mathematics in one central
//! place named `bahyway-algebra`.
//!
//! This module implements nothing new. It re-exports the real
//! primitives from the crates that actually implement them, grouped by
//! `ALGEBRA_GLOSSARY.md` section, each with a doc comment citing exact
//! provenance — the same discipline `algebra-arsenal` itself was built
//! under: "nothing is added to this file without a passing test in the
//! same commit. No aspirational re-exports."
//!
//! This crate's own top-level modules (`clifford`, `lie`, `octonion`,
//! `enbilulu`, `functor`, `orbital`, `shells`, `topology`,
//! `persistence`) are already directly reachable as
//! `bahyway_algebra::clifford` etc. — they need no re-export wrapper
//! here, unlike `algebra-arsenal`'s old `top_algebra` module, which
//! existed only because it lived in a *different* crate.
//!
//! ## The one real constraint: not everything can move here
//!
//! Three math homes are deliberately NOT re-exported through this
//! module, because each one already depends on `bahyway-algebra`
//! itself — pulling them in here would create a circular dependency
//! Cargo refuses to compile:
//! - `ammas-engine::markov` (Markov chains) — `ammas-engine` depends on
//!   `bahyway-algebra`.
//! - `homt-engine` (HOMT convergence/field/render-mode math) —
//!   `homt-engine` depends on `bahyway-algebra`.
//! - `compare-tribe-schema::pauli_dedup` — `compare-tribe-schema`
//!   depends on `bahyway-algebra`.
//!
//! Those three stay indexed one level up (in what remains of
//! `algebra-arsenal`, now a thin pointer — see that crate's own doc
//! comment) rather than pretending the cycle doesn't exist.

/// ## Field (Part I) — `bahyway-field`
/// Real. `Field` is a genuine abstract algebraic field (ℝ, via
/// `RealField`). `Zmod240` is the sovereign B11 quantization ring —
/// corrected from the glossary's "field" label, since ℤ/240 has zero
/// divisors (240 is composite) and therefore cannot be a field.
pub mod field {
    pub use bahyway_field::{CommutativeRing, Field, RealField, Zmod240};
}

/// ## Vector Space / Inner Product Space (Part I) — `kinetic-engine::Vec7D`
/// Real. `Vec7D` implements the vector space axioms plus `dot`,
/// `magnitude`, `normalize` over ℝ⁷ — the concrete VGCA-Σ quality
/// space, not a type alias.
pub mod vector_space {
    pub use kinetic_engine::vec7d::Vec7D;
}

/// ## Inner Product Space, weighted (Part I) — `hepta-score`
/// Real. `HeptaVector::weighted_distance` / `health_score` implement
/// H(P) = 1/(1+√Σwᵢ(Pᵢ−Tᵢ)²) against a `TribeIdealPoint`.
pub mod hepta_inner_product {
    pub use hepta_score::domain::{HeptaScore, HeptaVector, TribeIdealPoint};
    pub use hepta_score::equation::{hepta_health_score, weighted_orbit_distance};
}

/// ## Simplex / Simplicial Complex (Part I) — `najaf-engine::topology`
/// Real. `SimplicialComplex` + `is_particle_in_complex` (barycentric
/// membership test in 7D) + `reconstruct_ghost` (missing-vertex
/// reconstruction via Gaussian elimination).
pub mod simplicial {
    pub use najaf_engine::topology::{
        is_particle_in_complex, reconstruct_ghost, SimplicialComplex,
    };
}

/// ## Eigenvalue / Eigenvector, Jordan (Part I) — `ea-agent-algebra`
/// Real, with an honest boundary. `SovereignMatrix::eigenvalues_2x2`
/// computes exact (including complex) eigenvalues via closed form.
/// `jordan::JordanAnalyzer` classifies tribe stability from spectral
/// radius. NOT covered: general Jordan Normal Form (P⁻¹AP with
/// generalized eigenvectors for defective n×n matrices) — nothing in
/// the workspace computes that; said plainly rather than implied.
pub mod eigen {
    pub use ea_agent_algebra::jordan::{JordanAnalyzer, JordanResult, StabilityState};
    pub use ea_agent_algebra::matrix::{Eigenvalue, SovereignMatrix};
}

/// ## Jordan Normal Form, symmetric case (Part I) — `ea-agent-algebra::jnf`
/// Real for symmetric matrices (complete, per the spectral theorem);
/// general defective-matrix JNF remains out of scope (see `eigen`).
pub mod jordan_normal_form {
    pub use ea_agent_algebra::jnf::{jacobi_eigen, jnf_shape_symmetric};
}

/// ## Manifold / Geodesic / Covariant Derivative / Riemannian Curvature
/// (Part I/II) — `vgca-engine::riemannian`
/// Real. General metric-tensor-field machinery, independently verified
/// against a known analytic case (round-sphere Gaussian curvature
/// K=1/r²), not just self-consistent on BahyWay's own flat Nabu metric.
pub mod differential_geometry {
    pub use vgca_engine::riemannian::RiemannianManifold;
}

/// ## Shannon Entropy / KL Divergence (Part V, Layer 7) — `vgca-engine`
pub mod information_theory {
    pub use vgca_engine::kl_divergence::{kl_divergence, normalize, KL_EPSILON};
    pub use vgca_engine::{is_arabic_char, shannon_entropy};
}

/// ## Directed Graph / PageRank / Betweenness Centrality / SCC
/// (Part V, Layer 6) — `graph-engine`
pub mod graph {
    pub use graph_engine::{DirectedGraph, PAGERANK_DAMPING};
}

/// ## Pauli Exclusion (Part I, agent layer) — `ea-agent-algebra::pauli`
/// Real. `PauliChecker::check` scans a particle batch for exclusion
/// violations (two particles occupying too-similar a state) and scores
/// severity; `check_pair` is the pairwise primitive underneath it.
/// See `docs/14_decisions_adr/adr_002_pauli_gates.md` and
/// `docs/04_gates/pauli_exclusion.md` for the sealed law this
/// implements.
pub mod pauli {
    pub use ea_agent_algebra::pauli::{PauliChecker, PauliResult, PauliSeverity, PauliViolation};
    pub use ea_agent_core::ParticleSnapshot;
}

/// ## Pauli Exclusion, longitudinal monitoring — `ea-agent-oracle::pauli_monitor`
/// Real. `PauliMonitor::scan` runs the same `PauliChecker` over
/// successive epochs and tracks `sovereign_streak`/`total_violations`
/// across time — the time-series reading of the same-instant `pauli`
/// module above.
pub mod pauli_monitor {
    pub use ea_agent_oracle::pauli_monitor::{MonitorReport, PauliMonitor};
}

/// ## VGCA Text/Block Validation Geometry (Part I/V) — `vgca-validation`
/// Real. `FieldSignatureVector`/`BlockFeatureVector` (7D/6D geometric
/// fingerprints), `vgca_score`/`geometric_fit` (distance-to-fit
/// classification), `vgca_delta` (block-to-block corruption signal),
/// `infer_column_type` (geometry-driven type inference) — the concrete
/// geometric-fit machinery `bin/bee-watchdog`'s real VGCA-Δ station
/// uses.
pub mod vgca_validation {
    pub use vgca_validation::vgca::{
        geometric_fit, infer_column_type, vgca_delta, vgca_score, BlockFeatureVector,
        ColumnGeometryDescriptor, CorruptionClass, FieldSignatureVector, GeometricFit,
        InferredColumnType, VgcaBlockResult, VgcaTextResult,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_field::{CommutativeRing, Field};

    #[test]
    fn field_real_field_is_usable_through_the_index() {
        let a = field::RealField(2.0);
        let inv = a.inverse().unwrap();
        assert!((a.0 * inv.0 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn field_zmod240_is_reachable_and_still_not_a_field() {
        assert_eq!(field::Zmod240(2).checked_inverse(), None);
    }

    #[test]
    fn vector_space_vec7d_arithmetic_through_the_index() {
        let a = vector_space::Vec7D::from_array([1.0; 7]);
        let b = vector_space::Vec7D::from_array([1.0; 7]);
        let sum = a + b;
        assert_eq!(sum.to_array(), [2.0; 7]);
    }

    #[test]
    fn hepta_inner_product_perfect_particle_scores_one() {
        let ideal = hepta_inner_product::TribeIdealPoint::sovereign_arabic_mdm();
        let p = hepta_inner_product::HeptaVector::perfect();
        assert_eq!(p.health_score(&ideal), 1.0);
    }

    #[test]
    fn simplicial_complex_is_constructible_through_the_index() {
        let verts: Vec<[f64; 7]> = vec![
            [0.0; 7],
            [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ];
        let complex = simplicial::SimplicialComplex::new(verts);
        let _ = complex;
    }

    #[test]
    fn eigen_2x2_complex_pair_through_the_index() {
        let m = eigen::SovereignMatrix::from_slice(2, 2, &[0.0, -1.0, 1.0, 0.0]);
        let ev = m.eigenvalues_2x2();
        assert!(ev.iter().all(|e| !e.is_real()));
    }

    #[test]
    fn jnf_symmetric_eigen_reachable_through_the_index() {
        let a = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
        let (mut eig, _v) = jordan_normal_form::jacobi_eigen(&a, 100, 1e-12);
        eig.sort_by(|x, y| x.partial_cmp(y).unwrap());
        assert!((eig[0] - 1.0).abs() < 1e-6);
        assert!((eig[1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn differential_geometry_sphere_curvature_reachable_through_the_index() {
        let r = 2.0;
        let m = differential_geometry::RiemannianManifold::new(2, move |x: &[f64]| {
            let theta = x[0];
            vec![
                vec![r * r, 0.0],
                vec![0.0, r * r * theta.sin() * theta.sin()],
            ]
        });
        let k = m.gaussian_curvature_2d(&[std::f64::consts::FRAC_PI_2, 0.7]);
        assert!((k - 1.0 / (r * r)).abs() < 1e-2);
    }

    #[test]
    fn information_theory_kl_divergence_reachable_through_the_index() {
        let p = information_theory::normalize(&[1.0, 3.0]);
        let d = information_theory::kl_divergence(&p, &p);
        assert!(d.abs() < 1e-9);
    }

    #[test]
    fn graph_pagerank_reachable_through_the_index() {
        let mut g = graph::DirectedGraph::new(2);
        g.add_edge(0, 1);
        let pr = g.pagerank(graph::PAGERANK_DAMPING, 100, 1e-12);
        assert!((pr.iter().sum::<f64>() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn pauli_checker_reachable_through_the_index() {
        let checker = pauli::PauliChecker::new();
        let result = checker.check(&[]);
        assert!(!result.has_violations());
    }

    #[test]
    fn pauli_monitor_reachable_through_the_index() {
        let mut monitor = pauli_monitor::PauliMonitor::new();
        let report = monitor.scan(1, 0, &[]);
        assert_eq!(report.summary().is_empty(), false);
    }

    #[test]
    fn vgca_validation_score_reachable_through_the_index() {
        // Zero distance from the centroid must classify as Clean.
        let score = vgca_validation::vgca_score(0.0, 1.0);
        assert_eq!(score, 1.0);
        assert_eq!(
            vgca_validation::geometric_fit(score),
            vgca_validation::GeometricFit::Clean
        );
    }
}
