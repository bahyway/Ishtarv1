# ADR-008 — Orbits-Oriented Ontology: Mathematical Foundation, KAKI Roles, and the 17 Forbidden Operations

> **DubSar Help** | `ADR > 008` | Architecture Decisions

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-06-05"
  concept_depth:  240
  riksu_count:    4
  snapshot_epoch: "2026-06-06"

concept:          "OOO Foundation"
summary:          "Orbits-Oriented Ontology — 8-layer mathematical foundation, three KAKI roles, and 17 Forbidden Operations."
sovereign_laws:   ["§2.4 — no assessments in KAKI nucleus", "§8.3 — CrossTribe state computed on PROBE only"]

riksu_bindings:
  - target: "enlil_algebra.md"
    concept: "Enlil TOP Algebra Layer 4"
    type: "PEER"
  - target: "top_algebra.md"
    concept: "Tribe-Orbit-Particle triality"
    type: "PEER"
  - target: "adr_009_algebra_additions_and_hardening_evaluation.md"
    concept: "Layers 6-8"
    type: "PEER"
  - target: "adr_010_heptascript_language_design.md"
    concept: "17 Forbidden Operations"
    type: "PEER"

orbit_tags:       ["OOO Mathematical Foundation", "KAKI Sovereignty", "Pauli Exclusion Gates"]
rag_keywords:     ["ORBIT", "PROBE", "OOO", "Orbits-Oriented Ontology", "Forbidden Operations", "KAKI roles", "DGA", "spinors"]
-->

## Status: Accepted

---

## Critical Disambiguation

**Orbits-Oriented Ontology (BahyWay OOO) is not related to and must not be
confused with Graham Harman's academic philosophical school "Object-Oriented
Ontology".** The names share a three-letter abbreviation and nothing else.

BahyWay Orbits-Oriented Ontology is an **original sovereign mathematical
ontology** authored by Bahaa Fadam as the foundational layer of
BahyWay.Ecosystem v4.0. It is grounded in five mathematical layers:

- **Semantic Data Modeling (SDM)** — the structural grammar of how sovereign
  entities relate, compose, and inherit meaning
- **Simplicial Complexity with Jordan Normal Form (JNF)** — the topological
  framework that gives particles, tribes, and orbits their geometric and
  algebraic structure
- **VGCA-Δ (BFV 6D binary delta)** — the geometric cleansing operator that
  drives continuous quality assessment in the sovereign field
- **Enlil Algebra (TOP Algebra)** — the composite algebraic system:
  Tribe Algebra + Orbits Calculus + Particles Algebra
- **Differential Geometric Algebra (DGA)** — the continuous geometric calculus
  that governs orbit trajectories, manifold curvature in quality space, and the
  ModularNaviIndex built from Maryna Viazovska's modular forms

Three further layers were added by ADR-009 (2026-06-05) after an algebra audit:
- **Graph Algebra + Flow Networks (Layer 6)** — PageRank, betweenness
  centrality, SCC detection for AML ring structures and network-level fraud
- **Information Theory (Layer 7)** — Shannon entropy + KL divergence over
  EAV attribute distributions; detects distributional fraud that geometric
  VGCA measures miss
- **Stochastic Processes / Markov Chains (Layer 8)** — quality lane transition
  matrices; transforms AMMAS from a scorer into a predictive physics engine

See ADR-009 for formal W5H2 records, mathematical specifications, and the
complete 13-domain algebraic hardening evaluation of the ecosystem.

These eight mathematical layers, together with the sovereign internal languages
(AAOL, HeptaScript, WAY), constitute the complete OOO stack. Every
architectural decision in ADR-001 through ADR-007 is a consequence of this
stack — not of any external philosophical tradition.

---

## Context

ADR-001 through ADR-007 each describe a specific architectural mechanism.
Taken individually, each decision appears as a pragmatic engineering choice.
Taken together, they form a coherent and internally consistent sovereign
system whose coherence requires explanation.

