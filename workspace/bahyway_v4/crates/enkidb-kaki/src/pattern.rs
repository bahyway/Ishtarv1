#![forbid(unsafe_code)]
//! Pattern-KAKI derivation — deterministic minting for GA-cluster patterns.
//!
//! A Pattern-KAKI is NOT minted randomly.  It is computed deterministically
//! from the cluster geometry so the same pattern discovered independently by
//! two NISABA nodes produces identical bytes.  This is the canonical
//! deduplication key for the ENKI-PATTERN registry.
//!
//! Derivation inputs (§PATTERN-KAKI-001):
//!   - `pattern_type`            — PatternType discriminant (1 byte)
//!   - `ga_centroid`             — 7D fixed-point centroid (56 bytes)
//!   - `constituent_merkle_root` — Merkle root over sorted constituent KAKIs (32 bytes)
//!   - `confidence`              — GA confidence as u16 millipercent (0..=10_000)
//!   - `orbital_formed`          — BASE orbital counter at emergence (u64)
//!
//! The derivation uses FNV-1a mixing (no external crates, no_std compatible).

use crate::{Kaki, KakiRole, KakiType};
use bahyway_core::BahywayError;
use bahyway_crc::crc16;

/// 7-dimensional fixed-point coordinate.
/// Each axis is stored as i32 in millimetre units (i32::MAX ≈ 2 147 km).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FixedCoord7D {
    pub d: [i32; 7],
}

impl FixedCoord7D {
    pub fn zero() -> Self {
        Self { d: [0; 7] }
    }

    /// Serialize to 28 bytes (7 × i32 big-endian).
    pub fn to_bytes(self) -> [u8; 28] {
        let mut out = [0u8; 28];
        for (i, v) in self.d.iter().enumerate() {
            out[i * 4..(i + 1) * 4].copy_from_slice(&v.to_be_bytes());
        }
        out
    }
}

/// Pattern type discriminant — determines the physical/semantic domain of the pattern.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PatternType {
    /// Crowd flow corridor detected in outdoor space.
    CrowdFlow = 0x01,
    /// Aviation corridor (en-route airspace).
    AviationCorridor = 0x02,
    /// Aviation holding pattern.
    AviationHolding = 0x03,
    /// Indoor hallway / passage.
    IndoorHallway = 0x04,
    /// Indoor room / enclosed space.
    IndoorRoom = 0x05,
    /// Indoor transition zone (stairs, lifts, doorways).
    IndoorTransition = 0x06,
    /// Water flow channel (rivers, canals, drainage).
    WaterFlow = 0x07,
    /// Custom application-defined pattern.
    Custom = 0xFF,
}

impl PatternType {
    pub fn from_byte(b: u8) -> Result<Self, BahywayError> {
        match b {
            0x01 => Ok(PatternType::CrowdFlow),
            0x02 => Ok(PatternType::AviationCorridor),
            0x03 => Ok(PatternType::AviationHolding),
            0x04 => Ok(PatternType::IndoorHallway),
            0x05 => Ok(PatternType::IndoorRoom),
            0x06 => Ok(PatternType::IndoorTransition),
            0x07 => Ok(PatternType::WaterFlow),
            0xFF => Ok(PatternType::Custom),
            other => Err(BahywayError::InvalidKakiType(other)),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PatternType::CrowdFlow => "CrowdFlow",
            PatternType::AviationCorridor => "AviationCorridor",
            PatternType::AviationHolding => "AviationHolding",
            PatternType::IndoorHallway => "IndoorHallway",
            PatternType::IndoorRoom => "IndoorRoom",
            PatternType::IndoorTransition => "IndoorTransition",
            PatternType::WaterFlow => "WaterFlow",
            PatternType::Custom => "Custom",
        }
    }
}

impl core::fmt::Display for PatternType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lifecycle state of a pattern in the ENKI-PATTERN registry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PatternLifecycle {
    /// GA confidence ≥ 0.60 but < 0.85. Observed, not yet stable.
    Emerging = 0x01,
    /// GA confidence ≥ 0.85. Reproducible across multiple orbitals.
    Stable = 0x02,
    /// Approved by AICouncil + Šàtamu. Authoritative routing reference.
    Canonical = 0x03,
    /// Superseded by a newer pattern. No longer used for routing.
    Deprecated = 0x04,
}

