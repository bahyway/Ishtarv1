# WPDEngine — Water Pipeline Diagnostic Engine

**Standalone component reference. Follows `docs/TRANSPARENCY_STANDARD.md`.
No claim below is asserted without a tag and a citation.**

## Correction (2026-07-24, same day as the original version of this doc)

The original version of this document, written earlier the same day, stated
**"WPDEngine does not exist as a `crates/` entry"** and tagged the whole
component 🧩 PARTIAL / design-and-prototype-only. That claim was wrong — a
real, substantial `crates/wpd-engine` already existed in this repository,
with commits dated 2026-07-08 and 2026-07-11 (`git log --oneline -- crates/wpd-engine`),
**predating this session entirely**. The error was mine: I evaluated only
the browser prototype and the algebra-discussion document the Architect
had just uploaded, without doing a full crate search first. The Architect
caught this by pointing at their own file browser. This correction is left
visible rather than silently fixed, per this standard's own discipline.

## Status: ✅ VERIFIED — real, tested Rust crate exists; NOT the same design as the browser prototype

`crates/wpd-engine`: 12 source files, **50 passing tests**
(`cargo test -p wpd-engine`), builds clean as a workspace member. It also
has a real consumer: `crates/dubsar-visualizer/src/panels/wpd.rs` renders
its 7 seeded segments as cards in an egui panel (`dubsar-visualizer` is a
separate, egui-based visualizer alongside the Godot-based `dubsar-theater`
— not the same UI).

**Important: this crate and the browser prototype (`WPDENGINE_prototype.html`,
still described below) are two different, unreconciled design lineages
under the same name.** The crate does not implement the prototype's
Dijkstra-graph-routing or SPH-billion-particle-mixing ideas at all — it's
a segment-registry + multispectral-scan + Enbilulu-junction-assessment
architecture instead. Neither document currently points the reader to the
other. Treat them as two proposals, not one system with a prototype and a
production port.

## What the real crate actually does

| Module | What's real | Citation |
|---|---|---|
| `defect.rs` | `DefectClass` (8 classes: ActiveLeak/Corrosion/Crack/JointFailure/Tuberculation/RootIntrusion/Subsidence/Healthy), score→class mapping, severity 0–100, per-class recommended action text, water-loss estimate for active leaks | ✅ VERIFIED, 5 tests |
| `spectral.rs` | 12-band multispectral model (4 VNIR + 4 SWIR + 4 TIR, real wavelengths 450nm–12000nm), thermal/moisture/corrosion/vegetation indices, weighted composite defect score, seed spectral signatures for water-leak/oil-leak/sewage-blockage | ✅ VERIFIED, 6 tests — physically-motivated formulas (NDVI-style vegetation index, TIR-band thermal anomaly), not fabricated |
| `junction.rs` | Consumes `bahyway_algebra::enbilulu` (Phi_Enbi weighted score, TIAMAT bands, Terru mechanism/cause diagnosis, Milu alert severity via the real `alert-engine`, Enbi horizon in weeks) rather than re-deriving the math locally — the crate's own doc comment states this is "the real call site proving WPDEngine consumes geo-engine rather than duplicating its math" | ✅ VERIFIED, 3 tests. The underlying `enbilulu.rs` carries its own honest source note: only the weights, thresholds, and the `baru_residual` factor are Architect-confirmed; three of five Phi_Enbi factor identities are placeholders pending a source document that isn't in this checkout |
| `priority.rs` | `RepairPriority` (Routine/Planned/Urgent/Emergency from severity), composite scheduling score = severity×0.50 + risk×0.30 + population density×0.20, descending sort | ✅ VERIFIED, 3 tests |
| `segment.rs` | `PipelineSegment` (material, age, diameter, lat/lon endpoints, status), `risk_score()` = defect×0.50 + material-susceptibility×0.30 + age×0.20, `PipelineRegistry` (flat store, lookup by id/sector, urgent filter) | ✅ VERIFIED, 5 tests — but see gap below: no adjacency/graph structure |
| `sector.rs` | 7 Baghdad sectors (Green Zone, Al-Kadhimiya, Sadr City, Karrada, Rashid, Al-Jadria, Al-Mansour) mapped to a heptagram/planetary naming scheme, each with its own KAKI tribe_id (0x3000–0x3006, following the same reserved-range precedent as EnkiduLLM/ESARHADDON) | ✅ VERIFIED, 6 tests |
| `kaki.rs` | 16-byte KAKI derivation from segment id + composite score + confidence + timestamp | ✅ VERIFIED, 4 tests — see gap below: different KAKI system from the rest of the ecosystem |
| `nav.rs` | `SiteAccess` cost model, a real, tested `haversine_m()` great-circle distance function | ✅ VERIFIED for the pieces that are real — see gap below: routing itself is stubbed |
| `seed.rs` | 7 real-coordinate (33°N 44°E) seeded Baghdad segments, one per sector, with plausible material/age/defect-score variety | ✅ VERIFIED, 5 tests — illustrative seed data, not sourced from an actual Baghdad utility survey |
| `domain.rs` | `PipelineType` (Water/Oil/Sewage), `PipeMaterial` defect-susceptibility ranking, a separate `DefectSeverity` 5-tier scale | ✅ VERIFIED, 3 tests — see gap below: appears unused outside its own tests |

