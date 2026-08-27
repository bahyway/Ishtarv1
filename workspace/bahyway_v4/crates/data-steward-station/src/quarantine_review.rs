//! QuarantineReviewQueue — the Data Steward's loop-back path for EnkiQDB's
//! fuzzy backlog (§12.3b).
//!
//! EnkiQDB holds particles BlackBox Station could not confirm as harmful
//! (`EventCause::BlackBoxRoutedFuzzy`) pending Data Steward review.  The
//! Steward inspects each case and picks one of three real outcomes:
//!   - resolves it clean        → requeues a fresh Pending particle into EnkiSDB
//!   - confirms it harmful      → seals it into the Storage Sector (terminal)
//!   - promotes it directly     → skips re-validation entirely, straight to EnkiODB Active
//!     (2026-07-31 — the Architect's third decision outcome: "clean enough
//!     that a fresh SDB pass would be redundant")
//!
//! EnkiQDB is append-only: reviewing a case here never touches or removes
//! the original `QuarantineRecord` — that record is the permanent audit
//! trail. Resolution only creates new downstream state. Note on
//! `resolve_promote_to_odb` specifically: `QuarantineRecord` never
//! carried the particle's original EAV attribute data (only identity/
//! metadata), so a caller wanting real attribute data on the promoted
//! ODB particle must supply it themselves (e.g. re-read from whatever
//! source still has it) — this module cannot recover data QDB never
//! stored.

use enkidb_journal::entry::EavTriple;
use enkidb_journal::{EventCause, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki, KakiMinter, KakiRole};
use enkiodb::OdbStore;
use enkiqdb::{QdbStore, QuarantineRecord};
use enkisdb::sdb_store::{SdbStatus, StagedParticle};
use enkisdb::SdbStore;
use storage_sector::StorageSector;

/// One case pulled from EnkiQDB awaiting a Steward decision.
#[derive(Debug, Clone)]
pub struct StewardCase {
    /// Index of the source record inside `QdbStore` (audit reference only —
    /// the record itself is never mutated or removed).
    pub qdb_index: usize,
    pub kaki_bytes: [u8; 16],
    pub tribe_id: u16,
    pub epoch: u32,
    pub color_rgb: [u8; 3],
}

/// The Steward's EnkiQDB review backlog.
pub struct QuarantineReviewQueue {
    cases: Vec<StewardCase>,
    next_qdb_index: usize,
}

impl QuarantineReviewQueue {
    pub fn new() -> Self {
        QuarantineReviewQueue {
            cases: Vec::new(),
            next_qdb_index: 0,
        }
    }

    /// Pull any EnkiQDB records not yet enqueued for review.  Only records
    /// caused by `BlackBoxRoutedFuzzy` are eligible — confirmed-harmful
    /// particles never reach EnkiQDB, they go straight to the Storage
    /// Sector.  Returns how many new cases were enqueued.
    pub fn pull_from_qdb(&mut self, qdb: &QdbStore) -> usize {
        let all = qdb.all();
        let mut pulled = 0;
        while self.next_qdb_index < all.len() {
            let rec: &QuarantineRecord = &all[self.next_qdb_index];
            if rec.quarantine_cause == EventCause::BlackBoxRoutedFuzzy {
                self.cases.push(StewardCase {
                    qdb_index: self.next_qdb_index,
                    kaki_bytes: rec.kaki_bytes,
                    tribe_id: rec.tribe_id,
                    epoch: rec.epoch,
                    color_rgb: rec.color_rgb,
                });
                pulled += 1;
            }
            self.next_qdb_index += 1;
        }
        pulled
    }

    pub fn cases(&self) -> &[StewardCase] {
        &self.cases
    }
    pub fn len(&self) -> usize {
        self.cases.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }

