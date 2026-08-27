//! enkidb-ingest — CSV → Particle → EnkiDb ingestion pipeline.
//!
//! No external dependencies. CSV parsing is hand-written; value serialisation
//! uses the sovereign AkkValue codec.
//!
//! Pipeline:
//!   `CsvIngester::from_str(csv_text, tribe_id)`
//!   → `ingest_into(db)` → `IngestReport { rows, particles_written }`
//!   → `db` now contains all particles, queryable via HeptaScript.
//!
//! Each row's ledger write and NĀRU audit record commit through
//! [`kispu::commit`] — all-or-nothing, see that module for why.

#![forbid(unsafe_code)]

pub mod bridge;
pub mod csv;
pub mod ingest;
pub mod kispu;

pub use ingest::{CsvIngester, IngestError, IngestReport};
pub use kispu::KispuError;
