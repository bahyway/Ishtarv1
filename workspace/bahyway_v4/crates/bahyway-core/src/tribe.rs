//! TribeId — the 2-byte PA-15 sovereignty identifier (κ[4..5]).
//!
//! Tribe membership is a structural fact set at birth and immutable.
//! Functional tribe state (phantom/functional/dead) is computed at query time.

/// Big-endian 2-byte sovereign tribe identifier.  Encoded at κ[4..5].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TribeId(pub u16);

impl TribeId {
    pub const fn from_u16(v: u16) -> Self {
        TribeId(v)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }

    pub fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }

    pub fn from_be_bytes(b: [u8; 2]) -> Self {
        TribeId(u16::from_be_bytes(b))
    }
}

impl core::fmt::Display for TribeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "tribe:{:#06x}", self.0)
    }
}
