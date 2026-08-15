# HeptaScript Language Design Specification

> **DubSar Help** | `Languages > HeptaScript` | Language Reference

<!--META
doc_kaki:
  concept_hash:   "0x00000000"
  tribe:          "DT-LA"
  concept_type:   "0x03"
  epoch:          "2026-06-06"
  concept_depth:  235
  riksu_count:    2
  snapshot_epoch: "2026-06-06"

concept:          "HeptaScript Grammar Specification"
summary:          "Full HeptaScript grammar: orbit-based particle algebra language with sovereign vocabulary — three tiers, EBNF, crate ownership."
sovereign_laws:   []

riksu_bindings:
  - target: "adr_010_heptascript_language_design.md"
    concept: "HeptaScript ADR"
    type: "CHILD"
  - target: "ALGEBRA_GLOSSARY.md"
    concept: "mathematical definitions"
    type: "CHILD"

orbit_tags:       ["HeptaScript Sovereign Language", "OOO Mathematical Foundation"]
rag_keywords:     ["PROJECT", "ORBIT", "PROBE", "MINT", "APPEND", "ASSESS", "WATCH", "SEAL", "FORECAST", "TRACE", "DIVERGE", "RANK", "ENTROPY", "DUB", "ME", "RIKSU", "KIŠIB", "ZIKRU", "PARZU"]
-->

**Version:** 1.0
**Date:** 2026-06-06
**ADR:** ADR-010
**Status:** Canonical

---

## Overview

HeptaScript is the sovereign query and transformation language of
BahyWay.Ecosystem v4.0. It is an **orbit-based particle algebra** — not a
query language, not a scripting language, not a DSL over a relational store.

HeptaScript programs describe **sovereign operations** on KAKI-sealed
particles arranged in Tribe → Orbit → Particle triality. The language is
defined by the BahyWay Sovereign Algebra (eight OOO mathematical layers,
ADR-008 + ADR-009) and is compiled by the `heptascript-engine` crate.

---

## Fundamental Principle: Not SQL

HeptaScript shares **zero vocabulary** with SQL, HiveQL, SparkSQL, PRQL,
GraphQL, or any language derived from the Relational Model (Codd, 1970).

**Why this is not a style preference — it is a mathematical necessity:**

SQL's relational algebra operates on sets of tuples:
```
σ_condition(R) → filtered set
π_attributes(R) → projected columns
R₁ ⋈ R₂ → joined set
```

HeptaScript's sovereign algebra operates on orbits of particles:
```
PROBE p WITHIN TRIBE T → sovereign interrogation
PROJECT p AT EPOCH e  → simplicial state materialization
ORBIT O WITHIN TRIBE T → ring traversal with KAKI continuity
```

These operations are **not equivalent** and cannot be expressed in each
other's vocabulary without semantic corruption.

---

## Sovereign Vocabulary — Complete Reference

### Tier 1 — OOO Mathematical Operations

#### `PROJECT`

Materialize the canonical state of a particle at a given epoch, using
Index 7 snapshot sparse B-tree for O(log k) lookup.

```heptascript
PROJECT particle <kaki-literal>
  AT EPOCH <epoch-number>
  YIELD <fields>
```

**Mathematical basis:** Simplicial map over the Journal chain, anchored at
the nearest Index 7 snapshot entry, then delta-applied forward. Without
INDEX 7, this would be O(n) full journal scan.

---

#### `ORBIT`

Iterate over all particles in an orbit ring within a tribe. The orbit ring
is defined by Enlil TOP Algebra's orbit calculus — particles in the same
orbit share structural proximity in the 6D VGCA quality space.

```heptascript
ORBIT [<orbit-name>] WITHIN TRIBE <tribe-name>
  [ASSESS <expr> AS <binding>]
  [THRESHOLD <condition>]
  [GROUPED BY <field>]
  [LAST <n> epochs]
  YIELD <fields>
```

---

#### `PROBE`

Interrogate one or more particles without mutation. Implements the OOO IDU
Probing Rule (ADR-008 §5): a PROBE can read any field but cannot trigger
any state transition.

```heptascript
-- Single particle probe
PROBE particle <kaki-literal>
  [AT EPOCH <epoch>]
  YIELD <fields>

-- Pair probe (orbit-bounded, not Cartesian)
PROBE PAIR (p1, p2) WITHIN TRIBE <tribe-name>
  [ASSESS <expr> AS <binding>]
  [THRESHOLD <condition>]+
  YIELD <fields>
```

