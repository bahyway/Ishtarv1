//! Canonical byte serialisation — deterministic, platform-independent.
//! Used for all signing operations so serde version/field order never matters.

/// Domain separator constants for canonical byte streams.
pub mod domains {
    /// Domain separator for SargonPassport seal operations.
    pub const SARGON_PASSPORT: &[u8] = b"BahyWay.SargonPassport.v4.KANAKU";
}

/// Accumulates bytes in a deterministic, length-prefixed format.
/// All multi-byte integers are little-endian.
pub struct CanonicalBytes {
    data: Vec<u8>,
}

impl CanonicalBytes {
    /// Create a new stream anchored with a domain separator.
    pub fn new(domain: &[u8]) -> Self {
        let mut cb = Self { data: Vec::with_capacity(512) };
        cb.push_bytes(domain);
        cb
    }

    /// Append length-prefixed bytes (u32 LE length + payload).
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.push_u32(bytes.len() as u32);
        self.data.extend_from_slice(bytes);
    }

    /// Append a length-prefixed UTF-8 string.
    pub fn push_str(&mut self, s: &str) {
        self.push_bytes(s.as_bytes());
    }

    /// Append a fixed-size byte slice with NO length prefix.
    pub fn push_fixed(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Append a u64 in little-endian.
    pub fn push_u64(&mut self, v: u64) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    /// Append a u32 in little-endian.
    pub fn push_u32(&mut self, v: u32) {
        self.data.extend_from_slice(&v.to_le_bytes());
    }

    /// Append a single byte.
    pub fn push_u8(&mut self, v: u8) {
        self.data.push(v);
    }

    /// Consume the builder and return the accumulated bytes.
    pub fn finish(self) -> Vec<u8> {
        self.data
    }
}
