//! Real session authentication for shakkanakku-web, built directly on
//! `kupru`'s actual cryptographic primitives — not a label, not a flag.
//!
//! **Login credential**: a Sargon-format encrypted vault file (the exact
//! byte layout `sargon-passport-manager`'s Godot tool already writes to
//! `user://sargon_vault.dat`: 32 raw Argon2id salt bytes, then a
//! ChaCha20-Poly1305-sealed JSON array of `{signing_key_b64,
//! verifying_key_b64, passport_json}` entries) plus the passphrase that
//! unlocks it. Root phrase (`"sargon-vault-root"`) and AAD
//! (`"BahyWay.Ecosystem.v4.VaultBlob"`) are the same constants
//! `kupru-gdext`'s `encrypt_vault_blob`/`decrypt_vault_blob` already use —
//! a vault either the Sargon Passport Manager or the Gilgamesh Master Key
//! tool produced (Gilgamesh mints architect-privilege passports; an
//! Architect saves that exported passport JSON into a Sargon-format vault
//! entry the same way any gardener passport is saved) opens here
//! unmodified, no format translation.
//!
//! **Every layer here is real**: `SargonPassport::verify_seal()` is an
//! Ed25519 signature check — tampering with `istar.privilege_level`
//! invalidates the seal (kupru's own test suite proves this,
//! `passport.rs::tampering_with_privilege_level_is_detected`), so there is
//! no way to "just edit the JSON" to self-promote. Session tokens are
//! HMAC-signed (kupru's `fast_hasher().keyed_digest`, SHA3-256) with a key
//! generated fresh in memory at process start — a restart invalidates
//! every session, and a stolen cookie can't be forged without that key.
//! Cookie verification uses `subtle::ConstantTimeEq`, the same
//! constant-time comparator kupru's own `KupruLayer::verify` uses, so a
//! timing side-channel can't leak which byte of a guessed token is wrong.

use kupru::{AkkadianCipher, AkkadianRoot, SargonKdf, SargonPassport};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Must match `kupru-gdext`'s `VAULT_ROOT_PHRASE` exactly, or every real
/// vault the Sargon Passport Manager / Gilgamesh Master Key tools write
/// fails to decrypt here.
const VAULT_ROOT_PHRASE: &str = "sargon-vault-root";
/// Must match `kupru-gdext`'s `VAULT_AAD` exactly.
const VAULT_AAD: &[u8] = b"BahyWay.Ecosystem.v4.VaultBlob";
const SALT_LEN: usize = 32;

/// A web session is capped shorter than a passport's own 54h SATTATU_MAX
/// lifetime — defense in depth: even a valid, unexpired passport doesn't
/// keep a browser tab authenticated indefinitely.
const SESSION_TTL_SECS: u64 = 8 * 3600;

#[derive(Deserialize)]
struct VaultEntry {
    passport_json: String,
}

/// The identity a successful login established, distilled from the
/// strongest passport the vault actually contained (see
/// `open_vault_and_authenticate`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthedIdentity {
    pub subject_kaki_hex: String,
    pub realm: String,
    pub privilege_level: u8,
    pub passport_id: String,
    /// Unix seconds — the *passport's* own expiry, not the session's.
    pub passport_expires_at: u64,
}

impl AuthedIdentity {
    /// "Architect" only at privilege_level 7 (kupru's `IshtarLayer::architect()`
    /// scale — see passport.rs) — the level Shakkanakku's own CSR-08 halt
    /// banner already means by "the Architect". Every other level is a
    /// viewer, not a governor.
    pub fn is_architect(&self) -> bool {
        self.privilege_level >= 7
    }
}

#[derive(Debug)]
pub enum LoginError {
    /// Too short to even contain a salt, or JSON parse failed after
    /// decrypt — a structurally broken file, not a wrong passphrase.
    MalformedVault(String),
    /// AEAD authentication failed — deliberately not distinguished from
    /// "file corrupted": ChaCha20-Poly1305 can't tell the two apart by
    /// design, and neither should an attacker probing this endpoint.
    WrongPassphraseOrCorrupted,
    /// The vault decrypted fine but contained no entry whose passport
    /// seal actually verifies (expired, tampered, or forged).
    NoValidPassport,
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedVault(m) => write!(f, "malformed vault file: {m}"),
            Self::WrongPassphraseOrCorrupted => write!(f, "wrong passphrase, or the vault file is corrupted"),
            Self::NoValidPassport => write!(f, "vault opened, but no passport inside it has a valid, unexpired seal"),
        }
    }
}