**Note:** `PROBE PAIR` is **not** a Cartesian product. It iterates over
orbit-adjacent particle pairs, bounded by the tribe's orbit ring structure.
This is O(|orbit|) — not O(|tribe|²).

---

#### `MINT`

Create a new sovereign particle. Assigns KAKI (ADR-003): uuid_hash κ[0..3],
tribe_id κ[4..5], kaki_type κ[6], kaki_role κ[7], seq_counter κ[8..11],
sovereign_epoch κ[12..13], CRC-16 κ[14..15].

```heptascript
MINT particle INTO TRIBE <tribe-name>
  ME <attribute> = <value>
  [ME <attribute> = <value>]*
  KIŠIB
  YIELD p.kaki
```

`KIŠIB` (cylinder seal) finalizes the KAKI and makes it immutable. A MINT
block without `KIŠIB` is a compile error.

---

#### `APPEND`

Write a superseding Event-Kaki to the particle's Journal. The prior state
is never modified — it is preserved and pointed to by the new Event-Kaki.
This is the only write operation in HeptaScript (INSERT to the Journal —
no UPDATE, no DELETE).

```heptascript
APPEND TO particle <kaki-literal>
  ME <attribute> = <value>
  [ME <attribute> = <value>]*
  YIELD event_kaki
```

---

#### `ASSESS`

Compute or re-compute the VGCA quality score B11 ∈ [0, 240] for a particle.
Uses DELTA_FRAG = 0.35 (sovereign constant, never change).
B11 = round(H(P) × 240) where H(P) = 1/(1 + √Σwᵢ(Pᵢ − Tᵢ)²).
QUALITY_DIVISOR = 240.0 — never 255.

```heptascript
ASSESS particle <kaki-literal>
  YIELD b11, quality_class, vgca_vector
```

As a sub-clause inside ORBIT or PROBE:

```heptascript
ORBIT WITHIN TRIBE persons
  ASSESS quality_score(p) AS b11
  THRESHOLD b11 < 80
  YIELD p.kaki, b11
```

---

#### `WATCH`

Register a continuous orbit surveillance rule. Backed by Markov chain
steady-state monitoring (ADR-009 §3). Triggers an AlertEngine event when
the THRESHOLD condition becomes true.

```heptascript
WATCH ORBIT <orbit-name> WITHIN TRIBE <tribe-name>
  [ASSESS <expr> AS <binding>]
  THRESHOLD <condition>
  PARZU alert_level = <CRITICAL|HIGH|MEDIUM|LOW>
  YIELD p.kaki, <fields>, current_epoch
```

---

#### `SEAL`

Apply a Pauli Exclusion lock to a particle. After SEAL, no APPEND is
permitted (Shamash gate). Used for golden record finalization.

```heptascript
SEAL particle <kaki-literal>
  PARZU reason = "<string>"
  YIELD sealed_at_epoch
```

---

#### `FORECAST`

Predict mean epochs until a particle transitions from one state to another,
using the tribe's Markov transition matrix M (ADR-009 §3).
Mean first-passage time M_ij = 1/πⱼ + Σ_k≠j M_kj · P_ik.

```heptascript
FORECAST particle <kaki-literal>
  FROM state <state-name>
  TO   state <state-name>
  [CALIBRATE tribe_matrix WITHIN TRIBE <tribe-name>]
  YIELD mean_epochs, confidence, transition_path
```

---

#### `TRACE`

Walk the complete Journal of a particle from KIŠIB to latest Event-Kaki.
Reports seq_counter continuity — any gap (missing ordinal) is flagged.
A gap means an event was removed, which violates ADR-006.

```heptascript
TRACE particle <kaki-literal>
  [FROM EPOCH <start>]
  [TO   EPOCH <end>]
  YIELD event_kaki, seq_counter, delta_b11, event_type
  [THRESHOLD gap_detected = TRUE]
```

---

#### `DIVERGE`

Compute the spinor divergence between two particles using Clifford algebra
Cl(7) bivectors. Measures geometric distance in the 7D quality space.
High divergence with identical B11 is a fraud signal (ADR-010 Example 1).