    /// Steward resolves the case at `case_pos` (index into `cases()`) as
    /// clean: requeue a fresh Pending particle back into EnkiSDB for
    /// another pass through the pipeline.  Returns the new EnkiSDB index.
    pub fn resolve_clean(
        &mut self,
        case_pos: usize,
        sdb: &mut SdbStore,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> Option<usize> {
        if case_pos >= self.cases.len() {
            return None;
        }
        let case = self.cases.remove(case_pos);

        let idx = sdb.stage(StagedParticle {
            kaki_bytes: case.kaki_bytes,
            tribe_id: case.tribe_id,
            epoch: case.epoch,
            eav: Vec::new(),
            color_rgb: case.color_rgb,
            status: SdbStatus::Pending,
            arrived_tick: current_tick,
            malware_flag: false,
        });

        journal_requeue(&case, journal, minter);
        Some(idx)
    }

    /// Steward confirms the case at `case_pos` is actually harmful after
    /// all: seal it into the Storage Sector instead of returning it to the
    /// pipeline.  Returns the new Storage Sector index.
    pub fn resolve_confirmed_harmful(
        &mut self,
        case_pos: usize,
        storage_sector: &mut StorageSector,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> Option<usize> {
        if case_pos >= self.cases.len() {
            return None;
        }
        let case = self.cases.remove(case_pos);

        let particle = StagedParticle {
            kaki_bytes: case.kaki_bytes,
            tribe_id: case.tribe_id,
            epoch: case.epoch,
            eav: Vec::new(),
            color_rgb: case.color_rgb,
            status: SdbStatus::Quarantined,
            arrived_tick: current_tick,
            malware_flag: true,
        };
        Some(storage_sector.seal(&particle, journal, minter, current_tick))
    }

    /// Steward judges the case at `case_pos` clean enough to skip
    /// re-validation entirely: promote it straight into EnkiODB as
    /// Active. `eav` lets the caller supply real attribute data QDB
    /// itself never stored (see this module's own doc comment); pass
    /// `Vec::new()` if none is available. Returns the new EnkiODB index.
    pub fn resolve_promote_to_odb(
        &mut self,
        case_pos: usize,
        eav: Vec<EavTriple>,
        odb: &mut OdbStore,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> Option<usize> {
        if case_pos >= self.cases.len() {
            return None;
        }
        let case = self.cases.remove(case_pos);

        odb.ingest_from_steward_promotion(
            case.kaki_bytes,
            case.tribe_id,
            case.epoch,
            eav,
            case.color_rgb,
            journal,
            minter,
            current_tick,
        )
    }
}

impl Default for QuarantineReviewQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn journal_requeue(case: &StewardCase, journal: &mut Journal, minter: &KakiMinter) {
    if let Some(target_kaki) = reconstruct_identity(&case.kaki_bytes) {
        let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
            .expect("KakiMinter always produces valid event KAKIs");
        let entry = JournalEntry::new(event_kaki, target_kaki, case.epoch, Vec::new())
            .with_color_snapshot(case.color_rgb, 0.0) // drift=0.0 — cleared, returning to pipeline
            .with_event_cause(EventCause::StewardResolvedRequeue);
        let _ = journal.append(entry);
    }
}

fn reconstruct_identity(bytes: &[u8; 16]) -> Option<IdentityKaki> {
    Kaki::from_bytes(*bytes)
        .ok()
        .and_then(|k| IdentityKaki::try_from_kaki(k).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiMinter;
    use enkiodb::{OdbSource, OdbStatus};

    fn make_minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0001))
    }

    fn make_fuzzy_staged(minter: &KakiMinter) -> StagedParticle {
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: 1,
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [100, 100, 100],
            status: SdbStatus::Quarantined,
            arrived_tick: 0,
            malware_flag: false,
        }
    }

    #[test]
    fn pull_from_qdb_only_takes_fuzzy_causes() {
        let minter = make_minter();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);

        qdb.accept(
            &make_fuzzy_staged(&minter),
            EventCause::BlackBoxRoutedFuzzy,
            &mut jnl,
            &minter,
            0,
        );
        qdb.accept(
            &make_fuzzy_staged(&minter),
            EventCause::SdbValidationFail,
            &mut jnl,
            &minter,
            0,
        );