## Real gaps, stated plainly

- **No graph/topology model.** `PipelineRegistry` is a flat `Vec` — no
  adjacency between segments, no junction connectivity graph. Despite the
  name, `RepairNavigator::route_to()` does not route: both its waypoints
  use hardcoded coordinates (`lat: 33.3400, lon: 44.3900` — identical for
  both entries), and `distance_metres` is a hardcoded `1_000` regardless
  of the segment. The comment says "real coordinates come from drone
  dispatch" — i.e. this is explicitly a stub. `haversine_m()` itself is
  real and tested, but nothing calls it from `route_to()`. This means the
  browser prototype's actual Dijkstra-over-a-graph routing (see below)
  has not been ported here at all.
- **Two unreconciled KAKI systems in this workspace.** `wpd-engine::kaki`
  derives a raw `nusku_engine::KakiPK` (`[u8; 16]`) directly from a hash
  function, with no minting, no `KakiRole`, and no tribe_id embedded in
  the KAKI bytes themselves (tribe routing lives as a separate field on
  `JunctionAssessment` instead). This is a different mechanism from
  `enkidb_kaki::KakiMinter`/`IdentityKaki`/`EventKaki` — the system this
  session's Error Registry/Journal work (`docs/components/ENKIMDB_REGISTRIES.md`)
  is built on. A `DefectEvent`'s KAKI today cannot be journaled through
  EnkiMDB's WriteNode/Journal without a bridge that doesn't exist yet.
- **Scoped to Baghdad only**, with a hardcoded 7-sector heptagram/planet
  naming scheme — not yet the generic "any city" framing the algebra
  discussion and the browser prototype both use.
- **`PipelineSegment::age_years()` hardcodes `2026u16`** as "now" instead
  of reading wall-clock time — silently wrong once the calendar moves on.
- **Sector `density_index()` values (0.30–1.00) are illustrative,** not
  sourced from real Baghdad census/utility data — treat them as relative
  weights for demonstrating the priority formula, not ground truth.
- **`DefectSeverity` (domain.rs) appears orphaned.** It's a separate 5-tier
  0–1 scale from `DefectClass::severity()` (defect.rs's 8-class 0–100
  scale); nothing outside `domain.rs`'s own tests calls
  `DefectSeverity::from_score`. Not necessarily a bug, but worth
  reconciling or removing before it causes confusion about which severity
  scale is authoritative.

## What the prototype (unchanged from before) still separately demonstrates

*(This section is retained from the original version of this document —
the prototype's content and its own honest limits are unaffected by the
correction above; what changed is only the surrounding claim that no real
crate existed.)*

Status: 🧩 PARTIAL — design + browser prototype only for this specific
lineage. What exists:

