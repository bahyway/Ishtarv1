//! ArchitectKey — the sovereign root of trust, and how it mints
//! break-glass access without ever being a single point of failure.
//!
//! There is deliberately NO "master key that decrypts everything." That
//! pattern is a permanent, unrecoverable catastrophe waiting to happen --
//! whoever holds it, forever, holds the whole ecosystem, and losing or
//! leaking it once is unrecoverable. Instead:
//!
//!   1. The Architect holds one Ed25519 `SealKeyPair` -- its PUBLIC half
//!      is the ecosystem's trusted root; the PRIVATE half never lives on
//!      a running server.
//!   2. That private key is split via Shamir's Secret Sharing into N
//!      shares, M of which are required to reconstruct it (M-of-N).
//!      No single share -- no single backup location, no single person --
//!      is enough on its own.
//!   3. Reconstructing the key doesn't "unlock" anything directly. It
//!      lets the Architect MINT a fresh `SargonPassport` at
//!      `IshtarLayer::architect()` privilege (level 7, full wildcard
//!      scopes) -- still bound to the same SATTATU_MAX TTL every other
//!      passport obeys (kupru's own constitutional constraint: "No
//!      credential may outlive it," no exception carved out here). Even
//!      a leaked architect passport dies on schedule.
//!   4. The reconstructed private key and every individual share are
//!      zeroized the moment they're no longer needed -- they should
//!      never persist in memory longer than the single mint operation.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{
    akkadian_root::{AkkadianName, LinguisticProof},
    akkadian_seal::SealKeyPair,
    passport::{IshtarLayer, NaruLayer, SargonPassport},
    KupruError, KupruResult,
};

/// One share of a split ArchitectKey. On its own it reveals nothing
/// about the underlying private key -- only `threshold`-many distinct
/// shares (from the same split) can reconstruct it.
///
/// Store each share in a DIFFERENT trusted location (different person,
/// different physical safe, different offline medium). Never store two
/// shares from the same split together -- that defeats the entire point.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct ArchitectKeyShare {
    /// Which split this share belongs to -- shares from different splits
    /// (e.g. after a rotation) must never be mixed during reconstruction.
    pub split_id: [u8; 16],
    /// This share's index within the split (not secret, safe to label).
    #[zeroize(skip)]
    pub index: u8,
    /// Minimum number of shares required to reconstruct (not secret).
    #[zeroize(skip)]
    pub threshold: u8,
    /// Total shares generated in this split (not secret).
    #[zeroize(skip)]
    pub total_shares: u8,
    /// The actual secret share bytes -- sensitive, zeroized on drop.
    pub share_bytes: Vec<u8>,
}

/// Split an Ed25519 signing key into `total_shares` Shamir shares,
/// `threshold` of which are required to reconstruct it.
///
/// `threshold` must be at least 2 (a threshold of 1 defeats the purpose
/// -- any single share would reconstruct the key) and at most
/// `total_shares`.
pub fn split_architect_key(
    keypair: &SealKeyPair,
    threshold: u8,
    total_shares: u8,
) -> KupruResult<Vec<ArchitectKeyShare>> {
    if threshold < 2 {
        return Err(KupruError::InvalidInput(
            "DŪRU: threshold must be >= 2 -- a threshold of 1 means any single \
             share reconstructs the key, defeating the split entirely"
                .into(),
        ));
    }
    if total_shares < threshold {
        return Err(KupruError::InvalidInput(
            "DŪRU: total_shares must be >= threshold".into(),
        ));
    }

    use rand::RngCore;
    let mut split_id = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut split_id);

    let signing_key_bytes = keypair.signing_key_bytes();

    let sharks = sharks::Sharks(threshold);
    let dealer = sharks.dealer(&signing_key_bytes);

    let shares: Vec<ArchitectKeyShare> = dealer
        .take(total_shares as usize)
        .enumerate()
        .map(|(i, share)| ArchitectKeyShare {
            split_id,
            index: i as u8,
            threshold,
            total_shares,
            share_bytes: Vec::from(&share),
        })
        .collect();

    Ok(shares)
}

