//! enkisdb — EnkiDB Stage Database (§12.1).
//!
//! Particles enter through the LandingZone (ZIP, CSV, TSV).  Each batch is
//! scanned by Musarû before extraction.  Staged particles wait in SdbStore
//! until the ValidationSweep runs (every 900 ticks by default).
//!
//! Flow:
//!   LandingZone → SdbPipeline → SdbStore (Pending)
//!                   ↓ (every 900 ticks)
//!              ValidationSweep
//!                   ├── pass → SdbStore (Promoted) → EnkiODB
//!                   └── fail → SdbStore (Quarantined) → EnkiQDB

pub mod sdb_store;
pub mod sdb_pipeline;
pub mod validation_sweep;

pub use sdb_store::{SdbStore, StagedParticle, SdbStatus, SdbStats};
pub use sdb_pipeline::{SdbPipeline, SdbPipelineStats, SdbAlert, SdbAlertSeverity};
pub use validation_sweep::{ValidationSweep, SweepResult, DEFAULT_SWEEP_INTERVAL_TICKS};

pub mod prelude {
    pub use super::sdb_store::{SdbStore, StagedParticle, SdbStatus};
    pub use super::sdb_pipeline::SdbPipeline;
    pub use super::validation_sweep::{ValidationSweep, DEFAULT_SWEEP_INTERVAL_TICKS};
}
