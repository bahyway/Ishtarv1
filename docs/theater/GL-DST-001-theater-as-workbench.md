# GL-DST-001 — Theater-as-Workbench Law
**Status:** SEALED (concept). Implementation deferred until BeeMDM ETL
General/Specific testing completes.
**Surface:** DubSar Theater (Godot 4.7) — the ONLY sovereign
IDE of BahyWay.Ecosystem v4.0.

## 1. Law Statement
DubSar Theater is the end-to-end mathematical workbench of
the ecosystem: one continuous surface spanning design-time
and runtime as two acts on the same stage. In Triple-O,
mathematics and visualization were never separate —
everything is particle, so the mathematics IS the visible
object. Theater is where the Architect inhabits that fact.

## 2. The Continuous Arc (one stage, seven scenes)
1. **PDM Modeler lens** — simplicial complexes composed
   visually; floors, sectors, stakeholders become vertices
   and faces.
2. **Gate G4 monitor lens** — Z3 certification of the
   stakeholder equation Θ watched live (design-time only;
   Z3 never ships in the sovereign binary).
3. **Tablet lens** — the .akk inscription born from AAOL
   Core, its NUZI genealogy chain visible.
4. **ORBIT 3D lens** — composed orbit density (BIGRING,
   13M+ GPUParticles3D), H attribute driving radius,
   ColourID B11 hues on the 240 scale.
5. **GRID lens** — tabular/EAV inspection (existing).
6. **SECTION lens (Nēberu)** — draggable slice plane,
   Poincaré crossings, per GL-MRD-002.
7. **WAVEBAND panel** — the seven harmonics under metric
   g = diag(w₁…w₇); solution candidates (DETECT → PROVE →
   PREDICT → PRESCRIBE, per GL-MRD-002 Rev. 2) reviewed
   and accepted or rejected by the Architect here.

## 3. Full Advantage over Conventional Tools (MATLAB class)
- **No compute/plot divide:** the model, the proof, the
  render, and the analysis are one continuous object.
- **Proof before existence:** nothing reaches runtime that
  Z3 did not certify at Pre-Template (Gate G4).
- **Witnessed provenance:** every act — a modeled complex,
  a slice, a solution candidate — is inscribed as an
  immutable KAKI tablet. Reproducible by law, not habit.
- **Sovereign & offline:** pure-Rust engines, no cloud,
  no license server, no telemetry. EriduOS-native.
- **One language:** HeptaScript (Anti-SQL) drives every
  lens; no MATLAB-script/plot-API split.

## 4. Guarded Boundary — Stage, Never Truth
Theater renders, orchestrates lenses, and captures the
Architect's gestures. It NEVER computes mathematical truth.
**GeoEngine** (= `bahyway-algebra`, confirmed 2026-07-10 — the
crate the sealed concept documents mean by this name) remains
the single mathematical truth source; MardukEngine, WPDEngine,
and the EnkiDB pipeline (EnkiSDB→EnkiODB→EnkiQDB→EnkiDB→
EnkiDW→EnkiMDB→EnkiDDB, ports 7001–7007) perform all
computation. Any Godot-side calculation of truth values is a
violation of this law. Query transport remains the HEPT binary
TCP protocol (magic 0x48455054).

**Pipeline-order note (added on landing, 2026-07-11):** the EAV
lifecycle ordering above (SDB→ODB→QDB→DB→DW→MDB→DDB) is one of
several orderings that have circulated across this ecosystem's design
documents — the "É-DUBBA gate sequence" has at least three mutually
inconsistent tellings recorded elsewhere (see
`docs/CLOSING_SUMMARY_28_DOCUMENTS_VERIFIED_2026-07-07.md`), still
awaiting one authoritative Architect ruling. This law's use of the
ordering above should not be read as resolving that open question.

## 5. Authority Note
The workbench inherits all standing authority laws:
advisory outputs never block (NINSUN/Namtila pattern);
CSR-08 Architect Sovereignty governs every seal (sealed as
governance law; not yet expressed as a coded ConEngine rule
alongside CSR-01–07 as of 2026-07-11); runtime obeys the
1-billion-particles-under-1-second law.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.

