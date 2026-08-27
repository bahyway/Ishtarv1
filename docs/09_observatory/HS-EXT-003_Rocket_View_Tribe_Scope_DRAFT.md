# HS-EXT-003 (candidate) — THE ROCKET VIEW
## HeptaScript Extension · Tribe-Scope Selection & Ring-Granularity Topology
### BahyWay.Ecosystem v4.0 · Phase Two (GL-STD-002 compliant) · Status: SEALED — landed by `crates/rocket-view`, 6/6 tests passing (SCOPE altitude ordering, IN BIGRING selection, NESU's never-merges/quick-merge cases, CONSTELLATION coherence, ASCENT_SHARE/HORIZON read-through)

---

## 0 · Principle

At BIGRING altitude the unit of thought is the ring, not the particle. This
extension adds **no new verbs and no new language** (HS-EXT-001 precedent): the
five sovereign operations — ORBIT / EMIT / PROVE / SYNC / WITNESS — are aimed at
a higher tier of particle. Foundation: in Triple-O, a tribe's sealed read-model
is itself a particle (templates are particles; summaries are particles). The
rocket view was always latent in the ontology; these clauses name the altitude.

**Runtime law preserved.** All topological quantities (Betti numbers,
persistence, isolation) are computed by LamassuEngine on its medium cadence and
written as EAV attributes onto tribe-summary particles. HeptaScript READS
topology; it never computes it at query time. The sub-second law is untouched.

---

## 1 · New Geometric Nouns (clauses, not verbs)

### 1.1 SCOPE — the altitude clause
Declares which particle tier the verbs address. Five altitudes, matching the
camera tiers of the V-2 dolly (query-at-altitude = view-at-altitude — the
language and the court share the same rocket):

    SCOPE PARTICLE      — the ground: individual particles (default; today's behavior)
    SCOPE SUBRING       — composed sub-rings (e.g. warehouses inside Pharmacy)
    SCOPE TRIBE         — tribe-summary particles (the BIGRING as unit)
    SCOPE FEDERATION    — federation read-model particles
    SCOPE UNION         — the Core Sun tier (Golden Ascent read-models)

### 1.2 IN BIGRING — the ring container noun
Binds the orbit to a named composed ring:

    ORBIT TRIBES IN BIGRING "MedicalFederation"

### 1.3 CONSTELLATION — cross-tier selection noun
Selects a set of tribes as one unit for comparative topology (the storytelling
selection: "these rings, together, as one shot"):

    CONSTELLATION (Laboratory, Pharmacy, Testimony)

---

## 2 · New PROVE Functions (read from EAV, Lamassu-computed)

    BETTI(0) , BETTI(1) , BETTI(2)      — Betti numbers at the declared SCOPE
    PERSIST(k)                           — persistence (lifetime) of the k-th feature
    NESU(tribe)                          — isolation measure: the filtration scale at
                                           which the tribe's H₀ component finally merges
                                           with the main structure; ∞-persistent H₀
                                           = never merges = conceptually isolated
    ASCENT_SHARE(tribe)                  — GOLDEN fraction ascended under GL-FED-001
    HORIZON(tribe)                       — decay-vs-rite horizon of the ring as a whole

*nesû (Akk.): to be distant, to withdraw.* Isolation is not distance at one
moment; it is refusal to merge across all scales. A tribe with high NESU is the
striking conceptual isolation the court highlights — the ring that belongs to
the federation by seal but not yet by shape.

---

## 3 · Worked Queries (W5H2-compliant; WHERE/WHEN are clause words, never SQL)

**Q1 — the rocket survey: which rings are drifting away?**

    ORBIT TRIBES IN BIGRING "MedicalFederation"
      SCOPE TRIBE
      WINDOW LAST 30 DAYS
      PRESENT tribe, BETTI(1), NESU(tribe), tau, epsilon
      PROVE NESU(tribe) ABOVE nesu_horizon
      WITNESS isolated_tribes → StoryEngine

**Q2 — the storytelling shot: three rings as one constellation**

    ORBIT CONSTELLATION (Pharmacy, Laboratory, Testimony)
      IN BIGRING "MedicalFederation"
      SCOPE TRIBE
      PRESENT constellation, BETTI(0), PERSIST(0), ASCENT_SHARE
      PROVE BETTI(0) IS 1
      EMIT court_shot → DubSar Theater

  (PROVE BETTI(0) IS 1: the three rings are one connected story — if it fails,
   the constellation itself is fragmented, and *that* is the story.)

**Q3 — Golden Ascent audit at union altitude**

    ORBIT TRIBES IN BIGRING "MedicalFederation"
      SCOPE UNION
      PRESENT tribe, ASCENT_SHARE, HORIZON(tribe)
      PROVE ASCENT_SHARE ABOVE 0.85
      SYNC core_sun_readmodel

**Q4 — descend the recursion (same law, next radius down)**

    ORBIT SUBRINGS IN BIGRING "Pharmacy.GulaFederation"
      SCOPE SUBRING
      PRESENT subring, BETTI(1), NESU(subring), HORIZON(subring)
      PROVE NESU(subring) ABOVE nesu_horizon
      WITNESS drifting_warehouses → EnkiQDB candidates

---

## 4 · Storytelling Binding (the court contract)

Each SCOPE tier binds to one camera altitude of the V-2 dolly; a query's SCOPE
therefore *is* its shot. The Theater renders what the query proved — selected
tribes lit, NESU-isolated rings pulled into visual quarantine at the rim,
constellation members chorded together — and remains stage, never truth
(GL-DST-001): every lit ring cites the EAV attributes and Lamassu run that
justified its lighting.

## 5 · Codex Compliance
- **A-1 zero new mathematics**: composes persistent homology (Lamassu),
  H₀/H₁ Betti alarms, decay-vs-rite horizons, Golden Ascent (GL-FED-001),
  τ/ε honest display. New are only *nouns and read-functions*.
- **A-4 members cited**: GL-ALG-002 topological tier · GL-FED-001 ·
  GL-TPL-002 (summaries as particles) · HS-EXT-001 (nouns/WINDOW/PROVE
  precedent) · HS-EXT-002 (constellation kinship with ORBIT PAIR).
- **No SQL, ever**: SCOPE / IN BIGRING / CONSTELLATION are geometric nouns in
  the HS-EXT lineage; the five verbs are unchanged.

## 6 · Open seals for CSR-08

**Resolved by this seal** (2026-08-27, explicit chat confirmation, CSR-08):
HS-EXT-003 adoption · NESU as the isolation measure's sovereign name.

**Still open, not decided here:** nesu_horizon's default per sector, and
whether CONSTELLATION also enters HS-EXT-002 as a resonance noun -- both are
tuning/scope decisions for other tablets or per-domain configuration, left
to a future decree.

## 7 · Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-27
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work.
```

*Recorded in the reign of Gudea 1.0, Phase Two.*
