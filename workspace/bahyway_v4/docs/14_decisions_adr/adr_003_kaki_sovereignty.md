# ADR-003 — KAKI Sovereignty: Byte Layout, Immutability, and Reserved Bytes

> **DubSar Help** | `ADR > 003` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-01-01"
  concept_depth:  240
  riksu_count:    3
  snapshot_epoch: "2026-06-06"

concept:          "KAKI Sovereignty"
summary:          "KAKI is a 16-byte sovereign identity seal minted once at birth and immutable for the particle's lifetime."
sovereign_laws:   ["§2.4 — no assessments in KAKI nucleus", "§8.3 — CrossTribe state computed on PROBE only"]

riksu_bindings:
  - target: "identity_kaki.md"
    concept: "KAKI byte layout"
    type: "PEER"
  - target: "master_glossary.md"
    concept: "KAKI declaration"
    type: "PEER"
  - target: "adr_008_ooo_foundation_kaki_roles_forbidden_operations.md"
    concept: "KAKI roles"
    type: "PEER"

orbit_tags:       ["KAKI Sovereignty"]
rag_keywords:     ["MINT", "KIŠIB", "KAKI", "sovereign identity", "seq_counter", "birthday paradox", "FNV-1a"]
-->

## Status: Accepted — Extended 2026-06-05

---

## Context

The KAKI (kaqqadu 𒋼𒁀) is the 16-byte sovereign primary key of every particle
in BahyWay.Ecosystem v4.0. Three constraints are fundamental to its design:

1. The KAKI nucleus must encode structural identity only — never assessments (§2.4).
2. CrossTribe effective state must be computed at query time, never stored (§8.3).
3. The identity space must remain collision-resistant at any realistic deployment
   scale — including high-frequency tribes (IoT sensors, financial tickers)
   producing thousands of particles per sovereign epoch.

Constraint 3 was identified during the ADR-008/ADR-009 algebra audit on
2026-06-05 and resolved by this ADR extension.

---

## Decision

### Decision 1 — The Canonical KAKI Byte Layout

```
κ[0..3]   uuid_hash    32 bits   FNV-1a hash of particle content at birth (D1)
κ[4..5]   tribe_id     16 bits   Tribe registration identifier (D2)
κ[6]      kaki_type     8 bits   Physical type: 0x01 Identity, 0x02 Event, 0x03 CrossTribe
κ[7]      kaki_role     8 bits   Logical role: 0x01 KISHIB, 0x02 ZIKRU, 0x03 PARZU
κ[8..11]  seq_counter  32 bits   Per-tribe-per-epoch sequence counter (Decision 3 below)
κ[12..13] timestamp    16 bits   Sovereign epoch at creation (D6)
κ[14..15] checksum     16 bits   CRC-16/CCITT over κ[0..13] — deterministic integrity seal
```

The checksum at `κ[14..15]` is always derived from the preceding 14 bytes.
It adds no entropy but provides tamper detection: any modification to any
byte in `κ[0..13]` produces a checksum mismatch detectable by any reader.

### Decision 2 — Structural-Facts-Only Rule

**The KAKI bytes encode structural identity. They never encode assessments.**

What is permanently forbidden in KAKI bytes (Structural-Facts-Only Rule §2.4):

| Forbidden content | Correct location |
|---|---|
| Quality scores (B11, VGCA-Σ) | EAV attribute, computed by score-engine |
| Quality lane (GEM/TRIBE/ACTIVE/FUZZY/DEAD) | Derived from B11 at query time |
| ColorID 7D quality vector | EAV attribute, maintained by vgca-engine |
| CrossTribe effective state (Gold/Orange/Gray) | Computed at query time via IDU Probing Rule §8.3 |

This rule is the structural guarantee that KAKI bytes are invariant for the
lifetime of the particle. A particle whose B11 degrades from 220 to 45 does
not change a single byte in its KAKI PK. The partition routing derived from
`κ[4..5]` and `κ[12..13]` remains stable regardless of quality evolution.

### Decision 3 — Reserved Bytes `κ[8..11]` Become `seq_counter` (Accepted 2026-06-05)

**The 4 bytes previously designated as `reserved = 0x00000000` are formally
assigned as a per-tribe-per-epoch sequence counter.**

#### The Birthday Paradox Problem

Within a single sovereign epoch (same `κ[12..13]`), within a single tribe
(same `κ[4..5]`), with the same type and role (`κ[6]`, `κ[7]`), the only
differentiating field was `uuid_hash` — 32 bits = 2^32 ≈ 4.3 billion values.

By the birthday paradox, 50% collision probability is reached at:

```
n ≈ 1.177 × √(2^32) ≈ 77,163 particles per tribe per epoch
```

For civil registries (< 100 particles/second) this is negligible. For
high-frequency tribes (IoT sensors, financial tickers producing 10,000–
100,000 particles per second), this is a real collision risk that would
corrupt the sovereign identity guarantee.

#### The Solution — `seq_counter`

`κ[8..11]` = a monotonically increasing 32-bit counter, scoped to:
- One tribe (`κ[4..5]`)
- One kaki_type (`κ[6]`)
- One kaki_role (`κ[7]`)
- One sovereign epoch (`κ[12..13]`)

Reset to `0x00000001` at the start of each new epoch. Incremented atomically
for each KAKI minted within that scope. The `KakiMinter` in `enkidb-kaki`
is the sole authority for this counter — no external sequence generator,
consistent with ADR-001.

#### Why Option B Is More Sovereign Than Option A

Option A (extend uuid_hash from 4 to 8 bytes) would give 2^64 probabilistic
collision resistance. It is correct but relies on hash distribution.

