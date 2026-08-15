# pii-vault — Encrypted PII Field Store

Encrypted storage for personally identifiable information (PII) with GDPR Article 17 "forgotten key" erasure. Provides both a low-level field-encryption API (`PiiVault`) and a structured sidecar store for grave-record entities (`PiiSidecar`).

## Architecture

```
GraveParticle (public record — kaki_hash only)
         │
         ▼
PiiSidecar ── per-entity PiiSidecarEntry (name_sealed, year_sealed, …)
         │
         ▼
PiiVault ── encrypt/decrypt via AkkadianCipher (ChaCha20-Poly1305)
         │
         ▼
kupru::fast_hasher() ── SHA3-256 key derivation
```

No plaintext PII ever touches the particle database. The sidecar stores only sealed ciphertext; plaintext is only materialised on demand by an authorised caller holding the master key.

## Key Derivation

```
field_key = SHA3-256(master_key ‖ kaki_hash.to_le_bytes() ‖ field_tag)
```

- `master_key` — 32-byte deployment secret (rotate with `PiiVault::restore_erased_set`)
- `kaki_hash` — entity identity (FNV-1a over all 16 KAKI bytes)
- `field_tag` — 1-byte domain-specific constant (see `FIELD_*` constants)

Different field tags → different keys per entity. Same field on two entities → different keys. There is no shared key for two entities.

## GDPR Article 17 Erasure — "Forgotten Key" Pattern

```
erase(kaki_hash)
  → adds kaki_hash to HashSet<erased>
  → all subsequent encrypt/decrypt calls for that hash return PiiError::Erased
```

The ciphertext in the sidecar is **not deleted**. The key is simply never derived again. Without the key, the ciphertext is permanently unreadable. This is compliant with GDPR Art.17 cryptographic erasure (Recital 26: data rendered unintelligible = no longer personal data).

Erasure is idempotent and can be persisted via `erased_hashes()` + `restore_erased_set()` across restarts.

## AAD Binding

The authentication tag covers `kaki_hash.to_le_bytes()` as Additional Authenticated Data (AAD). This means:

- Ciphertext encrypted for entity `A` **cannot** be decrypted under entity `B`'s key, even if the key bytes accidentally collide.
- Transcript confusion attacks (swapping sealed fields between entities) are rejected at the Poly1305 tag verification step.

## Field Tag Constants

| Constant               | Value | Used for                         |
|------------------------|-------|----------------------------------|
| `FIELD_OWNER_NAME`     | `1`   | Arabic personal name             |
| `FIELD_DEATH_YEAR`     | `2`   | Hijri death year                 |
| `FIELD_TRIBE_DETAIL`   | `3`   | Extended tribe information       |
| `FIELD_FAMILY_LINEAGE` | `4`   | Nasab / lineage string           |
| `FIELD_CIVIL_REF`      | `5`   | Civil registration reference     |
| `FIELD_WPD_OPERATOR`   | `6`   | WPD operator personal data       |

## PiiVault API

```rust
let vault = PiiVault::new(&master_key_32_bytes);

// Encrypt a field
let sealed: Vec<u8> = vault.encrypt(kaki_hash, FIELD_OWNER_NAME, plaintext)?;

// Decrypt a field
let plaintext: Vec<u8> = vault.decrypt(kaki_hash, FIELD_OWNER_NAME, &sealed)?;

// GDPR erasure
vault.erase(kaki_hash);

// Restore after restart
vault.restore_erased_set(previously_erased_hashes);

// Audit
vault.erased_count();
vault.erased_hashes();  // returns &HashSet<u32>
```

### Error Types

| `PiiError` variant       | Meaning                                      |
|--------------------------|----------------------------------------------|
| `Erased`                 | Entity has been GDPR-erased                  |
| `CryptoFailure(String)`  | AEAD seal/open error (bad ciphertext/tag)    |
| `InvalidInput(&str)`     | Empty plaintext or zero-length ciphertext    |
| `FieldAbsent`            | Sidecar entry exists but field not set       |

## PiiSidecar API

```rust
let mut sidecar = PiiSidecar::new(&master_key_32_bytes);

// Store fields
sidecar.set_owner_name(kaki_hash, "محمد النجفي".as_bytes())?;
sidecar.set_death_year(kaki_hash, &1443u32.to_le_bytes())?;
sidecar.set_family_lineage(kaki_hash, lineage_bytes)?;

// Retrieve fields (returns decrypted bytes)
let name_bytes: Vec<u8> = sidecar.owner_name(kaki_hash)?;
let year_bytes: Vec<u8> = sidecar.death_year(kaki_hash)?;

// GDPR erasure — marks entity erased in vault; sidecar entry record remains
sidecar.erase(kaki_hash);

// Audit
sidecar.entry_count();
sidecar.erased_count();
sidecar.is_erased(kaki_hash);
```

## Wired Ciphertext Format

Delegated to `kupru::AkkadianCipher` (ChaCha20-Poly1305):

```
version (1 byte) ‖ nonce (12 bytes) ‖ ciphertext+tag (n + 16 bytes)
```

Total overhead per field: 29 bytes.

## Compliance

| Requirement | SLA ID | Status   |
|-------------|--------|----------|
| PII field encryption at rest | 2001 | Implemented |
| GDPR Art.17 erasure mechanism | 1004 | Implemented |
| Cross-entity AAD binding | 2001 | Implemented |
| PII sidecar (NajafEngine) | 5001 | Implemented |
| Key rotation compatibility | 2101 | `restore_erased_set()` supports re-keying |

## Dependencies

- `kupru` — `AkkadianCipher` (ChaCha20-Poly1305), `fast_hasher()` (SHA3-256)
- `bahyway-core` — `BahyWayError`, common types
- No external crates. No unsafe code.

## See Also

- `crates/kupru/MANUAL.md` — cipher and hasher internals
- `crates/sla-engine/MANUAL.md` — compliance scoring (req 2001, 5001)
- `crates/enkidb-kaki/MANUAL.md` — KAKI byte layout and `uuid_hash()`
- `policies/sla_governance_protocol.akk` — `PiiEncryptionGuard`, `ErasureGuard`
