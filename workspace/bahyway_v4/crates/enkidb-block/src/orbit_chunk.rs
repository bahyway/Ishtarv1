//! OrbitChunk — EAV Orbit chunk header and payload container (§9.3).
//!
//! Each chunk holds EAV attributes for one particle (one kaki_uuid).
//! Payload is natively zstd-compressed (zstd is already installed in EriduuOS).

/// Per-chunk header stored before each EAV payload in the block.
#[derive(Debug, Clone)]
pub struct OrbitChunkHeader {
    /// Which particle's EAV attributes are in this chunk.
    pub kaki_uuid:       [u8; 16],
    /// Orbit shell identifier (partitioning sub-key).
    pub shell_id:        u32,
    /// Number of EAV attribute triples in the payload.
    pub attribute_count: u32,
    /// Size of the (possibly compressed) payload in bytes.
    pub payload_size:    u32,
    /// CRC-16/CCITT of the payload bytes.
    pub payload_checksum: u16,
}

impl OrbitChunkHeader {
    pub const SERIALIZED_SIZE: usize = 16 + 4 + 4 + 4 + 2; // = 30 bytes

    pub fn to_bytes(&self) -> [u8; Self::SERIALIZED_SIZE] {
        let mut buf = [0u8; Self::SERIALIZED_SIZE];
        buf[0..16].copy_from_slice(&self.kaki_uuid);
        buf[16..20].copy_from_slice(&self.shell_id.to_be_bytes());
        buf[20..24].copy_from_slice(&self.attribute_count.to_be_bytes());
        buf[24..28].copy_from_slice(&self.payload_size.to_be_bytes());
        buf[28..30].copy_from_slice(&self.payload_checksum.to_be_bytes());
        buf
    }

    pub fn from_bytes(raw: &[u8; Self::SERIALIZED_SIZE]) -> Self {
        OrbitChunkHeader {
            kaki_uuid:        raw[0..16].try_into().unwrap(),
            shell_id:         u32::from_be_bytes(raw[16..20].try_into().unwrap()),
            attribute_count:  u32::from_be_bytes(raw[20..24].try_into().unwrap()),
            payload_size:     u32::from_be_bytes(raw[24..28].try_into().unwrap()),
            payload_checksum: u16::from_be_bytes([raw[28], raw[29]]),
        }
    }
}

/// An OrbitChunk = header + compressed EAV payload bytes.
pub struct OrbitChunk {
    pub header:  OrbitChunkHeader,
    /// Raw (possibly zstd-compressed) EAV payload.
    pub payload: Vec<u8>,
}

impl OrbitChunk {
    pub fn new(header: OrbitChunkHeader, payload: Vec<u8>) -> Self {
        OrbitChunk { header, payload }
    }

    pub fn verify_payload_checksum(&self) -> bool {
        bahyway_crc::crc16(&self.payload) == self.header.payload_checksum
    }
}
