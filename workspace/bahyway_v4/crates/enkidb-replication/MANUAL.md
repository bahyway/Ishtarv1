# EnkiDB-Replication — Developer Manual

> **DubSar Help** | `Manuals > EnkiDB-Replication` | Crate Reference

## Overview

`enkidb-replication` implements the sovereign **7-layer ENKWAL replication
pipeline** between the Write Pod and Read Pod EnkiDB instances.

Replication is achieved through an **append-only log file** (`.enkwal` format)
on a shared Podman named volume.  No TCP connection exists between the pods —
the Broker runs with `--network none`.  The log is the sole channel, and every
frame on it is cryptographically sealed and chain-hashed.

The three components form a one-way pipeline:

```text
┌─────────────────┐    .enkwal log (:ro)   ┌─────────────────┐    delta (:rw)    ┌─────────────────┐
│  Write Pod      │ ─────────────────────▶ │  Broker         │ ────────────────▶ │  Read Pod       │
│ ReplicationEmitter│                      │ ReplicationBroker│                   │ReplicationConsumer│
│ (Ed25519 sign)  │                        │ (7-layer verify) │                   │ (apply delta)   │
└─────────────────┘                        └─────────────────┘                   └─────────────────┘
```

---

## Wire Format — KakiSealedEvent

The `.enkwal` frame format (all integers little-endian):

```text
[4]  FRAME_MAGIC   0xEE 0x4B 0x57 0x04
[4]  frame_len     u32 LE  (bytes that follow, including seal+digest)
[8]  seq           u64 LE  (monotonically increasing from 1)
[4]  epoch         u32 LE  (seconds since sovereign epoch)
[4]  write_pod_kaki_hash  u32 LE
[32] prev_digest   [u8;32] (SHA3-256 of previous frame; genesis=all-zero)
[4]  payload_fnv   u32 LE  (FNV-1a of delta bytes — fast pre-check)
[1]  event_kind    u8
[4]  delta_len     u32 LE
[N]  delta         [u8; delta_len]
[64] seal          [u8;64] Ed25519 over canonical bytes (KANĀKU)
[32] digest        [u8;32] SHA3-256 of all bytes before this field (ŠIPIR ŠARRI)
```

### Constants

```rust
pub const FRAME_MAGIC:    [u8; 4] = [0xEE, 0x4B, 0x57, 0x04];
pub const REPL_DOMAIN:    &[u8]   = b"BahyWay.Replication.v4.ENKWAL";
pub const REPL_MAX_SECS:  u32     = 300;     // 5-minute epoch freshness window
pub const GENESIS_DIGEST: [u8;32] = [0u8;32]; // anchor for the first event
```

---

## ReplEventKind

| Value | Variant          | Meaning                                     |
|-------|------------------|---------------------------------------------|
| `0`   | `ParticleInsert` | New particle written to the database        |
| `1`   | `ParticleUpdate` | Existing particle fields updated            |
| `2`   | `ParticleDelete` | Particle logically deleted                  |
| `3`   | `Checkpoint`     | Periodic snapshot marker; delta = token     |

---

## ReplicationEmitter — Write Pod Side

`ReplicationEmitter` is owned by the Write Pod.  It holds the signing keypair
and appends sealed frames to the log file.

```rust
use enkidb_replication::ReplicationEmitter;
use enkidb_replication::event::ReplEventKind;
use kupru::SealKeyPair;

// New log (seq=1, genesis digest anchor)
let keypair = SealKeyPair::generate()?;
let mut emitter = ReplicationEmitter::new_log(
    write_pod_kaki_hash,   // u32 — the Write Pod's KAKI uuid_hash()
    keypair,               // SealKeyPair — Ed25519; ZeroizeOnDrop
    "/mnt/repl/write.enkwal",
);

// Emit one event
let ev: KakiSealedEvent = emitter.emit(
    ReplEventKind::ParticleInsert,
    delta_bytes,           // Vec<u8> — serialised particle delta
    now_epoch,             // u32
)?;

// State after emit
emitter.next_seq()      // u64 — next expected sequence number
emitter.last_digest()   // [u8;32] — SHA3-256 of the last written frame
```

### Resuming After Restart

```rust
// Resume from a non-zero starting point after restart:
let emitter = ReplicationEmitter::new(
    write_pod_kaki_hash,
    keypair,
    log_path,
    last_seq + 1,        // u64 — next sequence number
    last_digest,         // [u8;32] — last committed frame digest
);
```

---

## ReplicationBroker — 7-Layer Verification Gate

`ReplicationBroker` runs with `--network none` in Podman.  It reads from the
Write Pod's log (`:ro` volume) and appends verified frames to the Read Pod's
delta file (`:rw` volume).

### 7 Verification Layers (fail-fast, cheapest first)

| # | Check | Failure error |
|---|-------|---------------|
| 1 | Frame magic + structural integrity (`from_frame`) | `InvalidFrame` |
| 2 | Epoch freshness: `now_epoch − event.epoch ≤ REPL_MAX_SECS` | `EpochExpired` |
| 3 | Sequence monotonicity: `seq == last_seq + 1` | `SequenceGap` |
| 4 | Chained digest: `prev_digest == broker.last_digest` (constant-time) | `ChainBroken` |
| 5 | KAKI hash matches configured Write Pod | `KakiBlocked` |
| 5b| KANĀKU Ed25519 seal: `verifier.verify(canonical_bytes, seal)` | `SealInvalid` |
| 6 | ŠIPIR ŠARRI: SHA3-256 digest verified by `from_frame` | `DigestMismatch` |
| 7 | HeptaSecSentinel: `inspect_packet → Allow` | `KakiBlocked` |

