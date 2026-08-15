// ============================================================================
// dagan-engine — Sovereign pure-Rust data-analysis station
// BahyWay.Ecosystem v4.0
//
// From the iPhone-session discussion that proposed a "ShamashEngine" data
// analysis station for BeeMDM's ETL chain, embedding Polars/nalgebra/ndarray
// as pure-Rust Pandas/NumPy alternatives. Renamed: SHAMASH already names
// something else real in this workspace (Gate 4 -- dead-particle/zombie
// judgment, docs/04_gates/shamash_gate.md). Named for Dagan instead.
//
// Zero third-party math dependency: every algorithm here (Welford,
// streaming covariance + power-iteration PCA, projection, deterministic
// k-means seeding, Mahalanobis distance via Gauss-Jordan inversion) is
// implemented natively, matching the original discussion's own conclusion
// that Pandas/NumPy have no real gap pure Rust can't close.
// ============================================================================
#![forbid(unsafe_code)]

pub mod covariance;
pub mod kmeans;
pub mod mahalanobis;
pub mod welford;

pub const D: usize = 7; // Hepta dimension, matches this ecosystem's own 7D particle space.

pub use covariance::{PcaResult, StreamingCovariance};
pub use kmeans::{farthest_first_init, lloyd_iterate, KMeansResult};
pub use mahalanobis::{invert_symmetric, mahalanobis_distance};
pub use welford::WelfordStats;
