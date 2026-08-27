//! etl-orchestrator — per-entry-point `etl_schema` + real batch/chunk
//! movement across EnkiSDB/EnkiODB.
//!
//! Closes the gap the Architect named directly: these processes must
//! not depend only on a particle's own State EAV attribute to move
//! between the 7 Types EnkiDB — something needs to actually MOVE a
//! whole batch or chunk through the pipeline, and which stations/stores
//! apply must be declared per entry point, not hardcoded once.
//!
//! Two real entry points exist in this ecosystem today, and they
//! genuinely need different schemas:
//!   - `ZipDrop` — `bin/bee-watchdog`'s path: a compressed batch lands,
//!     goes through the full per-record DQ station chain (structure →
//!     cleanse → quality/score → B11 routing), and is staged into
//!     EnkiSDB first (`Pending`), for `eridu-runtime::SchedulerLoop`'s
//!     ValidationSweep to later promote or quarantine.
//!   - `EnterpriseApiDirect` — a client application's own API call: the
//!     Architect's own description ("the Enterprise Apps path will have
//!     less stations needed") — still real structure/cleanse/quality
//!     checks, but no SDB staging: batch-lands straight in EnkiODB as
//!     Active, skipping the ValidationSweep re-check entirely.
//!
//! HONEST SCOPE: this crate does NOT reimplement `bee-watchdog`'s
//! file-format-specific stages (Musarû security gate, VGCA-Δ block
//! analysis, ZIP extraction, `adad-gate` KAKI minting) — those remain
//! real, tested, `bee-watchdog`-specific work, unduplicated here. What
//! this crate adds is the layer neither `bee-watchdog` nor
//! `eridu-runtime::SchedulerLoop` has today: a real, inspectable
//! `EtlSchema` value describing which stations/stores apply per entry
//! point, and a real batch mover that lands a whole `Vec` of records in
//! one call rather than one record at a time with no declared schema.
#![forbid(unsafe_code)]

use bahyway_core::TribeId;
use enkidb_journal::entry::EavTriple;
use enkidb_journal::Journal;
use enkidb_kaki::KakiMinter;
use enkiodb::OdbStore;
use enkisdb::sdb_store::{SdbStatus, StagedParticle};
use enkisdb::SdbStore;

/// Which real path a batch of records arrived through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtlEntryPoint {
    /// `bin/bee-watchdog`'s compressed-file landing-zone path.
    ZipDrop,
    /// A client application's own direct API call.
    EnterpriseApiDirect,
}

/// Where a batch lands once its declared stations have run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandingTarget {
    /// Stage into EnkiSDB as `Pending`, awaiting ValidationSweep.
    SdbStaging,
    /// Land directly in EnkiODB as `Active`, skipping SDB re-validation.
    OdbDirect,
}

/// The declared station sequence + landing target for one entry point —
/// real, inspectable data, not a hardcoded chain buried in a binary's
/// `main()`. Station names here name real crates this workspace already
/// has (`data-structure-station`, `data-cleansing-station`,
/// `client-dq-profile`, `score-engine`); this crate does not re-execute
/// them inline (see module doc's honest-scope note) — `stations` is the
/// declared, queryable schema a caller (or future real chain-runner)
/// consults, not a function-pointer pipeline.
#[derive(Debug, Clone)]
pub struct EtlSchema {
    pub entry_point: EtlEntryPoint,
    pub stations: Vec<&'static str>,
    pub lands_in: LandingTarget,
}

/// The declared schema for a given entry point.
pub fn schema_for(entry_point: EtlEntryPoint) -> EtlSchema {
    match entry_point {
        EtlEntryPoint::ZipDrop => EtlSchema {
            entry_point,
            stations: vec![
                "data-structure-station",
                "data-cleansing-station",
                "client-dq-profile",
                "score-engine",
            ],
            lands_in: LandingTarget::SdbStaging,
        },
        EtlEntryPoint::EnterpriseApiDirect => EtlSchema {
            entry_point,
            stations: vec![
                "data-structure-station",
                "data-cleansing-station",
                "client-dq-profile",
            ],
            lands_in: LandingTarget::OdbDirect,
        },
    }
}

/// One record ready for batch placement: already identified (real KAKI
/// bytes) and carrying whatever EAV the declared schema's stations
/// produced upstream of this orchestrator.
#[derive(Debug, Clone)]
pub struct BatchRecord {
    pub kaki_bytes: [u8; 16],
    pub tribe_id: u16,
    pub epoch: u32,
    pub eav: Vec<EavTriple>,
    pub color_rgb: [u8; 3],
}

/// Outcome of moving one batch through its entry point's schema.
#[derive(Debug, Clone, Default)]
pub struct BatchOutcome {
    pub entry_point: Option<EtlEntryPoint>,
    pub landed: usize,
    pub skipped: usize,
    pub sdb_indices: Vec<usize>,
    pub odb_indices: Vec<usize>,
}

