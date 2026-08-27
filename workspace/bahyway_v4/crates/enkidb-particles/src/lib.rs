//! enkidb-particles — The 7D EAV Particle, sovereign unit of stored knowledge.
//!
//! Every assertion in EnkiDB is stored as a `Particle`:
//!
//!   P = (E, A, V, T, S, Z, M)
//!
//!   E — Entity:         IdentityKaki  — which thing this is about
//!   A — Attribute:      String        — what dimension of that thing
//!   V — Value:          AkkValue      — the assertion (31-variant sovereign type)
//!   T — Time:           u64           — epoch of this assertion (UNIX seconds)
//!   S — Shard:          u32           — shard identifier (0 = MVP single-shard)
//!   Z — ProofContext:   u32           — proof/verification context (0 = unverified)
//!   M — Materialization: u32          — materialized view tag (0 = base particle)
//!
//! At MVP, S/Z/M are always 0. They are present in the struct for
//! forward-compatibility: when full 7D is introduced no data migration is needed.
//!
//! Architectural constraints (from BahyWay v4.0):
//!   - Birth-state particles (M=0) are sealed at creation; never mutated.
//!   - Quality scores live here as AkkValue::QualityScore, not in KAKI bytes.
//!   - Cross-tribe relationships (Riksu) are Particles with E = CrossTribeKaki.
//!   - "Now" semantics: the current value of an attribute is the Particle with
//!     the largest T for that (E, A) pair.

#![forbid(unsafe_code)]

pub mod null_semantics;
pub mod particle;

pub use null_semantics::NullSemantics;
pub use particle::{Particle, ParticleBirthState};
