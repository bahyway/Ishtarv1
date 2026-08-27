//! PersistedDb — EnkiDb wrapped with a crash-safe disk journal (§11).
//!
//! Data layout on disk:
//!   {data_dir}/tribe-{tribe_id:04x}/journal.bin
//!
//! On open:  load existing journal entries → replay into in-memory EnkiDb.
//! On commit: serialize entry → append_committed (disk-first) → apply to db.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bahyway_core::{Result, TribeId};
use enkidb_engine::EnkiDb;
use enkidb_journal::entry::{EavTriple, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki};
use enkidb_storage::{AppendWriter, FsyncPolicy};

use crate::disk_journal::{load_entries, serialize_entry};

/// Statistics for the persistence layer.
#[derive(Debug, Default, Clone)]
pub struct PersistStats {
    /// Entries loaded from disk on open (crash-recovery replay).
    pub entries_replayed: usize,
    /// Entries written to disk since this instance was opened.
    pub entries_written: usize,
}

/// EnkiDb with a crash-safe append-only disk journal.
pub struct PersistedDb {
    db: EnkiDb,
    writer: AppendWriter,
    stats: PersistStats,
}

impl PersistedDb {
    /// Open (or create) a persisted database for one tribe.
    ///
    /// Creates `{data_dir}/tribe-{tribe_id:04x}/` and `journal.bin` if they
    /// don't exist, then replays any committed entries into the in-memory db.
    pub fn open(data_dir: &Path, tribe_id: TribeId, policy: FsyncPolicy) -> Result<Self> {
        let tribe_dir = tribe_dir_path(data_dir, tribe_id);
        fs::create_dir_all(&tribe_dir)?;

        let journal_path = tribe_dir.join("journal.bin");
        let mut db = EnkiDb::new(tribe_id);
        let mut stats = PersistStats::default();

        // Replay committed entries that already exist on disk.
        if journal_path.exists() {
            let entries = load_entries(&journal_path)?;
            stats.entries_replayed = entries.len();
            let mut registered: BTreeSet<u32> = BTreeSet::new();
            for entry in entries {
                let hash = entry.target_kaki.uuid_hash();
                if registered.insert(hash) {
                    db.register_particle(&entry.target_kaki)?;
                }
                db.append_event(entry.event_kaki, entry.target_kaki, entry.epoch, entry.eav)?;
            }
        }

        let writer = AppendWriter::open(&journal_path, policy)?;
        Ok(PersistedDb { db, writer, stats })
    }

    /// Write an event to disk and then apply it to the in-memory database.
    ///
    /// The disk write happens first — if the process crashes after the write
    /// but before the in-memory update, the entry is replayed on next open.
    pub fn commit(
        &mut self,
        event_kaki: EventKaki,
        target: IdentityKaki,
        epoch: u32,
        eav: Vec<EavTriple>,
    ) -> Result<()> {
        let entry = JournalEntry::new(event_kaki, target, epoch, eav.clone());
        let bytes = serialize_entry(&entry);

        // Disk-first — crash-safe
        self.writer.append_committed(&bytes)?;
        self.stats.entries_written += 1;

        // Now apply to in-memory state
        self.db.append_event(event_kaki, target, epoch, eav)
    }

    /// Register a new particle and record it durably.
    ///
    /// Unlike `commit`, registration has no journal entry — it only updates
    /// the in-memory indexes.  Call this before the first `commit` for a new
    /// identity so that StoryEngine projection works immediately.
    pub fn register_particle(&mut self, particle: &IdentityKaki) -> Result<()> {
        self.db.register_particle(particle)
    }

    /// Force fsync regardless of the current `FsyncPolicy`.
    pub fn sync(&self) -> std::io::Result<()> {
        self.writer.sync_now()
    }

    pub fn db(&self) -> &EnkiDb {
        &self.db
    }
    pub fn db_mut(&mut self) -> &mut EnkiDb {
        &mut self.db
    }
    pub fn stats(&self) -> &PersistStats {
        &self.stats
    }
    pub fn tribe_id(&self) -> TribeId {
        self.db.tribe_id()
    }
}

