//! enkiodb — EnkiDB Operational Database (§12.2).
//!
//! Active validated particles live here after passing the SDB ValidationSweep
//! or being committed via the EnkiDB GUI transaction path.
//!
//! Flow (corrected 2026-07-31 — QDB never feeds EnkiDW directly; only
//! EnkiDB's own snapshot/partition job feeds EnkiDW):
//!   EnkiSDB (Promoted)        → OdbStore.drain_from_sdb()             ──→ OdbStore (Active)
//!   EnkiDB  (GUI commit)      → OdbStore.ingest_from_gui()            ──→ OdbStore (Active)
//!   Data Steward (QDB review) → OdbStore.ingest_from_steward_promotion() ──→ OdbStore (Active)
//!                                                                     ↓
//!                                         OdbStore.promote_to_golden() ──→ EnkiDb (Golden Records)
//!                                                                            │
//!                                                                            └─→ snapshot/partition → EnkiDW
//!
//! `OdbStore.retire()` still exists (Active → Retired, `ArchiveMove`
//! journal marker) but has zero real callers anywhere in the workspace
//! — a pre-existing, separately-flagged loose end, not touched by this
//! correction.

pub mod odb_store;

pub use odb_store::{OdbParticle, OdbSource, OdbStats, OdbStatus, OdbStore};

pub mod prelude {
    pub use super::odb_store::{OdbParticle, OdbSource, OdbStatus, OdbStore};
}
