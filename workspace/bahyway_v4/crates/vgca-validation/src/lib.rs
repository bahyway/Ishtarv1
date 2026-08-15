//! vgca-validation — template-based EAV validation beam (§10.3)
//! and VGCA geometric cleansing analysis (ADR-008, VGCA-Σ/Δ/Λ).

pub mod beam;
pub mod vgca;

pub use beam::{ValidationResult, validate, validate_with_types};
pub use vgca::{
    // VGCA-Σ
    FieldSignatureVector, GeometricFit, VgcaTextResult,
    vgca_score, geometric_fit,
    CLEAN_THRESHOLD, SUSPECT_THRESHOLD, SIGMA_MULTIPLIER,
    // VGCA-Δ
    BlockFeatureVector, VgcaBlockResult, vgca_delta, DELTA_FRAG,
    // VGCA-Λ
    ColumnGeometryDescriptor, InferredColumnType, infer_column_type,
    // CorruptionClass (ADR-008 §4.4)
    CorruptionClass,
};