That explanation is Orbits-Oriented Ontology. OOO is not metadata about the
system — it is the mathematical and semantic ground from which the system
is derived. A developer who understands OOO can derive every correct
architectural decision in novel situations without consulting a rules list.
A developer who does not understand OOO will eventually introduce a violation
that appears locally reasonable but is globally inconsistent with the
sovereign model.

---

## Decision

### Decision 1 — The Four Mathematical Layers of OOO

#### Layer 1: Semantic Data Modeling (SDM)

SDM provides the structural grammar of sovereign entities. In BahyWay OOO,
SDM governs:

- **Entity semantics** — what it means for something to be a sovereign
  particle vs. an attribute vs. a relation
- **Inheritance of meaning** — how a TRIBE particle inherits semantic context
  from its Domain without inheriting identity
- **Composition rules** — how Event-Kakis compose into a projected state via
  the StoryEngine without the composition being the particle itself
- **Schema contracts** — how `bahyway-fabric` enforces that externally-sourced
  data satisfies the semantic contract before entering the sovereign field

SDM is the reason the system has EAV (Entity-Attribute-Value) storage rather
than fixed schema tables: sovereign particles carry heterogeneous semantic
payloads that cannot be predetermined at schema design time. The EAV model
is the SDM consequence of a system where particles evolve their semantic
surface over time.

#### Layer 2: Simplicial Complexity with Jordan Normal Form (JNF)

Simplicial topology provides the geometric framework for how particles,
tribes, and orbits relate. A **simplex** in BahyWay OOO is a minimal
sovereign unit — a particle and its neighborhood of direct relations. A
**simplicial complex** is a tribe: a collection of particles whose simplex
neighborhoods form a topologically consistent whole.

The **Jordan Normal Form** (JNF) is applied at the algebra layer to
characterise the state-transition structure of a particle's Journal:

```
A particle's Journal over time can be represented as a linear operator J
acting on a state space S. JNF decomposes J into:

    J = P · Λ · P⁻¹

where:
    Λ = Jordan blocks (each block = one invariant sub-manifold of particle state)
    P = the basis change that recovers readable EAV attributes from raw events

In practice:
    - Each Jordan block corresponds to one independently-evolving EAV attribute
    - Off-diagonal entries in Λ represent coupling between attributes (rare —
      most EAV attributes are independent Jordan blocks of size 1)
    - Nilpotent blocks in Λ reveal transient state dimensions that decay to
      zero — these map to FUZZY and DEAD lane transitions
```

JNF is the algebraic foundation of the Hepta-Score computation: the 7D
quality vector is the eigenvalue decomposition of the particle's state
transition operator applied to the 7 universal EAV dimensions.

#### Layer 3: VGCA-Δ (BFV 6D Binary Delta — Geometric Cleansing Operator)

VGCA has two complementary forms:

| Form | Name | Dimensions | Domain | Role |
|---|---|---|---|---|
| **VGCA-Σ** | FSV 7D text | 7 floating-point | Hepta-Score space | Continuous quality scoring; DomainCentroid calibration |
| **VGCA-Δ** | BFV 6D binary | 6 binary flags | Delta (change) space | Geometric cleansing of state transitions; event-level quality gate |

VGCA-Δ operates on the **delta** between two consecutive Event-Kakis. Each
of its 6 binary dimensions encodes a geometric property of the transition:

```
VGCA-Δ(event_n, event_{n-1}) → [d₁, d₂, d₃, d₄, d₅, d₆] ∈ {0,1}⁶

where:
    d₁ = identity consistency (KAKI bytes unchanged across transition)
    d₂ = temporal monotonicity (epoch_n > epoch_{n-1})
    d₃ = semantic coherence (EAV schema contract satisfied)
    d₄ = tribe boundary respect (no unauthorized CrossTribe write)
    d₅ = lineage chain integrity (hash_{n} chains from hash_{n-1})
    d₆ = quality monotonicity gate (B11 change within permitted bounds)
```

