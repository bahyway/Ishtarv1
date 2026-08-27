//! `SovereignHash` — algorithm-agnostic hashing trait.
//!
//! No consumer of `kupru` ever imports a hash algorithm by name.
//! They call `SovereignHash::digest()` and receive bytes. The underlying
//! algorithm is an implementation detail of KUPRU, upgradeable at any time.

use crate::{KupruError, KupruResult};
use serde::{Deserialize, Serialize};

/// Identifies which sovereign hash algorithm is in use.
/// Stored in the RĒŠU header of every `.akk` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HashAlgorithm {
    /// Current default — 512-bit output, quantum-resistant digest length.
    SovereignDigest512 = 0x01,
    /// 256-bit variant for performance-sensitive paths.
    SovereignDigest256 = 0x02,
    /// Reserved for future post-quantum hash (e.g. SPHINCS+).
    PostQuantum = 0xFF,
}

/// Sovereign hashing interface. Never exposes an algorithm name.
pub trait SovereignHash: Send + Sync {
    /// Produce a digest of `data`. Length is `Self::output_len()` bytes.
    fn digest(&self, data: &[u8]) -> Vec<u8>;

    /// Digest output length in bytes.
    fn output_len(&self) -> usize;

    /// Identify which algorithm variant is active (for `.akk` RĒŠU header).
    fn algorithm(&self) -> HashAlgorithm;

    /// Produce a keyed digest (HMAC-equivalent). Used for KUPRU layer integrity.
    fn keyed_digest(&self, key: &[u8], data: &[u8]) -> KupruResult<Vec<u8>>;
}

// ── Internal implementations ──────────────────────────────────────────────────
// These structs are pub(crate) — nothing outside kupru ever names them.

pub(crate) struct Sha3Digest512;
pub(crate) struct Sha3Digest256;

impl SovereignHash for Sha3Digest512 {
    fn digest(&self, data: &[u8]) -> Vec<u8> {
        use sha3::{Digest, Sha3_512};
        Sha3_512::digest(data).to_vec()
    }
    fn output_len(&self) -> usize {
        64
    }
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::SovereignDigest512
    }
    fn keyed_digest(&self, key: &[u8], data: &[u8]) -> KupruResult<Vec<u8>> {
        use hmac::{Hmac, Mac};
        use sha3::Sha3_512;
        type HmacSha3_512 = Hmac<Sha3_512>;
        let mut mac = HmacSha3_512::new_from_slice(key)
            .map_err(|e| KupruError::InvalidInput(e.to_string()))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

impl SovereignHash for Sha3Digest256 {
    fn digest(&self, data: &[u8]) -> Vec<u8> {
        use sha3::{Digest, Sha3_256};
        Sha3_256::digest(data).to_vec()
    }
    fn output_len(&self) -> usize {
        32
    }
    fn algorithm(&self) -> HashAlgorithm {
        HashAlgorithm::SovereignDigest256
    }
    fn keyed_digest(&self, key: &[u8], data: &[u8]) -> KupruResult<Vec<u8>> {
        use hmac::{Hmac, Mac};
        use sha3::Sha3_256;
        type HmacSha3_256 = Hmac<Sha3_256>;
        let mut mac = HmacSha3_256::new_from_slice(key)
            .map_err(|e| KupruError::InvalidInput(e.to_string()))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

/// Construct the default sovereign hasher (SHA3-512, 512-bit output).
/// Only public factory function — no algorithm name exposed to callers.
pub fn default_hasher() -> Box<dyn SovereignHash> {
    Box::new(Sha3Digest512)
}

/// Construct the fast sovereign hasher (SHA3-256) for performance-sensitive paths.
pub fn fast_hasher() -> Box<dyn SovereignHash> {
    Box::new(Sha3Digest256)
}
