//! # import — Smart Schema Auto-Detection & Adapter Router
//! **PollutionEngine v4.0 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! **Ze kiest EAV schema** — the system *chooses* the right EAV schema
//! by analysing the incoming payload's field signatures, source hints,
//! and value-range heuristics. No manual domain tagging required.
//!
//! ## Pipeline
//! ```text
//!  Raw Payload (CSV / JSON / API stream)
//!       │
//!       ▼
//!  ImportRouter          ← detects format + source origin
//!       │
//!       ▼
//!  SchemaDetector        ← scores domain confidence (AIR/WATER/OIL)
//!       │
//!       ├─ confidence >= threshold  →  AdapterRegistry::select()
//!       │                                      │
//!       │                                      ▼
//!       │                               SourceAdapter::normalize()
//!       │                                      │
//!       │                                      ▼
//!       │                               Vec<RawSensorReading>
//!       │
//!       └─ confidence < threshold   →  ImportError::Ambiguous
//! ```

pub mod adapters;
pub mod detector;
pub mod registry;
pub mod router;

pub use detector::{DetectionResult, DomainConfidence, SchemaDetector};
pub use registry::{AdapterRegistry, SourceAdapter};
pub use router::{ImportPayload, ImportRouter, PayloadFormat, SourceHint};

use crate::domain::PollutionError;
use crate::sensors::RawSensorReading;

// ─────────────────────────────────────────────────────────────
//  IMPORT ERROR  (sovereign — no thiserror)
// ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ImportError {
    Ambiguous {
        top: String,
        confidence: f32,
        threshold: f32,
        candidates: String,
    },
    UnsupportedFormat(String),
    NoAdapter(String),
    FieldMappingFailed {
        field: String,
        reason: String,
    },
    CsvParseError {
        row: usize,
        reason: String,
    },
    /// Sovereign JSON parse error (replaces serde_json::Error)
    JsonParseError(String),
    UnitConversionFailed {
        field: String,
        reason: String,
    },
    EmptyPayload,
    DomainError(PollutionError),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ambiguous {
                top,
                confidence,
                threshold,
                candidates,
            } => write!(
                f,
                "Schema ambiguous: top domain '{top}' confidence {confidence:.2} \
                     < threshold {threshold:.2}. Candidates: {candidates}"
            ),
            Self::UnsupportedFormat(fmt) => write!(f, "Unsupported payload format: {fmt}"),
            Self::NoAdapter(s) => write!(f, "No adapter registered for source '{s}'"),
            Self::FieldMappingFailed { field, reason } => write!(
                f,
                "Field mapping failed — field '{field}' not found in payload: {reason}"
            ),
            Self::CsvParseError { row, reason } => {
                write!(f, "CSV parse error at row {row}: {reason}")
            }
            Self::JsonParseError(msg) => write!(f, "JSON parse error: {msg}"),
            Self::UnitConversionFailed { field, reason } => {
                write!(f, "Unit conversion failed for field '{field}': {reason}")
            }
            Self::EmptyPayload => write!(f, "Empty payload — no rows to process"),
            Self::DomainError(e) => write!(f, "Pollution domain error: {e}"),
        }
    }
}

impl std::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DomainError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<PollutionError> for ImportError {
    fn from(e: PollutionError) -> Self {
        Self::DomainError(e)
    }
}

// ─────────────────────────────────────────────────────────────
//  IMPORT RESULT
// ─────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ImportResult {
    pub readings: Vec<RawSensorReading>,
    pub row_errors: Vec<(usize, ImportError)>,
    pub detection: DetectionResult,
    pub adapter_used: String,
    pub source_hint: String,
}

impl ImportResult {
    pub fn success_count(&self) -> usize {
        self.readings.len()
    }
    pub fn error_count(&self) -> usize {
        self.row_errors.len()
    }
    pub fn total_rows(&self) -> usize {
        self.readings.len() + self.row_errors.len()
    }
    pub fn is_clean(&self) -> bool {
        self.row_errors.is_empty()
    }

    pub fn akk_summary(&self) -> String {
        format!(
            "IMPORT source={} adapter={} domain={:?} confidence={:.2} ok={} err={}",
            self.source_hint,
            self.adapter_used,
            self.detection.top_domain(),
            self.detection.top_confidence(),
            self.success_count(),
            self.error_count(),
        )
    }
}

// ─────────────────────────────────────────────────────────────
//  TOP-LEVEL ENTRY POINT
// ─────────────────────────────────────────────────────────────

/// One-call smart import.
/// Detects format → chooses schema → normalizes → returns readings.
pub fn smart_import(
    raw: &str,
    source_url: &str,
    confidence_threshold: f32,
) -> Result<ImportResult, ImportError> {
    // 1. Detect format and build payload
    let router = ImportRouter;
    let payload = router.parse(raw, source_url)?;

    // 2. Score domain confidence
    let detector = SchemaDetector;
    let detection = detector.detect(&payload);

    // 3. Reject if ambiguous — sovereign GATE 5: bypass for known (non-Unknown) sources
    let is_known_source = !matches!(payload.source, SourceHint::Unknown(_));
    if !is_known_source && detection.top_confidence() < confidence_threshold {
        return Err(ImportError::Ambiguous {
            top: format!("{:?}", detection.top_domain()),
            confidence: detection.top_confidence(),
            threshold: confidence_threshold,
            candidates: detection.scores_summary(),
        });
    }

    // 4. Select adapter from registry
    let registry = AdapterRegistry::default();
    let adapter_name = registry.select(&detection, &payload)?;
    let adapter = registry
        .get(&adapter_name)
        .ok_or_else(|| ImportError::NoAdapter(adapter_name.clone()))?;

    // 5. Normalize rows
    let (readings, row_errors) = adapter.normalize(&payload);

    Ok(ImportResult {
        readings,
        row_errors,
        detection,
        adapter_used: adapter_name,
        source_hint: source_url.to_string(),
    })
}
