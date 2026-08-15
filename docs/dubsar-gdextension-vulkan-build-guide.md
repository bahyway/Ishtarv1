# DubSar Theater GDExtension — Vulkan/RenderingDevice Build Guide
**BIGRING picking · orbit isolation · bench stabilization · decree editing ·
StoryEngine registration · Houdini-style procedural graph**

BahyWay.Ecosystem v4.0 · implements GL-VIZ-002, GL-DST-002/003, GL-STY-001.
Law reminders wired throughout: the stage never computes truth (GL-DST-001 §4);
edits APPEND via engines (GL-DST-003 §2); picking is O(1) (GL-VIZ-002 §1).

---

## Step 0 — Where Vulkan actually lives in this build

Inside Godot 4.x you do not call `vkCreateDevice`. Godot's **RenderingDevice
(RD)** API is a thin Vulkan-shaped layer: storage buffers, compute pipelines,
framebuffers, samplers, barriers. Every recipe in the Kosarevsky cookbook maps
onto it one-to-one; what the book gives you is the mental model of what each RD
call costs underneath (descriptor updates, pipeline binds, sync). Rule of thumb:

| Vulkan concept              | RenderingDevice equivalent                      |
|-----------------------------|--------------------------------------------------|
| VkBuffer (SSBO)             | `rd.storage_buffer_create()`                     |
| VkShaderModule + pipeline   | `rd.shader_create_from_spirv()` + `rd.compute_pipeline_create()` |
| vkCmdDispatch               | `rd.compute_list_begin/dispatch/end()`           |
| Offscreen pass + attachment | `rd.framebuffer_create()` with R32_UINT texture  |
| vkCmdCopyBuffer             | `rd.buffer_copy()`                               |
| Readback                    | `rd.texture_get_data()` / `rd.buffer_get_data()` |

Use raw Vulkan (or Rust `ash`) only if you later build outside Godot; the
Fedora-VDI egui path stays WGPU and shares the same concepts.

## Step 1 — Scaffold the GDExtension

1. Clone `godot-cpp` matching your Godot 4.7 build; set up SConstruct.
2. Create one module: `dubsar_theater/` with classes:
   - `BigRingRenderer` (Node3D) — owns buffers, passes, camera modes
   - `WitnessPicker` — ID pass + readback + scope resolution
   - `MadanuBench` — freeze/snapshot + edit staging
   - `HeptClient` — TCP client for HEPT (magic 0x48455054)
   - `TheaterGraph` (GraphEdit host) — the procedural graph (Step 9)
3. Register in `register_types.cpp`; verify the extension loads in the editor
   with a debug print before writing any GPU code. (Half of GDExtension pain
   is build plumbing; prove it first.)

## Step 2 — Particle data layout (the Buzu chunk as SSBO)

One SSBO per Buzu chunk; chunks are keyed by (tribe_id, state) — this keying
is what makes orbit/cohort selection a metadata operation later.

```glsl
// std430 — 32 bytes/particle, GPU-friendly, KAKI-faithful
struct Particle {
    uint  uuid_hash;    // κ[0..3]
    uint  key;          // κ[4..5] tribe << 16 | κ[6] type << 8 | κ[7] role
    float angle;        // orbital phase
    float radius_rel;   // 0..1 within the orbit band
    float height;       // ecliptic jitter
    uint  state_palette;// PASHIRU palette index (EAV-derived — NEVER κ bytes)
    uint  flags;        // bit0 SELECTED · bit1 FROZEN · bit2 DIMMED · bit3 BENCH
    uint  dwell_days;
};
```

At 1B particles total, chunks stream: only bound-house chunks resident
(GL-DST-002 §2), Hubble Mode holds exactly one chunk at max LOD (GL-VIZ-002 §3).

## Step 3 — Compute pass A: the orbit integrator

One dispatch per resident chunk, local_size_x = 256:

```glsl
void main() {
    uint i = gl_GlobalInvocationID.x;
    Particle p = particles[i];
    if ((p.flags & FROZEN) == 0u)                 // ← stabilization is a bit
        p.angle += orbit_speed(p.key) * dt;
    positions[i] = project_bigring(p, cam);       // writes vec4 world pos
    particles[i].angle = p.angle;
}
```

The FROZEN check is the entire "stabilize the section" mechanism at the
physics level. No forces, no springs — a bit the integrator respects.

## Step 4 — Draw pass: instanced points from the SSBO

Two options, in order of effort:
1. **MultiMeshInstance3D** with a shader that reads the positions SSBO via a
   sampler/texture buffer — fastest to ship, fine to ~5–10M points.
2. **RD indirect draw**: vertex-less point pipeline pulling from the SSBO,
   with a GPU-written `VkDrawIndirectCommand` per chunk — the cookbook's
   indirect-draw chapters, and the road to the billion-particle budget.

Color = palette lookup from `state_palette` (Hepta axis 7 via PASHIRU/EAV).
Dimming for scope emphasis = alpha from `flags & DIMMED`.

## Step 5 — The ID pass: O(1) picking (GL-VIZ-002 Clause 1)

1. Second render target: `R32_UINT` texture, same projection, each particle
   writes `chunk_base + i` (its global index) as the "color."
   With MRT you can emit it in the same pass as the beauty draw.
2. On right-click: `rd.texture_get_data()` of a 1×1 region under the cursor.
   Never read the same frame synchronously — request at frame N, read at
   N+1 (or use a 3-frame ring) to avoid a pipeline stall.
3. Index → CPU-side chunk directory → full κ identity + EAV state.
   Total cost: one pixel. Identical at 30K and at 1B particles.

