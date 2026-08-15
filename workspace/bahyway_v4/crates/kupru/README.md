# kupru — Sovereign Cryptographic Security Layer
𒀊𒈗𒆪 *kanākum* — "to seal with a cylinder seal"

> **Layer 9.1 · Sovereign Crypto** | Ed25519 · ChaCha20-Poly1305 · Argon2id · SHA3-512

---

## W5H2 Manual

### WHO — 𒀭 Who Uses This Crate

| Persona | Role |
|---|---|
| **File system layer** (`crates/enkidb-persist`, `permanent-storage`) | Seals and verifies `.akk` files at rest |
| **Pipeline stations** (adad-gate, musaru-security) | Issues and validates `SargonPassport` credentials |
| **AkkadiKernel** (`bin/akkadi-cli`) | Derives keys from Akkadian passphrases via `SargonKdf` |
| **Istar access control** (`crates/istar`) | Imports `SargonPassport` to check scopes and timing |
| **Any crate that writes sovereign data** | Must seal output with `AkkadianSeal` before persistence |

---

### WHAT — 𒁾 What This Tablet Contains

`kupru` is the **single source of truth for all cryptographic operations** in BahyWay.Ecosystem.  
It implements the **ŠARRUKIN five-layer `.akk` file format** and the **four-layer SargonPassport**.

#### Five-Layer `.akk` File Format (ŠARRUKIN)

```
┌──────────────────────────────────────────────────────┐
│  Layer 0 — RĒŠU (Head)                               │
│  Magic: ACKv4 | Version: 0x04 | Domain: KAKI B10     │
│  Timestamp: sovereign_now() | Flags: 0x0000           │
├──────────────────────────────────────────────────────┤
│  Layer 1 — MUDÛ (Knowledge)                          │
│  Name | Quality (0–240) | Creator KAKI PK | Tags     │
├──────────────────────────────────────────────────────┤
│  Layer 2 — KANĀKU (Seal)                             │
│  Algorithm: 0x01=Ed25519 | Verifier key | Signature  │
├──────────────────────────────────────────────────────┤
│  Layer 3 — ŠIPRU (Work / Payload)                    │
│  The actual content bytes (encrypted or plain)       │
├──────────────────────────────────────────────────────┤
│  Layer 4 — ŠIPIR ŠARRI (Royal Work / Integrity)      │
│  Algorithm: 0x03=SHA3-256 | 32-byte digest           │
└──────────────────────────────────────────────────────┘
```

#### Four-Layer SargonPassport (ŠARRUKIN ṬUPPU)

```
┌──────────────────────────────────────────────────────┐
│  QUPPU  (Basket)   UUID v4 | created_at | expires_at │
│                    issuer_kaki [u8; 16]               │
├──────────────────────────────────────────────────────┤
│  KUPRU  (Bitumen)  SHA3-512 content hash             │
│                    HMAC-SHA3-512 | nonce [u8; 16]    │
├──────────────────────────────────────────────────────┤
│  NĀRU   (River)    subject_kaki | AkkadianName       │
│                    LinguisticProof | realm | mudu     │
├──────────────────────────────────────────────────────┤
│  IŠTAR  (Favor)    isretu_scopes | access_matrix     │
│                    nadanu_chain | privilege_level 1–7 │
└──────────────────────────────────────────────────────┘
       │
       ▼
  outer_seal (Ed25519 over canonical binary bytes — Fix 4)
```

#### Modules at a glance

