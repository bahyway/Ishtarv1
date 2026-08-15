# GL-DDB-001 — Simple Conceptual Graph (SCG) for the EnkiDDB Knowledge Graph
**RECOVERY COPY — assembled 2026-08-05 from the sealed session record
("SCG mapping tribes to Knowledge Graph with W5H2 framework", 2026-07-27).
Section 1 is VERBATIM from that session; sections 2–4 are faithful
reconstructions marked as such. The authoritative original was
GL-DDB-001_EnkiDDB_SCG_Design.md; if a disk copy resurfaces, it rules.**

**Status:** Design draft under CSR-08. **Scope:** EnkiDDB (Documents, port
7007). **Formalism:** Sowa Simple Conceptual Graphs (concepts in `[ ]`,
relations in `( )`), mapped onto KAKI particles, EAV Mandatory Attributes,
and W5H2.

---

## 1. Core Mapping Principle  *(verbatim)*

| SCG element | BahyWay v4.0 realization |
|---|---|
| Concept node | KAKI particle (16-byte κ) with unique 7D Hepta Space position |
| Concept type `TRIBE` | Category — `tribe_id` in κ[4..5] |
| Concept type `PARAGRAPH` | Identity particle, κ[6] = 0x01, content in EnkiDDB, quality in EAV |
| Concept type `CODE_SNIPPET` | Identity particle, κ[6] = 0x01, language/form in EAV |
| Cross-category relation | CrossTribe particle, κ[6] = 0x03 |
| Conceptual relation labels | W5H2 clause words (EAV Mandatory Attributes) |
| Graph provenance | NUZI lineage; seal via AkkadianSeal (Ed25519) at É-DUBBA |

**W5H2 ↔ Sowa relation correspondence** (this is the elegant part — Sowa's
canonical relations already *are* W5H2):

| W5H2 | Sowa relation | EAV Mandatory Attribute |
|---|---|---|
| WHO | (AGNT) agent | `w5h2.who` — scribe / source system |
| WHAT | (THME) theme | `w5h2.what` — topic / content class |
| WHEN | (PTIM) point-in-time | `w5h2.when` — mirrors κ[12..13] |
| WHERE | (LOC) location | `w5h2.where` — section path in the document orbit |
| WHY | (RSON) reason | `w5h2.why` — intent / governing law reference |
| HOW | (MANR) manner | `w5h2.how` — form: prose \| hepta \| rust \| akk |
| HOW MUCH | (MEAS) measure | `w5h2.howmuch` — token count, κ-count, Φ metrics |

All seven live **exclusively in EAV** — never in KAKI bytes (canonical
layout law).

## 2. Example SCG Instance  *(reconstructed)*

A paragraph documenting the KISPU commit, written by the Architect,
belonging to the ENGINES tribe, citing the GL-DST-003 law:

    [PARAGRAPH: κ 7a31…] -
        (AGNT/WHO)  -> [SCRIBE: DUB.SAR]
        (THME/WHAT) -> [TOPIC: KISPU four-way commit]
        (PTIM/WHEN) -> [ERA: Zagesi · κ timestamp]
        (LOC/WHERE) -> [SECTION: engines/kispu/overview]
        (RSON/WHY)  -> [LAW: GL-DST-003]
        (MANR/HOW)  -> [FORM: prose+rust]
        (MEAS/HOWMUCH) -> [MEASURE: 214 tokens]

A CrossTribe particle (κ[6]=0x03) bridges this paragraph to the CODE
tribe's snippet particle implementing the same commit — the bridge itself
carries its own W5H2 EAV set describing the relation.

## 3. HeptaScript Access  *(reconstructed)*

    PRESENT PARAGRAPH
      WHAT  topic = "KISPU"
      WHY   law   = GL-DST-003
    ORBIT DocumentRing

    WITNESS BRIDGE
      WHO tribe_a = DOCS · tribe_b = CODE
    EMIT SelectionEvent

## 4. Governance  *(reconstructed)*

Ingestion of any tablet into EnkiDDB is a NUZI-recorded, sealed act; the
graph is append-only per GL-DST-003; topic assignment is advisory-only
(NINSUN pattern) with the Architect ratifying; and every KG node/edge is
HeptaScript-queryable through the W5H2 clause words, so the knowledge
graph audits itself with the ecosystem's own tongue.

— Recovered for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
