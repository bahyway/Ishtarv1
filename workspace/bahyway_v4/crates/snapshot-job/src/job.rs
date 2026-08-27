//! SnapshotJob — executes snapshot captures for a list of particles.
//!
//! Identified by a VectorId in Default.Jobs — NOT a particle (§5.5).
//! Each run projects current state via StoryEngine and appends a snapshot
//! Event-Kaki to the particle's Journal (§3.6).

use std::time::{SystemTime, UNIX_EPOCH};

use bahyway_core::Result;
use enkidb_journal::{
    entry::{EavTriple, JournalEntry},
    Journal,
};
use enkidb_kaki::{mint::KakiMinter, IdentityKaki, KakiRole};
use enkidb_snapshot::{
    snapshot_record::{ATTR_SNAPSHOT_DATE, ATTR_SNAPSHOT_STATE},
    SnapshotRecord,
};
use enkidb_vector_id::VectorId;
use story_engine::StoryEngine;

pub struct SnapshotJobResult {
    pub snapshots_taken: usize,
    pub particles_skipped: usize,
}

/// The Snapshot_Job operational mechanism.
pub struct SnapshotJob {
    /// This job's own VectorId in Default.Jobs.
    pub vector_id: VectorId,
    /// The tribe whose particles this job manages.
    tribe_minter: KakiMinter,
}

impl SnapshotJob {
    pub fn new(vector_id: VectorId, minter: KakiMinter) -> Self {
        SnapshotJob {
            vector_id,
            tribe_minter: minter,
        }
    }

    /// Run one pass: snapshot all particles that have events since their last snapshot.
    pub fn run(
        &self,
        particles: &[IdentityKaki],
        journal: &mut Journal,
        snapshots: &mut Vec<SnapshotRecord>,
    ) -> Result<SnapshotJobResult> {
        let mut taken = 0;
        let mut skipped = 0;
        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for particle in particles {
            let engine = StoryEngine::new(journal, snapshots);
            let history_len = engine.event_count(particle);

            // Find the events since the last snapshot
            let last_snap_epoch = snapshots
                .iter()
                .filter(|s| s.particle.bytes() == particle.bytes())
                .map(|s| s.snapshot_date)
                .max()
                .unwrap_or(0);

            let events_since: usize = journal
                .read_particle_history(particle)
                .iter()
                .filter(|e| e.epoch as u64 > last_snap_epoch)
                .count();

            if events_since == 0 {
                skipped += 1;
                continue;
            }

            // Project current state
            let projected = engine.project(particle);

            // Encode state as bytes for Snapshot_State
            let state_bytes = story_engine::projection::encode_state(projected.state).to_vec();

            // Append a snapshot Event-Kaki to the Journal
            let snap_ek =
                enkidb_kaki::EventKaki::try_from_kaki(self.tribe_minter.event(KakiRole::Zikru))
                    .unwrap();
            let eav = vec![
                EavTriple::new(ATTR_SNAPSHOT_DATE, now_epoch.to_be_bytes().to_vec()),
                EavTriple::new(ATTR_SNAPSHOT_STATE, state_bytes.clone()),
            ];
            let epoch = (now_epoch & 0xFFFF_FFFF) as u32;
            journal.append(JournalEntry::new(snap_ek, *particle, epoch, eav))?;

            // Record the SnapshotRecord for the index
            snapshots.push(SnapshotRecord::new(
                *particle,
                now_epoch,
                state_bytes,
                self.vector_id,
            ));

            taken += 1;
            let _ = history_len; // informational
        }

        Ok(SnapshotJobResult {
            snapshots_taken: taken,
            particles_skipped: skipped,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::{ParticleState, TribeId};
    use enkidb_journal::entry::{EavTriple, JournalEntry};
    use enkidb_kaki::{mint::KakiMinter, EventKaki, KakiRole};
    use story_engine::projection::{encode_state, ATTR_STATE};

    fn make_particle_with_events(
        m: &KakiMinter,
        jnl: &mut Journal,
        state: ParticleState,
    ) -> IdentityKaki {
        let target = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav = vec![EavTriple::new(ATTR_STATE, encode_state(state).to_vec())];
        jnl.append(JournalEntry::new(ek, target.clone(), 1, eav))
            .unwrap();
        target
    }

    #[test]
    fn snapshot_job_captures_state() {
        let tribe_id = TribeId::from_u16(0x0001);
        let m = KakiMinter::new(tribe_id);
        let mut jnl = Journal::new(64);
        let mut snaps: Vec<SnapshotRecord> = vec![];

        let p = make_particle_with_events(&m, &mut jnl, ParticleState::Golden);
        let job = SnapshotJob::new(VectorId::generate(), KakiMinter::new(tribe_id));
        let res = job.run(&[p.clone()], &mut jnl, &mut snaps).unwrap();

        assert_eq!(res.snapshots_taken, 1);
        assert_eq!(snaps.len(), 1);
    }

    #[test]
    fn no_snapshot_if_no_new_events() {
        let tribe_id = TribeId::from_u16(0x0002);
        let m = KakiMinter::new(tribe_id);
        let mut jnl = Journal::new(64);

        let p = make_particle_with_events(&m, &mut jnl, ParticleState::Golden);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Pre-seed a snapshot so it looks like the particle is already captured
        let mut snaps = vec![SnapshotRecord::new(
            p.clone(),
            now,
            b"GOLDEN".to_vec(),
            VectorId::NEVER_SNAPSHOT,
        )];

        let job = SnapshotJob::new(VectorId::generate(), KakiMinter::new(tribe_id));
        let res = job.run(&[p], &mut jnl, &mut snaps).unwrap();

        assert_eq!(res.snapshots_taken, 0);
        assert_eq!(res.particles_skipped, 1);
    }
}
