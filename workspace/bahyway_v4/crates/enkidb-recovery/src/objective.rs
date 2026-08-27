//! Recovery objectives (RPO/RTO) per §11.4.

use std::time::Duration;

/// Recovery Point Objective — maximum data loss acceptable.
#[derive(Debug, Clone)]
pub enum RecoveryPointObjective {
    /// At most one Event-Kaki uncommitted (fsync-per-commit policy).
    OneEventKaki,
    /// At most one fsync batch window of events (batched fsync policy).
    FsyncBatch { window: Duration },
}

/// Recovery Time Objective — maximum acceptable restart time.
#[derive(Debug, Clone)]
pub struct RecoveryObjective {
    pub rpo: RecoveryPointObjective,
    /// Target maximum RTO; actual time depends on journal size since last snapshot.
    pub rto_target: Duration,
}

impl RecoveryObjective {
    /// Iraqi deployment profile (§11.6): fsync-per-commit, < 30 seconds RTO.
    pub fn iraqi_deployment() -> Self {
        RecoveryObjective {
            rpo: RecoveryPointObjective::OneEventKaki,
            rto_target: Duration::from_secs(30),
        }
    }

    /// Cloud/data-center profile: batched durability, < 5 seconds RTO.
    pub fn datacenter() -> Self {
        RecoveryObjective {
            rpo: RecoveryPointObjective::FsyncBatch {
                window: Duration::from_millis(10),
            },
            rto_target: Duration::from_secs(5),
        }
    }
}
