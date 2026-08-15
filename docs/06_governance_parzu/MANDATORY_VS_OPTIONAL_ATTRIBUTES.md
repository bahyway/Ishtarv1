# Internal Mandatory Attributes vs. External Optional Attributes
## The Three-Layer EAV Model for EnkiDDB Schema Design

**BahyWay.Ecosystem v4.0 — reference document, 2026-07-13**
**Companion to:** GL-EAV-001 (evaluated the same day — see the evaluation in
session history) · PB-184 (Storage) · PB-185 (Anu Index)
**Grounded in:** real, sealed, tested code — every claim below is a file:line
citation, not a restatement of a design document.

> This document exists to answer one question precisely: **when EnkiDDB
> particles are given attributes, which ones are mandatory on every particle
> regardless of Subject Area, and which ones are optional and specific to a
> Subject Area?** The Architect has stated this distinction many times. This
> document shows that most of the mandatory side is *already built*, names
> exactly what it is, and identifies the real gaps rather than inventing new
> ones.

---

## Verdict on the approach

**No objection.** The Mandatory/Optional split is sound, and the codebase
already implements more of the mandatory side than GL-EAV-001 gave it credit
for. GL-EAV-001's error was narrow: it claimed the KAKI role byte κ[7]
*already* carries DAMA's Business/Technical/Operational trichotomy. It
doesn't — κ[7] is Kishib/Zikru/Parṣu (artifact kind: file-seal / record-entity
/ logic-rule), a different axis entirely, and it's not reusable for this
purpose because it's already sealed for something else. But that error was
about *which byte*, not about *whether the mandatory/optional split is real*.
It is real — it lives in EAV attribute space, not in the KAKI header, and it
already has a name in the sealed source: **"The Hepta of Universal
Assessments."**

---

## Layer 0 — The KAKI Immutable Header (not an attribute at all)

Before any EAV attribute exists, every particle carries a 16-byte sealed
identity (`enkidb-kaki/src/kaki.rs`, KAKI_v4.0.pdf §1.2):

```
κ[0..4]   minted_id   — numeric ID minted at creation
κ[4..6]   tribe_id    — PA-15 sovereignty (2 bytes, big-endian)
κ[6]      kaki_type   — 0x01 Identity / 0x02 Event / 0x03 CrossTribe
κ[7]      kaki_role   — 0x01 KISHIB / 0x02 ZIKRU / 0x03 PARZU
κ[8..12]  reserved    — zeroed
κ[12..14] timestamp   — birth timestamp
κ[14..16] checksum    — CRC-16/CCITT over κ[0..14]
```

Immutability Rules I–III (`kaki.rs`): byte values never modified, never
reassigned to a different particle, no `&mut Kaki` exists anywhere in the
type system. This is **not** where "mandatory attributes" live — this is what
makes the particle a particle at all. It answers "what is this" and "whose is
this," never "how is this doing." That question belongs to Layer 1.

---

## Layer 1 — Internal Mandatory Universal EAV Attributes ("the Hepta of Universal Assessments")

Real, sealed, §4.2, verified in two places:

- `story-engine/src/projection.rs`: `// Mandatory universal EAV attributes —
  The Hepta of Universal Assessments`
- `story-engine/src/projected_state.rs`: `ProjectedState.attributes` doc
  comment — *"The seven mandatory EAV attributes (§4.2) + tribe-specific
  attrs."*

These seven `ATTR_*` hashes (CRC-16/CCITT of the canonical name) are present
on every particle in every tribe, in every Subject Area, with no
registration step — they are not what GL-EAV-001's registry governs:

| # | Attribute | Hash | What it is | Real implementation |
|---|---|---|---|---|
| 1 | `state` | `0x1A4B` | GOLDEN / FUZZY / DEAD | `story-engine::decode_state`/`encode_state` |
| 2 | `quality` | `0x2C5E` | The eav_quality feeding B11 | `fuzzy-engine::FuzzyDimensions` (D1–D9) → `score-engine::score()` |
| 3 | `color_rgb` | `0x3D7F` | ColorID RGB shade | `score-engine::compute_color()` |
| 4 | `freshness` | `0x4E89` | Time-decay factor | `score-engine::FreshnessDecay` |
| 5 | `Snapshot_Date` | `0x9A1D` | When last snapshotted | `enkidb-snapshot::SnapshotRecord` |
| 6 | `Snapshot_State` | `0x7E32` | Projected state at that snapshot | `enkidb-snapshot::SnapshotRecord` |
| 7 | `Snapshot_Frequency` | `0x4B0F` | Governing Snapshot_Job Vector-ID | `snapshot-job::SnapshotSchedule` |

Four more real attributes exist alongside the Hepta-7 for StoryWay temporal
queries (not part of the mandatory seven, but equally real, same file):
`color_id_snapshot` (`0x5F3A`), `color_drift` (`0x6C2B`), `event_cause`
(`0x7D1C`), `source_kaki` (`0x8E0D`).

### How this maps to what the Architect described

- **"keeps the Immutable Kaki immutable but keeps the state of the
  particle"** — exactly Layer 0 vs. Layer 1: the 16-byte header never
  changes; `state`/`quality`/`freshness` are separate EAV triples appended by
  new Event-Kakis, the header underneath is untouched.