## Step 6 — Scopes without iteration

Picked particle gives (tribe, state). Selection scopes never loop on CPU:
- **WITNESS PARTICLE**: set SELECTED on one index (tiny buffer update).
- **WHOLE ORBIT / STATE COHORT**: one compute pass over resident chunks:
  `p.flags = matches(p.key, sel_key) ? p.flags|SELECTED : p.flags|DIMMED;`
- **Chunk-level shortcut**: because chunks are keyed by (tribe,state), most
  scope math is "which chunks" — skip entire dispatches/draws for
  non-matching chunks. Selection is metadata, per law.
Every scope change EMITs a SelectionEvent (Step 8).

## Step 7 — Isolation (Hubble Mode) and the Bench (stabilization)

**Hubble Mode** — a `SubViewport` with its own camera:
- Draw only the matching chunk(s); animate camera tilt 0.36→1.0 and scale
  over ~60 ticks (the flight); wheel drives zoom; LOD ladder =
  density texture → point sprites → full particles (three strata,
  matching the Buzu strata decision).

**The MADANU Bench** — when the steward opens a batch for work:
1. Compute pass sets FROZEN|BENCH on the scope (live ring keeps flowing
   for everything else).
2. `rd.buffer_copy()` the scope's particles into a **bench SSBO**
   (a GPU snapshot — Vulkan's vkCmdCopyBuffer, one barrier).
3. The bench view re-projects frozen particles into a **calm lattice**
   (grid layout by dwell or pattern) so edits never chase a moving target.
4. On bench close: clear flags; the live buffer was never disturbed.
This is Houdini's freeze-node semantics: downstream works on a cooked,
stable copy while upstream keeps simulating.

## Step 8 — Editing + the only lawful commit path

The SSBO is a projection. The edit path is one-directional:

```
UI edit / verdict
  → compose HeptaScript SYNC decree (WHO/WHAT/WHERE/WHEN/WHY/HOWMANY)
  → HeptClient sends over HEPT to the WRITE node (700x)
  → engines APPEND new state via KISPU (GL-DST-003 §2)   ← truth changes HERE
  → StoryEngine events written (GL-STY-001 ontology):
       WITNESS family: SelectionEvent (pick/scope/isolate)
       DECREE  family: per verdict, with decree id + WHY note
  → HEPT confirmation returns
  → ONLY NOW: update the projection SSBO (small compute patch or re-stream
    the chunk) and refresh census counters (witnessed from node ENLIL rings)
```

If HEPT fails, the projection never changes and the bench shows the decree
as REJECTED — the stage can never drift ahead of truth.
`HeptClient` implementation: plain non-blocking TCP in C++ (or a thin Rust
sidecar speaking HEPT natively, exposed to the extension over a local socket —
keeps the protocol code in one sovereign Rust crate).

## Step 9 — The Houdini-style procedural graph (TheaterGraph)

Build on Godot's **GraphEdit/GraphNode** controls. Node vocabulary (SOP-like):

```
[SourceHouse]  → binds a house (Tupsimati rite; GL-DST-002)
[FilterState]  → WHAT state = …            (GPU flag pass or chunk cull)
[FilterDwell]  → WHEN dwell > Nd           (GPU flag pass)
[FilterPattern]→ arsenal match cohort      (HEPT → CompareEngine)
[IsolateOrbit] → Hubble Mode on scope
[Bench]        → freeze + snapshot (Step 7)
[Edit/Verdict] → PROMOTE|REWORK|KILL|HOLD + WHY note
[CommitHEPT]   → decree execution (CSR-08 confirm dialog lives HERE)
[WitnessEmit]  → explicit StoryEngine event node (auto-inserted after
                 every scope/commit; visible so provenance is on-canvas)
```

Cooking semantics (this is what makes it Houdini and not a macro recorder):
- Each node's cook is **deterministic**: same inputs → same scope/result.
  Filters compile to either a HeptaScript fragment (sent over HEPT) or a
  GPU flag pass — the node stores which.
- **Dirty propagation**: editing a node marks downstream dirty; recook only
  the dirty chain. Upstream cooked results (bench snapshots, scope masks)
  are cached GPU buffers.
- The graph serializes to a tablet (JSON now; .akk via AAOL later), gets
  KAKI identity, and lives in the arsenal — **a procedural recipe is a
  particle too**, versioned and steward-approved, reusable across clients
  exactly like a .tmpl. A saved graph like
  `SourceHouse(EnkiQDB) → FilterState(FUZZY) → FilterDwell(>30d) →
  Bench → Verdict(PROMOTE) → CommitHEPT`
  is a repeatable steward rite — one click, same governance, every week.

## Step 10 — Build order (milestones you can verify)

1. Extension loads; empty Node3D prints hello.            (day 1)
2. 100K static particles from one SSBO via MultiMesh.     (week 1)
3. Integrator compute pass; ring rotates; FROZEN bit works.
4. ID pass + 1px readback; console prints κ on right-click. (the keystone)
5. Scope flag pass; dim/bright; SelectionEvent stub logged.
6. Hubble SubViewport flight on isolate.
7. Bench: freeze + buffer_copy + lattice re-projection.
8. HeptClient round-trip against a mock WRITE node; decree → append →
   projection patch. StoryEngine events real.
9. TheaterGraph MVP: Source → Filter → Bench → Commit, with dirty recook.
10. Indirect draw + chunk streaming + LOD ladder — the road to 1B.

Steps 1–5 are two to four weeks of evenings and give you the complete
witness loop; everything after is scale and comfort.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
