# PDM Shape Operator Graph — Node Editor Specification
**Houdini-class procedural editor · pure Rust core · Godot GraphEdit +
egui skins · cooks Shape Tablets that command the DataStructure Station**

BahyWay.Ecosystem v4.0 · fulfills the PDM Modeler lens (GL-DST-001 #1) and
TheaterGraph (build guide Step 9). No Python anywhere. Draft math only in
the editor; sealing via HEPT → GeoEngine → Lamassu → Gate G4 → AAOL.

---

## 1. Architecture — one cook core, two faces

    pdm-graph-core  (pure Rust crate, zero Godot deps)
    ├─ graph model: nodes, typed ports, wires, params
    ├─ cook engine: deterministic evaluation, dirty propagation,
    │  cooked-buffer cache, cycle detection
    ├─ diagnostics: Z2 boundary reduction (draft Betti), gap watch
    ├─ tablet emitter: JSON + HeptaScript contract + bivector
    └─ serde: graph ⇄ tablet (KAKI-identified recipe particle)

    Face A: Godot GDExtension via godot-rust (gdext)
      GraphEdit/GraphNode canvas · RD compute previews (rotor ring,
      density) · CSR-08 confirm dialog on Seal node
    Face B: egui (Fedora VDI, WGPU) via egui-snarl / egui_node_graph
      same core, sovereign monitoring side, no Godot

One cook core guarantees the two faces can never disagree — the same law
that lets one bivector drive two renderers.

## 2. Typed wires (port colors follow the Šala charter)

| Wire          | Carries                                   | Color    |
|---------------|-------------------------------------------|----------|
| AttributeFlow | EAV attribute defs + relations            | teal     |
| ShapeFlow     | a (partial) simplicial shape              | gold     |
| PatternFlow   | arsenal Template references + relations   | purple   |
| TabletFlow    | a cooked, emit-ready Shape Tablet         | ink/pale |

Ports are strictly typed; illegal connections are refused at wire time
(Houdini behavior). Every relation wire carries a W5H2/Sowa relation type
(GL-DDB-001), so the editor graph IS an EnkiDDB conceptual graph.

## 3. Node catalog (Shape Operators)

### Sources
- **TribeSource(tribe_id)** → ShapeFlow — loads a tribe's current canonical
  shape (or empty scaffold) from EnkiMDB over HEPT.
- **PatternSource(template κ)** → PatternFlow — an arsenal Template.
- **ShapeImport(.tmpl | inferred)** → ShapeFlow — DataStructure Station's
  inferred shape from a sample file, or a .tmpl from disk.

### Attribute operators (intra-tribe: your first relation class)
- **AttributeDefine(name, type, w5h2_role, optional=true)** → AttributeFlow
  — an EAV Client Optional Attribute (never a KAKI byte).
- **AttributeRelate(rel_type)** (A-in, A-in) → AttributeFlow — attr↔attr
  edge within the tribe; rel_type ∈ {DERIVES, CONSTRAINS, CO-OCCURS,
  UNIT-OF, IDENTIFIES, DESCRIBES} mapped to Sowa canonical relations.
- **Constraint(expr)** — range/unit/regex law attached to an attribute;
  becomes a station validation rule verbatim.
- **CompositeBind(k fields)** → ShapeFlow — declares a k-simplex.

### Cross-tribe operators (Tribe A ↔ Tribe B: your second class)
- **CrossTribeRelate(rel_type)** (ShapeFlow A, ShapeFlow B) →
  ShapeFlow — attribute-level mapping between tribes; cooks into
  CrossTribe-KAKI mapping particles validated at ingest.
- **PuhuExchange** (ShapeFlow A, ShapeFlow B) → ShapeFlow — asserts
  same-entity identity under exchanged representation (four costumes,
  one customer); grounded in PH-002 pending the Architect's GL-MDM-001
  Clause 7 ruling. Cook REQUIRES the two-witness verdict (structural
  Jaccard + embedding proximity) above threshold, else refuses.

### Pattern operators (pattern ↔ pattern: your third class)
- **PatternRelate(rel_type)** (P-in, P-in) → PatternFlow —
  rel_type ∈ {SPECIALIZES, PRECEDES, EXCLUDES, CO-OCCURS}; tunes the
  matcher (e.g. EXCLUDES sharpens disambiguation between near patterns).
- **PatternBindShape** (PatternFlow, ShapeFlow) → ShapeFlow — declares
  which lifecycle patterns this shape's particles are EXPECTED to follow;
  deviations route to tickets.

### Topology & diagnostics
- **ComplexAssemble** (ShapeFlow…) → ShapeFlow — union of inputs into one
  complex (fan-splits composites, dedups edges).
- **BettiProbe** → passthrough + side panel — draft β₀/β₁/β₂ + verdict
  (HEALTHY / HOLD / REVIEW) with the ETL diagnosis table.
- **GapWatch(expected…)** → passthrough — HeptaMap Gap detection.
- **OrbitBivector(a, b)** → passthrough — sets the shape's plane e_a∧e_b;
  drives the RD rotor preview live.

### Outputs
- **ShapeTablet** (ShapeFlow) → TabletFlow — cooks the canonical contract:
  fields, relations, cross-tribe mappings, pattern bindings, constraints,
  betti, gaps, bivector, HeptaScript block. Always stamped DRAFT.
- **StationBind(station)** (TabletFlow) — declares which DataStructure
  Station instance enforces this tablet (per client landing pipeline).
- **Seal** (TabletFlow) — the CSR-08 node: routes over HEPT for GeoEngine
  verification, Lamassu certification, Gate G4 proof, AAOL emission.
  The ONLY node with side effects; everything upstream is pure.

## 4. Cooking law (what makes it Houdini, not a macro recorder)

1. **Determinism** — same graph + same inputs ⇒ byte-identical tablet.
   All randomness seeded from the graph's KAKI hash.
2. **Dirty propagation** — a param edit dirties only downstream; recook
   touches the dirty chain; upstream cooked buffers are cached
   (CPU structs; GPU buffers for previews).
3. **Purity boundary** — every node except Seal is side-effect-free;
   HEPT reads (TribeSource, PatternSource) are cached snapshots with a
   witnessed timestamp, so cooks are reproducible offline.
4. **Cycle law** — the wire graph is a DAG; cycles refused at connect
   time. (β₁ loops belong INSIDE the shape being modeled, never in the
   operator graph modeling it.)
5. **Recipe-as-particle** — serialize graph → tablet with full KAKI
   identity, version, release era, approval; stored in the arsenal.
   `SourceHouse → AttributeRelate… → CrossTribeRelate → BettiProbe →
   ShapeTablet → Seal` replayed at Client-B is one click, same
   governance. The .hip file, elevated to constitutional status.

## 5. Command path into the ETL chain

    cooked ShapeTablet (DRAFT)
      → Seal node (CSR-08) → HEPT
      → GeoEngine verifies algebra · Lamassu certifies persistence
      → Gate G4 (Z3, design-time only) → AAOL emits .akk + contract
      → DataStructure Station loads the CANONICAL:
          fields/types/roles        → conform-or-reject per file
          attribute relations       → referential checks per station
          constraints               → value validation rules
          cross-tribe mappings      → CrossTribe-KAKI emission validated
          pattern bindings          → two-witness matcher expectations
          betti + gaps              → HOLD / REVIEW / ticket routing
          bivector                  → the tribe's BIGRING geometry
      divergence at ingest → one-cause-one-ticket (GL-TKT-001)
      → MADANU decree → append (GL-DST-003) → StoryEngine remembers.

## 6. Build milestones

1. `pdm-graph-core` crate: model + cook + Betti + tablet, with CLI test
   (`pdm-cook graph.json → tablet.json`) — provable before any UI.
2. gdext skin: GraphEdit canvas, 6 nodes (TribeSource, AttributeDefine,
   AttributeRelate, ComplexAssemble, BettiProbe, ShapeTablet).
3. RD rotor preview wired to OrbitBivector.
4. CrossTribeRelate + PatternRelate + Seal (mock HEPT).
5. egui-snarl skin over the same core on the Fedora VDI.
6. Recipe serialization + arsenal registration.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