/// Decrypts a Sargon-format vault (`salt(32) || sealed`) with `passphrase`
/// and returns the STRONGEST identity among every entry whose
/// `SargonPassport::verify_seal()` actually passes — an Architect
/// (privilege 7) entry in the same vault as a gardener (privilege 1) entry
/// logs the Architect in, because that's what's cryptographically proven,
/// not because of a UI label.
pub fn open_vault_and_authenticate(vault_bytes: &[u8], passphrase: &[u8]) -> Result<AuthedIdentity, LoginError> {
    if vault_bytes.len() < SALT_LEN + 1 + 12 + 16 {
        return Err(LoginError::MalformedVault("file too short to contain a salt + sealed blob".into()));
    }
    let (salt_bytes, sealed) = vault_bytes.split_at(SALT_LEN);
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(salt_bytes);

    let root = AkkadianRoot::from_phrase(VAULT_ROOT_PHRASE)
        .map_err(|e| LoginError::MalformedVault(e.to_string()))?;
    let kdf = SargonKdf::with_salt(root, salt);
    let key = kdf.derive_key(passphrase).map_err(|e| LoginError::MalformedVault(e.to_string()))?;
    let cipher = AkkadianCipher::from_key_bytes(&key).map_err(|e| LoginError::MalformedVault(e.to_string()))?;
    let plaintext = cipher.open(sealed, VAULT_AAD).map_err(|_| LoginError::WrongPassphraseOrCorrupted)?;

    let entries: Vec<VaultEntry> =
        serde_json::from_slice(&plaintext).map_err(|e| LoginError::MalformedVault(format!("not a valid vault entry list: {e}")))?;

    let mut best: Option<AuthedIdentity> = None;
    for entry in &entries {
        let Ok(passport) = serde_json::from_str::<SargonPassport>(&entry.passport_json) else { continue };
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
                passport_expires_at: passport.quppu.expires_at,
            });
        }
    }
    best.ok_or(LoginError::NoValidPassport)
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

struct Session {
    identity: AuthedIdentity,
    expires_at_unix: u64,
}

/// Server-side session store. kupru itself has no session concept —
/// `SargonPassport::verify_seal()` only proves the passport in hand is
/// authentic at the moment you have it — so this is that layer, built on
/// top, exactly as designed: a fresh random 256-bit session ID per login,
/// HMAC-signed (so a client can't mint or edit its own cookie), looked up
/// server-side on every request, capped by both its own TTL and the
/// underlying passport's real expiry.
pub struct SessionStore {
    signing_key: [u8; 32],
    sessions: Mutex<HashMap<String, Session>>,
}

impl SessionStore {
    /// A fresh random key every process start — restarting the server
    /// invalidates every outstanding session, which is the correct
    /// failure mode for a local governor tool (no persisted session
    /// secret to ever leak from disk).
    pub fn new() -> Self {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self { signing_key: key, sessions: Mutex::new(HashMap::new()) }
    }

    fn sign(&self, id_hex: &str) -> String {
        let mac = kupru::fast_hasher()
            .keyed_digest(&self.signing_key, id_hex.as_bytes())
            .expect("HMAC over a fixed-length key can't fail");
        hex::encode(mac)
    }

    /// Mints a new session for `identity` and returns the cookie value
    /// (`"<session id>.<hmac>"`) — the caller sets this as an HttpOnly,
    /// Secure cookie.
    pub fn mint(&self, identity: AuthedIdentity) -> String {
        use rand::RngCore;
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);
        let id_hex = hex::encode(id);
        let mac_hex = self.sign(&id_hex);

        let ttl_capped_expiry = unix_now().saturating_add(SESSION_TTL_SECS).min(identity.passport_expires_at);
        let mut sessions = self.sessions.lock().unwrap();
        prune_expired(&mut sessions);
        sessions.insert(id_hex.clone(), Session { identity, expires_at_unix: ttl_capped_expiry });
        format!("{id_hex}.{mac_hex}")
    }

    /// Verifies a cookie value's HMAC (constant-time, matching kupru's own
    /// `KupruLayer::verify` pattern) and looks up the still-live session.
    pub fn verify(&self, cookie_value: &str) -> Option<AuthedIdentity> {
        let (id_hex, mac_hex) = cookie_value.split_once('.')?;
        let expected = self.sign(id_hex);

        use subtle::ConstantTimeEq;
        let ok: bool = expected.as_bytes().ct_eq(mac_hex.as_bytes()).into();
        if !ok {
            return None;
        }

        let mut sessions = self.sessions.lock().unwrap();
        prune_expired(&mut sessions);
        let session = sessions.get(id_hex)?;
        if session.expires_at_unix <= unix_now() {
            return None;
        }
        Some(session.identity.clone())
    }

    pub fn revoke(&self, cookie_value: &str) {
        if let Some((id_hex, _)) = cookie_value.split_once('.') {
            self.sessions.lock().unwrap().remove(id_hex);
        }
    }
}

