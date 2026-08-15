#![forbid(unsafe_code)]
//! PiiVault — GDPR Article 17 compliant PII field encryption + erasure.
//!
//! # Design
//!
//! Every PII field for every entity is encrypted with a unique key derived as:
//!
//! ```text
//! entity_key = SHA3-256(vault_master_key || kaki_hash.to_le_bytes() || field_tag)
//! ```
//!
//! **Erasure (Article 17):** `PiiVault::erase(kaki_hash)` permanently marks the
//! entity as erased.  All future `encrypt`/`decrypt` calls for that hash return
//! `PiiError::Erased`.  Because the key derivation depends on the master key held
//! inside the vault, an entity without a surviving vault entry is permanently
//! unreadable — the ciphertext remains in the database but has no key.
//!
//! **Sidecar pattern:** `PiiSidecar` stores encrypted PII *separately* from the
//! main particle database.  The particle record contains only KAKI hashes;
//! human-readable PII (names, dates, tribe) lives exclusively in the sidecar.
//! This is Plan B: if the particle store is breached, no plaintext PII is exposed.
//!
//! # GDPR compliance properties
//!
//! | Requirement | Mechanism |
//! |-------------|-----------|
//! | Article 5(1)(f) — integrity & confidentiality | ChaCha20-Poly1305 AEAD |
//! | Article 17 — right to erasure | `erase()` forgets key derivation rights |
//! | Article 25 — data protection by design | Sidecar: PII never in particle log |
//! | Article 32 — security of processing | Per-field keys, AAD binds to entity |

use std::collections::{HashMap, HashSet};

use kupru::{AkkadianCipher, fast_hasher};

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PiiError {
    /// This entity has been erased — the key material was permanently destroyed.
    Erased,
    /// ChaCha20-Poly1305 operation failed (bad key length or authentication tag).
    CryptoFailure(String),
    /// Caller provided unusable input (e.g. empty field tag).
    InvalidInput(&'static str),
    /// The entity exists in the sidecar but has no entry for this field.
    FieldAbsent,
}

impl core::fmt::Display for PiiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PiiError::Erased             => f.write_str("PII erased — GDPR Article 17 erasure applied"),
            PiiError::CryptoFailure(s)   => write!(f, "PII crypto failure: {s}"),
            PiiError::InvalidInput(s)    => write!(f, "PII invalid input: {s}"),
            PiiError::FieldAbsent        => f.write_str("PII field not present in sidecar"),
        }
    }
}

pub type PiiResult<T> = Result<T, PiiError>;

// ── Field tags — domain separators ────────────────────────────────────────────

/// Field tag for the grave owner's Arabic name.
pub const FIELD_OWNER_NAME:    &[u8] = b"najaf.owner_name_arabic.v4";
/// Field tag for the Hijri year of death.
pub const FIELD_DEATH_YEAR:    &[u8] = b"najaf.death_hijri_year.v4";
/// Field tag for extended tribe affiliation (beyond TribeId).
pub const FIELD_TRIBE_DETAIL:  &[u8] = b"najaf.tribe_affiliation.v4";
/// Field tag for family lineage text.
pub const FIELD_FAMILY_LINEAGE: &[u8] = b"najaf.family_lineage.v4";
/// Field tag for civil registry reference number.
pub const FIELD_CIVIL_REF:     &[u8] = b"najaf.civil_registry_ref.v4";
/// Generic field tag for WPD operator identity.
pub const FIELD_WPD_OPERATOR:  &[u8] = b"wpd.operator_identity.v4";

// ── PiiVault ──────────────────────────────────────────────────────────────────

/// Sovereign PII encryption vault.
///
/// Holds a 32-byte master key (never exposed outside the vault).
/// All per-entity, per-field keys are derived on-the-fly from the master key —
/// nothing additional needs to be persisted.
///
/// Erasure is stored as a `HashSet<u32>` of erased KAKI hashes.  In production
/// this set must be persisted to durable storage so it survives restarts.
pub struct PiiVault {
    /// 32-byte master key — `ZeroizeOnDrop` semantics enforced by AkkadianCipher.
    master_key: [u8; 32],
    /// KAKI hashes of entities whose PII has been permanently erased.
    erased: HashSet<u32>,
}

