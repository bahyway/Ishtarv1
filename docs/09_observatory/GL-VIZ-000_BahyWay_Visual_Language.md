# GL-VIZ-000 (candidate) — THE BAHYWAY VISUAL LANGUAGE
## Phase Two Animation Vocabulary · The Toolkit Godot Instruments Draw From
### BahyWay.Ecosystem v4.0 · Phase Two (GL-STD-002) · Status: SEALED-CONCEPT (per CSR-08 chat confirmation, 2026-08-15)

---

## 0 · Why This Tablet Exists

The ecosystem is built in order: **internal first** (Input → Processing →
Streaming → Output; every GOLDEN record scored and made available), **Enterprise
Instruments after** (medical, civil). But the *visual language* those instruments
will speak must be researched now, so it is ready when they are. Godot is the
chosen instrument-builder; Triple-O the architecture philosophy.

This tablet consolidates the animation vocabulary invented in Phase Two into one
reusable toolkit. Each pattern is a **motion primitive with a fixed meaning** —
so every future Godot instrument speaks the same language, and a viewer who
learns it once reads any BahyWay instrument.

Governing principle (inherited, binding): **every motion means a sealed number.**
Morphology proposes; the algebra proves (GL-VIZ-001 D-1). No primitive renders a
value it cannot cite.

---

## 1 · The Motion Primitives (the vocabulary)

