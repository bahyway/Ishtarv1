#![forbid(unsafe_code)]
//! # NARAMSIN Stage 1 — Sovereign Format Parsing
//!
//! Parses extracted files (CSV, JSON, XML) into a row-oriented intermediate
//! representation. Schema-level malformation (broken delimiters, invalid JSON,
//! unclosed XML tags) is classified here and rejected with NRM_MALFORMED
//! (EN-NARAMSIN-001 §3, Stage 1).
//!
//! ## Supported Formats (Stage 1)
//! - CSV — the primary format for NANSHE/ESARHADDON/ASHNAN sensor exports
//! - JSON — external bulletin and API ingestion
//! - XML — legacy data-body formats and FAO structured feeds
//!
//! ## Explicit Non-Goals (until a sovereign module has a justified requirement)
//! - NetCDF, Parquet, Shapefile, Excel binary

pub mod row;
pub mod csv;
pub mod json;
pub mod xml;
pub mod error;

pub use error::FormatError;
pub use row::{Row, Field};

/// File format variants supported by NARAMSIN Stage 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// Comma-separated values.
    Csv,
    /// JSON document.
    Json,
    /// XML document.
    Xml,
}

impl FileFormat {
    /// Human-readable name for audit logging.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Csv  => "csv",
            Self::Json => "json",
            Self::Xml  => "xml",
        }
    }
}
