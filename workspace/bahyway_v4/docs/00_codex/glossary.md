# Glossary

> **DubSar Help** | `Codex > Glossary` | Non-Negotiables

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-00"
  concept_type:   "0x01"
  epoch:          "2026-01-01"
  concept_depth:  200
  riksu_count:    2
  snapshot_epoch: "2026-06-06"

concept:          "Sovereign Glossary"
summary:          "Authoritative sovereign glossary of all BahyWay v4.0 terms — KAKI, OOO, HeptaScript, VGCA, and Akkadian primitives."
sovereign_laws:   []

riksu_bindings:
  - target: "master_glossary.md"
    concept: "master index"
    type: "PEER"
  - target: "adr_003_kaki_sovereignty.md"
    concept: "KAKI definition"
    type: "CHILD"

orbit_tags:       ["KAKI Sovereignty", "OOO Mathematical Foundation", "HeptaScript Sovereign Language"]
rag_keywords:     ["KAKI", "ORBIT", "PROBE", "MINT", "APPEND", "ASSESS", "OOO", "VGCA", "DUB", "ME", "RIKSU", "KIŠIB"]
-->

---

## KAKI — Sovereign Declaration

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

**Semantic** — Knowledge–Akkadian–Keyword–Identity encodes the four pillars
of the framework:
- **Knowledge** — the particle carries structured semantic knowledge about a real-world entity via its EAV attribute space
- **Akkadian** — the system's philosophical and etymological root: the first writing system that encoded sovereign identity
- **Keyword** — Knowledge-Aware Keyword Indexing (KAKI) resolves entity identity through seven native sovereign indexes, not through probabilistic string matching
- **Identity** — each particle has one and only one KAKI — its permanent, immutable, sovereign coordinate in the data universe

**Implementation — Knowledge-Aware Keyword Indexing:**
KAKI resolves records representing the same real-world entity through
Semantic Data Modeling rather than probabilistic similarity scoring.
Identity is determined by sovereign structure: tribe membership (`κ[4..5]`),
content hash (`κ[0..3]`), creation ordinal (`κ[8..11]`), and sovereign epoch
(`κ[12..13]`). The 7D VGCA quality vector — not an ML embedding — provides
the geometric measure of how close a particle is to its sovereign ideal.
Entity resolution is deterministic, auditable, and requires no external
model, no training data, and no neural network.

---

## Terms

| Term | Definition |
| :--- | :--- |
| **KAKI** | 16-byte sovereign particle identity seal: `uuid_hash` · `tribe_id` · `kaki_type` · `kaki_role` · `seq_counter` · `timestamp` · `checksum`. See declaration above. |
| **HPS** | Hepta Priority Score — a scalar in [0,1] derived from the 7D Hepta vector. |
| **δ_T(p)** | Quality distance: 1 − HPS(p). Measures divergence from Tribe Ideal. |
| **Orbit** | The Jordan Chain of historical state transitions for one KAKI. |
| **Tribe** | A mathematically isolated manifold (Jordan Block) of related particles. |
| **Golden Record** | A particle with δ_T(p) → 0; the idealized ground truth. |
| **Dead Particle** | A particle with δ_T(p) → 1; archived, no active motion. |
| **Stewardship Gap** | Transition state between Dead and Active; requires Data Steward intervention. |
| **ADAD** | Gate 1 — Temporal Exclusion (signal de-duplication). |
| **ANU** | Gate 2 — Authority Exclusion (source hierarchy). |
| **MARDUK** | Gate 3 — Structural Exclusion (transformation lock). |
| **SHAMASH** | Gate 4 — State Exclusion (Dead vs. Active judgment). |
| **PA-n** | Theorem n in the Particles Algebra specification. |
| **TOP Algebra** | Ṭupšarrūtu Algebra — maps Tribe-Orbit-Particle to Octonion structure. |
| **Enlil Algebra** | The Jordan Normal Form algebra governing EnkiDB indexing and sharding. |
| **seq_counter** | `κ[8..11]` — per-tribe-per-epoch creation ordinal; makes KAKI uniqueness deterministic and enables gap detection (ADR-003). |
| **KISHIB** | KAKI logical role `0x01` — External Document (sovereign witness to an external artefact). |
| **ZIKRU** | KAKI logical role `0x02` — Record (structured sovereign knowledge in EAV space). |
| **PARZU** | KAKI logical role `0x03` — Logic / Template (sovereign procedural knowledge). |

