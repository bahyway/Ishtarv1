//! diagnosis-engine — real-time Color_ID(RGB) drift watcher (§8.2).
//!
//! Reads `color_id_snapshot` EAV triples from the StoryWay Journal and
//! computes drift against each particle's Tribe Root Color.  When drift
//! crosses sovereign thresholds, a DiagnosisWatch / DiagnosisWarning /
//! DiagnosisCritical event is written back to the Journal so HeptaScript
//! can answer temporal color-health queries.

pub mod color_drift;
pub mod engine;

pub use color_drift::{drift_axis, rgb_drift, DriftAxis};
pub use engine::{DiagnosisEngine, DiagnosisResult, DriftThresholds};

pub mod prelude {
    pub use super::color_drift::{rgb_drift, DriftAxis};
    pub use super::engine::{DiagnosisEngine, DiagnosisResult, DriftThresholds};
}
