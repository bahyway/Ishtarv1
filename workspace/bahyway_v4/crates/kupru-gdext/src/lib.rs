//! kupru-gdext — the GDExtension bridge exposing `kupru`'s real
//! sovereign crypto (Argon2id, Ed25519, Shamir M-of-N) to Godot.
//!
//! Before this crate existed, `AkkadiSafeBridge.gd` was an honest stub
//! (`vault_available()` hard-coded `false`, `seal_profile()` returning
//! the literal marker `"UNSEALED-DEV"`). This crate is what makes that
//! stub replaceable with real crypto -- it is the ONLY place raw key
//! material crosses the Rust/GDScript boundary, and it does so
//! deliberately narrowly: one coarse-grained `KupruBridge` class with
//! stateless methods on Godot-native types (String, PackedByteArray,
//! Dictionary) plus JSON strings for the structured SargonPassport /
//! ArchitectKeyShare types kupru already derives Serialize/Deserialize
//! for. No long-lived Rust object handles cross the FFI boundary --
//! that's the fragile part of most GDExtension designs, avoided here
//! on purpose.

use godot::prelude::*;
use kupru::{
    AkkadianCipher, AkkadianName, AkkadianRoot, ArchitectKeyShare, IshtarLayer, LinguisticProof,
    NaruLayer, SargonKdf, SargonPassport, SealKeyPair,
};

/// Every method here returns a heterogeneous result (bool/String/
/// PackedByteArray/i64/Array mixed together), so the Dictionary value
/// type is the universal `Variant`.
type VDict = Dictionary<GString, Variant>;

/// Domain separator bound into every vault blob's AEAD tag -- prevents
/// a sealed vault blob from being replayed as some other kind of
/// sealed payload elsewhere in the ecosystem.
const VAULT_AAD: &[u8] = b"BahyWay.Ecosystem.v4.VaultBlob";

struct KupruExtension;

#[gdextension]
unsafe impl ExtensionLibrary for KupruExtension {}

#[derive(GodotClass)]
#[class(base=RefCounted, init)]
struct KupruBridge {
    base: Base<RefCounted>,
}

#[godot_api]
impl KupruBridge {
    // ── Keys ─────────────────────────────────────────────────────────

