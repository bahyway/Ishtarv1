# How BahyWay.Ecosystem v4.0 Works With TDA

**Sealed:** 2026-07-30; H2/relationship-complex-builder update 2026-07-31
**Scope:** every real place Topological Data Analysis (persistent
homology) touches this ecosystem — the math engine (now H0/H1/H2, §1/§6),
the sentinel that calls it, where its verdicts land in governance, EnkiDW's
shape-signature/trajectory/cross-tenant-comparison work, the new
relationship-based complex-builder feeding DubSar PDM's schema-discovery
paradigm (§6b), and — because the question that prompted this document
was "how do we differ from Ayasdi, not resemble it" — a corrected,
fact-checked comparison against Gunnar Carlsson's Mapper and the company
he built on it.
**Companion laws:** `docs/01_mathematics/GL-MRD-002-neberu-slicer.md` (Analysis-
to-Solution Law, DETECT→PROVE→PREDICT→PRESCRIBE), `docs/01_mathematics/
GL-MRD-003-orbit-spectral-diagnostics.md` (the complementary *rhythm*
axis — spectral statistics, not shape). This document does not restate
either; it shows how the real code underneath all three actually runs.

Every claim below is checked against a real file in this repo. Paths are
given throughout — if a future reader (human or otherwise) finds a claim
here doesn't match the code, the code is the truth and this document is
wrong, per this ecosystem's own no-fabrication discipline.

---

## 1. The math engine — GeoEngine

**File:** `crates/bahyway-algebra/src/persistence.rs`

This is the *sole* place persistent homology is computed anywhere in
BahyWay.Ecosystem (design law, sealed 2026-07-17 in the Particle Death &
Legacy Law discussion). Every other module that wants a topological
reading calls into this one; none recomputes homology itself.

- **Algorithm:** boundary-matrix reduction over GF(2) (Zomorodian &
  Carlsson, "Computing Persistent Homology," 2005), implemented directly
  over sorted `Vec<usize>` columns — pure `std`, zero external
  dependencies. Fully dimension-generic: a class's dimension is derived
  from its simplex's vertex count, never hardcoded to a triangle/edge
  case, so the same `reduce_filtration` function serves H0/H1/H2 without
  modification.
- **Filtration:** Vietoris-Rips, over a 3D point cloud (`Point3 = [f64;
  3]`) — plus, new as of 2026-07-31, `clique_complex_persistence`: the
  same reduction over an arbitrary caller-supplied weighted graph (edges
  in, not Euclidean distance), for complexes built from discovered
  business relationships rather than geometric proximity (§6b).
- **Scope, updated 2026-07-31: H0 (connected components), H1 (loops),
  AND H2 (voids) — the design's own stated ceiling ("downsampled
  representative cloud per Tribe per epoch, capped at H2") is now
  reached.** The gap this section used to describe (tetrahedra
  enumeration + a second boundary-matrix pass) turned out to need only
  the enumeration step — the reduction algorithm required zero changes,
  since it was already dimension-generic. Real, tested cost: tetrahedra
  enumeration is O(n⁴) versus triangles' O(n³), so H2-enabled callers
  must downsample more aggressively than an H0/H1-only scan would need.
  Proven on a genuine hollow shape, not just unit tests of the plumbing:
  `hollow_octahedron_shell_has_one_persistent_h2_void` (6 points on an
  octahedron's vertices — a real enclosed 3D void, one infinite-
  persistence H2 bar) and `filling_the_center_point_collapses_the_void`
  (add the missing 7th point at the center — the void count drops to
  0), both in `persistence.rs`'s own test module.

## 2. The sentinel — LamassuEngine

**File:** `crates/lamassu-engine/src/lib.rs`

LamassuEngine samples a Tribe's particles into that 3D point cloud (via
`bahyway_algebra::orbital::orbital_position`, PA-14 — each particle's
KAKI byte 12 becomes an azimuth, byte 14 an altitude, and its `delta`
quality-distance becomes an orbital radius) and asks GeoEngine for the
real persistence diagram. It never recomputes homology; it only samples
and classifies.

