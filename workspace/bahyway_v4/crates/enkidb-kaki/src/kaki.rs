//! The Kaki struct — a 16-byte sealed sovereign identity.
//!
//! Byte layout (KAKI_v4.0.pdf §1.2):
//!   κ[0..4]   minted_id   — numeric ID minted at creation; use uuid_hash() as firewall key
//!   κ[4..6]   tribe_id    — PA-15 sovereignty (2 bytes, big-endian)
//!   κ[6]      kaki_type   — 0x01 Identity / 0x02 Event / 0x03 CrossTribe
//!                           (`types.rs` also defines 0x04 Pattern for NISABA
//!                           GA-cluster-derived KAKIs — not in the canonical
//!                           PDF's byte-layout table; corrected 2026-07-07,
//!                           needs Architect ruling on whether it's canonical
//!                           or an intentionally out-of-band extension)
//!   κ[7]      kaki_role   — 0x01 KISHIB / 0x02 ZIKRU / 0x03 PARZU
//!   κ[8..12]  reserved    — zeroed; future structural markers
//!   κ[12..14] timestamp   — birth timestamp (u16 big-endian, high byte → azimuth PA-14)
//!   κ[14..16] checksum    — CRC-16/CCITT over κ[0..14]
//!
//! Immutability rules (§2):
//!   Rule I   — byte values never modified
//!   Rule II  — never reassigned to a different particle
//!   Rule III — only held via Copy or shared &Kaki; no &mut Kaki exists
//!   Rule IV  — no assessment data (state/quality/color) in these bytes

use core::fmt;

use bahyway_core::{BahywayError, Result, TribeId};
use bahyway_crc::crc16;

use crate::types::{KakiRole, KakiType};

/// A 16-byte sealed KAKI.  `Copy` is intentional: pass by value, never by &mut.
/// No public constructor except `Kaki::from_bytes` (deserialization / storage read)
/// and `Kaki::mint` (only accessible inside this crate via the KakiMinter gate).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Kaki {
    bytes: [u8; 16],
}

impl Kaki {
    // ── Construction ────────────────────────────────────────────────────

    /// Mint a new KAKI.  Crate-internal only — callers use `KakiMinter`.
    pub(crate) fn mint(
        uuid_hash: u32,
        tribe_id: TribeId,
        kaki_type: KakiType,
        kaki_role: KakiRole,
        timestamp: u16,
    ) -> Self {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&uuid_hash.to_be_bytes());
        bytes[4..6].copy_from_slice(&tribe_id.to_be_bytes());
        bytes[6] = kaki_type as u8;
        bytes[7] = kaki_role as u8;
        // κ[8..12] reserved — remain zeroed
        bytes[12..14].copy_from_slice(&timestamp.to_be_bytes());
        let cs = crc16(&bytes[0..14]);
        bytes[14..16].copy_from_slice(&cs.to_be_bytes());
        Kaki { bytes }
    }

    /// Reconstruct a `Kaki` from raw bytes (storage read path).
    /// Validates CRC and enum discriminants before returning.
    pub fn from_bytes(raw: [u8; 16]) -> Result<Self> {
        // Validate structural bytes first (fast-fail before CRC)
        KakiType::from_byte(raw[6])?;
        KakiRole::from_byte(raw[7])?;

        // CRC check over κ[0..14]
        let stored = u16::from_be_bytes([raw[14], raw[15]]);
        let computed = crc16(&raw[0..14]);
        if stored != computed {
            return Err(BahywayError::ChecksumMismatch {
                expected: computed,
                actual: stored,
            });
        }
        Ok(Kaki { bytes: raw })
    }

    // ── Accessors (read-only) ────────────────────────────────────────────

    /// The raw 16 bytes.  No mutable accessor exists.
    #[inline]
    pub fn bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// κ[0..4] — the numeric ID minted into the KAKI at creation.
    /// Used for byte-layout inspection only; do NOT use as a firewall key.
    #[inline]
    pub fn minted_id(&self) -> u32 {
        u32::from_be_bytes(self.bytes[0..4].try_into().unwrap())
    }

    /// FNV-1a hash over all 16 KAKI bytes — the firewall identity key.
    ///
    /// Uses all bytes (including tribe_id, type, role, timestamp, checksum) so
    /// two KAKIs with the same minted numeric ID but different tribe/type/role
    /// produce different hashes.  This prevents cross-tribe firewall impersonation.
    #[inline]
    pub fn uuid_hash(&self) -> u32 {
        const OFFSET: u32 = 2_166_136_261;
        const PRIME: u32 = 16_777_619;
        let mut h = OFFSET;
        for b in &self.bytes {
            h ^= *b as u32;
            h = h.wrapping_mul(PRIME);
        }
        h
    }

    /// κ[4..6] — tribe sovereign membership (PA-15).
    #[inline]
    pub fn tribe_id(&self) -> TribeId {
        TribeId::from_be_bytes([self.bytes[4], self.bytes[5]])
    }

    /// κ[6] — physical KAKI type.
    #[inline]
    pub fn kaki_type(&self) -> KakiType {
        KakiType::from_byte(self.bytes[6]).expect("validated at construction")
    }

    /// κ[7] — logical KAKI role.
    #[inline]
    pub fn kaki_role(&self) -> KakiRole {
        KakiRole::from_byte(self.bytes[7]).expect("validated at construction")
    }

    /// κ[8..12] — reserved bytes (must be zero in v4.0).
    #[inline]
    pub fn reserved(&self) -> [u8; 4] {
        self.bytes[8..12].try_into().unwrap()
    }

    /// κ[12..14] — birth timestamp (u16 big-endian).
    /// High byte κ[12] → azimuth in orbital space (PA-14).
    #[inline]
    pub fn timestamp(&self) -> u16 {
        u16::from_be_bytes([self.bytes[12], self.bytes[13]])
    }

    /// κ[14..16] — CRC-16/CCITT structural integrity checksum.
    /// High byte κ[14] → altitude in orbital space (PA-14).
    #[inline]
    pub fn checksum(&self) -> u16 {
        u16::from_be_bytes([self.bytes[14], self.bytes[15]])
    }

    /// Verify that the checksum bytes still match — sanity check after storage read.
    #[inline]
    pub fn verify_checksum(&self) -> bool {
        bahyway_crc::verify(&self.bytes[0..14], self.checksum())
    }

    // ── PA-14 Position Fixing ────────────────────────────────────────────

    /// PA-14 azimuth in orbital space — high byte of the birth timestamp.
    /// Angular position is immutable because the timestamp bytes are immutable.
    #[inline]
    pub fn azimuth(&self) -> u8 {
        self.bytes[12]
    }

    /// PA-14 altitude in orbital space — high byte of the checksum.
    #[inline]
    pub fn altitude(&self) -> u8 {
        self.bytes[14]
    }
}

