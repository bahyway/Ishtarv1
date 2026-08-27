//! bahyway-algebra — BahyWay Unified Algebra (BUA) v4.0
//!
//! Honorary title (sealed 2026-08-01, naming-registry): **Ṭupšarrūtu**
//! ("the scribal art," root of DUB.SAR) — already the sealed name of
//! TOP Algebra (`docs/01_mathematics/top_algebra.md`), the mathematics
//! this crate now holds. The crate's own Cargo package name stays
//! plain `bahyway-algebra` deliberately (a rename here would touch
//! 135+ tests and every crate `arsenal` indexes, for no functional
//! gain) -- Ṭupšarrūtu is this crate's name in prose and documentation,
//! never a `pub mod`/type identifier.
//!
//! CORRECTED 2026-07-10: this is the crate that at least four sealed
//! concept documents (HS-EXT-002's epsilon_geo rule, GL-MRD-002's Nabu
//! metric g=diag(w1..w7), GL-DST-001's "Theater is the stage, GeoEngine
//! is the truth" boundary, TPL-001's index-covered-proof law) refer to
//! by the name "GeoEngine." A prior pass this session built a separate,
//! empty `geo-engine` crate instead of checking whether the real math
//! home already existed here -- it did, with real dependents already
//! (ammas-engine, homt-engine, tribe-orbit-engine, and
//! bin/enkidb-query-server itself). That mistake is corrected: Enbilulu
//! Calculus (see `enbilulu`) now lives here, and the standalone
//! `geo-engine` crate has been removed.
//!
//! This crate encodes the category-theory decomposition HOMT = O∘F∘D:
//!   D  (Decode)    — raw bytes → decoded particle data
//!   F  (Interpret) — decoded data → projected scores / fuzzy values
//!   O  (Evolve)    — current state + event → next state (CQRS fold)
//!
//! All 20 topological operators are in `topology`.
//! PA-13 (Trajectory Momentum) is in `axioms`.
//! PA-14 (Orbital Position) is in `orbital`.
//! PA-13 Shell Decomposition is in `shells`.
//! Enbilulu Calculus (junction defect potential, TIAMAT bands, horizon
//! prediction, Terru diagnosis, Milu alerts) is in `enbilulu`.
//! Persistent homology (H0/H1 over a Vietoris-Rips filtration — the
//! sole math-truth source for LamassuEngine's TDA sentinel) is in
//! `persistence`.
//!
//! Every OTHER real math domain in the ecosystem that does not already
//! depend on this crate (field theory, vector spaces, weighted inner
//! product, simplicial complexes, eigenvalues/Jordan analysis,
//! Riemannian geometry, information theory, graph algorithms, Pauli
//! exclusion, VGCA validation geometry) is re-exported, with real
//! passing tests proving reachability, in `arsenal` — absorbed from
//! the standalone `algebra-arsenal` crate 2026-08-01 at the Architect's
//! direct instruction to put ALL BahyWay mathematics in one central,
//! named place. Three domains stay indexed one level up (in what
//! remains of `algebra-arsenal`) because they already depend on this
//! crate and pulling them in here would be circular: `ammas-engine`
//! (Markov chains), `homt-engine`, `compare-tribe-schema::pauli_dedup`.
//!
//! Permanent constraint: zero third-party (crates.io) dependencies —
//! internal workspace crates (e.g. `alert-engine`, for Milu's alert
//! bridge) are not third-party and don't violate this.

pub mod anshar;
pub mod arsenal;
pub mod axioms;
pub mod clifford;
pub mod enbilulu;
pub mod enlil;
pub mod fields;
pub mod functor;
pub mod lie;
pub mod octonion;
pub mod orbital;
pub mod persistence;
pub mod rotor;
pub mod shells;
pub mod topology;

pub use axioms::{trajectory_momentum, PA13_THETA_MAX};
pub use functor::{Decode, Evolve, Interpret};
pub use orbital::{
    altitude_scatter, is_rim_particle, orbital_position, orbital_ring_layer, sub_ring_azimuth,
    OrbitalPosition, RIM_BIRTH_THRESHOLD,
};
pub use shells::{
    b11_to_shell, quality_distance, shell_boundaries, shell_index, sovereign_5_shells,
    sovereign_shell_label,
};
pub use topology::{
    boundary,
    // Algebraic topology
    boundary_n,
    cap_product,
    closing,
    closure,
    cup_product,
    curl,
    // Morphological
    dilation,
    // DE-9IM binary relations
    disjoint,
    divergence,
    erosion,
    exterior,
    exterior_derivative,
    // Field (vector calculus)
    gradient,
    hodge_star,
    // Set-theoretic
    interior,
    intersects,
    laplacian,
    opening,
    overlaps,
    touches,
    within,
};

pub use anshar::{
    AnsharEngine, AnsharParticle, SigmaOp, VerificationResult, ANSHAR_ALGEBRA, SIGMA_BW,
};
pub use enlil::{
    pauli_check, EnlilGate, ExclusionResult, JnfShape, QuantumCoord, QuantumState, ENLIL_ALGEBRA,
};
pub use fields::{SemanticField, SemanticPhase};
pub use persistence::{vietoris_rips_persistence, PersistenceDiagram, PersistencePair, Point3};

pub mod prelude {
    pub use super::anshar::{AnsharEngine, AnsharParticle, SigmaOp, ANSHAR_ALGEBRA};
    pub use super::axioms::{trajectory_momentum, PA13_THETA_MAX};
    pub use super::enlil::{pauli_check, EnlilGate, JnfShape, QuantumState, ENLIL_ALGEBRA};
    pub use super::functor::{Decode, Evolve, Interpret};
    pub use super::orbital::{orbital_position, OrbitalPosition};
    pub use super::shells::{quality_distance, shell_index, sovereign_5_shells};
}