```heptascript
DIVERGE (p1, p2)
  YIELD divergence_scalar, bivector_components
```

As `StoryEngine::spinor_divergence` built-in:

```heptascript
PROBE PAIR (p1, p2) WITHIN TRIBE <tribe>
  ASSESS divergence(p1, p2) AS div
  THRESHOLD div > 0.60
  YIELD p1.kaki, p2.kaki, div
```

---

#### `RANK`

Compute PageRank influence score for particles in a tribe graph.
r = (1-d)/|V| + d·Aᵀr; damping factor d = 0.85 (sovereign constant).
Used for Network Topology Analysis (ADR-009 §1).

```heptascript
RANK ORBIT <orbit-name> WITHIN TRIBE <tribe-name>
  [CALIBRATE damping = 0.85]
  [LAST <n> epochs]
  YIELD p.kaki, rank_score
```

---

#### `ENTROPY`

Measure Shannon entropy H(X) = -Σ p(x) log p(x) over a particle's EAV
attribute space or across an orbit. Detects data homogenization and
distributional anomalies (ADR-009 §2).

```heptascript
ENTROPY ORBIT <orbit-name> WITHIN TRIBE <tribe-name>
  [GROUPED BY attribute_name]
  [LAST <n> epochs]
  YIELD attribute_name, H_shannon, divergence_from_prior
```

---

### Tier 2 — Akkadian Primitives

#### `DUB` (𒁾)

Declare a sovereign governance law (.akk file). Governance rules are
first-class language constructs, not configuration files.

```heptascript
DUB law "<law-name>"
  PARZU <condition>
  [RIKSU <orbit-name>]
  YIELD violation_count
```

---

#### `ME` (𒈨)

Assign sovereign attributes to a particle (inside MINT or APPEND) or to a
tribe (inside a tribe declaration block). From Sumerian *mē* — the divine
attributes that define sovereign being.

```heptascript
ME <attribute_name> = <value>
```

---

#### `RIKSU`

Bind two particles via a sovereign contract. The contract is stored as an
Event-Kaki pair, never as a foreign key column. Binding is auditable through
TRACE.

```heptascript
RIKSU (p1, p2)
  ME contract_type = "<type>"
  ME effective_epoch = <epoch>
  KIŠIB
  YIELD riksu_kaki
```

---

#### `KIŠIB` (𒆠𒅆𒁉)

Apply the cylinder seal — finalize and immutably lock a KAKI or sovereign
contract. Always the last statement in a MINT or RIKSU block.

---

#### `ZIKRU`

Declare a named sovereign constant. Constants declared with ZIKRU are
immutable after KIŠIB and are referenced across the ecosystem.

```heptascript
ZIKRU DELTA_FRAG    = 0.35
ZIKRU GEM_RATE_TARGET = 0.354
ZIKRU QUALITY_DIVISOR = 240.0
KIŠIB
```

---

#### `PARZU`

Define a gate rule — a Pauli Exclusion condition that prevents incompatible
particle states from coexisting. Used in WATCH, SEAL, and DUB blocks.

```heptascript
PARZU <condition>
PARZU alert_level = <CRITICAL|HIGH|MEDIUM|LOW>
PARZU reason = "<string>"
```

---

### Tier 3 — Mathematical English Qualifiers

These are **qualifiers** — sub-clauses that modify a block. They are not
standalone verbs.

| Qualifier | Meaning | Use |
|---|---|---|
| `WITHIN TRIBE <name>` | Scope to tribe | All orbit operations |
| `WITHIN ORBIT <name>` | Scope to orbit | Nested orbit access |
| `AT EPOCH <n>` | Time coordinate | PROJECT, PROBE |
| `LAST <n> epochs` | Tail of epoch sequence | ORBIT, ENTROPY, RANK |
| `GROUPED BY <field>` | Partition output | ORBIT, ENTROPY |
| `THRESHOLD <cond>` | Filter on computed value | PROBE, ORBIT, WATCH |
| `YIELD <fields>` | Emit result | All blocks (required) |
| `PAIR` | Orbit-bounded pair iterator | PROBE PAIR |
| `CALIBRATE` | Set algorithm parameter | FORECAST, RANK |
| `AS <name>` | Bind computed value | ASSESS sub-clause only |

---

## Forbidden Patterns