| # | Primitive | Meaning (the sealed quantity) | First proven in |
|---|-----------|-------------------------------|-----------------|
| P-1 | **Hubble Dolly** — one camera, one spline, cosmos→detail, no cuts | scale-invariance; zoom = change of SCOPE (HS-EXT-003) | BIGRING court |
| P-2 | **Fractal Descent** — diving into a node reveals the same law one level down, endlessly | recursion of the seven laws; person→tribe→federation→union | BIGRING-of-BIGRINGs |
| P-3 | **Decay Colour** — green→amber→grey over time | GOLDEN→FUZZY→DEAD state (EAV trichotomy) | Living Shape Cosmos |
| P-4 | **ε-Jitter** — trembling amplitude | uncertainty ε (higher ε = more tremor) | Inner Life of Data |
| P-5 | **τ-Membrane** — shell thickness around a body | transparency deficit τ | Inner Life of Data |
| P-6 | **Kinesin Transport** — a carrier walks cargo along a track | a dependency edge, delivering data where needed | Inner Life of Data |
| P-7 | **Outward Drift** — a particle migrating to larger radius | cost accumulation (radius-as-instrument law) | Living Shape / Igigi |
| P-8 | **Golden Ascent** — sealed copies rising to a core sun | only GOLDEN reaches the Single Point of Truth (GL-FED-001) | BIGRING Golden Sun |
| P-9 | **Seizure & Detention** — a particle yanked from flow into a cage | Nergal verdict → Ṣibittu quarantine | Nergal Gate |
| P-10 | **Focus Disc (the Lens)** — a region that de-blurs on contact | resolving FUZZY data; ε made local and reducible | Fuzzy Lens |
| P-11 | **Two-Voice Facing** — two testimonies rendered apart with the gap between | Birītu gap (never averaged) | (Birītu court, pending) |
| P-12 | **Betti Alarm** — a glowing closed loop | β₁ topological alarm (a cycle that shouldn't be open) | Igigi / Shape Atlas |
| P-13 | **Uniqueness Bounce** — a selected particle bounces in place on a KAKI-derived phase | restores Hepta uniqueness lost to 2D projection; prevents masking at billion-scale | Nergal Gate (jailed UNKNOWN) |

---

## 2 · Composition Rules

- **C-1 · Primitives compose; they don't conflict.** A single instrument may run
  P-1 + P-3 + P-4 + P-5 at once (fly through a decaying, jittering, τ-membraned
  cosmos). The Living Shape Cosmos is P-1·P-2·P-3·P-4·P-5·P-7 composed.
- **C-2 · One meaning per primitive.** Colour is always state; jitter is always
  ε; thickness is always τ. A viewer never re-learns a primitive per instrument.
- **C-3 · Every primitive is cite-able (D-2).** In production each rendered
  primitive links to the sealed EAV/topology value that drove it — hover in
  prototype, provenance in Godot.
- **C-4 · Reduced-motion honesty.** Every primitive has a static form that
  preserves meaning (colour/thickness survive; motion is the enhancement).
- **C-5 · The Uniqueness Bounce Law (P-13).** Derived directly from the Hepta
  Space Uniqueness Law: no two particles share a 7D position at the same time —
  but **projection 7D→2D collapses that guarantee**: two genuinely distinct
  particles can land on the same screen pixel, so a selected particle in a dense
  cluster is masked by its neighbours. At billion-scale this is not an edge case;
  it is the norm in any dense region. The bounce restores uniqueness in the one
  dimension projection did not consume — **time**: the selected particle moves on
  a periodic path while all others hold still, and motion pop-out (pre-attentive
  in human vision) makes it impossible to lose even when its pixel is shared.
  - **C-5a · Phase from KAKI.** The bounce phase MUST derive from the particle's
    KAKI (already unique), so that even co-located, co-selected particles bounce
    out of phase and never re-collapse into one visual mass. Uniqueness in the
    data drives uniqueness on the screen.
  - **C-5b · Selection-only.** The bounce is a selection signature, not an
    ambient state; an unselected particle never bounces (ambient motion is
    reserved for P-3/P-4/P-7, which carry their own meanings). Deselect returns
    it to stillness.

---

## 3 · The Godot Mapping (how the language ships)

Each primitive becomes a reusable Godot node/shader the instruments instantiate:

| Primitive | Godot form |
|---|---|
| P-1 Hubble Dolly | Camera3D on a Curve3D with SCOPE-tier LOD swap |
| P-3 Decay Colour | shader uniform `age` → gradient; driven by EAV state |
| P-4 ε-Jitter | vertex/compute jitter, amplitude = `epsilon` uniform |
| P-5 τ-Membrane | translucent shell mesh, radius = base + `tau` |
| P-6 Kinesin Transport | PathFollow3D carriers along dependency Curve3D |
| P-8 Golden Ascent | GPUParticles3D emitting only state==GOLDEN |
| P-10 Focus Disc | screen-space post-process, de-blur radius at cursor |
| P-13 Uniqueness Bounce | per-instance transform offset, phase = hash(KAKI); selected set only |

WGPU compute for the billion-particle scale (DubSar Visualizer lineage). The
prototypes (Šala HTML) are the choreography reference; Godot is the body.

---

## 4 · The Sequencing This Serves

- **Now**: internal ecosystem (I→P→S→O), GOLDEN scoring, the playbook program +
  TESTING_PHASE1. The visual language is researched in parallel, not ahead of.
- **Then**: Enterprise Instruments (medical Fuzzy Lens, civil services) built in
  Godot, drawing every motion from this sealed vocabulary.
- The Fuzzy Lens is the **target** that specifies what the internal ecosystem
  must ultimately deliver (real dated readings, thresholds, resolved statistics);
  this tablet is the **language** that target will be rendered in.

## 5 · Codex Compliance
- **A-1 zero new mathematics**: pure consolidation; every primitive maps to an
  existing sealed quantity. New is only the *catalogue and the Godot mapping*.
- **A-4**: cites GL-VIZ-001/002 · GL-FED-001 · HS-EXT-003 · GL-DDB-002 ·
  GL-MED-002 · decay-vs-rite · radius-as-instrument.
- Registers as the parent of the V-series patterns in GL-STD-002 §3.

## 6 · Open seals for CSR-08
GL-VIZ-000 adoption · the twelve primitive names as canonical · the Godot node
mapping as the sanctioned build path · whether P-11/P-12 need their own proving
court before sealing.

*Recorded in the reign of Gudea 1.0, Phase Two. Sealed under CSR-08 (chat confirmation), 2026-08-15 — see the Seal section below.*

## 7 · Seal

```
Sealed by: DUB.SAR 𒁾 (Bahaa Fadam), via explicit chat confirmation (CSR-08)
Date:      2026-08-15
AkkadianSeal (Ed25519): PENDING — no real signing infrastructure wired
                        yet (no Sargon/Gilgamesh passport ceremony run
                        against this tablet). The chat confirmation above
                        is the Architect's real CSR-08 act; the
                        cryptographic seal is separate, real follow-on
                        work, not fabricated here.
```
