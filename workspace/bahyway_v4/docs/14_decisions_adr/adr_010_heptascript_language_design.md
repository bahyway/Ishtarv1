# ADR-010 — HeptaScript Language Design: Sovereign Vocabulary, No SQL

> **DubSar Help** | `Decisions > ADR-010` | Architecture Decision Record

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-14"
  concept_type:   "0x02"
  epoch:          "2026-06-06"
  concept_depth:  235
  riksu_count:    3
  snapshot_epoch: "2026-06-06"

concept:          "HeptaScript Language Design"
summary:          "HeptaScript is an orbit-based particle algebra language — all SQL-like keywords are formally dismissed."
sovereign_laws:   []

riksu_bindings:
  - target: "heptascript_design.md"
    concept: "full grammar specification"
    type: "PEER"
  - target: "adr_008_ooo_foundation_kaki_roles_forbidden_operations.md"
    concept: "OOO mathematical operations"
    type: "CHILD"
  - target: "ALGEBRA_GLOSSARY.md"
    concept: "sovereign vocabulary"
    type: "CHILD"

orbit_tags:       ["HeptaScript Sovereign Language", "OOO Mathematical Foundation"]
rag_keywords:     ["PROJECT", "ORBIT", "PROBE", "MINT", "APPEND", "ASSESS", "WATCH", "SEAL", "FORECAST", "TRACE", "DUB", "ME", "RIKSU", "KIŠIB"]
-->

**Status:** Accepted
**Date:** 2026-06-06
**Author:** Bahaa Fadam
**Supersedes:** Any prior HeptaScript example using SQL-like syntax
**Related:** ADR-008 (OOO Foundation), ADR-009 (Algebra Layers 6-8), ADR-003 (KAKI Sovereignty)

---

## CRITICAL CORRECTION

Any prior documentation or example code that used the following keywords in a
HeptaScript context is **INCORRECT** and **SUPERSEDED** by this ADR:

```
INCORRECT (SQL-like — NEVER use in HeptaScript):
  FIND, WHERE, AND, OR, FROM, SELECT, UPDATE, DELETE,
  ORDER BY, GROUP BY, GROUPED BY, ANALYZE, WINDOW,
  OVER, JOIN, INNER JOIN, LEFT JOIN, HAVING, LIMIT,
  OFFSET, UNION, INTERSECT, EXCEPT, AS (SQL alias)
```

Specifically, the following code block from a prior response is **WRONG**:

```
-- WRONG: SQL-like HeptaScript (forbidden)
FIND particle_pairs (p1, p2)
WHERE p1.b11 = p2.b11
  AND spinor_divergence(p1, p2) > 0.60
  AND p1.tribe_id = p2.tribe_id
ORDER BY spinor_divergence(p1, p2) DESC

ANALYZE spinor_divergence(p, tribe_centroid)
OVER tribe = FINANCIAL_TRIBE
GROUPED BY sovereign_epoch
WINDOW last_90_epochs
```

---

## Context — Why HeptaScript Is Not SQL

SQL is the query language of the **Relational Model** (Codd, 1970). Its
mathematical foundation is the **Relational Algebra**:

| SQL Operation | Relational Algebra | What it does |
|---|---|---|
| SELECT … FROM | π (projection) | Pick columns from a set |
| WHERE | σ (selection) | Filter rows from a set |
| JOIN | ⋈ (natural join) | Combine two flat sets |
| GROUP BY | γ (grouping) | Aggregate over partitions of a set |
| ORDER BY | τ (ordering) | Sort a flat set |

SQL operates on **sets of tuples** — flat, unordered, stateless rows.
Entities have no identity beyond their primary key column value.

HeptaScript operates on **orbits of particles** — living, stateful,
KAKI-sealed sovereign entities arranged in Tribe → Orbit → Particle
triality. Its mathematical foundation is the **BahyWay Sovereign Algebra**:

