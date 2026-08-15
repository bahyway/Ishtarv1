//! Fix 4 — Passport seal using canonical binary (not JSON).
//!
//! Provides deterministic seal bytes regardless of platform,
//! serde version, or field ordering — unlike the JSON-based approach
//! used in the initial `SargonPassport::issue()`.

use crate::canonical::{CanonicalBytes, domains};
use crate::passport::{IshtarLayer, KupruLayer, NaruLayer, QuppuLayer};
use crate::akkadian_seal::{AkkadianSeal, SealKeyPair, SealVerifier, SovereignVerifier};
use crate::KupruResult;

/// Produce canonical seal bytes for a SargonPassport.
///
/// This is the ONLY function that should be called when signing or verifying
/// a passport in v4.0. Never use serde_json for seal operations.
pub fn passport_canonical_bytes(
    quppu: &QuppuLayer,
    kupru: &KupruLayer,
    naru:  &NaruLayer,
    istar: &IshtarLayer,
) -> Vec<u8> {
    let mut cb = CanonicalBytes::new(domains::SARGON_PASSPORT);

    // QUPPU layer — deterministic field order
    cb.push_fixed(quppu.passport_id.as_bytes());
    cb.push_u64(quppu.created_at);
    cb.push_u64(quppu.expires_at);
    cb.push_fixed(&quppu.issuer_kaki);

    // KUPRU layer
    cb.push_bytes(&kupru.content_hash);
    cb.push_bytes(&kupru.hmac);
    cb.push_fixed(&kupru.nonce);
    cb.push_u8(kupru.algorithm_id);

    // NĀRU layer
    cb.push_fixed(&naru.subject_kaki);
    cb.push_str(&naru.akkadian_name.sumu);
    cb.push_str(naru.akkadian_name.abu.as_deref().unwrap_or(""));
    cb.push_str(naru.akkadian_name.asaredu.as_deref().unwrap_or(""));
    cb.push_bytes(&naru.linguistic_proof.commitment);
    cb.push_fixed(&naru.linguistic_proof.nonce); // 32 bytes (Fix 2)
    cb.push_u8(naru.linguistic_proof.mudu);
    cb.push_str(&naru.realm);

    // IŠTAR layer
    let scope_bytes: Vec<u8> = istar
        .isretu_scopes
        .iter()
        .flat_map(|s| {
            let b = s.as_bytes();
            let mut v = (b.len() as u32).to_le_bytes().to_vec();
            v.extend_from_slice(b);
            v
        })
        .collect();
    cb.push_bytes(&scope_bytes);
    cb.push_u8(istar.privilege_level);

    cb.finish()
}

/// Sign the canonical passport bytes with the issuer key pair.
pub fn passport_seal(
    quppu:   &QuppuLayer,
    kupru:   &KupruLayer,
    naru:    &NaruLayer,
    istar:   &IshtarLayer,
    keypair: &SealKeyPair,
) -> KupruResult<Vec<u8>> {
    let bytes = passport_canonical_bytes(quppu, kupru, naru, istar);
    keypair.seal(&bytes)
}

/// Verify a passport seal against canonical bytes.
pub fn passport_verify_seal(
    quppu:          &QuppuLayer,
    kupru:          &KupruLayer,
    naru:           &NaruLayer,
    istar:          &IshtarLayer,
    seal_bytes:     &[u8],
    verifying_key:  &[u8],
) -> KupruResult<bool> {
    let bytes    = passport_canonical_bytes(quppu, kupru, naru, istar);
    let verifier = SovereignVerifier::from_bytes(verifying_key)?;
    Ok(verifier.verify(&bytes, seal_bytes))
}