An event that fails any VGCA-Δ dimension is a **cleansing rejection** — it
is not appended to the Journal. The rejection itself becomes a DEAD-lane
Event-Kaki recording the attempt.

VGCA-Δ is the geometric cleansing operator that makes the Journal a
topologically sound simplicial manifold: every transition is a valid edge
in the simplicial complex, and no invalid edges are ever committed.

#### Layer 4: Enlil Algebra — TOP Algebra Stack

Enlil Algebra is the sovereign composite algebra of BahyWay.Ecosystem v4.0:

```
Enlil Algebra = TOP Algebra
TOP Algebra   = Tribe Algebra ⊕ Orbits Calculus ⊕ Particles Algebra
```

**Tribe Algebra** governs the algebraic properties of tribe relations:
- Tribe membership is a closed set under the tribe's sovereign rules
- Tribe composition (CrossTribe-Kaki creation) is a partial operation —
  not all tribes can compose
- Tribe identity elements: each tribe has a canonical centroid particle
  (the DomainCentroid) that is the algebraic identity for quality scoring

**Orbits Calculus** governs the trajectory of particles through quality space:
- A particle's orbit is its trajectory through VGCA-Σ 7D quality space
  over time, parameterised by sovereign epoch
- Orbit convergence: a particle whose orbit converges to the GEM region
  (B11 ≥ 200) is a healthy sovereign particle
- Orbit divergence: radial drift outward in ColorID space is the algebraic
  signal of quality degradation and early fraud/defect detection
- The Orbits Calculus defines the algebraic operations on these trajectories:
  orbit composition, orbit projection at a given epoch, and orbit distance
  (the geometric basis of the Hepta-Score H(P) formula)

**Particles Algebra** governs the algebraic properties of KAKI particles:
- Particle birth: the INSERT operator creates a new element in the particle
  domain P with a unique KAKI PK
- Event composition: the Journal is a monoid under Event-Kaki append —
  associative, with the birth Event-Kaki as identity
- Projection: `StoryEngine::project` is the evaluation homomorphism from
  the Journal monoid to the current state space S

#### Layer 5: Differential Geometric Algebra (DGA) and ModularNaviIndex

Differential Geometric Algebra extends the Simplicial + JNF framework into
the **continuous** domain. Where JNF characterises discrete state-transition
operators, DGA characterises the continuous trajectories particles follow
through quality space between those transitions.

**DGA in the Orbits Calculus:**

A particle's orbit in 7D VGCA-Σ quality space is a smooth manifold curve
parameterised by sovereign epoch:

```
γ : ℝ₊ → ℝ⁷    (sovereign epoch → quality vector)

The tangent vector γ'(t) is the instantaneous quality drift velocity.
The covariant derivative ∇_γ'(γ') is the quality acceleration — the rate
at which drift is changing. A particle in stable orbit has ∇_γ'(γ') ≈ 0.
A particle approaching the DEAD lane has ∇_γ'(γ') pointing radially outward
in ColorID space — DGA detects this before B11 crosses the lane boundary.
```

**The curvature invariant:**

The Riemannian curvature of the quality manifold is computed from the
DomainCentroid — GEM-lane particles define the curvature of the "healthy"
region. Particles that deviate from this curvature (geodesic deviation) are
the early-warning signal for fraud and defect detection in the AlertEngine:

```
ColorID radial drift     = ‖γ(t) − DomainCentroid‖   (orbit radius)
Geodesic deviation       = ‖∇_γ'(γ') − ∇_centroid‖   (curvature mismatch)

Both measures increasing simultaneously → DGA fraud signal
```

**ModularNaviIndex — Viazovska Modular Forms in DGA:**