        let mut queue = QuarantineReviewQueue::new();
        let pulled = queue.pull_from_qdb(&qdb);

        assert_eq!(pulled, 1);
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn pull_from_qdb_is_idempotent_cursor() {
        let minter = make_minter();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);
        qdb.accept(
            &make_fuzzy_staged(&minter),
            EventCause::BlackBoxRoutedFuzzy,
            &mut jnl,
            &minter,
            0,
        );

        let mut queue = QuarantineReviewQueue::new();
        assert_eq!(queue.pull_from_qdb(&qdb), 1);
        assert_eq!(queue.pull_from_qdb(&qdb), 0); // no new records since last pull

        qdb.accept(
            &make_fuzzy_staged(&minter),
            EventCause::BlackBoxRoutedFuzzy,
            &mut jnl,
            &minter,
            1,
        );
        assert_eq!(queue.pull_from_qdb(&qdb), 1);
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn resolve_clean_requeues_into_sdb() {
        let minter = make_minter();
        let mut qdb = QdbStore::new();
        let mut sdb = SdbStore::new();
        let mut jnl = Journal::new(64);

        qdb.accept(
            &make_fuzzy_staged(&minter),
            EventCause::BlackBoxRoutedFuzzy,
            &mut jnl,
            &minter,
            0,
        );

        let mut queue = QuarantineReviewQueue::new();
        queue.pull_from_qdb(&qdb);
        let new_idx = queue
            .resolve_clean(0, &mut sdb, &mut jnl, &minter, 1000)
            .unwrap();

        assert_eq!(queue.len(), 0); // case consumed
        assert_eq!(sdb.get(new_idx).unwrap().status, SdbStatus::Pending);
        assert!(!sdb.get(new_idx).unwrap().malware_flag);
        // Original QDB record is untouched (still there — append-only).
        assert_eq!(qdb.len(), 1);
    }

    #[test]
    fn resolve_promote_to_odb_lands_particle_as_active_without_revalidation() {
        let minter = make_minter();
        let mut qdb = QdbStore::new();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);

        qdb.accept(
            &make_fuzzy_staged(&minter),
            EventCause::BlackBoxRoutedFuzzy,
            &mut jnl,
            &minter,
            0,
        );

        let mut queue = QuarantineReviewQueue::new();
        queue.pull_from_qdb(&qdb);
        let new_idx = queue
            .resolve_promote_to_odb(0, Vec::new(), &mut odb, &mut jnl, &minter, 1000)
            .unwrap();

        assert_eq!(queue.len(), 0); // case consumed
        assert_eq!(odb.all()[new_idx].status, OdbStatus::Active);
        assert_eq!(odb.all()[new_idx].source, OdbSource::StewardPromotion);
        // Original QDB record is untouched (still there — append-only).
        assert_eq!(qdb.len(), 1);
    }

    #[test]
    fn resolve_promote_to_odb_on_unknown_case_returns_none() {
        let minter = make_minter();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let mut queue = QuarantineReviewQueue::new();

        assert!(queue
            .resolve_promote_to_odb(0, Vec::new(), &mut odb, &mut jnl, &minter, 1000)
            .is_none());
    }

    #[test]
    fn resolve_confirmed_harmful_seals_storage_sector() {
        let minter = make_minter();
        let mut qdb = QdbStore::new();
        let mut sector = StorageSector::new();
        let mut jnl = Journal::new(64);

        qdb.accept(
            &make_fuzzy_staged(&minter),
            EventCause::BlackBoxRoutedFuzzy,
            &mut jnl,
            &minter,
            0,
        );

        let mut queue = QuarantineReviewQueue::new();
        queue.pull_from_qdb(&qdb);
        let sealed_idx = queue
            .resolve_confirmed_harmful(0, &mut sector, &mut jnl, &minter, 1000)
            .unwrap();

        assert_eq!(queue.len(), 0);
        assert_eq!(sector.len(), 1);
        assert_eq!(sector.all()[sealed_idx].sealed_tick, 1000);
    }
}
