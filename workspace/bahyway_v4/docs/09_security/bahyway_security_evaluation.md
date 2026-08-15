# BahyWay Ecosystem — Honest Security Evaluation

> **DubSar Help** | `Docs > 09 Security > Security Evaluation` | Assessment

---

## Purpose

This document provides an honest, unvarnished evaluation of the security and
privacy posture of the BahyWay v4.0 ecosystem as of 2026-06-03.  It separates
what is genuinely strong from what has real gaps, and gives a concrete action
list ordered by severity.

---

## What Is Being Protected

- **People's identities** — KAKI sovereign particles tied to real individuals,
  family lineages, and burial records (Wadi al-Salam, NajafEngine)
- **Sensitive personal data** — Arabic names, death years, tribe affiliations,
  family trees (`FamilyLineage`, `GraveParticle`)
- **Data flow pipelines** — replication between Write Pod and Read Pod
- **Web sessions** — HeptaScript website, DubSar IDE, EriduOS notebooks
- **Infrastructure records** — WPD pipeline data (pressure, leak candidates)

---

## Genuine Strengths

### Cryptographic Foundations — Strong Choices

| Component | Choice | Why It Is Good |
|-----------|--------|----------------|
| Asymmetric signing | Ed25519 | 128-bit security, constant-time, immune to timing attacks, no random per-signature |
| Symmetric encryption | ChaCha20-Poly1305 | AEAD (confidentiality + integrity in one pass), TLS 1.3 preferred cipher, safe on non-AES hardware |
| Key derivation | Argon2id | Gold standard, memory-hard (64 MiB), GPU-resistant, correct choice over PBKDF2/bcrypt |
| Hashing | SHA3-256/512 | NIST post-quantum safe hash, unlike SHA2 not structurally related to MD5 |
| Comparison | `subtle::ConstantTimeEq` | Prevents timing side-channel attacks on credential verification |
| Key zeroing | `ZeroizeOnDrop` on `SealKeyPair` | Signing keys cleared from memory immediately on drop |

### Architecture — What Is Genuinely Novel

1. **KAKI Dead Axiom** — "A packet without a valid KAKI cannot exist."  This is
   architecturally stronger than most systems, which have authenticated endpoints
   alongside unauthenticated ones.  Here unauthenticated = non-existent.

2. **SargonPassport SATTATU_MAX = 54 hours** — Short credential lifetime.  A
   stolen credential is useless after 54 hours without renewed identity proof.

3. **CQRS + Podman blast-radius isolation** — Frontend compromise is physically
   contained to a read-only pod.  No network path from Read Pod to Write Pod.

4. **7-layer replication verification** — Each replication event must pass frame
   integrity, epoch freshness, sequence monotonicity, chained digest, Ed25519
   KANĀKU seal, SHA3-256 ŠIPIR ŠARRI, and HeptaSecSentinel independently.
   All seven must fail simultaneously for a breach — no shared failure mode.

5. **Chained hash log** — The `.enkwal` append-only log forms a hash chain.
   Deleting, reordering, or injecting events breaks the chain detectably.

6. **CanonicalBytes (Fix 4)** — All signing uses deterministic binary
   serialisation, not JSON.  A serde version bump cannot silently invalidate
   existing credentials.

7. **`#![forbid(unsafe_code)]` on every crate** — Eliminates entire classes of
   memory safety vulnerabilities (buffer overflows, use-after-free, dangling
   pointers) that account for ~70% of CVEs in systems software.

8. **LinguisticProof 32-byte nonce (Fix 2)** — Birthday bound at 2^128
   (computationally unbreachable), upgraded from the original 16-byte nonce
   (2^64 — potentially reachable in a long-lived system).

---

## Honest Weaknesses

These are real gaps, ordered from most to least critical.

---

### CRITICAL

#### 1. FNV-1a is Not a Cryptographic Hash

