//! BlackBoxStation — scans EnkiSDB's Quarantined particles and routes each
//! one to its final jail.

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use enkidb_journal::{Journal, EventCause};
use enkisdb::SdbStore;
use enkisdb::sdb_store::SdbStatus;
use enkiqdb::QdbStore;
use storage_sector::StorageSector;

/// Outcome of one BlackBox Station scan pass.
#[derive(Debug, Default, Clone)]
pub struct BlackBoxReport {
    /// Total Quarantined particles scanned this pass.
    pub scanned:                 usize,
    /// Confirmed harmful — sealed into the Storage Sector.
    pub routed_to_storage_sector: usize,
    /// Fuzzy / unknown — routed to EnkiQDB for Data Steward review.
    pub routed_to_qdb:            usize,
}

/// The BlackBox Station.  Stateless — the routing decision is derived
/// entirely from each particle's `malware_flag`, set by Musarû earlier in
/// the pipeline.
#[derive(Debug, Default, Clone, Copy)]
pub struct BlackBoxStation;

impl BlackBoxStation {
    pub fn new() -> Self { BlackBoxStation }

    /// Scan every Quarantined particle in `sdb` and route it to its final
    /// jail: `StorageSector` for confirmed-harmful particles, `EnkiQDB` for
    /// fuzzy ones.  This replaces an unconditional "everything quarantined
    /// goes to EnkiQDB" drain — EnkiQDB is a Steward review jail, not a
    /// dumping ground for confirmed malware.
    pub fn route_from_sdb(
        &self,
        sdb:            &SdbStore,
        qdb:            &mut QdbStore,
        storage_sector: &mut StorageSector,
        journal:        &mut Journal,
        minter:         &KakiMinter,
        _tribe_id:      TribeId,
        current_tick:   u64,
    ) -> BlackBoxReport {
        let mut report = BlackBoxReport::default();

        let quarantined: Vec<_> = sdb
            .all()
            .iter()
            .filter(|p| p.status == SdbStatus::Quarantined)
            .cloned()
            .collect();

        for p in &quarantined {
            report.scanned += 1;
            if p.malware_flag {
                storage_sector.seal(p, journal, minter, current_tick);
                report.routed_to_storage_sector += 1;
            } else {
                qdb.accept(p, EventCause::BlackBoxRoutedFuzzy, journal, minter, current_tick);
                report.routed_to_qdb += 1;
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::KakiRole;
    use enkisdb::sdb_store::StagedParticle;

    fn make_minter() -> (KakiMinter, TribeId) {
        let tid = TribeId::from_u16(0x0001);
        (KakiMinter::new(tid), tid)
    }

    fn make_quarantined(minter: &KakiMinter, malware: bool) -> StagedParticle {
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        StagedParticle {
            kaki_bytes:   *ik.bytes(),
            tribe_id:     1,
            epoch:        1,
            eav:          Vec::new(),
            color_rgb:    [200, 50, 50],
            status:       SdbStatus::Quarantined,
            arrived_tick: 0,
            malware_flag: malware,
        }
    }

    #[test]
    fn harmful_particle_routes_to_storage_sector_not_qdb() {
        let (minter, tid) = make_minter();
        let mut sdb  = SdbStore::new();
        let mut qdb  = QdbStore::new();
        let mut sector = StorageSector::new();
        let mut jnl  = Journal::new(64);

        sdb.stage(make_quarantined(&minter, true));
        let station = BlackBoxStation::new();
        let report  = station.route_from_sdb(&sdb, &mut qdb, &mut sector, &mut jnl, &minter, tid, 900);

        assert_eq!(report.scanned, 1);
        assert_eq!(report.routed_to_storage_sector, 1);
        assert_eq!(report.routed_to_qdb, 0);
        assert_eq!(sector.len(), 1);
        assert_eq!(qdb.len(), 0);
    }

    #[test]
    fn fuzzy_particle_routes_to_qdb_not_storage_sector() {
        let (minter, tid) = make_minter();
        let mut sdb  = SdbStore::new();
        let mut qdb  = QdbStore::new();
        let mut sector = StorageSector::new();
        let mut jnl  = Journal::new(64);

        sdb.stage(make_quarantined(&minter, false));
        let station = BlackBoxStation::new();
        let report  = station.route_from_sdb(&sdb, &mut qdb, &mut sector, &mut jnl, &minter, tid, 900);

        assert_eq!(report.routed_to_qdb, 1);
        assert_eq!(report.routed_to_storage_sector, 0);
        assert_eq!(qdb.len(), 1);
        assert_eq!(qdb.all()[0].quarantine_cause, EventCause::BlackBoxRoutedFuzzy);
        assert_eq!(sector.len(), 0);
    }

    #[test]
    fn mixed_batch_split_correctly() {
        let (minter, tid) = make_minter();
        let mut sdb  = SdbStore::new();
        let mut qdb  = QdbStore::new();
        let mut sector = StorageSector::new();
        let mut jnl  = Journal::new(64);

        sdb.stage(make_quarantined(&minter, true));
        sdb.stage(make_quarantined(&minter, false));
        sdb.stage(make_quarantined(&minter, false));

        let station = BlackBoxStation::new();
        let report  = station.route_from_sdb(&sdb, &mut qdb, &mut sector, &mut jnl, &minter, tid, 900);

        assert_eq!(report.scanned, 3);
        assert_eq!(report.routed_to_storage_sector, 1);
        assert_eq!(report.routed_to_qdb, 2);
        assert_eq!(sector.len(), 1);
        assert_eq!(qdb.len(), 2);
    }

    #[test]
    fn pending_particles_are_not_scanned() {
        let (minter, tid) = make_minter();
        let mut qdb  = QdbStore::new();
        let mut sector = StorageSector::new();
        let mut jnl  = Journal::new(64);

        let mut sdb = SdbStore::new();
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        sdb.stage(StagedParticle {
            kaki_bytes: *ik.bytes(), tribe_id: 1, epoch: 1, eav: Vec::new(),
            color_rgb: [1, 2, 3], status: SdbStatus::Pending, arrived_tick: 0, malware_flag: true,
        });

        let station = BlackBoxStation::new();
        let report  = station.route_from_sdb(&sdb, &mut qdb, &mut sector, &mut jnl, &minter, tid, 0);
        assert_eq!(report.scanned, 0);
    }
}
