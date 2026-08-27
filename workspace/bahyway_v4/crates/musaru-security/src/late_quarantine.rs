//! Musarû Late Quarantine — post-extraction discovery path.
//!
//! When a malware ZIP is discovered AFTER its contents have already been
//! extracted and staged in the SDB, `emit_late_quarantine` marks all
//! affected particles in the Journal with EventCause::MusaruLateQuarantine.
//!
//! The caller (SDB scheduler or admin command) then calls
//! `enkisdb::SdbStore::quarantine()` for each affected particle to ensure
//! the in-memory state matches the Journal record.

use enkidb_journal::{EventCause, Journal, JournalEntry};
use enkidb_kaki::{EventKaki, IdentityKaki, Kaki, KakiMinter, KakiRole};

/// One affected particle in a late-quarantine event.
#[derive(Debug, Clone)]
pub struct AffectedParticle {
    pub kaki_bytes: [u8; 16],
    pub epoch: u32,
    /// Color at time of late discovery (may already be degraded).
    pub color_rgb: [u8; 3],
}

/// Summary of a late quarantine emission.
#[derive(Debug, Default, Clone)]
pub struct LateQuarantineReport {
    /// Number of particles successfully journalled.
    pub journalled: usize,
    /// Number of particles whose KAKI bytes could not be reconstructed.
    pub invalid_kaki: usize,
}

/// Emit `MusaruLateQuarantine` JournalEntries for all affected particles.
///
/// `source_file` — the filename of the offending ZIP (for human audit).
/// `current_tick` — current scheduler tick (informational only; not stored
///                  in wire format, only used for color drift encoding).
pub fn emit_late_quarantine(
    affected: &[AffectedParticle],
    journal: &mut Journal,
    minter: &KakiMinter,
    source_file: &str,
) -> LateQuarantineReport {
    let _ = source_file; // reserved for future audit trail EAV
    let mut report = LateQuarantineReport::default();

    for p in affected {
        let target_kaki = match Kaki::from_bytes(p.kaki_bytes)
            .ok()
            .and_then(|k| IdentityKaki::try_from_kaki(k).ok())
        {
            Some(ik) => ik,
            None => {
                report.invalid_kaki += 1;
                continue;
            }
        };

        let event_kaki = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru))
            .expect("KakiMinter always produces valid event KAKIs");

        // drift=1.0 signals terminal degradation; color set to [255,0,0] (max R = alert).
        let entry = JournalEntry::new(event_kaki, target_kaki, p.epoch, Vec::new())
            .with_color_snapshot([255, 0, 0], 1.0)
            .with_event_cause(EventCause::MusaruLateQuarantine);

        if journal.append(entry).is_ok() {
            report.journalled += 1;
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_journal::Journal;
    use enkidb_kaki::KakiMinter;

    fn make_minter() -> KakiMinter {
        KakiMinter::new(TribeId::from_u16(0x0001))
    }

    fn valid_particle(minter: &KakiMinter) -> AffectedParticle {
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        AffectedParticle {
            kaki_bytes: *ik.bytes(),
            epoch: 1,
            color_rgb: [100, 200, 255],
        }
    }

    #[test]
    fn three_particles_journalled() {
        let minter = make_minter();
        let mut jnl = Journal::new(64);
        let particles: Vec<AffectedParticle> = (0..3).map(|_| valid_particle(&minter)).collect();

        let r = emit_late_quarantine(&particles, &mut jnl, &minter, "suspect.zip");

        assert_eq!(r.journalled, 3);
        assert_eq!(r.invalid_kaki, 0);
        assert_eq!(jnl.entry_count(), 3);
    }

    #[test]
    fn invalid_kaki_bytes_counted() {
        let minter = make_minter();
        let mut jnl = Journal::new(64);
        let bad = AffectedParticle {
            kaki_bytes: [0xFF; 16], // deliberately invalid
            epoch: 1,
            color_rgb: [0, 0, 0],
        };
        let good = valid_particle(&minter);

        let r = emit_late_quarantine(&[bad, good], &mut jnl, &minter, "test.zip");

        assert_eq!(r.invalid_kaki, 1);
        assert_eq!(r.journalled, 1);
    }

    #[test]
    fn journal_entry_has_late_quarantine_cause() {
        let minter = make_minter();
        let ik =
            enkidb_kaki::IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        let mut jnl = Journal::new(64);
        let p = AffectedParticle {
            kaki_bytes: *ik.bytes(),
            epoch: 1,
            color_rgb: [50, 100, 200],
        };

        emit_late_quarantine(&[p.clone()], &mut jnl, &minter, "test.zip");

        let history = jnl.read_particle_history(&ik);
        assert_eq!(history.len(), 1);
        assert_eq!(
            history[0].event_cause(),
            Some(EventCause::MusaruLateQuarantine)
        );
        assert_eq!(history[0].color_id_snapshot(), Some([255, 0, 0]));
    }

    #[test]
    fn empty_list_is_noop() {
        let minter = make_minter();
        let mut jnl = Journal::new(64);
        let r = emit_late_quarantine(&[], &mut jnl, &minter, "empty.zip");
        assert_eq!(r.journalled, 0);
        assert_eq!(jnl.entry_count(), 0);
    }
}
