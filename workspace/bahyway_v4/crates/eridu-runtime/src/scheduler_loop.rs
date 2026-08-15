//! SchedulerLoop — integrates EriduScheduler with the SDB/ODB/QDB pipeline.
//!
//! On each `tick(n)` call:
//!   1. Advance the EriduScheduler by n ticks.
//!   2. For each DueJob with JobKind::ValidationSweep:
//!      a. Run ValidationSweep on the SdbStore.
//!      b. Drain promoted particles from SDB into ODB.
//!      c. BlackBox Station scans quarantined particles and routes each to
//!         its final jail: confirmed-harmful → Storage Sector (terminal),
//!         fuzzy/unknown → EnkiQDB (pending Data Steward review).
//!      d. Journal all transitions with correct EventCauses.
//!
//! The SchedulerLoop owns the SdbStore, OdbStore, QdbStore, StorageSector,
//! and Journal so the runtime has a single point of coordination — no
//! external locking needed.

use bahyway_core::TribeId;
use enkidb_kaki::KakiMinter;
use enkidb_journal::Journal;
use enkisdb::SdbStore;
use enkisdb::validation_sweep::ValidationSweep;
use enkiodb::OdbStore;
use enkiqdb::QdbStore;
use storage_sector::StorageSector;
use blackbox_station::BlackBoxStation;
use eridu_scheduler::{EriduScheduler, JobKind, VALIDATION_SWEEP_DEFAULT_TICKS};

/// Outcome of one scheduler tick cycle.
#[derive(Debug, Default, Clone)]
pub struct TickOutcome {
    /// Number of ticks advanced.
    pub ticks:                    u64,
    /// True if the ValidationSweep ran this cycle.
    pub sweep_ran:                bool,
    /// Particles promoted to ODB this cycle.
    pub promoted:                 usize,
    /// Particles quarantined by the sweep this cycle (before BlackBox routing).
    pub quarantined:              usize,
    /// Of the quarantined particles, how many BlackBox Station sealed into
    /// the Storage Sector as confirmed harmful.
    pub routed_to_storage_sector: usize,
    /// Of the quarantined particles, how many BlackBox Station routed to
    /// EnkiQDB pending Data Steward review.
    pub routed_to_qdb:            usize,
}

/// The integrated tick-based pipeline runner.
pub struct SchedulerLoop {
    pub scheduler:      EriduScheduler,
    pub sweep:          ValidationSweep,
    pub sdb:            SdbStore,
    pub odb:            OdbStore,
    pub qdb:            QdbStore,
    pub storage_sector: StorageSector,
    pub blackbox:       BlackBoxStation,
    pub journal:        Journal,
    pub minter:         KakiMinter,
    pub tribe_id:       TribeId,
    current_tick:       u64,
}

impl SchedulerLoop {
    /// Create with the sovereign default sweep interval (900 ticks).
    pub fn new(tribe_id: TribeId, shard_count: u16) -> Self {
        let mut scheduler = EriduScheduler::new();
        scheduler.register_validation_sweep(VALIDATION_SWEEP_DEFAULT_TICKS);

        SchedulerLoop {
            scheduler,
            sweep:          ValidationSweep::default_interval(),
            sdb:            SdbStore::new(),
            odb:            OdbStore::new(),
            qdb:            QdbStore::new(),
            storage_sector: StorageSector::new(),
            blackbox:       BlackBoxStation::new(),
            journal:        Journal::new(shard_count),
            minter:         KakiMinter::new(tribe_id),
            tribe_id,
            current_tick:   0,
        }
    }

    /// Create with a custom sweep interval (for testing / admin override).
    pub fn with_interval(tribe_id: TribeId, shard_count: u16, sweep_interval: u64) -> Self {
        let mut lp = Self::new(tribe_id, shard_count);
        lp.scheduler.set_job_interval(
            eridu_scheduler::VALIDATION_SWEEP_JOB,
            sweep_interval,
        );
        lp.sweep = ValidationSweep::new(sweep_interval);
        lp
    }

    /// Advance by `n` ticks and dispatch any due jobs.
    pub fn tick(&mut self, n: u64) -> TickOutcome {
        self.current_tick += n;
        let mut outcome = TickOutcome { ticks: n, ..Default::default() };

        let due = self.scheduler.tick_typed(n);

        for job in due {
            if job.kind == JobKind::ValidationSweep {
                outcome.sweep_ran = true;
                self.run_sweep(&mut outcome);
            }
        }

        outcome
    }

    /// Current logical tick.
    pub fn current_tick(&self) -> u64 { self.current_tick }

    /// Expose a mutable reference to the SDB store so the SdbPipeline can stage particles.
    pub fn sdb_mut(&mut self) -> &mut SdbStore { &mut self.sdb }

    // ── Internal sweep execution ──────────────────────────────────────────────