| Module | What it provides |
|---|---|
| `akk_cipher` | `AkkadianCipher` — ChaCha20-Poly1305 AEAD |
| `akkadian_seal` | `AkkadianSeal` trait, `SealKeyPair` (Ed25519), `SovereignVerifier` |
| `akkadian_root` | `AkkadianRoot`, `LinguisticProof` (Fix 2: 32-byte nonce) |
| `akk_file` | `AkkFile`, `ResuHeader`, `MuduMetadata` — ŠARRUKIN format |
| `sargon_kdf` | `SargonKdf` — Argon2id KDF, `SATTATU_MAX`, timing guard |
| `sovereign_hash` | `SovereignHash` trait, SHA3-512 (default), SHA3-256 (fast) |
| `passport` | `SargonPassport`, all four layers |
| `passport_seal` | Fix 4 — canonical binary signing |
| `canonical` | `CanonicalBytes` — deterministic byte accumulator |
| `architect_key` | `ArchitectKeyShare`, Shamir M-of-N split/reconstruct + `issue_architect_passport` — see `ARCHITECT_KEY_CEREMONY.md` |
| `error` | `KupruError` (6 variants) |

---

### WHEN — 𒌓 When Is This Invoked

```
Credential lifecycle
─────────────────────────────────────────────────────────
1. ISSUANCE   SargonKdf::new(root, passphrase)
              → derive_key(passphrase)
              → AkkadianCipher::from_key_bytes(key)
              → SealKeyPair::generate()
              → SargonPassport::issue(naru, istar, kaki, keypair, integrity_key)
              → passport_seal() [Fix 4 — canonical binary]

2. STORAGE    AkkFile::new(domain, name, quality, creator_id, payload)
              → file.sign(&seal_keypair)           [KANĀKU layer]
              → file.to_bytes()                    [JSON envelope]

3. RETRIEVAL  AkkFile::from_bytes(data)
              → file.verify_integrity()            [ŠIPIR ŠARRI]
              → file.verify_seal(&verifier)        [KANĀKU]

4. AUTH CHECK SargonPassport::verify_seal()        [timing + outer seal]
              → passport.has_scope("enki:write")
              → istar::AkkFirewall::evaluate(ctx)
```

---

### WHERE — 𒆳 Architectural Position

```
                    kupru
                   ┌─────────────────────────────────┐
                   │  Ed25519  ChaCha20  Argon2id     │
                   │  SHA3-512  SHA3-256  HMAC         │
                   └──────────────┬──────────────────┘
                                  │ used by
            ┌─────────────────────┼──────────────────────┐
            │                     │                      │
         istar              enkidb-persist         musaru-security
    (access control)     (file sealing)          (pipeline gate)
            │
         All crates that perform writes
```

`kupru` has **no dependencies on any BahyWay crate** — it is the security foundation.

---

### WHY — 𒀊 Why This Exists

**Why Ed25519 (not RSA)?**  
32-byte keys, fast constant-time ops, no padding oracle, post-quantum upgrade path to Dilithium  
without changing the `AkkadianSeal` trait interface (algorithm_id becomes 0x02).

**Why Argon2id with 54 iterations?**  
54 = Sargon's 54-year reign (`SATTATU_MAX`). Intentionally symbolic and publicly visible.  
Memory-hard (64 MiB) makes GPU brute-force economically prohibitive.

**Why 32-byte nonce in `LinguisticProof` (Fix 2)?**  
16-byte nonce → birthday bound at 2^64 collisions (~4 billion files, unsafe for long-lived systems).  
32 bytes → birthday bound at 2^128, computationally unbreachable.

**Why canonical binary for passport sealing (Fix 4)?**  
JSON field ordering is not guaranteed across serde versions. A serde update could silently  
invalidate all existing passports. Canonical binary (`CanonicalBytes`) is deterministic forever.

**Why `SATTATU_MAX = 54 hours`?**  
No credential in BahyWay.Ecosystem may outlive 54 hours (194,400 seconds). This is a  
constitutional constraint, not a configuration option. It prevents credential hoarding.

---

### HOW — 𒅗 How It Works

#### Seal a payload

```rust
use kupru::{AkkadianCipher, AkkFile, SealKeyPair};

// 1. Generate a key pair for the issuer
let keypair = SealKeyPair::generate()?;

// 2. Build and seal an .akk file
let mut file = AkkFile::new(
    0x03,           // domain byte (KAKI B10)
    "sensor_batch", // name
    185,            // quality (TribeMember lane)
    "adad-gate",    // creator ID
    payload_bytes,
);
file.sign(&keypair);  // populates KANĀKU layer

let sealed = file.to_bytes()?;
```