/// Returns `{data_dir}/tribe-{tribe_id:04x}`.
fn tribe_dir_path(data_dir: &Path, tribe_id: TribeId) -> PathBuf {
    data_dir.join(format!("tribe-{:04x}", tribe_id.as_u16()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::ParticleState;
    use bahyway_core::TribeId;
    use enkidb_kaki::{KakiMinter, KakiRole};
    use story_engine::projection::{encode_state, ATTR_STATE};

    fn tmp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("enkidb_persist_{}", name));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn cleanup(p: &Path) {
        let _ = fs::remove_dir_all(p);
    }

    #[test]
    fn open_empty_dir_creates_journal() {
        let dir = tmp_dir("open_empty");
        let tid = TribeId::from_u16(0x0001);
        let pdb = PersistedDb::open(&dir, tid, FsyncPolicy::Never).unwrap();
        assert_eq!(pdb.stats().entries_replayed, 0);
        assert_eq!(pdb.stats().entries_written, 0);
        let jpath = dir.join("tribe-0001").join("journal.bin");
        assert!(jpath.exists());
        cleanup(&dir);
    }

    #[test]
    fn commit_survives_reopen() {
        let dir = tmp_dir("reopen");
        let tid = TribeId::from_u16(0x0002);
        let m = KakiMinter::new(tid);

        let particle = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();

        {
            let mut pdb = PersistedDb::open(&dir, tid, FsyncPolicy::Never).unwrap();
            pdb.register_particle(&particle).unwrap();
            pdb.commit(
                ek,
                particle.clone(),
                1,
                vec![EavTriple::new(
                    ATTR_STATE,
                    encode_state(ParticleState::Golden).to_vec(),
                )],
            )
            .unwrap();
            assert_eq!(pdb.stats().entries_written, 1);
        } // pdb dropped — AppendWriter flushed

        // Re-open: entry must be replayed
        let pdb2 = PersistedDb::open(&dir, tid, FsyncPolicy::Never).unwrap();
        assert_eq!(pdb2.stats().entries_replayed, 1);
        let ps = pdb2.db().project(&particle);
        assert_eq!(ps.state, ParticleState::Golden);
        cleanup(&dir);
    }

    #[test]
    fn multiple_commits_replayed_in_order() {
        let dir = tmp_dir("multi");
        let tid = TribeId::from_u16(0x0003);
        let m = KakiMinter::new(tid);

        let particle = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();

        {
            let mut pdb = PersistedDb::open(&dir, tid, FsyncPolicy::Never).unwrap();
            pdb.register_particle(&particle).unwrap();

            for (ep, state) in [(1, ParticleState::Golden), (2, ParticleState::Dead)] {
                let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
                pdb.commit(
                    ek,
                    particle.clone(),
                    ep,
                    vec![EavTriple::new(ATTR_STATE, encode_state(state).to_vec())],
                )
                .unwrap();
            }
            assert_eq!(pdb.stats().entries_written, 2);
        }

        let pdb2 = PersistedDb::open(&dir, tid, FsyncPolicy::Never).unwrap();
        assert_eq!(pdb2.stats().entries_replayed, 2);
        assert_eq!(pdb2.db().project(&particle).state, ParticleState::Dead);
        cleanup(&dir);
    }

    #[test]
    fn different_tribes_use_separate_dirs() {
        let dir = tmp_dir("tribes");
        let tid1 = TribeId::from_u16(0x0010);
        let tid2 = TribeId::from_u16(0x0020);

        let _p1 = PersistedDb::open(&dir, tid1, FsyncPolicy::Never).unwrap();
        let _p2 = PersistedDb::open(&dir, tid2, FsyncPolicy::Never).unwrap();

        assert!(dir.join("tribe-0010").join("journal.bin").exists());
        assert!(dir.join("tribe-0020").join("journal.bin").exists());
        cleanup(&dir);
    }
}
