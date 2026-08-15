#!/usr/bin/env bash
# 02_forge_codex.sh — v2.0
# Creates the documentation folder structure for bahyway_v4 and seeds
# DubSar IDE Help page placeholders in every subdirectory.
#
# Run from: workspace/bahyway_v4/
# Safe to run multiple times — does not overwrite existing files.
#
# Placeholder format:
#   Each .md file follows the DubSar IDE Help page skeleton:
#     # Title
#     > DubSar Help | `SyntaxToken` | Category
#     ## Purpose / ## Mechanism / ## Sovereign Constraints / ## See Also

set -euo pipefail

if [ ! -f Cargo.toml ]; then
    echo "Error: Run this script from the workspace/bahyway_v4/ directory (where Cargo.toml lives)."
    exit 1
fi

echo "Forging the codex folder structure..."

# ─── Directory tree ────────────────────────────────────────────────────────────
mkdir -p docs/_start_here
mkdir -p docs/_meta/01_templates
mkdir -p docs/_diagrams
mkdir -p docs/00_codex
mkdir -p docs/01_mathematics
mkdir -p docs/02_identity
mkdir -p docs/03_kernel_mummu
mkdir -p docs/04_gates
mkdir -p docs/05_storage
mkdir -p docs/06_governance_parzu
mkdir -p docs/07_file_formats
mkdir -p docs/08_pipeline_alaktu
mkdir -p docs/09_observatory
mkdir -p docs/10_operations
mkdir -p docs/11_tooling
mkdir -p docs/12_examples
mkdir -p docs/13_changelog
mkdir -p docs/14_decisions_adr
mkdir -p docs/15_howto
mkdir -p docs/16_runbooks
mkdir -p docs/17_troubleshooting
mkdir -p docs/18_security
mkdir -p docs/19_roadmap
mkdir -p docs/99_index

# Helper: write a file only if it does not already exist
new_file() {
    local path="$1"
    if [ ! -f "$path" ]; then
        cat > "$path"
        echo "  created $path"
    fi
}

# ─── docs/README.md ────────────────────────────────────────────────────────────
new_file docs/README.md <<'EOF'
# bahyway_v4 — Codex Documentation

Organized by concern. Empty folders hold a `.gitkeep`; replace it with real
content as documents are written.

## Onboarding