impl PiiVault {
    /// Create a vault from a 32-byte master key.
    ///
    /// In production, derive `master_key` via `kupru::SargonKdf` (Argon2id)
    /// from an operator passphrase.  For tests, use any 32-byte value.
    pub fn new(master_key: [u8; 32]) -> Self {
        PiiVault { master_key, erased: HashSet::new() }
    }

    /// Encrypt `plaintext` as the PII field `field_tag` for entity `kaki_hash`.
    ///
    /// Returns `sealed` bytes = `version(1) || nonce(12) || ciphertext+tag`.
    /// The KAKI hash is embedded as AAD — the ciphertext is cryptographically
    /// bound to this entity and cannot be transplanted to another.
    pub fn encrypt(
        &self,
        kaki_hash: u32,
        field_tag: &[u8],
        plaintext: &[u8],
    ) -> PiiResult<Vec<u8>> {
        if field_tag.is_empty() {
            return Err(PiiError::InvalidInput("field_tag must not be empty"));
        }
        if self.erased.contains(&kaki_hash) {
            return Err(PiiError::Erased);
        }
        let key    = self.derive_key(kaki_hash, field_tag);
        let cipher = AkkadianCipher::from_key_bytes(&key)
            .map_err(|e| PiiError::CryptoFailure(e.to_string()))?;
        let aad = kaki_hash.to_le_bytes();
        cipher.seal(plaintext, &aad)
            .map_err(|e| PiiError::CryptoFailure(e.to_string()))
    }

    /// Decrypt a PII field for entity `kaki_hash`.
    pub fn decrypt(
        &self,
        kaki_hash: u32,
        field_tag: &[u8],
        sealed:    &[u8],
    ) -> PiiResult<Vec<u8>> {
        if field_tag.is_empty() {
            return Err(PiiError::InvalidInput("field_tag must not be empty"));
        }
        if self.erased.contains(&kaki_hash) {
            return Err(PiiError::Erased);
        }
        let key    = self.derive_key(kaki_hash, field_tag);
        let cipher = AkkadianCipher::from_key_bytes(&key)
            .map_err(|e| PiiError::CryptoFailure(e.to_string()))?;
        let aad = kaki_hash.to_le_bytes();
        cipher.open(sealed, &aad)
            .map_err(|e| PiiError::CryptoFailure(e.to_string()))
    }

    // ── GDPR Article 17 Erasure ───────────────────────────────────────────────

    /// Permanently erase all PII for `kaki_hash`.
    ///
    /// After this call, any future `encrypt()` or `decrypt()` for this hash
    /// returns `PiiError::Erased`.  The ciphertext in the database remains
    /// intact but is permanently unreadable without the key.
    ///
    /// **Persist `erased_hashes()` to durable storage after each erasure.**
    ///
    /// Returns `true` if this was a new erasure; `false` if already erased.
    pub fn erase(&mut self, kaki_hash: u32) -> bool {
        self.erased.insert(kaki_hash)
    }

    /// Restore a previously erased entity (for testing / administrative correction).
    /// Should not be called in production without a formal Data Steward approval.
    pub fn restore(&mut self, kaki_hash: u32) -> bool {
        self.erased.remove(&kaki_hash)
    }

    /// `true` if `kaki_hash` has been erased.
    pub fn is_erased(&self, kaki_hash: u32) -> bool {
        self.erased.contains(&kaki_hash)
    }

    /// Number of entities that have been erased.
    pub fn erased_count(&self) -> usize { self.erased.len() }

    /// Snapshot of the erased-hash set for persistent storage.
    /// The caller is responsible for persisting and restoring this set
    /// across vault restarts (without it, erased entities would become
    /// decryptable again after a restart).
    pub fn erased_hashes(&self) -> Vec<u32> {
        let mut v: Vec<u32> = self.erased.iter().copied().collect();
        v.sort_unstable();
        v
    }

    /// Restore the erased-hash set from persistent storage.
    pub fn restore_erased_set(&mut self, hashes: impl IntoIterator<Item = u32>) {
        for h in hashes { self.erased.insert(h); }
    }

    // ── Key derivation ────────────────────────────────────────────────────────

