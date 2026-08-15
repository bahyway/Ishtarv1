# EnkiDB-KAKI — Developer Manual

> **DubSar Help** | `Manuals > EnkiDB-KAKI` | Crate Reference

## Overview

`enkidb-kaki` defines the **16-byte sovereign KAKI identity**.  A KAKI is born
once and never altered.  All change lives in the Orbit layer — the bytes
themselves are sealed by `CRC-16/CCITT` and are immutable after minting.

The KAKI is the mandatory identity carrier for every particle, event, and
cross-tribe link in the BahyWay v4.0 sovereignty layer.  It flows into
`HeptaSecFirewall`, `HeptaSecSentinel`, and `KakiPacket` as the primary
trust-table key.

---

## Byte Layout

```text
κ[0..4]   minted_id   — numeric ID stored at mint time; NOT the firewall key
κ[4..6]   tribe_id    — PA-15 sovereignty (2 bytes, big-endian)
κ[6]      kaki_type   — 0x01 Identity / 0x02 Event / 0x03 CrossTribe
κ[7]      kaki_role   — 0x01 KISHIB / 0x02 ZIKRU / 0x03 PARZU
κ[8..12]  reserved    — zeroed; future structural markers
κ[12..14] timestamp   — birth timestamp (u16 big-endian; κ[12] = azimuth PA-14)
κ[14..16] checksum    — CRC-16/CCITT over κ[0..14]; κ[14] = altitude PA-14
```

**Immutability rules:**
- Rule I   — byte values are never modified
- Rule II  — a KAKI is never reassigned to a different particle
- Rule III — only held via `Copy` or shared `&Kaki`; `&mut Kaki` does not exist
- Rule IV  — no assessment data (state, quality, color) lives in these bytes

---

## KakiType — κ[6]

| Byte  | Variant     | Meaning                                          |
|-------|-------------|--------------------------------------------------|
| `0x01`| `Identity`  | Birth certificate of a sovereign particle        |
| `0x02`| `Event`     | Immutable record of a state-transition event     |
| `0x03`| `CrossTribe`| Persistent linkage across tribe boundaries       |

---

## KakiRole — κ[7]

| Byte  | Variant | Akkadian | Meaning                                       |
|-------|---------|----------|-----------------------------------------------|
| `0x01`| `Kishib`| كيشب     | External file / blob / source artifact seal   |
| `0x02`| `Zikru` | ذِكر     | Record or entity in a tribe domain            |
| `0x03`| `Parzu` | پارزو    | Logic, template, axiom, or rule               |

---

## Kaki — Core Accessors

```rust
let k: Kaki = minter.identity(KakiRole::Zikru);

// ── Structural bytes ───────────────────────────────────────────────────
k.bytes()           // &[u8; 16]      — raw 16 bytes
k.minted_id()       // u32            — numeric ID stored at mint (bytes[0..4])
k.tribe_id()        // TribeId        — κ[4..6]
k.kaki_type()       // KakiType       — κ[6]
k.kaki_role()       // KakiRole       — κ[7]
k.reserved()        // [u8; 4]        — κ[8..12], always [0,0,0,0]
k.timestamp()       // u16            — κ[12..14], birth timestamp
k.checksum()        // u16            — κ[14..16], CRC-16/CCITT
k.verify_checksum() // bool           — re-derives and compares CRC

// ── Firewall identity key ──────────────────────────────────────────────
k.uuid_hash()       // u32            — FNV-1a over ALL 16 bytes (the firewall key)

// ── PA-14 position fixing ──────────────────────────────────────────────
k.azimuth()         // u8             — κ[12], high byte of timestamp
k.altitude()        // u8             — κ[14], high byte of checksum
```

---

## uuid_hash() vs minted_id() — Security Contract

These are two different things and must not be confused:

| Method        | Returns                       | Use for                         |
|---------------|-------------------------------|---------------------------------|
| `minted_id()` | `u32::from_be_bytes(κ[0..4])` | Byte-layout inspection, sharding|
| `uuid_hash()` | `FNV-1a(κ[0..16])`            | Firewall trust-table key        |

**Why this matters:** two KAKIs can have the same `minted_id` (same numeric
input to `mint()`) but different `tribe_id`, `kaki_type`, `kaki_role`, or
`timestamp`.  Using `minted_id()` as the firewall key would allow a KAKI in
tribe 0x0002 to impersonate a KAKI in tribe 0x0001 at the `HeptaSecSentinel`.

