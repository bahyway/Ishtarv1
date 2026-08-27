//! RecoveryProcedure — deterministic crash recovery (§11.3).
//!
//! The same recovery path is used for all shutdown types (SIGKILL, power loss,
//! clean shutdown) — BahyWay has only one recovery path, not two (§11.5).

use bahyway_core::Result;
use enkidb_journal::Journal;
use enkidb_snapshot::SnapshotRecord;

/// Steps of the recovery procedure in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStep {
    ValidatingSnapshot,
    ScanningJournalFrontier,
    ProjectingState,
    RebuildingIndexes,
    Complete,
}

/// Outcome of a completed recovery run.
#[derive(Debug)]
pub struct RecoveryOutcome {
    pub events_replayed: usize,
    pub indexes_rebuilt: usize,
    pub journal_frontier: u32,
    pub snapshot_used: Option<u64>,
}

/// Drives the deterministic recovery procedure.
pub struct RecoveryProcedure<'a> {
    journal: &'a Journal,
    snapshots: &'a [SnapshotRecord],
}

impl<'a> RecoveryProcedure<'a> {
    pub fn new(journal: &'a Journal, snapshots: &'a [SnapshotRecord]) -> Self {
        RecoveryProcedure { journal, snapshots }
    }

    /// Run the full deterministic recovery procedure.
    ///
    /// In production this reads persisted files; this implementation operates
    /// over the in-memory Journal for integration testing.
    pub fn run(&self) -> Result<RecoveryOutcome> {
        // Step 1: find the latest valid snapshot
        let latest_snapshot = self.snapshots.iter().max_by_key(|s| s.snapshot_date);

        let snapshot_epoch = latest_snapshot.map(|s| s.snapshot_date);

        // Step 2: determine journal frontier (all entries are valid in-memory)
        let journal_frontier = self.journal.entry_count() as u32;

        // Step 3: count events to replay (from snapshot epoch forward)
        let events_replayed = match snapshot_epoch {
            Some(_epoch) => {
                self.journal.entry_count() // simplified: in production, filter by epoch
            }
            None => self.journal.entry_count(),
        };

        // Step 4: indexes rebuilt (0 in this in-memory implementation)
        let indexes_rebuilt = 0;

        Ok(RecoveryOutcome {
            events_replayed,
            indexes_rebuilt,
            journal_frontier,
            snapshot_used: snapshot_epoch,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_kaki::{mint::KakiMinter, EventKaki, IdentityKaki, KakiRole};

    #[test]
    fn recovery_with_empty_journal() {
        let jnl = Journal::new(64);
        let snaps = vec![];
        let proc = RecoveryProcedure::new(&jnl, &snaps);
        let out = proc.run().unwrap();
        assert_eq!(out.events_replayed, 0);
        assert!(out.snapshot_used.is_none());
    }

    #[test]
    fn recovery_with_events() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let tgt = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        for i in 0..10u32 {
            let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
            let eav = vec![EavTriple::new(0x0001, b"GOLDEN".to_vec())];
            jnl.append(JournalEntry::new(ek, tgt.clone(), i, eav))
                .unwrap();
        }
        let proc = RecoveryProcedure::new(&jnl, &[]);
        let out = proc.run().unwrap();
        assert_eq!(out.events_replayed, 10);
        assert_eq!(out.journal_frontier, 10);
    }
}
