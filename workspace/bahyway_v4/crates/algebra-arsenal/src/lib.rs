//! algebra-arsenal — SUPERSEDED 2026-08-01.
//!
//! At the Architect's direct instruction ("put ALL BahyWay mathematics
//! in one central place and call it bahyway-algebra"), this crate's
//! whole re-export index moved into `bahyway_algebra::arsenal` — same
//! content, same discipline (nothing added without a passing test in
//! the same commit), just under the name the Architect wants.
//!
//! **New code should depend on `bahyway-algebra` directly and use
//! `bahyway_algebra::arsenal::*`, not this crate.**
//!
//! This crate is kept, not deleted (ADR-006, no-delete), as a thin
//! re-export of that same module, plus the ONE piece that genuinely
//! cannot move into `bahyway-algebra`: `ammas-engine::markov`.
//! `ammas-engine` already depends on `bahyway-algebra` (for the same
//! reason `homt-engine` and `compare-tribe-schema` do), so pulling
//! Markov-chain math *into* `bahyway-algebra` would create a circular
//! dependency Cargo refuses to compile. This crate sits one level
//! above `bahyway-algebra` in the dependency graph specifically so it
//! CAN see both sides of that boundary — that is still its one real
//! remaining job.

/// Re-export of everything `bahyway_algebra::arsenal` indexes — field,
/// vector_space, hepta_inner_product, simplicial, eigen,
/// jordan_normal_form, differential_geometry, information_theory,
/// graph, pauli, pauli_monitor, vgca_validation. See that module's own
/// doc comments for exact provenance.
pub use bahyway_algebra::arsenal::*;

/// ## Markov Chain / Steady-State / Mean First Passage Time
/// (Part V, Layer 8) — `ammas-engine::markov`
/// Real. The one math domain that stays indexed here rather than in
/// `bahyway_algebra::arsenal` (see this crate's own doc comment for why).
pub mod markov {
    pub use ammas_engine::markov::MarkovChain;
}

#[cfg(test)]
mod tests {
    use super::*;
    use field::Field;

    #[test]
    fn arsenal_reexports_are_reachable_through_the_superseded_crate() {
        let a = field::RealField(2.0);
        let inv = a.inverse().unwrap();
        assert!((a.0 * inv.0 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn markov_steady_state_reachable_through_the_index() {
        let chain = markov::MarkovChain::new(vec![vec![0.9, 0.1], vec![0.2, 0.8]]);
        let pi = chain.steady_state(1e-12, 10_000);
        assert!((pi.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }
}
