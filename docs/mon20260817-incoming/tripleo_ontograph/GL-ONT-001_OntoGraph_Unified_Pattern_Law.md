# GL-ONT-001 — OntoGraph Law
## The Unified Pattern "Nebuchadnezzar" for the Nasaru Instrument

**Ecosystem:** BahyWay.Ecosystem v4.0
**Domain:** GL (Global Law) — Ontology / Topology / Pattern Minting
**Status:** DRAFT — awaiting Architect seal (CSR-08)
**Author:** DUB.SAR 𒁾
**Related tablets:** GL-ONT-002 (The Non-Substitution Law — the ontological foundation beneath this one: OntoGraph nodes are non-substitutable individuals, not type-instances), GL-TPL-001 (Pattern Minting), GL-TPL-002 (Living Shape & Drift), GL-DDB-001 (EnkiDDB SCG / W5H2), NL-001 (Release Codename Law), KAKI v4.0 canonical byte layout, Way of Work rules 4–5.

**Series note:** The ONT series has two tablets. **GL-ONT-002** is the deeper *foundation* (why data are non-substitutable individuals); **GL-ONT-001** (this tablet) builds on it (how those individuals' patterns are minted into the OntoGraph). Read GL-ONT-002 first for the ontology, this tablet for the instrument.

---

## 1. Purpose

OntoGraph is the innovation name of the analytical instrument inside the Nasaru tool that reads scanned particles from ontological and topological perspectives, discovers their nodes, binds those nodes into a hypergraph, and mints the resulting **Unified Pattern — Nebuchadnezzar** as a template particle that Nasaru visualizes.

OntoGraph is an *instrument*, not an Engine. It carries the innovation name OntoGraph in the same register as Triple-O, HeptaScript and BahyWay. Should OntoGraph ever be promoted to a standalone Engine, it must receive a god name under NL-001; the name OntoGraph remains as the innovation label.

## 2. Definitions

**Particle** — any scanned datum addressed by a KAKI v4.0 sixteen-byte key.

**Ontological perspective** — the reading of a particle through EnkiDDB vocabulary: the W5H2 clause words (WHO / WHAT / WHEN / WHERE / WHY / HOW / HOW MUCH) and their Sowa conceptual relations (AGNT / THME / PTIM / LOC / RSON / MANR / MEAS) as sealed in GL-DDB-001.

**Topological perspective** — the reading of the same particles through LamassuEngine persistent homology: Betti classes β₀ / β₁ / β₂ and their persistence, mapped to the state trichotomy (GOLDEN = loud persistent H₁, FUZZY = short-lived loops, DEAD = diagonal-clustered).

**Formal context** — the FCA incidence relation ⟨G, M, I⟩ where G = particles, M = the union of Mandatory-EAV facets and topological classes, I = incidence.

**Formal concept** — a pair (A, B) with A ⊆ G, B ⊆ M, A′ = B and B′ = A. Every formal concept is a **node** of the OntoGraph.

**Hyperedge** — the extent A of a formal concept. One hyperedge binds all particles of the extent simultaneously; there are no pairwise edges in OntoGraph.

**Unified Pattern (Nebuchadnezzar)** — the concept lattice ⟨𝔅(G,M,I), ≤⟩ together with its hyperedges, minted as a template particle.

## 3. The Three-Layer Attribute Law (sealed)

| Layer | Contents | Schema | Values | Writer |
|---|---|---|---|---|
| KAKI v4.0 | κ[0..3] uuid_hash, κ[4..5] tribe_id, κ[6] kaki_type, κ[7] kaki_role, κ[8..11] reserved, κ[12..13] timestamp, κ[14..15] CRC‑16/CCITT | locked permanently | immutable | KISPU only |
| Mandatory EAV | W5H2 clause words + Sowa relations, state class (GOLDEN/FUZZY/DEAD), ColourID (via PASHIRU), freshness, domain | sealed by law | mutable | sealed engines only |
| Optional EAV | everything OntoGraph discovers; everything an organization brings under DMBOK semantics | open, discoverable | mutable | OntoGraph, stewards, ETL |

**Clause 3.1 — No meaning in KAKI.** OntoGraph never reads meaning from, and never writes to, the KAKI bytes. KAKI is address only.

**Clause 3.2 — Mandatory is spine.** OntoGraph reads Mandatory attributes as the invariant columns of the formal context. Every particle is guaranteed to have them; therefore the lattice always has a stable spine.

**Clause 3.3 — Optional is harvest.** OntoGraph mints only Optional attributes. Discovered outputs are:
`onto.concept_id`, `onto.lattice_rank`, `onto.hyperedge_ids`, `onto.betti_signature`, `onto.extent_size`, `onto.intent_size`, `onto.stability`.
Organizational (DMBOK) attributes are Optional by definition: `dmbok.glossary_term`, `dmbok.steward`, `dmbok.data_domain`, `dmbok.lineage_ref`, `dmbok.classification`, `dmbok.retention`, `dmbok.quality_dimension`.

**Clause 3.4 — No runtime promotion.** OntoGraph may *propose* that a discovered concept is universal across all seven EnkiDB types (EnkiSDB → EnkiODB → EnkiQDB → EnkiDB → EnkiDW → EnkiMDB → EnkiDDB). Promotion of any Optional attribute to Mandatory is a law change: design-time only, at Gate G4, with Z3 composite proof, and Architect seal under CSR-08.

## 4. Pipeline (three rites)

**Rite I — Reading.** For each scanned particle, OntoGraph collects the Mandatory facets from EAV, requests the topological class from LamassuEngine, and assembles one row of the formal context. Nothing is written yet.

**Rite II — Closure.** OntoGraph computes the concept lattice by NextClosure over bitset rows (pure Rust, no dependencies). Each concept becomes a node; each extent becomes a hyperedge. The lattice with its hyperedges is Nebuchadnezzar.

**Rite III — Minting.** Nebuchadnezzar is minted as a template particle under GL-TPL-001: it receives its own KAKI (kaki_type = template), its own Hepta Space coordinates under the Uniqueness Law, and its concepts are written back to each member particle as Optional EAV. The mint is committed by KISPU as a single four-way atomic commit and witnessed in the NĀRU journal.

## 5. Validation ladder

1. Pure-Rust structural checks: lattice is a complete lattice; every extent is closed; hyperedge ids resolve to existing KAKIs; CRC‑16 of every KAKI verifies.
2. NINSUN advisory (never blocks): semantic anomaly on concept labels.
3. barû anomaly / King-plot on lattice-rank distribution and Betti signatures.
4. Z3 composite proof — **only** at Gate G4, only when Nebuchadnezzar is promoted to a sealed tablet. Never at runtime, never in the shipped binary.

## 6. Visualization

Nasaru visualizes Nebuchadnezzar as any other tribe: hyperedges as orbits, concepts as particles at their Hepta Space coordinates, lattice order as radial depth. Rehearsal renders may use Šala HTML prototypes. Production render is DubSar egui/WGPU only (Way of Work rule 4). No HTML dashboards in production (rule 5).

## 7. HeptaScript surface (sovereign, Anti-SQL)

```
ORBIT tribe(<scanned tribe>) PRESENT onto.concept_id, onto.hyperedge_ids
PROVE pattern(Nebuchadnezzar) WITNESS naru
```

No SELECT / FROM / WHERE / JOIN anywhere in OntoGraph.

## 8. Naming clause (NL-001 amendment §6b — Landmark Pattern Clause)

A Unified Pattern that marks an epoch of the ecosystem may bear a king's name. Such a name is **not spent** from the era roster: the pattern and the era that first ships it may share the name, because the pattern *is* the mark of that era. Nebuchadnezzar therefore names the OntoGraph Unified Pattern **and** remains available on the era roster. Orthography Clause §6a applies: unbroken single word, plain Latin, no diacritics.

## 9. Playbooks

- **PB-322** — OntoGraph crate scaffold inside the Nasaru workspace, three rites as modules, sealed attribute contract, unit tests, cargo build/test verification (HOST → dubsar-workstation VDI).
- **PB-323** (reserved) — LamassuEngine ↔ OntoGraph topological-class bridge.
- **PB-324** (reserved) — KISPU mint of Nebuchadnezzar template + NĀRU witness.

## 10. Seal

```
Sealed by: ______________________  (DUB.SAR 𒁾, CSR-08)
Date:      ______________________
AkkadianSeal (Ed25519): ______________________
```
