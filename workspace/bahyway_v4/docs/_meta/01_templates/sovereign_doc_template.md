# [Primary Concept Name]

> **DubSar Help** | `[Tribe] > [Concept]` | [Category]

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-00"
  concept_type:   "0x01"
  epoch:          "YYYY-MM-DD"
  concept_depth:  0
  riksu_count:    0
  snapshot_epoch: "YYYY-MM-DD"

concept:          "[Primary concept name — matches H1 title]"
summary:          "[One sentence, max 120 chars, no SQL vocabulary]"
sovereign_laws:   []

riksu_bindings:   []

orbit_tags:       []
rag_keywords:     []
-->

---

## [Section 1 — What This Concept Is]

[Definition. Pure mathematical or sovereign meaning. No analogies to SQL,
relational databases, or external frameworks.]

## [Section 2 — BahyWay Action]

[What operation does this concept enable in BahyWay v4.0? Which HeptaScript
verb maps to it? Which crate implements it?]

## [Section 3 — Sovereign Constraints]

[§ references from the relevant ADR. One bullet per constraint.]

- §X.Y: [constraint text]

## [Section 4 — RIKSU Bindings — Related Concepts]

[Do NOT use markdown hyperlinks as the primary navigation. List RIKSU
bindings here — they mirror the riksu_bindings in the META block.]

| Target Doc | Shared Concept | Binding Type |
|---|---|---|
| `[filename.md]` | [concept] | PEER \| PARENT \| CHILD \| GROUNDS |

## [Section 5 — Example] *(if concept_type = 0x07 or 0x03)*

```heptascript
[HeptaScript example using sovereign vocabulary only — ADR-010]
```

---

<!--
TEMPLATE INSTRUCTIONS (delete this block before committing):

concept_hash:   Leave as 0x00000000 — MetaEngine computes from H1 title via FNV-1a
tribe:          One of: DT-00 DT-01 DT-02 DT-03 DT-04 DT-05 DT-06
                        DT-09 DT-14 DT-20 DT-99 DT-ST DT-LA DT-EX
concept_type:   0x01=Definition 0x02=Decision 0x03=Spec 0x04=Procedure
                0x05=Reference  0x06=Mathematical 0x07=Example
concept_depth:  0–240 (NEVER 255). Low=overview, High=full specification.
riksu_count:    Must equal len(riksu_bindings) — MetaEngine validates.
orbit_tags:     Use names from docs/20_meta_engine/orbit_rings.md
rag_keywords:   HeptaScript sovereign vocabulary only (ADR-010).
                FORBIDDEN: SELECT FROM WHERE JOIN GROUP ORDER FIND ANALYZE WINDOW
-->
