//! snapshot-job — Snapshot_Job operational mechanism (§3.6, §5.5)
//!
//! The Snapshot_Job is a Vector-ID-identified mechanism (NOT a particle).
//! It runs from EriduOS or EnkiDB on a configured schedule.
//! For each particle it manages, it:
//!   1. Computes current projected state via the StoryEngine
//!   2. Appends a new Event-Kaki to the particle's Journal updating
//!      Snapshot_Date, Snapshot_State, and (optionally) Snapshot_Frequency
//!
//! The snapshot Event-Kaki obeys all normal rules — append-only, immutable
//! once committed.  There is no special "snapshot event type" — snapshots
//! are normal Event-Kakis (kaki_type = 0x02) that happen to update the
//! three snapshot attributes.

pub mod job;
pub mod schedule;

pub use job::{SnapshotJob, SnapshotJobResult};
pub use schedule::SnapshotSchedule;

pub mod prelude {
    pub use super::job::{SnapshotJob, SnapshotJobResult};
    pub use super::schedule::SnapshotSchedule;
}
