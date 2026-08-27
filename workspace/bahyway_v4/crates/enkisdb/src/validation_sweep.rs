//! ValidationSweep — runs every 900 ticks (15 min at 1 tick/s) and routes
//! pending SDB particles to EnkiODB (promote) or EnkiQDB (quarantine).
//!
//! Decision rule: a particle is promoted if it has no malware flag and its
//! KAKI sovereignty check passes. Any other condition quarantines it.
//! Both outcomes are recorded as Journal events with the correct EventCause.

use bahyway_core::TribeId;
use enkidb_journal::{EventCause, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki, KakiMinter, KakiRole};

use crate::sdb_store::SdbStore;
use musaru_security::check::{check_sovereignty, SecurityResult};

/// Default sweep cadence (§12.1): 900 ticks = 15 minutes.
pub const DEFAULT_SWEEP_INTERVAL_TICKS: u64 = 900;

/// Outcome of one sweep run.
#[derive(Debug, Default, Clone)]
pub struct SweepResult {
    pub promoted: usize,
    pub quarantined: usize,
}

/// Tick-based sweep scheduler.  Call `is_due()` each tick; call `run()` when due.
pub struct ValidationSweep {
    interval_ticks: u64,
    last_run_tick: u64,
}

impl ValidationSweep {
    pub fn new(interval_ticks: u64) -> Self {
        ValidationSweep {
            interval_ticks,
            last_run_tick: 0,
        }
    }

    /// Convenience constructor using the sovereign default (900 ticks).
    pub fn default_interval() -> Self {
        Self::new(DEFAULT_SWEEP_INTERVAL_TICKS)
    }

