//! SnapshotSchedule — controls how often the Snapshot_Job runs for a tribe.
//!
//! Different particles change at different rates (§3.6):
//!   Civil registry: rarely → DailyOrEventCount(10000)
//!   Sensor stream: continuously → EventCount(100)
//!   Iraqi deployment: power-outage risk → HourlyMinimum

use std::time::Duration;

/// Snapshot frequency policy.  Set per Template, stored as Snapshot_Frequency EAV.
#[derive(Clone, Debug)]
pub enum SnapshotSchedule {
    /// Never snapshot — particles assigned the NeverSnapshot VectorId.
    /// Journal always replayed from the beginning.
    Never,
    /// Snapshot every N events (fixed event-count trigger).
    EventCount(u32),
    /// Snapshot at most once per time window.
    TimeBased(Duration),
    /// Snapshot on every state-transition event (for high-churn sensors).
    OnEveryTransition,
}

impl SnapshotSchedule {
    /// Iraqi deployment: minimum hourly (§11.6 — frequent power outages).
    pub fn iraqi_deployment() -> Self {
        SnapshotSchedule::TimeBased(Duration::from_secs(3600))
    }

    /// Civil registry: every 10,000 events or weekly.
    pub fn civil_registry() -> Self {
        SnapshotSchedule::EventCount(10_000)
    }

    /// Sensor streams: every 100 events.
    pub fn sensor_stream() -> Self {
        SnapshotSchedule::EventCount(100)
    }

    /// Should a snapshot be taken after `events_since_last` events and
    /// `elapsed` time since the last snapshot?
    pub fn should_snapshot(&self, events_since_last: u32, elapsed: Duration) -> bool {
        match self {
            SnapshotSchedule::Never => false,
            SnapshotSchedule::EventCount(n) => events_since_last >= *n,
            SnapshotSchedule::TimeBased(w) => elapsed >= *w,
            SnapshotSchedule::OnEveryTransition => events_since_last > 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_schedule_never_snapshots() {
        assert!(!SnapshotSchedule::Never.should_snapshot(9999, Duration::from_secs(99999)));
    }

    #[test]
    fn event_count_triggers_at_threshold() {
        let s = SnapshotSchedule::EventCount(100);
        assert!(!s.should_snapshot(99,  Duration::ZERO));
        assert!(s.should_snapshot(100, Duration::ZERO));
        assert!(s.should_snapshot(200, Duration::ZERO));
    }

    #[test]
    fn time_based_triggers_after_window() {
        let s = SnapshotSchedule::TimeBased(Duration::from_secs(3600));
        assert!(!s.should_snapshot(0, Duration::from_secs(3599)));
        assert!(s.should_snapshot(0, Duration::from_secs(3600)));
    }
}
