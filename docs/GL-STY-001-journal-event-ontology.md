# GL-STY-001 — StoryEngine Journal & Event Ontology Law
**Status:** SEALED (concept). Implementation queued behind PB-160.
**Owners:** StoryEngine (chronicle layer over NUZI);
ENLIL 7-index ring (exact retrieval); NabuEngine SSE +
CompareEngine (semantic tier); EnkiMDB · 7006 (ontology
residence).

## Clause 1 — The Closed Event Ontology
The industry lesson, learned in the negative: Kafka
defines no domain event types (opaque records; internal
control markers and tombstones only) and RabbitMQ ships
a free-form `type` property beside four exchange types —
the vocabulary is always left to the architect, and
ungoverned estates rot into free-string chaos. BahyWay
therefore seals a CLOSED registry:
- Every event type is itself a Template particle in
  EnkiMDB (per GL-TPL-001 Clause 6): versioned, release-
  era named (kings), approved, data-structure referenced,
  AkkadianSeal-signed.
- Encoding: kappa[6] kaki_type = Event; kappa[7]
  kaki_role = the event-type ordinal from this registry.
  256 roles available; ~64-80 expected at genesis.
- Seven founding families: LIFECYCLE (GENESIS,
  ORBIT_SHIFT, PROVE_CERT, KISPU_RITE, REWORK...),
  DECREE (per MADANU verdict, GL-DST-003), TICKET (per
  GL-TKT-001 transition), CONNECTION (Tupsimati rites,
  GL-DST-002), SECURITY (UrNammu attestations, Nisaba
  alerts), THRESHOLD (SUSA scanner provenance), and
  WITNESS (selection events, GL-VIZ-002).
- A new event type enters ONLY through the minting rite
  (steward-approved, CSR-08). Free-string event types
  are an ontology violation, rejected at the WRITE node.

## Clause 2 — The Two-Tier Query Law
TIER 1 — EXACT (ENLIL, deterministic): "all Event-KAKIs
of this Identity-KAKI" is a lineage lookup by uuid_hash
linkage through the ring indexes — O(k) in the event
count, complete, sub-millisecond. No similarity
instrument may be spent on an exact question.
    WITNESS STORY
      WHO identity = kappa <hex>
    PRESENT EVENTS ORBIT ChronicleRing
TIER 2 — SEMANTIC (NabuEngine SSE + CompareEngine):
fuzzy discovery only — "events LIKE this", cross-client
sequence similarity, journal-wide phrase search. Uses
GL-TPL-001 shingles and MinHash signatures over the
CLOSED vocabulary, which keeps Jaccard sharp: sealed
sets cannot drift.
The router is part of the law: exact predicates route
to Tier 1; LIKE/similarity predicates route to Tier 2.

## Clause 3 — The Sedimentation Law
The journal is append-only forever (GL-DST-003; the
plate is a memorial), and forever requires tiers:
- HOT segment: under ENLIL's live ring, write path,
  KISPU-committed.
- COLD sealed segments: rolled by size/era, checksummed,
  AkkadianSeal-signed, mounted read-only; the SSE may
  shingle-index them, the write path never touches them.
Never deletion; only sedimentation. Client-tribe
segments honor tenancy (kappa[4..5]) and the GL-TPL-001
abstraction discipline in any cross-client analysis.

## Authority Note
Inherits all standing law: append-only decrees
(GL-DST-003); pattern minting (GL-TPL-001); ticket law
(GL-TKT-001); witness scopes (GL-VIZ-002); NUZI inward
jurisdiction (memory, lineage, provenance); CSR-08;
NL-001 orthography; the 1-billion-particles-under-
1-second law.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
