# KAKI v4.0 — 16-Byte Sovereign Particle Identity

**Standalone component reference. Follows `docs/08_pipeline_alaktu/TRANSPARENCY_STANDARD.md`
— every claim below is tagged and cited. Re-derived directly from
`workspace/bahyway_v4/crates/enkidb-kaki/src/*.rs` on 2026-07-11, not
copied from a prior document.**

---

## 1. What it is

**🔒 LAW.** Every particle in BahyWay.Ecosystem — engine, crate, data
record, documentation entry, agent — has exactly one KAKI: a 16-byte,
immutable, self-describing sovereign identity. It is generated once at
creation and never changed, never reassigned, never refused by a
future release.

## 2. Byte layout

**✅ VERIFIED** — `enkidb-kaki/src/kaki.rs:1-15`, the crate's own header
comment, citing `KAKI_v4.0.pdf §1.2`.

```
κ[0..4]   minted_id / uuid_hash  — numeric ID minted at creation.
                                    uuid_hash() is the firewall key.
κ[4..6]   tribe_id               — u16, big-endian, PA-15 sovereignty
κ[6]      kaki_type              — see §3
κ[7]      kaki_role              — see §3
κ[8..12]  reserved               — zeroed; future structural markers
κ[12..14] timestamp              — birth timestamp, u16 big-endian
                                    (high byte doubles as azimuth PA-14)
κ[14..16] checksum               — CRC-16/CCITT over κ[0..14]
```

**⚠ COLLISION, open, not resolved by this document:** `ADR-003`
proposes reassigning κ[8..12] to a `seq_counter`. Three independent
sources (the canonical PDF, the 2026-06-27 Architecture Reference, and
the code itself) call it `reserved`. This document does not decide the
dispute — it records both positions so nobody builds against either
silently.

### Immutability rules (crate doc comment §2, verbatim)

- Rule I — byte values never modified
- Rule II — never reassigned to a different particle
- Rule III — only held via `Copy` or shared `&Kaki`; **no `&mut Kaki`
  exists anywhere in the crate** — confirmed by the type's own
  definition: `#[derive(Copy, Clone, PartialEq, Eq, Hash)]`,
  `#[repr(transparent)]`, no public constructor except
  `Kaki::from_bytes` (deserialization) and `Kaki::mint` (crate-private,
  reachable only through `KakiMinter`).
- Rule IV — no assessment data (state/quality/color) in these bytes;
  all quality/state assessment lives in EAV.

## 3. `kaki_type` (κ[6]) and `kaki_role` (κ[7])

**✅ VERIFIED** — `enkidb-kaki/src/types.rs`, full file read this pass.

### KakiType — 4 values, not 3

```rust
pub enum KakiType {
    Identity   = 0x01,  // birth certificate of a sovereign particle
    Event      = 0x02,  // immutable record of a state-transition
    CrossTribe = 0x03,  // persistent linkage across tribes
    Pattern    = 0x04,  // emergent GA-cluster structure, NISABA-derived
}
```

**⚠ COLLISION, open.** Every canonical byte-layout source (the KAKI
PDF, ADR-008's Forbidden Operations list, the 2026-06-27 Architecture
Reference) lists only 3 `kaki_type` values (0x01–0x03). `Pattern =
0x04` is real, live code — it has its own derivation module (§5 below)
— but has never been promoted into any canonical byte-layout table.
Needs an Architect ruling: promote it, or explicitly mark it an
intentional out-of-band extension.

### KakiRole — 3 values, all Akkadian

```rust
pub enum KakiRole {
    Kishib = 0x01,  // external file / blob / source artifact seal
    Zikru  = 0x02,  // record or entity in a tribe domain
    Parzu  = 0x03,  // logic, template, axiom, or rule
}
```

## 4. Identity categories (usage pattern, not a byte field)

