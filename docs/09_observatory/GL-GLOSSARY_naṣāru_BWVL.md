# GLOSSARY — naṣāru / BWVL · The Symbolic Visualization Programming Language
## BahyWay.Ecosystem v4.0 · consolidates GL-VIZ-003/004/005/006 · GL-KAKI-002 · GL-MED-003 · GL-HS3-001 · Status: SEALED-CONCEPT (per CSR-08 chat confirmation, 2026-08-15)

*Reign of Gudea 1.0. Terms are grouped by theme. Every term here is DRAFT until sealed.*

---

## I · The language and its soul

**naṣāru** (𒌷 URU₃; root n-ṣ-r) — the Akkadian name of BWVL. Verb: "to guard,
watch, protect, preserve, treasure." Names the *act* of guarding-and-watching the
truth of the data. Also the astronomers' verb for the star-watch (*maṣṣartu
naṣāru* = "to stand guard / to gaze at the stars"). A verb, not a deity — clean
under the "gods = engines" law. Runtime codename form: **nasaru** (plain Latin).

**BWVL** — BahyWay Visual Language: the Symbolic Visualization Programming
Language. NOT a chart, NOT a graph tool — a *language for navigating scale
itself*, from the billion-particle field to the single particle. "NOT EQUAL TO
ANYTHING ALREADY KNOWN."

**Bārû** — the human Pattern-Diviner / Visual Analyst who reads the naṣāru field
for hidden patterns; reads GOLDEN by Identity-KAKI + EAV, reads provenance by the
Event-KAKI chain.

---

## II · Particle Monism (GL-VIZ-003)

**Particle** — the one and only visual primitive of BWVL. Everything that exists
has a KAKI; to have a KAKI is to be a particle; to be a particle is to be
visualizable. The particle is the atom of meaning and the start-point of all.

**Particle Monism** — the law that there is exactly one primitive. Node-ness,
edge-ness, label-ness, icon-ness, image-ness are **patterns particles form at a
zoom depth**, not distinct kinds of thing. THE difference from every knowledge
graph (which reifies nodes and edges as two primitives).

**Dissolve** — the act of entering any element (cluster, filament, label, icon,
tribe) and having it resolve into its own constituent particles. Local
per-element depth: a relation dissolves into the records that *are* it; a node
into its members; a record into its KAKI anatomy.

**Cluster** — particles read as a "node" at a scale (a dense assembly).
**Filament** — particles read as an "edge" at a scale (a stream). An edge is
**not a line** — it is the records that ground the relation.

**Generator** — the runtime engine: query → GOLDEN retrieval → particles
*arranged* (not built) into clusters/filaments/labels. The product is the
generator, not any one example scene.

---

## III · The three KAKI v4.0 types (GL-KAKI-002)

**Identity-KAKI** — immutable 16-byte identity, minted at birth. κ[0..3] uuid_hash
· κ[4..5] tribe_id · κ[6] kaki_type · κ[7] kaki_role · κ[8..11] reserved ·
κ[12..13] timestamp · κ[14..15] CRC-16. Carries NO colour/state/quality. Answers
*who is this particle?*

**Event-KAKI** — append-only; one per event (each state / ColourID change through
ETL stations). Read by the StoryEngine Journal to tell history and reveal origin.
Answers *what has happened to it?* The first event (birth) is permanent → origin
always recoverable.

**CrossTribe-KAKI** — derived from 7D Hepta near-adjacency (no two particles share
a post, so "near" across tribes is meaningful). Defines tribe-to-tribe location
and relation; federates the BIGRINGs. Answers *how do tribes relate?*

---

## IV · ColourID lifecycle (GL-VIZ-004)

**ColourID** — a Mandatory EAV attribute on Entity=Tribe (NEVER in KAKI, because
it is mutable). The particle's colour.

**Birth Root Shade** — the immutable tribe/lineage colour assigned at first
ingest (with Identity-KAKI). Never changes; revealed on right-click / StoryEngine
→ shows origin & tribe. Immutable because no Event-KAKI overwrites the birth
event.