`uuid_hash()` on a KAKI returns a `u32` computed via FNV-1a.  This is used as
the KAKI identity key throughout `HeptaSecSentinel`, `KakiFirewall`, and the
`KakiPacket` src/dst fields.

**The problem:** FNV-1a has trivially computable collisions.  An adversary who
controls their own KAKI registration can engineer `my_kaki.uuid_hash() ==
trusted_kaki.uuid_hash()` in milliseconds, then impersonate the trusted KAKI
at the firewall.

**The fix:** Replace `uuid_hash() → u32` with a truncated SHA3-256 output.
A 64-bit identifier (`u64`) eliminates the birthday-bound problem for any
realistic system size.  Migration requires updating all KAKI hash storage.

---

#### 2. PolicyCondition::RateExceeds Always Returns False

Rate limiting is the most commonly needed defense against brute-force login
attacks and DDoS on the `Quarantine` queue.  It is stubbed out in Phase 1:

```rust
PolicyCondition::RateExceeds { .. } => false
```

An attacker can flood the system with Unknown-KAKI packets indefinitely.
Each generates a `SecurityEvent`, filling the 500-entry ring in seconds and
destroying audit evidence of the actual attack.

**The fix:** Implement a sliding-window counter (token bucket, no external crate
needed — 50 lines of pure Rust) keyed by `src_kaki_hash` or source IP.

---

#### 3. Security Event Ring Buffer Overflow = Evidence Destruction

`HeptaSecSentinel` holds 500 events; `KakiFirewall` holds 1000.  Under a
sustained flood attack (or even normal high traffic), old events are silently
evicted.

A security audit system that destroys its own evidence under load is not a
security system — it is a liability.

**The fix:** Persist `SecurityEvent`s to an append-only log file (the same
`.enkwal` format used by replication).  The ring buffer becomes a
recent-events cache; the file is the authoritative record.

---

### HIGH

#### 4. No Post-Quantum Asymmetric Cryptography

Ed25519 is broken by a sufficiently large quantum computer via Shor's algorithm.
The `algorithm_id` hook exists (`0x01 = Ed25519, 0x02 = Dilithium`) but
Dilithium-3 is not implemented.