/// Reconstruct the Architect's `SealKeyPair` from >= `threshold` shares
/// of the SAME split (same `split_id`). The reconstructed key exists
/// only for the caller's use -- drop it (it zeroizes automatically) as
/// soon as the mint operation that needed it is done.
pub fn reconstruct_architect_key(shares: &[ArchitectKeyShare]) -> KupruResult<SealKeyPair> {
    if shares.is_empty() {
        return Err(KupruError::InvalidInput("DŪRU: no shares provided".into()));
    }

    let split_id = shares[0].split_id;
    let threshold = shares[0].threshold;

    if shares.iter().any(|s| s.split_id != split_id) {
        return Err(KupruError::InvalidInput(
            "DŪRU: shares from different splits cannot be combined -- \
             check they all came from the same split_id"
                .into(),
        ));
    }
    if (shares.len() as u8) < threshold {
        return Err(KupruError::InvalidInput(format!(
            "DŪRU: {} shares provided, but this split requires {threshold}",
            shares.len()
        )));
    }

    let sharks = sharks::Sharks(threshold);
    let parsed: KupruResult<Vec<sharks::Share>> = shares
        .iter()
        .map(|s| {
            sharks::Share::try_from(s.share_bytes.as_slice())
                .map_err(|e| KupruError::Crypto(format!("NAKRU: malformed share: {e}")))
        })
        .collect();
    let parsed = parsed?;

    let mut signing_key_bytes = sharks
        .recover(&parsed)
        .map_err(|e| KupruError::Crypto(format!("NAKRU: reconstruction failed: {e}")))?;

    if signing_key_bytes.len() != 32 {
        signing_key_bytes.zeroize();
        return Err(KupruError::Crypto(
            "NAKRU: reconstructed key has the wrong length -- shares may be corrupt \
             or from an unrelated split"
                .into(),
        ));
    }

    let keypair = SealKeyPair::from_bytes(&signing_key_bytes);
    signing_key_bytes.zeroize();
    keypair
}

