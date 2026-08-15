//! AdadGate — the sole ingestion entry point (§10.1, PA-15).
//!
//! Only AdadGate holds a KakiMinter. All Identity-Kakis entering the system
//! are created here. The gate assigns tribe sovereignty (PA-15) at birth.

use bahyway_core::{Result, TribeId};
use enkidb_kaki::{IdentityKaki, EventKaki, KakiRole, mint::KakiMinter};
use enkidb_journal::entry::EavTriple;

/// Raw incoming record — (attr_hash, value) pairs supplied by the caller.
pub struct ArrivalRecord {
    pub attrs: Vec<(u32, Vec<u8>)>,
    /// Journal epoch for this event.
    pub epoch: u32,
    /// Kaki role the Identity particle should bear.
    pub role: KakiRole,
}

/// What the gate produces after minting KAKIs for a single arrival.
pub struct GateResult {
    pub particle:   IdentityKaki,
    pub event_kaki: EventKaki,
    pub epoch:      u32,
    pub eav:        Vec<EavTriple>,
}

/// The Adad Gate — owns the KakiMinter, sole KAKI creator.
pub struct AdadGate {
    tribe_id: TribeId,
    minter:   KakiMinter,
}

impl AdadGate {
    /// Construct the gate for a tribe. One gate per tribe (sovereignty).
    pub fn new(tribe_id: TribeId) -> Self {
        AdadGate { tribe_id, minter: KakiMinter::new(tribe_id) }
    }

    pub fn tribe_id(&self) -> TribeId {
        self.tribe_id
    }

    /// Ingest one arrival: mint an IdentityKaki + EventKaki and wrap EAV.
    pub fn ingest(&self, record: ArrivalRecord) -> Result<GateResult> {
        let particle   = IdentityKaki::try_from_kaki(self.minter.identity(record.role))?;
        let event_kaki = EventKaki::try_from_kaki(self.minter.event(record.role))?;
        let eav: Vec<EavTriple> = record.attrs
            .into_iter()
            .map(|(attr, val)| EavTriple::new(attr, val))
            .collect();
        Ok(GateResult { particle, event_kaki, epoch: record.epoch, eav })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;

    #[test]
    fn ingest_creates_valid_kakis() {
        let gate = AdadGate::new(TribeId::from_u16(0x0001));
        let record = ArrivalRecord {
            attrs: vec![(0x1A4B, b"Golden".to_vec())],
            epoch: 42,
            role:  KakiRole::Zikru,
        };
        let result = gate.ingest(record).unwrap();
        assert_eq!(result.epoch, 42);
        assert_eq!(result.eav.len(), 1);
        assert_eq!(result.eav[0].attr_hash, 0x1A4B);
        assert!(result.particle.verify_checksum());
        assert!(result.event_kaki.verify_checksum());
    }

    #[test]
    fn gate_tribe_id_matches() {
        let tid = TribeId::from_u16(0x00FF);
        let gate = AdadGate::new(tid);
        assert_eq!(gate.tribe_id(), tid);
    }

    #[test]
    fn ingest_empty_attrs() {
        let gate = AdadGate::new(TribeId::from_u16(0x0002));
        let record = ArrivalRecord { attrs: vec![], epoch: 1, role: KakiRole::Kishib };
        let result = gate.ingest(record).unwrap();
        assert!(result.eav.is_empty());
    }
}