The following constructs are a **compile error** in HeptaScript:

```
-- FORBIDDEN: SQL relational operations
SELECT, FROM, WHERE, JOIN, GROUP BY, ORDER BY, HAVING
FIND, ANALYZE, OVER, WINDOW, GROUPED BY (with SQL semantics)
MERGE, UPSERT, UPDATE, DELETE, DROP, TRUNCATE

-- FORBIDDEN: OOO violations
STORE CrossTribe <anything>       -- ADR-008 §3
DELETE particle <kaki>            -- ADR-006 §2
UPDATE particle SET <field>=<val> -- ADR-006 §3 (use APPEND instead)
MINT particle WITHOUT KIŠIB       -- ADR-003 (KIŠIB is mandatory)
```

---

## Grammar Sketch (EBNF)

```ebnf
program         ::= statement+
statement       ::= block | constant_decl
block           ::= verb subject scope? qualifier* yield_clause
verb            ::= "PROJECT" | "ORBIT" | "PROBE" | "MINT" | "APPEND"
                  | "ASSESS" | "WATCH" | "SEAL" | "FORECAST" | "TRACE"
                  | "DIVERGE" | "RANK" | "ENTROPY"
                  | "DUB" | "RIKSU" | "KIŠIB" | "ZIKRU" | "PARZU"
subject         ::= particle_ref | pair_ref | orbit_ref | tribe_ref
particle_ref    ::= "particle" kaki_literal
pair_ref        ::= "PAIR" "(" binding "," binding ")"
orbit_ref       ::= "ORBIT" orbit_name?
tribe_ref       ::= "TRIBE" tribe_name
scope           ::= "WITHIN" ("TRIBE" tribe_name | "ORBIT" orbit_name)
qualifier       ::= me_clause | assess_clause | threshold_clause
                  | grouped_clause | epoch_clause | last_clause
                  | calibrate_clause | parzu_clause
me_clause       ::= "ME" attribute_name "=" value
assess_clause   ::= "ASSESS" expr "AS" binding
threshold_clause::= "THRESHOLD" condition
grouped_clause  ::= "GROUPED" "BY" field_name
epoch_clause    ::= "AT" "EPOCH" epoch_number
last_clause     ::= "LAST" integer "epochs"
calibrate_clause::= "CALIBRATE" param_name "=" value
parzu_clause    ::= "PARZU" (condition | param "=" value)
yield_clause    ::= "YIELD" field_list
constant_decl   ::= "ZIKRU" name "=" value "KIŠIB"
kaki_literal    ::= "kaki:" hex_uuid "_" hex16 "_" hex8 "_" hex8
                    "_" hex32 "_" hex16 "_" hex16
```

---

## Crate Ownership

| Component | Crate | Status |
|---|---|---|
| HeptaScript parser | `heptascript-engine` | Not built — depends on ADR-010 |
| PROJECT / snapshot lookup | `enkidb-snapshot` + Index 7 | Partial |
| ORBIT / tribe ring | `enkidb-storage` | Partial |
| MINT / KAKI minting | `kaki-core` | Exists |
| APPEND / Journal write | `enkidb-journal` | Exists |
| ASSESS / VGCA-Δ | `vgca-engine` | Partial |
| DIVERGE / spinor_divergence | `story-engine` + `bahyway-algebra` | Not built |
| RANK / PageRank | `graph-engine` | Not built |
| ENTROPY / Shannon | `vgca-engine` | Partial (byte only) |
| FORECAST / Markov | `ammas-engine` | Not built |
| WATCH / AlertEngine | `alert-engine` | Not built |
| DUB / governance | `parzu-engine` | Not built |

---

## References

- ADR-010: HeptaScript Language Design Decision (canonical)
- ADR-003: KAKI Sovereignty (MINT, KIŠIB, seq_counter)
- ADR-006: No-Delete + Mandatory Partitioning (APPEND supersedes; forbidden DELETE/UPDATE)
- ADR-007: Mandatory Snapshot Scheduler (PROJECT + Index 7)
- ADR-008: OOO Foundation (IDU Probing Rule; PROBE semantics; Forbidden Operations)
- ADR-009: Algebra Layers 6-8 (RANK, ENTROPY, FORECAST, WATCH)
- `ALGEBRA_GLOSSARY.md`: Mathematical definitions for all operations
