//! Verify a SealedAdvisory: signature over the canonical JSON bytes
//! of the `advisory` object, key delivered out-of-band or pinned.
//! Pinning note: in production the verify key is compiled into the
//! apk (sealed at release, NL-001 era naming), NOT trusted from the
//! response — the response copy is a convenience for prototypes.

use base64::Engine as _;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

#[derive(Debug, PartialEq)]
pub enum SealCheck { Valid, Invalid, Malformed }

pub fn verify_sealed_advisory(sealed_json: &str, pinned_key_b64: Option<&str>) -> SealCheck {
    let v: serde_json::Value = match serde_json::from_str(sealed_json) {
        Ok(v) => v, Err(_) => return SealCheck::Malformed };
    let advisory = &v["advisory"];
    if advisory.is_null() { return SealCheck::Malformed; }
    let canonical = match serde_json::to_vec(advisory) {
        Ok(c) => c, Err(_) => return SealCheck::Malformed };
    let key_b64 = pinned_key_b64
        .or_else(|| v["verify_key_b64"].as_str())
        .unwrap_or_default();
    let sig_b64 = v["seal_b64"].as_str().unwrap_or_default();
    let b64 = base64::engine::general_purpose::STANDARD;
    let (Ok(kb), Ok(sb)) = (b64.decode(key_b64), b64.decode(sig_b64)) else {
        return SealCheck::Malformed };
    let (Ok(karr), Ok(sarr)) = (<[u8;32]>::try_from(kb.as_slice()),
                                <[u8;64]>::try_from(sb.as_slice())) else {
        return SealCheck::Malformed };
    let Ok(key) = VerifyingKey::from_bytes(&karr) else { return SealCheck::Malformed };
    match key.verify(&canonical, &Signature::from_bytes(&sarr)) {
        Ok(()) => SealCheck::Valid,
        Err(_) => SealCheck::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn round_trip_seal() {
        let key = SigningKey::from_bytes(&[7u8; 32]);
        let advisory = serde_json::json!({
            "medicine": "Ceftriaxone 1g inj",
            "generated_utc": "2026-08-11T12:00:00Z",
            "staleness_note": "reported stock, not a reservation — call or go and ask",
            "anchors": [] });
        let canonical = serde_json::to_vec(&advisory).unwrap();
        let sig = key.sign(&canonical);
        let b64 = base64::engine::general_purpose::STANDARD;
        let sealed = serde_json::json!({
            "advisory": advisory,
            "seal_alg": "Ed25519",
            "seal_b64": b64.encode(sig.to_bytes()),
            "verify_key_b64": b64.encode(key.verifying_key().to_bytes()) });
        assert_eq!(verify_sealed_advisory(&sealed.to_string(), None), SealCheck::Valid);

        // Tamper: change a quantity => Invalid
        let mut tampered = sealed.clone();
        tampered["advisory"]["medicine"] = "Something else".into();
        assert_eq!(verify_sealed_advisory(&tampered.to_string(), None), SealCheck::Invalid);
    }
}