    /// Generate a fresh Ed25519 keypair. Returns a Dictionary with
    /// "signing_key" and "verifying_key" as 32-byte PackedByteArrays.
    /// The caller (GDScript) is responsible for encrypting
    /// "signing_key" before it ever touches disk.
    #[func]
    fn generate_keypair(&self) -> VDict {
        match SealKeyPair::generate() {
            Ok(kp) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set(
                    "signing_key",
                    &PackedByteArray::from(kp.signing_key_bytes().as_slice()),
                );
                d.set(
                    "verifying_key",
                    &PackedByteArray::from(kp.verifying_key_bytes().as_slice()),
                );
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    // ── Argon2id key derivation ─────────────────────────────────────

    /// Derive a 32-byte key from a root phrase (>= 3 consonants, per
    /// AkkadianRoot::from_phrase) and a passphrase. Returns "key" and
    /// "salt" -- persist the salt alongside whatever this key
    /// encrypts, so it can be re-derived identically later.
    #[func]
    fn derive_key(&self, root_phrase: GString, passphrase: GString) -> VDict {
        let root = match AkkadianRoot::from_phrase(&root_phrase.to_string()) {
            Ok(r) => r,
            Err(e) => return Self::error_dict(&e.to_string()),
        };
        let pass_bytes = passphrase.to_string().into_bytes();
        let kdf = match SargonKdf::new(root, &pass_bytes) {
            Ok(k) => k,
            Err(e) => return Self::error_dict(&e.to_string()),
        };
        match kdf.derive_key(&pass_bytes) {
            Ok(key) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set("key", &PackedByteArray::from(key.as_slice()));
                d.set("salt", &PackedByteArray::from(kdf.salt().as_slice()));
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    /// Re-derive the SAME 32-byte key `derive_key` produced earlier, given
    /// the salt it returned at that time. `derive_key`'s own doc comment
    /// already says to persist that salt "so it can be re-derived
    /// identically later" -- this is that re-derivation path. Without it,
    /// nothing that calls `derive_key` twice (encrypt now, decrypt later)
    /// can ever succeed: `SargonKdf::new` mints a fresh random salt on
    /// every call by design (see kupru::sargon_kdf), so two independent
    /// `derive_key` calls with identical root_phrase/passphrase produce
    /// two DIFFERENT keys, and any AEAD decrypt against the first key
    /// using the second always fails authentication -- confirmed live
    /// (2026-07-27) via PB-261's own round-trip check, which is exactly
    /// what caught this gap before it shipped in an actual login flow.
    #[func]
    fn derive_key_with_salt(
        &self,
        root_phrase: GString,
        passphrase: GString,
        salt: PackedByteArray,
    ) -> VDict {
        let root = match AkkadianRoot::from_phrase(&root_phrase.to_string()) {
            Ok(r) => r,
            Err(e) => return Self::error_dict(&e.to_string()),
        };
        if salt.len() != 32 {
            return Self::error_dict(&format!(
                "salt must be exactly 32 bytes, got {}",
                salt.len()
            ));
        }
        let mut salt_arr = [0u8; 32];
        salt_arr.copy_from_slice(salt.as_slice());
        let pass_bytes = passphrase.to_string().into_bytes();
        let kdf = SargonKdf::with_salt(root, salt_arr);
        match kdf.derive_key(&pass_bytes) {
            Ok(key) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set("key", &PackedByteArray::from(key.as_slice()));
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    // ── Vault-at-rest encryption (ChaCha20-Poly1305) ─────────────────

    /// Encrypt `plaintext` (e.g. a JSON blob listing every passport in
    /// the vault) under a 32-byte key -- get that key from `derive_key`
    /// with a real vault master passphrase, never a placeholder.
    /// Returns "sealed" (nonce + ciphertext + tag, ready to write to
    /// disk as-is).
    #[func]
    fn encrypt_vault_blob(&self, key: PackedByteArray, plaintext: PackedByteArray) -> VDict {
        let cipher = match AkkadianCipher::from_key_bytes(key.as_slice()) {
            Ok(c) => c,
            Err(e) => return Self::error_dict(&e.to_string()),
        };
        match cipher.seal(plaintext.as_slice(), VAULT_AAD) {
            Ok(sealed) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set("sealed", &PackedByteArray::from(sealed.as_slice()));
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    /// Decrypt a blob produced by `encrypt_vault_blob`. Fails honestly
    /// (wrong key, corrupted file, or tampering all produce the same
    /// AEAD authentication failure -- ChaCha20-Poly1305 doesn't
    /// distinguish them, by design).
    #[func]
    fn decrypt_vault_blob(&self, key: PackedByteArray, sealed: PackedByteArray) -> VDict {
        let cipher = match AkkadianCipher::from_key_bytes(key.as_slice()) {
            Ok(c) => c,
            Err(e) => return Self::error_dict(&e.to_string()),
        };
        match cipher.open(sealed.as_slice(), VAULT_AAD) {
            Ok(plaintext) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set("plaintext", &PackedByteArray::from(plaintext.as_slice()));
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    // ── SargonPassport ───────────────────────────────────────────────

    /// Issue a new gardener (read-only, entry level) SargonPassport.
    /// `signing_key_bytes` is the 32-byte private key from
    /// `generate_keypair()`. Returns "passport_json" -- serialize this
    /// straight into the local vault file.
    #[func]
    fn issue_gardener_passport(
        &self,
        signing_key_bytes: PackedByteArray,
        identity_phrase: GString,
        realm: GString,
        subject_label: GString,
    ) -> VDict {
        self.issue_passport_inner(
            signing_key_bytes,
            identity_phrase,
            realm,
            subject_label,
            false,
        )
    }

    /// Issue a new architect-privilege (level 7, full wildcard scope)
    /// SargonPassport. Same shape as issue_gardener_passport -- still
    /// bound to the same SATTATU_MAX expiry as every other passport.
    #[func]
    fn issue_architect_passport_local(
        &self,
        signing_key_bytes: PackedByteArray,
        identity_phrase: GString,
        realm: GString,
        subject_label: GString,
    ) -> VDict {
        self.issue_passport_inner(
            signing_key_bytes,
            identity_phrase,
            realm,
            subject_label,
            true,
        )
    }

    /// Verify a passport's seal (signature + timing). `passport_json`
    /// is exactly what issue_*_passport's "passport_json" produced.
    #[func]
    fn verify_passport(&self, passport_json: GString) -> VDict {
        let passport: SargonPassport = match serde_json::from_str(&passport_json.to_string()) {
            Ok(p) => p,
            Err(e) => return Self::error_dict(&format!("malformed passport JSON: {e}")),
        };
        let mut d = VDict::new();
        d.set("ok", true);
        match passport.verify_seal() {
            Ok(()) => d.set("valid", true),
            Err(e) => {
                d.set("valid", false);
                d.set("error", e.to_string());
            }
        }
        d
    }

    /// True if this passport is due for renewal (80% of its lifetime
    /// elapsed) -- check this on vault load / on a periodic timer.
    #[func]
    fn passport_renewal_due(&self, passport_json: GString) -> bool {
        serde_json::from_str::<SargonPassport>(&passport_json.to_string())
            .map(|p| p.renewal_due())
            .unwrap_or(false)
    }

    /// Human-readable summary for a vault list UI.
    #[func]
    fn passport_summary(&self, passport_json: GString) -> VDict {
        let mut d = VDict::new();
        match serde_json::from_str::<SargonPassport>(&passport_json.to_string()) {
            Ok(p) => {
                d.set("ok", true);
                d.set("passport_id", p.passport_id());
                d.set("realm", p.naru.realm.clone());
                d.set("privilege_level", p.istar.privilege_level as i64);
                d.set("created_at", p.quppu.created_at as i64);
                d.set("expires_at", p.quppu.expires_at as i64);
                d.set("renewal_due", p.renewal_due());
            }
            Err(e) => {
                d.set("ok", false);
                d.set("error", e.to_string());
            }
        }
        d
    }

    // ── Architect Key ceremony (Shamir M-of-N) ──────────────────────

    /// Split a 32-byte signing key into `total_shares` Shamir shares,
    /// `threshold` of which reconstruct it. Returns "shares_json" as
    /// an Array of JSON strings -- save each to a SEPARATE file or
    /// location per ARCHITECT_KEY_CEREMONY.md. Never save two shares
    /// from the same split together.
    #[func]
    fn split_architect_key(
        &self,
        signing_key_bytes: PackedByteArray,
        threshold: i64,
        total_shares: i64,
    ) -> VDict {
        let keypair = match Self::keypair_from_bytes(&signing_key_bytes) {
            Ok(k) => k,
            Err(e) => return Self::error_dict(&e),
        };
        match kupru::split_architect_key(&keypair, threshold as u8, total_shares as u8) {
            Ok(shares) => {
                let arr: Array<GString> = shares
                    .iter()
                    .map(|s| GString::from(serde_json::to_string(s).unwrap_or_default().as_str()))
                    .collect();
                let mut d = VDict::new();
                d.set("ok", true);
                d.set("shares_json", &arr);
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    /// Reconstruct the Architect's signing key from >= threshold
    /// gathered shares (each a JSON string produced by
    /// split_architect_key). Fewer than threshold, or shares mixed
    /// from different splits, both fail honestly rather than
    /// returning a wrong key silently.
    #[func]
    fn reconstruct_architect_key(&self, shares_json: Array<GString>) -> VDict {
        let parsed: Result<Vec<ArchitectKeyShare>, String> = shares_json
            .iter_shared()
            .map(|s| serde_json::from_str(&s.to_string()).map_err(|e| e.to_string()))
            .collect();
        let shares = match parsed {
            Ok(s) => s,
            Err(e) => return Self::error_dict(&format!("malformed share JSON: {e}")),
        };
        match kupru::reconstruct_architect_key(&shares) {
            Ok(kp) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set(
                    "signing_key",
                    &PackedByteArray::from(kp.signing_key_bytes().as_slice()),
                );
                d.set(
                    "verifying_key",
                    &PackedByteArray::from(kp.verifying_key_bytes().as_slice()),
                );
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    // ── internals ────────────────────────────────────────────────────

    fn issue_passport_inner(
        &self,
        signing_key_bytes: PackedByteArray,
        identity_phrase: GString,
        realm: GString,
        subject_label: GString,
        architect: bool,
    ) -> VDict {
        let keypair = match Self::keypair_from_bytes(&signing_key_bytes) {
            Ok(k) => k,
            Err(e) => return Self::error_dict(&e),
        };

        let phrase_string = identity_phrase.to_string();
        let proof = match LinguisticProof::create(&phrase_string) {
            Ok(p) => p,
            Err(e) => return Self::error_dict(&e.to_string()),
        };

        // Real Argon2id-derived integrity key (never a hardcoded
        // placeholder) -- reuses the same root the linguistic proof
        // just validated, derived from the identity phrase itself.
        let phrase_bytes = phrase_string.into_bytes();
        let kdf = match SargonKdf::new(proof.root.clone(), &phrase_bytes) {
            Ok(k) => k,
            Err(e) => return Self::error_dict(&e.to_string()),
        };
        let integrity_key = match kdf.derive_key(&phrase_bytes) {
            Ok(k) => k,
            Err(e) => return Self::error_dict(&e.to_string()),
        };

        use sha3::{Digest, Sha3_256};
        let mut h = Sha3_256::new();
        h.update(subject_label.to_string().as_bytes());
        let digest = h.finalize();
        let mut subject_kaki = [0u8; 16];
        subject_kaki.copy_from_slice(&digest[0..16]);

        let realm_string = realm.to_string();
        let naru = NaruLayer {
            subject_kaki,
            akkadian_name: AkkadianName::dubsar(),
            linguistic_proof: proof,
            realm: realm_string.clone(),
            mudu_score: if architect { 7 } else { 5 },
        };
        let istar = if architect {
            IshtarLayer::architect(&realm_string)
        } else {
            IshtarLayer::gardener(&realm_string)
        };

        match SargonPassport::issue(naru, istar, [0u8; 16], &keypair, &integrity_key) {
            Ok(passport) => {
                let mut d = VDict::new();
                d.set("ok", true);
                d.set(
                    "passport_json",
                    serde_json::to_string(&passport).unwrap_or_default(),
                );
                d
            }
            Err(e) => Self::error_dict(&e.to_string()),
        }
    }

    fn keypair_from_bytes(bytes: &PackedByteArray) -> Result<SealKeyPair, String> {
        SealKeyPair::from_bytes(bytes.as_slice()).map_err(|e| e.to_string())
    }

    fn error_dict(msg: &str) -> VDict {
        let mut d = VDict::new();
        d.set("ok", false);
        d.set("error", msg);
        d
    }
}
