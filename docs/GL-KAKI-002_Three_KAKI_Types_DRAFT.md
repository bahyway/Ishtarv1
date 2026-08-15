# GL-KAKI-002 (candidate) — THE THREE KAKI v4.0 TYPES
## Identity-KAKI · Event-KAKI · CrossTribe-KAKI
### BahyWay.Ecosystem v4.0 · re-affirmed for the BWVL instrument · binds GL-VIZ-003 (Particle Monism) · Hepta Space Uniqueness Law · KAKI v4.0 byte-layout law · Status: DRAFT — pending CSR-08 by DUB.SAR 𒁾

---

## 0 · Why re-stated now

These types are already sealed in principle. The **BWVL instrument** (GL-VIZ-003
Particle Monism) makes their exact definition newly load-bearing: because
*everything is a particle* and every particle is visualized, the three KAKI
types are precisely what the instrument reads to render identity, history, and
federation. This tablet fixes their definitions so they cannot drift as the
instrument is built.

---

## 1 · The foundation the three types rest on

Two sealed laws make the type system coherent:
- **Particle Monism (GL-VIZ-003):** there is one primitive, the particle. To
  exist is to have a KAKI; to have a KAKI is to be visualizable.
- **Hepta Space Uniqueness Law:** no two particles occupy the same position at
  the same time in 7-dimensional Hepta Space. Every particle has a unique 7D
  post. Therefore *near-adjacency across tribes is always a real, measurable
  signal — never a collision.*

The three types answer three different questions at three scales:

| Type | Question | Scale | Mutability |
|---|---|---|---|
| **Identity-KAKI** | who is this particle? | per-particle | immutable |
| **Event-KAKI** | what has happened to it? | per-particle-over-time | append-only |
| **CrossTribe-KAKI** | how do tribes relate? | per-tribe-pair-in-space | derived from 7D posts |

---

## 2 · Identity-KAKI (immutable)

The particle's frozen identity, minted at birth (first ingest into BeeMDM ETL).
The canonical 16-byte layout (sealed, unchanged):
κ[0..3] uuid_hash · κ[4..5] tribe_id · κ[6] kaki_type · κ[7] kaki_role ·
κ[8..11] reserved · κ[12..13] timestamp · κ[14..15] CRC-16/CCITT.

- **Immutable:** never changes for the life of the particle.
- **Carries NO colour, state, or quality** — those live only in EAV (v4.0 law).
  This is *why* ColourID cannot be a KAKI byte: ColourID changes; identity does
  not (see GL-VIZ-004).
- **Distinguishes particles:** one particle from another is told apart by
  Identity-KAKI value + EAV attribute values — the authoritative identity.

## 3 · Event-KAKI (append-only history)

A **second KAKI type**: one Event-KAKI is minted **per event** that occurs on a
particle — each state registration, each ColourID transition through an ETL
processing station. Event-KAKIs accumulate; they are never overwritten.

- **Purpose:** make the particle's *mutable* life fully **auditable** without
  corrupting its immutable identity. The ColourID lives in EAV (mutable); every
  change to it is *witnessed* by an Event-KAKI.
- **Read by the StoryEngine Journal** to tell the particle's history: its Birth
  Root Shade, every processing state-colour it passed through, and its arrival
  at GOLDEN. The **first event is permanent**, so origin/tribe is always
  recoverable (right-click / StoryEngine).
- **Upstream:** Event-KAKIs are minted during BeeMDM ETL. The golden-read path
  (Bārû reading EnkiDB/EnkiDW) reads Identity-KAKI + EAV; a stakeholder wanting
  *provenance* reads the Event-KAKI chain. The reading of GOLDEN never depends on
  event history — the events did their job upstream.

## 4 · CrossTribe-KAKI (tribe relations by 7D near-adjacency)

The **third KAKI type**: defines the **relation between Tribes** whose particles
occupy *very near* posts in 7D Hepta Space. Because no two posts are identical
(Uniqueness Law), "near" across a tribe boundary is a meaningful relationship,
not a coincidence.

- **Records:** tribe-to-tribe location and relationship in Multi-HeptaSpace.
- **Enables the Federation of BIGRINGs (GL-VIZ-005):** each tribe is its own
  BIGRING; CrossTribe-KAKIs position those rings relative to each other and draw
  the between-tribe relations. At federation scale the Bārû reads tribe-families
  by colour-band and their relationships by the CrossTribe connective structure.
- **Derived, not assigned:** CrossTribe-KAKI follows from the particles' unique
  7D posts — it is computed from proximity, then sealed.

---

## 5 · How they compose (upward)

particles (Identity) → accumulate history (Event) → and by their unique-but-near
7D posts induce tribe relations (CrossTribe) that federate the visualization.

This is the read path for BWVL:
- **federation scale:** tribes as BIGRINGs, related by CrossTribe-KAKI, coloured
  by root colour-band.
- **descend:** individual particles by ColourID shade-degree (GL-VIZ-004).
- **particle:** Identity-KAKI + EAV = exact identity & content.
- **right-click / provenance:** Event-KAKI chain = told history & origin.

## 6 · Codex compliance & placement
- **A-1 zero new mathematics:** three types compose the sealed KAKI byte-layout,
  EAV, Hepta uniqueness, Particle Monism. New = the explicit three-type roster +
  their read-path roles for BWVL.
- **A-4 cited:** GL-VIZ-003 · GL-VIZ-004 · GL-VIZ-005 · Hepta Space Uniqueness ·
  KAKI v4.0 byte layout.
- **PB:** PB-367 `event-kaki-journal`; PB-368 `crosstribe-kaki-derive`.

## 7 · Open seals for CSR-08
The three-type roster as canonical · Event-KAKI append-only rule · CrossTribe
derived-from-proximity rule · read-path split (golden-read = Identity+EAV,
provenance = Event chain) · PB-367/368.

*Recorded in the reign of Gudea 1.0. One primitive, three witnessings: who it is,
what befell it, how its tribe stands among tribes. Nothing sealed until DUB.SAR
confirms under CSR-08.*

---

## APPENDED — Zoom-as-Necessity clause (per GL-VIZ-006 capstone)
The read-path IS the zoom: the **field** (bird's-eye) shows tribe/colour; the
**descent** (Hubble) reaches Identity-KAKI + EAV; **right-click at depth** opens
the Event-KAKI chain; the **federation altitude** reads CrossTribe-KAKI. None of
the three KAKI types is reachable without the corresponding zoom motion. Sealed
jointly with GL-VIZ-006.