#### Derive a key from an Akkadian passphrase

```rust
use kupru::{SargonKdf, AkkadianRoot, VowelPattern};

let root = AkkadianRoot::new('Š', 'R', 'K', VowelPattern::Ui); // Sargon
let kdf  = SargonKdf::new(root, b"my-sovereign-passphrase")?;
let key  = kdf.derive_key(b"my-sovereign-passphrase")?; // [u8; 32]
// Argon2id: 64 MiB · 4 threads · 54 iterations
```

#### Issue a SargonPassport

```rust
use kupru::{SargonPassport, NaruLayer, IshtarLayer, SealKeyPair};

let istar_layer = IshtarLayer::gardener("pollution");  // read-only
let naru_layer  = NaruLayer { subject_kaki: [0u8;16], /* … */ };
let keypair     = SealKeyPair::generate()?;
let integrity   = [0x42u8; 32]; // derive from SargonKdf in production

let passport = SargonPassport::issue(
    naru_layer, istar_layer, [0u8;16], &keypair, &integrity
)?;
assert!(passport.has_scope("pollution:read"));
```

#### Validate a credential

```rust
passport.verify_seal()?;    // timing + Ed25519 outer seal
// Err(KupruError::Expired) if now > expires_at
// Err(KupruError::SealBroken) if signature invalid
```

---

### HOW MUCH — 𒀸 Sovereign Metrics

| Metric | Value |
|---|---|
| Source files | 10 |
| Lines of Rust | ~1,200 |
| Algorithms | ChaCha20-Poly1305, Ed25519, Argon2id, SHA3-512, SHA3-256, HMAC-SHA3 |
| KDF iterations | **54** (Sargon's 54-year reign) |
| KDF memory | 64 MiB |
| `SATTATU_MAX` | 54 hours (194,400 seconds) |
| Nonce size (LinguisticProof) | **32 bytes** (birthday bound 2^128) — Fix 2 |
| Signing | **Canonical binary** (not JSON) — Fix 4 |
| KupruError variants | 6 |

---

## Sovereign Constraints

- `#![forbid(unsafe_code)]` — zero unsafe Rust
- Signing key **zeroized on drop** (`ZeroizeOnDrop` on `SealKeyPair`)
- Cipher key **zeroized on drop** (`ZeroizeOnDrop` on `AkkadianCipher`)
- `SealKeyPair::Debug` impl **redacts signing key** (`[REDACTED]`)
- `QUALITY_DIVISOR = 240.0` — KAKI B11 quality byte (ADR-001)
- `COMMITMENT_DOMAIN = b"BahyWay.LinguisticProof.v353"` — protocol constant, immutable

---

## Files

```
crates/kupru/
├── Cargo.toml          (deps: serde, zeroize, rand, sha3, subtle,
│                         chacha20poly1305, ed25519-dalek, argon2, hmac, uuid)
└── src/
    ├── lib.rs           — crate root, all re-exports
    ├── error.rs         — KupruError (6 variants), KupruResult
    ├── canonical.rs     — CanonicalBytes, domains::SARGON_PASSPORT
    ├── akkadian_root.rs — AkkadianRoot, LinguisticProof (Fix 2)
    ├── akkadian_seal.rs — AkkadianSeal trait, SealKeyPair, SovereignVerifier
    ├── akk_cipher.rs    — AkkadianCipher (ChaCha20-Poly1305), CipherText
    ├── akk_file.rs      — AkkFile, ResuHeader, MuduMetadata, ShipirSharri
    ├── sargon_kdf.rs    — SargonKdf (Argon2id), SATTATU_MAX, timing guard
    ├── sovereign_hash.rs— SovereignHash trait, Sha3Digest512/256, HMAC
    ├── passport.rs      — SargonPassport, all four layers
    └── passport_seal.rs — Fix 4: passport_canonical_bytes/seal/verify_seal
```
