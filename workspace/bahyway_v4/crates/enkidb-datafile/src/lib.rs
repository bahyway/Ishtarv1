//! enkidb-datafile — ADR-012 Read Node storage.
//!
//! The Read Node holds NO journal (per the Data Files Law). Instead,
//! particles materialize into a Data File (`{base}.data`, append-only)
//! with a sorted on-disk Index (`{base}.idx`) that a point lookup
//! binary-searches directly via `seek`/`read_exact` — never loading the
//! index or the data into memory, never replaying history on open.
//!
//! - `DataFileWriter` appends particles and stages index entries cheaply
//!   (O(1) per append); `compact_index()` merge-sorts staging into the
//!   final sorted `.idx` atomically (write `.idx.new`, rename over `.idx`).
//! - `DataFileReader::get()` is O(log n) disk reads for a point lookup,
//!   proved by `get_traced()`'s probe count in this crate's own tests —
//!   not O(n) full scan, not O(total history) journal replay.
//!
//! No `unsafe`, no external dependencies, no memmap2 crate: plain
//! `std::fs::File` + `Seek` + `Read` against fixed-size (28-byte) index
//! records is enough to binary-search a file far larger than available
//! RAM without ever loading it wholesale.
#![forbid(unsafe_code)]

pub mod index;
pub mod reader;
pub mod record;
pub mod writer;

pub use index::{IndexRecord, INDEX_RECORD_SIZE};
pub use reader::DataFileReader;
pub use record::{decode_particle, encode_particle, RecordError};
pub use writer::DataFileWriter;
