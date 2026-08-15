# BahyWay.Ecosystem — The Onion Layers Design

**Sealed:** 2026-07-19. Companion to a published SVG artifact
(link kept by the Architect) visualizing this same structure as concentric
rings. This document is the text-and-table reference; the artifact is the
poster.

## The Dilemma This Answers

> "I cannot build one and only one piece of BahyWay.Ecosystem like EnkiDDB.
> Because everything is related to another in the Ecosystem just like a
> Human Body... Patterns within Patterns, just like the Russian Baboshkas,
> or an Onion Layers." — the Architect, 2026-07-19

This is a real property of the system, not an exaggeration. NISABA's
exposure preview needs the Hepta Gates; the Hepta Gates need the sealed
sovereign-name pattern; that pattern needs IRKALLA; IRKALLA needs Nergal to
defend it; Nergal needs `kupru` for crypto and `hepta-sec-firewall` for
KAKI validity; both feed the same EnkiQDB jail EnkiDDB's own quarantine
flow uses. Pull one thread, four others arrive with it.

The resolution isn't that the circularity is imaginary — it's that Rust's
compiler will not allow it to exist in the *code*, only in the *domain
model*, and this workspace has been resolving that gap the same way every
time it's come up.

## The Four Laws

1. **Inner layers never know outer layers exist.** `bahyway-algebra`
   (GeoEngine, Layer 0) has never heard of `nisaba` or `lamassu-engine` —
   they call into it, not the reverse. Cargo's dependency resolver forbids
   a true cycle from compiling at all.
2. **KAKI is the circulatory system, not a convention.** Every particle,
   document, artifact, alert and template across all 25 layers is minted
   from the same 16-byte `enkidb-kaki` identity and the same EAV/Journal
   shape (Layer 1). Organs don't need each other's internals — they need
   the same blood chemistry.
3. **When two real things want to depend on each other, a third thing is
   born above both.** `alert-engine` depends on `story-engine`. When
   StoryEngine findings needed to raise alerts, the fix wasn't to force a
   cycle — it was `story-sentinel`, a thin crate consulting both, owning
   neither's logic. Same shape for `lamassu-engine` (consults
   `bahyway-algebra::persistence`, never recomputes homology) and
   `nisaba::orchestrator` (consults `alert-engine`, `data-steward-station`,
   `story-sentinel`, owns none of their logic).
4. **A sealed *name* is not a built *crate*.** `AkkadiSafeEngine`,
   `AkkadiRulesEngine` and `AkkadiCipherEngine` were locked as the
   canonical v4.0 naming convention in PB-88 — real, deliberate, sealed
   intent. None exist as an implemented crate today. The ghost layer
   below exists so that distinction is never lost again.

## The 25 Sealed Layers

Transcribed directly from `workspace/bahyway_v4/Cargo.toml`'s own member
list and its own layer comments — nothing here is invented; this is what
the workspace already organizes itself into, just never drawn before.

