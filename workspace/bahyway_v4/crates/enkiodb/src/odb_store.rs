//! OdbStore — active validated particle store.
//!
//! Particles arrive here via two paths:
//!   1. SDB promotion: EnkiSDB ValidationSweep promoted them (batch ingestion)
//!   2. GUI transaction: EnkiDB committed a GUI event that passed sovereignty
//!
//! Particles live here until retired to EnkiDW (cold archive).
//! State is in-memory; for persistent ODB use enkidb-persist behind OdbStore.

use enkidb_engine::EnkiDb;
use enkidb_journal::entry::EavTriple;
use enkidb_journal::{EventCause, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki, KakiMinter, KakiRole};
use enkisdb::sdb_store::StagedParticle;
use permanent_storage::PermanentStore;

/// Lifecycle status of a particle in the Operational Database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OdbStatus {
    /// Active — visible to queries and the diagnosis engine.
    Active,
    /// Retired — moved to EnkiDW; kept for index integrity but not served.
    Retired,
}

/// One active (or recently-retired) particle in the ODB.
#[derive(Debug, Clone)]
pub struct OdbParticle {
    pub kaki_bytes: [u8; 16],
    pub tribe_id: u16,
    pub epoch: u32,
    pub eav: Vec<EavTriple>,
    pub color_rgb: [u8; 3],
    pub status: OdbStatus,
    /// Tick when this particle was promoted into the ODB.
    pub promoted_tick: u64,
    /// Source path: came from SDB batch or GUI transaction.
    pub source: OdbSource,
}

/// How this particle entered the ODB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdbSource {
    /// Promoted from EnkiSDB after validation sweep.
    SdbPromotion,
    /// Committed directly from the EnkiDB GUI transaction path.
    GuiTransaction,
    /// A Data Steward judged a quarantined (EnkiQDB) particle clean
    /// enough to skip re-validation entirely — the third of the
    /// Steward's three real QDB-review outcomes (data-steward-station::
    /// QuarantineReviewQueue::resolve_promote_to_odb).
    StewardPromotion,
}

/// ODB summary counts.
#[derive(Debug, Default, Clone)]
pub struct OdbStats {
    pub total_ingested: usize,
    pub active_count: usize,
    pub retired_count: usize,
}

/// In-memory Operational Database store.
pub struct OdbStore {
    particles: Vec<OdbParticle>,
    stats: OdbStats,
}

impl OdbStore {
    pub fn new() -> Self {
        OdbStore {
            particles: Vec::new(),
            stats: OdbStats::default(),
        }
    }

