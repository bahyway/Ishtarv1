//! `SargonKdf` — Sovereign Key Derivation Function.
//!
//! Named after Sargon of Akkad's 54-year reign. The KDF combines:
//!   1. `AkkadianRoot` tri-consonantal entropy (semantic key material)
//!   2. A random salt (ephemeral, per-credential)
//!   3. Argon2id memory-hard derivation (64 MiB, 4-parallel, 54 iterations)
//!
//! **ŠATTĀTU_MAX** (54 hours) encodes Sargon's reign as the sovereign
//! credential TTL. No credential in the BahyWay ecosystem may outlive it.

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{akkadian_root::AkkadianRoot, error::KupruError, KupruResult};

// ── Sovereign constants ───────────────────────────────────────────────────────

/// **ŠATTĀTU_MAX** — 54 hours (Sargon's *"four and fifty years"*), in seconds.
/// No credential may exceed this TTL — intentionally public and visible.
pub const SATTATU_MAX: u64 = 54 * 3600; // 194,400 seconds

const KDF_MEMORY_KIB: u32 = 65_536; // 64 MiB
const KDF_PARALLELISM: u32 = 4;
const KDF_ITERATIONS: u32 = 54; // mirrors SATTATU_MAX symbolically
const KDF_OUTPUT_LEN: usize = 32;

// ── SargonKdf ─────────────────────────────────────────────────────────────────

/// Sovereign key derivation combining Akkadian linguistic entropy with
/// Argon2id memory-hard derivation.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SargonKdf {
    root: AkkadianRoot,
    salt: [u8; 32],
    iterations: u32,
    memory_kib: u32,
    parallelism: u32,
}

impl SargonKdf {
    /// Create a new `SargonKdf` with a fresh random salt.
    ///
    /// Passphrase is combined with root key material — neither alone
    /// is sufficient to reproduce the derived key.
    pub fn new(root: AkkadianRoot, passphrase: &[u8]) -> KupruResult<Self> {
        use rand::RngCore;

        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        // Mix root material into the salt so an identical passphrase
        // still produces different keys for different Akkadian identities.
        let root_material = root.to_key_material();
        for (i, b) in root_material.iter().enumerate() {
            salt[i % 32] ^= b;
        }
        salt[31] ^= (passphrase.len() & 0xFF) as u8;

        Ok(Self {
            root,
            salt,
            iterations: KDF_ITERATIONS,
            memory_kib: KDF_MEMORY_KIB,
            parallelism: KDF_PARALLELISM,
        })
    }

    /// Re-derive with a known salt (for verification from stored credential).
    pub fn with_salt(root: AkkadianRoot, salt: [u8; 32]) -> Self {
        Self {
            root,
            salt,
            iterations: KDF_ITERATIONS,
            memory_kib: KDF_MEMORY_KIB,
            parallelism: KDF_PARALLELISM,
        }
    }

    /// Derive `output_len` bytes of key material.
    ///
    /// Passphrase is combined with `root.to_key_material()` before
    /// Argon2id runs, so a compromised passphrase without the root is useless.
    pub fn derive(&self, passphrase: &[u8], output_len: usize) -> KupruResult<Vec<u8>> {
        use argon2::{Algorithm, Argon2, Params, Version};

        let root_material = self.root.to_key_material();
        let mut combined = Vec::with_capacity(passphrase.len() + 32);
        combined.extend_from_slice(passphrase);
        combined.extend_from_slice(&root_material);

        let params = Params::new(
            self.memory_kib,
            self.iterations,
            self.parallelism,
            Some(output_len),
        )
        .map_err(|e| KupruError::InvalidInput(e.to_string()))?;

        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut output = vec![0u8; output_len];
        argon
            .hash_password_into(&combined, &self.salt, &mut output)
            .map_err(|e| KupruError::SealBroken(e.to_string()))?;

        Ok(output)
    }

    /// Derive the standard 32-byte key (convenience wrapper).
    pub fn derive_key(&self, passphrase: &[u8]) -> KupruResult<[u8; 32]> {
        let bytes = self.derive(passphrase, KDF_OUTPUT_LEN)?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    }

    /// Expose the salt for storage alongside the sealed artifact.
    /// The salt is not secret — it lives in the MUDÛ metadata layer.
    pub fn salt(&self) -> &[u8; 32] {
        &self.salt
    }

