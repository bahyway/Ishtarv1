//! AkkadianSeal — Sovereign signature trait.
//! 𒀊𒈗𒆪 *kanākum* — "to seal with a cylinder seal".
//! Default: Ed25519 (upgradeable to Dilithium without consumer changes).

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::{KupruError, KupruResult};

pub trait AkkadianSeal: Send + Sync {
    fn seal(&self, message: &[u8]) -> KupruResult<Vec<u8>>;
    fn verify(&self, message: &[u8], seal_bytes: &[u8]) -> bool;
    fn verifier_bytes(&self) -> Vec<u8>;
    fn algorithm_id(&self) -> u8;
}

pub trait SealVerifier: Send + Sync {
    fn verify(&self, message: &[u8], seal_bytes: &[u8]) -> bool;
    fn algorithm_id(&self) -> u8;
}

/// Sovereign key pair — signing key zeroized on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SealKeyPair {
    signing_key:   [u8; 32],
    verifying_key: [u8; 32],
}

impl std::fmt::Debug for SealKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SealKeyPair")
            .field("verifying_key", &hex_encode(self.verifying_key))
            .field("signing_key",   &"[REDACTED]")
            .finish()
    }
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

impl SealKeyPair {
    pub fn generate() -> KupruResult<Self> {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Ok(Self { signing_key: signing_key.to_bytes(), verifying_key: verifying_key.to_bytes() })
    }

    pub fn from_bytes(bytes: &[u8]) -> KupruResult<Self> {
        if bytes.len() != 32 {
            return Err(KupruError::InvalidInput("LEMNĪ: signing key must be 32 bytes".into()));
        }
        use ed25519_dalek::SigningKey;
        let mut b = [0u8; 32];
        b.copy_from_slice(bytes);
        let sk = SigningKey::from_bytes(&b);
        Ok(Self { signing_key: b, verifying_key: sk.verifying_key().to_bytes() })
    }

    pub fn verifying_key_bytes(&self) -> [u8; 32] { self.verifying_key }

    /// The raw private signing key bytes. Only exists to support
    /// `architect_key::split_architect_key()` -- the sole legitimate
    /// reason to ever pull this out of its zeroize-on-drop wrapper.
    /// Callers must zeroize the returned array themselves once done
    /// (`architect_key.rs` does this immediately after use).
    pub fn signing_key_bytes(&self) -> [u8; 32] { self.signing_key }

    pub fn verifier(&self) -> SovereignVerifier {
        SovereignVerifier { verifying_key: self.verifying_key }
    }
}

impl AkkadianSeal for SealKeyPair {
    fn seal(&self, message: &[u8]) -> KupruResult<Vec<u8>> {
        use ed25519_dalek::{Signer, SigningKey};
        let sk = SigningKey::from_bytes(&self.signing_key);
        Ok(sk.sign(message).to_bytes().to_vec())
    }

    fn verify(&self, message: &[u8], seal_bytes: &[u8]) -> bool {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        let Ok(vk)  = VerifyingKey::from_bytes(&self.verifying_key)          else { return false; };
        let Ok(arr) = <[u8; 64]>::try_from(seal_bytes)                       else { return false; };
        let sig = Signature::from_bytes(&arr);
        vk.verify(message, &sig).is_ok()
    }

    fn verifier_bytes(&self) -> Vec<u8> { self.verifying_key.to_vec() }
    fn algorithm_id(&self) -> u8 { 0x01 } // Ed25519
}

/// Public-only verification handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignVerifier {
    verifying_key: [u8; 32],
}

impl SovereignVerifier {
    pub fn from_bytes(bytes: &[u8]) -> KupruResult<Self> {
        if bytes.len() != 32 {
            return Err(KupruError::InvalidInput("LEMNĪ: verifying key must be 32 bytes".into()));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self { verifying_key: key })
    }
}

impl SealVerifier for SovereignVerifier {
    fn verify(&self, message: &[u8], seal_bytes: &[u8]) -> bool {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        let Ok(vk)  = VerifyingKey::from_bytes(&self.verifying_key) else { return false; };
        let Ok(arr) = <[u8; 64]>::try_from(seal_bytes)              else { return false; };
        let sig = Signature::from_bytes(&arr);
        vk.verify(message, &sig).is_ok()
    }
    fn algorithm_id(&self) -> u8 { 0x01 }
}