- `WPDENGINE_algebra_discussion.md` — 📄 DOCUMENTED. A conversation
  exploring which algebra/algorithm families can find structural patterns
  (leaks, contamination plumes, hidden pipe alignments) across a
  city-scale, billion-particle representation of water infrastructure.
- `WPDENGINE_prototype.html` — 🧩 PARTIAL. A self-contained,
  dependency-free browser JS/Canvas proof-of-concept, validating the
  *shape* of three proposed algorithms:

| Tab | What it shows | Tag |
|---|---|---|
| Dijkstra Grid Routing | Real O(N²) shortest-path over a randomly generated pipe network; leak-flagged nodes get a ×10 edge-weight penalty; live rerouting on click | ✅ VERIFIED — real algorithm, not ported to the Rust crate (see gap above) |
| Drone Multispectral Matrix | **Synthetic** NDVI/thermal grid from a fixed formula, not real image analysis — but conceptually now real-ish in the Rust crate's `spectral.rs` (12-band model with tested thermal/moisture/corrosion indices), just not fed by an actual drone/imagery pipeline yet | 🧩 PARTIAL both places, for different reasons |
| SPH Wave Particle Mixture | Real spatial-hash-accelerated (25px cells, 3×3 neighbor scan) local-diffusion simulation, ~450 particles, toy scale vs. the 1-billion target; has a known JS mid-iteration mutation bug, harmless in the prototype but won't compile as-is in Rust | ✅ VERIFIED at toy scale, not in the Rust crate |

## The algebra recommendation (from the discussion, partially realized)

| Tool | Recommended for | Status now |
|---|---|---|
| Geometric / Clifford Algebra | Spatial structure, planar alignments | ✅ VERIFIED real (`bahyway-algebra`'s Cl(7)/spinors/rotors) — and now concretely *used* by `wpd-engine::junction` via `bahyway_algebra::enbilulu`, though Enbilulu itself is a weighted-scalar model, not GA-native |
| Lie Algebra | Continuous symmetry, conserved quantities | ❌ NOT FOUND for this purpose |
| TDA / Hopf Algebra | Global topology — voids, filaments, contamination boundaries | ❌ NOT FOUND for this purpose — `lamassu-engine` exists for a different domain |
| Spinors | Rotational/orientation pattern detection | ✅ VERIFIED same `bahyway-algebra` primitives, not yet applied inside `wpd-engine` specifically |
| SPH + Spatial Hashing | Billion-particle fluid mixing at tractable cost | ❌ NOT FOUND as a crate — prototype only |

## Files

- `crates/wpd-engine/` — the real, tested Rust crate (this correction's subject)
- `crates/dubsar-visualizer/src/panels/wpd.rs` — its real UI consumer
- `crates/bahyway-algebra/src/enbilulu.rs` — the real math `junction.rs` consumes
- `WPDENGINE_algebra_discussion.md`, `WPDENGINE_prototype.html` — the separate prototype lineage

## Next steps

1. **Reconcile the two lineages** — decide whether the crate's
   segment/spectral/junction architecture supersedes the
   graph/SPH-particle vision, or whether they're meant to merge (e.g. the
   crate's `PipelineRegistry` gains real adjacency and the prototype's
   Dijkstra logic gets ported onto it). Right now a reader of either
   document has no way to know the other lineage exists.
2. Give `PipelineRegistry` real segment-to-segment adjacency and port
   `RepairNavigator::route_to()` off its hardcoded stub onto real
   graph search (the prototype's Dijkstra is the proven shape to port).
3. Bridge `wpd-engine`'s KAKI derivation to `enkidb_kaki`/EnkiMDB's
   WriteNode so `DefectEvent`s can be journaled through the same Error
   Registry/Journal machinery the rest of the ecosystem uses, instead of
   a parallel, un-integrated KAKI scheme.
4. Resolve or remove the orphaned `DefectSeverity` scale.
5. Only once the above is stable: generalize past Baghdad's hardcoded
   7-sector scheme toward the "any city" framing, and revisit the
   SPH/TDA/Lie-algebra pieces the discussion doc scoped as genuinely new
   work.