/// Derive a deterministic Pattern-KAKI from GA-cluster inputs.
///
/// The output is reproducible: given the same inputs, any NISABA node
/// on any machine will produce exactly the same 16 bytes.
///
/// # Arguments
/// * `pattern_type`            — domain of the emergent pattern
/// * `ga_centroid`             — 7D cluster centroid in fixed-point millimetres
/// * `constituent_merkle_root` — Merkle root over the sorted constituent KAKIs
/// * `confidence`              — GA confidence in millipercent (0..=10_000 → 0.00%..100.00%)
/// * `orbital_formed`          — BASE orbital counter at the moment of emergence
pub fn derive_pattern_kaki(
    pattern_type: PatternType,
    ga_centroid: FixedCoord7D,
    constituent_merkle_root: [u8; 32],
    confidence_millipercent: u16,
    orbital_formed: u64,
) -> Kaki {
    // ── Mix all inputs into 16 bytes using FNV-1a ────────────────────────
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;

    let mut h = FNV_OFFSET;

    // Feed pattern_type discriminant
    h ^= pattern_type as u64;
    h = h.wrapping_mul(FNV_PRIME);

    // Feed 7D centroid bytes
    for byte in ga_centroid.to_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }

    // Feed Merkle root
    for byte in constituent_merkle_root {
        h ^= byte as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }

    // Feed confidence
    for byte in confidence_millipercent.to_be_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }

    // Feed orbital_formed
    for byte in orbital_formed.to_be_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }

    // ── Build the 16-byte KAKI layout ───────────────────────────────────
    // κ[0..4]  — high 32 bits of the FNV hash (serves as minted_id)
    // κ[4..6]  — tribe_id = 0x0000 (patterns are cross-tribe by nature)
    // κ[6]     — kaki_type = 0x04 (Pattern)
    // κ[7]     — kaki_role = 0x03 (Parzu — pattern is a rule/structure)
    // κ[8..12] — low 32 bits of the FNV hash
    // κ[12..14]— confidence_millipercent (big-endian, auditable in raw bytes)
    // κ[14..16]— CRC-16/CCITT over κ[0..14]

    let mut bytes = [0u8; 16];
    let h_hi = (h >> 32) as u32;
    let h_lo = h as u32;

    bytes[0..4].copy_from_slice(&h_hi.to_be_bytes());
    bytes[4..6].copy_from_slice(&[0x00, 0x00]); // cross-tribe: no single tribe_id
    bytes[6] = KakiType::Pattern as u8; // 0x04
    bytes[7] = KakiRole::Parzu as u8; // 0x03
    bytes[8..12].copy_from_slice(&h_lo.to_be_bytes());
    bytes[12..14].copy_from_slice(&confidence_millipercent.to_be_bytes());
    let cs = crc16(&bytes[0..14]);
    bytes[14..16].copy_from_slice(&cs.to_be_bytes());

    // Safety: we set valid KakiType (0x04) and KakiRole (0x03) above,
    // and computed the CRC correctly — from_bytes will not fail.
    Kaki::from_bytes(bytes).expect("derive_pattern_kaki: internal layout error")
}

/// Extract the confidence (in millipercent) encoded directly in κ[12..14].
/// This allows quick inspection without re-deriving the full KAKI.
pub fn pattern_kaki_confidence(kaki: &Kaki) -> Option<u16> {
    if kaki.kaki_type() != KakiType::Pattern {
        return None;
    }
    Some(u16::from_be_bytes([kaki.bytes()[12], kaki.bytes()[13]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_centroid() -> FixedCoord7D {
        FixedCoord7D {
            d: [1_000, 2_000, 3_000, 4_000, 5_000, 6_000, 7_000],
        }
    }

    fn sample_merkle() -> [u8; 32] {
        let mut m = [0u8; 32];
        for (i, b) in m.iter_mut().enumerate() {
            *b = i as u8;
        }
        m
    }

    #[test]
    fn deterministic_derivation() {
        let k1 = derive_pattern_kaki(
            PatternType::CrowdFlow,
            sample_centroid(),
            sample_merkle(),
            8_500,
            42,
        );
        let k2 = derive_pattern_kaki(
            PatternType::CrowdFlow,
            sample_centroid(),
            sample_merkle(),
            8_500,
            42,
        );
        assert_eq!(k1, k2, "same inputs must produce identical KAKI bytes");
    }

    #[test]
    fn different_type_different_kaki() {
        let k1 = derive_pattern_kaki(
            PatternType::CrowdFlow,
            sample_centroid(),
            sample_merkle(),
            8_500,
            42,
        );
        let k2 = derive_pattern_kaki(
            PatternType::IndoorHallway,
            sample_centroid(),
            sample_merkle(),
            8_500,
            42,
        );
        assert_ne!(k1, k2);
    }

    #[test]
    fn kaki_type_is_pattern() {
        let k = derive_pattern_kaki(
            PatternType::CrowdFlow,
            sample_centroid(),
            sample_merkle(),
            8_500,
            42,
        );
        assert_eq!(k.kaki_type(), KakiType::Pattern);
    }

    #[test]
    fn checksum_valid() {
        let k = derive_pattern_kaki(
            PatternType::CrowdFlow,
            sample_centroid(),
            sample_merkle(),
            8_500,
            42,
        );
        assert!(k.verify_checksum());
    }

    #[test]
    fn confidence_readable() {
        let k = derive_pattern_kaki(
            PatternType::CrowdFlow,
            sample_centroid(),
            sample_merkle(),
            7_250,
            99,
        );
        assert_eq!(pattern_kaki_confidence(&k), Some(7_250));
    }

    #[test]
    fn confidence_none_for_non_pattern() {
        use crate::mint::KakiMinter;
        use bahyway_core::TribeId;
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let k = m.identity(KakiRole::Zikru);
        assert_eq!(pattern_kaki_confidence(&k), None);
    }

    #[test]
    fn pattern_type_round_trip() {
        for &pt in &[
            PatternType::CrowdFlow,
            PatternType::AviationCorridor,
            PatternType::AviationHolding,
            PatternType::IndoorHallway,
            PatternType::IndoorRoom,
            PatternType::IndoorTransition,
            PatternType::WaterFlow,
            PatternType::Custom,
        ] {
            assert_eq!(PatternType::from_byte(pt as u8).unwrap(), pt);
        }
    }
}
