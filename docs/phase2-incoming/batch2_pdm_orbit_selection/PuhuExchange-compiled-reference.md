# PuhuExchange — Compiled Reference (Recovery Edition)
**BahyWay.Ecosystem v4.0 · the entity-identity operator and its law**
Assembled 2026-08-05. No standalone PuhuExchange document previously
existed — the node was born in the PDM Shape Operator spec (this era);
its law and mathematics come from two earlier seals. Sources: [A] PB-178
/ PH-002 tablet (2026-07-26, verbatim); [B] the BahyWay textbook,
Chapter 15 "TOP Algebra and the Puhu Law" (2026-08-03, verbatim
formalism); [C] pdm-shape-operator-graph-spec.md §3 + GL-MDM-001
Clause 7 + the PDM v2–v4 rehearsals (this era). Disk originals rule
if they resurface.

---

## 1. The Ritual Root  [A, verbatim]

The Akkadian rite of the **šar pūḫi** — the substitute king. When omens
threatened the throne, a substitute was enthroned in the king's place;
the *person* was exchanged, and kingship — the pattern — persisted
undisturbed. *Pūḫu*: exchange, substitution. The Puhu Law (plain Latin
per NL-001): **exchanging the nucleus does not destroy the pattern,
for the Tribe is the throne and not the king.**

## 2. The Law — PH-002  [A, sealed by PB-178]

Layer: TOP Algebra (Tribes Algebra + Orbits Calculus + Particles
Algebra). The Tribe is the LOGICAL nucleus of the Orbits Particles
Pattern; exchanging the concrete nucleus does not destroy the pattern.
Status: sealed as a docs tablet via PB-178 on eriduous-vdi at
`~/bahyway/v4/docs/PH-002.md`; running PB-178 was the CSR-08
ratification. Kinship note: at the Enlil layer, Jordan Normal Form is
the canonical instance — similarity transforms exchange the basis, the
eigenvalue pattern survives. JNF is to matrices what the Tribe is to
particles.

## 3. The Formal Statement  [B, verbatim]

For an orbit O with nucleus n and observable Φ:

    n ≡ n′  ⟹  Φ(O[n]) = Φ(O[n′])          (PH-002)

**Theorem (invariance composes):** if Φ is Puhu-invariant and g is any
function of observables, then g∘Φ is Puhu-invariant. Consequence:
prove invariance once at the primitive observables (census, radius,
period), and every derived metric inherits it for free — invariance,
like integrity, is checked at the bottom and trusted at the top.

**Operational reading:** this is the mathematical guarantee behind
refactoring, migration, and failover — EnkiDB nodes moving from VMs to
Podman quadlets, a Template reissued, a write node yielding to its
passive twin — with every orbit-level truth (census, period,
stability, τ) preserved. A migration playbook's PROVE stage is
literally a Puhu check.

## 4. The Node — PuhuExchange  [C]

The Shape Operator that applies PH-002 to MDM entity identity:
"four costumes, one customer."

**Contract:**
- Ports: (ShapeFlow A, ShapeFlow B) → ShapeFlow. Wire class:
  cross-tribe operator, gold.
- Cook REQUIRES the two-witness verdict above threshold, else the
  node REFUSES (turns crimson; NO output; nothing downstream cooks):
    (a) STRUCTURAL — Jaccard over role:type shape shingles
        (MinHash at scale);
    (b) SEMANTIC — embedding/synonym proximity of field names
        (NabuEngine candle; NL↔EN bridge: klant↔customer,
        naam↔name, stad↔city).
  Both high → same-entity cluster: shapes merge, the cooked shape
  carries `puhu_identity: true` into the Shape Tablet.
  Disagree or both low → REFUSED with witnesses logged.
- Thresholds are EAV attributes (rehearsal default 0.5/0.5),
  tunable per domain, never hard-coded.
- Purity: like every operator except Seal, PuhuExchange is
  side-effect-free and deterministic.

**The design sentence worth keeping:** *the editor physically cannot
draw an entity-identity claim the mathematics has not earned* — Puhu
refusal is MDM discipline enforced at the UI layer.

**Rehearsal record (PDM v2–v4):** Klant↔Customer passes (structure
agrees across languages via the semantic witness; J and sem shown in
the node header); Customer↔SensorLog is refused; the refusal is a
teaching moment, not an error path.

## 5. Standing Elevation — GL-MDM-001 Clause 7  [C, PENDING]

Proposed, NOT yet sealed: PH-002 as the formal ground of same-entity
identity under exchanged representation — harmonization as a THEOREM
with an enforcement pipeline rather than a heuristic. Under this
reading, the two witnesses are the practical test of n ≡ n′ for
representational nuclei, and canonical minting (GL-TPL-001) is the
enthronement of the invariant pattern. **The Architect rules whether
Puhu bears this weight.** Until sealed, PuhuExchange operates lawfully
as an operator whose refusal semantics stand on the two-witness
verdict alone.

## 6. Where It Lives

PH-002 tablet: PB-178 → eriduous-vdi docs (re-run PB-178 to restore).
Node spec: pdm-shape-operator-graph-spec.md §3. Working rehearsal:
shala_tab_pdm_modeler_v4.html (Puhu ✓ / Puhu ✗ sample graphs).
Manual: pdm-manual.md Fig. 2 + Appendix C. Production home when the
gate opens: pdm-graph-core (pure Rust), witnesses served by
CompareEngine + NabuEngine over HEPT.

— Compiled for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