The `ModularNaviIndex` in `crates/heptascript` is the sovereign application
of DGA to routing geometry. It is grounded in Maryna Viazovska's proof of
E8 and Leech lattice optimal sphere packing via *magic functions* — modular
forms whose Fourier transforms share vanishing points at the same discrete
radii.

The connection to NaviEngine routing geometry is non-trivial:

| Viazovska Concept | NaviEngine Realisation |
|---|---|
| E8 lattice sphere packing | Heptagram NaviNode layout — 6 outer nodes at `e^(2πik/7)` on the unit circle form a sublattice of ℤ[ζ₇] (the 7th cyclotomic field) |
| Modular forms for Γ₀(7) | The congruence subgroup of level 7 has exactly the symmetry of the heptagram; Eisenstein series E₄ at τ=ζ₇ gives the heptagram's resonance frequency |
| Theta series Fourier coefficients `a(n)` | Edge cost histogram — `a(n)` = number of NaviNode pairs at effective routing cost n |
| Magic function vanishing radii | Cost equidistance — Spoke (500m × 0.80 = 400m) and Rim (400m × 1.00 = 400m) collapse to the same bucket: the heptagram was already calibrated to cost-match all chord types |
| Sphere packing optimality | Delta-function spectrum — the 7-node map produces `a(4) = 24`, a single-bucket uniform topology; maximum resonance everywhere, zero variance |

**`ModularNaviIndex` structure (Stage 1 — `crates/heptascript`):**

```rust
ModularNaviIndex {
    fourier_coeffs: Vec<u32>,   // theta-series histogram from directed edges
    weight: i32,                // modular form weight k (chord type)
    level: u32,                 // congruence subgroup level N=7
    signature: u16,             // CRC-16 over non-zero coefficients
}

// Stage 1 methods:
from_graph(g)          → builds theta-series from all directed edges
resonance_score(cost)  → a(n)/peak — normalised Fourier weight for any cost
spectral_peak_cost()   → the map's natural frequency (most common cost)
is_equivalent(other)   → O(1) routing-equivalence via CRC-16 signature
```

**`E₂FourierWeights` — Stage 2 (Viazovska sacred weight derivation):**

Stage 2 replaces hand-chosen NajafSector weights (0.85, 0.88, 0.90...) with
mathematically derived values. The weight-2 Eisenstein series E₂ is evaluated
at τ = k/7 for k = 0..6, giving 7 weights with provable optimality in the
sphere-packing sense:

```
E₂(τ) = 1 − 24 Σ_{n≥1} σ₁(n) q^n    (q = e^{2πiτ})

For τ = k/7:
    C(0) = 1.62   → Central hub (dominant — all outer sectors cluster near this)
    C(1..6)       → Outer sectors; C(2) and C(5) most elevated by cosine geometry
    Normalized    → weights ∈ [0.80, 1.00] matching HeptaChordType range
```

This makes the 6 outer NaviNode sector weights provably optimal — minimum
cost-density packing of paths through geographic space — derived from the
same mathematics that proved E8 sphere packing.

**DGA governs the connection between these layers:** the `resonance_score`
function is a discrete sampling of the continuous DGA curvature measure.
A NaviMap with high resonance variance has a high curvature quality manifold —
routing is irregular. A NaviMap with delta-function resonance (like the
heptagram) has a flat quality manifold — routing is maximally efficient.

---

### Decision 2 — The Sovereign Internal Language Stack

Three internal languages serve BahyWay.Ecosystem v4.0. They are not
third-party languages — they are sovereign instruments derived from OOO:

| Language | Purpose | Relation to OOO Layer |
|---|---|---|
| **AAOL** (Attribute Algebra Operational Language) | Expresses EAV attribute operations, composition rules, and schema contracts in a formally verifiable syntax | SDM Layer — AAOL is the operational realisation of SDM grammar |
| **HeptaScript** | Domain-specific scripting language for defining the 7 mandatory EAV attribute behaviours, quality scoring policies, and Hepta presets per tribe | VGCA-Σ Layer — HeptaScript governs how Hepta-Score is computed and interpreted |
| **WAY Languages** | The family of sovereign query and routing languages that replace SQL in the BahyWay query surface | TOP Algebra Layer — WAY is the user-facing realisation of Tribe Algebra + Orbits Calculus |

