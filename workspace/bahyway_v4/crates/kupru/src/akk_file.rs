//! AkkFile — Sovereign five-layer .akk file format.
//! 𒁾 *ṭuppu* — "clay tablet / sovereign document".
//!
//! Five layers (ŠARRUKIN format):
//!   [0] RĒŠU     — file header (magic, version, timestamp, domain)
//!   [1] MUDÛ     — metadata (name, quality, creator identity)
//!   [2] KANĀKU   — seal section (AkkadianSeal signature)
//!   [3] ŠIPRU    — payload / content body
//!   [4] ŠIPIR ŠARRI — royal integrity check (BLAKE3 / SHA3-256)

use serde::{Deserialize, Serialize};
use crate::{KupruError, KupruResult};

pub const AKK_MAGIC:   &[u8; 4] = b"\xACKv4";  // v4.0 sovereign magic
pub const AKK_VERSION: u8       = 0x04;          // BahyWay v4.0

// ── RĒŠU — File Header ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResuHeader {
    pub magic:      [u8; 4],
    pub version:    u8,
    pub domain:     u8,      // KAKI B10
    pub timestamp:  u64,     // sovereign_now()
    pub flags:      u16,
}

impl ResuHeader {
    pub fn new(domain: u8) -> Self {
        Self {
            magic:     *AKK_MAGIC,
            version:   AKK_VERSION,
            domain,
            timestamp: sovereign_now(),
            flags:     0x0000,
        }
    }

    pub fn validate(&self) -> KupruResult<()> {
        if &self.magic != AKK_MAGIC {
            return Err(KupruError::InvalidInput("LEMNĪ: invalid .akk magic".into()));
        }
        if self.version != AKK_VERSION {
            return Err(KupruError::InvalidInput(format!(
                "LEMNĪ: unsupported .akk version {:#04x}", self.version
            )));
        }
        Ok(())
    }
}

// ── MUDÛ — Metadata ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuduMetadata {
    pub name:       String,
    pub quality:    u8,       // KAKI B11 (QUALITY_DIVISOR=240.0)
    pub creator_id: String,   // SargonPassport ID
    pub tribe:      Option<String>,
    pub tags:       Vec<String>,
}

impl MuduMetadata {
    pub fn new(name: impl Into<String>, quality: u8, creator_id: impl Into<String>) -> Self {
        Self {
            name:       name.into(),
            quality:    quality.min(240),
            creator_id: creator_id.into(),
            tribe:      None,
            tags:       Vec::new(),
        }
    }
}

// ── KANĀKU — Seal Section ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanakuSeal {
    pub algorithm_id: u8,    // 0x01 = Ed25519
    pub verifier_key: Vec<u8>,
    pub signature:    Vec<u8>,
}

// ── ŠIPIR ŠARRI — Royal Integrity Check ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipirSharri {
    pub digest_algo: u8,     // 0x03 = SHA3-256 (stub: BLAKE3 = 0x04)
    pub digest:      [u8; 32],
}

impl ShipirSharri {
    pub fn compute(payload: &[u8]) -> Self {
        use sha3::{Digest, Sha3_256};
        let mut h = Sha3_256::new();
        h.update(payload);
        Self { digest_algo: 0x03, digest: h.finalize().into() }
    }

    pub fn verify(&self, payload: &[u8]) -> bool {
        use sha3::{Digest, Sha3_256};
        use subtle::ConstantTimeEq;
        let mut h = Sha3_256::new();
        h.update(payload);
        let expected: [u8; 32] = h.finalize().into();
        self.digest.ct_eq(&expected).into()
    }
}

// ── AkkFile — Assembled Document ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkkFile {
    pub header:   ResuHeader,
    pub metadata: MuduMetadata,
    pub seal:     Option<KanakuSeal>,
    pub payload:  Vec<u8>,
    pub integrity: ShipirSharri,
}

impl AkkFile {
    pub fn new(
        domain:     u8,
        name:       impl Into<String>,
        quality:    u8,
        creator_id: impl Into<String>,
        payload:    Vec<u8>,
    ) -> Self {
        let header    = ResuHeader::new(domain);
        let metadata  = MuduMetadata::new(name, quality, creator_id);
        let integrity = ShipirSharri::compute(&payload);
        Self { header, metadata, seal: None, payload, integrity }
    }

    /// Seal the file with a signing key (populates KANĀKU layer).
    pub fn sign<S: crate::AkkadianSeal>(&mut self, signer: &S) {
        let msg = self.sign_payload();
        if let Ok(sig) = signer.seal(&msg) {
            self.seal = Some(KanakuSeal {
                algorithm_id: signer.algorithm_id(),
                verifier_key: signer.verifier_bytes(),
                signature:    sig,
            });
        }
    }

    /// Verify the KANĀKU seal using a verifier.
    pub fn verify_seal<V: crate::SealVerifier>(&self, verifier: &V) -> bool {
        let Some(k) = &self.seal else { return false };
        let msg = self.sign_payload();
        verifier.verify(&msg, &k.signature)
    }

    /// Verify the ŠIPIR ŠARRI integrity digest.
    pub fn verify_integrity(&self) -> bool {
        self.integrity.verify(&self.payload)
    }

    /// Serialise to bytes (JSON envelope).
    pub fn to_bytes(&self) -> KupruResult<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| KupruError::Io(format!("LEMNĪ: serialise failed: {e}")))
    }

    /// Deserialise from bytes.
    pub fn from_bytes(data: &[u8]) -> KupruResult<Self> {
        serde_json::from_slice(data)
            .map_err(|e| KupruError::Io(format!("LEMNĪ: deserialise failed: {e}")))
    }

    fn sign_payload(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.header.magic);
        msg.push(self.header.version);
        msg.push(self.header.domain);
        msg.extend_from_slice(self.metadata.name.as_bytes());
        msg.push(self.metadata.quality);
        msg.extend_from_slice(&self.payload);
        msg
    }
}

fn sovereign_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