`uuid_hash()` mixes all 16 bytes so that no two structurally distinct KAKIs
can produce the same firewall hash.

```rust
let k1 = Kaki::mint(0xDEAD_BEEF, TribeId(0x0001), ...);
let k2 = Kaki::mint(0xDEAD_BEEF, TribeId(0x0002), ...);  // different tribe

k1.minted_id() == k2.minted_id()  // true  — same numeric ID
k1.uuid_hash() != k2.uuid_hash()  // true  — different firewall hash ✓
```

---

## Kaki::from_bytes — Deserialisation

```rust
// Reconstruct from storage / network:
let k = Kaki::from_bytes(raw_16_bytes)?;
// Validates: KakiType discriminant, KakiRole discriminant, CRC-16/CCITT
```

Returns `BahywayError::ChecksumMismatch` if the CRC fails, or
`InvalidKakiType`/`InvalidKakiRole` if the discriminant bytes are out of range.

---

## KakiMinter — Creating New KAKIs

`KakiMinter` is the sole gated constructor for new KAKIs.  One minter
instance per tribe.

```rust
use enkidb_kaki::{KakiMinter, KakiRole};
use bahyway_core::TribeId;

let minter = KakiMinter::new(TribeId::from_u16(0x0001));

// Auto-generated minted_id (nanoseconds + counter + tribe mix)
let identity = minter.identity(KakiRole::Zikru);
let event    = minter.event(KakiRole::Kishib);
let cross    = minter.crosstribe(KakiRole::Parzu);

// Deterministic minted_id (for tests, migrations, or canonical seeds)
let canonical = minter.mint_identity(0xDEAD_BEEF, KakiRole::Zikru);
```

**Uniqueness guarantee:** each `identity()` / `event()` / `crosstribe()` call
mixes `SystemTime::now().subsec_nanos()` with a per-minter counter and the
tribe ID.  Collision probability is negligible over any realistic deployment
lifetime.  No `rand` or `uuid` crate is used.

---

## Newtype Wrappers

Type-level enforcement of KAKI physical kind — prevents passing an
`IdentityKaki` where an `EventKaki` is expected at compile time.

```rust
use enkidb_kaki::{IdentityKaki, EventKaki, CrossTribeKaki};

let k      = minter.identity(KakiRole::Zikru);
let ik     = IdentityKaki::try_from_kaki(k)?;  // Ok
let bad    = EventKaki::try_from_kaki(k);       // Err(InvalidKakiType(0x01))

// All Kaki accessors available via Deref
ik.uuid_hash()
ik.tribe_id()
```

| Wrapper         | Accepted `kaki_type` |
|-----------------|----------------------|
| `IdentityKaki`  | `0x01` (Identity)    |
| `EventKaki`     | `0x02` (Event)       |
| `CrossTribeKaki`| `0x03` (CrossTribe)  |

---

## Display Format

```rust
format!("{}", k)
// → "deadbeef-0001-0102-00000000010047b2"
//    ^^^^^^^^ minted_id  ^^^^ tribe/type/role  ^^^^^^^^^^^^^^^^ reserved+ts+cs
```

The display is 32 hex chars + 3 dashes in `8-4-4-16` grouping matching the
16-byte layout.

---

## Using a KAKI as a Firewall Key

```rust
// Register with HeptaSecSentinel after BeeMDM B11 scoring:
sentinel.register_kaki_b11(k.uuid_hash(), b11_score, now_epoch);

// Build a KakiPacket:
let packet = KakiPacket {
    src_kaki_hash: Some(k.uuid_hash()),
    dst_kaki_hash: Some(destination.uuid_hash()),
    ...
};
```

Always use `uuid_hash()` — never `minted_id()` — when constructing `KakiPacket`
or calling `sentinel.register_kaki_b11()`.

---

## Dependencies

- `bahyway-core` — `TribeId`, `BahywayError`, `Result`
- `bahyway-crc`  — `crc16()`, `verify()` for CRC-16/CCITT

No external crates.

---

## See Also

- `crates/hepta-sec-firewall/MANUAL.md` — TrustState ladder, KakiFirewall
- `crates/hepta-sec-sentinel/MANUAL.md` — HeptaSecSentinel integrated pipeline
- `crates/hepta-sec-web/MANUAL.md`      — HTTP X-Kaki-Hash header extraction
- `crates/enkidb-replication/MANUAL.md` — ENKWAL replication events (write_pod_kaki_hash)