These languages are not optional tooling. They are the surface through which
the OOO stack is exposed to developers, data stewards, and tribe administrators.
An operation that cannot be expressed in AAOL, HeptaScript, or WAY is either
not a valid sovereign operation or requires an extension to the language, not
a bypass of it.

---

### Decision 3 — Three KAKI Physical Types and Three KAKI Logical Roles

#### Physical Types (encoded in `κ[6]` `kaki_type`)

| Byte | Type | Meaning |
|---|---|---|
| `0x01` | **Identity-Kaki** | A sovereign entity — a person, asset, transaction, sensor |
| `0x02` | **Event-Kaki** | A state-change event appended to a particle's Journal |
| `0x03` | **CrossTribe-Kaki** | A relation between particles across tribe boundaries |

#### Logical Roles (encoded in `κ[7]` `kaki_role`)

The three logical roles describe what the KAKI represents in the sovereign
epistemic model, independent of its physical storage type. They map to the
three epistemic positions in SDM:

| Byte | Role | Name | SDM Position | Meaning |
|---|---|---|---|---|
| `0x01` | **KISHIB** | External Document | Witness | Represents a file, certificate, contract, or any external artefact that has sovereign identity but whose content lives outside the system |
| `0x02` | **ZIKRU** | Record | Structured Knowledge | Represents a structured data record — a registry entry, a transaction, a measurement — whose attributes are EAV facts stored inside the system |
| `0x03` | **PARZU** | Logic / Template | Procedural Sovereignty | Represents a rule, template, policy, or algorithm — something that governs how other particles behave |

- **KISHIB** — the particle *witnesses* an external artefact; its sovereignty
  is over the reference, not the content
- **ZIKRU** — the particle *knows* structured facts; its sovereignty is over
  the EAV attribute space
- **PARZU** — the particle *governs* behaviour; its sovereignty is procedural,
  not factual — it makes other particles do things

---

### Decision 4 — The Structural-Facts-Only Rule (§2.4)

**The KAKI bytes (`κ[0..15]`) encode structural identity, never assessments.**

This rule is a direct consequence of the SDM Layer: in SDM, identity
predicates and quality assessments belong to different ontological categories.
Conflating them produces an unstable partition key — a particle whose identity
bytes change as its quality changes cannot be reliably located in a
partitioned store.

| KAKI Byte Range | Content | OOO Layer Grounding |
|---|---|---|
| `κ[0..3]` | `uuid_hash` (D1) | Particles Algebra — birth hash, invariant under Journal growth |
| `κ[4..5]` | `tribe_id` (D2) | Tribe Algebra — tribe membership, assigned at tribe registration |
| `κ[6]` | `kaki_type` | SDM — physical epistemic type, immutable at birth |
| `κ[7]` | `kaki_role` | SDM — logical epistemic role, immutable at birth |
| `κ[8..11]` | reserved | Reserved for future TOP Algebra sovereign dimensions — **conflict flagged 2026-07-07**: ADR-003 (same-day accepted, 2026-06-05) reassigns these bytes to `seq_counter`; this table was never updated to match. `KAKI_v4.0.1_canonical.pdf` (2026-07-05) agrees with this table's "reserved" but doesn't cite ADR-003 either. Three documents, one byte range, unresolved — needs an Architect ruling, not a silent pick. See ADR-011 §"Corrections to prior documents" item 3. |
| `κ[12..13]` | `timestamp` (D6) | Orbits Calculus — birth epoch, the starting point of the particle's orbit |
| `κ[14..15]` | `checksum` (D7) | Structural integrity — CRC-16/CCITT over `κ[0..13]` |

