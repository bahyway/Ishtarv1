// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  wpd-engine · src/lib.rs
//  𒀭𒍪 Sovereign water/pipeline defect detection engine
//  BaghdadSector×7 heptagram, 12-band spectral (VNIR+SWIR+TIR),
//  KAKI-keyed defect events. Junction defect potential (Enbilulu
//  Calculus) is consumed from geo-engine, the shared math truth
//  source — see junction.rs.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub mod defect;
pub mod domain;
pub mod error;
pub mod junction;
pub mod kaki;
pub mod nav;
pub mod priority;
pub mod sector;
pub mod seed;
pub mod segment;
pub mod spectral;

// ── Flat re-exports ───────────────────────────────────────────────
pub use defect::{DefectClass, DefectClassifier, DefectEvent, DetectionMethod};
pub use domain::{DefectSeverity, PipeMaterial, PipelineType};
pub use error::{WpdError, WpdResult};
pub use junction::{assess_junction, JunctionAssessment};
pub use kaki::{derive_wpd_kaki, WPDWAY_DOMAIN};
pub use nav::{haversine_m, RepairNavigator, RepairPrefs, RepairRoute, RepairWaypoint, SiteAccess};
pub use priority::{RepairJob, RepairPriority, RepairScheduler};
pub use sector::BaghdadSector;
pub use seed::{seed_registry, sovereign_segments};
pub use segment::{PipelineRegistry, PipelineSegment, SegmentStatus};
pub use spectral::{
    oil_leak_signature, sewage_blockage_signature, water_leak_signature, SpectralBand,
    SpectralScan, WPD_SPECTRAL_BANDS,
};
