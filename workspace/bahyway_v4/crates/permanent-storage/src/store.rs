//! PermanentStore — commits cleansed Golden Records to the EnkiDb Journal (§10.7).
//!
//! This is the final station in the ingestion pipeline. Only records that
//! pass all upstream checks (Musarû → VGCA → Structure → Cleansing)
//! reach PermanentStore for append-only commit.
//!
//! "Permanent" means append-only — committed records are never modified.

use bahyway_core::Result;
use enkidb_kaki::{EventKaki, IdentityKaki};
use enkidb_journal::entry::EavTriple;
use enkidb_engine::EnkiDb;

/// Records how many events have been committed in this session.
pub struct CommitStats {
    pub events_committed: usize,
}

/// The Permanent Store — wraps EnkiDb for the final pipeline step.
pub struct PermanentStore<'a> {
    db:    &'a mut EnkiDb,
    stats: CommitStats,
}

impl<'a> PermanentStore<'a> {
    pub fn new(db: &'a mut EnkiDb) -> Self {
        PermanentStore { db, stats: CommitStats { events_committed: 0 } }
    }

    /// Commit one event to the Journal (append-only, no UPDATE/DELETE).
    pub fn commit(
        &mut self,
        event_kaki:  EventKaki,
        particle:    IdentityKaki,
        epoch:       u32,
        eav:         Vec<EavTriple>,
    ) -> Result<()> {
        // Register the particle's identity slot on first commit.
        // register_particle is idempotent (overwrites same uuid_hash slot).
        self.db.register_particle(&particle)?;
        self.db.append_event(event_kaki, particle, epoch, eav)?;
        self.stats.events_committed += 1;
        Ok(())
    }

    pub fn stats(&self) -> &CommitStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_engine::EnkiDb;
    use enkidb_kaki::{KakiRole, mint::KakiMinter};
    use bahyway_core::{ParticleState, TribeId};
    use story_engine::projection::{encode_state, ATTR_STATE};

    #[test]
    fn commit_and_project() {
        let tid = TribeId::from_u16(0x0001);
        let m   = KakiMinter::new(tid);
        let mut db = EnkiDb::new(tid);
        let mut ps = PermanentStore::new(&mut db);

        let p  = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let ek = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let eav = vec![EavTriple::new(ATTR_STATE, encode_state(ParticleState::Golden).to_vec())];

        ps.commit(ek, p.clone(), 1, eav).unwrap();
        assert_eq!(ps.stats().events_committed, 1);

        let state = db.project(&p);
        assert_eq!(state.state, ParticleState::Golden);
    }

    #[test]
    fn two_commits_tracked() {
        let tid = TribeId::from_u16(0x0001);
        let m   = KakiMinter::new(tid);
        let mut db = EnkiDb::new(tid);
        let mut ps = PermanentStore::new(&mut db);

        let p   = IdentityKaki::try_from_kaki(m.identity(KakiRole::Zikru)).unwrap();
        let ek1 = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        let ek2 = EventKaki::try_from_kaki(m.event(KakiRole::Zikru)).unwrap();
        ps.commit(ek1, p.clone(), 1, vec![]).unwrap();
        ps.commit(ek2, p.clone(), 2, vec![]).unwrap();
        assert_eq!(ps.stats().events_committed, 2);
    }
}
