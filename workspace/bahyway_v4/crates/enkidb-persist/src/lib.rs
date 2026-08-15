//! enkidb-persist — disk persistence layer for EnkiDB v4 (§11).
//!
//! Provides:
//!   - `disk_journal`: binary serialization and crash-safe deserialization
//!   - `PersistedDb`:  EnkiDb wrapped with an append-only disk journal

pub mod disk_journal;
pub mod persisted_db;

pub use persisted_db::{PersistedDb, PersistStats};