| HeptaScript Operation | Algebraic Foundation | What it does |
|---|---|---|
| PROJECT | Simplicial map + Index 7 snapshot | Materialize particle state at epoch |
| ORBIT | Enlil TOP Algebra orbit calculus | Navigate the orbit ring of a tribe |
| PROBE | OOO IDU Probing Rule (ADR-008 §5) | Interrogate a particle without mutating |
| MINT | KAKI sovereignty (ADR-003) | Create a new particle with sealed identity |
| APPEND | Event-Kaki journal (ADR-006 §3) | Supersede particle state via new event |
| ASSESS | VGCA-Δ Layer 3 (ADR-008 §3) | Compute quality score B11 ∈ [0,240] |
| WATCH | Markov chain steady-state (ADR-009 §3) | Register continuous orbit surveillance |
| SEAL | Pauli Exclusion gates (ADR-008 §6) | Lock a particle against state transitions |
| FORECAST | Markov first-passage time M_ij | Predict state transition probability |
| TRACE | Journal audit + seq_counter gap detection | Walk the full event history of a particle |

These are **orbit-based particle operations** — they have no equivalents in
relational algebra because the concepts themselves do not exist in SQL.

---

## Decision

### Decision 1 — Dismiss All SQL-Like Keywords

HeptaScript **MUST NOT** use any keyword from SQL or any SQL-derived language
(HiveQL, SparkSQL, PRQL, dbt, etc.). The algebraic foundations are
**incompatible**:

- SQL: set × set → set (binary, flat, stateless)
- HeptaScript: Tribe ⊗ Orbit ⊗ Particle → Projection (triadic, stateful, sovereign)

There is no mapping between these algebras. Using SQL keywords in HeptaScript
is not a style decision — it is a **semantic error**.

### Decision 2 — Three Sovereign Vocabulary Sources

HeptaScript vocabulary derives from exactly three sources:

**Source 1 — OOO Mathematical Operations** (from Orbits-Oriented Ontology,
ADR-008):

| Keyword | Mathematical Root | Meaning in HeptaScript |
|---|---|---|
| `PROJECT` | Simplicial projection + Index 7 | Materialize particle state at a given epoch |
| `ORBIT` | Enlil algebra orbit calculus | Iterate over particles in an orbit ring |
| `PROBE` | OOO IDU Probing Rule | Read particle without state mutation |
| `MINT` | KAKI ADR-003 sovereignty | Create particle: assigns KAKI, seq_counter, epoch |
| `APPEND` | Event-Kaki journal INSERT | Write superseding event; old state preserved |
| `ASSESS` | VGCA-Δ 6D binary delta | Compute or re-compute B11 quality score |
| `WATCH` | Markov surveillance (ADR-009 §3) | Register continuous orbit change detection |
| `SEAL` | Pauli Exclusion / Shamash gate | Lock particle; no further APPEND permitted |
| `FORECAST` | Markov M_ij first-passage | Predict mean epochs to target state |
| `TRACE` | Journal + seq_counter walk | Enumerate full event history; detect gaps |
| `DIVERGE` | Spinor divergence Cl(7) bivector | Compute geometric distance between particles |
| `RANK` | PageRank r = (1-d)/|V| + d·Aᵀr | Score particle influence in tribe graph |
| `ENTROPY` | Shannon H(X) = -Σ p log p | Measure EAV attribute disorder |

**Source 2 — Akkadian Primitives** (from Sumerian/Akkadian cuneiform heritage):

| Keyword | Akkadian Root | Meaning in HeptaScript |
|---|---|---|
| `DUB` | 𒁾 *dubbu* — clay tablet | Declare a law file (.akk) or governance rule |
| `ME` | 𒈨 *mē* — divine attributes | Assign sovereign attributes to a tribe |
| `RIKSU` | *riksu* — binding covenant | Bind two particles via sovereign contract |
| `KIŠIB` | 𒆠𒅆𒁉 *kišib* — cylinder seal | Apply KAKI seal; freeze identity permanently |
| `ZIKRU` | *zikru* — spoken name, proclamation | Declare a named sovereign constant |
| `PARZU` | *parzu* — threshold, gate | Define a Pauli Exclusion gate rule |

