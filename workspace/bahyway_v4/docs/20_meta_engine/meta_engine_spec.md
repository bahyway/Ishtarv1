# MetaEngine — Sovereign Documentation Control Layer

> **DubSar Help** | `MetaEngine > Spec` | Architecture

**Version:** 1.0
**Date:** 2026-06-06
**Status:** Canonical — controls all BahyWay v4.0 documentation

---

## Concept — What the Visualization Shows

The image that inspired MetaEngine shows:

```
[Binary Wall / Raw Journal]  ←  multiple colored orbital streams  ←  [Two focal attractor rings]
                                                                        ↑
                                                            Tribe→Orbit→Particle triality
```

Every documentation file in BahyWay v4.0 is a **doc-particle**:
- It belongs to a **doc-tribe** (one of 14 documentation sectors)
- It orbits in one or more **concept-orbit rings** (shared sovereign concepts)
- It is identified by a **doc-KAKI** header (seven sovereign index fields)
- Cross-subject links are **RIKSU bindings** between doc-particles — NOT hyperlinks

The MetaEngine is the sovereign layer that:
1. Assigns and validates doc-KAKI headers on every `.md` file
2. Builds the 7D orbital index for RAG retrieval
3. Enforces the meta-template for all documentation tribes
4. Generates the orbital visualization data consumed by DubSar Observatory

---

## Seven Doc-Tribes (Documentation Sectors)

Each doc-tribe corresponds to a directory. Every file belongs to exactly ONE tribe.

| Tribe ID | Tribe Name | Directory | Orbit Attractor Color |
|---|---|---|---|
| `DT-00` | Codex | `00_codex/` | White |
| `DT-01` | Mathematics | `01_mathematics/` | Cyan |
| `DT-02` | Identity | `02_identity/` | Gold |
| `DT-03` | Kernel | `03_kernel_mummu/` | Orange |
| `DT-04` | Gates | `04_gates/` | Red |
| `DT-05` | Storage | `05_storage/` | Blue |
| `DT-06` | Governance | `06_governance_parzu/` | Purple |
| `DT-09` | Observatory | `09_observatory/` | Teal |
| `DT-14` | Decisions ADR | `14_decisions_adr/` | Green |
| `DT-20` | MetaEngine | `20_meta_engine/` | Magenta |
| `DT-99` | Index | `99_index/` | Silver |
| `DT-ST` | StartHere | `_start_here/` | Yellow |
| `DT-LA` | Languages | `09_languages/` | Pink |
| `DT-EX` | Examples | `12_examples/` | Lime |

---

## Seven Sovereign Doc-Indexes (7D Orbital Space)

These are the seven dimensions of the orbital visualization. They correspond
directly to the seven native sovereign indexes from ADR-007, adapted for
documentation particles.

### Index 1 — Concept Hash (κ[0..3] analog)
FNV-1a hash of the document's primary concept name (the H1 title stripped
of articles and punctuation). Used for exact concept retrieval.

```
concept_hash = fnv1a_32(normalize(doc.primary_concept))
```

### Index 2 — Tribe Membership (κ[4..5] analog)
The doc-tribe identifier (DT-00 through DT-LA). A doc-particle can belong
to only one tribe. Cross-tribe visibility is computed via RIKSU bindings,
never stored.

### Index 3 — Concept Type (κ[6] analog)
The semantic type of the documentation particle:

| Type Code | Meaning |
|---|---|
| `0x01` | Definition (glossary entry, term declaration) |
| `0x02` | Decision (ADR, architectural record) |
| `0x03` | Specification (formal language/protocol spec) |
| `0x04` | Procedure (howto, runbook, operational guide) |
| `0x05` | Reference (index, master glossary, cross-reference) |
| `0x06` | Mathematical (theorem, algebra, proof) |
| `0x07` | Example (worked example, code sample) |

### Index 4 — Sovereign Epoch (κ[12..13] analog)
The creation or last-superseded epoch of the document. Format: `YYYY-MM-DD`
encoded as `(year - 2020) × 10000 + month × 100 + day` → 16-bit integer.

### Index 5 — Concept Depth (VGCA quality analog)
A 0–240 scalar measuring how deep this document goes into its concept
(from introductory overview = low, to full mathematical specification = high).
Computed as: `round(completeness_score × 240)`.

### Index 6 — RIKSU Binding Count (Graph layer analog)
Number of RIKSU bindings (cross-document concept links) declared in this
doc-particle's header. High binding count = orbit attractor node (appears
at focal points in the visualization, like the pink/yellow attractors in
the image).

