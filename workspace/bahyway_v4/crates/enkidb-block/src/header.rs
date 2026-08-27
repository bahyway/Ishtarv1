//! BlockHeader — 256-byte header prepended to every cold storage block.

use bahyway_core::{BahywayError, Result, TribeId};
use bahyway_crc::crc16;

use crate::{BLOCK_MAGIC, BLOCK_SIZE_BYTES, CHECKSUM_ALGO_CRC16_CCITT};

/// Serialized size of the block header.  Must be exactly 256 bytes.
pub const HEADER_SIZE: usize = 256;

/// 256-byte block header per §9.3.
///
/// Wire layout (offsets are absolute from start of header):
///   [0..8]   magic             b"ENKIv4.0"
///   [8..10]  tribe_id          u16 big-endian
///   [10..14] block_size        u32 big-endian (always BLOCK_SIZE_BYTES)
///   [14]     checksum_algo     0x01 = CRC-16/CCITT
///   [15..17] header_checksum   CRC-16/CCITT of bytes [0..15]
///   [17..256] reserved         zeroed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub tribe_id: TribeId,
    pub block_size: u32,
    pub checksum_algo: u8,
}

impl BlockHeader {
    /// Construct a new header for the given tribe.
    pub fn new(tribe_id: TribeId) -> Self {
        BlockHeader {
            tribe_id,
            block_size: BLOCK_SIZE_BYTES,
            checksum_algo: CHECKSUM_ALGO_CRC16_CCITT,
        }
    }

    /// Serialize to 256 bytes.
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..8].copy_from_slice(BLOCK_MAGIC);
        buf[8..10].copy_from_slice(&self.tribe_id.to_be_bytes());
        buf[10..14].copy_from_slice(&self.block_size.to_be_bytes());
        buf[14] = self.checksum_algo;
        let cs = crc16(&buf[0..15]);
        buf[15..17].copy_from_slice(&cs.to_be_bytes());
        // [17..256] stay zero
        buf
    }

    /// Deserialize and validate a 256-byte block header.
    pub fn from_bytes(raw: &[u8; HEADER_SIZE]) -> Result<Self> {
        if &raw[0..8] != BLOCK_MAGIC {
            let mut magic = [0u8; 8];
            magic.copy_from_slice(&raw[0..8]);
            return Err(BahywayError::BlockMagicMismatch(magic));
        }
        let stored_cs = u16::from_be_bytes([raw[15], raw[16]]);
        let computed_cs = crc16(&raw[0..15]);
        if stored_cs != computed_cs {
            return Err(BahywayError::ChecksumMismatch {
                expected: computed_cs,
                actual: stored_cs,
            });
        }
        Ok(BlockHeader {
            tribe_id: TribeId::from_be_bytes([raw[8], raw[9]]),
            block_size: u32::from_be_bytes(raw[10..14].try_into().unwrap()),
            checksum_algo: raw[14],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let h = BlockHeader::new(TribeId::from_u16(0x0042));
        let raw = h.to_bytes();
        assert_eq!(raw.len(), HEADER_SIZE);
        let h2 = BlockHeader::from_bytes(&raw).unwrap();
        assert_eq!(h, h2);
    }

    #[test]
    fn magic_rejection() {
        let mut raw = BlockHeader::new(TribeId::from_u16(0x0001)).to_bytes();
        raw[0] = b'X'; // corrupt magic
                       // Also recompute checksum to isolate magic check
        let cs = crc16(&raw[0..15]);
        raw[15..17].copy_from_slice(&cs.to_be_bytes());
        assert!(matches!(
            BlockHeader::from_bytes(&raw),
            Err(BahywayError::BlockMagicMismatch(_))
        ));
    }

    #[test]
    fn checksum_rejection() {
        let mut raw = BlockHeader::new(TribeId::from_u16(0x0001)).to_bytes();
        raw[16] ^= 0xFF; // corrupt checksum
        assert!(matches!(
            BlockHeader::from_bytes(&raw),
            Err(BahywayError::ChecksumMismatch { .. })
        ));
    }
}
