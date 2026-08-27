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

pub use check::{check_sovereignty, SecurityResult};
pub use emergency::{emergency_scribe, EmergencyLevel, EmergencyTablet};
pub use failover::{
    check_failover, cluster_failover_state, FailoverState, QUANTUM_FREEZE_CRITICAL_THRESHOLD,
    QUANTUM_FREEZE_WARNING_THRESHOLD,
};
pub use late_quarantine::{emit_late_quarantine, AffectedParticle, LateQuarantineReport};
pub use spectral_defender::{detect_gap, GapDetectionResult, GAP_SIGMA_THRESHOLD};
pub use vocab_gate::{
    scan_dir as vocab_scan_dir, scan_file as vocab_scan_file, scan_text as vocab_scan_text,
    Violation as VocabViolation, VocabScanReport, FORBIDDEN_PORTS, FORBIDDEN_TERMS,
    GATED_EXTENSIONS,
};
pub use way_kernel::{
    check_radius_and_trust,
    is_orbit_unstable,
    orbit_entropy_from_counts,
    orbit_entropy_from_lanes,
    // Metric functions
    trust_at,
    Action,
    AlwaysContinuousValidator,
    AlwaysDiscontinuousValidator,
    Capability,
    // Sorts
    CapabilityId,
    PropagationError,
    PropagationPath,
    Scope,
    SealState,
    // Errors
    SecurityError,
    TrustState,
    ValidationResult as WayValidationResult,
    // Validator trait + fixtures
    Validator,
    // WAY STIR pipeline
    WayPipelineStage,
    // Metric constants
    ALPHA_CROSS,
    DEFAULT_DECAY,
    EPSILON_UNSTABLE,
    PI_THROTTLE,
    TAU_MIN,
    TAU_RADIUS,
};
pub use zip_scan::{scan as zip_scan, scan_all as zip_scan_all, ZipScanResult, MALWARE_SIGS};
