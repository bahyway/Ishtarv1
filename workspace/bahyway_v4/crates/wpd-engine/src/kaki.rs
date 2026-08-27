// ── WPD KAKI derivation ───────────────────────────────────────────
// Byte layout (mirrors nusku-engine §4.1 but WPD-specific dimensions):
//   D1 [0-3]  = segment identity avalanche hash
//   D2 [4-5]  = domain = WPDWAY_DOMAIN / 0xFF
//   D3 [6-7]  = composite_score (anomaly)
//   D4 [8-9]  = confidence (quality)
//   D5 [10-11]= timestamp freshness (normalised epoch)
//   D6 [12-13]= epoch high-word (temporal)
//   D7 [14-15]= integrity checksum fold

use nusku_engine::KakiPK;

/// WPD domain byte in the KAKI system.
pub const WPDWAY_DOMAIN: u8 = 0x03;

const PHI: u32 = 0x9e37_79b9; // golden-ratio fractional (Knuth)
const C: u32 = 0x6c62_272e; // Murmur3-style secondary constant

/// Avalanche hash of a UTF-8 segment ID string for KAKI D1 (identity).
fn segment_id_hash(segment_id: &str) -> f32 {
    let mut v: u32 = PHI;
    for (i, b) in segment_id.bytes().enumerate() {
        v = v.wrapping_add((b as u32).wrapping_mul(PHI));
        v ^= v >> 16;
        v = v.wrapping_mul(C.wrapping_add(i as u32 + 1));
    }
    // Wang hash finalisation
    v ^= v >> 16;
    v = v.wrapping_mul(0x45d9_f3b7);
    v ^= v >> 16;
    v as f32 / u32::MAX as f32
}

/// Derive a 16-byte WPD KAKI PK for a defect event.
pub fn derive_wpd_kaki(
    segment_id: &str,
    composite_score: f32,
    confidence: f32,
    timestamp_ms: u64,
) -> KakiPK {
    let d1 = segment_id_hash(segment_id);
    let d2 = WPDWAY_DOMAIN as f32 / 255.0;
    let d3 = composite_score.clamp(0.0, 1.0);
    let d4 = confidence.clamp(0.0, 1.0);
    // D5: normalised against year-2100 epoch in seconds
    let ts_secs = (timestamp_ms / 1000) as f32;
    let d5 = (ts_secs / 4_102_444_800.0_f32).clamp(0.0, 1.0);
    // D6: high 32 bits of timestamp_ms normalised
    let d6 = ((timestamp_ms >> 32) as f32 / u32::MAX as f32).clamp(0.0, 1.0);
    // D7: integrity fold of identity, anomaly, quality and freshness
    let d7 = ((d1 + d3 + d4 + d5) / 4.0).clamp(0.0, 1.0);

    let mut kaki = [0u8; 16];
    kaki[0..4].copy_from_slice(&((d1 * u32::MAX as f32) as u32).to_le_bytes());
    kaki[4..6].copy_from_slice(&((d2 * u16::MAX as f32) as u16).to_le_bytes());
    kaki[6..8].copy_from_slice(&((d3 * u16::MAX as f32) as u16).to_le_bytes());
    kaki[8..10].copy_from_slice(&((d4 * u16::MAX as f32) as u16).to_le_bytes());
    kaki[10..12].copy_from_slice(&((d5 * u16::MAX as f32) as u16).to_le_bytes());
    kaki[12..14].copy_from_slice(&((d6 * u16::MAX as f32) as u16).to_le_bytes());
    kaki[14..16].copy_from_slice(&((d7 * u16::MAX as f32) as u16).to_le_bytes());
    kaki
}

#[cfg(test)]
mod tests {
    use super::*;
    use nusku_engine::kaki_to_hex;

    #[test]
    fn kaki_is_16_bytes() {
        let k = derive_wpd_kaki("BGH-SC-001", 0.78, 0.85, 1_717_000_000_000);
        assert_eq!(k.len(), 16);
    }

    #[test]
    fn different_segments_produce_different_kakis() {
        let k1 = derive_wpd_kaki("BGH-SC-001", 0.78, 0.85, 1_717_000_000_000);
        let k2 = derive_wpd_kaki("BGH-GZ-001", 0.78, 0.85, 1_717_000_000_000);
        assert_ne!(k1, k2);
    }

    #[test]
    fn domain_byte_is_wpd() {
        let k = derive_wpd_kaki("X", 0.0, 1.0, 0);
        let d2 = u16::from_le_bytes([k[4], k[5]]);
        let expected = (WPDWAY_DOMAIN as f32 / 255.0 * u16::MAX as f32) as u16;
        assert_eq!(d2, expected);
    }

    #[test]
    fn hex_is_32_char_uppercase() {
        let k = derive_wpd_kaki("BGH-KZ-001", 0.45, 0.90, 1_717_000_000_000);
        let hex = kaki_to_hex(&k);
        assert_eq!(hex.len(), 32);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }
}