- **_start_here/** — role-based reading paths.
- **_meta/** — style guide and templates.
- **_diagrams/** — shared visual assets.

## Specification Tree

| Folder | Contents |
| :--- | :--- |
| 00_codex/ | Motto, principles, axioms, glossary. |
| 01_mathematics/ | Enlil Algebra, TOP Algebra, PA-1 to PA-16. |
| 02_identity/ | KAKI Triad: Identity, Events, CrossTribe. |
| 03_kernel_mummu/ | Lean 4 + Z3 verification engine. |
| 04_gates/ | ADAD, ANU, MARDUK, SHAMASH — Pauli Exclusion governance. |
| 05_storage/ | EnkiDB hot plane, EnkiDW, EnkiStream. |
| 06_governance_parzu/ | PARZU laws and template registry. |
| 07_file_formats/ | .akk, .hepta, .way, .tmpl grammars. |
| 08_pipeline_alaktu/ | CI/CD: submission, verification, release. |
| 09_observatory/ | Orbital visualization, PA-13 to PA-16. |
| 10_operations/ | Deployment topology, installation, HA/DR. |
| 11_tooling/ | Nabu CLI and adjacent tools. |
| 12_examples/ | End-to-end worked scenarios. |
| 13_changelog/ | Version-to-version differences. |
| 14_decisions_adr/ | Architecture Decision Records. |
| 15_howto/ | Task-oriented guides. |
| 16_runbooks/ | On-call procedures. |
| 17_troubleshooting/ | Symptom → cause → fix. |
| 18_security/ | Threat model, secrets management, audit trail. |
| 19_roadmap/ | Versioning policy, planned features, deprecations. |
| 99_index/ | Cross-cutting indices and master glossary. |
EOF

# ─── _start_here ───────────────────────────────────────────────────────────────
new_file docs/_start_here/for_architects.md <<'EOF'
# Start Here — System Architects

> **DubSar Help** | `Help > Architects` | Onboarding

## Reading Path

1. `00_codex/motto.md` — the sovereign axiom.
2. `01_mathematics/enlil_algebra.md` — the algebraic foundation.
3. `01_mathematics/top_algebra.md` — Tribe-Orbit-Particle triality.
4. `04_gates/high_council.md` — the four Pauli Exclusion gates.
5. `05_storage/enkidb.md` — storage architecture.
6. `09_observatory/orbital_visualization.md` — the visual model.

## Purpose

Architects use this tree to understand the mathematical invariants that
constrain every implementation decision in BahyWay.

## Sovereign Constraints

§2.4: No assessments in KAKI nucleus.
§8.3: CrossTribe state computed on PROBE, never stored.
EOF

new_file docs/_start_here/for_stewards.md <<'EOF'
# Start Here — Data Stewards

> **DubSar Help** | `Help > Stewards` | Onboarding

## Reading Path

1. `04_gates/shamash_gate.md` — Dead vs. Active state judgment.
2. `04_gates/marduk_gate.md` — transformation lock and stewardship gap.
3. `04_gates/pauli_exclusion.md` — why two conflicting truths cannot coexist.
4. `06_governance_parzu/parzu_laws.md` — the law file (.akk) lifecycle.
5. `09_observatory/orbital_visualization.md` — reading the orbital view.

## Purpose

Data Stewards use this tree to understand when and how to intervene in
particle state transitions: correcting Dead particles, resolving Stewardship
Gaps, and auditing Golden Record promotions.

## Sovereign Constraints

§8.3: CrossTribe state computed on PROBE, never stored.
EOF

new_file docs/_start_here/for_engineers.md <<'EOF'
# Start Here — Engineers

> **DubSar Help** | `Help > Engineers` | Onboarding

## Reading Path

1. `02_identity/kaki_triad.md` — the three KAKI types.
2. `01_mathematics/tri_kaki_index.md` — O(1) indexing strategy.
3. `05_storage/enkidb.md` — storage crate interface.
4. `07_file_formats/akk_format.md` — the .akk law file grammar.
5. `11_tooling/nabu_cli.md` — the CLI reference.
6. `08_pipeline_alaktu/submission.md` — the CI/CD pipeline.

## Purpose

Engineers use this tree to locate API contracts, file format grammars,
and crate dependency maps before writing code.

## Sovereign Constraints

Pure Rust only. No third-party databases. No ORM.
EOF

# ─── _meta ─────────────────────────────────────────────────────────────────────
new_file docs/_meta/01_templates/help_page_template.md <<'EOF'
# [Title]

> **DubSar Help** | `` `[SyntaxToken]` `` | [Category]

## Purpose

[One sentence. What does the user see when they hover over this syntax, button,
or CLI flag?]

## Mechanism

- [How it works, step 1.]
- [How it works, step 2.]
- [Edge cases or constraints.]

## Sovereign Constraints

[§ references from Particles Algebra or GhishKhur Algebra.]

## See Also

- `[related_page.md]`
EOF

new_file docs/_meta/style_guide.md <<'EOF'
# Documentation Style Guide

> **DubSar Help** | `Help > Style` | Meta

## Rules

- Every file begins with `# Title` then the `> DubSar Help` blockquote.
- Mechanism sections use bullet points, not paragraphs.
- § references must point to exact sections in the algebra specifications.
- No marketing language. No placeholder TODO paragraphs — leave the file empty
  instead and rely on the `.gitkeep` convention.
- Akkadian gate names (ADAD, ANU, MARDUK, SHAMASH) are written in ALL CAPS.
- Rust types use backtick monospace: `PauliState`, `GhishKhurGate`.
EOF

# ─── _diagrams ─────────────────────────────────────────────────────────────────
new_file docs/_diagrams/orbital_layers_diagram.md <<'EOF'
# Orbital Layers Diagram Reference

> **DubSar Help** | `Observatory > Orbital Layers` | Diagrams

## What This Diagram Shows

The orbital layer diagram visualises Theorems PA-13 to PA-16:

- **Shells** (PA-13): logarithmically spaced rings, δ_T(p) = 1 − HPS(p).
- **Particle position** (PA-14): radius = δ · R_max, azimuth from KAKI byte 12,
  altitude from KAKI byte 14.
- **Tribe birthing** (PA-15): the outer rim spawning a child orbital system.
- **Rendering modes** (PA-16): PointSprite / Instanced / Volumetric by count.

## Asset Location

Place rendered `.svg` or `.png` files in this directory with the naming pattern:
`pa-{theorem_number}_{short_name}.{ext}` (e.g. `pa-14_particle_position.svg`).

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/particle_position.md`
EOF

new_file docs/_diagrams/high_council_diagram.md <<'EOF'
# High Council — Four Gates Diagram Reference

> **DubSar Help** | `Gates > High Council Diagram` | Diagrams

## What This Diagram Shows

The data-flow through the four Pauli Exclusion gates in order:

```
Signal ──► ADAD (Temporal) ──► ANU (Authority) ──► MARDUK (Transform) ──► SHAMASH (State)
                                                                                     │
                                                                              Active Orbit
```

## See Also

- `04_gates/high_council.md`
- `04_gates/adad_gate.md`
- `04_gates/anu_gate.md`
- `04_gates/marduk_gate.md`
- `04_gates/shamash_gate.md`
EOF

# ─── 00_codex ──────────────────────────────────────────────────────────────────
new_file docs/00_codex/motto.md <<'EOF'
# The Sovereign Motto

> **DubSar Help** | `Codex > Motto` | Non-Negotiables

## The Motto

> "Built on Ancient Wisdom. Engineered for Infinite Tomorrows."

## The Axiom

Data in BahyWay is not stored — it is governed. Every particle has an identity
(KAKI), a trajectory (Orbit), and a community (Tribe). These three are not
metaphors; they are mathematical objects derived from Enlil Algebra.

## Sovereign Constraints

No particle exists without a KAKI. No KAKI is assessed inside the nucleus (§2.4).
EOF

new_file docs/00_codex/principles.md <<'EOF'
# Core Design Principles

> **DubSar Help** | `Codex > Principles` | Non-Negotiables

## The Seven Principles

1. **Sovereignty** — Every particle is self-describing via its KAKI.
2. **No External Databases** — Pure Rust. EnkiDB is the only persistence layer.
3. **Identity Immutability** — An Identity-KAKI, once minted, never changes.
4. **Pauli Governance** — No two conflicting truths occupy the same state space.
5. **Determinism** — Given KAKI + Tribe Ideal, every output is reproducible.
6. **Auditable Trajectory** — Every state transition is KAKI-stamped in StoryWay.
7. **Scale by Geometry** — Scaling adds dimensions (Tribes), not infrastructure.
EOF

new_file docs/00_codex/axioms.md <<'EOF'
# Mathematical Axioms

> **DubSar Help** | `Codex > Axioms` | Non-Negotiables

## Axiom 1 — Particle Identity

For any particle p, there exists a unique KAKI κ(p) ∈ {0..255}^16 such that
κ(p) is computable in O(1) and is stable across all storage locations.

## Axiom 2 — Tribe Ideal

For any tribe T, there exists a Tribe Ideal vector ι_T ∈ [0,1]^7 that defines
the target state for all particles in T.

## Axiom 3 — Quality Distance

δ_T(p) = 1 − HPS(p), where HPS is the Hepta Priority Score. δ_T(p) = 0 means
perfect alignment; δ_T(p) = 1 means maximum divergence.

## Axiom 4 — Pauli Exclusion

No two particles may occupy the same (Identity-KAKI, Orbit-Level, QuantumState)
triplet simultaneously within a Tribe.

## See Also

- `01_mathematics/enlil_algebra.md`
- `04_gates/pauli_exclusion.md`
EOF

new_file docs/00_codex/glossary.md <<'EOF'
# Glossary

> **DubSar Help** | `Codex > Glossary` | Non-Negotiables

## Terms

| Term | Definition |
| :--- | :--- |
| **KAKI** | 16-byte particle DNA: 8-byte identity prefix + 8-byte attribute suffix. |
| **HPS** | Hepta Priority Score — a scalar in [0,1] derived from the 7D Hepta vector. |
| **δ_T(p)** | Quality distance: 1 − HPS(p). Measures divergence from Tribe Ideal. |
| **Orbit** | The Jordan Chain of historical state transitions for one KAKI. |
| **Tribe** | A mathematically isolated manifold (Jordan Block) of related particles. |
| **Golden Record** | A particle with δ_T(p) → 0; the idealized ground truth. |
| **Dead Particle** | A particle with δ_T(p) → 1; archived, no active motion. |
| **Stewardship Gap** | Transition state between Dead and Active; requires Data Steward intervention. |
| **ADAD** | Gate 1 — Temporal Exclusion (signal de-duplication). |
| **ANU** | Gate 2 — Authority Exclusion (source hierarchy). |
| **MARDUK** | Gate 3 — Structural Exclusion (transformation lock). |
| **SHAMASH** | Gate 4 — State Exclusion (Dead vs. Active judgment). |
| **PA-n** | Theorem n in the Particles Algebra specification. |
| **TOP Algebra** | Ṭupšarrūtu Algebra — maps Tribe-Orbit-Particle to Octonion structure. |
| **Enlil Algebra** | The Jordan Normal Form algebra governing EnkiDB indexing and sharding. |
EOF

# ─── 01_mathematics ────────────────────────────────────────────────────────────
new_file docs/01_mathematics/enlil_algebra.md <<'EOF'
# Enlil Algebra

> **DubSar Help** | `Math > Enlil` | Mathematics

## Purpose

Enlil Algebra is the Jordan Normal Form algebra that governs how EnkiDB stores,
shards, and indexes particles. It treats the entire data universe as a
block-diagonal matrix where each Tribe is an independent Jordan Block.

## Mechanism

- **Block-diagonal structure**: Tribe A on Node 1 has zero mathematical
  cross-talk with Tribe B on Node 999,000. Infinite horizontal scaling.
- **Jordan Chain**: each Orbit is a nilpotent chain; old events fall off
  naturally (no separate cleanup job).
- **Spectral Renormalization**: when Hubble-Zooming, compute the Trace and
  Spectral Radius of macro-clusters to render "Galaxies" before unpacking
  individual Jordan Blocks.
- **O(1) indexing**: all three KAKI types map directly to addresses without
  tree traversal (see `tri_kaki_index.md`).

## Sovereign Constraints

§8.3: CrossTribe state computed on PROBE, never stored.
No B-Tree indexes. No full-table scans.

## See Also

- `01_mathematics/tri_kaki_index.md`
- `02_identity/kaki_triad.md`
- `05_storage/enkidb.md`
EOF

new_file docs/01_mathematics/top_algebra.md <<'EOF'
# TOP Algebra — Ṭupšarrūtu

> **DubSar Help** | `Math > TOP` | Mathematics

## Purpose

TOP (Ṭupšarrūtu) Algebra maps the Tribe-Orbit-Particle triality to the
Octonion algebraic structure, establishing BahyWay as the first
Octonion-based database algebra in history.

## The Isomorphism

| Physics Triple-O | TOP Algebra | Common Ground |
| :--- | :--- | :--- |
| Octonions (8D) | KAKI (real) + 7D Heptagon (imaginary) | 8 dimensions to define a real object. |
| Triality (symmetry) | Tribe-Orbit-Particle | Each part of the triad is also a particle. |
| Jordan Algebra (observables) | Stakeholder Observation | State materialises only when observed. |

## Mechanism

- **Non-associative logic**: A observed by B ≠ B observed by A. The 7D
  Heptagon captures asymmetric truth.
- **GPU resonance query**: `dot(p.hepta, J * p.hepta)` applies the Jordan
  Matrix in one Vulkan compute shader pass — no sequential scan.
- **Manifold materialization**: data in ABZU is "withdrawn"; a query
  "collapses" the field into a result set.

## Sovereign Constraints

Pure Rust + Vulkan compute. No BLAS/LAPACK third-party libraries.

## See Also

- `01_mathematics/enlil_algebra.md`
- `01_mathematics/particles_algebra.md`
- `04_gates/high_council.md`
EOF

new_file docs/01_mathematics/particles_algebra.md <<'EOF'
# Particles Algebra

> **DubSar Help** | `Math > Particles Algebra` | Mathematics

## Purpose

Particles Algebra is the canonical specification for all particle operations in
BahyWay. Theorems PA-1 through PA-16 define identity, scoring, aggregation,
shell decomposition, rendering, and tribe birthing.

## Key Theorems

| Theorem | Name | Summary |
| :--- | :--- | :--- |
| PA-12 | Hepta Priority Score | HPS(p) = weighted dot product in [0,1]^7. |
| PA-13 | Orbital Layer Decomposition | Shell assignment by δ_T(p) = 1 − HPS(p). |
| PA-14 | Particle Position in Orbital Space | 3D position = (radius, azimuth, altitude) from KAKI bytes. |
| PA-15 | Tribe Birthing on the Rim | Child tribe emerges from outer-shell particle aggregation. |
| PA-16 | Multi-Scale Particle Rendering | PointSprite / Instanced / Volumetric by shell particle count. |

## Sovereign Constraints

§2.4: No assessments in KAKI nucleus.
§6.1: HPS is the canonical quality metric.
§6.2: Σ_T is the canonical tribe aggregate operator.

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/particle_position.md`
- `09_observatory/rendering_modes.md`
EOF

new_file docs/01_mathematics/tri_kaki_index.md <<'EOF'
# Tri-Kaki Enlil Index

> **DubSar Help** | `Math > Tri-Kaki Index` | Mathematics

## Purpose

The Tri-Kaki Enlil Index replaces all B-Tree and hash-table indexes in EnkiDB
with three algebraically direct lookup strategies, each O(1).

## The Three Strategies

### 1 — Identity-KAKI Index (Shamash Primary)
- Maps Identity-KAKI → Jordan Block memory address via spectral hash.
- Combined with Bloom Filter (ADAD Gate), the system knows instantly whether
  the particle exists and which Tribe it belongs to.
- Complexity: O(1).

### 2 — Events-KAKI Index (ADAD Temporal)
- Tracks only the tail of the Jordan Chain (most recent event).
- New Event-KAKI appends to the LRU tail; indexing = one pointer update.
- Old events fall off via nilpotency — no cleanup job needed.
- Complexity: O(1) append.

### 3 — CrossTribe-KAKI Index (ANU Relational)
- Stores basis-transformation matrices P and P⁻¹ between Tribe Jordan Blocks.
- Relationship traversal = one matrix multiplication (SIMD-accelerated).
- Materialised only on PROBE (§8.3), never stored permanently.
- Complexity: O(1) matrix multiply.

## Sovereign Constraints

§8.3: CrossTribe state computed on PROBE, never stored.
No full-table scans. No secondary indexes consuming disk space.

## See Also

- `01_mathematics/enlil_algebra.md`
- `02_identity/kaki_triad.md`
- `05_storage/enkidb.md`
EOF

new_file docs/01_mathematics/orbital_theorems.md <<'EOF'
# Orbital Layer Theorems — PA-13 to PA-16

> **DubSar Help** | `Math > Orbital Theorems` | Mathematics

## PA-13 — Orbital Layer Decomposition

Quality distance: δ_T(p) = 1 − HPS(p) ∈ [0,1].

Logarithmic shell radii: r_k = 1 − (1 − ε)^k.

Shell membership is computable in O(1) from δ_T(p) alone — no neighbor
inspection, no global state.

## PA-14 — Particle Position in Orbital Space

```
radius(p)  = δ_T(p) · R_max
azimuth(p) = 2π · κ(p)[12] / 256      (KAKI byte 12: D6 timestamp high)
altitude(p) = (κ(p)[14]/256 − 0.5) · H_max  (KAKI byte 14: D7 integrity)
```

Position is a pure function of (κ(p), ι_T). Two particles with the same KAKI
in the same tribe always occupy the same orbital position.

## PA-15 — Tribe Birthing on the Rim

ι_{T_child} = lim Σ_T(Q_n) where Q_n ⊂ T_parent ∩ Σ_{N-1}.

Outer-rim particles with emergent coherence compose a new child tribe. Their
KAKIs do not change; their storytelling journals continue unbroken.

## PA-16 — Multi-Scale Particle Rendering

| Shell count | Render mode |
| :--- | :--- |
| < 10,000 | PointSprite — per-particle textured quad. |
| 10,000 – 999,999 | Instanced — GPU-side instanced draw calls. |
| ≥ 1,000,000 | Volumetric — compute-shader density field + ray march. |

In all three modes, every particle retains a queryable KAKI. Volumetric
rendering is a perceptual compression, not an information loss.

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/particle_position.md`
- `09_observatory/rendering_modes.md`
- `_diagrams/orbital_layers_diagram.md`
EOF

new_file docs/01_mathematics/jnf.md <<'EOF'
# Jordan Normal Form in EnkiDB

> **DubSar Help** | `Math > JNF` | Mathematics

## Purpose

Jordan Normal Form (JNF) is the algebraic structure underlying every Tribe in
EnkiDB. A Tribe is a Jordan Block; its particles are the basis vectors; the
Orbit is the nilpotent chain connecting historical states.

## Structure

```
J_λ = [ λ  1  0  0 ]
      [ 0  λ  1  0 ]
      [ 0  0  λ  1 ]
      [ 0  0  0  λ ]
```

- λ = particle eigenvalue (HPS-derived).
- Off-diagonal 1s = the Orbit chain connecting successive states.
- Nilpotency: N^k = 0 for k ≥ orbit length — old events vanish naturally.

## Scaling Property

Because the full EnkiDB matrix is block-diagonal (one Jordan Block per Tribe),
adding a new Tribe adds a new block with zero interference to existing blocks.
This is the mathematical basis for infinite horizontal scaling.

## See Also

- `01_mathematics/enlil_algebra.md`
- `05_storage/enkidb.md`
EOF

# ─── 02_identity ───────────────────────────────────────────────────────────────
new_file docs/02_identity/kaki_triad.md <<'EOF'
# The KAKI Triad

> **DubSar Help** | `Identity > KAKI Triad` | Identity

## Purpose

The KAKI Triad is the three-way classification of all KAKI types in BahyWay.
Each type maps to a different indexing strategy and a different algebraic role.

## The Three Types

| Type | Algebraic Role | Index Strategy |
| :--- | :--- | :--- |
| **Identity-KAKI** | Nucleus (Eigenvalue λ) | O(1) spectral hash pointer |
| **Events-KAKI** | Orbit tail (Jordan chain entry) | O(1) LRU tail append |
| **CrossTribe-KAKI** | Basis transformation (P matrix) | O(1) matrix multiply (PROBE-only) |

## Sovereign Constraints

§2.4: No assessments in KAKI nucleus.
§8.3: CrossTribe state computed on PROBE, never stored.

## See Also

- `02_identity/identity_kaki.md`
- `02_identity/events_kaki.md`
- `02_identity/crosstribe_kaki.md`
- `01_mathematics/tri_kaki_index.md`
EOF

new_file docs/02_identity/identity_kaki.md <<'EOF'
# Identity-KAKI

> **DubSar Help** | `κ_id` | Identity

## Purpose

The Identity-KAKI is the immutable 16-byte nucleus that uniquely identifies a
particle across all storage nodes and all time. It is the "real component" in
the Octonion analogy.

## Mechanism

- Bytes 0–7: deterministic identity prefix derived from source system keys.
- Bytes 8–15: attribute suffix encoding particle type and creation context.
- Spectral hash maps the Identity-KAKI to a Jordan Block address in O(1).
- Bloom filter at the ADAD Gate pre-checks existence without a full lookup.

## Sovereign Constraints

§2.4: No assessments (scores, rankings, opinions) may be encoded in the
Identity-KAKI nucleus. The nucleus is a coordinate, not a judgment.

## See Also

- `02_identity/kaki_triad.md`
- `04_gates/adad_gate.md`
- `01_mathematics/tri_kaki_index.md`
EOF

new_file docs/02_identity/events_kaki.md <<'EOF'
# Events-KAKI

> **DubSar Help** | `κ_ev` | Identity

## Purpose

The Events-KAKI is the KAKI variant that represents a state-change event in a
particle's Orbit. It is the "Orbit tail" — the entry point of the Jordan Chain.

## Mechanism

- Each new event appends to the nilpotent chain without modifying prior events.
- The ADAD Gate (temporal exclusion) de-duplicates rapid-fire events within the
  configured `ADAD_BREATH_MS` window before an Events-KAKI is minted.
- Old events fall off the chain naturally via nilpotency — no archive job.

## Sovereign Constraints

The Events-KAKI is always subordinate to its Identity-KAKI. It cannot exist
without a parent nucleus.

## See Also

- `02_identity/kaki_triad.md`
- `04_gates/adad_gate.md`
- `01_mathematics/jnf.md`
EOF

new_file docs/02_identity/crosstribe_kaki.md <<'EOF'
# CrossTribe-KAKI

> **DubSar Help** | `κ_ct` | Identity

## Purpose

The CrossTribe-KAKI encodes the relationship between two particles belonging to
different Tribes. It is the algebraic "bridge" between Jordan Blocks.

## Mechanism

- Maps to a basis-transformation matrix P between two Tribe Jordan Blocks.
- The inverse P⁻¹ is stored alongside P so the traversal is bidirectional.
- The CrossTribe relationship is **materialised on PROBE only** — it is never
  persisted as a physical record in EnkiDB (§8.3).
- The ANU Gate governs which Tribe holds authority when CrossTribe particles
  conflict.

## Sovereign Constraints

§8.3: CrossTribe state computed on PROBE, never stored.
The CrossTribe-KAKI is a virtual coordinate, not a stored row.

## See Also

- `02_identity/kaki_triad.md`
- `04_gates/anu_gate.md`
- `01_mathematics/tri_kaki_index.md`
EOF

# ─── 03_kernel_mummu ───────────────────────────────────────────────────────────
new_file docs/03_kernel_mummu/lean4_verification.md <<'EOF'
# Lean 4 Verification Engine

> **DubSar Help** | `Kernel > Lean4` | Mathematical Brain

## Purpose

The Lean 4 prover is the Mathematical Brain (Mummu) of BahyWay. Every
algebraic identity, Pauli Exclusion rule, and KAKI uniqueness constraint is
expressed as a Lean 4 theorem before it is implemented in Rust.

## Mechanism

- Theorems PA-1 through PA-16 have corresponding Lean 4 proofs.
- The Pauli Exclusion gates (ADAD, ANU, MARDUK, SHAMASH) are formalised as
  type-level constraints in Lean 4.
- Proof obligations are checked in CI before any Rust crate is published.

## Sovereign Constraints

No runtime behaviour that is not covered by a Lean 4 proof may be merged into
the main branch.

## See Also

- `03_kernel_mummu/z3_validation.md`
- `01_mathematics/particles_algebra.md`
EOF

new_file docs/03_kernel_mummu/z3_validation.md <<'EOF'
# Z3 SMT Constraint Validation

> **DubSar Help** | `Kernel > Z3` | Mathematical Brain

## Purpose

Z3 is used alongside Lean 4 for constraint satisfaction problems that are
easier to express as satisfiability queries than as formal proofs — particularly
threshold validation for the four Pauli Exclusion gates.

## Mechanism

- ADAD temporal windows, ANU authority ranks, MARDUK noise limits, and
  SHAMASH judgment scores are validated as Z3 constraints.
- Z3 queries run in the pre-commit hook; a threshold that creates an
  unsatisfiable exclusion rule is rejected before it reaches CI.

## See Also

- `03_kernel_mummu/lean4_verification.md`
- `04_gates/high_council.md`
EOF

# ─── 04_gates ──────────────────────────────────────────────────────────────────
new_file docs/04_gates/high_council.md <<'EOF'
# The High Council of Data — Four Gates Overview

> **DubSar Help** | `Gates > High Council` | Governance

## Purpose

The High Council is the set of four Pauli Exclusion gates that every signal
must pass before its particle enters or updates the Active Orbit. The four
gates correspond to the four "occurrence spots" where data state conflicts arise
in a Smart City ecosystem.

## The Four Gates in Order

```
Signal ──► ADAD ──► ANU ──► MARDUK ──► SHAMASH ──► Active Orbit
```

| Gate | Spot | Law | Action on Violation |
| :--- | :--- | :--- | :--- |
| **ADAD** | Signal Input | Temporal Exclusion | Signal queued in Temporal Buffer |
| **ANU** | Source Hierarchy | Authority Exclusion | Signal demoted to Valence Band |
| **MARDUK** | Transformation | Structural Exclusion | Signal sent to Stewardship Gap |
| **SHAMASH** | Life/Death State | State Exclusion | Signal rejected until Steward approves |

## The Pauli Principle in Data Terms

No two particles may occupy the same (Identity-KAKI, Orbit-Level, QuantumState)
triplet simultaneously. The four gates enforce this principle at the four points
where violations naturally occur.

## See Also

- `04_gates/adad_gate.md`
- `04_gates/anu_gate.md`
- `04_gates/marduk_gate.md`
- `04_gates/shamash_gate.md`
- `04_gates/pauli_exclusion.md`
- `_diagrams/high_council_diagram.md`
EOF

new_file docs/04_gates/pauli_exclusion.md <<'EOF'
# Pauli Exclusion Principle in TOP Algebra

> **DubSar Help** | `Gates > Pauli Exclusion` | Governance

## Purpose

The Pauli Exclusion Principle is applied to the attributes (particles) orbiting
an Identity-KAKI, not to the KAKI nucleus itself. It ensures that only one
"Active" value can occupy the "Golden State" at any given time for a given KAKI.

## The Quantum Numbers

Each particle has three quantum numbers that define its exclusion coordinate:

1. **λ (Eigenvalue)** — Identity-KAKI stability. Derived from HPS.
2. **n (Principal Number)** — Position in the Jordan Chain (Orbit depth).
3. **s (Spin/Status)** — State: Dead (−1), Stewardship (0), Active (+1).

## The Three Energy Shells

| Shell | State | Physics Analogy | Behaviour |
| :--- | :--- | :--- | :--- |
| Active | `s = +1` | Conduction Band | Golden Record candidate. |
| Stewardship | `s = 0` | Forbidden Gap | Awaiting Data Steward correction. |
| Dead | `s = −1` | Valence Band | Archived. Acts as a mathematical barrier. |

## Why the KAKI Nucleus Does Not Violate Pauli

The Identity-KAKI is the nucleus (the atom). Pauli Exclusion applies to the
attributes (electrons) orbiting that nucleus. Multiple values for the same KAKI
can coexist only if they occupy different orbit levels or different states.

## See Also

- `04_gates/high_council.md`
- `00_codex/axioms.md`
EOF

new_file docs/04_gates/adad_gate.md <<'EOF'
# ADAD Gate — Temporal Exclusion

> **DubSar Help** | `ADAD` | Gates

## Akkadian Identity

ADAD (also Adad or Hadad) — the Lord of Wind, Storm, and Decree. Controls the
"air" through which signals enter the system. He decides when a signal is
allowed into the lungs of the database.

## Purpose

ADAD is Gate 1. It de-duplicates rapid-fire signals for the same Identity-KAKI
to prevent "Data Flapping" where the Golden Record switches between two
identical-looking values faster than the system can stabilise.

## Mechanism

- **Temporal window**: `ADAD_BREATH_MS` (configurable; default 50 ms).
- If two signals for the same Identity-KAKI arrive within the window, ADAD
  promotes one to the Active Orbit and queues the other in the Temporal Buffer
  (next shell).
- Combined with the Bloom Filter pre-check, existence tests are O(1) before
  any Jordan Block lookup is attempted.

## Rust Gate Constant

```rust
pub const ADAD_BREATH_MS: u64 = 50;
```

## Sovereign Constraints

ADAD operates at the signal input layer, before any particle state is written.
It does not modify existing particles — it only controls admission.

## See Also

- `04_gates/high_council.md`
- `02_identity/events_kaki.md`
- `01_mathematics/tri_kaki_index.md`
EOF

new_file docs/04_gates/anu_gate.md <<'EOF'
# ANU Gate — Authority Exclusion

> **DubSar Help** | `ANU` | Gates

## Akkadian Identity

ANU (An) — the Sky Father, Supreme Authority, source of all kingship. He sits
at the highest energy level and determines who has the "Right to Rule."

## Purpose

ANU is Gate 2. It governs multi-source authority: when a CRM, a Power Grid
sensor, and an ERP all claim the same Identity-KAKI, ANU determines which
source occupies the Master Position (n = 0) in the Jordan Block.

## Mechanism

- Each signal carries a `source_rank` (authority score).
- `ANU_AUTHORITY_RANK` defines the minimum rank required to occupy the Master
  Position.
- High-authority source: Active shell (Conduction Band).
- Low-authority source: Excluded to Valence Band (Historical/Backup orbit).
- CrossTribe-KAKI relationships are governed by ANU when two Tribes conflict
  over the same Identity-KAKI.

## Rust Gate Constant

```rust
pub const ANU_MIN_AUTHORITY: u32 = 100;
```

## Sovereign Constraints

§8.3: CrossTribe state computed on PROBE, never stored.
ANU never overwrites a higher-authority value with a lower-authority one.

## See Also

- `04_gates/high_council.md`
- `02_identity/crosstribe_kaki.md`
EOF

new_file docs/04_gates/marduk_gate.md <<'EOF'
# MARDUK Gate — Structural Exclusion

> **DubSar Help** | `MARDUK` | Gates

## Akkadian Identity

MARDUK — The Architect of Order, who defeated Tiamat (chaos) and structured the
universe. Master of the "Order of Things."

## Purpose

MARDUK is Gate 3. It manages transformation locks: if a particle is currently
being cleansed by the VGCA Delta engine, MARDUK prevents simultaneous manual
edits or automated updates from creating a race condition.

## Mechanism

- **Governance Lock**: when a particle enters the Cleansing State, MARDUK sets
  an exclusive lock on that (Identity-KAKI, Orbit-Level) coordinate.
- **Human authority wins**: if a Data Steward edits a particle already locked
  by the automated pipeline, MARDUK pauses the pipeline (lower quantum spin)
  and elevates the Steward's edit (higher quantum spin).
- **Noise threshold**: `MARDUK_ORDER_INDEX` — if the VGCA delta exceeds this
  threshold, the particle is sent to the Stewardship Gap instead of being
  passed through.

## Rust Gate Constant

```rust
pub const MARDUK_ORDER_INDEX: f64 = 0.35;
```

## Sovereign Constraints

While MARDUK holds a lock, no other process may modify the locked particle.
Locks are held for the minimum duration of the transformation pipeline stage.

## See Also

- `04_gates/high_council.md`
- `04_gates/shamash_gate.md`
- `06_governance_parzu/parzu_laws.md`
EOF

new_file docs/04_gates/shamash_gate.md <<'EOF'
# SHAMASH Gate — State Exclusion

> **DubSar Help** | `SHAMASH` | Gates

## Akkadian Identity

SHAMASH (Šamaš) — The Sun God and Judge of Truth. He sees all, brings hidden
things to light, and judges both the living and the dead.

## Purpose

SHAMASH is Gate 4. It guards the boundary between the Active Orbit and the Dead
State. It prevents "Zombie" data — a signal for a Dead particle that should not
be reactivated — from corrupting the Active Orbit.

## Mechanism

- **Dead particle detection**: if an incoming signal's Identity-KAKI matches a
  record in the Dead State, SHAMASH intercepts it.
- **Steward approval required**: the signal is placed in the Stewardship Gap.
  The Data Steward must decide:
  - **Zombie**: the signal is a mistake → reject.
  - **Reincarnation**: the sensor/entity was replaced → approve and re-mint
    a new particle with a new Events-KAKI lineage.
- **Judgment score**: `SHAMASH_JUDGMENT_SCORE` — the confidence threshold
  required for automatic reincarnation without Steward intervention.
- **Spectral archeology**: Dead particles are never deleted; they remain in
  the Ground State as mathematical barriers and audit records.

## Rust Gate Constant

```rust
pub const SHAMASH_JUDGMENT_SCORE: f64 = 0.95;
```

## Sovereign Constraints

A Dead particle is never overwritten. Its archived state is permanent and
queryable by the storytelling system (StoryWay).

## See Also

- `04_gates/high_council.md`
- `04_gates/pauli_exclusion.md`
- `09_observatory/orbital_visualization.md`
EOF

# ─── 05_storage ────────────────────────────────────────────────────────────────
new_file docs/05_storage/enkidb.md <<'EOF'
# EnkiDB — Hot Plane Storage

> **DubSar Help** | `EnkiDB` | Storage

## Purpose

EnkiDB is the sovereign operational memory of BahyWay. It stores particles as
Entity-Attribute-Value (EAV) triples in a block-diagonal Jordan Block layout.
There are no tables, no rows, no foreign keys.

## Mechanism

- **Hot plane**: particles in the Active and Stewardship states live in the
  hot plane (VRAM or high-speed memory).
- **Tri-Kaki indexing**: Identity-KAKI → O(1) block pointer; Events-KAKI → O(1)
  tail append; CrossTribe-KAKI → O(1) basis-transform on PROBE.
- **GPU resonance query**: Vulkan compute shader applies the Jordan Observable
  matrix across all particles in one pass — no sequential scan.
- **No third-party databases**: EnkiDB is implemented entirely in Pure Rust.

## Sovereign Constraints

§2.4: No assessments in KAKI nucleus.
§8.3: CrossTribe state computed on PROBE, never stored.
No PostgreSQL, SQLite, or any external database engine.

## See Also

- `05_storage/enkidw.md`
- `05_storage/enkistream.md`
- `01_mathematics/enlil_algebra.md`
EOF

new_file docs/05_storage/enkidw.md <<'EOF'
# EnkiDW — Data Warehouse Persistence

> **DubSar Help** | `EnkiDW` | Storage

## Purpose

EnkiDW is the persistence layer for historical Orbit data. It stores the full
Jordan Chain of every particle — the immutable audit trail of all state
transitions since particle birth.

## Mechanism

- Dead particles are "frozen" into EnkiDW in their Ground State.
- Spectral Renormalization compresses petabyte-scale Tribe histories into
  Macro-Eigenvalue summaries for Hubble-Zoom queries.
- EnkiDW is append-only; no record is ever modified or deleted.

## See Also

- `05_storage/enkidb.md`
- `04_gates/shamash_gate.md`
- `09_observatory/orbital_visualization.md`
EOF

new_file docs/05_storage/enkistream.md <<'EOF'
# EnkiStream — Replication

> **DubSar Help** | `EnkiStream` | Storage

## Purpose

EnkiStream handles real-time replication of particle state changes across
EnkiDB nodes. It uses the Events-KAKI as its unit of replication.

## Mechanism

- Each Events-KAKI that passes all four Pauli Gates is written to the
  EnkiStream log before being applied to the hot plane.
- Replication consumers read the log and apply the same Jordan Chain append
  on their local node.
- Divergence detection: if a consumer's Jordan Block hash does not match
  the primary, a reconciliation PROBE is triggered.

## See Also

- `05_storage/enkidb.md`
- `02_identity/events_kaki.md`
EOF

# ─── 06_governance_parzu ───────────────────────────────────────────────────────
new_file docs/06_governance_parzu/parzu_laws.md <<'EOF'
# PARZU Governance Laws

> **DubSar Help** | `PARZU` | Governance

## Purpose

PARZU is the governance layer that enforces the .akk law files at runtime.
Every Tribe has a PARZU ruleset that defines which particle states are valid,
which state transitions are permitted, and what thresholds trigger a Stewardship
Gap referral.

## Mechanism

- .akk files declare the PARZU rules for a Tribe in the Akkadian Scripting
  Language.
- At runtime, the PARZU engine evaluates each particle transition against its
  Tribe's .akk rules.
- Rule violations route the particle to the appropriate Pauli Gate for
  exclusion or escalation.

## See Also

- `07_file_formats/akk_format.md`
- `04_gates/high_council.md`
EOF

new_file docs/06_governance_parzu/template_registry.md <<'EOF'
# Template Registry

> **DubSar Help** | `PARZU > Templates` | Governance

## Purpose

The Template Registry is the centralised store of .tmpl files that define
reusable PARZU rule patterns. Tribes instantiate templates rather than
authoring raw .akk files from scratch.

## See Also

- `07_file_formats/tmpl_format.md`
- `06_governance_parzu/parzu_laws.md`
EOF

# ─── 07_file_formats ───────────────────────────────────────────────────────────
new_file docs/07_file_formats/akk_format.md <<'EOF'
# .akk File Format

> **DubSar Help** | `.akk` | File Formats

## Purpose

The .akk file is the law file for a Tribe or particle family. It declares the
PARZU governance rules that the MARDUK Gate enforces during transformation.

## Grammar (sketch)

```
akk_file     := header rule*
header       := "tribe" tribe_id version
rule         := "on" event "when" condition "do" action
condition    := expr ("&&" | "||" expr)*
action       := "promote" | "demote" | "steward" | "archive"
```

## Sovereign Constraints

Every .akk change must be stamped with a reason context (the documentation
system) so auditors can trace why a governance rule changed.

## See Also

- `06_governance_parzu/parzu_laws.md`
- `07_file_formats/tmpl_format.md`
EOF

new_file docs/07_file_formats/hepta_format.md <<'EOF'
# .hepta File Format

> **DubSar Help** | `.hepta` | File Formats

## Purpose

The .hepta file stores the 7D Hepta vector snapshot for a particle or tribe.
It is the persisted form of the Heptagon quality score.

## Structure

7 × f32 values in little-endian binary, one per dimension:

| Byte offset | Dimension |
| :--- | :--- |
| 0–3 | D1 |
| 4–7 | D2 |
| 8–11 | D3 |
| 12–15 | D4 |
| 16–19 | D5 |
| 20–23 | D6 (timestamp-derived) |
| 24–27 | D7 (integrity checksum-derived) |

## See Also

- `01_mathematics/particles_algebra.md`
- `09_observatory/particle_position.md`
EOF

new_file docs/07_file_formats/way_format.md <<'EOF'
# .way File Format

> **DubSar Help** | `.way` | File Formats

## Purpose

The .way file is the WAYv2.0 Language source file. It is reserved exclusively
for the WAYv2.0 scripting language and must not be confused with the deprecated
"Way" suffix on crate names.

## Sovereign Constraints

The suffix "Way" is deprecated from all crate and engine names. It is reserved
exclusively for WAYv2.0 Language `.way` files.

## See Also

- `07_file_formats/akk_format.md`
EOF

new_file docs/07_file_formats/tmpl_format.md <<'EOF'
# .tmpl File Format

> **DubSar Help** | `.tmpl` | File Formats

## Purpose

The .tmpl file is a reusable PARZU rule template. Tribes instantiate .tmpl
files by binding Tribe-specific parameters to the template placeholders.

## See Also

- `06_governance_parzu/template_registry.md`
- `07_file_formats/akk_format.md`
EOF

# ─── 08_pipeline_alaktu ────────────────────────────────────────────────────────
new_file docs/08_pipeline_alaktu/submission.md <<'EOF'
# Submission Pipeline

> **DubSar Help** | `Pipeline > Submit` | CI/CD

## Purpose

The submission stage is the entry point of the ALAKTU pipeline. A particle or
.akk file is submitted, validated by the four Pauli Gates, and queued for the
verification stage.

## Steps

1. Signal arrives at ADAD Gate (temporal de-duplication).
2. Signal passes ANU Gate (authority check).
3. Signal passes MARDUK Gate (structure / noise check).
4. Signal passes SHAMASH Gate (state / archive check).
5. Particle is staged in the hot plane with `state = Stewardship`.
6. Pipeline notifies the verification stage.

## See Also

- `08_pipeline_alaktu/verification.md`
- `04_gates/high_council.md`
EOF

new_file docs/08_pipeline_alaktu/verification.md <<'EOF'
# Verification Stage

> **DubSar Help** | `Pipeline > Verify` | CI/CD

## Purpose

The verification stage applies Lean 4 proof obligations and Z3 constraint
checks to the staged particle before it is released to the Active Orbit.

## Steps

1. Lean 4 prover checks algebraic invariants (§ references from Particles Algebra).
2. Z3 validates threshold constraints for ADAD, ANU, MARDUK, SHAMASH.
3. If all checks pass: particle state transitions `Stewardship → Active`.
4. If any check fails: particle remains in Stewardship; Steward is notified.

## See Also

- `08_pipeline_alaktu/submission.md`
- `08_pipeline_alaktu/release.md`
- `03_kernel_mummu/lean4_verification.md`
EOF

new_file docs/08_pipeline_alaktu/release.md <<'EOF'
# Release Pipeline

> **DubSar Help** | `Pipeline > Release` | CI/CD

## Purpose

The release stage publishes a verified particle or .akk file to the EnkiDB
hot plane and EnkiStream replication log.

## Steps

1. Particle state confirmed as `Active`.
2. Events-KAKI minted and appended to Jordan Chain.
3. CrossTribe-KAKI links computed (PROBE-only; §8.3).
4. EnkiStream log entry written.
5. Orbital visualization updated (shell re-binning if HPS changed).

## See Also

- `08_pipeline_alaktu/verification.md`
- `05_storage/enkistream.md`
EOF

# ─── 09_observatory ────────────────────────────────────────────────────────────
new_file docs/09_observatory/orbital_visualization.md <<'EOF'
# Orbital Visualization — Overview

> **DubSar Help** | `Observatory` | Observatory

## Purpose

The orbital visualization renders every particle in a Tribe as a point in 3D
space, where position is a pure function of the particle's KAKI and the Tribe
Ideal (§PA-14). Shell membership is determined by quality distance (§PA-13).

## Architecture

- **Shell decomposition** (PA-13): logarithmic rings, inner = gold, outer = dead.
- **Particle position** (PA-14): (radius, azimuth, altitude) from KAKI bytes.
- **Tribe birthing** (PA-15): outer-rim child tribes visible as sub-ring systems.
- **Rendering modes** (PA-16): PointSprite / Instanced / Volumetric.
- **Click-to-storytelling**: every particle retains its KAKI in all render modes;
  click → KAKI → full storytelling journal in StoryWay.

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/particle_position.md`
- `09_observatory/tribe_birthing.md`
- `09_observatory/rendering_modes.md`
- `09_observatory/hubble_zoom.md`
- `_diagrams/orbital_layers_diagram.md`
EOF

new_file docs/09_observatory/shell_decomposition.md <<'EOF'
# Shell Decomposition — PA-13

> **DubSar Help** | `Observatory > Shells` | Observatory

## Purpose

PA-13 defines how every particle in a Tribe is assigned to exactly one orbital
shell based on its quality distance from the Tribe Ideal.

## Formal Definition

```
δ_T(p) = 1 − HPS(p)          ∈ [0, 1]
r_k    = 1 − (1 − ε)^k       (logarithmic shell radii)
Σ_k    = {p ∈ T : r_k ≤ δ_T(p) < r_{k+1}}
```

- δ_T(p) = 0 → perfect alignment with Tribe Ideal (Golden Record).
- δ_T(p) = 1 → maximum divergence (Dead Particle).
- Shell membership is O(1) from δ_T(p) — no iteration over other particles.

## DubSar IDE Hover

When you hover over a shell ring in the Observatory, the tooltip shows:
- Shell index k
- Shell radius range [r_k, r_{k+1})
- Particle count |Σ_k|
- Current render mode for this shell

## See Also

- `09_observatory/particle_position.md`
- `01_mathematics/orbital_theorems.md`
EOF

new_file docs/09_observatory/particle_position.md <<'EOF'
# Particle Position in Orbital Space — PA-14

> **DubSar Help** | `Observatory > Particle Position` | Observatory

## Purpose

PA-14 defines the 3D position of every particle as a pure function of its KAKI
and the Tribe Ideal. No layout algorithm, no force-directed placement.

## Formula

```
radius(p)   = δ_T(p) · R_max
azimuth(p)  = 2π · κ(p)[12] / 256      ← KAKI byte 12 (D6 timestamp high)
altitude(p) = (κ(p)[14] / 256 − 0.5) · H_max  ← KAKI byte 14 (D7 integrity)
```

## Properties

- **Deterministic**: same KAKI + same Tribe Ideal → same position, always.
- **Sovereign**: orbital position follows from identity, not from circumstance.
- **Temporal locality**: particles created at similar times cluster at similar
  azimuths (KAKI byte 12 is derived from the D6 timestamp).
- **Integrity stratification**: particles with similar checksums sit at similar
  altitudes (KAKI byte 14 is derived from the D7 integrity checksum).

## See Also

- `09_observatory/shell_decomposition.md`
- `09_observatory/rendering_modes.md`
- `01_mathematics/orbital_theorems.md`
EOF

new_file docs/09_observatory/tribe_birthing.md <<'EOF'
# Tribe Birthing on the Rim — PA-15

> **DubSar Help** | `Observatory > Tribe Birthing` | Observatory

## Purpose

PA-15 describes how a child Tribe is born from the outer-rim particles of a
parent Tribe when those particles exhibit emergent coherence.

## The Rim-Birth Pattern

```
ι_{T_child} = lim_{n→∞} Σ_T(Q_n)
where Q_n ⊂ T_parent ∩ Σ_{N-1}
```

- The child Tribe Ideal emerges as the aggregate of outer-shell particles that
  point in a coherent direction.
- Once T_child is born, those particles migrate from T_parent's outer rim to
  T_child's inner orbit.
- Their KAKIs do not change. Their storytelling journals continue unbroken.

## Visual Appearance

In the orbital visualization: the outer ring of the parent Tribe carries small
sub-ring systems. Each sub-ring is an emerging child Tribe with its own center
and its own orbital shells.

## See Also

- `09_observatory/orbital_visualization.md`
- `09_observatory/shell_decomposition.md`
EOF

new_file docs/09_observatory/rendering_modes.md <<'EOF'
# Multi-Scale Rendering Modes — PA-16

> **DubSar Help** | `Observatory > Rendering` | Observatory

## Purpose

PA-16 defines three rendering modes that switch automatically based on the
particle count in a shell (|Σ_k|). In all three modes, every particle retains
its KAKI — no information is lost.

## The Three Modes

### PointSprite (|Σ_k| < 10,000)
- Each particle is a textured quad with KAKI-derived colour.
- Click-to-storytelling: direct KAKI lookup.
- Used for: small tribes, inner gold-quality shells, zoomed-in views.

### Instanced (10,000 ≤ |Σ_k| < 1,000,000)
- GPU-side instanced rendering; tens of thousands per draw call.
- Click-to-storytelling: picking buffer maps screen pixel → KAKI.
- Used for: mid-density shells.

### Volumetric (|Σ_k| ≥ 1,000,000)
- Density field accumulated by compute shader; ray-marched for display.
- Click-to-storytelling: clicking a region triggers a zoom-in that switches
  to Instanced or PointSprite mode. 2–3 clicks to reach any individual KAKI.
- Used for: high-density outer shells where individual particles are visually
  indistinguishable.

## The Storytelling Preservation Theorem

> In all three rendering modes, every particle retains a queryable KAKI.
> Volumetric rendering is a compression for visual perception, not for the data.

## See Also

- `09_observatory/orbital_visualization.md`
- `09_observatory/hubble_zoom.md`
EOF

new_file docs/09_observatory/hubble_zoom.md <<'EOF'
# Hubble-Zoom Interface

> **DubSar Help** | `Observatory > Hubble-Zoom` | Observatory

## Purpose

The Hubble-Zoom is the progressive-disclosure navigation model of the
Observatory. It allows the viewer to travel from an Exabyte-scale "galaxy" view
down to a single particle's storytelling journal in 2–3 clicks.

## Zoom Levels

| Level | Mathematical Mechanism | Visual |
| :--- | :--- | :--- |
| Exabyte (Universe) | Spectral Radius / Trace of macro-clusters | Galaxy clouds |
| Petabyte (Galaxy) | Spectral Aggregation | Tribe ring systems |
| Terabyte (System) | Jordan Block unpacking | Individual orbital shells |
| 1 Billion (Active View) | PointSprite / Instanced render | Individual particles |
| 1 Particle | KAKI → StoryWay lookup | Full storytelling journal |

## Sovereign Guarantee

At every zoom level, the KAKI of any visible particle is reachable within 2–3
clicks. No particle is ever "lost" behind a rendering mode.

## See Also

- `09_observatory/rendering_modes.md`
- `09_observatory/orbital_visualization.md`
- `01_mathematics/enlil_algebra.md`
EOF

# ─── 10_operations ─────────────────────────────────────────────────────────────
new_file docs/10_operations/deployment_topology.md <<'EOF'
# Deployment Topology

> **DubSar Help** | `Ops > Topology` | Operations

## Purpose

Describes how EnkiDB nodes, EnkiStream replication, and the Observatory are
deployed in a production BahyWay environment.

## See Also

- `05_storage/enkidb.md`
- `05_storage/enkistream.md`
- `16_runbooks/node_recovery.md`
EOF

# ─── 11_tooling ────────────────────────────────────────────────────────────────
new_file docs/11_tooling/nabu_cli.md <<'EOF'
# Nabu CLI Reference

> **DubSar Help** | `nabu` | Tooling

## Purpose

Nabu is the command-line interface for BahyWay. It translates HeptaScript
queries into Jordan Observable matrices and dispatches them to EnkiDB.

## Key Commands

```
nabu probe  <kaki>          Query a single particle by KAKI.
nabu tribe  <tribe_id>      List particles in a Tribe with HPS scores.
nabu ingest <source>        Submit a new signal through all four Pauli Gates.
nabu audit  <kaki>          Show the full Jordan Chain (storytelling journal).
nabu zoom   <tribe_id> <k>  Render shell k of a Tribe in the Observatory.
```

## Sovereign Constraints

`nabu probe` triggers CrossTribe-KAKI computation on demand (§8.3). The result
is displayed but never written back to EnkiDB.

## See Also

- `05_storage/enkidb.md`
- `01_mathematics/tri_kaki_index.md`
EOF

# ─── 12_examples ───────────────────────────────────────────────────────────────
new_file docs/12_examples/najaf_grave_lookup.md <<'EOF'
# Example — Najaf Grave Lookup via Hubble-Zoom

> **DubSar Help** | `Examples > Najaf` | Examples

## Scenario

A stakeholder wants to find the grave record for a specific citizen in the
Najaf database, which holds 1 billion particle records.

## Steps

1. `nabu probe <citizen_kaki>` — SHAMASH Gate confirms particle is Active.
2. Observatory Hubble-Zoom to Najaf Tribe at Exabyte view.
3. Zoom in to the citizen's shell (δ_T computed from HPS).
4. PointSprite render at inner shell; click the particle.
5. StoryWay journal opens: full lineage, .akk rule firings, HPS transitions.

## Sovereign Guarantee

The Hubble-Zoom reaches the individual particle in ≤ 3 clicks regardless of
the total particle count in the Najaf Tribe.

## See Also

- `09_observatory/hubble_zoom.md`
- `11_tooling/nabu_cli.md`
EOF

# ─── 13_changelog ──────────────────────────────────────────────────────────────
new_file docs/13_changelog/v4_0_2.md <<'EOF'
# v4.0.2 Changelog

> **DubSar Help** | `Changelog > v4.0.2` | Changelog

## Changes

- Added `enkidullm-core`, `enkidullm-ingest`, `zikru-embed`, `enkidullm-audit`
  crates with full MANUAL.md and L1–L4 test coverage.
- Fixed DEFLATE RFC 1951 BitReader `align_to_byte()` rewind bug.
- Fixed float roundtrip precision in CRC-16 audit payload construction.
- Fixed PDF `read_keyword` infinite loop on `/` delimiter.
- Fixed Epub `<dc:identifier` search and ISBN hyphen stripping.
- Fixed `sector_weights` shape from `[dim, 7]` to `[7, dim]`.
- Fixed constant-row quantization scale for zero-range matrices.

## See Also

- `19_roadmap/versioning_policy.md`
EOF

# ─── 14_decisions_adr ──────────────────────────────────────────────────────────
new_file docs/14_decisions_adr/adr_001_no_external_db.md <<'EOF'
# ADR-001 — No External Databases

> **DubSar Help** | `ADR > 001` | Architecture Decisions

## Status: Accepted

## Context

BahyWay must maintain data sovereignty. Third-party databases introduce
dependency on external schema migrations, license constraints, and network
latency between the data physics engine and the persistence layer.

## Decision

EnkiDB is implemented entirely in Pure Rust. No PostgreSQL, SQLite, Redis,
RocksDB, or any other external database engine is used or permitted.

## Consequences

- All indexing, journaling, and replication logic is owned by the BahyWay team.
- The Enlil Algebra indexing strategy (Tri-Kaki, O(1) lookups) must be
  implemented from scratch — no B-Tree library shortcuts.
- Maximum control over particle state lifecycle and KAKI sovereignty.
EOF

new_file docs/14_decisions_adr/adr_002_pauli_gates.md <<'EOF'
# ADR-002 — Four Pauli Exclusion Gates

> **DubSar Help** | `ADR > 002` | Architecture Decisions

## Status: Accepted

## Context

Smart City data arrives from thousands of concurrent sources. Without formal
exclusion rules, the system would suffer from data flapping, race conditions,
source conflicts, and zombie data corrupting the Active Orbit.

## Decision

Adopt the four Pauli Exclusion gates (ADAD, ANU, MARDUK, SHAMASH) as the
canonical governance model. Every signal must pass all four gates before
entering the Active Orbit. The gates are named after Mesopotamian deities to
align with the broader BahyWay mythological naming system.

## Consequences

- All data state conflicts are handled by a single, composable pipeline.
- Gate thresholds (ADAD_BREATH_MS, ANU_AUTHORITY_RANK, etc.) are tunable per
  Tribe via .akk files.
- Lean 4 and Z3 provide formal verification of the exclusion rules.
EOF

new_file docs/14_decisions_adr/adr_003_kaki_sovereignty.md <<'EOF'
# ADR-003 — KAKI Sovereignty (§2.4, §8.3)

> **DubSar Help** | `ADR > 003` | Architecture Decisions

## Status: Accepted

## Context

Two constraints are fundamental to the KAKI identity model:
1. The KAKI nucleus must not contain assessments (§2.4).
2. CrossTribe state must be computed on PROBE, never stored (§8.3).

## Decision

These constraints are encoded as type-level invariants in the `enkidb-kaki`
crate and enforced by the Lean 4 prover in CI. No merge may pass that violates
either constraint.

## Consequences

- KAKI nuclei are coordinates, not judgments.
- CrossTribe relationships are virtual — they consume no storage.
- The system is provably free of "assessment inflation" in the identity layer.
EOF

# ─── 15_howto ──────────────────────────────────────────────────────────────────
new_file docs/15_howto/add_a_new_tribe.md <<'EOF'
# How To — Add a New Tribe

> **DubSar Help** | `HowTo > New Tribe` | How-To

## Steps

1. Write a `.akk` file defining the Tribe Ideal (ι_T ∈ [0,1]^7) and PARZU rules.
2. Register the `.akk` file in the Template Registry (`06_governance_parzu/`).
3. Submit via `nabu ingest <akk_file>` — this passes all four Pauli Gates.
4. Verify in the Observatory: the new Tribe ring should appear at the outer rim.
5. Seed particles using `nabu ingest <particle_source>`.

## See Also

- `07_file_formats/akk_format.md`
- `06_governance_parzu/parzu_laws.md`
- `11_tooling/nabu_cli.md`
EOF

new_file docs/15_howto/audit_a_golden_record.md <<'EOF'
# How To — Audit a Golden Record

> **DubSar Help** | `HowTo > Audit` | How-To

## Steps

1. `nabu probe <kaki>` — confirms Active state and HPS score.
2. `nabu audit <kaki>` — opens the full Jordan Chain (StoryWay journal).
3. Review each Events-KAKI entry: timestamp, source, ADAD/ANU/MARDUK/SHAMASH
   gate decisions, HPS delta, .akk rule firing that caused the transition.
4. If the promotion to Golden Record is disputed, trace the MARDUK gate log
   for the transformation that raised HPS to the threshold.

## Sovereign Guarantee

Every state transition is KAKI-stamped. The audit answer is deterministic —
no inference, no prediction. The recorded truth of the particle's trajectory
through Hepta-space.

## See Also

- `09_observatory/orbital_visualization.md`
- `04_gates/marduk_gate.md`
- `11_tooling/nabu_cli.md`
EOF

# ─── 16_runbooks ───────────────────────────────────────────────────────────────
new_file docs/16_runbooks/node_recovery.md <<'EOF'
# Runbook — EnkiDB Node Recovery

> **DubSar Help** | `Runbooks > Node Recovery` | Runbooks

## Trigger

An EnkiDB node goes offline. EnkiStream replication lag exceeds threshold.

## Steps

1. Confirm node failure via `nabu tribe <tribe_id>` — expect timeout or error.
2. Check EnkiStream log for last committed Events-KAKI before failure.
3. Restore the Jordan Block from the most recent EnkiDW snapshot.
4. Replay EnkiStream events from the snapshot timestamp to now.
5. Verify Jordan Block hash matches the primary node.
6. Re-enable the node in the replication topology.

## See Also

- `05_storage/enkistream.md`
- `05_storage/enkidw.md`
EOF

# ─── 17_troubleshooting ────────────────────────────────────────────────────────
new_file docs/17_troubleshooting/adad_temporal_collision.md <<'EOF'
# Troubleshooting — ADAD Temporal Collision

> **DubSar Help** | `Troubleshoot > ADAD` | Troubleshooting

## Symptom

Particles are being queued in the Temporal Buffer but not promoted to Active.
The ADAD Gate log shows repeated `NabuViolation` (temporal overlap) errors.

## Cause

Two or more sources are emitting signals for the same Identity-KAKI faster than
the `ADAD_BREATH_MS` window allows.

## Fix

1. Check the source emission rate — reduce polling frequency on the sensor.
2. If the collision is expected (e.g., two legitimate simultaneous readings),
   increase `ADAD_BREATH_MS` in the Tribe's .akk file.
3. If sources are duplicated (data feed misconfiguration), disable the
   duplicate source at the ANU Gate level by lowering its authority rank.

## See Also

- `04_gates/adad_gate.md`
- `04_gates/anu_gate.md`
EOF

new_file docs/17_troubleshooting/shamash_zombie_detection.md <<'EOF'
# Troubleshooting — SHAMASH Zombie Detection

> **DubSar Help** | `Troubleshoot > Zombie` | Troubleshooting

## Symptom

A signal arrives for an Identity-KAKI that is in the Dead State. The SHAMASH
Gate log shows `EreshkigalViolation` (unauthorized reincarnation).

## Cause

A replaced sensor or re-activated entity is emitting signals, but its
Identity-KAKI is still marked Dead in EnkiDB.

## Fix

1. `nabu audit <kaki>` — confirm the particle is in the Dead State and review
   the SHAMASH judgment that archived it.
2. If this is a **Zombie** (the signal is a mistake): reject the signal at the
   ADAD Gate by adding the Identity-KAKI to the blocklist.
3. If this is a **Reincarnation** (the sensor was genuinely replaced): the Data
   Steward approves the reincarnation. A new Events-KAKI lineage begins; the
   Dead particle remains archived.

## See Also

- `04_gates/shamash_gate.md`
- `04_gates/pauli_exclusion.md`
EOF

# ─── 18_security ───────────────────────────────────────────────────────────────
new_file docs/18_security/threat_model.md <<'EOF'
# Threat Model

> **DubSar Help** | `Security > Threat Model` | Security

## Assets

- Identity-KAKI namespace (forgery = identity theft).
- Jordan Chain audit trail (tampering = history falsification).
- Tribe Ideal vectors (manipulation = governance subversion).

## Threats

| Threat | Gate | Mitigation |
| :--- | :--- | :--- |
| Duplicate signal injection | ADAD | Temporal window de-duplication. |
| Low-authority source spoofing | ANU | Authority rank validation. |
| Mid-pipeline data corruption | MARDUK | Transformation lock + VGCA delta check. |
| Dead record reactivation | SHAMASH | Steward approval required for reincarnation. |
| KAKI nucleus forgery | KAKI crate | Lean 4 type invariants in CI. |

## See Also

- `04_gates/high_council.md`
- `18_security/audit_trail.md`
EOF

new_file docs/18_security/audit_trail.md <<'EOF'
# Audit Trail

> **DubSar Help** | `Security > Audit Trail` | Security

## Purpose

Every particle state transition is KAKI-stamped and written to EnkiDW in
append-only fashion. The audit trail is the Jordan Chain itself.

## What Is Recorded

- Events-KAKI of each transition.
- Timestamp (D6 byte of KAKI).
- Source authority rank (ANU Gate decision).
- Gate decisions (ADAD/ANU/MARDUK/SHAMASH pass/fail).
- VGCA delta that MARDUK evaluated.
- HPS value before and after the transition.
- .akk rule firing that triggered the transition.

## Queryable via

`nabu audit <kaki>` returns the full Jordan Chain for any particle.

## See Also

- `05_storage/enkidw.md`
- `04_gates/high_council.md`
EOF

# ─── 19_roadmap ────────────────────────────────────────────────────────────────
new_file docs/19_roadmap/versioning_policy.md <<'EOF'
# Versioning Policy

> **DubSar Help** | `Roadmap > Versioning` | Roadmap

## Version Format

`v{major}.{minor}.{patch}` — semantic versioning.

- **Major**: algebraic invariant changes (new PA theorem, gate redesign).
- **Minor**: new crates or features that do not break existing KAKI contracts.
- **Patch**: bug fixes, performance improvements, documentation updates.

## See Also

- `13_changelog/v4_0_2.md`
EOF

new_file docs/19_roadmap/orbital_visualizer_roadmap.md <<'EOF'
# Orbital Visualizer Roadmap

> **DubSar Help** | `Roadmap > Orbital Visualizer` | Roadmap

## Planned Milestones (PA-13 to PA-16 Implementation)

| Phase | Deliverable | Effort |
| :--- | :--- | :--- |
| 1 | δ_T and shell binning in `particles-algebra-core` | ~3 days |
| 2 | `position(p)` function (radius, azimuth, altitude) | ~1 day |
| 3 | PointSprite renderer (1K–10K particles) | ~2 weeks |
| 4 | Instanced renderer (~1M particles) | ~1 week |
| 5 | Volumetric renderer (density compute shader + ray march) | ~3–4 weeks |

**Total to billion-particle visualization: ~6–8 weeks.**

## See Also

- `09_observatory/rendering_modes.md`
- `01_mathematics/orbital_theorems.md`
EOF

# ─── 99_index ──────────────────────────────────────────────────────────────────
new_file docs/99_index/master_glossary.md <<'EOF'
# Master Glossary

> **DubSar Help** | `Index > Glossary` | Index

This file is the machine-readable index of all terms defined across the Codex.
It is generated from `00_codex/glossary.md` and supplemented by terms defined
in per-section files.

See `00_codex/glossary.md` for the authoritative term definitions.
EOF

new_file docs/99_index/cross_reference.md <<'EOF'
# Cross-Reference Index

> **DubSar Help** | `Index > Cross-Reference` | Index

## Pauli Gates → Crates

| Gate | Primary Crate |
| :--- | :--- |
| ADAD | `crates/adad-gate/` |
| ANU | `crates/bahyway-algebra/` |
| MARDUK | `crates/vgca-engine/` |
| SHAMASH | `crates/enkidb-journal/` |

## Theorems → Crates

| Theorem | Primary Crate |
| :--- | :--- |
| PA-13 (Shell decomposition) | `crates/tribe-orbit-engine/` |
| PA-14 (Particle position) | `crates/tribe-orbit-engine/` |
| PA-15 (Tribe birthing) | `crates/tribe-orbit-engine/` |
| PA-16 (Rendering modes) | `crates/dubsar-visualizer/` |

## KAKI Types → Crates

| KAKI Type | Primary Crate |
| :--- | :--- |
| Identity-KAKI | `crates/enkidb-kaki/` |
| Events-KAKI | `crates/enkidb-kaki/` |
| CrossTribe-KAKI | `crates/riksu-engine/` |
EOF

# Put .gitkeep in any remaining empty directories
find docs -type d -empty -exec touch {}/.gitkeep \;

echo ""
echo "=========================================="
echo "  CODEX v2.0 — COMPLETE"
echo "=========================================="
echo "  Location: $(pwd)/docs/"
echo "  Placeholder files seeded in all 23 dirs."
echo "=========================================="
echo ""
echo "DubSar IDE Help pages are ready for content."
echo "Each placeholder follows the standard skeleton:"
echo "  # Title"
echo "  > DubSar Help | \`SyntaxToken\` | Category"
echo "  ## Purpose / ## Mechanism / ## Sovereign Constraints / ## See Also"