**What is never stored in KAKI bytes (must remain in EAV attributes):**
- VGCA-Σ scores and B11 — these are orbit assessments, not structural identity
- Quality lane (GEM/TRIBE/ACTIVE/FUZZY/DEAD) — derived from B11 at query time
- ColorID 7D quality vector — orbit position, not identity
- CrossTribe effective state (Gold/Orange/Gray) — derived via IDU Probing Rule

---

### Decision 5 — The Granularity Principle: Three Tests for KAKI Eligibility

Not every concept in a domain earns a KAKI particle. The **Granularity
Principle** (§5.4) provides three tests rooted in the SDM and Particles
Algebra layers:

| Test | SDM Grounding | Question | Fail case |
|---|---|---|---|
| **Independent Identity** | Particles Algebra: a particle is an atomic element of P with its own KAKI — sub-elements of P do not earn separate elements | Does this entity have sovereign existence independent of its container? | An invoice line item is not independent → no KAKI |
| **State Evolution** | Orbits Calculus: only entities with orbits in quality space warrant a KAKI | Does this entity's state change in ways that must be tracked? | A static reference code with no orbit → no KAKI; store as EAV attribute |
| **Cross-Tribe Participation** | Tribe Algebra: CrossTribe-Kaki composition requires both participants to have KAKI PKs | Does this entity participate in relations across tribe boundaries? | An entity confined to one tribe's private domain → KAKI warranted only if the first two tests also pass |

If all three tests pass: the entity earns a KAKI Identity-Kaki.
If any test fails: the entity is an EAV attribute or a sub-field of an existing particle.

---

### Decision 6 — The IDU Probing Rule for CrossTribe-Kaki (§8.3)

A CrossTribe-Kaki relates two particles from different tribes. Its effective
state (Gold / Orange / Gray) is **never stored** — it is computed at query
time by the Orbits Calculus from the current projected orbits of both
related particles:

```
CrossTribe effective state — Orbits Calculus derivation:
    orbit_a = StoryEngine::project(kaki_a, at_epoch)   // current orbit position of A
    orbit_b = StoryEngine::project(kaki_b, at_epoch)   // current orbit position of B

    if orbit_a.state == ACTIVE && orbit_b.state == ACTIVE → Gold
    if orbit_a.state == ACTIVE XOR orbit_b.state == ACTIVE → Orange
    if orbit_a.state != ACTIVE && orbit_b.state != ACTIVE → Gray
```

The effective state is a function of two orbit positions at a given epoch.
It is not a property of the CrossTribe-Kaki itself. Storing it would violate
the Structural-Facts-Only Rule and produce a stale cached assessment that
diverges from the actual orbit states of both participants.

---

### Decision 7 — The 17 Forbidden Operations

Each forbidden operation violates one or more layers of the OOO mathematical
stack. The layer column identifies exactly which OOO layer the operation
would damage.

