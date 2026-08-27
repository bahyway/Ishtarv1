use crate::types::{Hepta, KakiPK};

// ── KAKI derivation ───────────────────────────────────────────────

/// Encode a Hepta coordinate into a 16-byte KAKI PK (Particles Algebra §4.1).
///
/// Each Hepta dimension is quantised into its byte-width slot.
/// The original NuskuWay v354 blake3 pass was immediately overwritten by this
/// Hepta encoding, making it a no-op — we omit it here (pure Rust, no deps).
pub fn derive_kaki(hepta: &Hepta, _zone_id: u8, _frame_id: u64) -> KakiPK {
    let mut kaki = [0u8; 16];
    kaki[0..4].copy_from_slice(&((hepta.d1_identity * u32::MAX as f32) as u32).to_le_bytes());
    kaki[4..6].copy_from_slice(&((hepta.d2_belonging * u16::MAX as f32) as u16).to_le_bytes());
    kaki[6..8].copy_from_slice(&((hepta.d3_domain * u16::MAX as f32) as u16).to_le_bytes());
    kaki[8..10].copy_from_slice(&((hepta.d4_quality * u16::MAX as f32) as u16).to_le_bytes());
    kaki[10..12].copy_from_slice(&((hepta.d5_freshness * u16::MAX as f32) as u16).to_le_bytes());
    kaki[12..14].copy_from_slice(&((hepta.d6_temporal * u16::MAX as f32) as u16).to_le_bytes());
    kaki[14..16].copy_from_slice(&((hepta.d7_integrity * u16::MAX as f32) as u16).to_le_bytes());
    kaki
}

/// Derive the D1-identity component for a zone/frame pair.
///
/// Pure-Rust avalanche hash using golden-ratio fractional constants;
/// replaces blake3 (external dep) from NuskuWay v354.
pub(crate) fn kaki_d1_hash(zone_id: u8, frame_id: u64) -> f32 {
    const PHI: u32 = 0x9e37_79b9; // golden ratio fractional bits (Knuth)
    const C: u32 = 0x6c62_272e; // Murmur3-style secondary constant
    let mut v = (zone_id as u32).wrapping_mul(PHI) ^ C;
    v ^= ((frame_id & 0xffff_ffff) as u32).wrapping_mul(PHI);
    v ^= ((frame_id >> 32) as u32).wrapping_mul(C);
    // Final avalanche (Wang hash)
    v ^= v >> 16;
    v = v.wrapping_mul(0x45d9_f3b7);
    v ^= v >> 16;
    v as f32 / u32::MAX as f32
}

/// Format a KakiPK as a 32-char uppercase hex string for audit logs.
pub fn kaki_to_hex(kaki: &KakiPK) -> String {
    kaki.iter().fold(String::with_capacity(32), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02X}");
        s
    })
}

// ── Tests ─────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Hepta;

    fn unit_hepta() -> Hepta {
        Hepta {
            d1_identity: 1.0,
            d2_belonging: 1.0,
            d3_domain: 1.0,
            d4_quality: 1.0,
            d5_freshness: 1.0,
            d6_temporal: 1.0,
            d7_integrity: 1.0,
        }
    }

    #[test]
    fn derive_kaki_produces_16_bytes() {
        let h = unit_hepta();
        let k = derive_kaki(&h, 0, 0);
        assert_eq!(k.len(), 16);
    }

    #[test]
    fn different_heptas_produce_different_kakis() {
        let h1 = unit_hepta();
        let h2 = Hepta {
            d1_identity: 0.5,
            ..h1
        };
        assert_ne!(derive_kaki(&h1, 0, 0), derive_kaki(&h2, 0, 0));
    }

    #[test]
    fn kaki_to_hex_is_uppercase_32_chars() {
        let kaki = [0xabu8; 16];
        let hex = kaki_to_hex(&kaki);
        assert_eq!(hex.len(), 32);
        assert_eq!(&hex[..4], "ABAB");
    }

    #[test]
    fn d1_hash_different_zones_differ() {
        let a = kaki_d1_hash(0, 42);
        let b = kaki_d1_hash(1, 42);
        assert!(
            (a - b).abs() > 1e-5,
            "zone 0 and zone 1 should produce different d1"
        );
    }

    #[test]
    fn d1_hash_in_range() {
        for z in 0u8..16 {
            let v = kaki_d1_hash(z, 1000);
            assert!(
                v >= 0.0 && v <= 1.0,
                "d1_hash out of range for zone {z}: {v}"
            );
        }
    }
}