fn prune_expired(sessions: &mut HashMap<String, Session>) {
    let now = unix_now();
    sessions.retain(|_, s| s.expires_at_unix > now);
}

// ── Login defender: real per-IP rate limiting + lockout ─────────────────

const FAILURE_THRESHOLD: u32 = 5;
const FAILURE_WINDOW: Duration = Duration::from_secs(5 * 60);
const LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60);

struct AttemptRecord {
    failures: Vec<Instant>,
    locked_until: Option<Instant>,
}

/// A real brute-force defender: `FAILURE_THRESHOLD` failed logins from one
/// source IP within `FAILURE_WINDOW` locks that IP out for
/// `LOCKOUT_DURATION`, regardless of whether a later attempt has the
/// correct passphrase. This is the layer kupru itself deliberately doesn't
/// have (it only proves a credential-in-hand is authentic; nothing there
/// throttles *attempts*) and nothing else in this workspace provides for
/// login endpoints — see `musaru-security`/`hepta-sec-*`, which are
/// content-scanning and in-process trust-scoring tools, not this.
pub struct LoginDefender {
    attempts: Mutex<HashMap<IpAddr, AttemptRecord>>,
}

impl LoginDefender {
    pub fn new() -> Self {
        Self { attempts: Mutex::new(HashMap::new()) }
    }

    /// `Ok(())` if this IP may attempt a login right now; `Err(remaining)`
    /// if it's locked out, with how much longer.
    pub fn check(&self, ip: IpAddr) -> Result<(), Duration> {
        let attempts = self.attempts.lock().unwrap();
        if let Some(rec) = attempts.get(&ip) {
            if let Some(until) = rec.locked_until {
                let now = Instant::now();
                if until > now {
                    return Err(until - now);
                }
            }
        }
        Ok(())
    }

    pub fn record_failure(&self, ip: IpAddr) {
        let now = Instant::now();
        let mut attempts = self.attempts.lock().unwrap();
        let rec = attempts.entry(ip).or_insert_with(|| AttemptRecord { failures: vec![], locked_until: None });
        rec.failures.retain(|t| now.duration_since(*t) < FAILURE_WINDOW);
        rec.failures.push(now);
        if rec.failures.len() as u32 >= FAILURE_THRESHOLD {
            rec.locked_until = Some(now + LOCKOUT_DURATION);
        }
    }