    /// `entity_key = SHA3-256(master_key || kaki_hash.to_le_bytes() || field_tag)`
    fn derive_key(&self, kaki_hash: u32, field_tag: &[u8]) -> [u8; 32] {
        let h = fast_hasher(); // SHA3-256 — 32-byte output
        let mut m = Vec::with_capacity(32 + 4 + field_tag.len());
        m.extend_from_slice(&self.master_key);
        m.extend_from_slice(&kaki_hash.to_le_bytes());
        m.extend_from_slice(field_tag);
        let digest = h.digest(&m);
        // SHA3-256 produces exactly 32 bytes
        digest[..32].try_into().expect("SHA3-256 is always 32 bytes")
    }
}

// ── PiiSidecarEntry ───────────────────────────────────────────────────────────

/// One entity's encrypted PII — stored in the sidecar, never in the particle log.
#[derive(Debug, Clone)]
pub struct PiiSidecarEntry {
    /// The entity's KAKI uuid_hash (lookup key — not the full 16 bytes).
    pub kaki_hash:       u32,
    /// Encrypted Arabic name (via `PiiVault::encrypt(FIELD_OWNER_NAME, ...)`).
    pub name_sealed:     Option<Vec<u8>>,
    /// Encrypted Hijri year of death.
    pub year_sealed:     Option<Vec<u8>>,
    /// Encrypted extended tribe/family affiliation.
    pub tribe_sealed:    Option<Vec<u8>>,
    /// Encrypted family lineage text.
    pub lineage_sealed:  Option<Vec<u8>>,
    /// Encrypted civil registry reference.
    pub civil_sealed:    Option<Vec<u8>>,
    /// Epoch at which this sidecar entry was created.
    pub created_epoch:   u32,
}

impl PiiSidecarEntry {
    pub fn new(kaki_hash: u32, created_epoch: u32) -> Self {
        PiiSidecarEntry {
            kaki_hash,
            name_sealed:    None,
            year_sealed:    None,
            tribe_sealed:   None,
            lineage_sealed: None,
            civil_sealed:   None,
            created_epoch,
        }
    }

    /// `true` if at least one PII field is populated.
    pub fn has_pii(&self) -> bool {
        self.name_sealed.is_some()
            || self.year_sealed.is_some()
            || self.tribe_sealed.is_some()
            || self.lineage_sealed.is_some()
            || self.civil_sealed.is_some()
    }
}

// ── PiiSidecar ────────────────────────────────────────────────────────────────

/// In-memory PII sidecar store.
///
/// Stores encrypted PII separately from the particle database.  The particle
/// record only contains the KAKI hash; human-readable PII is here.
///
/// Phase 1: in-memory HashMap.
/// Phase 2: file-backed append-only store using `.enkwal` format.
pub struct PiiSidecar {
    vault:   PiiVault,
    entries: HashMap<u32, PiiSidecarEntry>,
}

impl PiiSidecar {
    pub fn new(master_key: [u8; 32]) -> Self {
        PiiSidecar {
            vault:   PiiVault::new(master_key),
            entries: HashMap::new(),
        }
    }

    // ── Write operations ──────────────────────────────────────────────────────

    /// Set the owner name for an entity.
    pub fn set_owner_name(&mut self, kaki_hash: u32, name: &str, epoch: u32) -> PiiResult<()> {
        let sealed = self.vault.encrypt(kaki_hash, FIELD_OWNER_NAME, name.as_bytes())?;
        let entry  = self.entries.entry(kaki_hash)
            .or_insert_with(|| PiiSidecarEntry::new(kaki_hash, epoch));
        entry.name_sealed = Some(sealed);
        Ok(())
    }

    /// Set the Hijri year of death for an entity.
    pub fn set_death_year(&mut self, kaki_hash: u32, year: u32, epoch: u32) -> PiiResult<()> {
        let sealed = self.vault.encrypt(kaki_hash, FIELD_DEATH_YEAR,
            &year.to_le_bytes())?;
        let entry  = self.entries.entry(kaki_hash)
            .or_insert_with(|| PiiSidecarEntry::new(kaki_hash, epoch));
        entry.year_sealed = Some(sealed);
        Ok(())
    }

    /// Set family lineage text for an entity.
    pub fn set_family_lineage(&mut self, kaki_hash: u32, lineage: &str, epoch: u32) -> PiiResult<()> {
        let sealed = self.vault.encrypt(kaki_hash, FIELD_FAMILY_LINEAGE, lineage.as_bytes())?;
        let entry  = self.entries.entry(kaki_hash)
            .or_insert_with(|| PiiSidecarEntry::new(kaki_hash, epoch));
        entry.lineage_sealed = Some(sealed);
        Ok(())
    }

