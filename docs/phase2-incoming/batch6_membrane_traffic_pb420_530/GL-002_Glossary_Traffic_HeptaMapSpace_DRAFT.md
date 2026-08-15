# GL-002 · The BahyWay Glossary · Traffic & HeptaMapSpace Arc (2026-08-11)
## DRAFT — Ḫubullu discipline: every sealed name beside its plain word. Unsealed until the Architect runs.

*Companion to GL-001 (Zagesi Rev). Covers the Sila Calculus, the census, the Hubble descent, the HeptaMapSpace membranes, the new HeptaScript clauses, and the established algorithms adopted into the stack.*

---

## I. The Sila Calculus (GL-NAV-002 candidate · Ḫendursaĝa's second court)

| Sealed term | Plain gloss |
|---|---|
| Sila Calculus | sila = street; the junction-and-bottleneck mathematics of parallel pipeline grids |
| the fundamental diagram | Greenshields q(ρ)=v_max·ρ(1−ρ/ρ_j); throughput rises to a **crest**, then falls |
| the crest law | stay left of the crest — past it, admitting more moves less; the rigorous form of the ladder |
| junction / street | a shared resource (core, gate, commit) / a pipeline stage |
| jam island (β₀) | a connected component of jammed streets — how many fires, not how big |
| gridlock loop (β₁) | a 1-cycle in the jam complex: vehicles waiting in a closed loop — deadlock as a hole |
| persistence FORMING | a loop alive below the density threshold — gridlock predicted before it closes |
| curl bruise | Hodge circulation around a block — the precursor of the β₁ loop, the warning before the hole |
| max-pressure | Tassiulas–Ephremides control: serve greatest queue-pressure differential; throughput-optimal by theorem |
| Wardrop equilibrium | selfish per-pipeline routing; provably worse than the coordinated optimum |
| Price of Anarchy | measured throughput ÷ best coordinated throughput; bounded (≤4/3 linear latencies) |
| Braess probe | close a shortcut; if flow improves, the street was a paradox (added capacity harmed the grid) |

## II. The Census & Scenarios (typed particles)

| Sealed term | Plain gloss |
|---|---|
| typed KAKI particle | a vehicle as a soul with an EAV vehicle_class (car/taxi/moto/bus/truck/pickup) |
| PCE weight | passenger-car-equivalent: a bus ≈ 2.5 cars, a truck ≈ 2.8, a motorcycle ≈ 0.4 |
| weighted density | ρ = Σ(PCE)/capacity, not a headcount — why a street of trucks reaches its crest sooner |
| reconciliation (PROVE totals) | live modal split vs a calibration target; the Δ discipline; max deviation reported |
| calibration placeholder | the target modal mix, labelled PLACEHOLDER until a real survey arrives via BeeMDM ETL |
| junction types | signalized · roundabout · priority/yield · grade-separated interchange · checkpoint (metered) |
| closure / checkpoint | first-class controls that reshape routing — the substrate every scenario needs |
| ambulance green-wave | pre-emption particle requesting green; verdict = delay vs free-flow (signal pre-emption / EVP) |
| military convoy | coupled formation behind a cordon of closures; verdict = formation-through |
| police pursuit | two-particle chase; verdict = interception probability vs the congestion field |
| flood egress | mass re-route to safe exits; verdict = percent cleared before the window, bounded by the crest |

## III. The Hubble Descent & the Hilbert Anchor

| Sealed term | Plain gloss |
|---|---|
| Hubble descent | one continuous LOD camera: Iraq → Baghdad Province → Districts → Street web, no cuts |
| altitude / LOD | each zoom band reveals its layer; streets thin as you rise (minor only at street altitude) |
| Hilbert anchor H0 | the HeptaSpace home = the Hilbert-curve centroid index of Baghdad center |
| the locality idea | nearby on the curve = nearby in space = **contiguous in KISPU storage** — one nearness |
| worst junction | the busiest node, auto-lit so the eye finds the bottleneck without reading a number |
| City Map Template | districts+arterials+crossings as a GL-TPL shape; future cities instantiate from it (PB-520) |

## IV. The HeptaMapSpace (the 7D membrane court)

| Sealed term | Plain gloss |
|---|---|
| HeptaMapSpace | the 7-dimensional space of all souls (uuid/tribe/time/checksum/EAV/orbit/colour) |
| membrane | any two of the seven dimensions chosen as the 2-D viewing plane |
| cluster / accumulation | a tribe agreeing on the membrane's two axes, forming a dense knot |
| the Uniqueness Law | no two souls share the exact 7D position (templates too) — the reach guarantee |
| the reach guarantee | density never hides a soul; it only asks for more zoom, because there is always room between two points |
| Hilbert micro-order | a deterministic sub-pixel offset per soul giving the camera a path into the densest knot |
| focus lens | a fisheye that spreads dense accumulation under the cursor without losing context |
| parallel membranes | small multiples of other dimension-pairs; a tribe tight here may spread there |
| the binding dimension | the axis on which a tribe is tightest — the dimension that carries its cause |

## V. New HeptaScript clauses (HS-EXT · drill-and-narrate grammar · PB-529)

| Clause | Plain gloss |
|---|---|
| `CLUSTER … BY` | group souls into an inspectable body by a dimension or attribute (the Vineyard op, general) |
| `ZOOM INTO … ALONG` | the Hubble camera as a query verb — drill one axis of a cluster |
| `SIMULATE … WHAT-IF` | the counterfactual as first-class: open a freight-only street, measure the crest shift + Braess check |
| `PROVE optimal … OVER` | search a small config space (e.g. junction type) for the best — **Z3 design-time only** (Gate G4) |

*Rule: geometric and counterfactual nouns, no new verbs, no new language — the Marduk discipline (as HS-EXT-001/002).*

## VI. Established algorithms adopted into the stack (honest provenance)

| Algorithm | Layer · role |
|---|---|
| Space-filling curve (Hilbert) | storage/compute: locality-preserving order + domain decomposition across nodes |
| Cell Transmission Model (Daganzo) | simulation: discretized LWR — the calibratable bridge to real traffic physics (PB-523) |
| MFD + perimeter control (Geroliminis–Daganzo) | management: whole-city crest law; gating = shed-at-the-gates, published (PB-524) |
| Max-pressure (Tassiulas–Ephremides) | management: throughput-optimal signal/junction control (PB-518) |
| ALINEA | management: classic deployed ramp-metering |
| DBSCAN + Getis-Ord Gi* | detection: incident hotspot clustering & significance; risk surface, not point-prediction (PB-525) |
| Hungarian / min-cost matching + dynamic pricing | optimization: OPSAM parking assignment; SFpark/Shoup (PB-526) |
| Queueing (M/M/c, priority) + facility-location | checkpoints: delay/queue analytics + placement |
| Max-flow/min-cut + betweenness centrality | resilience: critical-link and high-value-corridor identification |
| Contraction Hierarchies | routing: millisecond shortest paths at scale |
| Kalman / particle filters | assimilation: the digital-twin loop — correct the sim against sparse real sensors |

*Honest framing: the ecosystem's contribution is the **coherence** — one particle representation and one query language across storage, physics, topology, and control — not any single algorithm. Hilbert indexing sits *under* ITMS/OPSAM/networks, not against them.*

*Provenance: DRAFT, grade P0 · The Fadam Floor applies · Nothing herein is law until run and confirmed.*
