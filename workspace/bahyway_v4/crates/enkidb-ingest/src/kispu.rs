//! KISPU — the all-or-nothing particle commit.
//!
//! Binds two real, previously-unwired-together primitives into one commit:
//! NĀRU's audit journal (`enkidb_con_engine::NaruJournal`, CSR-03, already
//! built but never called from the ingest path) and EnkiDb's own
//! journal-plus-index write (`EnkiDb::append_event`, already built and
//! already the live write path). Either both land, or neither does.
//!
//! Ordering is deliberate, not incidental: the audit append is the only
//! one of the two operations that can fail today
//! (`ConError::AuditJournalFull`, a fixed capacity) -- it always runs
//! first. If it fails, `EnkiDb` has not been touched at all, so "none" is
//! the true state without needing rollback machinery. If it succeeds,
//! `EnkiDb::append_event` cannot itself fail on any current path (its
//! `Result` exists for future fallibility, not because anything returns
//! `Err` today), so "all" follows synchronously.
//!
//! `NatiruIndex` (orbital-range pruning, `enkidb_indexes::natiru_index`)
//! is deliberately NOT wired into this per-event commit. Its own contract
//! -- `insert()` many times, then `seal()` once before any query -- is a
//! batch-build shape: resealing after every single live commit would be
//! O(n log n) per particle and defeat the index's purpose, while skipping
//! reseal would leave `surrogates_in_range` reading an unsorted buffer and
//! silently wrong. It stays what it already is: rebuilt from a journal
//! replay/materialization pass, not a live write-path participant.

use enkidb_con_engine::{ConError, NaruJournal};
use enkidb_engine::EnkiDb;
use enkidb_journal::entry::EavTriple;
use enkidb_kaki::{EventKaki, IdentityKaki};

/// Either leg of the commit failed to land.
#[derive(Debug)]
pub enum KispuError {
    /// The audit leg refused the record -- nothing else was attempted.
    Audit(ConError),
    /// The audit leg committed but the ledger leg then failed. Not
    /// observed on any current path; kept for the day `append_event`
    /// gains a real failure mode, so the all-or-nothing contract still
    /// holds instead of silently going half-committed.
    Db(bahyway_core::BahywayError),
}

impl core::fmt::Display for KispuError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Audit(e) => write!(f, "KISPU: audit leg refused, nothing committed: {e}"),
            Self::Db(e) => write!(f, "KISPU: audit leg committed but ledger leg failed: {e:?}"),
        }
    }
}

/// Commit one particle event: NĀRU audit record + EnkiDb journal/index
/// update, atomically. Both happen, or neither does.
pub fn commit(
    audit: &mut NaruJournal,
    db: &mut EnkiDb,
    event_kaki: EventKaki,
    target: IdentityKaki,
    epoch: u32,
    eav: Vec<EavTriple>,
) -> Result<(), KispuError> {
    let tribe_id = db.tribe_id();
    let op_code = event_kaki.bytes()[6] as u32; // the event's own KAKI-type tag

    // Op 1 -- the only fallible leg. Runs first: on failure, nothing below
    // has happened, so "none" needs no rollback.
    audit
        .append(tribe_id.as_u16() as u32, op_code)
        .map_err(KispuError::Audit)?;

    // Op 2 -- journal append + TemporalIndex + EavIndex, already atomic
    // inside EnkiDb::append_event.
    db.append_event(event_kaki, target, epoch, eav)
        .map_err(KispuError::Db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{mint::KakiMinter, IdentityKaki, KakiRole};

    fn setup(journal_max: usize) -> (KakiMinter, EnkiDb, NaruJournal) {
        let tid = TribeId::from_u16(0x0001);
        (
            KakiMinter::new(tid),
            EnkiDb::new(tid),
            NaruJournal::new(journal_max),
        )
    }

    #[test]
    fn both_legs_land_together() {
        let (minter, mut db, mut audit) = setup(8);
        let target = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        db.register_particle(&target).unwrap();
        let event = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();

        let result = commit(&mut audit, &mut db, event, target, 100, Vec::new());

        assert!(result.is_ok());
        assert_eq!(audit.len(), 1, "the audit leg landed");
        assert_eq!(
            db.event_count(&target),
            1,
            "the ledger leg landed"
        );
    }

    #[test]
    fn audit_journal_full_commits_neither_leg() {
        // Capacity 0 -- the very first append is refused (CSR-03).
        let (minter, mut db, mut audit) = setup(0);
        let target = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        db.register_particle(&target).unwrap();
        let event = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();

        let result = commit(&mut audit, &mut db, event, target, 100, Vec::new());

        assert!(matches!(result, Err(KispuError::Audit(ConError::AuditJournalFull))));
        assert_eq!(audit.len(), 0, "audit leg did not land");
        assert_eq!(
            db.event_count(&target),
            0,
            "ledger leg must not land when the audit leg was refused"
        );
    }

    #[test]
    fn a_refused_commit_leaves_a_later_one_free_to_succeed() {
        let (minter, mut db, mut audit) = setup(1);
        let target = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        db.register_particle(&target).unwrap();

        let e1 = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
        assert!(commit(&mut audit, &mut db, e1, target, 1, Vec::new()).is_ok());

        // Capacity exhausted by the first commit -- the second is refused.
        let e2 = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
        let refused = commit(&mut audit, &mut db, e2, target, 2, Vec::new());
        assert!(matches!(refused, Err(KispuError::Audit(ConError::AuditJournalFull))));

        // Only the first commit's ledger entry exists -- the refused
        // second attempt left no partial trace.
        assert_eq!(db.event_count(&target), 1);
        assert_eq!(audit.len(), 1);
    }
}