/// Mint a fresh Architect-privilege (level 7, full wildcard scope)
/// SargonPassport using the (reconstructed or in-memory) root key.
///
/// This is the ONLY thing the Architect key is ever used for -- it does
/// not decrypt data, does not bypass any other passport's checks, and
/// does not itself grant access to anything. It produces a normal
/// SargonPassport that is subject to the same SATTATU_MAX expiry and
/// the same `verify_seal()` scrutiny as any other passport in the
/// ecosystem; it is simply issued with `IshtarLayer::architect()`
/// scopes instead of a narrower role.
pub fn issue_architect_passport(
    root_keypair: &SealKeyPair,
    realm: &str,
    subject_kaki: [u8; 16],
    subject_name: AkkadianName,
    linguistic_proof: LinguisticProof,
    issuer_kaki: [u8; 16],
    integrity_key: &[u8; 32],
) -> KupruResult<SargonPassport> {
    let naru = NaruLayer {
        subject_kaki,
        akkadian_name: subject_name,
        linguistic_proof,
        realm: realm.to_string(),
        mudu_score: 7, // architect-level linguistic trust, by convention
    };
    let istar = IshtarLayer::architect(realm);

    SargonPassport::issue(naru, istar, issuer_kaki, root_keypair, integrity_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::akkadian_root::LinguisticProof;
    use crate::akkadian_seal::{AkkadianSeal, SealVerifier};

    /// A 3-of-5 split: reconstructing with exactly the threshold (any 3
    /// of the 5) must yield a key that produces IDENTICAL signatures to
    /// the original -- proving this isn't just "some key," it's THE key.
    #[test]
    fn reconstruction_with_threshold_shares_recovers_the_same_key() {
        let original = SealKeyPair::generate().unwrap();
        let shares = split_architect_key(&original, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        let reconstructed = reconstruct_architect_key(&shares[0..3]).unwrap();
        assert_eq!(
            reconstructed.verifying_key_bytes(),
            original.verifying_key_bytes(),
            "reconstructed key must be byte-identical to the original"
        );

        // Prove it's functionally identical, not just same public bytes:
        // a signature made by the reconstructed key must verify against
        // the ORIGINAL key's verifier, and vice versa.
        let msg = b"BahyWay.Ecosystem architect ceremony test";
        let sig = reconstructed.seal(msg).unwrap();
        assert!(original.verifier().verify(msg, &sig));
    }

    /// Any 3-of-5 combination works, not just the first three -- proves
    /// there's no hidden ordering dependency.
    #[test]
    fn any_threshold_combination_of_shares_reconstructs() {
        let original = SealKeyPair::generate().unwrap();
        let shares = split_architect_key(&original, 3, 5).unwrap();

        let combo_a = vec![shares[0].clone(), shares[2].clone(), shares[4].clone()];
        let combo_b = vec![shares[1].clone(), shares[3].clone(), shares[4].clone()];

        let key_a = reconstruct_architect_key(&combo_a).unwrap();
        let key_b = reconstruct_architect_key(&combo_b).unwrap();
        assert_eq!(key_a.verifying_key_bytes(), original.verifying_key_bytes());
        assert_eq!(key_b.verifying_key_bytes(), original.verifying_key_bytes());
    }

    /// The actual security property: fewer than `threshold` shares must
    /// NOT reconstruct the key. This is what makes it safe to give any
    /// one trusted party a single share -- one share alone is useless.
    #[test]
    fn below_threshold_shares_refuses_to_reconstruct() {
        let original = SealKeyPair::generate().unwrap();
        let shares = split_architect_key(&original, 3, 5).unwrap();

        let insufficient = &shares[0..2]; // only 2 of the required 3
        let result = reconstruct_architect_key(insufficient);
        assert!(result.is_err(), "2 shares must not satisfy a 3-of-5 split");
    }

    #[test]
    fn single_share_reveals_nothing_reconstructable() {
        let original = SealKeyPair::generate().unwrap();
        let shares = split_architect_key(&original, 3, 5).unwrap();
        assert!(reconstruct_architect_key(&shares[0..1]).is_err());
    }

    #[test]
    fn mixing_shares_from_different_splits_is_rejected() {
        let original = SealKeyPair::generate().unwrap();
        let split_1 = split_architect_key(&original, 3, 5).unwrap();
        let split_2 = split_architect_key(&original, 3, 5).unwrap(); // different split_id

        let mixed = vec![split_1[0].clone(), split_1[1].clone(), split_2[2].clone()];
        let result = reconstruct_architect_key(&mixed);
        assert!(
            result.is_err(),
            "shares from different splits must never combine"
        );
    }

    #[test]
    fn threshold_of_one_is_rejected_outright() {
        let original = SealKeyPair::generate().unwrap();
        let result = split_architect_key(&original, 1, 5);
        assert!(
            result.is_err(),
            "threshold=1 defeats the entire purpose of splitting"
        );
    }

    /// End-to-end ceremony: split the root key, reconstruct it from
    /// shares (as would happen during an actual break-glass event), mint
    /// an architect passport with it, and confirm the resulting passport
    /// is a completely normal, fully verifiable SargonPassport -- full
    /// wildcard scope, still bound to the same SATTATU_MAX expiry as
    /// every other credential in the ecosystem.
    #[test]
    fn full_ceremony_mints_a_verifiable_architect_passport() {
        let root_keypair = SealKeyPair::generate().unwrap();
        let shares = split_architect_key(&root_keypair, 3, 5).unwrap();

        // Simulate gathering 3 shares from 3 different trusted locations.
        let gathered = vec![shares[1].clone(), shares[3].clone(), shares[4].clone()];
        let reconstructed_root = reconstruct_architect_key(&gathered).unwrap();

        let passport = issue_architect_passport(
            &reconstructed_root,
            "bahyway",
            [0xAAu8; 16],
            AkkadianName::dubsar(),
            LinguisticProof::create("sar-kibrat-arbaim").unwrap(),
            [0x01u8; 16],
            &[0x99u8; 32],
        )
        .unwrap();

        assert!(passport.verify_seal().is_ok());
        assert_eq!(passport.istar.privilege_level, 7);
        assert!(passport.has_scope("kaki:read"));
        assert!(passport.has_scope("enki:write"));
        assert!(passport.has_scope("zero:anything"));
        // Still a normal, expiring credential -- no special exemption.
        assert!(!passport
            .quppu
            .expires_at
            .checked_sub(passport.quppu.created_at)
            .is_none());
        assert_eq!(
            passport.quppu.expires_at - passport.quppu.created_at,
            crate::sargon_kdf::SATTATU_MAX
        );
    }
}
