//! enkidb-ingest — CSV → Particle → EnkiDb ingestion pipeline.
//!
//! No external dependencies. CSV parsing is hand-written; value serialisation
//! uses the sovereign AkkValue codec.
//!
//! Pipeline:
//!   `CsvIngester::from_str(csv_text, tribe_id)`
//!   → `ingest_into(db)` → `IngestReport { rows, particles_written }`
//!   → `db` now contains all particles, queryable via HeptaScript.

#![forbid(unsafe_code)]

pub mod bridge;
pub mod csv;
pub mod ingest;

pub use ingest::{CsvIngester, IngestReport, IngestError};