**Root colour / hue** — the tribe/lineage band. Domain sets base hue; sub-family
narrows it. Shared by kin.

**Shade-degree** — each particle's OWN unique RGB within its root band. No two
particles share it → a per-particle **visual fingerprint** distinguishable by eye
at billion-scale without reading KAKI.

**State-colour** — the surface colour showing the particle's current state
(blue/green/maroon/red/purple through ETL stations; GOLDEN in the store).

**Bounded GOLDEN transition** — at the golden store the colour shifts paler /
slightly yellowish, **never brown or black**, never corrupting the Birth Root
Shade.

**Aging / Decay** — Steward-governed: an append-alert on a GOLDEN record →
Data Steward marks OLD/Aged/Decay → content-in-use changes, root shade preserved.

**Distinguishing vs grouping** — particles are *distinguished/identified* by KAKI
+ EAV (authoritative); colour *groups* by lineage and *fingerprints* individuals
for the eye.

---

## V · Federation & scale (GL-VIZ-005, GL-VIZ-006)

**Tribe** — a root category / lineage; renders as a BIGRING in its root
colour-band.

**BIGRING** — a tribe's ring-structure in the field.

**Federation of BIGRINGs** — the widest view: many tribes as coloured rings,
positioned and related by CrossTribe-KAKI in Multi-HeptaSpace.

**Bird's-eye** — the zoom-out motion: read the WHOLE as a field (tribes by hue,
structure by CrossTribe, density & void). Enigmatic focus-camera flies to the
chosen region.

**Hubble-descent** — the zoom-in motion: field → region → cluster → the single
particle (its KAKI + EAV). Continuous, not stepped.

**Zoom as Necessity** — the capstone law: the two motions are not features but the
**existence-condition** of a readable billion-particle instrument. Without the
descent, self-describing particles at scale are visible but unreachable (the
Neo4j failure — "a gorgeous prison").

---

## VI · Data lifecycle & stores

**Online source** — public medical API data (e.g. Merck-grade GOLDEN sources)
downloaded by the pipeline.

**BeeMDM ETL** — the pipeline that ingests source data, births particles
(Identity-KAKI + Birth Root Shade), registers state as events (Event-KAKI) through
processing stations, until a particle reaches GOLDEN. **StoryEngine lives here
(upstream).**

**GOLDEN** — the settled state: full KAKI + EAV, state no longer changes. Read by
the Bārû via Identity-KAKI + EAV (not event history).

**EnkiDB** (port 7004) — OLTP golden store; saves GOLDEN particles.
**EnkiDW** (port 7005) — OLAP golden warehouse; partitions/snapshots from EnkiDB.
**EnkiDDB** (port 7007) — Graph-RAG "Patterns Detection Data Lifecycle
Instrument"; receives GOLDEN particles and builds its OWN category schemas for the
scenario. BWVL is based on EnkiDDB's Graph-RAG architecture.

**HeptaScript** — the Anti-SQL sovereign query language (ORBIT/EMIT/PROVE/SYNC/
WITNESS; ASK/PROVE/GHOST/WITNESS for grounded queries). Never SELECT/JOIN.

**HeptaScript Notebook** — the interactive surface where the Bārû writes ASK
queries that simulate and visualize the internal data lifecycle as a naṣāru
scene.

**Four-outcome honesty contract** — every answer is FACT (grounded solid) /
WEAK (thin) / GHOST (unproven, "needs research") / NONE (absence shown). BWVL
never fakes a connection.

---

## VII · The medical model (GL-MED-003)

**Symptom / Disease / Image particles** — the three GOLDEN particle kinds of the
medical model; edges: `presents_in`, `discriminates`, `differentiated_from`,
`shows`.

**Convergence query** — symptom-first: drop observed field signs → diseases rank
by weighted convergence → the discriminating sign is the finding. The inversion a
manual cannot do.

---

*Recorded in the reign of Gudea 1.0. Sealed under CSR-08 (chat confirmation), 2026-08-15 — see the Seal section below.*

## VIII · Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-15
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work, not fabricated here.
```
