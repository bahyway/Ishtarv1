//! diagnosis-templates — ColorID quality/freshness/state diagnostics (§6.4).

pub mod diagnosis;

pub use diagnosis::{classify, diagnose, Diagnosis, DiagnosisKind};