**Source 3 — Standard Mathematical English** (precision terms with no SQL
connotation):

| Keyword | Mathematical Meaning | Use in HeptaScript |
|---|---|---|
| `THRESHOLD` | ε-ball boundary | Filter on a computed scalar |
| `YIELD` | Generator / iterator output | Emit particles from ORBIT or PROBE block |
| `WITHIN` | Set membership ∈ | Scope a block to a tribe or orbit |
| `PAIR` | Cartesian pair (a, b) | Iterate over particle pairs |
| `EPOCH` | Discrete time index | Reference a sovereign time coordinate |
| `DIVERGENCE` | KL divergence D_KL(P‖Q) | Measure distributional distance |
| `CALIBRATE` | Parameter estimation | Tune Markov or VGCA parameters |
| `GROUPED` | Partition of a set | Partition ORBIT output by a field |
| `LAST` | Tail of sequence | Reference the n most recent epochs |

### Decision 3 — HeptaScript Grammar Primitives

A HeptaScript program is composed of **blocks**, not statements. Every block
has a **subject** (what sovereign object it operates on) and a **verb**
(what sovereign operation it performs):

```
<verb> <subject> [WITHIN <scope>]
  [<qualifier>]*
  YIELD <result>
```

Where:
- `<verb>` ∈ {PROJECT, ORBIT, PROBE, MINT, APPEND, ASSESS, WATCH, SEAL, FORECAST, TRACE, DIVERGE, RANK, ENTROPY, DUB, ME, RIKSU, KIŠIB, ZIKRU, PARZU}
- `<subject>` is a particle binding, tribe reference, orbit reference, or KAKI literal
- `<scope>` is a WITHIN TRIBE or WITHIN ORBIT clause
- `<qualifier>` is ASSESS, THRESHOLD, GROUPED, EPOCH, LAST, or PAIR
- `YIELD` emits results — it is NOT SELECT

### Decision 4 — No Implicit Joins

HeptaScript has no JOIN. Cross-particle relationships are expressed through:
- `RIKSU` — sovereign binding (explicit, stored as Event-Kaki)
- `PROBE PAIR` — orbit-level pair iteration (not a Cartesian product — bounded by tribe)
- `ORBIT … WITHIN TRIBE` — scoped traversal respecting tribe boundaries

CrossTribe state is NEVER stored (ADR-008 §3). CrossTribe comparisons are
performed only in PROBE blocks, computed on the fly, never persisted.

---

## Corrected Examples

### Example 1 — Fraud Sweep (replaces the WRONG SQL-like version above)

```heptascript
-- Batch fraud sweep: identify particle pairs with identical B11 but
-- geometrically divergent spinor position in the same tribe
PROBE PAIR (p1, p2) WITHIN TRIBE financial_accounts
  ASSESS divergence(p1, p2) AS div
  THRESHOLD p1.b11 = p2.b11
  THRESHOLD p1 ≠ p2
  THRESHOLD div > 0.60
  YIELD p1, p2, div
  GROUPED BY sovereign_epoch
```

### Example 2 — Epoch-Level Audit (replaces the WRONG SQL-like version above)

```heptascript
-- Epoch-level spinor divergence audit: measure each particle's geometric
-- distance from the tribe centroid, grouped by epoch
ORBIT WITHIN TRIBE financial_accounts
  ASSESS divergence(p, tribe_centroid) AS div
  GROUPED BY sovereign_epoch
  LAST 90 epochs
  YIELD p.kaki, sovereign_epoch, div
```

### Example 3 — Particle Minting

```heptascript
-- Mint a new sovereign particle in the person tribe
MINT particle INTO TRIBE persons
  ME name       = "Bahaa Fadam"
  ME tribe_role = SOVEREIGN_AUTHOR
  ME birth_date = 1980-01-01
  KIŠIB
  YIELD p.kaki
```

### Example 4 — State Projection at Epoch

