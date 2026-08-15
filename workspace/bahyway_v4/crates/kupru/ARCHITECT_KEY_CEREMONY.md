# The Architect Key Ceremony

How DUB.SAR 𒁾 (Bahaa Fadam) holds root trust over BahyWay.Ecosystem
without that trust ever being a single point of failure.

This document is the operational half of `crates/kupru/src/architect_key.rs`.
The code makes the ceremony *possible*; this document is what makes it
*safe*. Skipping the process below and just generating-and-storing a key
somewhere convenient defeats the entire point of splitting it.

---

## The core idea, in one sentence

**No single file, device, or person can ever reconstruct the Architect's
signing key alone.** Reconstructing it always requires a threshold number
of independently-held shares, and the key itself never *decrypts* or
*unlocks* anything directly — it only ever mints a fresh, normal,
time-boxed `SargonPassport` at architect privilege.

## Step 1 — Generate the root key (once, offline)

Run this on a machine that will **never** be a running server — a laptop
you then disconnect from the network, ideally. Never eriduous-vdi, never
any Podman/VM host that runs live services.

```rust
use kupru::SealKeyPair;
let root_keypair = SealKeyPair::generate()?;
```

The **public** half (`root_keypair.verifying_key_bytes()`) is what gets
distributed to every service in the ecosystem as the trusted architect
root — it's not secret, publish it freely (it's what every
`SargonPassport::verify_seal()` call checks architect-level passports
against).

The **private** half never gets written to disk as a single file. Go
straight to Step 2.

## Step 2 — Split it (M-of-N)

```rust
use kupru::split_architect_key;
let shares = split_architect_key(&root_keypair, /* threshold */ 3, /* total */ 5)?;
```

**Recommended starting point: 3-of-5.** Five shares, any three of which
reconstruct the key; any one or two, alone, reveal nothing.

Choosing M and N is a real tradeoff, not a formality:
- **Higher threshold (M closer to N)** = harder for an attacker who
  compromises multiple locations to reconstruct, but higher risk you
  lose legitimate access if too many holders become unreachable.
- **Lower threshold relative to N** = more resilient to holders being
  unreachable, but a smaller fraction of compromised locations is
  enough to reconstruct.
- 3-of-5 is a reasonable starting balance for a single architect with a
  handful of trusted backup locations. Revisit if the ecosystem grows a
  real multi-person leadership structure.

## Step 3 — Distribute shares (this is the part that actually matters)

Each of the 5 shares must go to a **genuinely different** location.
"Different" means different failure domain, not just a different folder:

| Share | Suggested location | Failure domain it survives |
|---|---|---|
| 1 | Hardware key (YubiKey or similar) you carry | Your laptop being stolen/wiped |
| 2 | Encrypted file on offline USB drive, in a physical safe | A ransomware/compromise of any online device |
| 3 | Encrypted file with a second trusted person (family, co-founder) | You being unreachable |
| 4 | Safe-deposit box or equivalent physical offsite storage | A fire/disaster at your primary location |
| 5 | Encrypted cloud storage (separate provider from your primary infra) | All physical locations being lost simultaneously |

Never store two shares in the same place "for convenience." Never email
a share in plaintext. Never store a share alongside its `split_id` and
enough context for someone to guess it needs only 2 more.

## Step 4 — Destroy the unsplit key

Once shares are distributed and you've verified (Step 5) that
reconstruction actually works, securely erase any copy of the full,
unsplit private key. It should not continue to exist anywhere after
this point — only the shares should.

## Step 5 — Verify the ceremony actually worked

Before trusting this as your recovery path, prove it:

```rust
use kupru::reconstruct_architect_key;
let test_reconstruction = reconstruct_architect_key(&shares[0..3])?; // any 3
assert_eq!(test_reconstruction.verifying_key_bytes(), root_keypair.verifying_key_bytes());
```

Do this with at least two *different* combinations of 3 shares before
destroying the original, to catch any corrupted share early while the
original still exists as a fallback.

## Break-glass: using the key

When you actually need architect-level access (e.g. every existing
credential in some part of the ecosystem is inaccessible):

1. Gather any `threshold` (e.g. 3 of 5) shares from their distributed
   locations.
2. `reconstruct_architect_key(&gathered_shares)` — this exists in memory
   only for the duration of the next step.
3. `issue_architect_passport(&reconstructed_key, ...)` — mints a normal
   `SargonPassport` at privilege level 7, full wildcard scope, bound to
   the same `SATTATU_MAX` (54h) expiry as every other passport in the
   ecosystem. No exception is carved out for architect passports — even
   this credential dies on schedule.
4. Let `reconstructed_root` drop immediately after minting — it
   zeroizes automatically. Do not hold it longer than the single mint
   operation requires.
5. **Log that this happened.** Break-glass access should be rare enough
   that every use is worth a note of when and why, even if only to
   yourself.

## Rotation — the root key itself is not permanent either

Rotate the root key (generate a brand-new one, re-run the whole ceremony,
distribute new shares) on:
- **A deliberate schedule** — annually is a reasonable default for a
  single-architect ecosystem.
- **Immediately**, if you have any reason to suspect a share location
  was compromised — even one you're not certain about. Reconstructing
  requires a threshold, but "I'm not sure that USB drive is still only
  in my safe" is reason enough to rotate rather than hope.

Rotating means:
1. Generate a new root keypair and repeat Steps 1-5 above for it.
2. Distribute its public half to every service that checks architect
   passports (this is the part that requires actual deployment work —
   plan for it, don't treat rotation as free).
3. Once the new root is live everywhere, treat the old root's public key
   as revoked — any passport signed by it should no longer be trusted,
   even one that hasn't technically expired yet.
4. Destroy the old shares.

## What this key is *not*

- It is **not** a decryption key for any stored data. It never touches
  ciphertext.
- It does **not** bypass `SargonPassport::verify_seal()` for anyone
  else's passport — it only ever mints a *new* passport for the
  Architect's own use.
- It is **not** meant to be used often. If you find yourself
  reconstructing it weekly, something about day-to-day access design
  needs fixing — this is the emergency door, not the front door.
