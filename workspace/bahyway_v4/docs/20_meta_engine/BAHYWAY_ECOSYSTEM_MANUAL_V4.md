# BahyWay.Ecosystem v4.0 — Complete Reference Manual
## New Engines, Services & Visual Infrastructure
### Method: W5H2 (What · Why · Where · When · Who · How · How Much)

---

> **Scope of this manual**: Covers every new engine, service, protocol, and component
> introduced in the three research sessions: Visualization Pure Rust Formalization,
> BahyWay Website & Navigation Architecture, and NaviEngine Pattern Intelligence.
> Nothing left unchecked. Everything defined with the W5H2 method.

---

## TABLE OF CONTENTS

1. [Foundation Refresher — Triple-O & KAKI](#1-foundation-refresher)
2. [Pattern-KAKI — The Fourth Type](#2-pattern-kaki)
3. [NAVIENGINE — 7D Navigation Engine](#3-naviengine)
4. [NISABA — Pattern Discovery Service](#4-nisaba)
5. [ENKI-GENESIS — Synthetic Particle Seeder](#5-enki-genesis)
6. [ENKI-PATTERN — Shared Pattern Registry](#6-enki-pattern)
7. [NARUDU-PATTERN — Event Stream Protocol](#7-narudu-pattern)
8. [ŠÀTAMU Interface — Human Stewardship Layer](#8-satamu-interface)
9. [AI Council — Pattern Validation Agents](#9-ai-council)
10. [ENLIL Algebra & 7D→3D Projection](#10-enlil-algebra)
11. [WGPU Render Pipeline — NaviEngine Viewport](#11-wgpu-render-pipeline)
12. [ENKI-VELOCITY — Performance & LOD Engine](#12-enki-velocity)
13. [NARUDU-RENDER — Distributed Rendering Cluster](#13-narudu-render)
14. [StewardLens — Mobile Stewardship Interface](#14-stewardlens-mobile)
15. [BeeMDM ETL — 7 Hepta Gates Visualization](#15-beemdm-etl-visualization)
16. [Nergal AV Dashboard — Threat Intelligence](#16-nergal-av-dashboard)
17. [BIG RING — Orbital Particle System](#17-big-ring)
18. [BahyWay Website Architecture](#18-bahyway-website)
19. [OrbitalClock — Universal Time System](#19-orbitalclock)
20. [Nash Equilibrium Layer](#20-nash-equilibrium)
21. [Complete Glossary — W5H2 Method](#21-complete-glossary)
22. [Shala4 — Gate Orbits & Playbook Dependency Discovery](#22-shala4-gate-orbits)

---

## 1. Foundation Refresher

### Triple-O (Orbit-Oriented Ontology)

| W5H2 | Answer |
|------|--------|
| **What** | The philosophical and engineering framework of BahyWay.Ecosystem. Everything is a Particle. Relationships are orbital, not hierarchical. |
| **Why** | To replace object-oriented thinking with a model that is relational, dynamic, and contextual — where meaning comes from orbit, not class. |
| **Where** | Embedded in every crate via `#![forbid(unsafe_code)]` and KAKI-first design. |
| **When** | Enforced from the moment any entity enters the system — birth = KAKI assignment. |
| **Who** | All participants: entities, services, events, query plans, AI agents — everything is a particle. |
| **How** | Three layers: Ontology (what is it?), Orbital (when/where is it?), Observation (how do we verify it?). |
| **How Much** | No limit. Every byte in the system that has identity has a KAKI. |

### KAKI — 16-byte Universal Identity Key

```
Layout: [type_prefix:2bits][entropy:110bits][checksum:16bits]
```

| Type | Prefix | Meaning |
|------|--------|---------|
| `identity-kaki` | `0b00` / `0x01` KISHIB | Individual entity (person, cargo, sensor, service) |
| `events-kaki` | `0b01` / `0x02` ZIKRU | Occurrence at a point in spacetime |
| `crosstribe-kaki` | `0b10` / `0x03` | Interaction boundary between tribes |
| `pattern-kaki` | `0b11` | Emergent structure from GA clustering *(new)* |

**Immutability law**: KAKI is generated ONCE at birth. Never changed. Never reused.
**ColorID law**: Color is stored in EAV, never in KAKI. ADR-001 sovereign constraint.

---

## 2. Pattern-KAKI — The Fourth Type

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | A new KAKI type (prefix `0b11`) representing an emergent structure discovered by GA (Genetic Algorithm) clustering of many individual particles. Not an individual. Not an event. A *pattern*. |
| **Why** | Aviation corridors, indoor hallways, crowd flows, and water channels are not single particles — they are emergent orbital patterns. They need identity (KAKI), lifecycle, and governance. |
| **Where** | Stored in `ENKI-PATTERN` registry. Indexed in HeptaMAP 7D spatial index. Referenced by NAVIENGINE for route planning. |
| **When** | Created when GA clustering finds a high-confidence cluster with `validation_count >= 3` and `tiamat_level <= KAKKAB`. |
| **Who** | Created by NISABA (Pattern Discovery). Validated by AI Council (TamuzAI/NINSUN/PAZUZU) and human DataSteward via Šàtamu. Used by NAVIENGINE. |
| **How** | Derived deterministically from: GA centroid (7D) + pattern type + constituent Merkle root + confidence + orbital_formed. |
| **How Much** | Memory footprint per pattern-kaki: ~512 bytes (EAV mandatory attributes). |

### Pattern Lifecycle

```
emerging → stable → canonical → deprecated
              ↓                      ↑
         (insufficient            (superseded
          validation)              by new pattern)
```

### Pattern Types

| Code | Name | Domain |
|------|------|--------|
| `0x01` | `CrowdFlow` | NISABA crowd tracking |
| `0x02` | `AviationCorridor` | NAVIENGINE air |
| `0x03` | `AviationHolding` | NAVIENGINE holding patterns |
| `0x04` | `IndoorHallway` | NAVIENGINE ground interior |
| `0x05` | `IndoorRoom` | NAVIENGINE ground interior |
| `0x06` | `IndoorTransition` | Stairwells, elevators, bridges |
| `0x07` | `WaterFlow` | NANSHE/ABZU maritime |
| `0xFF` | `Custom` | Client-defined |

### Mandatory EAV Attributes

| Attribute | Type | Role |
|-----------|------|------|
| `pattern_kaki` | `Kaki` | Self-reference |
| `constituent_count` | `u64` | How many particles formed this |
| `constituent_kakis_hash` | `[u8; 32]` | Merkle root for lineage integrity |
| `ga_centroid` | `FixedCoord7D` | Cluster geometric center |
| `ga_variance` | `[f64; 7]` | Per-dimension variance |
| `confidence` | `u64` | Fixed-point 0–10000 |
| `pattern_type` | `u8` | Enum discriminant |
| `orbital_formed` | `u64` | Birth orbital |
| `orbital_last_validated` | `u64` | Recency |
| `validation_count` | `u64` | Times confirmed |
| `state` | `u8` | emerging/stable/canonical/deprecated |
| `parent_pattern_kaki` | `Option<Kaki>` | Lineage — what this superseded |
| `child_pattern_kakis` | `[Kaki; 8]` | What superseded this |
| `nash_equilibrium_score` | `u64` | Fixed-point Nash stability |
| `tiamat_level` | `u8` | GREEN/DILBAT/KAKKAB/NERGAL/MAROON |
| `simulation_template_kaki` | `Option<Kaki>` | Link to generated simulation |

### Canonicalization Rules

A pattern-kaki is **canonical** when ALL of:
- `state == Canonical`
- `validation_count >= 3`
- `tiamat_level <= KAKKAB (2)`

A canonical pattern-kaki may be stored as a simulation template in EnkiMDB.

---

## 3. NAVIENGINE — 7D Navigation Engine

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The sovereign navigation engine operating in 7D space. Navigates outdoor (GPS), indoor (building interiors), and airspace (aviation corridors) in a unified coordinate system. |
| **Why** | Conventional GPS navigation is 3D and stops at building walls. NAVIENGINE extends navigation into building interiors, multiple floors, semantic zones, and temporal predictions — all within the 7D KAKI space. |
| **Where** | Crate: `crates/navi-engine`. Connects to: NISABA (patterns), ENKI-PATTERN (registry), Šàtamu (human override), WGPU viewport. |
| **When** | Active at all times. Particle movements trigger NAVIENGINE route queries in real-time. |
| **Who** | Used by: end-user navigation apps, emergency services, logistics operators, aviation managers, building operators. |
| **How** | Pattern-based: instead of building maps from scratch, NAVIENGINE subscribes to canonical patterns from ENKI-PATTERN registry. Routes are *simulations chosen from pattern-based proposals*. |
| **How Much** | Handles: outdoor (OSM planet, 500M edges via Contraction Hierarchies), indoor (per-building pattern graphs), aviation (10K+ concurrent flights). |

### 7 Dimensions in NAVIENGINE

| Dimension | Conventional | NAVIENGINE Extension |
|-----------|-------------|----------------------|
| d1: X | Longitude | Sub-meter precision |
| d2: Y | Latitude | Sub-meter precision |
| d3: Z | Elevation | Full volumetric height (floors, shafts, ceilings) |
| d4: T | Timestamp | Real-time + predictive orbital |
| d5 | — | Stakeholder view layer (transparency) |
| d6 | — | Building interior topology (rooms, corridors, infrastructure) |
| d7 | — | Semantic/functional + state/history dimension |

### Navigation Modes

| Mode | Domain | Algorithm | Data Source |
|------|--------|-----------|------------|
| Outdoor | Street-level | A* → Contraction Hierarchies | OSM .osm.pbf |
| Indoor | Building interior | Pattern graph traversal | ENKI-GENESIS synthetic + SLAM observed |
| Aviation | Airspace | Pattern-corridor routing | Canonical aviation patterns |
| Multi-domain | All unified | Cross-domain pattern fusion | ENKI-PATTERN with cross-tribe subscription |

### Routing Algorithm Roadmap

| Phase | Algorithm | Scale | Query Time |
|-------|-----------|-------|------------|
| 1 | Dijkstra | City | <100ms |
| 2 | A* with Haversine | Country | <10ms |
| 3 | A* with elevation | Regional | <50ms |
| 4 | Contraction Hierarchies | Planet | <100ms |

**Contraction Hierarchies preprocessing**: days for planet. Once built, queries in milliseconds. Store preprocessed graph as KAKI-addressed particles in cold storage.

---

## 4. NISABA — Pattern Discovery Service

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The sovereign pattern discovery engine. Monitors streams of identity-kaki and events-kaki, applies GA clustering, and mints pattern-kaki when emergent structures are found. Named after the Sumerian goddess of writing and record-keeping. |
| **Why** | Instead of hard-coding corridors and routes, NISABA discovers them from actual particle trajectories. This enables indoor navigation without floor plans and aviation routing without static charts. |
| **Where** | Crate: `crates/nisaba` (to be created). Feeds into: ENKI-PATTERN registry. Receives from: BeeMDM ETL pipeline, sensor streams, synthetic agent simulations. |
| **When** | Continuous. Runs at BASE orbital frequency (1 minute). Nash equilibrium is computed at MACRO frequency (1 hour). |
| **Who** | Internal service only. No direct external API. Publishes canonical patterns to ENKI-PATTERN. Receives validation from AI Council and Šàtamu. |
| **How** | GA clustering on 7D trajectory data → grid-based density counting → high-density region extraction → pattern-kaki minting → BeeMDM pipeline → GOLD. |
| **How Much** | Monte Carlo: 1000 iterations × 100 agents × 1000 steps. Trajectory points stored in cold tier. Pattern-kakis are hot tier. |

### NISABA → NAVIENGINE Protocol

```
NISABA                              NAVIENGINE
  │                                      │
  ├─ Pattern GOLD (canonical) ─────────► │  publish_pattern()
  │                                      │
  │ ◄─ subscribe_region(region, types) ──┤  subscribe_region()
  │                                      │
  ├─ Nash state update ────────────────► │  push_nash_update()
  │                                      │
  │ ◄─ query_simulation(scenario) ───────┤  query_simulation()
  │                                      │
  ├─ SIMULATION_COMPLETE ──────────────► │  (SimulationTicket resolved)
  │                                      │
  │ ◄─ pattern_validation decision ──────┤  (Šàtamu/DataSteward)
  │                                      │
  ├─ PATTERN_DEPRECATED ───────────────► │  (route invalidation)
```

---

## 5. ENKI-GENESIS — Synthetic Particle Seeder

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | A Monte Carlo simulation engine that generates synthetic particle trajectories for buildings that have no prior observation data. Seeds NISABA with artificial patterns before real particles arrive. |
| **Why** | New buildings have no trajectory history. Indoor navigation cannot wait months for real occupants to walk corridors. ENKI-GENESIS produces plausible patterns from a minimal architectural hint (building outline + entrance/exit points + intended function). |
| **Where** | Called before NAVIENGINE is deployed for a new building. Outputs synthetic patterns that enter BeeMDM pipeline with `is_synthetic = true` flag. |
| **When** | One-shot: run when a new building is registered. Re-run when building layout changes significantly. |
| **Who** | Building operator or DataSteward triggers it. AI validates synthetic-vs-real convergence via Šàtamu `SyntheticValidationBoard`. |
| **How** | 1. Parse `ArchitecturalHint` (boundary polygon, obstacles, entrances, exits, function). 2. Spawn 100 `SyntheticAgent` particles per iteration. 3. Apply Social Force model for 1000 steps. 4. Extract high-density grid regions. 5. Mint pattern-kaki candidates. |
| **How Much** | 1000 iterations × 100 agents × 1000 steps = 10M simulated trajectories. Grid: 100×100. Memory: bounded by `Vec::reserve(10_000_000.min(estimated))`. |

### Social Force Parameters

| Parameter | Default | Meaning |
|-----------|---------|---------|
| `attraction_coeff` | 2.0 | Pull toward target exit |
| `repulsion_coeff` | 5.0 | Push away from other agents |
| `wall_repulsion_coeff` | 10.0 | Strong push from walls |
| `neighbor_radius` | 2.0m | Distance for agent-agent interaction |
| `wall_radius` | 1.0m | Distance for wall interaction |
| `time_step` | 0.01s | Simulation granularity |
| `damping` | 0.95 | Velocity damping per step |

### Building Functions

| Code | Function | Flow Pattern |
|------|---------|-------------|
| `0x01` | Office | Bidirectional (corridors) |
| `0x02` | Retail | Mesh (open plan) |
| `0x03` | Transit | Unidirectional (security lanes) |
| `0x04` | Residential | Bidirectional (hallways) |
| `0x05` | Industrial | Custom |
| `0x06` | Worship | Radial (center↔periphery, Hajj application) |

### Synthetic-to-Real Transition

When real particles begin moving through the building:
1. NISABA measures actual trajectory density
2. Compares to synthetic pattern centroids
3. If `match_score >= 0.85` → synthetic pattern confirmed → `validation_count++`
4. If `match_score < 0.5` → synthetic pattern deprecated → new real-data pattern minted
5. Šàtamu `SyntheticValidationBoard` reviews all transitions

---

## 6. ENKI-PATTERN — Shared Pattern Registry

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The sovereign registry that stores, indexes, and serves canonical pattern-kakis across all domains (crowd, indoor, aviation, maritime). The single source of truth for pattern data. |
| **Why** | NISABA and NAVIENGINE must share patterns without coupling. Multiple subscribers need the same patterns simultaneously. Hot/warm/cold tiering ensures performance at scale. |
| **Where** | Hot tier: `HashMap<Kaki, PatternEntry>` in-memory. Warm tier: `SimpleLruCache`. Cold tier: `HeptaMapFixed<PatternSpatialKey, Kaki>`. Crystallized: `BodyTierManager`. |
| **When** | Patterns enter hot tier when NISABA publishes them as canonical. Move to warm after low access frequency. Move to cold after `crystallization_threshold` orbitals. |
| **Who** | NISABA writes. NAVIENGINE reads (via subscriptions). DataSteward modifies via Šàtamu. AI Council validates. |
| **How** | HeptaMAP 7D spatial index for region queries. LFU eviction for hot→warm transitions. Subscription model for real-time push notifications. |
| **How Much** | Hot tier: ~10K canonical patterns (all domains). Warm tier: 256 cached entries. Cold/crystallized: unbounded, queried on demand. |

### API Surface

| Method | Caller | Effect |
|--------|--------|--------|
| `publish_pattern()` | NISABA | Insert canonical pattern into hot tier + HeptaMAP |
| `subscribe_region()` | NAVIENGINE | Register for patterns in E7Region + get active patterns |
| `push_nash_update()` | NISABA | Update Nash state, notify subscribers if significant |
| `query_simulation()` | NAVIENGINE | Create SimulationTicket from active patterns + scenario |

### Significance Threshold for Nash Updates

A Nash state change is **significant** (triggers subscriber notification) when any of:
- Equilibrium delta > 10%
- Anarchy score crosses 0.5 threshold
- Deviation count doubles in one orbital

---

## 7. NARUDU-PATTERN — Event Stream Protocol

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The event bus that carries pattern lifecycle events between NISABA, NAVIENGINE, Šàtamu, and the NARUDU journal. Every pattern event is immutable, auditable, and journaled. |
| **Why** | Decouples producers (NISABA) from consumers (NAVIENGINE, DataSteward). Enables replay, audit, forensics. All pattern decisions become particles in the Trial Audit Journal. |
| **Where** | Hot buffer: `Vec<PatternEvent>` (max 10,000 events). Warm log: `EventLog` (persistent, queryable). Crystallized: `BodyTierManager`. |
| **When** | Every pattern publication, Nash shift, deprecation, simulation completion, and steward decision emits an event. |
| **Who** | NISABA publishes. NAVIENGINE subscribes. Šàtamu reads history. NARUDU journal receives everything. |
| **How** | Ring buffer in hot tier. Domain-specific routing. Filter by: PatternType, E7Region, NashThreshold, ConfidenceThreshold, Domain. |
| **How Much** | Hot buffer auto-drains when >10K entries (oldest 5K move to warm log). No data is ever lost — crystallized events are retained forever. |

### Event Types

| Event | Trigger | Subscribers Notified |
|-------|---------|---------------------|
| `Published` | NISABA mints canonical pattern | All regional subscribers |
| `NashShift` | Equilibrium changes significantly | Subscribers to that pattern |
| `Deprecated` | Pattern superseded | All subscribers + NAVIENGINE route invalidation |
| `SimulationComplete` | ENKI-GENESIS or simulation finishes | Requesting NAVIENGINE |
| `StewardDecision` | Human/AI acts on pattern | Journal + affected subscribers |

---

## 8. ŠÀTAMU Interface — Human Stewardship Layer

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The human-in-the-loop governance layer for pattern canonicalization, Nash overrides, and emergency rerouting. Named after the Akkadian term for a temple administrator / royal steward. |
| **Why** | AI agents and Nash equilibrium are not infallible. Humans must retain override authority for emergencies, regulatory requirements, and edge cases that statistics miss. Šàtamu binds human judgment to the KAKI ontology so all decisions are auditable. |
| **Where** | Three actor groups: `PatternCanonizationCouncil` (reviews emerging→canonical), `NashOverridePanel` (emergency reroute), `SyntheticValidationBoard` (confirms synthetic seeding). |
| **When** | Triggered by: AI Council split vote, TiamatLevel escalation to NERGAL/MAROON, pattern confidence drop, DataSteward manual review, emergency override. |
| **Who** | Human DataSteward (with clearance level 1–5). AI Council provides recommendations. StewardLens watches for anomalies. Architect KAKI seals all audit records. |
| **How** | `SatamuRecord` state machine. Every decision is a `StewardDecisionEntry` with: actor, action, justification, Ed25519 signature, orbital. Append-only. |
| **How Much** | Each `SatamuRecord` grows with decisions. Max decision chain depth is bounded by orbital retention policy (crystallized after 365 orbitals). |

### Decision Action Types

| Action | Category | Authority Required |
|--------|----------|-------------------|
| `Classify` | Ontology | Level 2 |
| `Reclassify` | Ontology | Level 3 |
| `Merge` | Ontology | Level 3 |
| `Validate` | Orbital | Level 1 |
| `Invalidate` | Orbital | Level 2 |
| `Extend` | Orbital | Level 1 |
| `Confirm` | Observation | Level 1 |
| `Reject` | Observation | Level 2 |
| `Quarantine` | Observation | Level 3 |
| `Override` | Emergency | Level 4 |
| `Deprecate` | Lifecycle | Level 4 |

### Emergency Override Protocol

```heptascript
dubsar KAKI:steward_01 {
    who: human { clearance: 5, biometric: attested, override_authority: true },
    what: steward {
        target_pattern: KAKI:pattern_7d4f...,
        action: override {
            type: emergency_reroute,
            duration: orbital[24_hours],
            cascade: true  -- propagate to child patterns
        },
        validation: { visual_confirm: true, ai_council: bypass }
    },
    how: { tier: hot, execution: immediate },
    how_much: { timeout_ms: 0, human_confirm: single_steward }
}
```

### Steward Lock Protocol

- Only one steward can modify a `SatamuRecord` at a time
- `steward_lock: Option<Kaki>` holds the locking steward's identity-kaki
- Lock auto-releases after 5 BASE orbitals if steward goes offline
- Emergency overrides bypass lock (Level 4+ only)

---

## 9. AI Council — Pattern Validation Agents

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | Three specialized AI agents (TamuzAI, NINSUN, PAZUZU) that evaluate pattern-kakis, vote on canonicalization, and provide recommendations to human DataSteward. Governed by a consensus protocol requiring 75% agreement. |
| **Why** | No single AI agent should control pattern governance. Multi-agent deliberation with quorum requirements prevents AI capture of the pattern space. |
| **Where** | Part of Šàtamu. Agents run in `crates/agent-council` (to be created). Receive pattern data. Send votes to AICouncil supervisor. |
| **When** | Triggered whenever a pattern reaches `stable` state or when Nash equilibrium shifts significantly. Also triggered by Šàtamu before any human override executes. |
| **Who** | TamuzAI = pattern quality scorer. NINSUN = cross-domain consistency checker. PAZUZU = anomaly detector / quarantine enforcer. |
| **How** | Phase 1: independent evaluation (each agent scores independently). Phase 2: deliberation (if entropy > 0.3, agents review each other's votes and may revise). Phase 3: consensus check (dominant action must have ≥ 75% weighted confidence). |
| **How Much** | 3 agents minimum. Quorum = 2 of 3. Escalation to human when split_metric > 0.3 and no consensus after deliberation. |

### Agent Roles

| Agent | Name Origin | Function |
|-------|-------------|---------|
| **TamuzAI** | Tammuz — Sumerian fertility god | Evaluates pattern quality: confidence, validation history, constituent count, variance stability |
| **NINSUN** | Sumerian goddess, mother of Gilgamesh | Validates cross-domain consistency: does indoor pattern align with crowd flow? Does aviation corridor respect ground clearance? |
| **PAZUZU** | Akkadian demon of the wind | Detects anomalies: sudden confidence drop, Nash equilibrium spike, synthetic patterns deviating from real observations, suspicious mass-deprecation |

### Council Protocol

```
Phase 1: INDEPENDENT EVALUATION
   TamuzAI.evaluate(pattern) → AgentVote
   NINSUN.evaluate(pattern)  → AgentVote
   PAZUZU.evaluate(pattern)  → AgentVote

Phase 2: DELIBERATION (if entropy > 0.3)
   Each agent sees peer votes → may revise

Phase 3: CONSENSUS CHECK
   agreement = dominant_confidence / total_confidence
   if agreement >= 0.75 → CouncilResult::Consensus
   else               → CouncilResult::Split → escalate to human
```

---

## 10. ENLIL Algebra & 7D→3D Projection

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The mathematical framework for projecting 7D particle coordinates to 3D screen space for rendering. Spatial dimensions (x,y,z) map directly. Semantic dimensions (d4,d5,d6) map to visual properties. Time (d7/t) maps to animation. |
| **Why** | 7D data cannot be rendered on a 3D screen without a semantic-preserving projection. ENLIL Algebra ensures that particles in similar orbits cluster visually and particles in different orbits separate — the projection preserves orbital relationships, not just spatial position. |
| **Where** | WGSL compute shader `enlil_projection.wgsl`. Shared uniform buffer `EnlilProjectionParams` across all render layers. |
| **When** | Every frame. The view matrix updates on camera movement. Time scale updates on animation tick. |
| **Who** | Used by all WGPU render layers: crowd (Layer 1), indoor (Layer 2), aviation (Layer 3). |
| **How** | `pos3d.(x,y,z)` = direct 3D position. `t` = z-offset (ghost trail). `d4` = particle size. `d5` = shape factor (heptagonal symmetry). `d6` = glow intensity. State = color modulation. KAKI type = special rendering (pattern-kaki gets confidence-based glow). |
| **How Much** | Uniform buffer: 256 bytes. Processed per particle per vertex shader invocation. On GPU, runs in parallel for all particles. |

### Dimension Mapping

| Dimension | 7D Name | 3D/Visual Mapping |
|-----------|---------|-------------------|
| d1 | X | `pos3d.x` — direct spatial |
| d2 | Y | `pos3d.y` — direct spatial |
| d3 | Z | `pos3d.z` — direct spatial |
| d4 | T (time) | `z += t * time_scale * 0.1` (ghost trail) |
| d5 | Semantic 1 | `size = base_size * (1 + d5 * scale)` |
| d6 | Semantic 2 | `shape_factor` — heptagonal symmetry coefficient |
| d7 | Semantic 3 | `glow_intensity` — emissive brightness |

### State Color Table

| State | Color | WGSL |
|-------|-------|------|
| alive (0) | Dim tribe color | `color * 0.8` |
| fuzzy (1) | Amber pulse | `vec4(1.0, 0.5, 0.0, 1.0)` |
| aged (2) | Stable tribe | `color * 1.0` |
| decay (3) | Fading | `color * 0.6` |
| dead (4) | Gray | `vec4(0.5, 0.5, 0.5, 0.5)` |
| hidden (5) | Purple veil | `vec4(0.7, 0.0, 0.7, 0.5)` |
| GOLD (6) | Shining gold | `vec4(1.0, 0.84, 0.0, 1.0)` + `glow *= 2.0` |

### Heptagonal Shape (7-fold symmetry)

```wgsl
fn heptagonal_shape(angle: f32, shape_factor: f32) -> f32 {
    return abs(cos(angle * 7.0)) * shape_factor;
}
// Used in fragment shader to create 7-pointed orbital probability cloud
// reflecting the 7D origin of every particle
```

### E7 Lattice & Viazovska Constraint

- Fanout limit for HeptaMAP queries: **126** (Viazovska E7 kissing number)
- This means no single query can touch more than 126 neighboring particles in HeptaMAP
- Aviation corridor fanout: `126 = 7 × 18` (heptagonal × orbital resonance)
- This is a hard architectural constraint, not a configuration option

---

## 11. WGPU Render Pipeline — NaviEngine Viewport

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The multi-layer WGPU rendering system that displays crowd flows, indoor corridors, aviation tubes, Nash heatmaps, and Šàtamu overlays in a single unified viewport. |
| **Why** | Transparency is the BahyWay motto. All stakeholders must see what is happening in 7D space — crowd pressure, indoor routes, airspace corridors, equilibrium stability — simultaneously and in real-time. |
| **Where** | `crates/dubsar-visualizer` — `NaviEngineViewport` struct. Backed by `wgpu` crate, `winit` for windowing, `glam` for math. |
| **When** | Running continuously during active navigation sessions. Updates every frame (target: 60 FPS on laptop GPU). |
| **Who** | Primary users: human DataStewards (via Šàtamu), building operators, aviation managers, emergency coordinators. |
| **How** | 6-layer compositing pipeline. Pass 1: render each layer to offscreen texture. Pass 2: composite all layers to screen. Pass 3: pixel picker for mouse interaction. |
| **How Much** | 6 layers × 1 offscreen texture each. Max particles per layer: Crowd=1M, Indoor=100K corridors, Aviation=10K tubes, Nash=1920×1080 heatmap, Šàtamu=1K billboards. |

### Layer Stack

| Layer | Domain | Geometry | Blend Mode | Max Instances |
|-------|--------|----------|------------|---------------|
| 0 | Base map | 2D polygons | Opaque | — |
| 1 | Crowd | Point sprites | Additive | 1,000,000 |
| 2 | Indoor | Line strips + triangle fans | Alpha | 100,000 |
| 3 | Aviation | 3D tube segments | Alpha | 10,000 |
| 4 | Nash heatmap | Fullscreen quad | Multiplicative | — |
| 5 | Šàtamu | Billboard quads | Overlay | 1,000 |

### Multi-Pass Architecture

```
Pass 1: Layer Rendering (one pass per visible layer)
   → Each layer renders to dedicated offscreen texture
   → Layer 0 clears with BLACK; subsequent layers use Load

Pass 2: Compositing (one fullscreen pass)
   → Blend all layer textures by opacity and blend mode
   → Result to swapchain surface texture

Pass 3: Pixel Picker
   → Separate render with KAKI IDs encoded in pixel color
   → On mouse click: read pixel → extract KAKI → query pattern registry
```

### Pixel Picker (Mouse Interaction)

The pixel picker renders KAKI bytes directly into a `Rgba32Uint` texture. When a stakeholder clicks on a particle in the viewport, the system:
1. Reads `PickedPixel` at (x, y) from `picker_buffer`
2. Extracts `kaki_bytes: [u8; 16]` — the exact KAKI of the clicked particle
3. Queries `PatternRegistry` for full details
4. Opens Šàtamu inspection panel or pattern detail view

This is the **transparency mechanism** — any stakeholder can click any particle and see its full KAKI, state, tribal affiliation, audit trail, and current Nash position.

---

## 12. ENKI-VELOCITY — Performance & LOD Engine

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The adaptive Level-of-Detail (LOD) and orbital culling system that maintains 60 FPS regardless of particle count by assigning particles to performance tiers based on orbital age and camera distance. |
| **Why** | 1M crowd particles + 100K corridors + 10K aviation tubes cannot all render at full fidelity simultaneously. LOD is the engineering necessity that makes global-scale visualization possible on consumer hardware. |
| **Where** | Integrated into `NaviEngineViewport`. `VelocityEngine` classifies each particle before rendering. Shader variants are selected per tier. |
| **When** | Every frame. Classification recalculated when: camera moves significantly, orbital clock advances, or particle state changes. |
| **Who** | Transparent to stakeholders. Operates automatically. DataSteward can force `Hot` tier for a region being actively reviewed. |
| **How** | `classify_particle()` → `PerformanceTier`. `select_shader_variant()` → different WGSL entry points with `#define` flags. `adapt_budget()` → adjusts particle count if FPS drops below 80% of target. |
| **How Much** | Reduces render cost by 60–80% at planetary scale by replacing distant individual particles with aggregate blobs. |

### Performance Tier Definitions

| Tier | Orbital Age | Distance | Shader Features | Steward |
|------|------------|---------|-----------------|---------|
| **HOT** | 0–3 orbitals | <1000 units | Full 7D, heptagonal shape, Nash live, glow full | Full deliberation |
| **WARM** | 3–30 orbitals | <10K units | 3D spatial, time baked, circular shape, Nash cached | AI-only |
| **COLD** | 30–365 orbitals | <100K units | 2D + altitude bucket, flat color, Nash historical | Crystallized |
| **CRYSTALLIZED** | 365+ orbitals | Any | Pattern-kaki blobs only, no individuals, query-only | Archived |

### Adaptive Budget

```
FPS < 80% target → particle_budget *= 0.9 → compute_budget_ms *= 0.9
FPS > 120% target → particle_budget *= 1.05 → compute_budget_ms *= 1.05
particle_budget clamped to [1000, max_hot_particles]
```

---

## 13. NARUDU-RENDER — Distributed Rendering Cluster

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | A distributed GPU cluster where one master node and N worker nodes split the viewport into tiles, each rendered independently, then composited into the final frame. |
| **Why** | Hajj-scale crowd (2M+ particles), global aviation (10K+ flights), multi-city indoor (1000+ buildings) cannot fit on a single GPU's VRAM. Distribution is the only path to global-scale real-time visualization. |
| **Where** | `RenderNode` struct with `NodeRole::Master` or `NodeRole::Worker`. Communication via `NetworkTransport` (ZeroMQ or gRPC). |
| **When** | Activated when total particle count exceeds single-GPU capacity threshold. Auto-scales workers on load. |
| **Who** | System-level infrastructure. Transparent to DataSteward and end-users. |
| **How** | Master distributes `AssignTile` messages. Workers render their tile to offscreen texture. Workers return `TileComplete` with compressed texture + depth + picker data. Master composites in depth order. |
| **How Much** | Tested for: 3 GPU nodes minimum, 32 nodes maximum. Tile size: 240×135 pixels (1920×1080 ÷ 8×8 grid). Network latency budget: 2ms per frame (must fit in 16ms total for 60 FPS). |

### Message Protocol

| Message | Direction | Contains |
|---------|-----------|---------|
| `AssignTile` | Master→Worker | tile_id, viewport_region, layer_mask, quality_tier, ENLIL params, orbital_clock |
| `SyncNashState` | Master→Worker | pattern_updates, global_equilibrium, anarchy_regions |
| `SyncSatamu` | Master→Worker | active_overrides, quarantine_regions, AI council decisions |
| `TileComplete` | Worker→Master | tile_id, render_time_ms, texture/depth/picker data |
| `NashContribution` | Worker→Master | local_patterns, deviation_events |
| `Heartbeat` | Bidirectional | node_id, load_factor, memory_pressure, temperature |
| `Emergency` | Bidirectional | code, affected_tiles, override_action |

### Shared State (Consistency Rules)

| State | Owner | Consistency |
|-------|-------|-------------|
| ENLIL view matrix | Master broadcasts | All workers use identical projection |
| Orbital clock | Master broadcasts | Workers sync within 1ms |
| Nash state | Master computes | Workers cache, refresh on `SyncNashState` |
| Šàtamu decisions | Master holds lock | Workers receive copy-on-change |

---

## 14. StewardLens — Mobile Stewardship Interface

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The mobile interface that brings Šàtamu decision-making to DataStewards on smartwatch, smartphone, tablet, and desktop — with tier-appropriate capability at each device level. |
| **Why** | Human stewards cannot be tied to workstations. Emergencies happen at 3am. A Šàtamu override must be possible from a smartwatch vibration + 2-button confirm without unlocking a laptop. |
| **Where** | Mobile app (`crates/bahyway-web` or dedicated mobile crate). Connects to NARUDU-PATTERN event stream and ENKI-PATTERN registry. |
| **When** | Always-on for emergency alerts. Full viewport available on demand. |
| **Who** | Human DataStewards at all clearance levels (1–5). Higher clearance = more device tiers available. |
| **How** | Four device tiers with progressively richer capability. Biometric attestation (FaceID/fingerprint) required for Level 3+ actions. Camera AR overlays patterns on real-world view (smartphone/tablet). |
| **How Much** | Smartwatch: 1-line status + 2 buttons. Smartphone: simplified WGPU at 30 FPS. Tablet: full WGPU at 60 FPS + multi-panel. Desktop: multi-monitor command center. |

### Device Tier Capabilities

| Tier | Device | Viewport | Actions | Biometric |
|------|--------|----------|---------|-----------|
| 1 | Smartwatch | None (haptic only) | Confirm/Reject AI recommendation | None |
| 2 | Smartphone | Simplified 30 FPS | Quarantine, Validate, Quick Override, AR camera | Fingerprint |
| 3 | Tablet | Full 60 FPS, multi-panel | Full Šàtamu + AI Council viewer + orbital scrubbing | FaceID |
| 4 | Desktop | Multi-monitor | HeptaScript editor + all Šàtamu + scripting | Hardware token |

### Alert Escalation

```
TIAMAT GREEN  → No alert (monitoring)
TIAMAT DILBAT → Smartphone notification
TIAMAT KAKKAB → Smartwatch vibrate (3 short pulses)
TIAMAT NERGAL → Smartwatch vibrate (long + strong) + phone call
TIAMAT MAROON → All devices simultaneously + emergency siren
```

---

## 15. BeeMDM ETL — 7 Hepta Gates Visualization

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The 3D visualization of the BeeMDM ETL pipeline showing particles flowing from the orbital ring into 7 processing gate sectors (APSU through ENLIL), stacking as colored cubes by quality, and either promoting to the Sun Nucleus (GOLD) or sinking to Irkalla (quarantine). |
| **Why** | Transparency. All stakeholders can see the live state of every particle in the ETL pipeline: which gate it is at, its quality (bad/fair/good), how long it has been waiting, and whether it is at risk. |
| **Where** | `crates/dubsar-visualizer` — `DashboardEngine` in BeeMDM mode. Uses `HeptaGate` enum, `GateSector` geometry, `CubeStackGeometry`. |
| **When** | Live during ETL processing. Can be replayed historically via orbital scrubbing. |
| **Who** | Seen by: DataStewards, tribe administrators, system architects, client-facing reports (via Šàtamu visual layer). |
| **How** | 50,000 particles orbit the BIG RING. Particles "drop" from the ring into gate sectors at rate 0.04–3.0 per frame. Stack as 3×3×N cubes colored by quality. Top bad layer = bright red emissive. Promoting particles rise to gold Sun Nucleus. |
| **How Much** | Ring capacity: 50,000 particles. Gate sectors: 7, each at radius 50 units, 9×N cube capacity. Nucleus grows with each mastered particle (scale = 1.0 + count × 0.0005). |

### Hepta Gate Mapping

| Gate | Sumerian Name | Function | Color |
|------|---------------|---------|-------|
| Gate 1 | **APSU** | Ingestion — raw data intake | `#00CCFF` cyan |
| Gate 2 | **ADAD** | Validation — schema, types | `#FFAA00` amber |
| Gate 3 | **SHEDU** | Enrichment — context augmentation | `#FF4488` pink |
| Gate 4 | **MUMMU** | Transformation — format conversion | `#FFFF00` yellow |
| Gate 5 | **ENKIDU** | Federation — cross-tribe merge | `#44FF88` green |
| Gate 6 | **DUBSAR** | Indexing — spatial/temporal | `#FF8800` orange |
| Gate 7 | **ENLIL** | Mastering — canonical record | `#AAAAF` lavender |

### Cube Quality Colors

| Quality | Color | State |
|---------|-------|-------|
| Bad (top layer) | `#FF0044` bright red emissive | Exception: requires remediation |
| Bad (lower) | `#CC2200` deep red | Exception: waiting |
| Fair | `#0088FF` blue | StagedFair: processing |
| Good | `#00FF88` green | StagedGood: ready to promote |
| Jailed | `#800020` dark purple | Quarantined: heading to Irkalla |
| Promoting | `#00FF88` green glowing | Moving to Sun Nucleus |

### Geometry Specifications

| Element | Specification |
|---------|---------------|
| Gate sector radius from center | 50 units |
| Cylinder height | 20 units |
| Cylinder radius | 9 units |
| Ring (torus) major radius | 9.5 units |
| Cube size | 2.4 units |
| Cube spacing | 2.6 units (slight gap) |
| Cubes per layer | 9 (3×3 grid) |
| Stack start Y position | -40 units |

---

## 16. Nergal AV Dashboard — Threat Intelligence

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | A 3D visualization of the sovereign antivirus/threat intelligence system. Shows 7 threat categories as sectors (mirroring the 7 Hepta Gates), with threats arriving from the orbital ring, being analyzed, jailed in Irkalla, or neutralized to the Sun Nucleus. |
| **Why** | The same Triple-O architecture that governs data quality (BeeMDM) also governs cybersecurity. Threats are particles. Quarantine is Irkalla. Neutralization is GOLD. Visualizing both pipelines in the same framework demonstrates architectural unity. |
| **Where** | `crates/dubsar-visualizer` — `DashboardEngine` in Nergal mode. Shares geometry (7 sectors, ring, nucleus) with BeeMDM mode but uses threat-specific shaders and colors. |
| **When** | Live during active threat scanning. Threat signatures arrive from external feeds (MalwareBazaar, ClamAV, YARA Community). |
| **Who** | Security operations center. DataStewards with cybersecurity clearance. System architects. |
| **How** | Irkalla nucleus replaces Sun Nucleus for quarantined threats (dark wireframe sphere, `#800020`). Threat categories occupy 7 gate sectors. Threat level affects particle size. Scan pulse animation shows active scanning. |
| **How Much** | 7 threat categories × N threats each. Irkalla capacity: 100,000 jailed threats. Auto-expunge after configured orbitals if not reviewed. |

### 7 Threat Categories

| Sector | Category | Color | Short |
|--------|----------|-------|-------|
| 0 | Computer Viruses | `#CC2244` | Viruses |
| 1 | Ransomware | `#FF4400` | Ransom |
| 2 | Trojan Horses | `#DD1155` | Trojans |
| 3 | Spyware & Keyloggers | `#AA0033` | Spyware |
| 4 | Network Worms | `#FF6600` | Worms |
| 5 | Adware & PUPs | `#CC3300` | Adware |
| 6 | Rootkits | `#800020` | Rootkits |

### Threat State Colors

| State | Color | Meaning |
|-------|-------|---------|
| Orbiting | Dim category color (35% intensity) | In database, scanning |
| Fuzzy | `#FF00FF` magenta pulse | Unclassified, heuristic match |
| Detected | `#FF0044` bright red | Confirmed threat, awaiting action |
| Analyzing | `#0055CC` blue | Under AI/automated analysis |
| Jailed | `#800020` dark purple | Quarantined in Irkalla |
| Neutralized | `#00FF88` green | Cleaned, moved to nucleus |
| FalsePositive | Gray dim | Cleared, not a threat |

---

## 17. BIG RING — Orbital Particle System

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The universal particle system visualizing all 50,000 tracked particles orbiting a central Sun Nucleus in a ring with inner radius 12 units, outer radius 47 units, colored by orbital distance (Gold → Green → Blue → Red → Gray). |
| **Why** | The BIG RING is the visual reification of Triple-O: all particles exist in orbital relationship to a gravitational center (the mastered state). Distance from center = quality/state. Color gradient = lifecycle position. |
| **Where** | `crates/dubsar-visualizer` — `BigRing` struct. `RingGeometry` defines all dimensions. WGPU ring buffer with 50,000 `OrbitalParticle` instances. |
| **When** | Always visible as background to BeeMDM and Nergal dashboards. Rotating at -0.002 radians/frame. |
| **Who** | All stakeholders see the BIG RING as the primary metaphor. The tribe that has all particles become GOLD achieves the Tribe Sphere — the inner gold band of the BIG RING. |
| **How** | `OrbitalParticle` stores: KAKI, state, `FixedCoord7D` position, `orbital_radius`, `orbital_angle`, `orbital_velocity`, `orbital_phase`, `ColorBand`. `DroppingParticle` handles falling animation (ring → gate sector). |
| **How Much** | 50,000 particles total. Ring rotation: -0.002 rad/frame. Drop rate: 0.04–3.0 particles/frame. Nucleus grows with each mastered particle. |

### Color Band Definition

| Band | Radius | Color | Hex | Meaning |
|------|--------|-------|-----|---------|
| Gold | < 18 units | `[1.0, 0.84, 0.0]` | `#FFD700` | Mastered, canonical — inner core |
| Green | 18–28 units | Lerp gold→green | | Stable, validated |
| Blue | 28–35 units | Lerp green→blue | | Processing, staged |
| Red | 35–47 units | Lerp blue→red | `#FF0044` | Raw, exception, fuzzy |
| Gray | > 47 units | `[0.33, 0.33, 0.33]` | `#555555` | Dead, dropping, decay |

### Drop Mechanics

```
ring particle → DropState::Falling (velocity_y: -0.2 to -0.8 units/frame)
             → DropState::Stacking (landing in gate sector at 3×3×N grid)
             → DropState::Cleansing (upward movement after remediation)
             → DropState::Promoting (to Sun Nucleus, gold glow, scale→0)
             → DropState::Jailing (to Irkalla, velocity_y: -0.55, fade purple→transparent)
```

---

## 18. BahyWay Website Architecture

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The public-facing and stakeholder-facing website for the BahyWay.Ecosystem, built in pure Rust compiling to WASM, using WGPU for 7D visualization in the browser. |
| **Why** | The website is not just a dashboard — it is a **particle observer**. Every stakeholder query is a particle. Every visualization is a particle state. The WGPU canvas renders the Tribe Sphere, HeptaMAP, and TIAMAT timeline in real-time. |
| **Where** | `crates/bahyway-web`. Deployed as static WASM + HTML to CDN or Fedora 44 self-hosted server. |
| **When** | Built after the core crates (KAKI, HeptaMAP, BeeMDM) are stable. Browser-tested on Chromium (WebGPU support). |
| **Who** | External stakeholders, tribal administrators, clients, regulatory bodies, emergency coordinators. |
| **How** | Rust → WASM via `wasm-bindgen` + `wasm-pack`. `trunk` as build tool. `wgpu` with WebGPU backend (fallback: WebGL2). `winit` event loop replaced with browser event listeners. Assets embedded via `include_str!`. |
| **How Much** | WASM binary target: < 5MB after `wasm-opt` LTO strip. Target: 60 FPS on mid-range laptop GPU. Tested on Chromium (WebGPU), Firefox (WebGL2 fallback). |

### Map Layer Strategy

Navigation visualization uses **two rendering contexts**:

| Context | Technology | Responsibility |
|---------|-----------|---------------|
| Base map layer | MapLibre GL JS (JavaScript, in `<div>`) | Terrain, 3D buildings, street network, satellite imagery, routing overlay |
| Ecosystem overlay | WGPU (Rust WASM, on `<canvas>`) | 7D particles, tribe spheres, pattern corridors, Nash heatmap, Šàtamu overlays |

Camera synchronization: MapLibre camera position → message to WASM → ENLIL view matrix update. Both layers use the same geographic coordinate origin.

### Why MapLibre, Not Kepler.gl

| Aspect | Kepler.gl | MapLibre | Decision |
|--------|-----------|----------|----------|
| Purpose | Geospatial analytics (heatmaps, scatter) | Navigation maps (routing, 3D buildings) | **MapLibre** |
| 3D buildings | Limited | Excellent | **MapLibre** |
| WGPU sync | WebGL only, difficult | WebGL, better hooks | **MapLibre** |
| Indoor maps | No | Limited (our extension) | Both need NAVIENGINE |

### Tile Server (Self-Hosted)

`martin` — a pure Rust vector tile server — runs on Fedora 44:
- Serves OpenStreetMap vector tiles (`.osm.pbf` pre-processed)
- Serves SRTM terrain elevation tiles
- No third-party tile API dependency
- KAKI-authenticated tile requests for sensitive areas

---

## 19. OrbitalClock — Universal Time System

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The hierarchical time system used throughout BahyWay.Ecosystem. Replaces Unix timestamps with a 7-level orbital frequency hierarchy from 100ms (Micro) to 365 days (Eon). |
| **Why** | Different processes operate at different time scales. BeeMDM ETL gates advance at BASE (1 minute). Nash equilibrium recomputes at MACRO (1 hour). Deep archive crystallizes at EPOCH (1 day). One uniform timestamp type prevents confusion. |
| **Where** | Used in every crate that touches time: particles, journal entries, pattern-kakis, ETL records, Šàtamu decisions, WGPU animation. |
| **When** | The canonical time reference for all events. `Orbital::from_unix_millis()` converts system time. `to_compact()` gives a `u64` for storage. |
| **Who** | All components. The OrbitalClock master runs in `eridu-scheduler`. Distributed via `RenderNode` heartbeats for NARUDU-RENDER cluster. |
| **How** | Struct with 7 fields: `eon` (365-day cycles), `era` (days), `epoch` (hours), `macro_orbital` (minutes), `base` (minute count), `milli` (seconds), `micro` (100ms). Packed to `u64` for storage. |
| **How Much** | `u64` packed format: 40 bits for eon, cascading down to 3 bits for micro. Max representable time: ~4 billion years. |

### Frequency Table

| Level | Name | Duration | Primary Use |
|-------|------|---------|------------|
| 0 | Micro | 100ms | Real-time collision, social force |
| 1 | Milli | 1 second | Position updates, velocity integration |
| 2 | Base | 1 minute | Pattern validation, Nash equilibrium |
| 3 | Macro | 1 hour | Canonicalization, Šàtamu review |
| 4 | Epoch | 1 day | Deep archive, crystallization |
| 5 | Era | 30 days | Seasonal patterns, regulatory audit |
| 6 | Eon | 365 days | Historical analysis, system evolution |

---

## 20. Nash Equilibrium Layer

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | The game-theory-based stability measurement applied to particle populations within each pattern-kaki. Nash equilibrium score measures whether particles are following their optimal routes (stable) or deviating (unstable). |
| **Why** | When a crowd pattern enters Nash equilibrium, no individual particle can improve its outcome by deviating unilaterally. This is a signal that the pattern is canonical and reliable. High anarchy score = the pattern is breaking down = route rerouting needed. |
| **Where** | Computed by NISABA at MACRO frequency (hourly). Stored in `NashState` within `PatternEntry`. Visualized by Nash heatmap (compute shader). |
| **When** | Recalculated when: `deviation_count` spikes, `equilibrium_score` changes > 10%, or `anarchy_score` crosses 0.5. |
| **Who** | NISABA computes. NAVIENGINE uses to select routes (prefer low-anarchy patterns). Šàtamu AI Council (NINSUN) validates cross-domain Nash consistency. |
| **How** | Equilibrium score: 0.0 = stable (all particles at Nash equilibrium), 1.0 = breaking. Anarchy score: price of anarchy (how much worse total outcome is than social optimum). Deviation count: particles deviating from Nash-optimal route this orbital. |
| **How Much** | Nash heatmap texture: 1920×1080 pixels. Compute shader: 16×16 workgroups. Gaussian influence radius per pattern: configurable, default varies by pattern type. |

### Heatmap Color Encoding

| Condition | Color | Visual Signal |
|-----------|-------|--------------|
| Stable equilibrium | `[0.0, green, blue]` | Cool, calm — routes reliable |
| Breaking equilibrium | `[1.0, 0.5, 0.0]` amber | Warning — route may degrade |
| High anarchy | `[0.7, 0.0, 0.7]` purple | Severe — consider alternative routes |

### TIAMAT → Data Tier → Alert Mapping

**Naming conflict flagged 2026-07-07, partially resolved 2026-07-07 — still
needs an Architect ruling on BC-ENV-001's variant.** This table's `NERGAL`
level collides with the separately-named **Nergal sovereign anti-virus
engine** (GLS-001). Two later documents independently noticed this and moved
away from it, but disagreed with each other and were never back-ported into
this table:

- `BC-ENV-001` (Enbilulu Calculus, Rev. 2) drops this whole 5-level ladder in
  favor of its own 4-level one — **Stable → Watch → Serious → ERRA** —
  explicitly noting *"Nergal remains reserved for the BahyWay sovereign AV
  engine."*
- `PAZUZU_SIMULATION.md` keeps this table's `GREEN`/`DILBAT`/`KAKKAB`/`MAROON`
  but swaps `NERGAL` → `ERRA` — a third, partially-overlapping variant.

**Update 2026-07-07 (first pass) — a 6-level ladder found, then superseded.**
`BahyWay_v4_ArchRef_Vol2_BeeMDM_TripleO_20260627.docx` §8.1 gives a 6-level
ladder `GREEN(0)→DILBAT(1)→KAKKAB(2)→ERRA(3)→NERGAL(4)→MAROON(5)` and states
the `NERGAL`-as-AV-engine / `NERGAL`-as-alert-level overlap is intentional.
This was recorded here as a resolution, but a later document overturns it —
see below. (Left in the history here rather than deleted, per this
ecosystem's own "old state is never erased" law.)

**Update 2026-07-07 (second pass) — `GL-001` (Glossary, "Living Document",
undated but internally later than Vol. II — it already references UrNammu
Engine and Kittu Engine, which postdate Vol. II) rules the other way and
looks like the standing decision.** Its Alert-levels entry reads verbatim:
*"DILBAT / KAKKAB / NERGAL / MAROON. ERRA 𒀭𒂗𒆳 replaces the incorrectly-named
NERGAL alert level (NERGAL is the BahyWay AV engine, not an alert)."* That is
the **opposite** call from Vol. II: not "keep both, `NERGAL` is dual-purpose
on purpose," but "`NERGAL` was a naming mistake at the alert-level layer,
`ERRA` replaces it there, `NERGAL` belongs to the AV engine alone." Under
this reading, PAZUZU's list (`GREEN/DILBAT/KAKKAB/ERRA/MAROON` — a 5-level
ladder with `ERRA` where this table has `NERGAL`) was the version tracking
the *current* rule, and this table (still showing `NERGAL`) is the one that
needs the edit — not the other way around, and not a 6-level ladder.

**Net effect — this table's `NERGAL` row is very likely wrong and should
become `ERRA`, matching PAZUZU_SIMULATION.md.** Not changed in this pass
because it's a two-source disagreement (Vol. II vs GL-001) inferred to be
resolved by *document recency*, not by an explicit dated seal — recommend
the Architect confirm directly, then this table's `NERGAL` row can be
renamed to `ERRA` in one edit. BC-ENV-001's separate `Stable/Watch/Serious/
ERRA` wording is a third, still-unreconciled vocabulary (different words
entirely, not clearly the same 4/5-level ladder) — unaffected by this
finding, still needs its own ruling.

| TIAMAT | Meaning | Data Tier | Nash Meaning |
|--------|---------|-----------|-------------|
| GREEN | Safe, low energy | Hot | Equilibrium stable |
| DILBAT | Moderate, watch | Warm | Equilibrium mildly unstable |
| KAKKAB | Elevated, caution | Cold | Equilibrium degrading |
| NERGAL | Critical, danger | Archive | Equilibrium breaking — reroute now |
| MAROON | Emergency | — | Full chaos — system override |

---

## 21. Complete Glossary — W5H2 Method

---

### AAOL
**What**: The Akkadian Assembly-Oriented Language used by the AkkA compiler for generating sovereign code. **Why**: Pure Rust codebase needs a meta-language for templates and code generation that is itself sovereign. **Where**: `crates/aaol`. **When**: At compile time. **Who**: AkkA compiler, system architects. **How**: Compiles `.akk` files to Rust/YAML/Python. **How Much**: Trusted compiler tool — outside the particle system.

---

### ArchitecturalHint
**What**: A minimal data structure describing a building's outline, obstacles, entrance/exit points, and intended function — the minimum input to `ENKI-GENESIS`. **Why**: Enables synthetic particle seeding for unmapped buildings. **Where**: `enki_genesis::ArchitecturalHint`. **When**: Provided once per new building registration. **Who**: Building operator or system administrator. **How**: Polygon boundaries + obstacle arrays + function enum → ENKI-GENESIS Monte Carlo. **How Much**: Boundary polygon: 64 points max. Obstacles: 32 polygons × 16 points. Entrances/exits: 8 each.

---

### BIGRING
**What**: The Orbit that contains all Tribe Orbits belonging to a given Client. BIGRING is mathematical topology — it has no KAKI of its own, no governance, no kill-switch. A `BIGRING_Schema` (EAV schema) exists where Entities belong to that schema. Every Particle inside the BIGRING retains its own immutable KAKI forever — KAKI is immutable and stays on the Particle regardless of which Orbit or Tribe it inhabits. **Why**: Assigning a KAKI to the ring geometry itself would create a sovereign kill-switch: whoever holds BIGRING_KAKI could control all tribes. Topology-only design prevents that. The EAV schema provides the structural anchor without introducing ownership. **Where**: `BIGRING_Schema` in the EAV schema registry. Visualised in `crates/dubsar-visualizer` → `BigRing` / `RingGeometry`. **When**: Present at every scale — it is the outermost orbital container for each Client's data universe. **Who**: All tribes and Particles coexist inside it. No single actor governs it. **How**: Mathematical union of all Tribe Orbital spaces for a Client. EAV Entities reference `BIGRING_Schema` as their top-level schema context. Particles carry their own KAKI unchanged. **How Much**: Encompasses the entire 7D space for one Client; one BIGRING per Client deployment. *(ADR-BIGRING closed 2026-06-30)*

---

### BodyTierManager
**What**: The storage manager for particle body data (non-KAKI, non-EAV content: files, images, documents) across Hot→Warm→Cold→Crystallized tiers. **Why**: KAKI and EAV alone hold identity and attributes. Body data (file contents) must be stored separately, referenced by hash in EAV. **Where**: `crates/enkidb-storage` or new `body-storage` crate. **When**: Every particle with a body (external files, documents) references it. **Who**: Used by NARAMSIN archive pipeline, NAVIENGINE indoor maps, BeeMDM external file ingestion. **How**: Hash-addressed. Body hash stored in EAV `body_hash` attribute. Retrieval verifies hash. Corruption triggers `LossDetectionSweep`. **How Much**: Hot: immediate access. Warm: <1s. Cold: minutes. Crystallized: hours.

---

### CircuitBreaker (Wayv2.0)
**What**: A safety mechanism that prevents healing cascades. Any healing action that triggers another healing action within the same orbital period is auto-suspended and escalated to DataSteward. **Why**: Self-healing systems can enter infinite healing loops that consume all resources while never actually healing. **Where**: `crates/enkidb-recovery` — Wayv2.0 module. **When**: Active whenever any ConEngine healing action executes. **Who**: Wayv2.0 healing engine. Escalation target: DataSteward. **How**: Action counter per orbital period. If count > 1 for same subsystem → suspend + alert. **How Much**: Counter resets every BASE orbital (1 minute).

---

### ColorID / ColourID B11
**What**: `round(score × 240)` — a numerical color identifier derived from quality score. **Why**: Color is not cosmetic — it is a derived, computed property reflecting particle health. **Where**: Stored in EAV mandatory attribute `color_id`. Never in KAKI (KAKI is immutable). **When**: Recomputed on every ETL gate transition. **Who**: BeeMDM pipeline stations. **How**: `color_id = round(quality_score × 240)`. Hard constraint: NEVER 255 (ADR-001 sovereign rule). **How Much**: Range: 0–240. 0 = dead/gray. 240 = near-GOLD.

---

### ConEngine
**What**: The distributed consistency engine managing partition recovery and split-brain healing for KAKI-addressed data. **Why**: In a distributed deployment, network partitions create split-brain scenarios where two nodes have conflicting views of the same particle. ConEngine heals this. **Where**: `crates/enkidb-engine` consensus module. **When**: Triggered on network partition detection. **Who**: System level — transparent to DataStewards and clients. **How**: Vector clocks for causality tracking. Circuit breaker prevents cascade. KAKI immutability means only EAV attributes can conflict (KAKI itself is always the same). **How Much**: Max partition healing time: 1 Macro orbital (1 hour) before escalation to DataSteward.

---

### CrowdFlow Pattern
**What**: A `pattern-kaki` with `pattern_type = CrowdFlow`, representing a stable path that large numbers of pedestrians naturally follow in a space. **Why**: Crowd flows are emergent — they cannot be hand-designed. NISABA discovers them. NAVIENGINE uses them. ENKI-VELOCITY renders them as the green crowd layer. **Where**: `ENKI-PATTERN` hot tier when canonical. Layer 1 (Crowd) in WGPU viewport. **When**: Created after sufficient observation orbitals (or synthetic seeding). **Who**: NISABA mints. AI Council validates. NAVIENGINE subscribes. **How**: GA clustering on pedestrian trajectory data. Nash equilibrium confirms stability. **How Much**: Typical confidence threshold for canonicalization: 9000 (out of 10000).

---

### DataSteward
**What**: A human operator with clearance level 1–5 who makes binding decisions about particle states, pattern canonicalization, and emergency overrides via the Šàtamu interface. **Why**: The system is sovereign but not autonomous. Human judgment is required for: edge cases statistics miss, regulatory decisions, emergency overrides, AI Council split votes. **Where**: Accessed via Šàtamu on any device (smartwatch to desktop). **When**: Called when: TiamatLevel > KAKKAB, AI Council split, scheduled validation orbitals, manual review request. **Who**: Trained human operators appointed by the tribe administrator. **How**: Signs every decision with identity-kaki. StewardLens (read-only auditor) monitors for anomalies. Forced re-fuzzification if override score < threshold. **How Much**: SLA: Level-4 emergencies must be actioned within 1 Micro orbital (100ms) on smartwatch, or escalated to next available steward.

---

### Domain
**What**: The categorization of a pattern-kaki's physical domain: Aviation, Indoor, Maritime, Crowd, Water, Mixed. **Why**: Different domains have different physics (aircraft separation vs. pedestrian spacing), different data sources, different Nash coupling rules. **Where**: `enki_pattern::Domain` enum. Stored in `PatternSpatialKey`. **When**: Assigned at pattern-kaki minting. Immutable thereafter. **Who**: NISABA assigns based on trajectory data origin. **How**: `Aviation = 0`, `Indoor = 1`, `Maritime = 2`, `Crowd = 3`, `Water = 4`, `Mixed = 5`. **How Much**: Mixed is used for cross-domain interaction zones (airport terminal: crowd + indoor + aviation all overlap).

---

### E7Region
**What**: A 7-dimensional axis-aligned bounding box defined by `min: [u64; 7]` and `max: [u64; 7]`. Used for spatial queries in HeptaMAP and subscription regions in ENKI-PATTERN. **Why**: The same structure used in ENKI-PATTERN subscriptions (`subscribe_region()`) and HeptaMAP queries ensures that navigation queries are physically meaningful. **Where**: Used in `PatternSubscription`, `HeptaMapFixed::query_region()`, `StewardOverlay::QuarantineRegion`. **When**: Every regional query. Every subscription. Every quarantine zone definition. **Who**: NAVIENGINE when subscribing to patterns. Šàtamu when quarantining. **How**: Encoded as `[min_d1..max_d1, min_d2..max_d2, ..., min_d7..max_d7]` in 7D. 112 bytes (`7 × 2 × 8`). **How Much**: Can span any sub-region of 7D space — from a single room (small E7Region) to a continent (large E7Region).

---

### ENLIL Algebra
**What**: The mathematical framework, named after the Sumerian god of air and storm, that governs 7D→3D dimensionality reduction for visualization. Ensures orbital relationships are preserved in projection. **Why**: Simple orthogonal projection of 7D to 3D loses orbital structure. ENLIL Algebra maps semantic dimensions to visual properties (size, shape, glow) rather than discarding them. **Where**: `enlil_projection.wgsl` WGSL shader. `EnlilProjectionParams` uniform buffer. **When**: Every vertex shader invocation for every visible particle. **Who**: All render layers use the shared ENLIL uniform. **How**: Spatial (d1,d2,d3) → position. Time (d4) → z-offset ghost trail. Semantic (d5,d6,d7) → size, shape, glow. State → color. KAKI type → special effects. **How Much**: 256-byte uniform buffer. Per-particle: 8 floats = 32 bytes of vertex data.

---

### ENKI-GENESIS
**What**: Synthetic Particle Seeder. (See Section 5.) Short form of the process that derives virtual crowds from architectural hints.

---

### ENKI-PATTERN
**What**: Shared Pattern Registry. (See Section 6.)

---

### ENKI-VELOCITY
**What**: Performance & LOD Engine. (See Section 12.)

---

### EnkiMDB
**What**: Services-as-particles database. Stores BahyWay.Ecosystem services including query plan templates as particles with KAKI. **Why**: Services themselves are particles — they have identity (KAKI), state (alive/deprecated), and lifecycle (EAV attributes). Query plans cached here as canonical templates. **Where**: `crates/enkidb-engine` or dedicated `enkimdb` crate. **When**: Consulted on every ŠUMU-UKIN query plan lookup. **Who**: Query engine (ŠUMU-UKIN), AkkA compiler (template registry), Šàtamu (for pre-approved query templates). **How**: Same KAKI + EAV + BeeMDM pipeline as all other particles. Plan KAKI derived deterministically from (query_hash, planner_version, tribe_kaki, schema_version, optimization_level). **How Much**: NARUDU cache: 64/256 entries, 87.3% hit rate. EnkiMDB holds the persisted warm-tier copies.

---

### EAV Architecture
**What**: Entity-Attribute-Value — the schema model for particle metadata. Mandatory attributes (system-defined) + Optional attributes (tribe-defined). **Why**: Different tribes have different data needs, but all particles share a common mandatory core. EAV allows schema flexibility without breaking KAKI immutability. **Where**: `crates/enkidb-sdb` (Structured Database) and `crates/enkidb-qdb` (Query Database). **When**: Created at particle birth. Updated at every ETL gate transition. **Who**: BeeMDM pipeline writes. NAVIENGINE reads. Šàtamu governs. **How**: KAKI → mandatory EAV row (state, ColorID, birth, tribe) + optional EAV rows (org-defined attributes). **How Much**: Mandatory: ~8 attributes × ~16 bytes each = ~128 bytes per particle minimum.

---

### Fuzzy Particle
**What**: A particle in the `fuzzy` state — unclassified, quality uncertain, requires DataSteward intervention via Šàtamu. **Why**: Not every particle is clearly good or bad. Fuzzy particles represent the uncertain middle ground that human judgment must resolve. **Where**: Stuck at a BeeMDM gate awaiting DataSteward decision. Visualized as magenta pulsing particles in WGPU viewport. **When**: Particle reaches `fuzzy` when quality_score < threshold at any gate. **Who**: DataSteward decides: Resolve (return to pipeline) / Reject (dead) / Hide (client-requested hidden). StewardLens monitors steward decisions for anomalies. **How**: AI score engine + fuzzy logic rules engine provide AI check on steward decisions. Forced re-fuzzification if score < threshold after resolution. **How Much**: SLA: fuzzy particles older than N orbitals escalate automatically to higher-clearance steward.

---

### GA Clustering (Genetic Algorithm)
**What**: The clustering algorithm used by NISABA to discover emergent patterns from trajectory data. Groups nearby trajectories into centroid-based clusters that become pattern-kaki candidates. **Why**: k-means and hierarchical clustering require knowing N clusters in advance. GA clustering discovers N naturally from density. **Where**: Inside NISABA's pattern extraction pipeline. Simplified by `extract_pattern_candidates()` (grid density → GA). **When**: After each simulation batch (Monte Carlo) or after sufficient real trajectory accumulation. **Who**: NISABA runs it. Output feeds into pattern-kaki derivation. **How**: Grid-based density counting (100×100 grid) as seeding approximation. Full production: 7D k-means with adaptive k. Centroids → `PatternKakiCandidate` → `derive_pattern_kaki()`. **How Much**: Grid: 100×100. Threshold: 2× average density. Full GA: 7D clustering, up to 126 clusters per region (Viazovska constraint).

---

### GOLD State
**What**: The terminal state of a particle that has passed all ETL gates, received all mandatory validations, and been promoted to the Tribe Sphere. GOLD is the ultimate trust state. **Why**: Not all particles reach GOLD. It signifies canonical, trusted, permanently recorded data. The Tribe Sphere glows brighter with each new GOLD particle. **Where**: Stored as `state = Gold (0x08)` in EAV. Visualized as the golden inner ring (radius < 18 units) and the Sun Nucleus in WGPU. **When**: Assigned by ENLIL gate (Gate 7, Mastering) after all validations pass. **Who**: ENLIL gate assigns. Tribe Sphere records. **How**: `quality_score > 9000` + all mandatory gates passed + no unresolved fuzzy flags. **How Much**: GOLD particles are append-only, immutable forever. ColorID = 240 (maximum non-255 value per ADR-001).

---

### HeptaGate
**What**: One of the 7 BeeMDM ETL processing stages (APSU, ADAD, SHEDU, MUMMU, ENKIDU, DUBSAR, ENLIL). Each gate has a distinct function, color, and geometric position in the WGPU visualization. **Why**: 7 gates reflect the 7-dimensional nature of the data. Each gate operates on a different aspect of quality: format, schema, enrichment, transformation, federation, indexing, mastering. **Where**: `etl::HeptaGate` enum. 7 `GateSector` objects in `BahyWayRuntime`. **When**: Particles advance through gates in order (conditional — failure at any gate triggers fuzzy state or exception). **Who**: BeeMDM ETL pipeline assigns. DataSteward can manually advance or hold at any gate. **How**: Linear pipeline with conditional branching. Bad quality → exception. Fuzzy → DataSteward. Good → advance. **How Much**: Gate sector capacity: 9×N layers (3×3 grid, unlimited height). Dwell time: configurable per gate, minimum 1 BASE orbital.

---

### HeptaMAP
**What**: The 7D spatial index that replaces BTreeRange in the ENLIL Index Stack (Layer 3). Hybrid architecture: Fixed Integer Grid for hot/warm tiers, Adaptive Variable Precision for cold/archive tiers. **Why**: "The orbit is not a tree." BTreeRange assumes linear ordering. 7D orbital space is not linear — it is cyclic, spatial, and multi-dimensional. HeptaMAP natively handles 7D proximity queries. **Where**: `crates/enkidb-indexes` — `HeptaMapHybrid`. **When**: Used for every spatial range query, pattern region subscription, and idx_archive lookup. **Who**: ENLIL Index Stack, NAVIENGINE route planning, ENKI-PATTERN registry. **How**: Hot/warm: Fixed Integer Grid (GPU-friendly, SIMD-native). Cold/archive: Adaptive Variable Precision (memory-efficient). Transition at GOLD crystallization. **How Much**: E7 kissing number = 126 neighbors maximum per query (Viazovska constraint). Memory savings vs BTreeRange: 12–35%.

---

### HeptaScript v2.0
**What**: The dual-layer sovereign query language for BahyWay.Ecosystem. Layer 1 (Meta HeptaScript) for architects/systems. Layer 2 (Šàtamu Visual) for all stakeholders. **Why**: Two audiences with different needs. Architects need full W5H2 expressiveness with tier keywords, state filters, and lineage traversal. Stakeholders need 8 simple keywords with no code execution risk. **Where**: `crates/heptascript` (Meta). `crates/dubsar-ide` (Šàtamu Visual parser). **When**: Every database query. Every pattern subscription. Every Šàtamu override command. **Who**: L1: System architects, AI agents, automated pipelines. L2: All stakeholders — management, regulators, field operators. **How**: L2 Šàtamu compiles to pre-approved L1 templates only. No arbitrary code execution in L2. Template registry with pre-compiled `.akk` particles. **How Much**: L2 keywords: 8 (`tribe`, `station`, `when`, `state`, `color`, `show`, `count`, `timeline`). L1 keywords: full W5H2 (7 dimensions).

---

### Irkalla
**What**: The quarantine zone for dead particles (gray, failed ETL) and jailed threats (Nergal AV). Named after the Akkadian underworld. **Why**: Dead and quarantined particles cannot simply be deleted — they must be retained for forensic analysis, regulatory compliance, and the Trial Audit Journal. Irkalla is the sovereign quarantine archive. **Where**: Visualized as a dark sphere below the gate sectors in WGPU. Stored in `QDB` (Quarantine Database) alongside the main database. **When**: Particles route to Irkalla when DataSteward decision = Reject, or automatic quarantine from ThreatState::Jailed. **Who**: BeeMDM pipeline sends rejected particles. Nergal AV sends jailed threats. DataSteward reviews and can release or permanently delete. **How**: `Irkalla::jailed_threats: Vec<Kaki>` + `auto_expunge_orbital` for automatic cleanup. **How Much**: Capacity: 100,000 jailed entities. Auto-expunge: configurable, default 365 orbitals.

---

### KAKI
**What**: 16-byte (128-bit) universal identity key — **KAKIv4, final, immutable forever**. Layout: `[type_prefix:2bits][entropy:110bits][checksum:16bits]`. **Why**: Every entity in the BahyWay.Ecosystem needs a permanent, unique, unforgeable identity. KAKI is the root of trust. Immutable. Eternal. Generated once at birth. The 16-byte size is a permanent sovereign constraint (ADR-KAKI closed 2026-06-30). **Where**: Every crate, every struct, every particle. `crates/enkidb-kaki`. **When**: Assigned at particle birth. Never changed. Never recycled. **Who**: Generated by the system (AkkA compiler, BeeMDM APSU gate). Assigned to: data records, services, stewards, patterns, events, AI agents, query plans. **How**: 16 bytes: 2-bit type prefix + 110-bit entropy + 16-bit checksum. Deterministic for pattern-kakis and query plan KAKIs. Cryptographically random for identity-kakis and events-kakis. **How Much**: 2^126 addressable values. Collision probability at 10^9 generations: negligible. Size will NOT change — all storage formats, wire protocols, and indexes are designed for exactly 16 bytes. *(ADR-KAKI closed 2026-06-30)*

---

### KISPU HeadStore
**What**: Layer 1 of the ENLIL Index Stack. An array-indexed store providing O(1) lookup from surrogate key (u32) to KAKI head pointer. **Why**: The SurrogateMap (Layer 0) gives a compact u32 per KAKI. KISPU uses that u32 as a direct array index for O(1) particle lookup — the fastest possible access pattern. **Where**: `crates/enkidb-indexes`. **When**: Every read query that knows the surrogate key. **Who**: Query engine (ŠUMU-UKIN), NAVIENGINE, BeeMDM pipeline. **How**: `kispu_array[surrogate_u32] = kaki_head_pointer`. **How Much**: Memory: 4 bytes per surrogate × N particles. At 1B particles: 4GB for the array alone — kept in DRAM, not GPU.

---

### MapLibre GL JS
**What**: The open-source navigation map renderer (JavaScript/WebGL) used as the base map layer in the BahyWay website. **Why**: MapLibre handles terrain, 3D buildings, satellite imagery, and navigation overlays natively — things that would take years to rebuild in pure Rust WGPU. It is the base. WGPU is the ecosystem overlay on top. **Where**: Browser layer, runs in `<div>` alongside the WGPU `<canvas>`. Served via CDN or self-hosted. **When**: Always running in the browser when the navigation view is active. **Who**: End-user website visitors. DataStewards on the web interface. **How**: Camera synchronization: MapLibre → WASM message → ENLIL view matrix update. Both share the same geographic coordinate origin. **How Much**: No third-party data dependency — tiles served from self-hosted `martin` server on Fedora 44.

---

### martin
**What**: A pure Rust vector tile server that self-hosts OpenStreetMap tiles on Fedora 44. **Why**: Eliminates dependency on Mapbox/Mapillary/Google tile APIs. Full data sovereignty — no third-party service can disable the tile feed. **Where**: Runs as a service on the Fedora 44 development/production server. Serves tiles to MapLibre GL JS in the browser. **When**: Always running when the BahyWay website navigation view is active. **Who**: MapLibre GL JS fetches tiles from it automatically. **How**: Reads pre-processed MBTiles files generated from `planet.osm.pbf`. **How Much**: A regional extract (Iraq + surrounding countries) is manageable (<5GB). Planet extract is ~100GB processed.

---

### Monte Carlo Simulation (ENKI-GENESIS)
**What**: The 1000-iteration × 100-agent × 1000-step simulation used by ENKI-GENESIS to discover synthetic indoor movement patterns. **Why**: Random sampling over many iterations reveals stable paths that deterministic algorithms miss. The law of large numbers ensures that if a path is natural, many random agents will find it. **Where**: `run_synthetic_simulation()` in ENKI-GENESIS. **When**: Once per building registration. Re-run on major layout changes. **Who**: Triggered by building operator or DataSteward. Results validated by Šàtamu SyntheticValidationBoard. **How**: `XorShift64` RNG (no_std compatible). 100 agents per iteration. Social Force model per step. Trajectory sampling every 10 steps. Grid density extraction. **How Much**: 1000 × 100 × (1000/10) = 10,000,000 trajectory points. Memory: pre-allocated with `Vec::reserve(min(10M, estimated))`.

---

### NARUDU
**What**: The query plan cache. Hot in-memory cache of compiled ŠUMU-UKIN query plans. Extended with archive tier for decompressed partitions. **Why**: Query compilation is expensive. Caching plans with 87.3% hit rate eliminates redundant compilation. **Where**: `crates/enkidb-engine` NARUDU module. 64 entries (base config), 256 entries (extended). **When**: Every query. Cache miss → compile → store in NARUDU + persist to EnkiMDB. **Who**: Query engine (ŠUMU-UKIN). EnkiMDB provides warm-tier persistence. **How**: LFU (Least Frequently Used) + priority eviction. Plan KAKI = deterministic hash of query text + planner version + tribe + schema + optimization level. Same inputs = same KAKI = O(1) cache hit. **How Much**: 64/256 entries × ~4KB per plan = 256KB–1MB NARUDU memory footprint.

---

### NARUDU-PATTERN
**What**: Event Stream Protocol for pattern lifecycle events. (See Section 7.)

---

### NARUDU-RENDER
**What**: Distributed Rendering Cluster. (See Section 13.)

---

### NAVIENGINE
**What**: 7D Navigation Engine. (See Section 3.)

---

### NashState / Nash Equilibrium
**What**: Game-theory stability measurement for particle populations within patterns. (See Section 20.)

---

### NISABA
**What**: Pattern Discovery Service. (See Section 4.)

---

### OrbitalParticle
**What**: A particle currently orbiting in the BIG RING, awaiting entry into the ETL pipeline. Contains: KAKI, state, `FixedCoord7D` position, orbital dynamics (radius, angle, velocity, phase), `ColorBand`, birth orbital. **Why**: The BIG RING is the visual holding area — particles orbit until the pipeline is ready to process them. Orbital position is not arbitrary: it reflects the particle's quality band (inner = gold/mastered, outer = raw/exception). **Where**: `orbit::OrbitalParticle` struct. 50,000 instances in `BigRing::particles: Vec<OrbitalParticle>`. **When**: Every particle begins its life in the BIG RING before entering a HeptaGate. **Who**: All incoming data records, threat signatures, and navigation events. **How**: Position computed from radius + angle. Color band derived from radius: `ColorBand::from_radius(radius)`. **How Much**: 50,000 particles × ~128 bytes each = ~6.4MB ring buffer.

---

### PatternRegistry
**What**: Shared Pattern Registry. (See Section 6 — ENKI-PATTERN.)

---

### PatternSubscription
**What**: A contract between NAVIENGINE and ENKI-PATTERN registry, establishing that NAVIENGINE will receive real-time updates for all canonical patterns within a specified E7Region and pattern type filter. **Why**: NAVIENGINE cannot poll the registry on every route request — that would be O(N patterns) per query. Subscriptions push updates in O(1) time when relevant patterns change. **Where**: `PatternSubscription` struct in `ENKI-PATTERN`. `subscribe_region()` API. **When**: Created when NAVIENGINE is deployed for a new geographic area. Cancelled when area is decommissioned. **Who**: NAVIENGINE subscribes. ENKI-PATTERN maintains subscriptions. NISABA push-notifies. **How**: Subscription_id = `KISPU_hash(subscriber_kaki ‖ region_bytes ‖ current_orbital)`. Active patterns delivered immediately at subscription time. Updates pushed via NARUDU-PATTERN. **How Much**: One subscription per NAVIENGINE deployment region. No limit on number of subscriptions (bounded by memory).

---

### PixelPicker
**What**: The WGPU mechanism that encodes KAKI IDs into pixel colors in a separate render pass, enabling mouse-click identification of any particle in the viewport. **Why**: Transparency requires that any stakeholder can click on any visible particle and immediately see its KAKI, state, audit trail, and pattern membership. **Where**: `PixelPicker` struct in `NaviEngineViewport`. Uses `Rgba32Uint` texture format (16 bytes per pixel = full KAKI). **When**: Pass 3 of every render frame. **Who**: Triggered by mouse click events in the viewport. **How**: Separate render pass writes KAKI bytes into `Rgba32Uint` picker texture. On click, `map_async(MapMode::Read)` → read pixel → extract `PickedPixel` → query registry. **How Much**: Picker texture: `width × height × 16 bytes`. At 1920×1080: 31.6MB. Kept on GPU, read only on click.

---

### SatamuRecord
**What**: The per-pattern append-only decision log maintained by Šàtamu. Records every action taken on a pattern-kaki by humans, AI agents, or automated systems. **Why**: All governance decisions must be auditable, immutable, and signed. The SatamuRecord is the governance equivalent of the Trial Audit Journal (NĀRU) for patterns. **Where**: One `SatamuRecord` per pattern-kaki. Stored in Šàtamu governance layer. **When**: Created when a pattern first enters Šàtamu review. Grows with every decision. **Who**: Written by DataStewards, AI Council agents, and automated systems. Read-only by StewardLens auditor. **How**: `Vec<StewardDecisionEntry>` — each entry has actor, action, justification, Ed25519 signature, orbital. Append-only. Never modified after entry is written. **How Much**: Max chain: bounded by orbital retention policy (crystallized after 365 orbitals). Each entry: ~256 bytes.

---

### SimulationTicket
**What**: A KAKI-addressed request for ENKI-GENESIS or NAVIENGINE to run a pattern-based simulation for a given scenario. The ticket tracks the simulation from `Queued` → `Running` → `Complete/Failed`. **Why**: Simulations are expensive. They must be tracked, deduplicated (same ticket KAKI = same simulation), and their results must be auditable. **Where**: Created by `query_simulation()` in ENKI-PATTERN. Stored with KAKI derived from `KISPU(subscription_id ‖ scenario_bytes ‖ orbital)`. **When**: NAVIENGINE requests a simulation when it needs to evaluate route options that pattern data alone cannot resolve. **Who**: NAVIENGINE creates. ENKI-GENESIS executes. Šàtamu AI Council validates results. **How**: `SimulationStatus`: Queued → Running → Complete (with `SimulationResult`) or Failed. Result includes: `InferredGeometry` (corridors, rooms, junctions, tubes). **How Much**: One simulation: 1000 iterations × 100 agents × 1000 steps. Expected runtime: <5 seconds on modern CPU.

---

### SocialForce
**What**: The pedestrian simulation model used by ENKI-GENESIS to generate synthetic trajectories. Each agent is attracted to its target (exit) and repelled from other agents and walls. **Why**: Social Force is the most validated pedestrian dynamics model in crowd science. It produces realistic emergent corridors without hardcoded rules. **Where**: `social_force()` function in `enki_genesis`. Called per agent per simulation step. **When**: Every step in Monte Carlo simulation. **Who**: ENKI-GENESIS runs it internally. Results feed into grid density extraction → pattern candidates. **How**: `F = attraction_to_target + Σ(repulsion_from_neighbors) + Σ(repulsion_from_walls)`. Velocity integrated with damping. Position updated with velocity. **How Much**: 100 agents per iteration × 1000 steps = 100,000 force evaluations per iteration. Agent-agent complexity: O(N²) per step (simplified; production uses spatial grid for O(N log N)).

---

### StewardLens (read-only auditor)
**What**: A pure Rust, `no_std`-compatible, read-only AI auditor that monitors DataSteward decisions for anomalies and reports exclusively to the Architect KAKI. **Why**: The DataSteward has enormous power over particle states. StewardLens ensures no single steward can unilaterally manipulate the system without detection. It is a check on the checker. **Where**: `crates/steward-lens` (to be created). `observe()` is ALWAYS read-only — never modifies particles or system state. **When**: Called after every DataSteward decision. `end_orbital()` emits `ArchitectReport` sealed to Architect KAKI. **Who**: Operates silently. Reports ONLY to Architect. Not visible to DataStewards, clients, or other services. **How**: 8 anomaly codes (0xS001–0xS008): LOW_SCORE_RESOLVE, HIGH_SCORE_REJECT, STEWARD_OVERLOAD, HIGH_RESOLVE_RATIO, SLA_MISMATCH, SUSPICIOUS_HIDE, RAPID_DECISIONS, DECISION_OUTLIER. **How Much**: 64-entry steward stats ring buffer. Anomaly threshold: configurable per code. Report: one `ArchitectReport` per orbital.

---

### StewardOverlay
**What**: The WGPU visual layer (Layer 5) that renders Šàtamu decisions as visual alerts, quarantine zones, and emergency override indicators in the viewport. **Why**: Governance decisions must be immediately visible to all stakeholders viewing the viewport. A quarantine zone should appear as a purple veil. An emergency override should pulse red. **Where**: `StewardOverlay` struct in `NaviEngineViewport`. **When**: Updated every frame. Alerts animate based on `TiamatLevel` (pulse speed: GREEN=0.5Hz, MAROON=8Hz). **Who**: Reflects DataSteward and AI Council decisions. Visible to all viewport users. **How**: `StewardAlert` = billboard quad at world position, projected to screen via ENLIL. `QuarantineRegion` = E7Region rendered as semi-transparent volume. `SelectionBox` = 2D drag-select rectangle for region operations. **How Much**: Max 1000 active alerts + 100 quarantine regions simultaneously before LOD reduction.

---

### ŠUMU-UKIN
**What**: The query execution plan system. Compiles HeptaScript queries into execution plans, caches plans in NARUDU, persists canonical plans in EnkiMDB. Each plan is a particle with a deterministic KAKI. **Why**: Query compilation is expensive. Plan reuse is critical for performance. KAKI-addressed plans enable deduplication, version comparison, and canonical template extraction. **Where**: `crates/enkidb-query` — ŠUMU-UKIN module. **When**: Every HeptaScript query. Cache miss → compile → NARUDU → EnkiMDB. Cache hit → O(1) plan retrieval. **Who**: Query engine runs it. Results served to HeptaScript callers, NAVIENGINE, Šàtamu. **How**: Plan KAKI = `KISPU(query_text_hash ‖ planner_version ‖ client_tribe_kaki ‖ schema_version ‖ optimization_level)`. Same inputs = same KAKI = same plan (deterministic). **How Much**: NARUDU: 64/256 hot plans (87.3% hit rate). EnkiMDB: thousands of persisted warm plans. Archive tier: 16 plans for decompressed partitions.

---

### TamuAI / NINSUN / PAZUZU
**What**: The three AI agents in the Pattern Validation AI Council. (See Section 9.)

---

### TIAMAT
**What**: The 5-level alert severity system: GREEN → DILBAT → KAKKAB → NERGAL → MAROON. Named after the Akkadian primordial goddess of chaos. **Why**: A unified severity system connects data tier (Hot/Warm/Cold/Archive) to operational alert level (normal/watch/caution/danger/emergency). **Where**: `TiamatLevel` enum. Used in: `PatternKaki`, `StewardAlert`, `NashState`, mobile alert routing. **When**: Assigned to patterns by NISABA based on Nash equilibrium deviation. Escalates when equilibrium breaks. **Who**: NISABA assigns. AI Council validates. DataSteward can override. StewardLens monitors for suspicious escalation patterns. **How**: GREEN=Hot data, normal. DILBAT=Warm, watch. KAKKAB=Cold, caution. NERGAL=Archive, route now. MAROON=Emergency, system-wide override. **How Much**: MAROON triggers simultaneous alerts on ALL steward devices.

---

### Trial Audit Journal (NĀRU)
**What**: The per-particle append-only immutable event log recording every state transition from birth to final state (dead or GOLD). Merkle-chained. Multi-signature. **Why**: Sovereign data requires a complete, unforgeable history. Every ETL gate transition, every DataSteward decision, every state change must be recorded and provable. **Where**: `crates/enkidb-journal`. Referenced from every particle's EAV. **When**: Written at every state transition. Never modified. Never deleted within retention policy. **Who**: Every BeeMDM gate signs its transitions. DataSteward signs decisions. Particle KAKI signs its own events. **How**: Merkle chain: each entry references the hash of the previous entry. Multi-signature: gate KAKI + particle KAKI + steward KAKI. **How Much**: Retention: minimum 7 years (regulatory). Archive tier: crystallized forever for GOLD particles.

---

### VectorClock (ConEngine)
**What**: A causality tracking mechanism added to KAKI structure or EAV mandatory attributes to enable partition recovery in distributed deployment. **Why**: When two nodes have conflicting EAV values after a network partition, vector clocks establish causal ordering: which update happened first, which is the true latest value. **Where**: EAV mandatory attributes (recommended: `vector_clock: [u64; N_nodes]`). **When**: Updated on every EAV write. Compared on ConEngine partition healing. **Who**: ConEngine healing engine compares clocks to resolve conflicts. DataSteward reviews any unresolvable conflicts. **How**: Lamport-style vector clock extended to N nodes. On partition heal: compare clocks, take causally-later value, flag causally-concurrent values for DataSteward. **How Much**: N_nodes × 8 bytes per particle per attribute. For 3-node cluster: 24 bytes overhead per EAV attribute.

---

### XorShift64
**What**: A simple, no_std-compatible pseudo-random number generator used in ENKI-GENESIS Monte Carlo and WGPU particle initialization. **Why**: `rand` crate is not no_std compatible without feature flags. XorShift64 gives good statistical quality with minimal code. **Where**: `XorShift64` struct with `next_u64()`, `next_usize(max)`, `next_f64()`. **When**: Every Monte Carlo iteration. Every synthetic particle spawn. WGPU particle color initialization. **Who**: ENKI-GENESIS. `BahyWayRuntime::process_drops()`. **How**: `x ^= x << 13; x ^= x >> 7; x ^= x << 17;` — 3 XOR-shift operations. Seed with `0xDEADBEEFCAFEBABE` or derive from KAKI bytes. **How Much**: Period: 2^64 - 1 (sufficient for all simulations). State: 8 bytes. Cost: ~3 CPU cycles per call.

---

## 22. Shala4 — Gate Orbits & Playbook Dependency Discovery

### W5H2 Definition

| W5H2 | Answer |
|------|--------|
| **What** | A real (not simulated) Three.js browsing view of the ~900 catalogued playbooks in EnkiDDB, organized first by the 7 real `bahyway_core::hepta_gate::HeptaGate` sectors, then by 7 domains per gate (49 total), with free-text search and real `depends-on`/`depended-on-by` dependency discovery between playbooks. |
| **Why** | A flat ~900-node Graph Explorer view was unreadable. Browsing by gate → domain → playbook, plus real dependency edges rendered as orbit rings, gives an Architect a genuine "root as Tribe, related particles as Orbits, till the last Particle" knowledge graph over the playbook corpus — built on the same CrossTribe-Kaki link primitive (`mint_link_edge`) already used elsewhere, not a new mechanism. |
| **Where** | `crates/shakkanakku/src/gate_review.rs`, `domain_review.rs`, `pb_dependency_review.rs` (suggest-then-approve scanners); `bin/shakkanakku_web.rs` (`/api/gates/*`, `/api/playbooks/search`, `/api/dependencies/*`); `bin/web_assets/gate_orbits.js` + vendored `three.module.js`/`OrbitControls.js` (the Shala4 tab itself). |
| **When** | Built 2026-08-02, this session, directly on top of the Phase 0 EnkiDDB generation-completeness fix (every materialized generation must be a complete, self-sufficient snapshot — classification/dependency tags would otherwise vanish on the next re-catalog run). |
| **Who** | Viewing (gate/domain browsing, search) is Public — no passport needed, matching Box Resources' access model. Classification and dependency approval (the "Review" flows) are Architect-only. |
| **How** | Suggest-then-approve throughout, never auto-decided: a scanner produces candidates (gate/domain mentions via `enkiddb::concepts::ConceptRegistry`; dependency mentions via a whole-word scan of each playbook's own text for another playbook's file stem or `PB-<n>`/`PB<n>`/`playbook_<n>` forms), an Architect reviews/edits them in a modal, and only an explicit Approve action calls `WriteNode::tag_gate`/`tag_domain`/`mint_link_edge` and re-materializes the EnkiDDB generation. |
| **How Much** | 7 gates × 7 domains = 49 domain buckets. Dependency scan is O(n²) whole-word text comparison across the full catalog (hundreds of playbooks, not millions) — an occasional Architect-triggered action, not a hot path. |

### A note on the name "HeptaGate" — two different things share it

Section 15 above ("BeeMDM ETL — 7 Hepta Gates Visualization") documents an
earlier, different 7-gate mapping (Ingestion/Validation/Enrichment/
Transformation/Federation/Indexing/Mastering — ETL *pipeline stages*).
Shala4's Gate Orbits uses the actual, real `HeptaGate` enum that shipped in
`bahyway_core` (Apsu=Storage, Adad=ETL, Shedu=Security, Mummu=Algebra,
Enkidu=AI Agents, Dubsar=Languages, Enlil=Governance — *domains of the
codebase*, not ETL pipeline stages). The two are genuinely different
concepts that happen to share the Sumerian gate names and the number 7;
this manual keeps both sections rather than silently reconciling them, per
this ecosystem's own rule against conflating distinct "7"s (a third,
`HeptaShellIndex`/the Anu Stack spatial index, and a fourth, the 25-layer/
5-band Onion Layers architecture stack, are likewise distinct from both).

### Playbook Dependency Discovery

Unlike gate/domain classification (a single scalar tag per playbook, correct
under last-write-wins semantics), a dependency is a relationship between
two distinct playbooks and is minted as its own dedicated edge entity via
`WriteNode::mint_link_edge` — never a repeated particle on a shared entity,
which would collapse under last-write-wins. Each approved pair appends to a
durable `pb_dependency_registry.jsonl` so it is never re-suggested.

A playbook's StoryEngine detail panel (opened from any gate/domain/search
view) renders:
- its `depends_on` list (what it references) and `depended_on_by` list
  (what references it), each row clickable to open that playbook's own
  StoryEngine view in turn — a real, recursive walk of the dependency
  chain, with a working "back" button at every step;
- a 3D orbit visualization: the selected playbook as a core sphere at
  center, its dependencies on an orange ring, its dependents on a purple
  ring — the literal rendering of "root as Tribe, related particles as
  Orbits."

A text mention is a candidate, not proof — the Architect's own standing
rule ("I do not want you to decide what I need to include") applies here
exactly as it does to gate/domain tagging.

### PB Compare State (planned, not yet built)

| W5H2 | Answer |
|------|--------|
| **What** | A new single-scalar `meta.compare_state` particle per playbook variant, holding one of 8 `Shala_<State>` values (see Glossary) that records how that variant's own code compares against what is currently checked into this repo. |
| **Why** | Comparing 2+ variants of the same playbook must not silently discard whichever ones aren't in the current repo — a variant may carry legitimate code not yet merged, or be a deliberately superseded version worth keeping for history. A single Golden/ignored binary loses that distinction; a real lifecycle state does not. |
| **Where** | Planned: `crates/shakkanakku/src/pb_compare_review.rs` (mirrors `gate_review.rs`/`domain_review.rs`/`pb_dependency_review.rs`'s exact suggest-then-approve shape), a new `WriteNode::tag_compare_state`, new Shala `/api/compare/*` endpoints. Not yet implemented. |
| **When** | Designed 2026-08-02, directly following the playbook dependency-discovery build; scheduled as its next extension. |
| **Who** | Viewing would be Public, matching the rest of Gate Orbits. Assigning/changing a compare state is Architect-only, suggest-then-approve — never auto-decided. |
| **How** | A scanner proposes a state per variant (exact text/hash match against the repo → `Shala_Golden` candidate; partial overlap → `Shala_Fuzzy`; no current match and stale → `Shala_Aged`); an Architect reviews and approves the suggestion or picks a different state (`Shala_Deprecated`/`Shala_Rejected`/`Shala_PartiallyAccepted`/`Shala_Dead`) in the same review-modal pattern already built for gates/domains/dependencies. A transition from `Shala_Golden` to `Shala_Deprecated` also calls the existing `WriteNode::supersede_document` (ADR-014's law) so the prior Golden's identity chain stays queryable, never erased — the scalar tag records current status, supersession records the actual lineage. |
| **How Much** | 8 possible states; one live value per playbook variant at a time (a scalar "current status," correctly last-write-wins, unlike a dependency edge). |

---

## Architectural Decisions Summary

| # | Decision | Status | Resolution |
|---|----------|--------|------------|
| 1 | BIGRING governance | **CLOSED — 2026-06-30** | Option A + clarification: BIGRING is the Orbit containing all Tribe Orbits for a Client. No KAKI on the ring geometry itself. A `BIGRING_Schema` (EAV schema) exists where Entities belong to that schema. All Particles retain their own immutable KAKIs. KAKI is immutable and stays forever on every Particle regardless of which ring or tribe orbit it inhabits. |
| 2 | Post-quantum KAKI size | **CLOSED — 2026-06-30** | KAKIv4 = 16 bytes, immutable, stays 16 bytes forever. No expansion. |
| 3 | Vector clocks location | Pending | Add to EAV mandatory attributes (`vector_clock: [u64; 3]`) |
| 4 | Circuit breaker threshold | Pending | 1 healing action per BASE orbital (1 minute) per subsystem |
| 5 | In-Flight Index crash recovery | Pending | Append-only log with replay on restart |
| 6 | AI model update validation | Pending | Model updates enter BeeMDM pipeline as particles |
| 7 | SIMD divergence detection | Pending | Compare SIMD vs scalar results periodically in tests |
| 8 | Schema quarantine period | Pending | N = 30 BASE orbitals isolation for new schema vertices |

---

## Implementation Priority Order

| Priority | Crate | What to Build | Blocks |
|----------|-------|---------------|--------|
| 1 | `steward-lens` | StewardLens read-only auditor (8 anomaly codes) | Šàtamu safety |
| 2 | `agent-council` | AICouncil + TamuzAI + NINSUN + PAZUZU | Pattern governance |
| 3 | `enkidb-indexes` | HeptaMapHybrid (Fixed + Adaptive) | Archive tier queries |
| 4 | `enkidb-indexes` | idx_archive as 7th ENLIL layer | 10-year queries |
| 5 | `heptascript` | v2.0 extensions (tier, state_filter, lineage, Šàtamu) | Query language |
| 6 | `dubsar-ide` | Šàtamu Visual Layer parser (8 keywords) | Stakeholder access |
| 7 | `nisaba` | Pattern discovery + GA clustering | Pattern ecosystem |
| 8 | `enki-genesis` | Synthetic seeder + Social Force | Indoor navigation |
| 9 | `enki-pattern` | Pattern registry hot/warm/cold tiers | NAVIENGINE routes |
| 10 | `naviengine` | Route engine A* → CH | Navigation |
| 11 | `dubsar-visualizer` | NaviEngineViewport multi-layer WGPU | Visualization |
| 12 | `bahyway-web` | WASM website + MapLibre integration | Public access |

---

*Manual generated from: `00_Kimi_Discussions_.md`, `01_Kimi_BahyWay_Website_.md`, `02_NaviEngine_And_Patterns_.md`, and `03_ETL_And_Nergal_Visualization_Code_.md`*

*Version: v4.0.2 — BahyWay.Ecosystem Sovereign Reference*