// ── Formatting ────────────────────────────────────────────────────────────

impl fmt::Debug for Kaki {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Kaki {{ {}/{} tribe:{:#06x} ts:{:#06x} cs:{:#06x} }}",
            self.kaki_type(),
            self.kaki_role(),
            self.tribe_id().as_u16(),
            self.timestamp(),
            self.checksum(),
        )
    }
}

impl fmt::Display for Kaki {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Hex representation of all 16 bytes grouped as 4-2-2-8
        let b = &self.bytes;
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3],
            b[4], b[5],
            b[6], b[7],
            b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity_kaki() -> Kaki {
        Kaki::mint(
            0xDEAD_BEEF,
            TribeId::from_u16(0x0001),
            KakiType::Identity,
            KakiRole::Zikru,
            0x0100,
        )
    }

    #[test]
    fn byte_layout() {
        let k = make_identity_kaki();
        let b = k.bytes();
        assert_eq!(&b[0..4], &0xDEAD_BEEFu32.to_be_bytes());
        assert_eq!(&b[4..6], &0x0001u16.to_be_bytes());
        assert_eq!(b[6], 0x01); // Identity
        assert_eq!(b[7], 0x02); // Zikru
        assert_eq!(&b[8..12], &[0u8; 4]); // reserved zeroed
        assert_eq!(&b[12..14], &0x0100u16.to_be_bytes());
    }

    #[test]
    fn checksum_valid() {
        let k = make_identity_kaki();
        assert!(k.verify_checksum());
    }

    #[test]
    fn from_bytes_round_trip() {
        let original = make_identity_kaki();
        let k2 = Kaki::from_bytes(*original.bytes()).unwrap();
        assert_eq!(original, k2);
    }

    #[test]
    fn from_bytes_rejects_bad_checksum() {
        let mut raw = *make_identity_kaki().bytes();
        raw[15] ^= 0xFF; // corrupt checksum
        assert!(Kaki::from_bytes(raw).is_err());
    }

    #[test]
    fn from_bytes_rejects_invalid_type() {
        let mut raw = *make_identity_kaki().bytes();
        raw[6] = 0x00; // invalid kaki_type
                       // Recompute checksum so it passes that check
        let cs = bahyway_crc::crc16(&raw[0..14]);
        raw[14..16].copy_from_slice(&cs.to_be_bytes());
        assert!(Kaki::from_bytes(raw).is_err());
    }

    #[test]
    fn pa14_position() {
        let k = make_identity_kaki();
        assert_eq!(k.azimuth(), k.bytes()[12]);
        assert_eq!(k.altitude(), k.bytes()[14]);
    }

    #[test]
    fn copy_semantics_no_mut_ref() {
        let k1 = make_identity_kaki();
        let k2 = k1; // Copy, no &mut
        assert_eq!(k1, k2);
        // k1 still usable — Copy, not Move
        let _ = k1.uuid_hash();
        let _ = k1.minted_id();
    }

    #[test]
    fn uuid_hash_covers_all_bytes() {
        // Two KAKIs with the same numeric ID but different tribe_id must
        // produce different uuid_hash values (cross-tribe impersonation prevention).
        let k1 = Kaki::mint(
            0xDEAD_BEEF,
            TribeId::from_u16(0x0001),
            KakiType::Identity,
            KakiRole::Zikru,
            0x0100,
        );
        let k2 = Kaki::mint(
            0xDEAD_BEEF,
            TribeId::from_u16(0x0002),
            KakiType::Identity,
            KakiRole::Zikru,
            0x0100,
        );
        assert_ne!(k1.uuid_hash(), k2.uuid_hash());
        // minted_id() is the same for both (same numeric ID was minted)
        assert_eq!(k1.minted_id(), k2.minted_id());
    }

    #[test]
    fn display_is_hex_string() {
        let k = make_identity_kaki();
        let s = k.to_string();
        assert_eq!(s.len(), 32 + 3); // 32 hex chars + 3 dashes (8-4-4-16 format)
    }
}