    /// True when enough ticks have elapsed since the last run.
    pub fn is_due(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.last_run_tick) >= self.interval_ticks
    }

    /// Examine all Pending particles in `store`.  Promote or quarantine each,
    /// writing a JournalEntry with the correct EventCause.
    ///
    /// `tribe_id` is the owning tribe of this SDB instance.
    pub fn run(
        &mut self,
        store: &mut SdbStore,
        journal: &mut Journal,
        minter: &KakiMinter,
        tribe_id: TribeId,
        current_tick: u64,
    ) -> SweepResult {
        self.last_run_tick = current_tick;
        let mut result = SweepResult::default();
        let indices = store.pending_indices();

        for idx in indices {
            let p = match store.get(idx) {
                Some(p) => p.clone(),
                None => continue,
            };

            let cause = self.decide_cause(&p, tribe_id);
            let is_pass = cause == EventCause::SdbValidationPass;

            // Reconstruct the IdentityKaki so we can write to the Journal.
            let target_kaki = match Kaki::from_bytes(p.kaki_bytes)
                .ok()
                .and_then(|k| IdentityKaki::try_from_kaki(k).ok())
            {
                Some(ik) => ik,
                None => {
                    // Unreconstructable KAKI → quarantine without a Journal entry.
                    store.quarantine(idx);
                    result.quarantined += 1;
                    continue;
                }
            };

            let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
                .expect("KakiMinter always produces valid event KAKIs");

            let entry = JournalEntry::new(event_kaki, target_kaki, p.epoch, Vec::new())
                .with_color_snapshot(p.color_rgb, 0.0)
                .with_event_cause(cause);

            let _ = journal.append(entry);

            if is_pass {
                store.promote(idx);
                result.promoted += 1;
            } else {
                store.quarantine(idx);
                result.quarantined += 1;
            }
        }

        result
    }

    fn decide_cause(&self, p: &crate::sdb_store::StagedParticle, tribe_id: TribeId) -> EventCause {
        if p.malware_flag {
            return EventCause::SdbValidationFail;
        }
        match Kaki::from_bytes(p.kaki_bytes)
            .ok()
            .and_then(|k| IdentityKaki::try_from_kaki(k).ok())
        {
            None => EventCause::SdbValidationFail,
            Some(ik) => match check_sovereignty(tribe_id, &ik) {
                SecurityResult::Approved => EventCause::SdbValidationPass,
                _ => EventCause::SdbValidationFail,
            },
        }
    }

    pub fn interval_ticks(&self) -> u64 {
        self.interval_ticks
    }
    pub fn last_run_tick(&self) -> u64 {
        self.last_run_tick
    }

    /// Admin reconfiguration — change the sweep cadence at runtime.
    pub fn set_interval(&mut self, ticks: u64) {
        assert!(ticks > 0, "sweep interval must be > 0 ticks");
        self.interval_ticks = ticks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdb_store::{SdbStatus, SdbStore, StagedParticle};
    use bahyway_core::TribeId;
    use enkidb_journal::Journal;
    use enkidb_kaki::{KakiMinter, KakiRole};

    fn make_sweep() -> ValidationSweep {
        ValidationSweep::new(10) // short interval for tests
    }

    fn make_store_with_valid_particle(minter: &KakiMinter) -> (SdbStore, usize) {
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut store = SdbStore::new();
        let idx = store.stage(StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: 1,
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [100, 200, 255],
            status: SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: false,
        });
        (store, idx)
    }

    #[test]
    fn is_due_respects_interval() {
        let sweep = ValidationSweep::new(10);
        assert!(!sweep.is_due(0));
        assert!(sweep.is_due(10));
        assert!(sweep.is_due(11));
    }

    #[test]
    fn valid_particle_promoted() {
        let tid = TribeId::from_u16(0x0001);
        let minter = KakiMinter::new(tid);
        let (mut store, idx) = make_store_with_valid_particle(&minter);
        let mut jnl = Journal::new(64);
        let mut sweep = make_sweep();

        let res = sweep.run(&mut store, &mut jnl, &minter, tid, 10);
        assert_eq!(res.promoted, 1);
        assert_eq!(res.quarantined, 0);
        assert_eq!(store.get(idx).unwrap().status, SdbStatus::Promoted);
        assert_eq!(jnl.entry_count(), 1);

        let entries = jnl.all_particles();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn malware_flagged_particle_quarantined() {
        let tid = TribeId::from_u16(0x0001);
        let minter = KakiMinter::new(tid);
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut store = SdbStore::new();
        let idx = store.stage(StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: 1,
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [255, 0, 0],
            status: SdbStatus::Pending,
            arrived_tick: 0,
            malware_flag: true,
        });

        let mut jnl = Journal::new(64);
        let mut sweep = make_sweep();
        let res = sweep.run(&mut store, &mut jnl, &minter, tid, 10);

        assert_eq!(res.quarantined, 1);
        assert_eq!(store.get(idx).unwrap().status, SdbStatus::Quarantined);
    }

    #[test]
    fn wrong_tribe_particle_quarantined() {
        let tid1 = TribeId::from_u16(0x0001);
        let tid2 = TribeId::from_u16(0x0002);
        let m1 = KakiMinter::new(tid1);
        let (mut store, idx) = make_store_with_valid_particle(&m1);
        let m2 = KakiMinter::new(tid2);

        let mut jnl = Journal::new(64);
        let mut sweep = make_sweep();
        // sweep runs in the context of tribe 2, but particle belongs to tribe 1
        let res = sweep.run(&mut store, &mut jnl, &m2, tid2, 10);

        assert_eq!(res.quarantined, 1);
        assert_eq!(store.get(idx).unwrap().status, SdbStatus::Quarantined);
    }

    #[test]
    fn second_run_ignores_already_decided() {
        let tid = TribeId::from_u16(0x0001);
        let minter = KakiMinter::new(tid);
        let (mut store, _) = make_store_with_valid_particle(&minter);
        let mut jnl = Journal::new(64);
        let mut sweep = make_sweep();

        sweep.run(&mut store, &mut jnl, &minter, tid, 10);
        let second = sweep.run(&mut store, &mut jnl, &minter, tid, 20);
        assert_eq!(second.promoted + second.quarantined, 0);
    }
}