    /// Ingest a promoted `StagedParticle` from the SDB.
    /// Writes an `EventCause::SdbValidationPass` Journal entry.
    /// Returns the store index.
    pub fn ingest_from_sdb(
        &mut self,
        particle: &StagedParticle,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> Option<usize> {
        let target_kaki = reconstruct_identity(&particle.kaki_bytes)?;

        let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
            .expect("minter always produces valid event KAKIs");

        let entry = JournalEntry::new(event_kaki, target_kaki, particle.epoch, Vec::new())
            .with_color_snapshot(particle.color_rgb, 0.0)
            .with_event_cause(EventCause::SdbValidationPass);
        let _ = journal.append(entry);

        Some(self.push(OdbParticle {
            kaki_bytes: particle.kaki_bytes,
            tribe_id: particle.tribe_id,
            epoch: particle.epoch,
            eav: particle.eav.clone(),
            color_rgb: particle.color_rgb,
            status: OdbStatus::Active,
            promoted_tick: current_tick,
            source: OdbSource::SdbPromotion,
        }))
    }

    /// Ingest a GUI-committed particle (EnkiDB → EnkiODB path).
    /// Writes an `EventCause::EnkidbTransactionCommit` Journal entry.
    // 2026-08-25, found live: clippy::too_many_arguments (9/7). Each
    // param mirrors one OdbParticle field (kaki_bytes/tribe_id/epoch/
    // eav/color_rgb) plus two infra deps (journal, minter) and
    // current_tick -- bundling these into a params struct would change
    // this fn's public signature and every call site, a real API
    // design call for the Architect, not a mechanical lint fix.
    #[allow(clippy::too_many_arguments)]
    pub fn ingest_from_gui(
        &mut self,
        kaki_bytes: [u8; 16],
        tribe_id: u16,
        epoch: u32,
        eav: Vec<EavTriple>,
        color_rgb: [u8; 3],
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> Option<usize> {
        let target_kaki = reconstruct_identity(&kaki_bytes)?;

        let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
            .expect("minter always produces valid event KAKIs");

        let entry = JournalEntry::new(event_kaki, target_kaki, epoch, Vec::new())
            .with_color_snapshot(color_rgb, 0.0)
            .with_event_cause(EventCause::EnkidbTransactionCommit);
        let _ = journal.append(entry);

        Some(self.push(OdbParticle {
            kaki_bytes,
            tribe_id,
            epoch,
            eav,
            color_rgb,
            status: OdbStatus::Active,
            promoted_tick: current_tick,
            source: OdbSource::GuiTransaction,
        }))
    }

    /// Ingest a particle a Data Steward promoted directly from EnkiQDB
    /// review, skipping re-validation entirely (the third of the
    /// Steward's three real decision outcomes — the other two,
    /// requeue-to-SDB and confirm-quarantine-to-StorageSector, already
    /// existed in `data-steward-station::QuarantineReviewQueue`).
    /// Writes an `EventCause::StewardPromotedToOdb` Journal entry —
    /// distinct from `ingest_from_gui`'s `EnkidbTransactionCommit`,
    /// since this is a Steward decision, not a GUI transaction.
    // Same clippy::too_many_arguments reasoning as ingest_from_gui above.
    #[allow(clippy::too_many_arguments)]
    pub fn ingest_from_steward_promotion(
        &mut self,
        kaki_bytes: [u8; 16],
        tribe_id: u16,
        epoch: u32,
        eav: Vec<EavTriple>,
        color_rgb: [u8; 3],
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> Option<usize> {
        let target_kaki = reconstruct_identity(&kaki_bytes)?;

        let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
            .expect("minter always produces valid event KAKIs");

        let entry = JournalEntry::new(event_kaki, target_kaki, epoch, Vec::new())
            .with_color_snapshot(color_rgb, 0.0)
            .with_event_cause(EventCause::StewardPromotedToOdb);
        let _ = journal.append(entry);

        Some(self.push(OdbParticle {
            kaki_bytes,
            tribe_id,
            epoch,
            eav,
            color_rgb,
            status: OdbStatus::Active,
            promoted_tick: current_tick,
            source: OdbSource::StewardPromotion,
        }))
    }

    /// Bulk-drain all Promoted particles from the SDB into the ODB.
    /// Particles already present in the ODB (by kaki_bytes) are skipped.
    /// Returns how many were newly transferred.
    pub fn drain_from_sdb(
        &mut self,
        sdb: &enkisdb::SdbStore,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> usize {
        use enkisdb::SdbStatus;
        let promoted: Vec<StagedParticle> = sdb
            .all()
            .iter()
            .filter(|p| p.status == SdbStatus::Promoted)
            .cloned()
            .collect();

        let mut count = 0;
        for p in &promoted {
            // Skip if this KAKI is already in the ODB (idempotent drain).
            if self.contains_kaki(&p.kaki_bytes) {
                continue;
            }
            if self
                .ingest_from_sdb(p, journal, minter, current_tick)
                .is_some()
            {
                count += 1;
            }
        }
        count
    }

    /// True if any particle with these kaki_bytes exists in the ODB (any status).
    pub fn contains_kaki(&self, kaki_bytes: &[u8; 16]) -> bool {
        self.particles.iter().any(|p| &p.kaki_bytes == kaki_bytes)
    }

    /// Mark particle at `idx` as Retired (moving to EnkiDW).
    /// Writes an `EventCause::ArchiveMove` Journal entry.
    pub fn retire(
        &mut self,
        idx: usize,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> bool {
        let _ = current_tick;
        let p = match self.particles.get_mut(idx) {
            Some(p) if p.status == OdbStatus::Active => p,
            _ => return false,
        };
        p.status = OdbStatus::Retired;
        self.stats.active_count = self.stats.active_count.saturating_sub(1);
        self.stats.retired_count += 1;

        // Journal the archive move
        if let Some(target_kaki) = reconstruct_identity(&p.kaki_bytes) {
            let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
                .expect("minter always produces valid event KAKIs");
            let entry = JournalEntry::new(event_kaki, target_kaki, p.epoch, Vec::new())
                .with_color_snapshot(p.color_rgb, 0.0)
                .with_event_cause(EventCause::ArchiveMove);
            let _ = journal.append(entry);
        }
        true
    }

    /// Commit an Active ODB particle into `EnkiDb` as a real Golden
    /// Record via `PermanentStore` — the corrected law: ODB never routes
    /// to EnkiDW directly; only EnkiDB's own snapshot/partition job
    /// feeds EnkiDW. Journals `EventCause::OdbPromotedToGolden` in ODB's
    /// own journal (a marker, same shape as `retire()`'s `ArchiveMove`
    /// marker) — the real EAV data itself is committed into `db` via
    /// `PermanentStore::commit`, which `retire()` never performed (it
    /// only flips a status bit). Marks the particle `Retired` in ODB's
    /// own bookkeeping: once it's a permanent Golden Record, it's no
    /// longer part of ODB's active operational set. Returns `false`
    /// without mutating anything if `idx` isn't Active, its KAKI bytes
    /// don't reconstruct, or the EnkiDb commit itself fails.
    pub fn promote_to_golden(
        &mut self,
        idx: usize,
        db: &mut EnkiDb,
        journal: &mut Journal,
        minter: &KakiMinter,
    ) -> bool {
        let identity = match self.particles.get(idx) {
            Some(p) if p.status == OdbStatus::Active => match reconstruct_identity(&p.kaki_bytes) {
                Some(id) => id,
                None => return false,
            },
            _ => return false,
        };

        let (epoch, eav, color_rgb) = {
            let p = &self.particles[idx];
            (p.epoch, p.eav.clone(), p.color_rgb)
        };

        let commit_event = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
            .expect("minter always produces valid event KAKIs");
        if PermanentStore::new(db)
            .commit(commit_event, identity, epoch, eav)
            .is_err()
        {
            return false;
        }

        let p = &mut self.particles[idx];
        p.status = OdbStatus::Retired;
        self.stats.active_count = self.stats.active_count.saturating_sub(1);
        self.stats.retired_count += 1;

        let marker_event = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
            .expect("minter always produces valid event KAKIs");
        let entry = JournalEntry::new(marker_event, identity, epoch, Vec::new())
            .with_color_snapshot(color_rgb, 0.0)
            .with_event_cause(EventCause::OdbPromotedToGolden);
        let _ = journal.append(entry);

        true
    }

    /// Lookup all active particles by tribe_id.
    pub fn active_by_tribe(&self, tribe_id: u16) -> Vec<&OdbParticle> {
        self.particles
            .iter()
            .filter(|p| p.status == OdbStatus::Active && p.tribe_id == tribe_id)
            .collect()
    }

    /// Find an active particle by its KAKI bytes.
    pub fn find_by_kaki(&self, kaki_bytes: &[u8; 16]) -> Option<&OdbParticle> {
        self.particles
            .iter()
            .find(|p| p.status == OdbStatus::Active && &p.kaki_bytes == kaki_bytes)
    }

    pub fn active(&self) -> impl Iterator<Item = &OdbParticle> {
        self.particles
            .iter()
            .filter(|p| p.status == OdbStatus::Active)
    }

    pub fn all(&self) -> &[OdbParticle] {
        &self.particles
    }
    pub fn stats(&self) -> &OdbStats {
        &self.stats
    }
    pub fn len(&self) -> usize {
        self.particles.len()
    }
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    fn push(&mut self, p: OdbParticle) -> usize {
        let idx = self.particles.len();
        self.particles.push(p);
        self.stats.total_ingested += 1;
        self.stats.active_count += 1;
        idx
    }
}

impl Default for OdbStore {
    fn default() -> Self {
        Self::new()
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
    use enkidb_journal::Journal;
    use enkidb_kaki::{KakiMinter, KakiRole};
    use enkisdb::sdb_store::{SdbStatus, StagedParticle};

    fn make_minter() -> (KakiMinter, TribeId) {
        let tid = TribeId::from_u16(0x0001);
        (KakiMinter::new(tid), tid)
    }

    fn make_staged_promoted(minter: &KakiMinter) -> StagedParticle {
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: 1,
            epoch: 5,
            eav: Vec::new(),
            color_rgb: [80, 200, 120],
            status: SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: false,
        }
    }

    #[test]
    fn ingest_from_sdb_writes_journal_entry() {
        let (minter, _) = make_minter();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let particle = make_staged_promoted(&minter);

        let idx = odb
            .ingest_from_sdb(&particle, &mut jnl, &minter, 900)
            .unwrap();

        assert_eq!(odb.len(), 1);
        assert_eq!(jnl.entry_count(), 1);
        assert_eq!(odb.all()[idx].source, OdbSource::SdbPromotion);
        assert_eq!(odb.stats().active_count, 1);
    }

    #[test]
    fn drain_from_sdb_transfers_promoted() {
        let (minter, _) = make_minter();
        let mut sdb = enkisdb::SdbStore::new();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);

        let i0 = sdb.stage(make_staged_promoted(&minter));
        let i1 = sdb.stage(make_staged_promoted(&minter));
        let i2 = sdb.stage(make_staged_promoted(&minter));
        sdb.promote(i0);
        sdb.promote(i1);
        // i2 stays Pending — should not be transferred

        let moved = odb.drain_from_sdb(&sdb, &mut jnl, &minter, 900);

        assert_eq!(moved, 2);
        assert_eq!(odb.len(), 2);
        assert_eq!(jnl.entry_count(), 2);
        let _ = i2;
    }

    #[test]
    fn retire_writes_archive_move_event() {
        let (minter, _) = make_minter();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let p = make_staged_promoted(&minter);

        let idx = odb.ingest_from_sdb(&p, &mut jnl, &minter, 0).unwrap();
        let ok = odb.retire(idx, &mut jnl, &minter, 900);

        assert!(ok);
        assert_eq!(odb.stats().retired_count, 1);
        assert_eq!(odb.stats().active_count, 0);
        assert_eq!(jnl.entry_count(), 2); // ingest + retire
        assert_eq!(odb.all()[idx].status, OdbStatus::Retired);
    }

    #[test]
    fn find_by_kaki_returns_active_only() {
        let (minter, _) = make_minter();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let p = make_staged_promoted(&minter);
        let kb = p.kaki_bytes;

        let idx = odb.ingest_from_sdb(&p, &mut jnl, &minter, 0).unwrap();
        assert!(odb.find_by_kaki(&kb).is_some());

        odb.retire(idx, &mut jnl, &minter, 0);
        assert!(odb.find_by_kaki(&kb).is_none()); // retired → not findable
    }

    #[test]
    fn promote_to_golden_commits_a_real_golden_record() {
        let (minter, tid) = make_minter();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let mut db = EnkiDb::new(tid);
        let p = make_staged_promoted(&minter);
        let kaki_bytes = p.kaki_bytes;

        let idx = odb.ingest_from_sdb(&p, &mut jnl, &minter, 0).unwrap();
        let ok = odb.promote_to_golden(idx, &mut db, &mut jnl, &minter);

        assert!(ok);
        assert_eq!(odb.stats().retired_count, 1);
        assert_eq!(odb.stats().active_count, 0);
        assert_eq!(odb.all()[idx].status, OdbStatus::Retired);

        // The real proof: EnkiDb itself now projects this particle as
        // registered -- not just an ODB-side status flip.
        let identity = reconstruct_identity(&kaki_bytes).unwrap();
        let projected = db.project(&identity);
        assert!(
            projected.events_seen > 0,
            "EnkiDb must have a real committed event for this particle"
        );

        // ODB's own journal recorded the correct, distinct marker cause
        // (not ArchiveMove, which retire() uses for a different meaning).
        let hist = jnl.read_particle_history(&identity);
        assert!(
            hist.iter()
                .any(|e| e.event_cause() == Some(EventCause::OdbPromotedToGolden)),
            "expected an OdbPromotedToGolden marker in ODB's own journal"
        );
    }

    #[test]
    fn promote_to_golden_refuses_an_already_retired_particle() {
        let (minter, tid) = make_minter();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let mut db = EnkiDb::new(tid);
        let p = make_staged_promoted(&minter);

        let idx = odb.ingest_from_sdb(&p, &mut jnl, &minter, 0).unwrap();
        assert!(odb.retire(idx, &mut jnl, &minter, 900)); // already retired via the OTHER path

        let ok = odb.promote_to_golden(idx, &mut db, &mut jnl, &minter);
        assert!(!ok, "an already-Retired particle must not be promotable");
    }

    #[test]
    fn promote_to_golden_refuses_an_out_of_range_index() {
        let (minter, tid) = make_minter();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let mut db = EnkiDb::new(tid);

        assert!(!odb.promote_to_golden(999, &mut db, &mut jnl, &minter));
    }

    #[test]
    fn steward_promotion_ingest() {
        let (minter, _) = make_minter();
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);

        let idx = odb
            .ingest_from_steward_promotion(
                *ik.bytes(),
                1,
                10,
                Vec::new(),
                [90, 90, 90],
                &mut jnl,
                &minter,
                0,
            )
            .unwrap();

        assert_eq!(odb.all()[idx].source, OdbSource::StewardPromotion);
        assert_eq!(jnl.entry_count(), 1);
        let hist = jnl.read_particle_history(&ik);
        assert_eq!(
            hist[0].event_cause(),
            Some(EventCause::StewardPromotedToOdb)
        );
    }

    #[test]
    fn gui_transaction_ingest() {
        let (minter, _) = make_minter();
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);

        let idx = odb
            .ingest_from_gui(
                *ik.bytes(),
                1,
                10,
                Vec::new(),
                [120, 180, 255],
                &mut jnl,
                &minter,
                0,
            )
            .unwrap();

        assert_eq!(odb.all()[idx].source, OdbSource::GuiTransaction);
        assert_eq!(jnl.entry_count(), 1);
        let hist = jnl.read_particle_history(&ik);
        assert_eq!(
            hist[0].event_cause(),
            Some(EventCause::EnkidbTransactionCommit)
        );
    }
}
