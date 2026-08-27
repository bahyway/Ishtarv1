//! QdbStore — permanent append-only quarantine archive.
//!
//! Once a particle is written here it can never be removed or promoted.
//! This is the terminal state for all particles rejected by EnkiSDB or
//! Musarû late-discovery.  Administrators use `QdbReport` for forensic
//! review.

use enkidb_journal::{EventCause, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki, KakiMinter, KakiRole};
use enkisdb::sdb_store::StagedParticle;

/// One permanently quarantined particle.
#[derive(Debug, Clone)]
pub struct QuarantineRecord {
    /// Raw 16-byte KAKI identity bytes.
    pub kaki_bytes: [u8; 16],
    /// Tribe this particle claimed to belong to.
    pub tribe_id: u16,
    /// Epoch at time of original arrival in the SDB.
    pub epoch: u32,
    /// Color_ID(RGB) at time of quarantine.
    pub color_rgb: [u8; 3],
    /// Exact EventCause that triggered quarantine.
    pub quarantine_cause: EventCause,
    /// Tick when the particle arrived in the SDB.
    pub arrived_tick: u64,
    /// Tick when the quarantine decision was made.
    pub quarantine_tick: u64,
    /// True if Musarû flagged a byte-level malware signature.
    pub malware_flag: bool,
}

/// Aggregate counts for the quarantine archive.
#[derive(Debug, Default, Clone)]
pub struct QdbReport {
    pub total_quarantined: usize,
    pub malware_count: usize,
    pub tribe_mismatch_count: usize,
    pub validation_fail_count: usize,
    pub late_quarantine_count: usize,
}

impl QdbReport {
    /// Security risk percentage: malware particles / total.
    pub fn malware_ratio(&self) -> f32 {
        if self.total_quarantined == 0 {
            return 0.0;
        }
        self.malware_count as f32 / self.total_quarantined as f32
    }
}

/// Append-only quarantine archive.  One per tribe.  Nothing is ever removed.
pub struct QdbStore {
    records: Vec<QuarantineRecord>,
}

impl QdbStore {
    pub fn new() -> Self {
        QdbStore {
            records: Vec::new(),
        }
    }

