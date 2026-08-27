//! StorageSector — the hardware-isolated, terminal jail for particles that
//! BlackBox Station has confirmed as harmful after its post-quarantine scan.
//!
//! Flow:
//!   EnkiSDB (ValidationSweep → Quarantined) ──→ BlackBox Station (scan)
//!     ├── confirmed harmful ──→ StorageSector.seal()   (terminal, this crate)
//!     └── fuzzy / unknown   ──→ EnkiQDB.accept()        (pending Steward review)
//!
//! Sealing is a one-way door: this type exposes no method to remove, read
//! back into the pipeline, or requeue a sealed particle.

use enkidb_journal::{EventCause, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki, KakiMinter, KakiRole};
use enkisdb::sdb_store::StagedParticle;

/// One particle permanently sealed in the Storage Sector.
#[derive(Debug, Clone)]
pub struct SealedParticle {
    /// Raw 16-byte KAKI identity bytes.
    pub kaki_bytes: [u8; 16],
    /// Tribe this particle claimed to belong to.
    pub tribe_id: u16,
    /// Epoch at time of original arrival in the SDB.
    pub epoch: u32,
    /// Color_ID(RGB) at time of sealing.
    pub color_rgb: [u8; 3],
    /// Tick when the particle first arrived in the SDB.
    pub arrived_tick: u64,
    /// Tick when the seal decision was made.
    pub sealed_tick: u64,
}

/// Aggregate counts for the sealed archive.
#[derive(Debug, Default, Clone)]
pub struct StorageSectorReport {
    pub total_sealed: usize,
}

/// The hardware-isolated terminal jail.  One per database tribe.
pub struct StorageSector {
    records: Vec<SealedParticle>,
}

impl StorageSector {
    pub fn new() -> Self {
        StorageSector {
            records: Vec::new(),
        }
    }

    /// Seal a confirmed-harmful particle.  Writes a `StorageSectorMove`
    /// JournalEntry so HeptaScript can query the audit trail.  There is no
    /// corresponding "unseal" — this is the terminal state.
    pub fn seal(
        &mut self,
        particle: &StagedParticle,
        journal: &mut Journal,
        minter: &KakiMinter,
        current_tick: u64,
    ) -> usize {
        let record = SealedParticle {
            kaki_bytes: particle.kaki_bytes,
            tribe_id: particle.tribe_id,
            epoch: particle.epoch,
            color_rgb: particle.color_rgb,
            arrived_tick: particle.arrived_tick,
            sealed_tick: current_tick,
        };

        if let Some(target_kaki) = reconstruct_identity(&particle.kaki_bytes) {
            let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
                .expect("KakiMinter always produces valid event KAKIs");
            let entry = JournalEntry::new(event_kaki, target_kaki, particle.epoch, Vec::new())
                .with_color_snapshot(particle.color_rgb, 1.0) // drift=1.0 signals terminal bad state
                .with_event_cause(EventCause::StorageSectorMove);
            let _ = journal.append(entry);
        }

        let idx = self.records.len();
        self.records.push(record);
        idx
    }

    /// Build a summary report.
    pub fn report(&self) -> StorageSectorReport {
        StorageSectorReport {
            total_sealed: self.records.len(),
        }
    }

    pub fn all(&self) -> &[SealedParticle] {
        &self.records
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for StorageSector {
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
    use enkidb_kaki::KakiMinter;
    use enkisdb::sdb_store::SdbStatus;

    fn make_minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0001))
    }

    fn make_staged(minter: &KakiMinter) -> StagedParticle {
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        StagedParticle {
            kaki_bytes: *ik.bytes(),
            tribe_id: 1,
            epoch: 1,
            eav: Vec::new(),
            color_rgb: [255, 0, 0],
            status: SdbStatus::Quarantined,
            arrived_tick: 0,
            malware_flag: true,
        }
    }

    #[test]
    fn seal_writes_journal_entry() {
        let minter = make_minter();
        let mut sector = StorageSector::new();
        let mut jnl = Journal::new(64);
        let particle = make_staged(&minter);

        sector.seal(&particle, &mut jnl, &minter, 900);

        assert_eq!(sector.len(), 1);
        assert_eq!(jnl.entry_count(), 1);
        assert_eq!(sector.all()[0].sealed_tick, 900);
    }

    #[test]
    fn report_counts_all_sealed() {
        let minter = make_minter();
        let mut sector = StorageSector::new();
        let mut jnl = Journal::new(64);

        sector.seal(&make_staged(&minter), &mut jnl, &minter, 1);
        sector.seal(&make_staged(&minter), &mut jnl, &minter, 2);

        assert_eq!(sector.report().total_sealed, 2);
    }

    #[test]
    fn terminal_no_removal() {
        let minter = make_minter();
        let mut sector = StorageSector::new();
        let mut jnl = Journal::new(64);

        sector.seal(&make_staged(&minter), &mut jnl, &minter, 0);
        sector.seal(&make_staged(&minter), &mut jnl, &minter, 1);

        assert_eq!(sector.len(), 2);
        // No method exists on StorageSector to remove or requeue — verified
        // by API surface: only `seal`, `report`, `all`, `len`, `is_empty`.
    }
}
