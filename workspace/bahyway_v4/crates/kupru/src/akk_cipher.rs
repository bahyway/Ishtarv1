//! AkkadianCipher — ChaCha20-Poly1305 AEAD.
//! 𒀊𒆪𒁲 *tupšarru* — "tablet writer / encryptor".

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{KupruError, KupruResult};

const CIPHER_VERSION: u8 = 0x01;

/// A key handle — 256-bit ChaCha20-Poly1305 key, zeroized on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct AkkadianCipher {
    key: [u8; 32],
}

impl std::fmt::Debug for AkkadianCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AkkadianCipher").field("key", &"[REDACTED]").finish()
    }
}

impl AkkadianCipher {
    pub fn from_key_bytes(key: &[u8]) -> KupruResult<Self> {
        if key.len() != 32 {
            return Err(KupruError::InvalidInput("LEMNĪ: cipher key must be 32 bytes".into()));
        }
        let mut k = [0u8; 32];
        k.copy_from_slice(key);
        Ok(Self { key: k })
    }

    pub fn generate() -> KupruResult<Self> {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Ok(Self { key })
    }

    /// Encrypt `plaintext` with optional `aad`. Returns `nonce || ciphertext`.
    pub fn seal(&self, plaintext: &[u8], aad: &[u8]) -> KupruResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));

        // Fix: generate nonce bytes directly to avoid double-move of GenericArray.
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, Payload { msg: plaintext, aad })
            .map_err(|e| KupruError::Crypto(format!("LEMNĪ: encrypt failed: {e}")))?;

        let mut out = Vec::with_capacity(1 + 12 + ciphertext.len());
        out.push(CIPHER_VERSION);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    /// Decrypt `sealed` (produced by `seal`). Returns plaintext.
    pub fn open(&self, sealed: &[u8], aad: &[u8]) -> KupruResult<Vec<u8>> {
        if sealed.len() < 1 + 12 + 16 {
            return Err(KupruError::Crypto("LEMNĪ: sealed payload too short".into()));
        }
        let version = sealed[0];
        if version != CIPHER_VERSION {
            return Err(KupruError::Crypto(format!(
                "LEMNĪ: unknown cipher version {version:#04x}"
            )));
        }
        let nonce_bytes: [u8; 12] = sealed[1..13].try_into().unwrap();
        let nonce = Nonce::from(nonce_bytes);
        let ciphertext = &sealed[13..];

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
        cipher
            .decrypt(&nonce, Payload { msg: ciphertext, aad })
            .map_err(|e| KupruError::Crypto(format!("LEMNĪ: decrypt failed: {e}")))
    }
}

/// Serialisable envelope for transport / storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherEnvelope {
    pub version:    u8,
    pub nonce_hex:  String,
    pub ct_hex:     String,
    pub aad_hex:    String,
}

impl CipherEnvelope {
    pub fn from_sealed(sealed: &[u8], aad: &[u8]) -> KupruResult<Self> {
        if sealed.len() < 13 {
            return Err(KupruError::Crypto("LEMNĪ: sealed too short for envelope".into()));
        }
        Ok(Self {
            version:   sealed[0],
            nonce_hex: hex_encode(&sealed[1..13]),
            ct_hex:    hex_encode(&sealed[13..]),
            aad_hex:   hex_encode(aad),
        })
    }
}

fn hex_encode(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Serialisable ciphertext blob (nonce prepended, version-tagged).
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct CipherText {
    pub version: u8,
    pub nonce:   [u8; 12],
    pub data:    Vec<u8>,
}

impl CipherText {
    pub fn from_sealed(sealed: &[u8]) -> KupruResult<Self> {
        if sealed.len() < 1 + 12 + 16 {
            return Err(KupruError::Crypto("LEMNĪ: sealed too short".into()));
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&sealed[1..13]);
        Ok(Self { version: sealed[0], nonce, data: sealed[13..].to_vec() })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + 12 + self.data.len());
        out.push(self.version);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.data);
        out
    }
}

/// A 12-byte ChaCha20 nonce — newtype for clarity.
#[derive(Debug, Clone, Copy, Zeroize)]
pub struct SovereignNonce(pub [u8; 12]);

impl SovereignNonce {
    pub fn generate() -> Self {
        let mut b = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut b);
        Self(b)
    }
}

/// Construct a ready-to-use `AkkadianCipher` from the default zero key.
/// Replace with `AkkadianCipher::from_key_bytes()` in production.
pub fn default_cipher() -> KupruResult<AkkadianCipher> {
    AkkadianCipher::from_key_bytes(&[0u8; 32])
}
