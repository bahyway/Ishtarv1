//! Passport audit records — every minted `SargonPassport` becomes a real
//! EnkiMDB particle, `passport.*` EAV namespace ("Passport_schema" in the
//! Architect's own naming; EnkiMDB has no literal SQL schemas, so a
//! namespace prefix is the real mechanism, exactly like `error_type.*`
//! and `attr_type.*` already are).
//!
//! HONEST SCOPE, checked before writing this: this is an AUDIT TRAIL, not
//! a copy of the passport itself. `PassportRecordSpec` carries only
//! non-secret metadata already public inside a passport's own JSON form
//! (`quppu`/`naru`/`istar` layers) -- passport_id, subject/issuer KAKI
//! hex, realm, privilege_level, scopes, created_at/expires_at. It
//! deliberately never carries `kupru.content_hash`/`kupru.hmac`/
//! `kupru.nonce` (the seal itself) or any signing/verifying key --
//! `SargonPassport`'s own serialized form never contains a private key
//! either (checked: `KupruLayer` only holds a hash/hmac/nonce, not key
//! material), but this stays deliberately narrower than "everything
//! serializable" on principle: an audit trail should record that a
//! credential existed and what it could do, not become a second copy of
//! the credential's own cryptographic seal.

use kupru::SargonPassport;

#[derive(Debug, Clone)]
pub struct PassportRecordSpec {
    pub passport_id: String,
    pub subject_kaki_hex: String,
    pub issuer_kaki_hex: String,
    pub realm: String,
    pub privilege_level: u8,
    pub scopes_csv: String,
    pub created_at: u64,
    pub expires_at: u64,
}

impl PassportRecordSpec {
    /// Extracts the audit-safe fields from a real, already-issued
    /// `SargonPassport`. Never called on anything but a passport that
    /// already passed `verify_seal()` -- callers are expected to verify
    /// before minting an audit record for it, the same discipline
    /// `open_vault_and_authenticate` already applies before trusting a
    /// passport's own claimed fields.
    pub fn from_passport(passport: &SargonPassport) -> Self {
        Self {
            passport_id: passport.passport_id(),
            subject_kaki_hex: hex::encode(passport.subject_kaki()),
            issuer_kaki_hex: hex::encode(passport.quppu.issuer_kaki),
            realm: passport.naru.realm.clone(),
            privilege_level: passport.istar.privilege_level,
            scopes_csv: passport.istar.isretu_scopes.join(","),
            created_at: passport.quppu.created_at,
            expires_at: passport.quppu.expires_at,
        }
    }
}
