# Orbit Rings — Sovereign Concept Attractor Registry

> **DubSar Help** | `MetaEngine > Orbit Rings` | Reference

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-20"
  concept_type:   "0x05"
  epoch:          "2026-06-06"
  concept_depth:  180
  riksu_count:    1
  snapshot_epoch: "2026-06-06"

concept:          "Orbit Rings"
summary:          "Canonical registry of all sovereign concept attractor orbit rings in the BahyWay v4.0 documentation orbital space."
sovereign_laws:   ["§2.4 — no assessments in KAKI nucleus", "§8.3 — CrossTribe state computed on PROBE only"]

riksu_bindings:
  - target: "meta_engine_spec.md"
    concept: "Orbital Visualization Data Model"
    type: "PARENT"

orbit_tags:       ["MetaEngine", "Orbital Visualization", "RAG"]
rag_keywords:     ["ORBIT", "orbit rings", "concept attractors", "orbital space", "visualization"]
-->

---

## What Is an Orbit Ring?

An orbit ring is a **sovereign concept attractor** — a named focal point
in the 7D documentation orbital space. Multiple doc-particles from different
doc-tribes can belong to the same orbit ring. This is what creates the
multi-colored convergent streams visible in the orbital visualization.

In the visualization image: the **pink attractor** and the **yellow attractor**
are two orbit rings — every colored stream (each a different doc-tribe's
particles) flows toward them because they share those concepts.

An orbit ring is defined by a shared sovereign concept, not by directory
structure. A doc-particle may belong to multiple orbit rings (cross-subject).

---

## Canonical Orbit Ring Registry

### Ring 001 — KAKI Sovereignty
**Color:** `#FFD700` (Gold)
**Core concept:** The KAKI 16-byte sovereign identity seal — minted once, immutable.

| Doc-Particle | Tribe | Depth |
|---|---|---|
| `adr_003_kaki_sovereignty.md` | DT-14 | 240 |
| `identity_kaki.md` | DT-02 | 220 |
| `master_glossary.md` | DT-99 | 200 |
| `glossary.md` | DT-00 | 200 |
| `for_engineers.md` | DT-ST | 150 |
| `for_stewards.md` | DT-ST | 160 |
| `for_architects.md` | DT-ST | 140 |
| `README.md` | DT-00 | 180 |

---

### Ring 002 — OOO Mathematical Foundation
**Color:** `#FF69B4` (Pink)
**Core concept:** Orbits-Oriented Ontology — 8-layer sovereign mathematical stack.

| Doc-Particle | Tribe | Depth |
|---|---|---|
| `adr_008_ooo_foundation_kaki_roles_forbidden_operations.md` | DT-14 | 240 |
| `enlil_algebra.md` | DT-01 | 230 |
| `top_algebra.md` | DT-01 | 230 |
| `adr_009_algebra_additions_and_hardening_evaluation.md` | DT-14 | 220 |
| `ALGEBRA_GLOSSARY.md` | DT-00 | 210 |

---

### Ring 003 — Sovereign Storage (No DELETE)
**Color:** `#4488FF` (Blue)
**Core concept:** EnkiDB Journal — INSERT supersedes, DELETE does not exist.

| Doc-Particle | Tribe | Depth |
|---|---|---|
| `adr_006_no_delete_mandatory_partitioning.md` | DT-14 | 240 |
| `adr_007_mandatory_snapshot_scheduler.md` | DT-14 | 230 |
| `enkidb.md` | DT-05 | 210 |

---

### Ring 004 — HeptaScript Sovereign Language
**Color:** `#FF44AA` (Magenta)
**Core concept:** Orbit-based particle algebra query language — no SQL.

| Doc-Particle | Tribe | Depth |
|---|---|---|
| `adr_010_heptascript_language_design.md` | DT-14 | 240 |
| `heptascript_design.md` | DT-LA | 235 |
| `ALGEBRA_GLOSSARY.md` | DT-00 | 120 |

---

### Ring 005 — VGCA Quality
**Color:** `#00FFCC` (Teal)
**Core concept:** 6D VGCA binary delta quality scoring — B11 ∈ [0,240].

| Doc-Particle | Tribe | Depth |
|---|---|---|
| `adr_009_algebra_additions_and_hardening_evaluation.md` | DT-14 | 180 |
| `ALGEBRA_GLOSSARY.md` | DT-00 | 150 |
| `master_glossary.md` | DT-99 | 140 |

---

### Ring 006 — Pauli Exclusion Gates
**Color:** `#FF3300` (Red)
**Core concept:** Four sovereign gates — Shamash, Marduk, Enlil, Nanna.

| Doc-Particle | Tribe | Depth |
|---|---|---|
| `high_council.md` | DT-04 | 230 |
| `pauli_exclusion.md` | DT-04 | 220 |
| `shamash_gate.md` | DT-04 | 200 |
| `marduk_gate.md` | DT-04 | 200 |
| `adr_008_ooo_foundation_kaki_roles_forbidden_operations.md` | DT-14 | 160 |

---

### Ring 007 — MetaEngine & RAG
**Color:** `#CC44FF` (Magenta-Purple)
**Core concept:** Sovereign documentation control and 7D orbital RAG indexing.

| Doc-Particle | Tribe | Depth |
|---|---|---|
| `meta_engine_spec.md` | DT-20 | 240 |
| `orbit_rings.md` | DT-20 | 180 |
| `orbital_visualization.md` | DT-09 | 200 |

---

## Orbit Ring Assignment Rules

1. A doc-particle joins a ring by adding the ring name to `orbit_tags` in its META block.
2. This registry is the **canonical list** — undefined ring names are rejected by MetaEngine.
3. A doc-particle may belong to a maximum of **4 orbit rings** (prevents concept diffusion).
4. Ring membership determines which orbital attractor a doc-particle flows toward in the visualization — like the colored streams converging on the pink and yellow focal points.
5. The **two highest-riksu_count rings** at any epoch become the primary visualization attractors (the two bright focal points in the image).

---

## Adding a New Orbit Ring

Submit an APPEND to this file (not a direct edit — ADR-006). A new ring requires:
- A unique Ring number
- A hex color code not already in use
- At least 3 doc-particles assigned at creation
- A sovereign concept definition in `docs/00_codex/glossary.md`
