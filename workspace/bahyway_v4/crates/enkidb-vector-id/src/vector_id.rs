//! VectorId — a self-generated 128-bit identifier for operational mechanisms.
//! Pure std: no uuid, no rand.

use std::time::{SystemTime, UNIX_EPOCH};

/// A 128-bit opaque identifier for an operational mechanism.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct VectorId([u8; 16]);

// Module-level counter for uniqueness within a process.
static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl VectorId {
    /// Generate a new unique VectorId — pure std, no third-party deps.
    pub fn generate() -> Self {
        let cnt = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Build 16 bytes from two u64s mixed with the counter
        let high = nanos ^ cnt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let low = cnt ^ nanos.wrapping_mul(0x6C62_272E_07BB_0142);

        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&high.to_be_bytes());
        bytes[8..16].copy_from_slice(&low.to_be_bytes());
        VectorId(bytes)
    }

    /// Reconstruct from raw bytes (storage read path).
    pub fn from_bytes(raw: [u8; 16]) -> Self {
        VectorId(raw)
    }

    pub fn bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Well-known Vector ID for the NeverSnapshot job (§3.6).
    pub const NEVER_SNAPSHOT: VectorId = VectorId([0u8; 16]);

    pub fn is_never_snapshot(&self) -> bool {
        self.0 == [0u8; 16]
    }
}

impl core::fmt::Display for VectorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let b = &self.0;
        write!(
            f,
            "vid:{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2],  b[3],
            b[4], b[5],
            b[6], b[7],
            b[8], b[9],
            b[10], b[11], b[12], b[13], b[14], b[15],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_generation() {
        let a = VectorId::generate();
        let b = VectorId::generate();
        assert_ne!(a, b);
    }

    #[test]
    fn round_trip() {
        let v = VectorId::generate();
        let v2 = VectorId::from_bytes(*v.bytes());
        assert_eq!(v, v2);
    }

    #[test]
    fn never_snapshot_sentinel() {
        assert!(VectorId::NEVER_SNAPSHOT.is_never_snapshot());
        assert!(!VectorId::generate().is_never_snapshot());
    }
}
