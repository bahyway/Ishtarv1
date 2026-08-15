//! bahyway-crc — Native CRC-16/CCITT (pure Rust, zero external deps)
//!
//! Polynomial: 0x1021 (x^16 + x^12 + x^5 + 1), init=0xFFFF, no inversion.
//! Used for KAKI structural integrity checks (κ[14..15]) and block headers.

/// CRC-16/CCITT lookup table — generated at compile time.
const CRC_TABLE: [u16; 256] = {
    let mut table = [0u16; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = (i as u16) << 8;
        let mut j = 0;
        while j < 8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

/// Compute CRC-16/CCITT over `data`.  Returns a big-endian u16.
#[inline]
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        let pos = (((crc >> 8) as u8) ^ byte) as usize;
        crc = (crc << 8) ^ CRC_TABLE[pos];
    }
    crc
}

/// Verify that `crc16(data)` matches the given expected checksum.
#[inline]
pub fn verify(data: &[u8], expected: u16) -> bool {
    crc16(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector_123456789() {
        // CRC-16/CCITT-FALSE: "123456789" → 0x29B1
        let result = crc16(b"123456789");
        assert_eq!(result, 0x29B1, "CRC-16/CCITT failed standard vector");
    }

    #[test]
    fn empty_slice() {
        assert_eq!(crc16(b""), 0xFFFF);
    }

    #[test]
    fn round_trip_verify() {
        let data = b"KAKI-v4.0-sovereignty";
        let cs = crc16(data);
        assert!(verify(data, cs));
        assert!(!verify(data, cs.wrapping_add(1)));
    }

    #[test]
    fn single_byte_values() {
        assert_ne!(crc16(&[0x00]), crc16(&[0x01]));
        assert_ne!(crc16(&[0xFF]), crc16(&[0xFE]));
    }
}

// ── CRC-32 (ISO 3309 / PKZIP, polynomial 0xEDB88320) ─────────────────────────

/// CRC-32 lookup table — reflected polynomial 0xEDB88320.
const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

/// CRC-32 (ISO 3309/PKZIP, polynomial 0xEDB88320).
/// Used for ZIP local-file-header and gzip checksum validation.
#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    crc ^ 0xFFFF_FFFF
}

/// Verify that `crc32(data)` matches the given expected checksum.
#[inline]
pub fn verify_crc32(data: &[u8], expected: u32) -> bool {
    crc32(data) == expected
}

#[cfg(test)]
mod crc32_tests {
    use super::*;

    #[test]
    fn known_vector_123456789() {
        // Standard CRC-32 check vector: "123456789" → 0xCBF43926
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn empty_slice_is_zero_complement() {
        // CRC-32 of empty input: init XOR final = 0xFFFFFFFF ^ 0xFFFFFFFF = 0
        assert_eq!(crc32(b""), 0x0000_0000);
    }

    #[test]
    fn verify_crc32_round_trip() {
        let data = b"BahyWay-NARAMSIN-v4.0";
        let cs = crc32(data);
        assert!(verify_crc32(data, cs));
        assert!(!verify_crc32(data, cs.wrapping_add(1)));
    }
}
