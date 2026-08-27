//! IndexRecord — a fixed-size (28 byte) entry mapping a KAKI to its
//! location in the Data File. Fixed size is what makes binary search on
//! disk possible: record `i` always lives at byte offset `i * 28`, so a
//! probe never needs to load anything but the one record it's checking.

/// `kaki(16) + offset(8) + len(4)` = 28 bytes.
pub const INDEX_RECORD_SIZE: usize = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexRecord {
    pub kaki: [u8; 16],
    pub offset: u64,
    pub len: u32,
}

impl IndexRecord {
    pub fn to_bytes(&self) -> [u8; INDEX_RECORD_SIZE] {
        let mut buf = [0u8; INDEX_RECORD_SIZE];
        buf[0..16].copy_from_slice(&self.kaki);
        buf[16..24].copy_from_slice(&self.offset.to_be_bytes());
        buf[24..28].copy_from_slice(&self.len.to_be_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut kaki = [0u8; 16];
        kaki.copy_from_slice(&bytes[0..16]);
        let offset = u64::from_be_bytes(bytes[16..24].try_into().unwrap());
        let len = u32::from_be_bytes(bytes[24..28].try_into().unwrap());
        IndexRecord { kaki, offset, len }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let r = IndexRecord {
            kaki: [7u8; 16],
            offset: 123_456_789,
            len: 42,
        };
        let bytes = r.to_bytes();
        assert_eq!(bytes.len(), INDEX_RECORD_SIZE);
        let back = IndexRecord::from_bytes(&bytes);
        assert_eq!(back, r);
    }
}
