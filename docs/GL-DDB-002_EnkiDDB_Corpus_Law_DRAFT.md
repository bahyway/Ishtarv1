# GL-DDB-002 (candidate) — ENKIDDB CORPUS LAW
## Graph-RAG Schema Mechanism · DAMA-DMBOK Concordance · Download → BeeMDM Chain Plan
### BahyWay.Ecosystem v4.0 · Phase Two (GL-STD-002 compliant) · Status: DRAFT — pending CSR-08 sealing by DUB.SAR 𒁾

---

## 0 · Principle

Schema law precedes download. A corpus without a minted schema arrives as a
pile; with one, it arrives as a tribe. This tablet defines the mechanism that
makes every downloaded document a lawful population of particles in EnkiDDB
(7007), governed by the existing EnkiDDB laws — **Simtu Facet Law** (the facets
by which a chunk is categorized), **Šasû Law** (how the corpus is read/queried),
**Eṭemmu Law** (the derived spirit-representations: embeddings and other
projections, never confused with the body of the text) — and scored against
DAMA-DMBOK before any corpus is declared GOLDEN.

Codex compliance: zero new mathematics (A-1); members cited (A-4): GL-TPL-001
pattern minting · Gate G4/Z3 design-time proof · GL-FED-001 Golden Ascent ·
KAKI/EAV separation (locked) · NUZI provenance · τ/ε transparency.

---

## 1 · The Chunk-Particle Model (the Essential EAV)

Every corpus decomposes: **Corpus → Document → Chunk**. Each tier is a particle.

**KAKI v4.0 (canonical, locked, no exceptions):**
κ[0..3] uuid_hash · κ[4..5] tribe_id (one tribe per corpus, e.g. "MIMIC-IV-demo",
"AHD-Arabic") · κ[6] type: 0x01 Identity (corpus, document, chunk),
0x02 Event (ingest, audit, score events), 0x03 CrossTribe (**every Graph-RAG
edge**) · κ[7] role · κ[8..11] reserved zeroed · κ[12..13] timestamp ·
κ[14..15] CRC-16/CCITT. All classification lives in EAV — never in KAKI bytes.

**EAV Mandatory Attributes per chunk (the Essential Model):**

| Group | Attributes |
|---|---|
| Body | text_ref (or asset_ref for binaries), char_span, language (ar/en/…), script direction |
| Provenance | source_url, corpus_id, document_id, NUZI lineage, download_event_kaki |
| **License (non-negotiable)** | license_id (ODbL/CC-BY/DUA-…), license_terms_ref, redistribution_flag — the Golden Store must always answer "under what right do I hold this particle?" |
| Simtu facets | domain (clinical/QA/encyclopedic), specialty, document_kind (note/lab/QA-pair/article), audience |
| Classification codes | mapped vocabularies where applicable: MeSH, ICD, ATC, SNOMED-ref — stored as EAV, minted by the deterministic layer (§3) |
| Quality/State | GOLDEN/FUZZY/DEAD, dedup_hash, completeness, ε (extraction uncertainty) |
| Eṭemmu refs | embedding_ref(s) with model_id + version — the spirit points to the body, never replaces it |

**Graph-RAG edges as particles.** Concept links, dependencies, and groupings are
CrossTribe 0x03 particles: `(chunk|concept) —relation→ (chunk|concept)` with EAV
{relation_kind: same-concept / depends-on / contradicts / translates (ar↔en) /
cites / part-of}, confidence, minting_agent, seal_status. Concept nodes
themselves are Identity particles in a Concept tribe (one per vocabulary).
**Consequence:** categorization, classification & grouping are auditable
populations, not metadata — the graph can be ORBITed, PROVEn, and WITNESSed
like any tribe, and HS-EXT-003's rocket view applies to the corpus itself.

Schemas are minted as **template particles** (GL-TPL-001), proven composite-
satisfiable at Gate G4 (Z3, design-time only), and registered in EnkiMDB.

---

## 2 · DAMA-DMBOK Concordance & Scoring (the "must score high" law)

Each DMBOK knowledge area maps to an existing BahyWay law; compliance is not an
essay but a **scorecard particle** per corpus (Event KAKI + EAV scores 0–100,
with ε on each score honestly displayed):

| DMBOK Knowledge Area | BahyWay instrument |
|---|---|
| Data Governance | CSR-08 sealing · GL-STD-002 · this tablet |
| Data Architecture | seven-Enki pipeline, one tribe per corpus |
| Data Modeling & Design | GL-TPL-001 templates + Gate G4 proof |
| Storage & Operations | RAID-Z2 datasets · GL-OPS-001/002 HA-DR |
| Data Security | ABAC (AkkadiRulesEngine) · UrNammu · Ed25519 seals |
| Integration & Interoperability | BeeMDM stations · SUSA boundary · vocabulary mappings |
| Documents & Content | EnkiDDB itself · Simtu/Šasû/Eṭemmu laws |
| Reference & Master Data | Concept tribes (MeSH/ICD/ATC) · GL-MDM-001 |
| DWH & BI | EnkiDW · Golden Ascent (GL-FED-001) |
| Metadata | EnkiMDB (read-only at runtime, Nergal-defended) |
| Data Quality | GOLDEN/FUZZY/DEAD assignment · PB-326-style audit rules · τ/ε |