| # | Forbidden Operation | OOO Layer Violated | Specific Rule |
|---|---|---|---|
| 1 | **DELETE** any particle or event | Particles Algebra | The particle domain P is a set with INSERT-only membership — no removal operator is defined |
| 2 | **UPDATE** (SQL sense) any stored Event-Kaki | Particles Algebra | The Journal monoid is append-only — mutation is not a monoid operation |
| 3 | **Modify** any byte in a committed KAKI PK | All layers | KAKI byte immutability — the identity element of Particles Algebra is fixed at birth |
| 4 | **Re-use** a KAKI PK after its particle is deprecated | Particles Algebra | KAKI Reference immutability — each element of P maps uniquely to its history |
| 5 | **Overwrite** rather than append to any Journal | Particles Algebra | Journal Storage Discipline — the monoid identity requires a complete history |
| 6 | **Store** VGCA scores or B11 in KAKI bytes | SDM | Structural-Facts-Only Rule §2.4 — orbit assessments are not identity predicates |
| 7 | **Store** CrossTribe effective state | Orbits Calculus | IDU Probing Rule §8.3 — effective state is a derived orbit function, not stored fact. **Clarified 2026-07-07 (ADR-011):** this governs the derived Gold/Orange/Gray health value only. It does not forbid an Amelu's Orbit (a PARZU-role CrossTribe-Kaki's EAV, per ADR-011) evolving via ordinary Event-Kaki EMIT — that is ordinary Orbit mutation, not a stored effective-state violation. |
| 8 | **Store** ColorID 7D vector in KAKI bytes | SDM + VGCA-Σ | Orbit position is not structural identity |
| 9 | **External sources** writing to the 7 mandatory Hepta EAV attributes | SDM + HeptaScript | Hepta attributes are governed by HeptaScript — external writes bypass the language layer |
| 10 | **Soft-delete** (is_deleted flag or convention) | Particles Algebra | No removal operator exists — soft-delete is a disguised DELETE |
| 11 | **Import** data bypassing `bahyway-fabric` | SDM | All external data must pass SDM schema contract validation — direct writes bypass it |
| 12 | **Use** an external database as storage substrate | All layers | External substrates have their own ontologies incompatible with TOP Algebra |
| 13 | **Use** SHA-256 or bcrypt | Particles Algebra | FNV-1a is the sovereign hash — the basis of KAKI D1 `uuid_hash` |
| 14 | **Set** QUALITY_DIVISOR ≠ 240 | VGCA-Σ | H(P) formula requires B11 ∈ 0..=240 — the 240-ceiling is derived from the 7D quality simplex |
| 15 | **Set** DELTA_FRAG ≠ 0.35 | VGCA-Δ | The sovereign fragmentation constant of the geometric cleansing operator |
| 16 | **Operate** without a registered SnapshotJob | Orbits Calculus | Unbounded Journal growth makes orbit projection O(∞) — violates Orbits Calculus boundedness |
| 17 | **Operate** without declaring partition axes | Tribe Algebra | Unpartitioned instances make Tribe Algebra routing undefined |

---

## W5H2

| W | Answer |
|---|---|
| **Who** | Every developer, architect, data steward, and system administrator working with BahyWay.Ecosystem v4.0 |
| **What** | The sovereign mathematical ontology (OOO) of BahyWay.Ecosystem v4.0: SDM grammar, Simplicial + JNF algebraic structure, VGCA-Δ geometric cleansing, Enlil/TOP Algebra stack, Differential Geometric Algebra + ModularNaviIndex (Viazovska E₂ modular forms), AAOL/HeptaScript/WAY language surface; the three KAKI logical roles (KISHIB/ZIKRU/PARZU); the Structural-Facts-Only Rule; the Granularity Principle; the IDU Probing Rule; the 17 Forbidden Operations |
| **When** | From BahyWay.Ecosystem v4.0 onwards — eternal, never revised. OOO is not a versioned design choice, it is the mathematical and semantic ground of the system |
| **Where** | This ADR is the normative reference. Canonical OOO specification: `docs/ooo_canonical/`. Implementations: `crates/kaki-core` (KAKI byte layout), `crates/enkidb-engine` (forbidden ops enforcement), `crates/bahyway-fabric` (SDM schema contracts), `crates/story-engine` (IDU probing, orbit projection), `crates/enkidb-snapshot` (Index 7), `crates/vgca` (VGCA-Σ and VGCA-Δ operators) |
| **Why** | Without OOO, each ADR appears as an isolated pragmatic decision. With OOO, each ADR is a logical consequence of the mathematical stack. Developers who understand the TOP Algebra stack can derive correct decisions in novel situations. Developers who do not will violate the sovereign model in locally reasonable but globally inconsistent ways |
| **How** | SDM provides entity grammar → Simplicial + JNF provides geometric and algebraic structure → VGCA-Δ provides geometric cleansing of every state transition → Enlil/TOP Algebra provides the composite algebraic operations → DGA provides continuous orbit calculus and the ModularNaviIndex derives routing weights from Viazovska E₂ modular forms → AAOL/HeptaScript/WAY expose the stack to users and developers. Forbidden Operations are operations that lack a valid definition in any layer of this stack |
| **How Much** | 5 OOO mathematical layers · 3 sovereign internal languages · 3 KAKI physical types · 3 KAKI logical roles · 7 mandatory Hepta EAV attributes · 6 VGCA-Δ binary dimensions · 7 NaviNode weights derived from E₂(k/7) · 3 Granularity tests · 17 Forbidden Operations · 0 external data substrates · 0 borrowed philosophical traditions |