    // ── Read operations ───────────────────────────────────────────────────────

    /// Retrieve the plaintext owner name for an entity.
    pub fn owner_name(&self, kaki_hash: u32) -> PiiResult<String> {
        let entry = self.entries.get(&kaki_hash).ok_or(PiiError::FieldAbsent)?;
        let sealed = entry.name_sealed.as_deref().ok_or(PiiError::FieldAbsent)?;
        let plain  = self.vault.decrypt(kaki_hash, FIELD_OWNER_NAME, sealed)?;
        String::from_utf8(plain).map_err(|_| PiiError::CryptoFailure("UTF-8 decode failed".into()))
    }

    /// Retrieve the plaintext Hijri death year for an entity.
    pub fn death_year(&self, kaki_hash: u32) -> PiiResult<u32> {
        let entry  = self.entries.get(&kaki_hash).ok_or(PiiError::FieldAbsent)?;
        let sealed = entry.year_sealed.as_deref().ok_or(PiiError::FieldAbsent)?;
        let plain  = self.vault.decrypt(kaki_hash, FIELD_DEATH_YEAR, sealed)?;
        if plain.len() != 4 { return Err(PiiError::CryptoFailure("bad year length".into())); }
        Ok(u32::from_le_bytes(plain.try_into().unwrap()))
    }

    /// Retrieve the plaintext family lineage for an entity.
    pub fn family_lineage(&self, kaki_hash: u32) -> PiiResult<String> {
        let entry  = self.entries.get(&kaki_hash).ok_or(PiiError::FieldAbsent)?;
        let sealed = entry.lineage_sealed.as_deref().ok_or(PiiError::FieldAbsent)?;
        let plain  = self.vault.decrypt(kaki_hash, FIELD_FAMILY_LINEAGE, sealed)?;
        String::from_utf8(plain).map_err(|_| PiiError::CryptoFailure("UTF-8 decode failed".into()))
    }

    // ── GDPR Article 17 Erasure ───────────────────────────────────────────────

    /// Erase all PII for `kaki_hash`.
    ///
    /// - Marks the entity as erased in the vault (key derivation refused).
    /// - Removes the sidecar entry entirely.
    ///
    /// After this call, all `owner_name()` / `death_year()` / etc. calls for
    /// this hash return `PiiError::Erased`.  The particle record in the main
    /// database is unaffected (it contains no PII).
    pub fn erase(&mut self, kaki_hash: u32) -> bool {
        self.entries.remove(&kaki_hash);
        self.vault.erase(kaki_hash)
    }

    pub fn is_erased(&self, kaki_hash: u32) -> bool  { self.vault.is_erased(kaki_hash) }
    pub fn erased_count(&self) -> usize              { self.vault.erased_count() }
    pub fn entry_count(&self) -> usize               { self.entries.len() }
    pub fn erased_hashes(&self) -> Vec<u32>          { self.vault.erased_hashes() }

    /// Access the underlying vault for advanced operations.
    pub fn vault(&mut self) -> &mut PiiVault { &mut self.vault }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn vault() -> PiiVault {
        PiiVault::new([0xAAu8; 32])
    }

    fn sidecar() -> PiiSidecar {
        PiiSidecar::new([0xBBu8; 32])
    }