## 6. Rev. 2 Amendment — Statistical Witnessing & GPU Deployment
Constraint (sealed 2026-07-27)

**Origin:** this amendment answers a real, previously-undocumented gap
found while checking whether Godot could actually render the BeeMDM ETL
chain in realtime. Two already-real GDScript scripts —
`orbit_multimesh.gd` (`MultiMeshInstance3D`, real live TCP query to
`enkidb-query-server`, real B11-classified particle colour) and
`bigring.gd` (`GPUParticles3D`, real TIAMAT-alert-driven colour) — both
work correctly today, but both cap at **80,272** particle slots (a fixed
`MAX_PARTICLES` constant). This law's own §2, item 4 (ORBIT 3D lens)
already sealed the target as "composed orbit density (**BIGRING, 13M+
GPUParticles3D**)." No renderer
on earth draws 13M+ (let alone the 1-billion-particle retrieval law's
full scale) as individually addressable GPU instances in realtime — that
gap was real and open, not a rendering-engine limitation Godot itself
could be blamed for.

**6.1 Statistical Witnessing (the resolution).** The ORBIT 3D lens
renders a **representative subsample**, not the full live population,
whenever live particle count exceeds a render budget (default
**1,000,000** — a real, documented number, not "as many as fit"):
- The subsample is **stratified by OrbitalSubtype/B11 bucket**
  (GOLDEN_GEM, GOLDEN_ALIVE, FUZZY_AGED, FUZZY_GRAY, FUZZY_DECAY,
  DEAD_EXPIRED, DEAD_SEALED — the same seven buckets `orbit_multimesh.gd`
  and `bigring.gd` already classify by), preserving each bucket's real
  proportion of the full population. Density, per-bucket colour mix, and
  velocity distribution are exact statistical projections of the full
  set — not an arbitrary or biased sample.
- This is the same principle GL-MRD-002 already uses for the SECTION
  lens (a faithful section of the full orbit space stands in for the
  whole) — Rev. 2 applies it to the ORBIT 3D lens instead of inventing a
  second pattern.
- **Drill-down is exact, not statistical.** Zooming into a station or a
  single κ particle hands off to the GRID lens (existing) or a future
  SECTION lens query (GL-MRD-002) against the real Read Node — the
  representative subsample is an overview-scale rendering choice only,
  never the record of truth. GeoEngine remains the sole math-truth
  source per §4 of this law, unchanged.
- Every ORBIT 3D render at overview scale is WITNESSed with its own
  subsample parameters (budget, per-bucket counts, sampling seed) so the
  choice of "which particles were shown" is an inscribed, reproducible
  fact — never a silent rendering-layer decision.

**6.2 GPU Deployment Constraint (a real, previously-unflagged risk).**
Theater (Godot 4.7) targets GPU-resident particle rendering
(`GPUParticles3D`, `MultiMeshInstance3D`) at the scale §6.1 describes.
`eriduous-vdi` — the fleet's real control node and the natural home for
the DubSar IDE (PB-226) — is a **genuine KVM-hosted VM** (confirmed
repeatedly across this fleet's own real operational history: hypervisor
reboots, `virsh`-managed lifecycle). Rootless-container concerns aside,
KVM's virtual GPU path (`virtio-GPU`/`virgl`) throttles GPU particle
counts badly compared to a real GPU. This is a **deployment decision
that must be made before Theater's ORBIT 3D lens is built for real**, not
discovered afterward as a performance regression:
- **Option A** — run Theater on the bare Fedora host directly (outside
  any VM), if the host is otherwise free for it.
- **Option B** — configure real PCI GPU passthrough to `eriduous-vdi`
  (requires IOMMU support, a dedicated/available GPU, and libvirt XML
  changes — none of this is automated by any playbook in this repo
  today).
- **Option C** — accept `virtio-GPU`/`virgl` and budget §6.1's
  representative-subsample count down further to whatever that path can
  actually sustain at 60fps, measured for real before committing to a
  number in this law.
No option is chosen by this amendment — this section exists so the
decision is made deliberately, with the real constraint named, instead
of being discovered mid-build.

— Amendment inscribed for DUB.SAR 𒁾, sealed 2026-07-27.
