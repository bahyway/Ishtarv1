//! KUPRU — Sovereign cryptographic security layer.
//! 𒀊𒈗𒆪 *kanākum* — "to seal with a cylinder seal".
//!
//! Provides: AkkadianCipher, AkkadianSeal, SargonKdf, SovereignHash,
//!           LinguisticProof, SargonPassport, AkkFile (5-layer format).

#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod akk_cipher;
pub mod akk_file;
pub mod akkadian_root;
pub mod akkadian_seal;
pub mod architect_key;
pub mod canonical;
pub mod error;
pub mod passport;
pub mod passport_seal;
pub mod sargon_kdf;
pub mod sovereign_hash;

pub use error::{KupruError, KupruResult};

pub use akk_cipher::{default_cipher, AkkadianCipher, CipherEnvelope, CipherText, SovereignNonce};
pub use akk_file::{AkkFile, MuduMetadata, ResuHeader, AKK_MAGIC, AKK_VERSION};
pub use akkadian_root::{AkkadianName, AkkadianRoot, LinguisticProof, VowelPattern};
pub use akkadian_seal::{AkkadianSeal, SealKeyPair, SealVerifier, SovereignVerifier};
pub use architect_key::{
    issue_architect_passport, reconstruct_architect_key, split_architect_key, ArchitectKeyShare,
};
pub use passport::{IshtarLayer, KupruLayer, NaruLayer, QuppuLayer, SargonPassport};
pub use passport_seal::{passport_canonical_bytes, passport_seal, passport_verify_seal};
pub use sargon_kdf::{validate_credential_timing, SargonKdf, SATTATU_MAX};
pub use sovereign_hash::{default_hasher, fast_hasher, HashAlgorithm, SovereignHash};
