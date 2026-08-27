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

pub mod sdb_pipeline;
pub mod sdb_store;
pub mod validation_sweep;

pub use sdb_pipeline::{SdbAlert, SdbAlertSeverity, SdbPipeline, SdbPipelineStats};
pub use sdb_store::{SdbStats, SdbStatus, SdbStore, StagedParticle};
pub use validation_sweep::{SweepResult, ValidationSweep, DEFAULT_SWEEP_INTERVAL_TICKS};

pub mod prelude {
    pub use super::sdb_pipeline::SdbPipeline;
    pub use super::sdb_store::{SdbStatus, SdbStore, StagedParticle};
    pub use super::validation_sweep::{ValidationSweep, DEFAULT_SWEEP_INTERVAL_TICKS};
}