**📄 DOCUMENTED** — this table describes conventions for combining
`kaki_type` + `tribe_id`, not a separate encoded field. Not
independently re-derived from a single source file this pass (it's a
convention observed across many crates' usage, not a type definition).

| Category | kaki_type | tribe_id | Examples |
|---|---|---|---|
| Internal File | Identity (0x01) | 0xFF00+ | Engines, crates, scripts, playbooks |
| External File | Identity (0x01) | 0x0001+ | CSV, Excel, PDF, ZIP batches |
| Record | Identity (0x01) | 0x0001+ | Grave record, sensor reading |
| Event | Event (0x02) | any | Gate transition, status update |
| CrossTribe | CrossTribe (0x03) | any | Requires Gilgamesh Passport (CSR-05) |
| Pattern | Pattern (0x04) | any | GA-cluster centroid, NISABA-derived |

## 5. Minting — the only way a KAKI comes into existence

**✅ VERIFIED** — `enkidb-kaki/src/mint.rs`, `pub struct KakiMinter`.
Public methods: `identity(role)`, `event(role)`, `crosstribe(role)`
(auto-generated `uuid_hash`), and `mint_identity(uuid_hash, role)` /
`mint_event(uuid_hash, role)` (caller-supplied, deterministic
`uuid_hash`, used when reproducibility matters — e.g. seeding a
fixture from a stable input). There is no path to construct a `Kaki`
with arbitrary bytes outside `KakiMinter` and `Kaki::from_bytes`.

**Pattern-KAKI derivation is a separate, deterministic path** — `enkidb-kaki/src/pattern.rs`,
`fn derive_pattern_kaki(...)` and `fn pattern_kaki_confidence(...) -> Option<u16>`.
Same inputs (a GA-cluster centroid) always produce the same KAKI —
this is why `Pattern` KAKIs are never minted randomly through
`KakiMinter`, and why `KakiType::is_pattern()` exists as a guard
against misuse of the deterministic path.

## 6. Reserved tribe IDs

**📄 DOCUMENTED**, from the 2026-06-27 Architecture Reference, not
independently re-verified against every crate's usage this pass:

| tribe_id | Name | Contents |
|---|---|---|
| 0xFF00 | BahyWay.Internal | All sovereign engines, crates, apps, scripts, playbooks |
| 0xFFFF | BahyWay.Templates | .akk, .way, .hepta files |
| 0xFF01 | BahyWay.Security | PAZUZU test runs, CSR audit events |
| 0xFF02 | BahyWay.Releases | Crate release profile KAKIs |
| 0x0001 | NAJAF_CEMETERY | 80,272 grave records — live production tribe |
| 0x0002+ | Client tribes | Assigned at onboarding |

**Separately, this session (2026-07-11) established a distinct,
non-colliding range for domain-engine tribe scoping**, ✅ VERIFIED
against real code:

| tribe_id range | Domain | Crate |
|---|---|---|
| 0x1001–0x10FF, 0x1200 | EnkiduLLM books/conversations | `enkidullm-*` |
| 0x2000–0x20FF | ESARHADDON earthquake hazard | `ninsun-steward-bridge::hazard` |
| 0x3000–0x3006 | WPDEngine Baghdad sectors | `wpd-engine::sector::BaghdadSector::tribe_id()` |

Both ranges are u16 and non-overlapping (0xFF00+ system range vs.
0x1000–0x3FFF domain-engine range) — no collision found, but this has
never been stated as a single unified table anywhere until now.

## 7. CrossTribe-KAKI v4.0.1 compliance

**✅ VERIFIED.** The v4.0.1 canonical spec's substantive change from
earlier drafts — removal of deprecated RED/GREEN/BLUE birth-state
bytes, all quality assessment moved to EAV — is fully compliant in
code:
- Zero RGB/red_score/green_score/blue_score bytes found anywhere in
  `enkidb-kaki` or `idu-prober` (grep-verified).
- `Kaki::mint()` writes κ[8..12] zeroed and never touches them
  elsewhere.
- `idu_prober::crosstribe::compose_n_anchors()` matches the IDU
  Probing Rule's effective-state table exactly (all-Golden → Gold,
  any-Dead → Gray, mixed → Orange), 5 tests including the N-anchor
  hyperedge case.

## 8. Test coverage

**✅ VERIFIED**, run 2026-07-11: `cargo test -p enkidb-kaki` → **23
passed, 0 failed**.

## 9. Open items (do not build against these silently)

1. κ[8..12] — `reserved` (3 sources) vs. ADR-003's `seq_counter` (1 source).
2. `KakiType::Pattern = 0x04` — real in code, absent from every canonical byte-layout table.
3. `tribe_id` byte width inconsistency — confirmed u16 in KAKI bytes
   (κ[4..6]); some session/registry contexts elsewhere in the
   ecosystem use u32 for what's described as the same concept (flagged
   in `docs/00_codex/BAHYWAY_ECOSYSTEM_V4_GLOSSARY.md` §T, "Tribe" — not
   resolved here either).