    pub fn record_success(&self, ip: IpAddr) {
        self.attempts.lock().unwrap().remove(&ip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kupru::{AkkadianCipher as Cipher, IshtarLayer, NaruLayer, SealKeyPair};

    /// Builds a real Sargon-format vault byte blob exactly the way
    /// `sargon-passport-manager`'s `_save_vault()` does (see
    /// `godot/sargon-passport-manager/scripts/main.gd`): salt(32) prepended
    /// to the sealed JSON-array-of-entries blob. This proves the Rust side
    /// speaks the same wire format the real Godot tool writes, not a
    /// reimplementation that happens to also call itself "Sargon".
    fn build_vault(passphrase: &[u8], passports: &[(SargonPassport, &str)]) -> Vec<u8> {
        let root = AkkadianRoot::from_phrase(VAULT_ROOT_PHRASE).unwrap();
        let kdf = SargonKdf::new(root, passphrase).unwrap();
        let salt = *kdf.salt();
        let key = kdf.derive_key(passphrase).unwrap();
        let cipher = Cipher::from_key_bytes(&key).unwrap();

        let entries: Vec<serde_json::Value> = passports
            .iter()
            .map(|(p, label)| {
                serde_json::json!({
                    "signing_key_b64": "unused-in-this-test",
                    "verifying_key_b64": "unused-in-this-test",
                    "passport_json": serde_json::to_string(p).unwrap(),
                    "subject_label": label,
                })
            })
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
        let istar = if privilege_level >= 7 { IshtarLayer::architect(realm) } else { IshtarLayer::gardener(realm) };
        SargonPassport::issue(naru, istar, [1u8; 16], &keypair, &[0x42u8; 32]).unwrap()
    }

    #[test]
    fn real_vault_round_trip_with_correct_passphrase_authenticates() {
        let vault = build_vault(b"correct horse battery staple", &[(issue(1, "bahyway"), "gardener")]);
        let identity = open_vault_and_authenticate(&vault, b"correct horse battery staple").unwrap();
        assert_eq!(identity.privilege_level, 1);
        assert!(!identity.is_architect());
    }

    #[test]
    fn wrong_passphrase_is_rejected() {
        let vault = build_vault(b"correct horse battery staple", &[(issue(1, "bahyway"), "gardener")]);
        let err = open_vault_and_authenticate(&vault, b"guess").unwrap_err();
        assert!(matches!(err, LoginError::WrongPassphraseOrCorrupted));
    }

    #[test]
    fn vault_with_an_architect_passport_authenticates_as_architect() {
        let vault = build_vault(
            b"my-vault-pass",
            &[(issue(1, "bahyway"), "gardener"), (issue(7, "bahyway"), "architect")],
        );
        let identity = open_vault_and_authenticate(&vault, b"my-vault-pass").unwrap();
        assert_eq!(identity.privilege_level, 7);
        assert!(identity.is_architect());
    }

    #[test]
    fn tampered_passport_inside_a_correctly_decrypted_vault_is_rejected() {
        let mut architect = issue(7, "bahyway");
        // Simulate an attacker who has the vault passphrase but tries to
        // hand-edit a gardener passport into an architect one after the
        // fact -- verify_seal() must still catch this exactly like kupru's
        // own passport.rs test does; this proves the rejection survives
        // the extra vault round-trip, not just the bare passport case.
        architect.istar.privilege_level = 1; // downgrade after minting as architect
        let vault = build_vault(b"pw", &[(architect, "tampered")]);
        // verify_seal() must fail because outer_seal no longer matches --
        // so NO valid passport remains in the vault at all.
        let err = open_vault_and_authenticate(&vault, b"pw").unwrap_err();
        assert!(matches!(err, LoginError::NoValidPassport));
    }

    #[test]
    fn malformed_vault_bytes_are_rejected_cleanly() {
        let err = open_vault_and_authenticate(b"too short", b"pw").unwrap_err();
        assert!(matches!(err, LoginError::MalformedVault(_)));
    }

    #[test]
    fn session_mint_and_verify_round_trip() {
        let store = SessionStore::new();
        let identity = AuthedIdentity {
            subject_kaki_hex: "aa".repeat(16),
            realm: "bahyway".into(),
            privilege_level: 7,
            passport_id: "test-id".into(),
            passport_expires_at: unix_now() + 3600,
        };
        let cookie = store.mint(identity);
        let verified = store.verify(&cookie).expect("freshly minted session must verify");
        assert_eq!(verified.privilege_level, 7);
    }

    #[test]
    fn tampered_session_cookie_is_rejected() {
        let store = SessionStore::new();
        let identity = AuthedIdentity {
            subject_kaki_hex: "bb".repeat(16),
            realm: "bahyway".into(),
            privilege_level: 1,
            passport_id: "test-id-2".into(),
            passport_expires_at: unix_now() + 3600,
        };
        let cookie = store.mint(identity);
        let (id, _mac) = cookie.split_once('.').unwrap();
        let forged = format!("{id}.{}", "0".repeat(64));
        assert!(store.verify(&forged).is_none(), "a forged HMAC must not verify");
    }

    #[test]
    fn revoked_session_no_longer_verifies() {
        let store = SessionStore::new();
        let identity = AuthedIdentity {
            subject_kaki_hex: "cc".repeat(16),
            realm: "bahyway".into(),
            privilege_level: 1,
            passport_id: "test-id-3".into(),
            passport_expires_at: unix_now() + 3600,
        };
        let cookie = store.mint(identity);
        store.revoke(&cookie);
        assert!(store.verify(&cookie).is_none());
    }

    #[test]
    fn session_ttl_is_capped_by_the_underlying_passport_expiry() {
        let store = SessionStore::new();
        let identity = AuthedIdentity {
            subject_kaki_hex: "dd".repeat(16),
            realm: "bahyway".into(),
            privilege_level: 7,
            passport_id: "test-id-4".into(),
            // Passport expires in 5 seconds -- far shorter than the 8h
            // default session TTL, so the cap must bind.
            passport_expires_at: unix_now() + 5,
        };
        let cookie = store.mint(identity);
        {
            let sessions = store.sessions.lock().unwrap();
            let (id, _) = cookie.split_once('.').unwrap();
            let s = sessions.get(id).unwrap();
            assert!(s.expires_at_unix <= unix_now() + 5, "session must not outlive the passport it came from");
        }
    }

    #[test]
    fn defender_locks_out_after_threshold_failures_and_clears_on_success() {
        let defender = LoginDefender::new();
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        for _ in 0..FAILURE_THRESHOLD {
            assert!(defender.check(ip).is_ok(), "must not be locked before the threshold");
            defender.record_failure(ip);
        }
        assert!(defender.check(ip).is_err(), "must be locked out at the threshold");

        // A distinct IP is never affected by another IP's failures.
        let other: IpAddr = "10.0.0.5".parse().unwrap();
        assert!(defender.check(other).is_ok());

        // record_success clears the lockout entirely (e.g. an operator
        // manually resets after confirming the real Architect).
        defender.record_success(ip);
        assert!(defender.check(ip).is_ok(), "record_success must clear the lockout");
    }
}
