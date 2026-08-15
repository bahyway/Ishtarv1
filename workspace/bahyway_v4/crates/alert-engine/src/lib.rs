//! alert-engine — ColorID drift monitoring and early-warning alerting (§4.5)
//!
//! The AlertEngine continuously monitors color drift and radial drift
//! across all Particles.  When a Particle's drift pattern matches a known
//! diagnostic signature, it flags the Particle to the Data Steward Station.
//!
//! This is the mechanism that catches:
//!   Defect  — manufacturing or data-entry errors
//!   Fraud   — deliberate distortion patterns
//!   Infection — propagation of bad data from a corrupted source
//!
//! Uses Index 5 (ColorID Spatial Index) to query drifters in O(k)
//! where k is the number of drifting particles, not total particles.

pub mod alert;
pub mod engine;

pub use alert::{Alert, AlertSeverity, DriftCause};
pub use engine::AlertEngine;

pub mod prelude {
    pub use super::alert::{Alert, AlertSeverity, DriftCause};
    pub use super::engine::AlertEngine;
}
