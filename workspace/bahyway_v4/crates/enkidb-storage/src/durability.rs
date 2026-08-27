//! FsyncPolicy — hardware durability barriers per §11.2.
//!
//! Iraqi deployment recommendation: FsyncPerCommit.
//! High-throughput default: Batched(10ms).

use std::time::Duration;

/// Controls when fsync() is called after writing an Event-Kaki.
#[derive(Clone, Debug)]
pub enum FsyncPolicy {
    /// fsync after every committed write — maximum durability.
    /// Recommended for Iraqi deployment (frequent power outages, §11.6).
    PerCommit,
    /// Batch fsync within the given time window — trades bounded data loss
    /// for higher write throughput.  Default window: 10ms.
    Batched { window: Duration },
    /// Never fsync — test/development only. NOT for production.
    Never,
}

impl Default for FsyncPolicy {
    fn default() -> Self {
        FsyncPolicy::Batched {
            window: Duration::from_millis(10),
        }
    }
}

impl FsyncPolicy {
    /// Iraqi-deployment hardened setting per §11.6.
    pub fn iraqi_deployment() -> Self {
        FsyncPolicy::PerCommit
    }
}