Option B (`seq_counter`) gives **deterministic** uniqueness:

```
(tribe_id, kaki_type, kaki_role, timestamp, seq_counter) is unique by construction.
No probability. No birthday paradox. No hash quality dependence.
```

Additional properties gained at zero cost:

| Property | How seq_counter provides it |
|---|---|
| **Deterministic uniqueness** | Two KAKIs minted in the same scope are unique because their seq_counter differs — not because their hashes happened to differ |
| **Auditable ordering** | KAKIs minted in the same tribe-epoch have a provable creation order: seq_counter 1 was born before seq_counter 2 |
| **Gap detection** | An auditor who finds seq_counter jumping from 47 to 49 within the same tribe-epoch knows that KAKI #48 is missing — which in a No-DELETE system means it was never minted, or the sequence was tampered with |
| **No external generator** | The counter is sovereign — maintained by `KakiMinter` per tribe per epoch, reset at each epoch boundary; no Redis, no PostgreSQL sequence, no UUID v4 randomness (ADR-001 compliant) |

#### Counter Capacity

```
seq_counter: u32 = 0x00000001 .. 0xFFFFFFFF
Maximum KAKIs per tribe per type per role per epoch = 4,294,967,294
```

At 100,000 particles per second, this overflows in ~11.9 hours. For any
tribe producing more than 4.2 billion particles per sovereign epoch, the
epoch definition should be made finer (sub-second ticks). For all known
BahyWay deployment targets, this limit is unreachable in practice.

### Decision 4 — CrossTribe Effective State Is Never Stored (§8.3)

CrossTribe-Kaki effective state (Gold / Orange / Gray) is computed at query
time by the IDU Probing Rule. It is never written to any field of the
CrossTribe-Kaki's KAKI bytes or EAV attributes. See ADR-008 Decision 6
for the full IDU computation specification.

---

## W5H2

| W | Answer |
|---|---|
| **Who** | `enkidb-kaki` crate (`KakiMinter`), every tribe administrator, every sovereignty auditor |
| **What** | The canonical 16-byte KAKI layout; the Structural-Facts-Only Rule; the seq_counter assignment to `κ[8..11]`; the CrossTribe non-storage rule |
| **When** | Original: BahyWay.Ecosystem v4.0 inception. Decision 3 (seq_counter) added 2026-06-05 after birthday paradox analysis during the ADR-009 algebra audit |
| **Where** | `crates/enkidb-kaki` — `KakiMinter` holds the per-tribe-per-epoch counter; `crates/enkidb-engine` — enforces Structural-Facts-Only at write time |
| **Why** | A sovereign identity that can suffer probabilistic collisions at scale is not sovereign. The seq_counter makes uniqueness deterministic, adds auditable ordering, enables gap detection, and costs zero additional bytes — the space was already reserved |
| **How** | `KakiMinter::mint(tribe_id, kaki_type, kaki_role, content) → Kaki`: compute uuid_hash = FNV-1a(content), fetch current epoch tick, atomically increment seq_counter for (tribe_id, type, role, epoch), assemble κ[0..13], compute CRC-16 checksum for κ[14..15] |
| **How Much** | 16 bytes total · 32-bit seq_counter · 4,294,967,294 max KAKIs per tribe per epoch · 0 external dependencies · 1 sovereign minting authority (`KakiMinter`) |

---

## Consequences

### Positive
- **Deterministic uniqueness** — collision is structurally impossible, not merely improbable
- **Free creation ordering** — auditors and query engines can order KAKIs within a tribe-epoch by seq_counter without a secondary index
- **Gap detection** — a missing seq_counter value within a tribe-epoch is provable evidence of a minting anomaly; in a No-DELETE system this is a tamper signal
- **ADR-001 compliant** — no external sequence generator; the counter is sovereign

### Constraints Introduced
- `KakiMinter` must maintain one atomic counter per (tribe_id, kaki_type, kaki_role, epoch) tuple — a small in-memory map, reset each epoch
- The counter state must survive process restart within an epoch — it must be persisted to `enkidb-journal` with each minting so that a restart does not re-issue a seq_counter value already written to a committed KAKI
- Existing KAKIs minted before this decision (with `κ[8..11] = 0x00000000`) are valid legacy KAKIs; their seq_counter field reads as zero, which is distinguishable from any seq_counter-aware KAKI (which starts at 0x00000001)

---

## Relationship to Other ADRs

| ADR | Interaction |
|---|---|
| **ADR-001** (No External DB) | seq_counter is maintained by `KakiMinter` with no external sequence service — the counter is sovereign |
| **ADR-006** (No DELETE) | Gap detection in seq_counter is only meaningful in a No-DELETE system; in a system with DELETE, gaps would be expected. The two decisions reinforce each other |
| **ADR-008** (OOO) | Structural-Facts-Only Rule (§2.4) is the OOO-layer grounding for Decision 2; seq_counter is a structural fact (creation ordinal), not an assessment — it belongs in KAKI bytes |
| **ADR-009** (Algebra Additions) | The birthday paradox analysis that prompted Decision 3 was conducted during the ADR-009 algebra audit |

---

## Sovereign Law Statement

> **A particle's identity is its KAKI. A KAKI is 16 bytes. Those 16 bytes
> are structural facts about who the particle is, when it was born, and
> in what order among its tribe-epoch peers. They are not judgements about
> its quality, its health, or its relations. Those bytes are sealed at
> birth by the KakiMinter and can never be altered. The seq_counter makes
> this seal unambiguous: every particle born in a tribe-epoch has a unique
> ordinal, provable without probability, auditable without trust.**

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | ADR-003 Extended 2026-06-05*