**Success threshold (S-DDB):** a corpus is declared GOLDEN-ingested only when
(a) every knowledge-area score ≥ 80 with no area below 70, (b) license
attributes present on 100% of particles, (c) dedup + checksum audits pass,
(d) the scorecard particle itself is sealed. Scores below threshold route the
corpus (or its failing subset) to EnkiQDB as quarantine — evidence, not shame.

---

## 3 · The Agent Architecture (categorization with law)

Three layers, strictly ordered, respecting the stewardship split:

1. **Deterministic layer — NARAMSIN class.** Vocabulary mapping (MeSH/ICD/ATC
   lookups), language detection, dedup hashing, license extraction, structural
   validation against the minted template. Output: sealed EAV facts. This layer
   alone may write classification codes as GOLDEN.
2. **Statistical layer — Eṭemmu plane.** Embedding generation + clustering for
   candidate groupings. Outputs are *candidates* (FUZZY by birth), stored as
   Eṭemmu refs and provisional edges awaiting confirmation.
3. **Advisory semantic layer — NINSUN class (É-DUBBA Stage 2b precedent).**
   The AI agent that searches for needed concepts and their link dependencies:
   proposes CrossTribe concept edges, flags semantic anomalies, suggests
   missing facets. **Advisory-only, always**: `advisory=true` on every emission,
   no blocking power, no cryptographic authority. Its proposals become GOLDEN
   edges only after deterministic corroboration or DUB.SAR confirmation at the
   gate. Candidate sovereign name for this corpus-librarian role: **Ummânu**
   (Akk. scholar/expert) — naming pending CSR-08; until sealed it runs
   anonymously as a NINSUN-class advisor.

---

## 4 · Plan of Steps (design → download → BeeMDM chain)

**Step 0 — this tablet** sealed or amended by DUB.SAR.

**Step 1 — Schema minting suite (PB-331…PB-333):**
- PB-331: `enkiddb-corpus-schema` crate — template particles for Corpus /
  Document / Chunk / ConceptNode / ConceptEdge; KAKI minting; EAV validation;
  unit tests (rule parity with any Šala inspector).
- PB-332: Concept tribes bootstrap — load open vocabularies (MeSH descriptors,
  ICD-10 codes, ATC) as Identity particles with license attributes.
- PB-333: DMBOK scorecard engine — computes the §2 scores as Event particles;
  thresholds as constants (byte-identical to any display).

**Step 2 — Download & landing suite (PB-334…PB-335):**
- PB-334: download-verify — fetch MIMIC-IV-demo (open), AHD (Mendeley, CC),
  MAQA; checksum, capture license text, mint download Event KAKIs; land in
  EnkiSDB. Credentialed sets (full MIMIC-IV, AmsterdamUMCdb) enter later via
  the same PB pattern once DUB.SAR's PhysioNet credentialing completes —
  the playbook downloads nothing it has no right to.
- PB-335: Synthea sibling — synthetic clinical patients for scale testing
  (engineering tier; real data remains the validation tier).

**Step 3 — First-ingest rite (PB-336…PB-337):** the eight-step DataStructure
Station rite per corpus (Landing→Safety→Inference→Arsenal match→Stakeholder
PDM→Gate G4→SLA dual seal→ETL to GOLDEN), then chunking + §3 agent layers +
edge minting into EnkiDDB; quarantine routing to EnkiQDB on any S-DDB failure.

**Step 4 — BeeMDM stations chain.** These corpora become the BeeMDM ETL
workload. **Governing law intact:** the BeeMDM run itself remains gated behind
completion of the existing playbook program and TESTING_PLAYBOOK_PHASE1 Blocks
A–F. Steps 1–3 are preparation and may proceed; Step 4 waits its turn.

---

## 5 · Extra Domain Databases — the verdict

**Avoidable in kind, twice unavoidable in annex.** The seven Enki types are
closed law; a new domain never means a new database type — it means new
**tribes/collections** inside the seven (a Medical-corpus tribe, a Concept
tribe, a License tribe). The two lawful annexes:
1. **Asset annex** (already precedented by forge/assets on the RAID-Z2 pool):
   large binaries — waveforms, images, PDFs — live as content-addressed blobs;
   EnkiDDB holds chunk particles with asset_refs, never the blobs themselves.
2. **Eṭemmu index**: the vector/similarity index for embeddings lives *inside*
   EnkiDDB's index stack (ENLIL lineage — and per the ring doctrine, no trees),
   as an index, not a database. A separate "vector DB" product is rejected:
   it would be an eighth database and a sovereignty breach in one.

## 6 · Open seals for CSR-08
GL-DDB-002 adoption · Ummânu naming for the corpus-librarian advisor ·
S-DDB thresholds (80/70) · PB-331…337 numbering · whether the Concept tribes
constitute the sector's Reference-&-Master-Data annex under GL-MDM-001.

*Recorded in the reign of Gudea 1.0, Phase Two. Nothing herein is sealed until
DUB.SAR confirms under CSR-08.*