### Index 7 — Snapshot Epoch (sparse B-tree analog)
The epoch at which this document was last snapshot-reviewed for canonical
accuracy. Enables incremental RAG re-indexing: only docs whose
snapshot_epoch < current_review_cycle need re-embedding.

---

## Doc-KAKI Header — Sovereign Meta Template

Every documentation file MUST begin with this header block immediately after
the H1 title and DubSar breadcrumb. This is the **mandatory meta-template**
enforced by MetaEngine.

```yaml
<!--META
doc_kaki:
  concept_hash:   "0x{8-char FNV-1a hex}"
  tribe:          "DT-{code}"
  concept_type:   "0x0{1-7}"
  epoch:          "YYYY-MM-DD"
  concept_depth:  {0-240}
  riksu_count:    {integer}
  snapshot_epoch: "YYYY-MM-DD"

concept:          "{primary concept name}"
summary:          "{one sentence, max 120 chars}"
sovereign_laws:   ["{§ref}", "{§ref}"]

riksu_bindings:
  - target: "{other-doc-filename.md}"
    concept: "{shared concept name}"
    type: "{PARENT|CHILD|PEER|SUPERSEDES|GROUNDS}"
  - target: "..."

orbit_tags:       ["{concept-orbit-ring-name}", "..."]
rag_keywords:     ["{keyword}", "..."]
-->
```

### Field Rules

| Field | Rule |
|---|---|
| `concept_hash` | FNV-1a-32 of normalized primary concept — computed by MetaEngine |
| `tribe` | Exactly one DT-code from the tribe table — no multi-tribe assignment |
| `concept_type` | One of 0x01–0x07 — compile error if missing |
| `epoch` | ISO date of original creation — immutable after KIŠIB |
| `concept_depth` | 0–240 — NEVER 255 (QUALITY_DIVISOR = 240.0, ADR-001) |
| `riksu_count` | Auto-computed from riksu_bindings length — MetaEngine validates |
| `snapshot_epoch` | Updated each review cycle — the only mutable field |
| `riksu_bindings` | Cross-doc RIKSU links — replaces markdown hyperlinks as primary navigation |
| `orbit_tags` | Concept orbit rings this doc-particle belongs to |
| `rag_keywords` | Sovereign vocabulary terms for RAG retrieval — MUST use HeptaScript vocabulary, no SQL terms |

---

## Orbital Visualization Data Model

The MetaEngine generates a JSON orbital map consumed by the Observatory
visualization renderer (docs/09_observatory/). This is the data behind the
image you shared — each focal attractor is an orbit ring, each colored
stream is a doc-tribe's particles flowing toward shared concept attractors.

```json
{
  "orbital_map": {
    "version": "1.0",
    "generated_epoch": "YYYY-MM-DD",
    "tribes": [
      {
        "tribe_id": "DT-14",
        "tribe_name": "Decisions ADR",
        "color": "#00FF88",
        "particles": [
          {
            "doc_kaki": "0xA1B2C3D4",
            "filename": "adr_010_heptascript_language_design.md",
            "concept": "HeptaScript Language Design",
            "concept_depth": 210,
            "riksu_count": 6,
            "orbit_tags": ["HeptaScript", "Language", "Sovereign Vocabulary"],
            "position_7d": [0.72, 0.14, 0.03, 0.88, 0.21, 0.60, 0.45]
          }
        ]
      }
    ],
    "orbit_attractors": [
      {
        "ring_name": "KAKI Sovereignty",
        "color": "#FFD700",
        "bound_particles": ["adr_003_kaki_sovereignty.md", "identity_kaki.md", "master_glossary.md"],
        "centroid_7d": [0.65, 0.50, 0.02, 0.75, 0.80, 0.90, 0.70]
      },
      {
        "ring_name": "OOO Mathematical Foundation",
        "color": "#FF69B4",
        "bound_particles": ["adr_008_ooo_foundation.md", "enlil_algebra.md", "top_algebra.md", "adr_009_algebra_additions.md"],
        "centroid_7d": [0.80, 0.30, 0.06, 0.60, 0.90, 0.85, 0.65]
      }
    ],
    "riksu_streams": [
      {
        "from": "adr_010_heptascript_language_design.md",
        "to": "adr_008_ooo_foundation.md",
        "concept": "OOO Mathematical Operations",
        "stream_color": "#FF69B4"
      }
    ]
  }
}
```

---

## MetaEngine RAG Pipeline — Seven Steps

