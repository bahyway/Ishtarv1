# Master Glossary

> **DubSar Help** | `Index > Glossary` | Index

This file is the machine-readable index of all terms defined across the Codex.
It is generated from `00_codex/glossary.md` and supplemented by terms defined
in per-section files.

See `00_codex/glossary.md` for the authoritative term definitions.

---

## KAKI — Sovereign Declaration (Canonical)

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

**Canonical byte layout:** `ADR-003` | **OOO grounding:** `ADR-008` Layer 1 (SDM)