    /// Current Unix time in seconds.
    pub fn sovereign_now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Compute expiry: `now + SATTATU_MAX`.
    pub fn sovereign_expiry() -> u64 {
        Self::sovereign_now() + SATTATU_MAX
    }

    /// True if `expires_at` is in the past.
    pub fn is_expired(expires_at: u64) -> bool {
        Self::sovereign_now() > expires_at
    }
}

// ── Timing guard ──────────────────────────────────────────────────────────────

/// Validate credential timing per sovereign law.
/// Returns `Err` if expired, future-dated (clock skew), or TTL exceeds max.
pub fn validate_credential_timing(created_at: u64, expires_at: u64) -> KupruResult<()> {
    let now = SargonKdf::sovereign_now();

    if now > expires_at {
        return Err(KupruError::Expired(format!(
            "ŠANÛ ŠATTĀTI: credential expired at {expires_at}, now is {now}"
        )));
    }

    const MAX_SKEW_SECS: u64 = 300;
    if created_at > now + MAX_SKEW_SECS {
        return Err(KupruError::Expired(format!(
            "ŠANÛ ŠATTĀTI: created_at {created_at} is in the future (now={now})"
        )));
    }

    if expires_at.saturating_sub(created_at) > SATTATU_MAX {
        return Err(KupruError::AccessDenied(format!(
            "DŪRU: credential TTL exceeds ŠATTĀTU_MAX ({SATTATU_MAX}s)"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::akkadian_root::AkkadianRoot;

    // 2026-07-27: regression test for the real bug PB-261's round-trip
    // check caught live -- every vault/ledger consumer of this KDF
    // (DubSar login, DubSar PDM login, Sargon Passport Manager's own
    // vault, Gilgamesh's ledger) called `new()` twice (once to encrypt,
    // once later to decrypt) and expected the same key back, which
    // `new()`'s fresh-random-salt-every-call design can never provide.
    // `with_salt` existed already but nothing actually used it. These
    // tests pin down the two properties that fix depends on.

    #[test]
    fn new_mints_a_different_salt_every_call_even_with_identical_inputs() {
        let root_a = AkkadianRoot::from_phrase("bahyway").unwrap();
        let root_b = AkkadianRoot::from_phrase("bahyway").unwrap();
        let kdf_a = SargonKdf::new(root_a, b"same-passphrase").unwrap();
        let kdf_b = SargonKdf::new(root_b, b"same-passphrase").unwrap();
        assert_ne!(
            kdf_a.salt(),
            kdf_b.salt(),
            "SargonKdf::new must mint a fresh random salt every call -- \
             this is deliberate, not a bug, but it's exactly why every \
             caller MUST persist salt() and use with_salt() to re-derive"
        );
        assert_ne!(
            kdf_a.derive_key(b"same-passphrase").unwrap(),
            kdf_b.derive_key(b"same-passphrase").unwrap(),
            "different salts must produce different keys even for an \
             identical root+passphrase"
        );
    }

    #[test]
    fn with_salt_reproduces_the_identical_key_new_originally_derived() {
        let root = AkkadianRoot::from_phrase("bahyway").unwrap();
        let original = SargonKdf::new(root.clone(), b"correct-passphrase").unwrap();
        let original_key = original.derive_key(b"correct-passphrase").unwrap();
        let stored_salt = *original.salt();

        // Simulate a later session: same root+passphrase, but re-derived
        // via with_salt(stored_salt) instead of a fresh new() call --
        // this is the exact fix every vault/ledger consumer now applies.
        let reopened = SargonKdf::with_salt(root, stored_salt);
        let reopened_key = reopened.derive_key(b"correct-passphrase").unwrap();

        assert_eq!(
            original_key, reopened_key,
            "with_salt() must reproduce the identical key new() derived, \
             given the same salt/root/passphrase -- this is the whole \
             point of persisting the salt at all"
        );
    }

    #[test]
    fn with_salt_and_wrong_passphrase_derives_a_different_key() {
        let root = AkkadianRoot::from_phrase("bahyway").unwrap();
        let original = SargonKdf::new(root.clone(), b"correct-passphrase").unwrap();
        let stored_salt = *original.salt();

        let reopened = SargonKdf::with_salt(root, stored_salt);
        let wrong_key = reopened.derive_key(b"wrong-passphrase").unwrap();
        let right_key = reopened.derive_key(b"correct-passphrase").unwrap();

        assert_ne!(
            wrong_key, right_key,
            "a wrong passphrase must not derive the right key"
        );
    }
}