The MetaEngine controls documentation for EnkiDB RAG through seven
sovereign operations, one per index dimension:

```
Step 1 — MINT doc-particles
  For each .md file: parse META block, assign concept_hash via FNV-1a,
  validate all 7 index fields. Reject files missing META block.

Step 2 — BUILD Index 1 (Concept Hash B-tree)
  FNV-1a-32 → exact concept lookup. O(1) retrieval for known concepts.

Step 3 — BUILD Index 2 (Tribe B-tree)
  Group particles by DT-code. Tribe-scoped RAG queries never cross tribe
  boundaries (CrossTribe state computed on PROBE, not stored — ADR-008 §3).

Step 4 — BUILD Index 3 (Type B-tree)
  Group by concept_type. A RAG query for "how do I..." routes to 0x04
  (Procedure); "what is..." routes to 0x01 (Definition) or 0x06 (Math).

Step 5 — BUILD Index 5 (Concept Depth ranking)
  For concept retrieval: prefer depth ≥ 150 for technical queries;
  depth < 80 for onboarding queries. Routing is automatic from query type.

Step 6 — BUILD Index 6 (RIKSU graph)
  PageRank over the RIKSU binding graph. High-rank nodes = orbit attractors
  (appear at focal points in the visualization). These are the sovereign
  "pink and yellow attractor nodes" from the image.

Step 7 — BUILD Index 7 (Snapshot B-tree)
  Track snapshot_epoch per doc-particle. On each RAG re-index cycle:
  only docs with snapshot_epoch < current_review_cycle need re-processing.
  O(log k) incremental re-indexing — not O(n) full rescan.
```

---

## RAG Query Routing — How MetaEngine Handles a Query

```heptascript
-- MetaEngine RAG PROBE (internal operation)
PROBE particle rag_query
  WITHIN TRIBE docs

  -- Step 1: route by concept type
  ASSESS concept_type_router(query.intent) AS doc_type

  -- Step 2: retrieve by concept hash (Index 1)
  THRESHOLD concept_hash IN index_1_btree(query.normalized_concept)

  -- Step 3: rank by RIKSU attractor proximity (Index 6)
  ASSESS pagerank_score(p) AS rank

  -- Step 4: filter by concept depth appropriate to query complexity
  THRESHOLD concept_depth >= complexity_threshold(query)

  -- Step 5: cross-tribe expansion via RIKSU bindings only
  RIKSU expand(p, depth=2)

  YIELD p.filename, p.summary, p.concept_depth, rank
  GROUPED BY orbit_tag
  LAST 1 snapshot_epoch
```

---

## MetaEngine Enforcement Rules

The MetaEngine enforces these sovereign laws on all documentation:

1. **No META-less files** — any `.md` file without a valid `<!--META` block
   is rejected from the RAG index (flagged, not deleted — ADR-006)
2. **No SQL keywords in rag_keywords** — ADR-010 applies to documentation
   metadata, not just code examples
3. **concept_depth ≤ 240** — QUALITY_DIVISOR = 240.0 is ecosystem-wide
4. **riksu_count must equal len(riksu_bindings)** — MetaEngine validates,
   mismatch is a compile error
5. **epoch is immutable after first commit** — snapshot_epoch is the only
   field that may be updated
6. **orbit_tags must reference known orbit ring names** — undefined rings
   are rejected (prevents orbital fragmentation)

---

## Crate Ownership

| Component | Crate | Status |
|---|---|---|
| META block parser | `meta-engine` | To build |
| FNV-1a concept_hash | `kaki-core` (FNV-1a already present) | Exists |
| 7D index builder | `meta-engine` | To build |
| Orbital JSON generator | `meta-engine` | To build |
| RAG PROBE router | `meta-engine` + `heptascript-engine` | To build |
| DubSar visualization | `dubsar-observatory` | To build |
| PageRank (RIKSU graph) | `graph-engine` (ADR-009 §1) | Not built |

---

## References

- ADR-003: KAKI Sovereignty (seq_counter, FNV-1a hash)
- ADR-007: Index 7 snapshot sparse B-tree (incremental re-indexing)
- ADR-008: OOO Foundation (IDU Probing Rule, CrossTribe non-storage)
- ADR-009: Graph Algebra PageRank (RIKSU attractor scoring)
- ADR-010: HeptaScript sovereign vocabulary (rag_keywords rule)
- `docs/09_observatory/orbital_visualization.md` — visualization renderer
- `docs/_meta/01_templates/sovereign_doc_template.md` — template file
