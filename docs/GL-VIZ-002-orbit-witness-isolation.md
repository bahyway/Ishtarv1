# GL-VIZ-002 — Orbit Witness & Isolation Law
**Status:** SEALED (concept). Implementation queued behind PB-160.
**Stage:** DubSar Theater, ORBIT 3D lens (GL-DST-001).
**Feeds on:** Buzu binary chunk artifacts (GL-VIZ-001).
**Escalates to:** Nēberu Slicer / SECTION lens (GL-MRD-002).

## Clause 1 — Pick (O(1) Witness Resolution)
Every particle on the BIGRING stage is pickable in O(1),
independent of total particle count, via an ID buffer:
each particle writes its KAKI identity hash into an
offscreen render pass; a right-click (context gesture)
reads exactly one pixel and resolves to the particle's
KAKI (uuid_hash kappa[0..3], tribe_id kappa[4..5]) plus
its EAV state. Picking NEVER iterates the particle set.
This clause is what keeps the interaction identical at
thirty thousand and at one billion particles.

## Clause 2 — Scope (Exactly Three)
The witness context menu offers exactly three scopes:
  1. WITNESS PARTICLE — the picked particle alone.
  2. SELECT ORBIT     — all particles sharing the picked
     particle's tribe_id AND EAV state (same colour,
     same station).
  3. SELECT COHORT    — all particles sharing the picked
     particle's EAV state across all orbits.
Orbit and cohort selection resolve as Buzu chunk-key
filters (chunks are keyed by tribe/state); selection is
a metadata operation, never a particle scan.

## Clause 3 — Isolate (Hubble Mode of the ORBIT 3D Lens)
A selection may be promoted to isolation. Isolation is
NOT a new lens: it is the ratified operating mode of the
existing ORBIT 3D lens ("Hubble Mode"). In Hubble Mode:
  - Only the selected orbit's Buzu chunk(s) stream into
    a dedicated SubViewport, re-projected face-on.
  - Progressive LOD zoom applies within the isolated
    chunk; the main stage is never re-filtered.
  - The memory ceiling of Hubble Mode is one orbit's
    chunk at maximum LOD — this is the budget clause
    that preserves the 1-billion-particle law.
  - If the isolated band remains too dense to read,
    escalation is the SECTION lens (Nēberu Slicer,
    GL-MRD-002) — Poincaré section through the band.
The Theater retains exactly seven lenses.

## Clause 4 — Stage, Never Truth
Selection is witnessing. Every pick, scope selection,
and isolation EMITs an immutable SelectionEvent KAKI
particle into EnkiODB (WHO tribe, WHAT state, WHERE
HeptaShell zone, HOW scope, HOWMUCH view window) and
mutates nothing. Godot renders and captures gestures;
GeoEngine remains the single mathematical truth source.
Sovereign semantics, Anti-SQL:

    WITNESS ORBIT
      WHO      tribe  = kappa[4..5] OF picked
      WHAT     state  = EAV.State OF picked
      WHERE    shell  = HeptaShell(picked)
      HOW      scope  = WHOLE_ORBIT
      HOWMUCH  window = VIEW HubbleMode
    EMIT SelectionEvent INTO EnkiODB

## Authority Note
Inherits all standing law: advisory outputs never block
(NINSUN/Namtila pattern); CSR-08 Architect Sovereignty
governs every seal; runtime obeys the 1-billion-
particles-under-1-second law; the void between orbit
shells is sacred (Nisaba, TUPSARRU doctrine).

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.

- name: "Register Hubble Mode amendment against GL-DST-001 (ORBIT 3D lens)"
ansible.builtin.lineinfile:
path: "{{ gl_dst_001_file }}"
line: ">> AMENDMENT (PB-185): The ORBIT 3D lens gains one operating mode — Hubble Mode (orbit isolation per GL-VIZ-002 Clause 3). Lens count remains seven."
create: false
state: present
