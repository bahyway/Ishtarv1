//! KUPRU — Sovereign cryptographic security layer.
//! 𒀊𒈗𒆪 *kanākum* — "to seal with a cylinder seal".
//!
//! Provides: AkkadianCipher, AkkadianSeal, SargonKdf, SovereignHash,
//!           LinguisticProof, SargonPassport, AkkFile (5-layer format).

#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod error;
pub mod canonical;
pub mod akkadian_root;
pub mod akkadian_seal;
pub mod akk_cipher;
pub mod akk_file;
pub mod sargon_kdf;
pub mod sovereign_hash;
pub mod passport;
pub mod passport_seal;
pub mod architect_key;

pub use error::{KupruError, KupruResult};

pub use akkadian_root::{AkkadianRoot, AkkadianName, LinguisticProof, VowelPattern};
pub use akkadian_seal::{AkkadianSeal, SealKeyPair, SealVerifier, SovereignVerifier};
pub use akk_cipher::{AkkadianCipher, CipherEnvelope, CipherText, SovereignNonce, default_cipher};
pub use akk_file::{AkkFile, ResuHeader, MuduMetadata, AKK_MAGIC, AKK_VERSION};
pub use sargon_kdf::{SargonKdf, SATTATU_MAX, validate_credential_timing};
pub use sovereign_hash::{SovereignHash, HashAlgorithm, default_hasher, fast_hasher};
pub use passport::{SargonPassport, QuppuLayer, KupruLayer, NaruLayer, IshtarLayer};
pub use passport_seal::{passport_canonical_bytes, passport_seal, passport_verify_seal};
pub use architect_key::{ArchitectKeyShare, split_architect_key, reconstruct_architect_key, issue_architect_passport};
