//! §A2.6 — the Seal Clause. A spoofed evacuation order is a weapon;
//! the seal is the shield. Verification needs the public key, no network.
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

pub fn card_bytes(zone_id: u16, siren: &Option<String>, directive: &str, epoch: &str) -> Vec<u8> {
    format!("KIDINNU|{}|{}|{}|{}", zone_id,
            siren.as_deref().unwrap_or("NO-SIREN"), directive, epoch)
        .into_bytes()
}
pub fn seal(sk: &SigningKey, bytes: &[u8]) -> Signature { sk.sign(bytes) }
pub fn verify(vk: &VerifyingKey, bytes: &[u8], sig: &Signature) -> bool {
    vk.verify(bytes, sig).is_ok()
}