---

## Relationship to Previous ADRs

| ADR | OOO Layer That Grounds It |
|---|---|
| **ADR-001** (No External DB) | TOP Algebra — external substrates have incompatible ontologies; their algebra cannot compose with Tribe/Orbits/Particles Algebra |
| **ADR-002** (Naming Discipline) | SDM — naming is part of the semantic model; ambiguous names produce ambiguous ontological categories |
| **ADR-003** (KAKI Sovereignty) | Particles Algebra — the 16-byte KAKI PK is the atomic element of the Particles Algebra domain |
| **ADR-004** (BeeMDM 4-lane) | VGCA-Σ — the four quality lanes are four regions of the 7D Hepta-Score space; lane transitions are orbit movements |
| **ADR-005** (Enterprise Data Fabric) | SDM — all external data must satisfy SDM schema contracts at the Fabric boundary before entering the sovereign field |
| **ADR-006** (No DELETE + Partitioning) | Particles Algebra (no DELETE = no removal operator) + Tribe Algebra (partitions = algebraic routing by tribe/hash/time/state) |
| **ADR-007** (Mandatory Snapshot) | Orbits Calculus + DGA — snapshot boundedness is a requirement of both the discrete calculus (O(delta) projection) and the continuous DGA curvature integral; without snapshots the DGA geodesic computation over the full journal becomes computationally unbounded |

---

## Sovereign Law Statement

> **Orbits-Oriented Ontology is the original mathematical and semantic
> foundation of BahyWay.Ecosystem v4.0. It is not borrowed from any
> external philosophical tradition. It is not a metaphor. It is a
> formally grounded five-layer stack:**
>
> **SDM provides the grammar of sovereign entities. Simplicial Complexity
> with Jordan Normal Form provides their geometric and algebraic structure
> — the Hepta-Score is an eigenvalue decomposition of the particle's
> state-transition operator. VGCA-Δ provides the geometric cleansing gate
> that keeps the sovereign field topologically sound — no invalid edge is
> ever committed to a Journal. Enlil Algebra — Tribe + Orbits + Particles
> — provides the composite algebraic laws that govern every sovereign
> operation. Differential Geometric Algebra provides the continuous orbit
> calculus: geodesic deviation in quality space is the early-warning
> signal for fraud and defect before B11 crosses any lane boundary.**
>
> **The ModularNaviIndex is where DGA meets routing geometry: Maryna
> Viazovska's modular forms — the mathematics that proved E8 optimal
> sphere packing — derive the NaviEngine's sacred sector weights
> mathematically, from the symmetry of the 7th cyclotomic field, not
> from editorial choice. The heptagram's spoke-to-rim cost equidistance
> is not a calibration decision — it is a consequence of the E₂ Eisenstein
> series evaluated at τ = k/7.**
>
> **AAOL, HeptaScript, and the WAY Languages are the surface of this
> five-layer stack — the instruments through which the mathematical
> foundation speaks to developers and data stewards. The 17 Forbidden
> Operations are not arbitrary prohibitions. They are operations that
> have no valid definition in any layer of the OOO stack. The system
> does not prevent them with locks or permissions — they do not exist
> in the grammar of a system whose grammar is mathematics.**

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | ADR-008 Accepted 2026-06-05*
