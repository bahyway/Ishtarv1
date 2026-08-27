//! vgca-validation — template-based EAV validation beam (§10.3)
//! and VGCA geometric cleansing analysis (ADR-008, VGCA-Σ/Δ/Λ).

pub mod beam;
pub mod vgca;

pub use beam::{validate, validate_with_types, ValidationResult};
pub use vgca::{
    geometric_fit,
    infer_column_type,
    vgca_delta,
    vgca_score,
    // VGCA-Δ
    BlockFeatureVector,
    // VGCA-Λ
    ColumnGeometryDescriptor,
    // CorruptionClass (ADR-008 §4.4)
    CorruptionClass,
    // VGCA-Σ
    FieldSignatureVector,
    GeometricFit,
    InferredColumnType,
    VgcaBlockResult,
    VgcaTextResult,
    CLEAN_THRESHOLD,
    DELTA_FRAG,
    SIGMA_MULTIPLIER,
    SUSPECT_THRESHOLD,
};