| Layer | Name | Crates | Members |
|---|---|---|---|
| L0 | Foundation | 6 | bahyway-core, bahyway-crc, bahyway-algebra, bahyway-field, algebra-arsenal, bahyway-fabric |
| L1 | KAKI Identity | 2 | enkidb-kaki, enkidb-vector-id |
| L1.5 | Particles (7D EAV) | 5 | enkidb-particles, enkiddb, enkiddb-ingest, enkimdb, susa-engine |
| L2 | Storage Substrate | 15 | enkidb-block, enkidb-journal, enkidb-storage, enkidb-snapshot, enkidb-recovery, enkidb-quantdb, enkidb-persist, enkidb-datafile, enkidb-readnode, enkidw, enkisdb, enkiqdb, enkiodb, storage-sector, blackbox-station |
| L3 | Native Indexes | 2 | enkidb-indexes, enkidb-dictionary |
| L4 | EnkiDB Engine | 4 | enkidb-engine, enkidb-query, enkidb-raft, enkidb-ingest |
| L4.5 | Physics & Intelligence | 7 | vgca-engine, tribe-orbit-engine, ammas-engine, shedu-engine, riksu-engine, vault-engine, graph-engine |
| L5 | Operational Engines | 22 | kinetic-engine, sumer-engine, pollution-engine, story-engine, story-sentinel, fuzzy-engine, score-engine, hepta-score, alert-engine, snapshot-job, navi-engine, najaf-engine, dmw-engine, nusku-engine, azuga-engine, nusku-fuzzy, iris-engine, panam-engine, nusku-score, shulman-engine, wpd-engine, homt-engine |
| L6 | Cross-Tribe / IDU | 2 | idu-prober, idu-batching |
| L7 | Templates & Governance | 5 | template-engine, template-library, diagnosis-templates, diagnosis-engine, damadmbok-dictionary |
| L8 | HeptaSec | 7 | hepta-sec-firewall, hepta-sec-policy, hepta-sec-sentinel, hepta-sec-web, enkidb-replication, pii-vault, sla-engine |
| L8.5 | ENKI-TERRA | 12 | esarhaddon, ashnan, naramsin-archive, naramsin-format, enkidb-session-registry, enkidb-con-engine, naramsin-integrity, naramsin-audit, tiamat-engine4, urnammu-attestationd, kittu-engine, naramsin-bridge |
| L8.6 | Pattern Intelligence | 7 | steward-lens, agent-council, nisaba, lamassu-engine, enki-genesis, enki-pattern, enkidu-protocol |
| L8-P | Pipeline / Stations | 12 | media-type-detector, akk-loader, adad-gate, musaru-security, compare-tribe-schema, vgca-validation, data-structure-station, data-cleansing-station, data-steward-station, permanent-storage, client-dq-profile, grave-discovery-station |
| L9 | Languages | 5 | hepta, aaol, heptascript, akkadi, akkadi-ir |
| L9.1 | Sovereign Crypto / Value | 3 | kupru, akkvalue, istar |
| L9.2 | Ezida (Query Compiler) | 1 | ezida-ir |
| L9.3 | DFG + HDF Bridge | 2 | dfg-engine, hdf-bridge |
| L9.5 | EnkiduLLM | 12 | enkidullm-core, enkidullm-model, enkidullm-memory, adapa-recall, enkidullm-chat, ea-agent-core, ea-agent-algebra, ea-agent-oracle, ea-agent-chat, enkidullm-ingest, zikru-embed, enkidullm-audit |
| L10 | Runtime / OS | 3 | eridu-runtime, eridu-scheduler, eridu-supervisor |
| L11 | UI / IDE | 3 | dubsar-ide, dubsar-visualizer, bee-mdm-bus |
| L12 | Website | 1 | bahyway-web |
| DT | DubSar Theater | 3 | utnapishtim, edubba-seal, enuma |
| BIN | Binaries & Agents | 20 | bahyway-server, enkidb-query-server, enkiddb-write-server, enkiddb-read-server, enkimdb-write-server, enkimdb-read-server, ninsun-agent, ninsun-steward-bridge, namtar-kaki, ereskigal-kaki, nanshe-ingest, esarhaddon-ingest, ashnan-ingest, bahyway-cli, najaf-ingest, dubsar, enkidw-cli, akkadi-cli, bee-watchdog, bahyway-api |
| TEST | Integration Tests | 2 | tests/e2e, tests/chaos |

**163 real workspace members, verified**
(`grep -c '^\s*"crates/\|^\s*"bin/\|^\s*"tests/' Cargo.toml`).

## The Ghost Layer — Sealed, Not Built

Four names, real and deliberately chosen, with **no implemented crate**
anywhere in this repository today:

| Name | Sealed in | Status |
|---|---|---|
| `AkkadiSafeEngine` | PB-88 (naming lock) | Named only. Conceptually: vault-unlock attempts (PB-110 glossary). |
| `AkkadiRulesEngine` | PB-88 (naming lock) | Named only. Conceptually: ABAC/usbguard device-policy decisions. Referenced as a still-open `// TODO` even inside `urnammu-attestationd`'s own real code (PB-137). |
| `AkkadiCipherEngine` | PB-88 (naming lock) | Named only. Strong hypothesis, not yet acted on: this is `kupru`'s sovereign display name — PB-88's exact phrase "Externally-facing cryptography routes through AkkadiCipherEngine" is the same sentence already sitting in `crates/sumer-engine/src/key.rs`, the crate that actually does that job. |
| `Nergal` (as a crate) | Architect-confirmed, 2026-07-19 | Real, live UI usage (`dubsar-visualizer::panels::nergal`, the AV/firewall defender for www.bahyway.com) — same status IRKALLA had before PB-188 gave it `storage-sector::SOVEREIGN_NAME`. No coded identity of its own yet. |

One correction on the record: an earlier answer in this same session
stated flatly that `AkkadiSafeEngine`/`AkkadiRulesEngine` "don't appear
anywhere in the codebase — not even as references." That was wrong — they
are real, sealed naming decisions from PB-88 and PB-110, uploaded by the
Architect as evidence. What's accurate is narrower: real as *locked
intent*, absent as *implementation*. The distinction this table encodes
is exactly the one that got missed.

## What Building "One Piece" Actually Means Here

EnkiDDB (Tigris) could be built this session without Nergal, without
AkkadiRulesEngine, without the other six EnkiDB Types existing yet —
not because it has no dependencies, but because its only *hard*
dependencies are Layers 0–1.5: `enkidb-kaki`, `enkidb-particles`,
`enkidb-journal`, `bahyway-core`. Everything built afterward
(`story-sentinel`, `lamassu-engine`, `nisaba::orchestrator`, the concept
registry and exposure preview) sits *on top of* EnkiDDB's stable
interface — never the reverse. The dependency web is real and it is
genuinely mesh-shaped at the concept level, but it has a direction at the
build level, the same way blood flows one way through a valve even though
every organ needs it.
