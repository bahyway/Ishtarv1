//! ProjectionAlgorithm — StoryEngine snapshot-accelerated projection (§3.6).
//!
//! Algorithm:
//!   1. Find the most recent SnapshotRecord at or before time T
//!   2. Use its snapshot_state as the starting projected state
//!   3. Apply all Event-Kakis between the snapshot timestamp and T
//!   4. Return the resulting projected state
//!
//! Cost: O(events since last snapshot) instead of O(total events).

use enkidb_kaki::IdentityKaki;
use enkidb_journal::Journal;

use crate::snapshot_record::SnapshotRecord;

/// Describes how the StoryEngine selects the starting point for projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionAlgorithm {
    /// Replay all events from the beginning (no snapshot available).
    FullReplay,
    /// Start from a snapshot, apply only the delta events.
    SnapshotAccelerated { snapshot_epoch: u64 },
}

/// Determine which projection algorithm to use for a given particle at time T.
pub fn choose_algorithm(
    particle:  &IdentityKaki,
    at_epoch:  u64,
    snapshots: &[SnapshotRecord],
) -> ProjectionAlgorithm {
    // Find the most recent snapshot at or before `at_epoch`.
    let best = snapshots
        .iter()
        .filter(|s| s.particle.bytes() == particle.bytes() && s.snapshot_date <= at_epoch)
        .max_by_key(|s| s.snapshot_date);

    match best {
        Some(s) => ProjectionAlgorithm::SnapshotAccelerated { snapshot_epoch: s.snapshot_date },
        None    => ProjectionAlgorithm::FullReplay,
    }
}

/// Count events that would need to be applied for the given projection window.
/// Used for performance estimation.
pub fn delta_event_count(
    particle:       &IdentityKaki,
    since_epoch:    u32,
    journal:        &Journal,
) -> usize {
    journal
        .read_particle_history(particle)
        .iter()
        .filter(|e| e.epoch >= since_epoch)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{KakiRole, mint::KakiMinter};
    use bahyway_core::TribeId;
    use enkidb_vector_id::VectorId;
    use crate::snapshot_record::SnapshotRecord;

    #[test]
    fn full_replay_when_no_snapshot() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let p = enkidb_kaki::IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let algo = choose_algorithm(&p, 1000, &[]);
        assert_eq!(algo, ProjectionAlgorithm::FullReplay);
    }

    #[test]
    fn accelerated_when_snapshot_exists() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let p = enkidb_kaki::IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let snap = SnapshotRecord::new(p.clone(), 500, vec![], VectorId::NEVER_SNAPSHOT);
        let algo = choose_algorithm(&p, 1000, &[snap]);
        assert_eq!(algo, ProjectionAlgorithm::SnapshotAccelerated { snapshot_epoch: 500 });
    }

    #[test]
    fn snapshot_after_query_time_ignored() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let p = enkidb_kaki::IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let snap = SnapshotRecord::new(p.clone(), 2000, vec![], VectorId::NEVER_SNAPSHOT);
        let algo = choose_algorithm(&p, 1000, &[snap]);
        assert_eq!(algo, ProjectionAlgorithm::FullReplay);
    }
}
