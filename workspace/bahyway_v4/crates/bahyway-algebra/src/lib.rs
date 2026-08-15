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

pub mod clifford;
pub mod fields;
pub mod lie;
pub mod rotor;
pub mod functor;
pub mod topology;
pub mod axioms;
pub mod shells;
pub mod orbital;
pub mod enlil;
pub mod anshar;
pub mod enbilulu;
pub mod octonion;
pub mod persistence;
pub mod arsenal;

pub use functor::{Decode, Interpret, Evolve};
pub use topology::{
    // Set-theoretic
    interior, boundary, closure, exterior,
    // DE-9IM binary relations
    disjoint, intersects, within, touches, overlaps,
    // Algebraic topology
    boundary_n, exterior_derivative, cup_product, cap_product, hodge_star,
    // Morphological
    dilation, erosion, opening, closing,
    // Field (vector calculus)
    gradient, divergence, curl, laplacian,
};
pub use axioms::{PA13_THETA_MAX, trajectory_momentum};
pub use shells::{quality_distance, shell_boundaries, shell_index, b11_to_shell,
                 sovereign_5_shells, sovereign_shell_label};
pub use orbital::{OrbitalPosition, orbital_position, orbital_ring_layer,
                  sub_ring_azimuth, altitude_scatter,
                  RIM_BIRTH_THRESHOLD, is_rim_particle};

pub use enlil::{
    ENLIL_ALGEBRA, EnlilGate, JnfShape,
    QuantumState, QuantumCoord, ExclusionResult, pauli_check,
};
pub use anshar::{
    SIGMA_BW, ANSHAR_ALGEBRA, SigmaOp, AnsharParticle,
    VerificationResult, AnsharEngine,
};
pub use fields::{SemanticField, SemanticPhase};
pub use persistence::{
    Point3, PersistencePair, PersistenceDiagram, vietoris_rips_persistence,
};

pub mod prelude {
    pub use super::functor::{Decode, Interpret, Evolve};
    pub use super::axioms::{PA13_THETA_MAX, trajectory_momentum};
    pub use super::shells::{quality_distance, shell_index, sovereign_5_shells};
    pub use super::orbital::{OrbitalPosition, orbital_position};
    pub use super::enlil::{ENLIL_ALGEBRA, EnlilGate, JnfShape, QuantumState, pauli_check};
    pub use super::anshar::{ANSHAR_ALGEBRA, SigmaOp, AnsharParticle, AnsharEngine};
}
