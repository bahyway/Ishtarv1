# SPEC-NBU-001 — NabuEngine Orbital Mechanics (companion to GL-NAV-001)
**RECOVERY COPY — assembled 2026-08-05 from the PB-184 session record.
The mechanics, thresholds, wiring, and crate facts below are VERBATIM
from the sealed playbook text; framing lines are reconstructed. Disk
copy rules if it resurfaces.**

**Status:** v1 mechanics, sealed via PB-184; implementation queued behind
PB-160. **Crate:** `nabu-engine` — "documentation knowledge graph as
dynamic orbits — topics as tribes, citations as gravity."
Dependencies: none — pure std. Sovereignty by arithmetic.

---

## The Mechanics (v1, sealed)  *(verbatim)*

    radius(d, T)   = 1 / (1 + cites(d, T))
        — more citations into tribe T pull document d closer to
          its nucleus.

    affinity(d, T) = cites(d, T) / total_cites(d)

    home tribe     = argmax affinity
    bridge         if second-best affinity >= 0.30
        (BRIDGE_THRESHOLD — tunable, sealed default)

    related(d)     = nearest neighbors by |radius difference|
        within the home tribe, plus all bridges touching it.

## Data Model  *(verbatim fields)*

    DocTablet {
        kaki_hex,             // full KAKI identity
        title,
        sector,               // APSU .. ENLIL
        era,                  // king-named release era
        citations[kaki_hex],  // curated at writing time —
                              // Truth Before Beauty is the KG's
                              // data source, never scraped afterward
    }

Bridges carry kaki_type 0x03 (CrossTribe) and are the most valuable
navigation results, not noise (GL-NAV-001 Clause 3).

## Post-Gate Wiring  *(verbatim)*

EnkiDDB feed (real tablets) · Shamash k-means topic discovery ·
Lamassu persistent-homology holes (cited-but-missing tablets =
documentation debt) · NINSUN advisory assignment (Architect ratifies) ·
DubSar Theater cosmos lens (Buzu-encodable) · AcadEngine
related-lectures panel · NUZI lineage records.

## Naming Flag (standing)

NabuEngine — Nabu already patronizes the Nabû Calculus (MardukEngine);
NL-001 permits one god patronizing a calculus AND an engine, but the
adjacency is an Architect decision, not an inheritance. PENDING ruling.

— Recovered for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