Three signatures, sealed in the Abiogenesis / BlackHoles-in-Orbits
design conversation:

| Signature | Meaning |
|---|---|
| **GOLDEN** | One loud, long-lived H1 bar — a genuine orbit announcing itself. |
| **FUZZY** | H1 bars exist but are short-lived relative to the scan radius — a seam, not yet a pattern. |
| **DEAD** | No persistent H1 at all — dust, no orbit (H0 only). |

Classification is a pure function (`classify`) over an already-computed
diagram: an H1 bar whose persistence exceeds `GOLDEN_PERSISTENCE_RATIO`
(0.3) of the scan's `max_epsilon` is GOLDEN; any H1 activity below that
is FUZZY; zero H1 classes is DEAD. Real, test-proven both ways: a full
16-point ring of particles reads GOLDEN, a 5-point arc segment that
never closes reads DEAD — `crates/lamassu-engine/src/lib.rs`'s own test
module.

**H2 is surfaced as evidence, not folded into the sealed 3-value
signature.** `TribeReading` (2026-07-31) now carries `void_count`/
`max_h2_persistence` alongside `component_count`, read straight from
GeoEngine's diagram — but `classify()` itself still reads H1 only.
That's deliberate, not an oversight: GOLDEN/FUZZY/DEAD is sealed to what
an *orbit* (a loop) looks like, and a Tribe's orbital sampling is a
ring/torus-like cloud by construction (see `full_ring_of_particles_is_
golden`'s own comment) — not shaped to reliably enclose a genuine 3D
void the way a deliberately-built relationship complex can (§6b). A
caller that wants void evidence reads `TribeReading.void_count`
directly; the three-value verdict's meaning is unchanged.

**Do not confuse this with `bahyway_core::ParticleState::Golden/Fuzzy/
Dead`.** Same three words, orthogonal axes. `ParticleState` is a
per-*particle* evidence-lifecycle verdict (is this one record still
valid) used throughout the ecosystem's EAV/StoryEngine machinery.
`TopologicalSignature` is a per-*Tribe* geometric verdict over the whole
particle cloud's shape. A Tribe full of individually-Golden particles
can still read topologically DEAD if they don't form a loop, and vice
versa. This distinction mattered enough that `crates/enkidw/src/
dw_analytics.rs`'s own module doc comment (§4 below) states it in
almost these exact words, because an earlier cross-session comparison
conflated them.

## 3. Where the verdict lands — governance, not exploration

**File:** `playbooks/playbook_190_nisaba_internal_interface_lamassu_tda.yml`

LamassuEngine's readings feed `NisabaOrchestrator`
(`crates/nisaba/src/orchestrator.rs`), which continuously joins
StoryEngine's high-priority alerts, the Data Steward's review queue, and
LamassuEngine's topological readings into one digest — under CSR-08's
propose-not-ratify discipline ("NISABA observes/orchestrates/proposes
continuously; nothing that changes ecosystem state executes without the
Architect's confirmation"). A GOLDEN/FUZZY/DEAD verdict is a signal an
automated system acts on or escalates, not a graph a human must first
sit down and read.

## 4. New: EnkiDW shape-signature and shape trajectory

**File:** `crates/enkidw/src/dw_analytics.rs` (this document's own commit)

Before this work, `DwAnalytics`/`DwReport` — EnkiDW's existing OLAP-style
analytics layer — produced state counts, epoch ranges, and top-N most
active particles, but no shape reading; LamassuEngine's math existed but
had no path into EnkiDW's own report. Closed by wiring the existing
engine in, not by inventing new math:

- **`DwReport.shape_signature: Option<lamassu_engine::TribeReading>`** —
  `None` from the plain `report()` call (shape-scanning costs O(n³), so
  it is opt-in, never silent). `DwAnalytics::report_with_shape(top_n,
  &lamassu)` populates it: samples the Tribe's current particle cloud
  and returns the real reading alongside the existing counts.
- **Delta proxy, stated honestly:** LamassuEngine's sampling needs a
  per-particle quality-distance (`delta`) to place it at an orbital
  radius. EnkiDW's mandatory EAV schema doesn't yet carry a continuous
  per-particle quality score generically across every Tribe, so
  `state_to_delta` uses a discrete three-tier stand-in derived from the
  one per-particle signal every Tribe already has —
  `ParticleState::Golden/Fuzzy/Dead` → 0.15/0.50/0.90. Documented in
  code as a proxy to replace, not a measured value.
- **`DwAnalytics::shape_trajectory(epoch_windows, &lamassu) ->
  ShapeTrajectory`** — the genuinely new capability an insert-only,
  daily-partitioned substrate makes possible that a one-shot exploratory
  tool structurally cannot offer: how a Tribe's shape reading changes
  across a sequence of epoch windows (e.g., one per daily partition).
  Each window samples every particle's state **as it was at that
  window's end epoch**, via `enkidb_engine`'s real time-travel
  projection (`project_at`, §3.4) — not repeated present-day snapshots.
  Particles that hadn't been minted yet by a given window are excluded
  from it (a Tribe that simply didn't exist yet reads as an empty
  window, never as DEAD).
- **`ShapeTrajectory::trend() -> ShapeTrend`** — `Emerging` /
  `Stable` / `Dissolving` / `Insufficient`, by comparing the first and
  last non-empty reading's signature. A rising trend means a real
  pattern is forming; a falling one is a real regression signal — the
  same "rate of change, not just current value" idea the Transparency
  Deficit Calculus design docs (§7) proposed for τ, applied here to
  shape instead.

13/13 real tests in `dw_analytics.rs` pass, including two that build
real GOLDEN/DEAD readings via `lamassu-engine`'s own point-cloud
technique (not synthetic stubs) to prove the trend classification
against genuine topology, not hand-picked enum values.

## 5. New: comparing shapes across a BIGRING boundary

**File:** `crates/lamassu-engine/src/lib.rs` (this document's own commit)

`ACROSS BIGRING ClientX | ALL` is a real HeptaScript v2.0 federation
clause (`crates/heptascript/src/lib.rs`) — a cross-tenant boundary a
query can span. No BIGRING transport, tenant registry, or cross-client
execution engine exists anywhere in this codebase, and none is invented
here. What's real and buildable now, without waiting for that
infrastructure, is the safe comparison primitive such a layer would call
once it exists:

```rust
pub fn compare_readings(a: &TribeReading, b: &TribeReading) -> ShapeComparison
```

`TribeReading` never carries raw KAKI bytes — only a tribe id, the
classified signature, the persistence diagram, and two counts — so
comparing two of them is structurally safe across a tenant boundary by
construction, not merely by promise. The comparison deliberately does
**not** treat component counts or sample sizes as numerically
comparable across tenants (different `LamassuEngine` configurations —
different `max_epsilon`/`r_max`/`h_max` — make those raw numbers
apples-to-oranges); the one thing every configuration already
normalizes to the same three-value scale is the classified signature,
so that's what `ShapeComparison.same_signature` actually compares. Two
tests prove both directions: two independently-scanned Tribes producing
matching GOLDEN readings, and a GOLDEN-vs-DEAD divergent pair.

## 6. H2 (voids) — closed 2026-07-31

H2 (β₂, voids) detects a genuinely different shape class from H1: a
hollow 2D enclosure in 3D — particles ringing an empty cavity — rather
than a 1D loop. This section used to record H2 as a deliberate, tracked
gap ("designed, not built"); it no longer is one. What actually closed
it, once `persistence.rs` was read end to end rather than estimated from
its module doc: the boundary-matrix reduction algorithm was *already*
fully dimension-generic (it derives a class's dimension from
`verts.len()`, never a hardcoded triangle/edge case) — the only missing
piece was the tetrahedra-enumeration step itself, mirroring the existing
triangle loop one dimension up, plus three new `PersistenceDiagram`
accessors (`h2_pairs`/`void_count`/`max_h2_persistence`). Not the "real
combinatorial jump" this document previously estimated it to be —
a correction made explicitly, not silently, when the estimate turned
out to be wrong.

The real, unavoidable cost is what this document's original recommendation
already named correctly: tetrahedra enumeration is O(n⁴) versus
triangles' O(n³), so an H2-enabled scan needs a harder downsample cap
than an H0/H1-only one. That cost argument justified deferring H2 for
LamassuEngine's *continuous* sentinel role (it still does — see §2's
"surfaced as evidence" note); it is much weaker for the *occasional*,
once-per-onboarding PDM-discovery use case this document's §6b covers,
which is what actually motivated closing the gap now rather than later.

## 6b. New: a relationship-based complex-builder — `pdm-discovery`

**Files:** `crates/pdm-discovery/src/lib.rs`, `bin/pdm-discover/`,
`crates/bahyway-algebra/src/persistence.rs::clique_complex_persistence`

H2 alone doesn't give a business-meaningful "missing data / structural
hole" reading unless the complex's edges encode actual relationships
(foreign keys, shared join keys) rather than geometric proximity —
LamassuEngine's only complex-building pipeline builds edges from
KAKI-derived orbital position, which is exactly geometric proximity, not
business meaning. Closing that gap needed a second, genuinely new
filtration path in GeoEngine itself:

- **`clique_complex_persistence(n_vertices, edges, max_weight)`** — the
  same enumeration-and-reduce machinery as `vietoris_rips_persistence`,
  refactored to share one `reduce_filtration` function, but triggered by
  an arbitrary caller-supplied weighted graph instead of Euclidean
  distance. Proven topologically identical to the geometric path on the
  same shape, expressed as a graph instead of coordinates:
  `clique_complex_octahedron_graph_has_one_persistent_h2_void` /
  `clique_complex_adding_the_center_vertex_collapses_the_void`.
- **`pdm-discovery`** — a v1, explicitly-scoped relationship-detection
  heuristic: profiles every column of uploaded tabular data
  (`profile_columns`), detects candidate join-key relationships by
  **exact value-overlap only** (`detect_join_keys`, gated by
  `MIN_CANDIDATE_CARDINALITY` against coincidental low-cardinality
  matches like boolean/status columns), then builds one graph vertex per
  table and hands the discovered relationship graph to
  `clique_complex_persistence` (`discover_schema` → `SchemaProposal`).
  Named limitations, in the crate's own module doc: this catches
  exact-match shared keys and nothing else — it misses renamed/
  reformatted keys, transformed keys (hashes, zero-padding), and any
  semantic relationship with no literal value overlap. Every
  `CandidateRelationship` it emits is a proposal for human review, never
  an applied schema change.
- **`bin/pdm-discover`** — a CLI front end (`table_name=path.csv` args →
  `SchemaProposal` JSON on stdout), smoke-tested end to end against real
  CSV files in this session, not just unit-tested in isolation.
- A chordless 4-table relationship ring (`discover_schema_chordless_
  four_table_cycle_is_one_persistent_h1_loop`) reads as a genuine
  persistent H1 class — "these tables only relate to their immediate
  neighbours, no anchor table ties them together" — a real, business-
  meaningful structural signal, not a synthetic demo of the math.

**Where this lands in DubSar PDM IDE:** a Conceptual/Logical/Physical
split was added to the PDM tab (`godot/dubsar-theater/scripts/
pdm_node_graph.gd`, `pdm_tab.gd`), mirroring ERwin/Archimate's data-
modeling layers — Conceptual wires which tables feed discovery (a new
`SCHEMA_DISCOVERY` node type, shelling out to `bin/pdm-discover` via
`OS.execute` since no GDExtension bridge into this workspace's Rust
crates exists, confirmed still absent), Logical reviews/approves
individual candidate relationships (never auto-applied), Physical
compiles the approved table set into real `.akk` — one block per table,
since `AkkadiCompiler`'s grammar supports one entity per query and no
real multi-table JOIN query capability exists in HeptaScript today (a
fabricated join syntax was deliberately not invented to paper over that).
**Honest status: statically reviewed only.** No Godot runtime was
available in the session that built this, so none of the GDScript above
has been exercised live — verify by hand before relying on it.

**Governance laid down, not yet enforced in code:** two ADRs
(`docs/14_decisions_adr/adr_015_sla_supremacy_and_structural_
amendment.md`, `adr_016_model_as_particle.md`) record the law for what
happens when a fresh discovery run disagrees with an already-approved
schema (propose a DUAL-sealed amendment, never apply one unilaterally —
modeled on this ecosystem's own CSR-08 and the `death_legacy.rs`
"advisory, never automatic" precedent) and for what an approved
`SchemaProposal` ontologically *is* (a KAKI-bearing Template particle,
following ADR-014's mint/supersede pattern exactly). Both ADRs are
explicit that they record the law, not its enforcement machinery: no
code yet stores an approved proposal as a persistent, versioned,
KAKI-minted record, diffs a fresh run against a prior one, or drives a
ratification workflow. `pdm_tab.gd`'s "Approve Selected" is in-memory
only today.

## 7. What this is *not* — a note on "Transparency Deficit Calculus"

A design conversation (not yet code — filed on the Architect's own
machine, `docs/__DialyWorks/Friday20260619/HeptaScriptv2.0/`) proposes a
HeptaScript v2.0 feature that computes the *financial cost of opacity*
in a **client's data domain** (e.g., a hospital's GDPR compliance
posture), with the cost-per-hour conversion factor sealed bilaterally in
an SLA KAKI. That is a real, well-reasoned, but entirely separate idea
from anything in this document — it audits a client's *business*
opacity, not this ecosystem's own topological blind spots. An earlier
cross-session comparison conflated the two ("your Transparency Deficit
Calculus measures what the topological instrument hides"); it does not,
and as of this writing it does not exist in code at all. Keep the two
concepts distinct in any whitepaper or academic material that cites
both.

## 8. Convergence and divergence — BahyWay contra Ayasdi, corrected

Gunnar Carlsson co-created Mapper and founded Ayasdi to commercialize
it. Asking whether BahyWay resembles Ayasdi is really asking whether
this ecosystem independently arrived at the architecture TDA's own
founder built when he productized it. Genuine convergence exists — both
draw on the same mathematics (persistent homology / Mapper), and both
treat "the shape of data carries meaning" as a real design principle.
The divergences matter more, and here they are stated without the
factual slips an earlier comparison introduced. One correction to this
document's own earlier text, made explicitly: it used to say "LamassuEngine
computes H0/H1 only, never a third Betti number" — true of
`TopologicalSignature`'s sealed 3-value contract (§2), but no longer true
of GeoEngine itself: `bahyway-algebra::persistence` computes H0/H1/H2 as
of 2026-07-31 (§6), and a second filtration path
(`clique_complex_persistence`, §6b) computes all three over discovered
business-relationship graphs, not just geometric point clouds. (Also
still true and unrelated to that correction: Enlil Algebra's Jordan-Block
tribes are a zero-cross-talk, mutually-*exclusive* partition — the
structural opposite of Mapper's deliberately-*overlapping* cover, not a
resemblance to it.)

| | Ayasdi / Mapper | BahyWay.Ecosystem v4.0 |
|---|---|---|
| **Where TDA lives** | Analytics layer, bolted onto conventional storage the customer already had | Native: LamassuEngine samples EnkiDB-family particles directly; EnkiDW's own `DwReport` now carries a shape reading as a report field, not an export |
| **What it's used for** | Exploring *unknown* datasets for undiscovered structure (drug discovery, fraud) | Governing *known*, self-generated Tribe data — a continuous sentinel, not a one-shot analyst tool |
| **Output** | A graph a human interprets | A three-value verdict (GOLDEN/FUZZY/DEAD) an automated system (NisabaOrchestrator, StoryEngine alerts) acts on directly |
| **Temporal model** | One shape, one export, one point in time | `shape_trajectory` — a real reading per epoch window, classified as EMERGING/STABLE/DISSOLVING, made possible specifically by the insert-only/daily-partitioned substrate Mapper was never run against |
| **Cross-dataset comparison** | One customer, one dataset, one run | `compare_readings` — comparing sealed, anonymized signatures across a BIGRING tenant boundary, never raw particles |
| **Provenance/trust** | A suggestive graph; trust it or don't | Ed25519-sealed reports (`shakkanakku/src/report.rs`), KAKI-Identity per element, no delete/APPEND-supersedes law (ADR-006) throughout |

**The historical record, checked:** Ayasdi became a SymphonyAI
portfolio company in May 2019 (renamed "Symphony AyasdiAI"); the
"Sensa" brand specifically dates to August 2022, not 2019 — two events
an earlier comparison conflated into one date. ([SymphonyAI](https://www.symphonyai.com/news/financial-services/symphony-ayasdiai-becomes-symphonyai-sensa-2/),
[PR Newswire](https://www.prnewswire.com/news-releases/symphony-ayasdiai-becomes-symphonyai-sensa-301608175.html))
TDA-as-a-standalone-product proved a hard business even for the field's
own founder — the honest lesson for positioning is directional, not
discouraging: don't sell "topological data analysis" as the product;
sell the sovereign platform (BeeMDM) with TDA as the moat underneath,
exactly as this ecosystem already does by wiring the math into EnkiDW's
own report rather than shipping it as a separate analytics tool.

---

## Summary — what's real, what's designed, what's a different thing entirely

| | Status |
|---|---|
| H0/H1/H2 persistent homology (GeoEngine) | **Real, built, tested.** `bahyway-algebra::persistence` — H2 closed 2026-07-31, see §6. |
| GOLDEN/FUZZY/DEAD sentinel (LamassuEngine) | **Real, built, tested.** `lamassu-engine`. Still H1-only by sealed design; H2 surfaced as separate evidence (`TribeReading.void_count`), see §2. |
| NisabaOrchestrator governance wiring | **Real, built.** `crates/nisaba`, PB-190 |
| Orbit-spectral rhythm diagnostics (GUE/Poisson) | **Real, built, tested** — the complementary, non-topological axis. `orbit-spectral-engine`, GL-MRD-003 |
| EnkiDW shape-signature (`report_with_shape`) | **Real, built, tested.** |
| Shape trajectory (`shape_trajectory`/`ShapeTrend`) | **Real, built, tested.** |
| Cross-BIGRING shape comparison (`compare_readings`) | **Real, built, tested.** Comparison primitive only; no federation transport exists. |
| Relationship-based complex-builder (`clique_complex_persistence`, `pdm-discovery`) | **Real, built, tested — new 2026-07-31.** v1 exact-match join-key heuristic; see §6b for named limitations. |
| DubSar PDM Conceptual/Logical/Physical tabs | **Built 2026-07-31, statically reviewed only** — no Godot runtime available to exercise it live. See §6b. |
| SLA Supremacy & Structural Amendment (ADR-015), Model-as-Particle (ADR-016) | **Law decided and documented 2026-07-31; enforcement/minting code NOT built.** See §6b. |
| BIGRING federation transport | **Not built anywhere.** Only the HeptaScript clause and the comparison primitive exist. |
| Transparency Deficit Calculus | **Designed, not built, and a different feature entirely** — see §7. Do not conflate with this document's topic. |