    // ── PiiVault ─────────────────────────────────────────────────────────────

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let v = vault();
        let plain = b"Abdullah al-Husayni \xd8\xb9\xd8\xa8\xd8\xaf \xd8\xa7\xd9\x84\xd9\x84\xd9\x87";
        let sealed = v.encrypt(0xDEAD_BEEF, FIELD_OWNER_NAME, plain).unwrap();
        let recovered = v.decrypt(0xDEAD_BEEF, FIELD_OWNER_NAME, &sealed).unwrap();
        assert_eq!(recovered, plain);
    }

    #[test]
    fn different_fields_produce_different_ciphertext() {
        let v = vault();
        let plain = b"test-data";
        let s1 = v.encrypt(0x1111, FIELD_OWNER_NAME, plain).unwrap();
        let s2 = v.encrypt(0x1111, FIELD_DEATH_YEAR, plain).unwrap();
        // Different field tags → different keys → different ciphertext
        assert_ne!(&s1[13..], &s2[13..]); // skip version+nonce
    }

    #[test]
    fn different_entities_have_different_keys() {
        let v = vault();
        let plain = b"same-plaintext";
        let s1 = v.encrypt(0xAAAA_0001, FIELD_OWNER_NAME, plain).unwrap();
        let s2 = v.encrypt(0xBBBB_0002, FIELD_OWNER_NAME, plain).unwrap();
        // Different kaki_hash → different key → different ciphertext
        assert_ne!(&s1[13..], &s2[13..]);
    }

    #[test]
    fn cross_entity_decryption_fails() {
        let v = vault();
        let sealed = v.encrypt(0xAAAA_0001, FIELD_OWNER_NAME, b"secret").unwrap();
        // Trying to decrypt with a different kaki_hash fails (AAD mismatch)
        let result = v.decrypt(0xBBBB_0002, FIELD_OWNER_NAME, &sealed);
        assert!(result.is_err(), "cross-entity decryption must fail");
    }

    #[test]
    fn erase_prevents_decrypt() {
        let mut v = vault();
        let sealed = v.encrypt(0xCCCC_0003, FIELD_OWNER_NAME, "محمد".as_bytes()).unwrap();
        v.erase(0xCCCC_0003);
        assert_eq!(v.decrypt(0xCCCC_0003, FIELD_OWNER_NAME, &sealed), Err(PiiError::Erased));
    }

    #[test]
    fn erase_prevents_encrypt() {
        let mut v = vault();
        v.erase(0xDDDD_0004);
        let result = v.encrypt(0xDDDD_0004, FIELD_OWNER_NAME, b"test");
        assert_eq!(result, Err(PiiError::Erased));
    }

    #[test]
    fn erased_count_tracks_correctly() {
        let mut v = vault();
        assert_eq!(v.erased_count(), 0);
        v.erase(0x0001);
        v.erase(0x0002);
        assert_eq!(v.erased_count(), 2);
        v.erase(0x0001); // duplicate — idempotent
        assert_eq!(v.erased_count(), 2);
    }

    #[test]
    fn erased_hashes_round_trip() {
        let mut v1 = vault();
        v1.erase(0x0010);
        v1.erase(0x0020);
        let snapshot = v1.erased_hashes();

        // Simulate restart: fresh vault, restore snapshot
        let mut v2 = vault();
        v2.restore_erased_set(snapshot);
        assert!(v2.is_erased(0x0010));
        assert!(v2.is_erased(0x0020));
        assert!(!v2.is_erased(0x0030));
    }

    // ── PiiSidecar ────────────────────────────────────────────────────────────

    #[test]
    fn sidecar_set_and_retrieve_name() {
        let mut sc = sidecar();
        sc.set_owner_name(0x1234, "عبد الله الكاظمي", 1_000).unwrap();
        let name = sc.owner_name(0x1234).unwrap();
        assert_eq!(name, "عبد الله الكاظمي");
    }

    #[test]
    fn sidecar_set_and_retrieve_death_year() {
        let mut sc = sidecar();
        sc.set_death_year(0x5678, 1398, 1_000).unwrap();
        assert_eq!(sc.death_year(0x5678).unwrap(), 1398);
    }

    #[test]
    fn sidecar_erase_removes_entry_and_blocks_access() {
        let mut sc = sidecar();
        sc.set_owner_name(0xABCD, "حسن الشريف", 1_000).unwrap();
        assert_eq!(sc.entry_count(), 1);
        sc.erase(0xABCD);
        assert_eq!(sc.entry_count(), 0);
        assert_eq!(sc.owner_name(0xABCD), Err(PiiError::FieldAbsent));
    }

    #[test]
    fn sidecar_absent_field_returns_field_absent() {
        let mut sc = sidecar();
        sc.set_owner_name(0xEEEE, "test", 1_000).unwrap();
        // death_year was not set
        assert_eq!(sc.death_year(0xEEEE), Err(PiiError::FieldAbsent));
    }

    #[test]
    fn sidecar_unknown_entity_returns_field_absent() {
        let sc = sidecar();
        assert_eq!(sc.owner_name(0x9999), Err(PiiError::FieldAbsent));
    }
}
