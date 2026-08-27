//! KAKI v4.0 canonical byte layout (locked permanently).
//! κ[0..3] uuid_hash | κ[4..5] tribe_id | κ[6] kaki_type | κ[7] kaki_role
//! κ[8..11] reserved | κ[12..13] timestamp | κ[14..15] CRC-16/CCITT
//! OntoGraph reads identity/tribe/type/role only. It NEVER writes KAKI.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Kaki(pub [u8; 16]);

pub const KAKI_TYPE_TEMPLATE: u8 = 0x10;

impl Kaki {
    pub fn uuid_hash(&self) -> u32 {
        u32::from_be_bytes([self.0[0], self.0[1], self.0[2], self.0[3]])
    }
    pub fn tribe_id(&self) -> u16 {
        u16::from_be_bytes([self.0[4], self.0[5]])
    }
    pub fn kaki_type(&self) -> u8 {
        self.0[6]
    }
    pub fn kaki_role(&self) -> u8 {
        self.0[7]
    }
    pub fn timestamp(&self) -> u16 {
        u16::from_be_bytes([self.0[12], self.0[13]])
    }
    pub fn crc(&self) -> u16 {
        u16::from_be_bytes([self.0[14], self.0[15]])
    }

    /// CRC-16/CCITT-FALSE over κ[0..14].
    pub fn compute_crc(bytes: &[u8; 16]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &b in &bytes[..14] {
            crc ^= (b as u16) << 8;
            for _ in 0..8 {
                crc = if crc & 0x8000 != 0 {
                    (crc << 1) ^ 0x1021
                } else {
                    crc << 1
                };
            }
        }
        crc
    }

    pub fn verify(&self) -> bool {
        Self::compute_crc(&self.0) == self.crc()
    }
}
