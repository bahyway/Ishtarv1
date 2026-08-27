//! enkidb-snapshot — Snapshot mechanism for StoryEngine projection acceleration (§3.6)
//!
//! Snapshots reduce StoryEngine projection cost from O(total events) to
//! O(events since last snapshot).  They are implemented as normal Event-Kakis
//! that update three of the seven mandatory universal EAV attributes:
//!   Snapshot_Date      — datetime of the most recent snapshot
//!   Snapshot_State     — full projected state at the snapshot moment
//!   Snapshot_Frequency — VectorId of the governing Snapshot_Job

pub mod projection;
pub mod snapshot_record;

pub use projection::ProjectionAlgorithm;
pub use snapshot_record::{
    SnapshotRecord, ATTR_SNAPSHOT_DATE, ATTR_SNAPSHOT_FREQ, ATTR_SNAPSHOT_STATE,
};

pub mod prelude {
    pub use super::projection::ProjectionAlgorithm;
    pub use super::snapshot_record::SnapshotRecord;
}
