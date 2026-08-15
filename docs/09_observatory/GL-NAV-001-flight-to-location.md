# GL-NAV-001 — Flight-to-Location Law

> **Extracted from `docs/phase2-incoming/batch1_compareengine_jaccard_flight/pb-301-flight-to-location-law-seal.yml`
> during Phase 2 integration (2026-08-14)**, per the PB-197+ convention:
> sealed design docs are committed directly into `docs/`, not deployed by
> an Ansible `copy:` task to a remote `$HOME` path. Content is verbatim
> from that playbook's embedded document; only the delivery mechanism
> changed. See `docs/phase2-incoming/RENUMBERING_MAP.md` for how this ID
> was confirmed against two other Phase 2 claimants on `GL-NAV-001`
> (batch 2's Knowledge-Graph Navigation law, now `GL-NAV-002`; batch 6's
> Hendursaga Charter, now filed as this law's Annex A —
> `docs/09_observatory/GL-NAV-001-AnnexA-hendursaga-charter_DRAFT.md`).

**Status:** SEALED (concept). Implementation BLOCKED behind
PB-150..PB-160 testing, per the Governing Law.

## 0. Telos
A person with no computer experience should reach a location by
SPEAKING, not by dragging a map with their fingers. The system
understands a vague sentence, then FLIES them down to the place
from a recognizable height — showing the neighborhood on the way,
so they arrive already oriented. Removing the map controls is the
feature, not a limitation.

## 1. The Three-Engine Contract
One sentence flows through three engines, each with ONE job.
The boundaries between them are law; no engine may absorb another.

### 1.1 Nabû — Semantic Search Engine (SSE)
- INPUT: a natural-language sentence, in Arabic or any tongue.
- WORK: embed the sentence (fastembed-rs, offline); keep the
  GROUNDED fragments (a bearing, a landmark, a name, a time band);
  explicitly DROP the sensory/episodic fragments that correspond
  to no stored feature ("sad music", "it felt cold").
- OUTPUT: a TARGET — a resolved particle or a bounded candidate
  set — plus a visible record of what was used and what was set
  aside. Honesty is emitted, not hidden.
- LAW: Nabû never retrieves a feature the data does not contain.
  A memory is not a query. When the sentence grounds nothing,
  Nabû yields to the Najaf narrowing loop (question-by-question
  geometric masking), it does not fabricate a match.