```heptascript
-- Project the canonical state of a particle at epoch 1440
PROJECT particle kaki:0x00A1B2C3_0042_01_00_00000001_05A2_F3C1
  AT EPOCH 1440
  YIELD p.state, p.b11, p.quality_class
```

### Example 5 — Orbit Surveillance

```heptascript
-- Watch the compliance orbit of the financial tribe for state degradations
WATCH ORBIT compliance WITHIN TRIBE financial_accounts
  THRESHOLD b11 < 120
  PARZU alert_level = CRITICAL
  YIELD p.kaki, p.b11, current_epoch
```

### Example 6 — Full Journal Trace with Gap Detection

```heptascript
-- Trace the full event history of a particle; detect seq_counter gaps
TRACE particle kaki:0x00A1B2C3_0042_01_00_00000001_05A2_F3C1
  YIELD event_kaki, seq_counter, delta_b11, event_type
  THRESHOLD gap_detected = TRUE
```

### Example 7 — CrossTribe Probe (computed only, never stored)

```heptascript
-- Probe a particle's identity across two tribes (computed on PROBE, not stored)
-- ADR-008 §3: CrossTribe state is never persisted
PROBE PAIR (p_finance, p_hr) WITHIN TRIBE financial_accounts, hr_records
  THRESHOLD p_finance.me[national_id] = p_hr.me[national_id]
  ASSESS divergence(p_finance, p_hr) AS cross_div
  YIELD p_finance.kaki, p_hr.kaki, cross_div
```

### Example 8 — Sovereign Law Declaration

```heptascript
-- Declare a sovereign governance rule using Akkadian primitive DUB
DUB law "no_dead_particle_in_golden_record"
  PARZU quality_class ≠ DEAD
  RIKSU golden_record_orbit
  YIELD violation_count
```

### Example 9 — Markov Forecast

```heptascript
-- Forecast mean epochs until a FUZZY particle reaches DEAD state
FORECAST particle kaki:0x00A1B2C3_0042_01_00_00000001_05A2_F3C1
  FROM state FUZZY
  TO   state DEAD
  CALIBRATE tribe_matrix WITHIN TRIBE financial_accounts
  YIELD mean_epochs, confidence
```

### Example 10 — Information Entropy Audit

```heptascript
-- Measure attribute entropy across a tribe to detect data homogenization
ENTROPY ORBIT data WITHIN TRIBE financial_accounts
  GROUPED BY attribute_name
  LAST 30 epochs
  YIELD attribute_name, H_shannon, divergence_from_prior
```

---

## Consequences

**Positive:**
- HeptaScript programs read as sovereign mathematical statements, not database queries
- No conceptual confusion with SQL's set-based semantics
- Akkadian primitives reinforce the BahyWay philosophical identity
- Grammar naturally prevents the 17 Forbidden Operations (DELETE, UPDATE, soft-delete) — there are no such verbs in the vocabulary

**Negative:**
- Higher learning curve for engineers trained on SQL
- Parser implementation must be written from scratch (no ANTLR SQL grammar to borrow from)

**Mitigation:**
- `docs/09_languages/heptascript_design.md` provides the full specification
- `docs/12_examples/` will contain worked examples for each verb
- DubSar IDE provides sovereign autocomplete using only this vocabulary

---

## References

- ADR-003: KAKI Sovereignty (seq_counter, gap detection)
- ADR-006: No-Delete + Mandatory Partitioning (INSERT supersedes; no UPDATE/DELETE verbs)
- ADR-007: Mandatory Snapshot Scheduler (Index 7; PROJECT uses snapshot B-tree)
- ADR-008: OOO Foundation (IDU Probing Rule; CrossTribe non-storage; Forbidden Operations)
- ADR-009: Algebra Layers 6-8 (PageRank → RANK; Shannon → ENTROPY; Markov → FORECAST/WATCH)
- `docs/09_languages/heptascript_design.md` — full grammar specification
- `ALGEBRA_GLOSSARY.md` — mathematical definitions for all operations above
