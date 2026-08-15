# ADR-020 — The Golden/DW → EnkiMDB/EnkiDDB Bridge Is a Link, Never a Copy

> **DubSar Help** | `Decisions > ADR-020` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-20"
  concept_type:   "0x02"
  epoch:          "2026-08-01"
  concept_depth:  0
  riksu_count:    2
  snapshot_epoch: "2026-08-01"

concept:          "EnkiDB(Golden)/EnkiDW to EnkiMDB/EnkiDDB bridge -- read-time TribeId link, never a data copy"
summary:          "The Architect's own corrected architecture diagram shows EnkiDB(Golden) and EnkiDW each pointing into EnkiMDB and EnkiDDB. The Architect's own clarification: this is NOT particles being sent/copied/re-minted into those databases -- it is a bridge that LINKS a Golden/Warehouse particle to its related EnkiMDB metadata and EnkiDDB documentation, resolved by the TribeId every particle in every one of the 7 Types already carries at birth. crates/enki-meta-doc-bridge implements this as a pure, read-only journal scan: given a tribe, return every EnkiMDB/EnkiDDB particle sharing it. Zero new data written anywhere."
sovereign_laws:   ["§BRIDGE-IS-A-LINK -- Golden/DW particles are never copied, re-minted, or duplicated into EnkiMDB/EnkiDDB; the bridge only resolves existing shared-tribe particles", "§TRIBE-IS-THE-KEY -- the join key is the TribeId every particle already carries (KAKI bytes 4..5), never a new stored reference field"]

riksu_bindings:
  - target: "adr_017_three_layer_pdm_paradigm.md"
    concept: "shares the same Triple-O 'particles are the shared currency' law this bridge relies on for a translation-free handoff"
    type: "GROUNDS"

orbit_tags:       ["EnkiDB", "EnkiMDB", "EnkiDDB", "EnkiDW", "TribeId", "bridge"]
rag_keywords:     ["Golden Records", "EnkiMDB bridge", "EnkiDDB bridge", "link not copy", "TribeId", "enki-meta-doc-bridge"]
-->

**Status:** Decision accepted 2026-08-01 — `crates/enki-meta-doc-bridge` built and tested this session (4/4 tests passing); the minting-convention gap below is NOT yet closed (see Consequences).
**Date:** 2026-08-01
**Author:** Bahaa Fadam

---

## Context

The Architect's own hand-drawn correction to the 7-Types EnkiDB flow diagram
added arrows from `EnkiDB(Golden)` and `EnkiDW` down into `EnkiMDB` and
`EnkiDDB`. My first reading of that diagram assumed those arrows meant
particles (or generated metadata/document records derived from them) get
written into EnkiMDB/EnkiDDB. The Architect corrected this directly:

> "What I meant is that EnkiDB can through a bridge to EnkiMDB & EnkiDDB
> show its particles related to its meta data in EnkiMDB and to its
> MetaData Documents in EnkiDDB. SO there is NO Sending data; there is
> ONLY Linking Golden Data to its related metadata and documentations.
> This is also true for EnkiDW with EnkiMDB & EnkiDDB."

## Decision

The arrows are a **read-time bridge**, not a write path. Nothing is ever
written to EnkiMDB or EnkiDDB by a Golden Record's or EnkiDW record's own
existence. The bridge resolves what already exists.

### The mechanism: TribeId, the join key every particle already has for free

Every particle in every one of the 7 Types carries a `TribeId` (KAKI bytes
4..5, immutable, set at birth — `IdentityKaki::tribe_id()`). This is exactly
the "particles are the shared currency" law ADR-018 already established for
`bahyway-algebra`/`LamassuEngine`/`NinurtaEngine` — no translation layer is
needed between databases because they were never speaking different
languages.

`crates/enki-meta-doc-bridge` implements this as two named, pure functions
over each database's own real `Journal`:

```rust
pub fn linked_metadata(tribe: TribeId, enkimdb_journal: &Journal) -> Vec<IdentityKaki>
pub fn linked_documentation(tribe: TribeId, enkiddb_journal: &Journal) -> Vec<IdentityKaki>
```

Given a Golden or EnkiDW particle's own tribe, both scan the target
database's journal and return every distinct particle sharing that tribe —
addressable, never copied. An empty result is a real, honest answer ("no
metadata/documentation shares this tribe yet"), not an error to hide.

## Consequences

- **Real today:** the bridge mechanism itself — 4/4 tests passing, including
  a same-tribe/multiple-events dedup case and a genuinely-empty-result case.
- **The one honest, NOT-yet-closed gap:** for this bridge to find real
  matches on CLIENT tribes, whatever EnkiMDB/EnkiDDB content is specific to
  that client needs to actually be minted under the client's own `TribeId`.
  Checked directly: today's real ingestion paths mint EnkiDDB documents
  under sovereign, ecosystem-level tribes (`enkiddb::DOCS_TRIBE_ID = 0x7160`,
  `enkiddb::PB_DOCS_TRIBE_ID = 0x7165`), not a client's business tribe. So a
  bridge query for a client's tribe today will honestly return empty until
  a real minting-convention decision is made and built: does client-specific
  EnkiMDB/EnkiDDB content get minted under that client's own business
  `TribeId` (enabling this bridge to work immediately, no further code
  needed), or does some other explicit reference mechanism get built instead?
  This ADR does not decide that — it only builds the read side honestly,
  connects it to this session's earlier open question (2026-07-31, "EnkiDDB/
  EnkiMDB not internal-only") about client-scoped clones of the EnkiDDB/
  EnkiMDB schema pattern, since the minting-convention answer likely
  resolves both at once.

BahyWay.Ecosystem v4.0 — written by one scribe, sealed with one seal. 𒁾