All seven must fail simultaneously for an event to reach the Read Pod — there
is no single failure mode.

```rust
use enkidb_replication::broker::{BrokerConfig, ReplicationBroker};
use kupru::SealKeyPair;

let config = BrokerConfig {
    write_pod_verifying_key: keypair.verifying_key_bytes(), // [u8;32]
    write_pod_kaki_hash:     write_pod_hash,
    input_log_path:          PathBuf::from("/mnt/repl/write.enkwal"),
    output_delta_path:       PathBuf::from("/mnt/repl/delta.enkwal"),
    passport_validator:      None,  // or Some(Box::new(|epoch| { ... }))
};

let mut broker = ReplicationBroker::new(config);

// Call periodically (every 1–5 seconds):
let forwarded: usize = broker.sweep(now_epoch)?;

// Stats
broker.forwarded()          // u64 — total events successfully forwarded
broker.rejected()           // u64 — total events rejected (any check failed)
broker.last_seq()           // u64 — last verified sequence number
broker.log_offset()         // u64 — bytes consumed from the input log
```

---

## ReplicationConsumer — Read Pod Side

`ReplicationConsumer` reads the Broker-verified delta file and applies each
event to the Read EnkiDB instance.  It holds only the Broker's **public
verifying key** — no private key ever enters the Read Pod.

```rust
use enkidb_replication::consumer::ReplicationConsumer;

let consumer = ReplicationConsumer::new(
    broker_verifying_key,   // [u8;32] — Broker's Ed25519 public key
    broker_kaki_hash,       // u32     — Broker's KAKI uuid_hash()
    "/mnt/repl/delta.enkwal",
    Box::new(|kind_byte, delta| {
        // Apply the delta to the Read EnkiDB
        read_db.apply(ReplEventKind::from_u8(kind_byte)?, delta)
    }),
);

// Call periodically:
let applied: usize = consumer.consume(now_epoch)?;

let stats = consumer.stats();
stats.events_applied    // u64
stats.events_rejected   // u64
stats.last_seq          // u64
```

---

## Chained Digest — Log Tamper Evidence

Each frame's `digest` field = `SHA3-256(frame[0..frame.len()-32])`.

The next frame stores this as `prev_digest`.  If any frame is:
- **deleted** → the following frame's `prev_digest` no longer matches
- **reordered** → sequence numbers break monotonicity
- **injected** → the digest chain is broken at the injection point

The only valid log is a continuous, unbroken sequence starting from
`GENESIS_DIGEST = [0u8; 32]`.

---

## Canonical Bytes — What KANĀKU Signs

Ed25519 signs the following deterministic byte sequence (domain-separated):

```text
len(REPL_DOMAIN) || REPL_DOMAIN
seq              (u64 LE)
epoch            (u32 LE)
write_pod_kaki_hash (u32 LE)
prev_digest      ([u8; 32])
event_kind       (u8)
len(delta)       (u32 LE)
delta            ([u8; delta_len])
```

`REPL_DOMAIN = b"BahyWay.Replication.v4.ENKWAL"` is the domain separator that
scopes the signature to this protocol.  The `seal` and `digest` fields are
excluded from the signed data.

---

## Error Types

```rust
pub enum ReplicationError {
    SequenceGap     { expected: u64, got: u64 },
    ChainBroken,
    SealInvalid,
    DigestMismatch,
    EpochExpired    { event_epoch: u32, now_epoch: u32 },
    PassportExpired,
    PassportScopeInsufficient,
    KakiBlocked,
    InvalidFrame,
    Io(String),
}
```

---

## Podman Volume Layout

```
/mnt/repl/
├── write.enkwal   — Write Pod appends here (:rw for Write Pod, :ro for Broker)
└── delta.enkwal   — Broker appends here (:rw for Broker, :ro for Read Pod)
```

The Broker container is started with:
```
podman run --network none \
  -v /mnt/repl/write.enkwal:/mnt/repl/write.enkwal:ro \
  -v /mnt/repl/delta.enkwal:/mnt/repl/delta.enkwal:rw \
  bahyway/broker
```

---

## Dependencies

- `kupru`              — `SealKeyPair`, `SovereignVerifier`, Ed25519, SHA3-256
- `hepta-sec-firewall` — `KakiPacket`, `PacketProtocol`, `FirewallVerdict`
- `hepta-sec-sentinel` — `HeptaSecSentinel` (7th layer check)
- `subtle`             — constant-time `ConstantTimeEq` for digest comparison
- `sha3`               — SHA3-256 (ŠIPIR ŠARRI digest)

---

## See Also

- `crates/enkidb-kaki/MANUAL.md`        — write_pod_kaki_hash = kaki.uuid_hash()
- `crates/hepta-sec-sentinel/MANUAL.md` — HeptaSecSentinel (7th layer)
- `crates/kupru/` — SealKeyPair, SovereignVerifier, AkkadianSeal trait
- `policies/enkidb_replication_protocol.akk` — WAYv4.0 GUARD-clause specification
- `docs/09_security/bahyway_security_evaluation.md` — honest security assessment
