//! musaru-security — sovereignty and role validation (§10.2, PA-15).
//!
//! Extended in v4.0.1 with emergency_scribe() — KAKI-stamped failure tablets.
//! Extended in v4.0.2 with zip_scan (pre-extraction) and late_quarantine
//! (post-extraction) completing the two Musarû discovery paths.

pub mod check;
pub mod emergency;
pub mod failover;
pub mod late_quarantine;
pub mod spectral_defender;
pub mod vocab_gate;
pub mod way_kernel;
pub mod zip_scan;

pub use check::{SecurityResult, check_sovereignty};
pub use emergency::{EmergencyLevel, EmergencyTablet, emergency_scribe};
pub use vocab_gate::{
    Violation as VocabViolation, VocabScanReport,
    scan_text as vocab_scan_text, scan_file as vocab_scan_file, scan_dir as vocab_scan_dir,
    FORBIDDEN_TERMS, FORBIDDEN_PORTS, GATED_EXTENSIONS,
};
pub use zip_scan::{ZipScanResult, scan as zip_scan, scan_all as zip_scan_all, MALWARE_SIGS};
pub use late_quarantine::{AffectedParticle, LateQuarantineReport, emit_late_quarantine};
pub use spectral_defender::{GapDetectionResult, detect_gap, GAP_SIGMA_THRESHOLD};
pub use failover::{
    FailoverState, check_failover, cluster_failover_state,
    QUANTUM_FREEZE_WARNING_THRESHOLD, QUANTUM_FREEZE_CRITICAL_THRESHOLD,
};
pub use way_kernel::{
    // Sorts
    CapabilityId, Capability, Action, Scope,
    PropagationPath, ValidationResult as WayValidationResult,
    TrustState, SealState,
    // Errors
    SecurityError, PropagationError,
    // Metric constants
    ALPHA_CROSS, TAU_RADIUS, EPSILON_UNSTABLE, PI_THROTTLE, TAU_MIN, DEFAULT_DECAY,
    // Metric functions
    trust_at, orbit_entropy_from_counts, orbit_entropy_from_lanes, is_orbit_unstable,
    check_radius_and_trust,
    // Validator trait + fixtures
    Validator, AlwaysContinuousValidator, AlwaysDiscontinuousValidator,
    // WAY STIR pipeline
    WayPipelineStage,
};