For a system claiming 54-year sovereign lifetimes (Sargon's reign), "harvest
now, decrypt later" attacks are a real threat.  An adversary capturing today's
KANĀKU-sealed replication events can decrypt them when quantum hardware matures.

**The risk window:** NIST estimates cryptographically relevant quantum computers
in the 2030–2040 range.  For a system storing 54-year identity records, this
falls within the data lifetime.

**The fix:** Implement `algorithm_id = 0x02` path in `SealKeyPair` and
`SovereignVerifier` using the `pqcrypto-dilithium` crate (or wait for
`rustcrypto` Dilithium).  No structural change needed — the hook is already
there.

---

#### 5. No Key Rotation Procedure

Once the Write Pod's `SealKeyPair` is deployed, there is no documented or
implemented procedure for rotating it.  If the signing key is compromised:
- All previously sealed events remain valid (attacker can replay them)
- The Broker has no way to distinguish old-key from new-key events
- The only option is to rebuild the chain from scratch

**The fix:** Add a `KeyRotationEvent` to `ReplEventKind` containing the new
verifying key, signed by the old key.  The Broker transitions to the new key
after verifying the rotation event.

---

#### 6. Key Storage at Rest Not Specified

The `SealKeyPair` holds a 32-byte Ed25519 signing key.  `ZeroizeOnDrop`
protects it in memory.  But where does it live on disk?

If it is a plain file (`write_pod.key`), host compromise = key compromise =
attacker can forge any replication event forever.

**The fix:** Encrypt the key at rest using `AkkadianCipher` (ChaCha20-Poly1305)
with a passphrase derived via `SargonKdf` (Argon2id).  Or use a hardware
security module (HSM) / TPM for the Write Pod on production hardware.

---

#### 7. GDPR Article 17 — Right to Erasure Conflicts With Append-Only Design

`GraveParticle` stores `owner_name_arabic`, `death_hijri_year`, `tribe` — PII
under GDPR and Iraqi personal data protection law, even for deceased persons
whose living relatives can invoke erasure rights.

`FamilyLineage` stores family trees — sensitive genealogical data.

The `.enkwal` chained hash log makes deletion structurally impossible: removing
any event breaks the chain for all subsequent events.

**This is an unresolved architectural conflict.**

**Mitigation options (none perfect):**
- Encrypt PII fields at rest with per-person keys; erasure = key deletion
  (data remains but becomes unreadable — accepted by some GDPR interpretations)
- Store PII in a separate erasable sidecar; the log contains only KAKI hashes
  (no names/dates in the log itself)
- Implement "log compaction" with a new genesis hash after erasure events

---

### MEDIUM

#### 8. purge_stale() Always Returns 0

The Sentinel's `purge_stale()` acknowledges the bug inline:

```rust
// active_entries doesn't change on expire_stale since entries remain
let purged = before_count.saturating_sub(after_count); // always 0
```

`SentinelStats.events_emitted` counting the `StalePurged` event is misleading —
operators think 0 entries were purged when in reality entries may have been
downgraded to Unknown.

**The fix:** Track a separate `downgraded_count` counter in `expire_stale()`.

---

#### 9. SargonPassport Renewal Has No Documented Fallback

The Write Pod's passport expires every 54 hours.  If the issuing authority is
unreachable during renewal:
- The Broker's `passport_validator` returns `PassportExpired`
- Replication halts completely
- The Read Pod serves stale data indefinitely

**The fix:** Define a grace period (e.g., 1 extra hour) and a "safe halt"
behaviour: Read Pod switches to read-only-of-last-known-state and emits an
alert, but does not serve error responses to users.

---

#### 10. hepta-sec-web Adapter Not Built

The internet-facing component — the most attacked surface in any system — does
not yet integrate with `HeptaSecSentinel`.  Until the web adapter is built and
deployed, the sophisticated firewall architecture has **no enforcement point at
the actual HTTP boundary**.

Packets arrive at Nginx/Actix without KAKI extraction.  All the HeptaSec logic
sits unused for web traffic.

**The fix:** Build `hepta-sec-web` (estimated 100 lines) before the HeptaScript
website is publicly reachable.

---

#### 11. DubSar IDE Execution Environment Not Assessed

A web-based notebook/code editor is one of the highest-risk components in any
system.  It requires:
- User code execution (sandbox escape risk)
- Arbitrary content rendering (XSS risk)
- File system access (path traversal risk)

None of the HeptaSec architecture has been applied specifically to the notebook
execution environment.  This needs a separate threat model.

---

### LOW (Important but Not Immediate)

#### 12. LinguisticProof Entropy Not Formally Analyzed

The `LinguisticProof` commitment uses Akkadian tri-consonantal roots as identity
entropy material.  The commitment scheme `SHA3-256(domain || phrase || nonce)`
is cryptographically sound.  However:
- How many distinct valid Akkadian roots exist? (~2,000–3,000)
- Are they evenly distributed? No — some roots are far more common
- Can an adversary enumerate them in a dictionary attack?

The Argon2id KDF in `SargonKdf` mitigates enumeration (64 MiB per attempt),
but a formal entropy analysis has not been published.

#### 13. No Formal Threat Model Document

No STRIDE, PASTA, or attack tree document exists.  This makes it difficult to
reason about whether all relevant threats are addressed, and makes security
reviews by external auditors difficult.

---

## Privacy Assessment — Protecting People's Identities

### What Is Being Done Well

- **Sovereignty**: KAKI identities are sovereign particles — not entries in
  AWS Cognito, Google Identity, or any third-party system.  The system operator
  cannot be compelled to hand over a user database to a third party.
- **Credential expiry**: 54-hour SATTATU_MAX limits the window of identity
  theft from a stolen credential.
- **Memory safety**: ZeroizeOnDrop means identity key material does not linger
  in process memory or swap.
- **Argon2id**: Password-based identity theft requires ~64 MiB RAM per guess,
  making GPU-based attacks impractical.
- **No plaintext credential storage**: Passwords are not stored — only
  Argon2id-derived keys.

### What Needs Attention

- **No encryption at rest for EnkiDB volumes by default**: Arabic names,
  death years, and family lineage data are stored in Podman volumes.
  If the host is compromised, this data is readable without the KAKI.
- **GDPR erasure conflict** (see above — Critical).
- **Access level enforcement**: `ATTR_ACCESS_LEVEL` EAV attribute (0–3) exists
  but no enforcement story has been implemented.  Public vs private vs
  restricted data is not enforced at query time.
- **Audit trail for data access**: Who read which grave record, when?
  No access log exists.

---

## Ratings

| Domain | Score | Notes |
|--------|-------|-------|
| Cryptographic primitives | **9/10** | Ed25519 + ChaCha20 + Argon2id + SHA3 — excellent |
| Identity model (KAKI) | **7/10** | Strong concept; FNV-1a hash weakness is real |
| Data integrity | **8/10** | Chained hash log + multi-seal is solid |
| Network security | **5/10** | HeptaSec strong but web adapter not built |
| Key management | **4/10** | No rotation, no HSM, no at-rest encryption story |
| Privacy compliance | **4/10** | GDPR erasure vs append-only is unresolved |
| Quantum resistance | **3/10** | Algorithm hook exists; Dilithium not implemented |
| Audit and monitoring | **4/10** | Ring buffers overwritten; no persistent audit log |
| **Overall** | **6/10** | Architecturally sophisticated foundation with real operational gaps |

---

## Recommended Action Priority

| Priority | Action | Effort |
|----------|--------|--------|
| P0 | Replace FNV-1a KAKI hash with truncated SHA3-256 (u64) | 2 days |
| P0 | Build `hepta-sec-web` adapter — KAKI extraction middleware | 1 day |
| P1 | Implement rate limiting (`PolicyCondition::RateExceeds`) | 1 day |
| P1 | Persist SecurityEvents to append-only `.enkwal` log | 1 day |
| P1 | Encrypt KAKI signing keys at rest (AkkadianCipher + SargonKdf) | 2 days |
| P2 | Fix `purge_stale()` counter — track downgraded entries | 2 hours |
| P2 | GDPR erasure strategy — PII in erasable sidecar vs encrypted fields | 1 week |
| P2 | SargonPassport renewal grace period and safe-halt procedure | 1 day |
| P3 | Key rotation procedure (`KeyRotationEvent` in replication log) | 3 days |
| P3 | Post-quantum signing (Dilithium-3) via `algorithm_id = 0x02` | 1 week |
| P3 | Formal threat model document (STRIDE analysis) | 3 days |
| P4 | DubSar IDE execution sandbox threat model | 1 week |
| P4 | Linguistic entropy analysis for `LinguisticProof` roots | research |

---

## Summary

BahyWay v4.0 is **architecturally more thoughtful than most systems of its
size**.  The KAKI sovereignty model, SargonPassport multi-layer credentials,
Argon2id KDF, and CQRS Podman isolation are genuine innovations in sovereign
identity design.  The cryptographic primitive choices are largely correct.

The system is **not yet production-ready for protecting real people's
identities** due to three unresolved gaps:

1. FNV-1a KAKI hash is cryptographically forgeable (P0).
2. The web adapter is not built — HeptaSec has no enforcement point (P0).
3. GDPR erasure conflicts with the append-only design (P2, but requires
   architectural decision before more data is stored).

Addressing P0 items before the HeptaScript website goes live is essential.
Everything else can be addressed iteratively after launch.
