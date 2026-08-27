//! kupru-vault — real Sargon-vault open+authenticate logic, in one place.
//!
//! Before this crate existed, the exact same ~40 lines
//! (`AkkadianRoot::from_phrase` + `SargonKdf::with_salt` +
//! `AkkadianCipher::open` + per-entry `verify_seal()` + "keep the
//! strongest passport") lived twice: once in
//! `anu_governor::web_auth::open_vault_and_authenticate` (private,
//! HTTP-session-shaped), once duplicated in `kupru-vault-cli` (shell-
//! callable). A third copy for a server-side authorizer would be one too
//! many — this crate is that one real implementation, callable from any
//! Rust caller. `anu_governor`'s own module is left untouched (BLK-1: no
//! blind edits to sealed source) — it can migrate to this crate later,
//! that migration is not done here.

pub mod namespace_authz;

use kupru::{AkkadianCipher, AkkadianRoot, SargonKdf, SargonPassport};
use serde::Deserialize;

/// Must match `kupru-gdext`'s `VAULT_ROOT_PHRASE` and
/// `anu_governor::web_auth::VAULT_ROOT_PHRASE` exactly, or every real
/// vault the Sargon Passport Manager / Gilgamesh Master Key tools write
/// fails to decrypt here.
pub const VAULT_ROOT_PHRASE: &str = "sargon-vault-root";
pub const VAULT_AAD: &[u8] = b"BahyWay.Ecosystem.v4.VaultBlob";
const SALT_LEN: usize = 32;

#[derive(Deserialize)]
struct VaultEntry {
    passport_json: String,
}

/// The identity a successful vault open establishes — audit-safe fields
/// only, same shape `PassportRecordSpec` (in `enkimdb`) extracts for the
/// write-side audit trail. No seal/key material.
#[derive(Debug, Clone)]
pub struct AuthedIdentity {
    pub subject_kaki_hex: String,
    pub realm: String,
    pub privilege_level: u8,
    pub passport_id: String,
    pub expires_at: u64,
}

impl AuthedIdentity {
    pub fn is_architect(&self) -> bool {
        self.privilege_level >= 7
    }
}

/// Decrypts a Sargon-format vault (`salt(32) || sealed`) with
/// `passphrase` and returns the STRONGEST identity among every entry
/// whose `SargonPassport::verify_seal()` actually passes.
pub fn open_vault_and_authenticate(
    vault_bytes: &[u8],
    passphrase: &[u8],
) -> Result<AuthedIdentity, String> {
    if vault_bytes.len() < SALT_LEN + 1 + 12 + 16 {
        return Err("file too short to contain a salt + sealed blob".to_string());
    }
    let (salt_bytes, sealed) = vault_bytes.split_at(SALT_LEN);
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(salt_bytes);

    let root = AkkadianRoot::from_phrase(VAULT_ROOT_PHRASE).map_err(|e| e.to_string())?;
    let kdf = SargonKdf::with_salt(root, salt);
    let key = kdf.derive_key(passphrase).map_err(|e| e.to_string())?;
    let cipher = AkkadianCipher::from_key_bytes(&key).map_err(|e| e.to_string())?;
    let plaintext = cipher
        .open(sealed, VAULT_AAD)
        .map_err(|_| "wrong passphrase, or the vault file is corrupted".to_string())?;

    let entries: Vec<VaultEntry> = serde_json::from_slice(&plaintext)
        .map_err(|e| format!("not a valid vault entry list: {e}"))?;

    let mut best: Option<AuthedIdentity> = None;
    for entry in &entries {
        let Ok(passport) = serde_json::from_str::<SargonPassport>(&entry.passport_json) else {
            continue;
        };
        if passport.verify_seal().is_err() {
            continue;
        }
        let level = passport.istar.privilege_level;
        if best.as_ref().is_none_or(|b| level > b.privilege_level) {
            best = Some(AuthedIdentity {
                subject_kaki_hex: hex::encode(passport.subject_kaki()),
                realm: passport.naru.realm.clone(),
                privilege_level: level,
                passport_id: passport.passport_id(),
                expires_at: passport.quppu.expires_at,
            });
        }
    }
    best.ok_or_else(|| {
        "vault opened, but no passport inside it has a valid, unexpired seal".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kupru::{AkkadianCipher as Cipher, IshtarLayer, NaruLayer, SealKeyPair};

    fn build_vault(passphrase: &[u8], passports: &[SargonPassport]) -> Vec<u8> {
        let root = AkkadianRoot::from_phrase(VAULT_ROOT_PHRASE).unwrap();
        let kdf = SargonKdf::new(root, passphrase).unwrap();
        let salt = *kdf.salt();
        let key = kdf.derive_key(passphrase).unwrap();
        let cipher = Cipher::from_key_bytes(&key).unwrap();

        let entries: Vec<serde_json::Value> = passports
            .iter()
            .map(|p| serde_json::json!({ "passport_json": serde_json::to_string(p).unwrap() }))
            .collect();
        let plaintext = serde_json::to_vec(&entries).unwrap();
        let sealed = cipher.seal(&plaintext, VAULT_AAD).unwrap();

        let mut out = salt.to_vec();
        out.extend_from_slice(&sealed);
        out
    }

    fn issue(privilege_level: u8, realm: &str) -> SargonPassport {
        let keypair = SealKeyPair::generate().unwrap();
        let naru = NaruLayer {
            subject_kaki: [7u8; 16],
            akkadian_name: kupru::AkkadianName::dubsar(),
            linguistic_proof: kupru::LinguisticProof::create("test-linguistic-phrase").unwrap(),
            realm: realm.into(),
            mudu_score: 5,
        };
        let istar = if privilege_level >= 7 {
            IshtarLayer::architect(realm)
        } else {
            IshtarLayer::gardener(realm)
        };
        SargonPassport::issue(naru, istar, [1u8; 16], &keypair, &[0x42u8; 32]).unwrap()
    }

    #[test]
    fn round_trip_with_correct_passphrase_authenticates() {
        let vault = build_vault(b"correct horse battery staple", &[issue(1, "bahyway")]);
        let id = open_vault_and_authenticate(&vault, b"correct horse battery staple").unwrap();
        assert_eq!(id.privilege_level, 1);
    }

    #[test]
    fn wrong_passphrase_is_rejected() {
        let vault = build_vault(b"correct horse battery staple", &[issue(1, "bahyway")]);
        assert!(open_vault_and_authenticate(&vault, b"wrong").is_err());
    }

    #[test]
    fn strongest_of_multiple_wins() {
        let vault = build_vault(b"pw", &[issue(1, "bahyway"), issue(7, "bahyway")]);
        let id = open_vault_and_authenticate(&vault, b"pw").unwrap();
        assert_eq!(id.privilege_level, 7);
    }
}
