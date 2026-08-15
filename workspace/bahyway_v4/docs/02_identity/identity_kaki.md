# Identity-KAKI

> **DubSar Help** | `κ_id` | Identity

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-02"
  concept_type:   "0x01"
  epoch:          "2026-01-01"
  concept_depth:  220
  riksu_count:    2
  snapshot_epoch: "2026-06-06"

concept:          "Identity KAKI"
summary:          "Identity-KAKI is the 16-byte immutable sovereign seal that gives every particle its permanent coordinate in the data universe."
sovereign_laws:   ["§2.4 — no assessments in KAKI nucleus"]

riksu_bindings:
  - target: "adr_003_kaki_sovereignty.md"
    concept: "KAKI byte layout"
    type: "CHILD"
  - target: "master_glossary.md"
    concept: "KAKI declaration"
    type: "PEER"

orbit_tags:       ["KAKI Sovereignty"]
rag_keywords:     ["MINT", "KIŠIB", "KAKI", "identity", "seq_counter", "tribe_id", "kaki_type", "kaki_role"]
-->

---

## Sovereign Declaration — What KAKI Means

**KAKI (Knowledge–Akkadian–Keyword–Identity)**

KAKI is BahyWay Ecosystem v4.0's sovereign approach to Semantic Data
Modeling (SDM) for deterministic Entity Resolution across heterogeneous
data sources.

The name carries two layers of meaning:

**Etymological** — from the Akkadian *kaku* (𒋼𒁀): armament, seal, sovereign
mark. In ancient Mesopotamia, a cylinder seal (*kaku*) was the proof of
identity and authority — pressed into clay, impossible to forge without
the original seal. BahyWay's KAKI is the digital equivalent: a 16-byte
sovereign seal minted once at birth and immutable for the particle's
lifetime.

**Semantic** — Knowledge–Akkadian–Keyword–Identity encodes the four pillars:
- **Knowledge** — the particle carries structured semantic knowledge about a real-world entity via its EAV attribute space
- **Akkadian** — the system's philosophical and etymological root: the first writing system that encoded sovereign identity
- **Keyword** — Knowledge-Aware Keyword Indexing resolves entity identity through seven native sovereign indexes, not through probabilistic string matching
- **Identity** — each particle has one and only one KAKI — its permanent, immutable, sovereign coordinate in the data universe

**Implementation — Knowledge-Aware Keyword Indexing:**
Identity is determined by sovereign structure: tribe membership (`κ[4..5]`),
content hash (`κ[0..3]`), creation ordinal (`κ[8..11]`), and sovereign epoch
(`κ[12..13]`). The 7D VGCA quality vector — not an ML embedding — provides
the geometric measure of how close a particle is to its sovereign ideal.
Entity resolution is deterministic, auditable, and requires no external
model, no training data, and no neural network.

---

## Purpose

The Identity-KAKI is the immutable 16-byte nucleus that uniquely identifies a
particle across all storage nodes and all time. It is the "real component" in
the Octonion analogy.

## Mechanism

- Bytes 0–7: deterministic identity prefix derived from source system keys.
- Bytes 8–15: attribute suffix encoding particle type and creation context.
- Spectral hash maps the Identity-KAKI to a Jordan Block address in O(1).
- Bloom filter at the ADAD Gate pre-checks existence without a full lookup.

## Sovereign Constraints

§2.4: No assessments (scores, rankings, opinions) may be encoded in the
Identity-KAKI nucleus. The nucleus is a coordinate, not a judgment.

## See Also

- `02_identity/kaki_triad.md`
- `04_gates/adad_gate.md`
- `01_mathematics/tri_kaki_index.md`