/// Move a whole batch through the entry point's declared schema in one
/// call. `ZipDrop` batch-stages every record into `sdb` as `Pending`
/// (real batch movement — not one particle discovered via polling, a
/// whole `Vec` landed together). `EnterpriseApiDirect` batch-ingests
/// every record straight into `odb` as `Active`, skipping SDB staging
/// entirely, per that entry point's schema.
pub fn run_batch(
    entry_point: EtlEntryPoint,
    batch: Vec<BatchRecord>,
    sdb: &mut SdbStore,
    odb: &mut OdbStore,
    journal: &mut Journal,
    minter: &KakiMinter,
    current_tick: u64,
) -> BatchOutcome {
    let schema = schema_for(entry_point);
    let mut outcome = BatchOutcome {
        entry_point: Some(entry_point),
        ..Default::default()
    };

    match schema.lands_in {
        LandingTarget::SdbStaging => {
            for record in batch {
                let idx = sdb.stage(StagedParticle {
                    kaki_bytes: record.kaki_bytes,
                    tribe_id: record.tribe_id,
                    epoch: record.epoch,
                    eav: record.eav,
                    color_rgb: record.color_rgb,
                    status: SdbStatus::Pending,
                    arrived_tick: current_tick,
                    malware_flag: false,
                });
                outcome.sdb_indices.push(idx);
                outcome.landed += 1;
            }
        }
        LandingTarget::OdbDirect => {
            for record in batch {
                match odb.ingest_from_gui(
                    record.kaki_bytes,
                    record.tribe_id,
                    record.epoch,
                    record.eav,
                    record.color_rgb,
                    journal,
                    minter,
                    current_tick,
                ) {
                    Some(idx) => {
                        outcome.odb_indices.push(idx);
                        outcome.landed += 1;
                    }
                    None => outcome.skipped += 1,
                }
            }
        }
    }

    outcome
}

/// Convenience: mint a fresh Identity-Kaki (role=Zikru) via `minter` and
/// build a `BatchRecord` from it plus tribe/epoch/eav/color — the shape
/// most callers assembling a batch from freshly-minted records need.
pub fn new_batch_record(
    minter: &KakiMinter,
    tribe: TribeId,
    epoch: u32,
    eav: Vec<EavTriple>,
    color_rgb: [u8; 3],
) -> BatchRecord {
    let kaki = minter.identity(enkidb_kaki::KakiRole::Zikru);
    BatchRecord {
        kaki_bytes: *kaki.bytes(),
        tribe_id: tribe.as_u16(),
        epoch,
        eav,
        color_rgb,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minter() -> (KakiMinter, TribeId) {
        let tid = TribeId::from_u16(0x0001);
        (KakiMinter::new(tid), tid)
    }

    #[test]
    fn zip_drop_schema_lands_in_sdb_with_full_station_list() {
        let schema = schema_for(EtlEntryPoint::ZipDrop);
        assert_eq!(schema.lands_in, LandingTarget::SdbStaging);
        assert_eq!(schema.stations.len(), 4);
        assert!(schema.stations.contains(&"score-engine"));
    }

    #[test]
    fn enterprise_api_direct_schema_lands_in_odb_with_fewer_stations() {
        let schema = schema_for(EtlEntryPoint::EnterpriseApiDirect);
        assert_eq!(schema.lands_in, LandingTarget::OdbDirect);
        assert!(schema.stations.len() < schema_for(EtlEntryPoint::ZipDrop).stations.len());
        assert!(
            !schema.stations.contains(&"score-engine"),
            "no B11 routing needed without SDB staging"
        );
    }

    #[test]
    fn zip_drop_batch_stages_every_record_as_pending() {
        let (minter, tid) = make_minter();
        let mut sdb = SdbStore::new();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);

        let batch: Vec<BatchRecord> = (0..5)
            .map(|_| new_batch_record(&minter, tid, 1, Vec::new(), [10, 20, 30]))
            .collect();

        let outcome = run_batch(
            EtlEntryPoint::ZipDrop,
            batch,
            &mut sdb,
            &mut odb,
            &mut jnl,
            &minter,
            0,
        );

        assert_eq!(outcome.landed, 5);
        assert_eq!(outcome.sdb_indices.len(), 5);
        assert!(outcome.odb_indices.is_empty());
        for idx in &outcome.sdb_indices {
            assert_eq!(sdb.get(*idx).unwrap().status, SdbStatus::Pending);
        }
        assert_eq!(
            odb.len(),
            0,
            "EnterpriseApiDirect's target, ODB, must stay untouched by a ZipDrop batch"
        );
    }

    #[test]
    fn enterprise_api_direct_batch_lands_straight_in_odb_as_active() {
        let (minter, tid) = make_minter();
        let mut sdb = SdbStore::new();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);

        let batch: Vec<BatchRecord> = (0..3)
            .map(|_| new_batch_record(&minter, tid, 1, Vec::new(), [10, 20, 30]))
            .collect();

        let outcome = run_batch(
            EtlEntryPoint::EnterpriseApiDirect,
            batch,
            &mut sdb,
            &mut odb,
            &mut jnl,
            &minter,
            0,
        );

        assert_eq!(outcome.landed, 3);
        assert_eq!(outcome.odb_indices.len(), 3);
        assert!(outcome.sdb_indices.is_empty());
        for idx in &outcome.odb_indices {
            assert_eq!(odb.all()[*idx].status, enkiodb::OdbStatus::Active);
            assert_eq!(odb.all()[*idx].source, enkiodb::OdbSource::GuiTransaction);
        }
        assert_eq!(
            sdb.len(),
            0,
            "ZipDrop's target, SDB, must stay untouched by an EnterpriseApiDirect batch"
        );
    }

    #[test]
    fn empty_batch_is_a_real_no_op_not_an_error() {
        let (minter, tid) = make_minter();
        let mut sdb = SdbStore::new();
        let mut odb = OdbStore::new();
        let mut jnl = Journal::new(64);
        let _ = tid;

        let outcome = run_batch(
            EtlEntryPoint::ZipDrop,
            Vec::new(),
            &mut sdb,
            &mut odb,
            &mut jnl,
            &minter,
            0,
        );
        assert_eq!(outcome.landed, 0);
        assert_eq!(outcome.skipped, 0);
    }
}
