//! enkiqdb — EnkiDB Quarantine Archive (§12.3).
//!
//! Permanent append-only store for particles that fail validation in EnkiSDB
//! or are discovered by Musarû after extraction (late quarantine).
//!
//! Flow:
//!   EnkiSDB (ValidationSweep → Quarantined) ──→ QdbStore.accept()
//!   Musarû late-discovery ──────────────────→ QdbStore.accept()
//!
//! Nothing ever leaves EnkiQDB.  It is the terminal state for all
//! sovereignty violations, malware vectors, and validation failures.

pub mod qdb_store;

pub use qdb_store::{QdbStore, QuarantineRecord, QdbReport};

pub mod prelude {
    pub use super::qdb_store::{QdbStore, QuarantineRecord, QdbReport};
}