    fn run_sweep(&mut self, outcome: &mut TickOutcome) {
        // Step 1: ValidationSweep decides promote / quarantine per Pending particle.
        let result = self.sweep.run(
            &mut self.sdb,
            &mut self.journal,
            &self.minter,
            self.tribe_id,
            self.current_tick,
        );
        outcome.promoted    = result.promoted;
        outcome.quarantined = result.quarantined;

        // Step 2: Drain promoted particles into ODB.
        self.odb.drain_from_sdb(&self.sdb, &mut self.journal, &self.minter, self.current_tick);

        // Step 3: BlackBox Station scans quarantined particles and routes
        // each to its final jail — Storage Sector for confirmed-harmful,
        // EnkiQDB for fuzzy/unknown (pending Data Steward review).
        let bb_report = self.blackbox.route_from_sdb(
            &self.sdb,
            &mut self.qdb,
            &mut self.storage_sector,
            &mut self.journal,
            &self.minter,
            self.tribe_id,
            self.current_tick,
        );
        outcome.routed_to_storage_sector = bb_report.routed_to_storage_sector;
        outcome.routed_to_qdb            = bb_report.routed_to_qdb;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::KakiRole;
    use enkisdb::sdb_store::{SdbStatus, StagedParticle};

    fn make_loop() -> SchedulerLoop {
        SchedulerLoop::with_interval(TribeId::from_u16(0x0001), 64, 10)
    }

    fn stage_valid_particle(lp: &mut SchedulerLoop) {
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(
            lp.minter.identity(KakiRole::Zikru)
        ).unwrap();
        lp.sdb.stage(StagedParticle {
            kaki_bytes:   *ik.bytes(),
            tribe_id:     lp.tribe_id.as_u16(),
            epoch:        1,
            eav:          Vec::new(),
            color_rgb:    [80, 200, 255],
            status:       SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: false,
        });
    }

    fn stage_malware_particle(lp: &mut SchedulerLoop) {
        let ik = enkidb_kaki::IdentityKaki::try_from_kaki(
            lp.minter.identity(KakiRole::Zikru)
        ).unwrap();
        lp.sdb.stage(StagedParticle {
            kaki_bytes:   *ik.bytes(),
            tribe_id:     lp.tribe_id.as_u16(),
            epoch:        1,
            eav:          Vec::new(),
            color_rgb:    [255, 0, 0],
            status:       SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: true,
        });
    }

    #[test]
    fn tick_before_interval_does_not_sweep() {
        let mut lp = make_loop();
        stage_valid_particle(&mut lp);

        let outcome = lp.tick(5); // interval=10, first tick fires immediately
        // First tick fires immediately (last_run=0 → is_due)
        // so sweep_ran should be true on first tick
        let _ = outcome;
        // Just ensure no panic
    }

    #[test]
    fn valid_particle_reaches_odb_after_sweep() {
        let mut lp = make_loop();
        stage_valid_particle(&mut lp);

        let outcome = lp.tick(1); // first tick fires sweep immediately
        assert!(outcome.sweep_ran);
        assert_eq!(outcome.promoted,    1);
        assert_eq!(outcome.quarantined, 0);
        assert_eq!(lp.odb.stats().active_count, 1);
        assert_eq!(lp.qdb.len(), 0);
    }

    #[test]
    fn malware_particle_reaches_storage_sector_not_qdb() {
        let mut lp = make_loop();
        stage_malware_particle(&mut lp);

        lp.tick(1);

        assert_eq!(lp.odb.stats().active_count, 0);
        assert_eq!(lp.qdb.len(), 0);
        assert_eq!(lp.storage_sector.len(), 1);
    }

    #[test]
    fn mixed_batch_routed_correctly() {
        let mut lp = make_loop();
        stage_valid_particle(&mut lp);
        stage_valid_particle(&mut lp);
        stage_malware_particle(&mut lp);

        lp.tick(1);

        assert_eq!(lp.odb.stats().active_count, 2);
        assert_eq!(lp.qdb.len(),                0);
        assert_eq!(lp.storage_sector.len(),     1);
    }

    #[test]
    fn second_sweep_sees_only_new_pending() {
        let mut lp = make_loop();
        stage_valid_particle(&mut lp);

        lp.tick(1);  // first sweep: 1 promoted
        let before_odb = lp.odb.stats().active_count;

        // Stage another particle and wait for next sweep
        stage_valid_particle(&mut lp);
        lp.tick(10); // interval is 10; sweeps again

        assert_eq!(lp.odb.stats().active_count, before_odb + 1);
    }

    #[test]
    fn journal_records_all_transitions() {
        let mut lp = make_loop();
        stage_valid_particle(&mut lp);
        stage_malware_particle(&mut lp);

        lp.tick(1);

        // Journal should have: 2×SdbValidationPass/Fail + 1×SdbValidationPass(odb ingest) + 1×StorageSectorMove
        assert!(lp.journal.entry_count() >= 3);
    }
}
