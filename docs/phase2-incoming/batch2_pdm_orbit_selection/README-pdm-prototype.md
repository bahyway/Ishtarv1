# DubSar PDM Prototype — Particle Data Modeling GDExtension
**Simplicial Complex + TDA + Geometric Algebra → Data Shapes that control
the BeeMDM ETL Processing Stations Chain**

BahyWay.Ecosystem v4.0 · rehearsal/prototype grade · implements the PDM
Modeler lens of GL-DST-001. Production sealing routes math to GeoEngine /
LamassuEngine over HEPT (stage never truth); the extension computes DRAFT
diagnostics for authoring feedback only.

---

## 1. The mathematical trinity

### 1a. Simplicial complex — the skeleton
A client data shape is authored as a complex K:
- **0-simplices (vertices):** fields (each with name, type, W5H2 role)
- **1-simplices (edges):** declared relations — key references,
  co-occurrence constraints, unit/derivation links
- **2-simplices (triangles) and up:** composites — a record type binding
  its fields, a nested object, a mandatory attribute trio

This matches the sealed design-time pipeline: contents → simplicial
complex in PDM → Z3 proves Θ at Gate G4 → AAOL emits tablet + HeptaScript.

### 1b. TDA — the health reading (draft here, Lamassu for the seal)
Boundary matrices over Z2: ∂1 (edges×verts), ∂2 (tris×edges).
With ranks r1 = rank(∂1), r2 = rank(∂2):

    β0 = V − r1          connected components
    β1 = E − r1 − r2     independent loops
    β2 = T − r2 − r3     voids (r3 = 0 when no tetrahedra declared)

ETL diagnosis table (the shape's Betti vector joins its contract):

| Reading      | Meaning in the shape            | Station consequence            |
|--------------|---------------------------------|--------------------------------|
| β0 > 1       | disconnected field islands      | HOLD at STAGE — unreachable fields cannot be validated from keys |
| β1 > 0       | referential loops               | Rework-Loop risk flagged BEFORE data flows; PROVE gate tightens |
| expected simplex absent | HeptaMap Gap         | blind spot inscribed; steward ticket on first ingest |
| β2 > 0       | enclosed void in composite web  | over-constrained nesting; schema review advised |

### 1c. Geometric Algebra — the motion law
Each tribe's orbit is a bivector B = e_a ∧ e_b in Cl(7) (a plane of Hepta
Space). Placement is the rotor sandwich:

    R = exp(−B θ/2) = cos(θ/2) − sin(θ/2) e_ab      (B unit, B² = −1)
    x(θ) = R x0 R̃

Because e_ab acts as a rotation only in its own plane, the GPU evaluates it
as a 2×2 rotation on the (a,b) coordinates of x0 — exact GA, cheap SIMD.
This is the SAME bivector orbit encoding sealed for Buzu chunks
(GL-VIZ-001): the shape tablet CARRIES its bivector, and every renderer —
Godot RD here, WGPU on the Fedora VDI — merely obeys it. The shape decides
the geometry of its own BIGRING.

## 2. How the Shape controls the ETL chain

`emit_shape_tablet()` produces a tablet with:
1. **Fields + types + W5H2 roles** → the DataStructure Station's validation
   contract (conform silently / reject to ticket per GL-TKT-001)
2. **Relations + composites** → referential checks per station
3. **Betti vector + gap list** → routing law: which anomalies HOLD, which
   PROVE-gate, which ticket
4. **Orbit bivector (a,b) + band radii** → BIGRING geometry for this tribe
5. **HeptaScript contract** (Anti-SQL), e.g.:

```
PRESENT SHAPE ClientA_Customer
  WHO   tribe   = 0007
  WHAT  fields  = 12 · relations = 14 · composites = 5
  HOW   betti   = (1, 0, 0)          // one component, no loops: healthy
  WHERE orbit   = BIVECTOR e2^e6     // Belonging ∧ OrbitalPosition plane
  WHY   gaps    = NONE
EMIT ShapeTablet SEAL AkkadianSeal
```

Sealing path: draft (this extension) → HEPT → GeoEngine verifies the
algebra, Lamassu certifies persistence at scale → Gate G4 (Z3, design-time
only) → AAOL emits .akk + station contract. The canonical shape then IS the
contract (GL-MDM-001 Clause 4).

## 3. Files

    pdm_modeler.h / pdm_modeler.cpp   — the GDExtension node (authoring API,
                                        Z2 Betti draft, RD compute dispatch,
                                        tablet emission)
    ga_orbit.comp.glsl                — rotor placement compute shader
    (register_types / SConstruct: follow the DubSar GDExtension build guide,
     Steps 1–2; this module drops in as one more registered class.)

## 4. Build & first run

1. Scaffold per the build guide (godot-cpp, SConstruct, register
   `PDMModeler`).
2. Compile `ga_orbit.comp.glsl` to SPIR-V (`glslc -fshader-stage=comp`),
   ship the .spv beside the library, load via
   `rd.shader_create_from_spirv()`.
3. In a test scene:

```gdscript
var pdm := PDMModeler.new()
pdm.define_field("customer_id", "UUID", "WHO")
pdm.define_field("name",        "TEXT", "WHAT")
pdm.define_field("city",        "TEXT", "WHERE")
pdm.link_fields("customer_id", "name")
pdm.link_fields("customer_id", "city")
pdm.add_composite(["customer_id", "name", "city"])
print(pdm.compute_draft_diagnostics())   # {b0:1, b1:0, b2:0, gaps:[]}
pdm.set_orbit_bivector(2, 6)             # e2 ^ e6
pdm.spawn_orbit_preview(20000)           # RD compute places the ring
print(pdm.emit_shape_tablet())           # JSON + HeptaScript contract
```

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
