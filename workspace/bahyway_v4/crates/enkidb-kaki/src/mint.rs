//! KakiMinter — the gated constructor for new KAKIs.
//!
//! Pure std — no uuid, no rand.  Uniqueness comes from mixing SystemTime
//! nanoseconds with a per-minter counter; collision probability over the
//! lifetime of any single deployment is astronomically small.

use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Kaki, KakiRole, KakiType};
use bahyway_core::TribeId;

/// Mints new KAKIs for a single tribe.
pub struct KakiMinter {
    tribe_id: TribeId,
    counter: Cell<u32>,
}

impl KakiMinter {
    pub fn new(tribe_id: TribeId) -> Self {
        KakiMinter {
            tribe_id,
            counter: Cell::new(0),
        }
    }

    /// Mint an Identity-Kaki (kaki_type = 0x01) with an auto-generated uuid_hash.
    pub fn identity(&self, role: KakiRole) -> Kaki {
        Kaki::mint(
            self.new_uuid_hash(),
            self.tribe_id,
            KakiType::Identity,
            role,
            self.timestamp(),
        )
    }

    /// Mint an Event-Kaki (kaki_type = 0x02) with an auto-generated uuid_hash.
    pub fn event(&self, role: KakiRole) -> Kaki {
        Kaki::mint(
            self.new_uuid_hash(),
            self.tribe_id,
            KakiType::Event,
            role,
            self.timestamp(),
        )
    }

    /// Mint a CrossTribe-Kaki (kaki_type = 0x03) with an auto-generated uuid_hash.
    pub fn crosstribe(&self, role: KakiRole) -> Kaki {
        Kaki::mint(
            self.new_uuid_hash(),
            self.tribe_id,
            KakiType::CrossTribe,
            role,
            self.timestamp(),
        )
    }

    /// Mint an Identity-Kaki with a caller-supplied uuid_hash (deterministic).
    pub fn mint_identity(&self, uuid_hash: u32, role: KakiRole) -> Kaki {
        Kaki::mint(
            uuid_hash,
            self.tribe_id,
            KakiType::Identity,
            role,
            self.timestamp(),
        )
    }

    /// Mint an Event-Kaki with a caller-supplied uuid_hash (deterministic).
    pub fn mint_event(&self, uuid_hash: u32, role: KakiRole) -> Kaki {
        Kaki::mint(
            uuid_hash,
            self.tribe_id,
            KakiType::Event,
            role,
            self.timestamp(),
        )
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    /// Generate a unique 32-bit hash by mixing nanosecond time with a counter.
    fn new_uuid_hash(&self) -> u32 {
        let cnt = self.counter.get();
        self.counter.set(cnt.wrapping_add(1));
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        // Mix: FNV-like xor-shift to spread bits
        let mut h = nanos ^ (cnt.wrapping_mul(0x9E37_79B9));
        h ^= h >> 17;
        h = h.wrapping_mul(0x45D9_F3B7);
        h ^= h >> 15;
        h ^= self.tribe_id.as_u16() as u32;
        h
    }

    /// Birth timestamp: seconds since Unix epoch truncated to u16.
    fn timestamp(&self) -> u16 {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        (secs & 0xFFFF) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{KakiRole, KakiType};
    use bahyway_core::TribeId;

    #[test]
    fn identity_kaki_correct_type() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let k = m.identity(KakiRole::Zikru);
        assert_eq!(k.kaki_type(), KakiType::Identity);
        assert_eq!(k.kaki_role(), KakiRole::Zikru);
        assert_eq!(k.tribe_id(), TribeId::from_u16(0x0001));
        assert!(k.verify_checksum());
        assert_eq!(k.reserved(), [0u8; 4]);
    }

    #[test]
    fn event_kaki_correct_type() {
        let m = KakiMinter::new(TribeId::from_u16(0x0002));
        let k = m.event(KakiRole::Kishib);
        assert_eq!(k.kaki_type(), KakiType::Event);
        assert_eq!(k.tribe_id(), TribeId::from_u16(0x0002));
        assert!(k.verify_checksum());
    }

    #[test]
    fn uniqueness_across_mints() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let k1 = m.identity(KakiRole::Zikru);
        let k2 = m.identity(KakiRole::Zikru);
        // counter differs between mints → uuid_hash differs
        assert_ne!(k1.uuid_hash(), k2.uuid_hash());
    }

    #[test]
    fn mint_identity_with_explicit_hash() {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let k = m.mint_identity(0xDEAD_BEEF, KakiRole::Zikru);
        // minted_id() returns what was passed to mint(); uuid_hash() is the
        // firewall identity key (FNV-1a over all 16 bytes — not the raw minted ID).
        assert_eq!(k.minted_id(), 0xDEAD_BEEF);
        assert_ne!(k.uuid_hash(), 0xDEAD_BEEF); // FNV-1a hash ≠ raw minted ID
        assert_eq!(k.kaki_type(), KakiType::Identity);
        assert!(k.verify_checksum());
    }
}