    /// Accept a quarantined `StagedParticle` from the SDB, record it here,
    /// and write a `QuarantineMove` JournalEntry.
    ///
    /// Returns the store index of the new record.
    pub fn accept(
        &mut self,
        particle: &StagedParticle,
        cause: EventCause,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> usize {
        let record = QuarantineRecord {
            kaki_bytes: particle.kaki_bytes,
            tribe_id: particle.tribe_id,
            epoch: particle.epoch,
            color_rgb: particle.color_rgb,
            quarantine_cause: cause,
            arrived_tick: particle.arrived_tick,
            quarantine_tick: current_tick,
            malware_flag: particle.malware_flag,
        };

        // Journal the quarantine move so HeptaScript can query it.
        if let Some(target_kaki) = reconstruct_identity(&particle.kaki_bytes) {
            let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
                .expect("KakiMinter always produces valid event KAKIs");
            let entry = JournalEntry::new(event_kaki, target_kaki, particle.epoch, Vec::new())
                .with_color_snapshot(particle.color_rgb, 1.0) // drift=1.0 signals terminal bad state
                .with_event_cause(EventCause::QuarantineMove);
            let _ = journal.append(entry);
        }

        let idx = self.records.len();
        self.records.push(record);
        idx
    }

    /// Drain all Quarantined particles from the SDB store into this QDB.
    /// Particles already recorded here (by kaki_bytes) are skipped, so this
    /// is safe to call repeatedly against a store whose Quarantined
    /// particles persist across sweeps (unlike EnkiODB's Active particles,
    /// nothing here ever transitions away from Quarantined — see
    /// `OdbStore::drain_from_sdb`'s identical idempotency guard).
    /// Returns how many were newly transferred.
    pub fn drain_from_sdb(
        &mut self,
        sdb: &enkisdb::SdbStore,
        cause: EventCause,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> usize {
        use enkisdb::SdbStatus;
        let quarantined: Vec<StagedParticle> = sdb
            .all()
            .iter()
            .filter(|p| p.status == SdbStatus::Quarantined)
            .cloned()
            .collect();

        let mut count = 0;
        for p in &quarantined {
            if self.contains_kaki(&p.kaki_bytes) {
                continue;
            }
            self.accept(p, cause, journal, minter, current_tick);
            count += 1;
        }
        count
    }

    /// True if any record with these kaki_bytes already exists in the QDB.
    pub fn contains_kaki(&self, kaki_bytes: &[u8; 16]) -> bool {
        self.records.iter().any(|r| &r.kaki_bytes == kaki_bytes)
    }

    /// Filter records by EventCause.
    pub fn by_cause(&self, cause: EventCause) -> Vec<&QuarantineRecord> {
        self.records
            .iter()
            .filter(|r| r.quarantine_cause == cause)
            .collect()
    }

    /// Filter records flagged by Musarû malware scan.
    pub fn malware_records(&self) -> Vec<&QuarantineRecord> {
        self.records.iter().filter(|r| r.malware_flag).collect()
    }

    /// Build a summary report.
    pub fn report(&self) -> QdbReport {
        let mut r = QdbReport {
            total_quarantined: self.records.len(),
            ..Default::default()
        };
        for rec in &self.records {
            if rec.malware_flag {
                r.malware_count += 1;
            }
            match rec.quarantine_cause {
                EventCause::MusaruMalwareDetected | EventCause::MusaruLateQuarantine => {
                    r.late_quarantine_count += 1
                }
                EventCause::SdbValidationFail => r.validation_fail_count += 1,
                EventCause::QuarantineMove => {} // generic move, counted via total
                _ => r.validation_fail_count += 1,
            }
        }
        r
    }

    pub fn all(&self) -> &[QuarantineRecord] {
        &self.records
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for QdbStore {
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

    fn make_staged(minter: &KakiMinter, malware: bool) -> StagedParticle {
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: 1,
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [255, 0, 0],
            status: SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: malware,
        }
    }

    #[test]
    fn accept_writes_journal_entry() {
        let (minter, _) = make_minter();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);
        let particle = make_staged(&minter, false);

        qdb.accept(
            &particle,
            EventCause::SdbValidationFail,
            &mut jnl,
            &minter,
            900,
        );

        assert_eq!(qdb.len(), 1);
        assert_eq!(jnl.entry_count(), 1);
        assert_eq!(qdb.all()[0].quarantine_cause, EventCause::SdbValidationFail);
    }

    #[test]
    fn malware_particle_reported() {
        let (minter, _) = make_minter();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);

        qdb.accept(
            &make_staged(&minter, true),
            EventCause::MusaruMalwareDetected,
            &mut jnl,
            &minter,
            10,
        );
        qdb.accept(
            &make_staged(&minter, false),
            EventCause::SdbValidationFail,
            &mut jnl,
            &minter,
            10,
        );

        let r = qdb.report();
        assert_eq!(r.total_quarantined, 2);
        assert_eq!(r.malware_count, 1);
        assert!((r.malware_ratio() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn by_cause_filter() {
        let (minter, _) = make_minter();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);

        qdb.accept(
            &make_staged(&minter, true),
            EventCause::MusaruMalwareDetected,
            &mut jnl,
            &minter,
            0,
        );
        qdb.accept(
            &make_staged(&minter, false),
            EventCause::SdbValidationFail,
            &mut jnl,
            &minter,
            0,
        );

        assert_eq!(qdb.by_cause(EventCause::MusaruMalwareDetected).len(), 1);
        assert_eq!(qdb.by_cause(EventCause::SdbValidationFail).len(), 1);
        assert_eq!(qdb.malware_records().len(), 1);
    }

    #[test]
    fn append_only_no_removal() {
        let (minter, _) = make_minter();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);

        qdb.accept(
            &make_staged(&minter, false),
            EventCause::QuarantineMove,
            &mut jnl,
            &minter,
            0,
        );
        qdb.accept(
            &make_staged(&minter, false),
            EventCause::QuarantineMove,
            &mut jnl,
            &minter,
            1,
        );

        assert_eq!(qdb.len(), 2);
        // No method to remove — only append.  Verified by API surface.
    }

    #[test]
    fn drain_from_sdb_transfers_quarantined_particles() {
        let (minter, _) = make_minter();
        let mut sdb = enkisdb::SdbStore::new();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);

        // Stage 3 particles, quarantine 2 of them
        let i0 = sdb.stage(make_staged(&minter, false));
        let i1 = sdb.stage(make_staged(&minter, true));
        let i2 = sdb.stage(make_staged(&minter, false));
        sdb.quarantine(i1);
        sdb.quarantine(i2);
        // i0 stays Pending

        let moved = qdb.drain_from_sdb(&sdb, EventCause::QuarantineMove, &mut jnl, &minter, 900);

        assert_eq!(moved, 2);
        assert_eq!(qdb.len(), 2);
        assert_eq!(jnl.entry_count(), 2);
        let _ = i0;
    }

    #[test]
    fn drain_from_sdb_is_idempotent_across_repeated_calls() {
        let (minter, _) = make_minter();
        let mut sdb = enkisdb::SdbStore::new();
        let mut qdb = QdbStore::new();
        let mut jnl = Journal::new(64);

        let i0 = sdb.stage(make_staged(&minter, true));
        sdb.quarantine(i0);

        let first = qdb.drain_from_sdb(&sdb, EventCause::QuarantineMove, &mut jnl, &minter, 900);
        let second = qdb.drain_from_sdb(&sdb, EventCause::QuarantineMove, &mut jnl, &minter, 1800);

        assert_eq!(first, 1);
        assert_eq!(
            second, 0,
            "already-recorded kaki must not be re-accepted on a later sweep"
        );
        assert_eq!(qdb.len(), 1);
        assert_eq!(jnl.entry_count(), 1);
    }
}