- **"freshness"** — `attribute #4`, real, three decay profiles already
  implemented (`score-engine/src/freshness.rs`): `civil_registry()`
  (~365-day half-life), `operational()` (~30-day), `sensor_stream()`
  (~1-hour).
- **"ColorID(RGB) the shade from the Tribe Root Color, changed through the
  BeeMDM ETL Processing Stations Chain"** — this is real and precisely
  named. `diagnosis-engine/src/color_drift.rs`: *"Drift = Euclidean distance
  in RGB space between a particle's current `color_id_snapshot` and its
  tribe's root color. R = severity/alert, G = quality, B = freshness."*
  `score-engine::compute_color(domain_byte, quality, freshness, state)`
  produces the live triple: R is fixed per domain, G and B shift continuously
  as quality and freshness change — literally "the shade from the root
  color" the Architect described. As particles pass through BeeMDM's
  stations (`data-cleansing-station`, `data-steward-station`,
  `data-structure-station`, `blackbox-station`), each station's writes are
  new Event-Kakis updating `quality`/`state`, which recomputes `color_rgb`
  on next projection — the shade genuinely drifts as a *consequence* of the
  stations chain, not by direct color-editing.

---

## Layer 2 — External Optional Subject-Area EAV Attributes (GL-EAV-001's actual, correctly-scoped concern)

This is what GL-EAV-001 Part I–III is actually for, and it's sound: **seven
Subject Areas** (Assets, Architecture, Algorithms, Batches, Processes,
Environments, Knowledge), each with an **Attribute Registry** (attribute_id,
name, datatype, domain, steward, dq_rules, lineage, classification,
merge_class, state) governing attributes that are *not* universal — they
apply only within their Subject Area, must be registered before first write
(Law E4), and (for EnkiDDB specifically, being distributed) must declare a
`merge_class` (Law E9). None of this collides with Layer 1: Layer 1 is the
seven attributes every particle already has before any Subject Area is even
assigned; Layer 2 is everything else, scoped per Subject Area, and requires
the registry precisely because it *isn't* universal.

---

## What's described but not yet in sealed code — real gaps, not invented ones

Two things the Architect described don't have a home in Layer 1 yet. Rather
than silently minting new `ATTR_*` hashes into this document (which would
repeat GL-EAV-001's mistake of asserting something exists when it doesn't),
these are named as open questions:

### 1. The steward/username on the PDM-shape-creating DubSar service

No `ATTR_STEWARD`, `ATTR_CREATED_BY`, or `ATTR_USERNAME` constant exists
anywhere in the workspace (verified by grep — zero hits). This is real,
wanted, and unbuilt. It doesn't belong in the Hepta-7 (which is fixed at
seven and already named "Hepta" throughout this ecosystem — HeptaScript,
Hepta metric `g = diag(w₁…w₇)`, HeptaShell's 7D lattice; growing it to eight
would break that naming everywhere it's used). The cleaner fit: a second
**mandatory-but-not-Hepta** class — call it **Provenance** — sitting
alongside the Hepta-7, carrying `created_by_username`, `created_by_service`
(e.g. the DubSar PDM template compiler), and `steward` (GL-EAV-001's Law E6
concept, but as a per-particle fact, not just a per-attribute-definition
fact). This is the genuine DAMA Business-metadata slot GL-EAV-001 mistakenly
assigned to κ[7] — it actually belongs here, in EAV space, not in the KAKI
header.

### 2. SLA from the Fuzzy Logic Engine + Score Engine

`sla-engine` is real (`SlaEvaluator::evaluate()` → `SlaReport{domain_scores,
overall_b11, go_live_ready, actions}`) — but it is **not currently wired to**
`fuzzy-engine` or `score-engine`. Checked its `Cargo.toml`: dependencies are
`bahyway-core`, `hepta-sec-policy`, `hepta-sec-firewall` only. It computes
its own, separate `overall_b11` via its own fuzzy scoring, independent of the
Hepta-7's `quality` attribute. Before SLA can be trusted as a mandatory,
always-present attribute the way the Architect described, this needs one of:
(a) `sla-engine` consuming `score-engine::ScoreResult` as an input rather
than re-deriving B11 itself, or (b) an explicit statement that SLA
compliance and particle quality are deliberately independent scores. This
is a real integration gap, not a design gap — the two pieces exist, they
just don't talk to each other yet.

---

## Summary table — what's mandatory, what's optional, what's missing

| Class | Scope | Registration required? | Status |
|---|---|---|---|
| **Layer 0** — KAKI header | every particle | no — sealed at mint | ✅ built, immutable |
| **Layer 1** — Hepta-7 | every particle, every tribe | no — universal by law | ✅ built (state, quality, color_rgb, freshness, snapshot×3) |
| **Layer 1.5** — Provenance (proposed) | every particle | TBD | ❌ not built — steward/username/creating-service |
| **Layer 2** — Subject-Area optional | only within its Subject Area | yes — GL-EAV-001's registry | ⚠️ registry design is sound, zero implementation yet |
| SLA as a mandatory attribute | every particle (Architect's intent) | — | ⚠️ `sla-engine` exists but isn't wired to the Hepta-7 quality pipeline |

---

*Truth Before Beauty · verified against the real repository, not restated
from a design document.*