### 1.2 NaviEngine — the plotter
- INPUT: Nabû's target.
- WORK: compute the descent PATH from galaxy altitude to the
  target; gather the NEIGHBORHOOD set — the named anchors around
  the target with their bearings and distances (the mosque to the
  south, the poet's tomb NW, the school 55 m east).
- OUTPUT: a flight path + a neighborhood set. Nothing rendered.
- LAW: NaviEngine plots over GeoEngine's coordinates; it never
  computes truth. The neighborhood is what orients a lost person —
  not the pin alone, but what surrounds it.

### 1.3 Hubble-Zooming — the flight
- INPUT: NaviEngine's path + neighborhood.
- WORK: fly the camera down the path through the sealed zoom
  ladder (Galaxy → Cluster → Orbit → Particle → Target/Event),
  streaming LOD so each level is fed by the right StoryEngine /
  EnkiDB query; labels and lanes fade in as altitude drops.
- OUTPUT: the cinematic descent, ending on the pulsing target.
- LAW: Hubble renders; it holds no truth. The zoom ladder is the
  existing six-level Hubble telescope (Galaxy…Atom), reused here.
  A zoom step IS a geometric mask over the field — the same
  operation as the narrowing loop, one dense pass, sub-second.

## 2. Guarded Boundary I — Layered Basemap
The world already mapped the surface of the Earth. We do not
rebuild it; we STAND ON it and own the layer above.

- APPROACH (city → gate): a public open-data basemap
  (OpenStreetMap; open licence permits offline tile caching and
  self-hosting — sovereignty preserved). Google imagery is
  REJECTED for production: its licence forbids the offline
  caching and derivative use our doctrine requires.
- INTERIOR (gate → lane → plot/node): the sovereign Hubble layer,
  fed by EnkiDB. This is the layer no public map has — Wādī
  al-Salām's interior lanes and 80,272+ grave particles, or the
  water net's junction/defect nodes. THE DATA MOAT LIVES HERE,
  not in the basemap.
- HAND-OFF: the transition from public basemap to sovereign layer
  is an explicit LOD boundary, inscribed per flight, not an
  accident of zoom. Above the hand-off: OSM tiles. Below it:
  sovereign vector geometry + EnkiDB particles. The flight must
  never descend from a real street view into an empty rectangle;
  at the hand-off altitude the sovereign interior takes the frame.

## 3. Guarded Boundary II — Truth
GeoEngine remains the single mathematical truth source. Nabû
resolves language, NaviEngine plots paths, Hubble renders frames —
none of the three computes a truth value. Any coordinate, defect
classification, or plot identity is GeoEngine's, transported over
the HEPT binary protocol. A renderer that computes truth is a
violation of this law.

## 4. Guarded Boundary III — Two Performance Laws, Not One
The impressive flight and the billion-particle law are DIFFERENT
problems and are guaranteed separately:
- FLIGHT law: camera interpolation over the small VISIBLE set,
  ~60 fps, eased (ease-in-out). Cheap. About animation, not
  throughput.
- FIELD law: what Hubble HOLDS beneath the descent — the
  1-billion-particles-under-1-second law governs LOD streaming and
  masking of the full field, so the descent stays smooth when the
  field is enormous. This is the real engine's hard part.
Conflating the two hides the actual engineering risk. They are
sealed as separate guarantees so neither masks the other.

## 5. Domain Neutrality
One pipeline, many fields. The SSE→Navi→Hubble contract is
domain-neutral; only the field and the anchor vocabulary change:
1. NajafEngine — a grave in Wādī al-Salām (anchors: shrine,
   mosque, famous tombs; bearings from a mourner's memory).
2. WPDEngine / EGDEngine — a defect node in a water or
   electricity net (anchors: pumping station, school, junction;
   symptoms from a citizen's complaint).
The same code flies both. The domain is a parameter, not a fork.

## 6. Rendering Home & Sovereignty
Per Way-of-Work rule 5 and GL-DST-001, the PRODUCTION flight lives
in the Godot stage (DubSar Theater), not in a browser. HTML builds
of this pipeline are REHEARSAL scaffolding only. Runtime is pure
sovereign Rust (forbid(unsafe_code)); offline-capable; tiles
cached locally, never fetched live in the sovereign path.

## 7. Candidate Rust Substrate (design note, not a commitment)
- Embedding (Nabû): fastembed-rs / rust-bert, ONNX, offline.
- Nearest-neighbour: hnsw_rs / instant-distance, held as a KAKI
  attribute in an EnkiDB orbit ring (no tree — "there is no tree
  in the Orbits").
- Basemap tiles (approach): OSM, self-hosted/cached; vector or
  raster tile pipeline TBD at build time.
- Interior + flight: Godot 4.7 witness renderer over sovereign
  vector geometry; LOD streaming from EnkiDB.
The substrate is honestly young for pure-Rust 3D geospatial;
a single cemetery/net interior is tractable precisely because it
is BOUNDED — no globe is required.

## 8. Authority Note
Advisory-only outputs never block (NINSUN/Namtila pattern).
CSR-08 Architect Sovereignty governs every seal. Nabû, NaviEngine,
and Hubble are stages and instruments; the Architect and GeoEngine
hold authority. Built with care for who uses it: for a grieving
person, the pipeline asks for a name, a direction, a landmark —
and does well with those — rather than promising that the feeling
of a memory will find the grave.

— Inscribed for DUB.SAR 𒁾, BahyWay.Ecosystem v4.0.
