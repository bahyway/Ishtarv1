# BC-ENV-001 — Enbilulu Calculus (Enbi)

**BahyWay.Ecosystem v4.0 — Architecture Design Reference**

| Field | Value |
| :---- | :---- |
| ADR ID | BC-ENV-001 |
| Title | Enbilulu Calculus — Water Net Deformation & Pre-Leak Detection |
| Short name | Enbi |
| Patron deity | 𒀭𒂗𒉈𒇻 Enbilulu — lord of rivers and canals, divine canal inspector, son of Enki |
| Status | SEALED (design) — build QUEUED behind v4.0 playbook completion law |
| Date sealed | 2026-07-07 |
| Author | DUB.SAR 𒁾 — Architect of BahyWay.Ecosystem |
| Style lineage | BC-SEC-001 (UrNammu Engine ADR) |
| Target component | EnbiluluEngine (EnkiDW analytical layer) |
| Governing law | No build before Phase A (PB-90–98) → Phase B (PB-99–109) → TESTING\_PLAYBOOK\_PHASE1 (Blocks A–F) → BeeMDM 50-zip ETL test |

---

## 1\. Context

Water distribution failures in municipal SCADA grids are overwhelmingly the terminal stage of a slow deformation process — corrosion wall-thinning, differential soil settlement, joint rotation, fatigue from pressure cycling — measurable weeks to months before water is lost. Existing SCADA alarms are threshold-based snapshots: they detect the leak, not the precursor. The ecosystem requires a **sovereign, deterministic, auditable calculus** that converts continuous deformation telemetry (distributed fiber optic sensing, pressure, flow, acoustic) into a per-junction defect potential and a predicted time-to-failure, with zero black-box ML authority and full KAKI/EAV/NĀRU provenance.

The mythological seal is exact: Enbilulu is the canal inspector of heaven and earth, charged with keeping the Tigris and Euphrates in order — and as a son of Enki, the calculus that lives inside EnkiDW is Enki's own son inspecting the canals.

## 2\. Decision

Adopt the **Enbilulu Calculus** as the single mathematical framework for water net deformation measurement, defect scoring, and pre-leak prediction, implemented (when the build window opens) as **EnbiluluEngine**, a pure-Rust analytical layer over EnkiDW.

---

## 3\. Mathematical definitions

### 3.1 The grid graph

The water net is a graph **G \= (J, E)**:

- **J** — junctions (nodes): valve chambers, tees, crosses, reducers, joints.  
- **E** — pipe segments (edges), each possibly kilometers long, instrumented with distributed fiber optic sensing (DFOS) along arc length *s*.

### 3.2 Segment quantities (1-forms on E)

For each segment *e* with strain field ε(s, t) from DFOS:

| Quantity | Definition | Meaning |
| :---- | :---- | :---- |
| ΔL\_e | ∫ ε(s) ds over the segment | Total elongation (km of fiber → one number) |
| B\_e | ∫ |∂²u/∂s²| ds | Accumulated bending / curvature load |
| κ\_e | RMS deviation of the (strain, pressure) point cloud from its fitted King-plot line | **Barû residual** — nonlinearity onset detector |

The barû residual κ\_e is the King-plot layer: under healthy Hookean (elastic) behavior, hoop strain vs. internal pressure across daily pressure cycles falls on a straight line. Sensor drift and seasonal offsets shift the line but do not bend it; only real physical nonlinearity — plasticity, crack initiation, soil-support loss — bends it. κ\_e is therefore calibration-free, exactly as the original King plot cancels unknown nuclear parameters.

### 3.3 Junction quantities (0-forms on J)

For each junction *j*, the deformation state tensor:

**D\_j \= (δ\_j, θ\_j, ω\_j)**

| Symbol | Quantity | Unit |
| :---- | :---- | :---- |
| δ\_j | Axial displacement of the joint | mm |
| θ\_j | Joint rotation | mrad |
| ω\_j | Ovality of the adjacent pipe cross-section | % |

### 3.4 Discrete divergence

At each junction *j*, sum the signed elongations of all incident segments:

**div\_j \= Σ\_{e ∋ j} sign(e, j) · ΔL\_e**

Healthy grid: div\_j ≈ 0 (the ground moves everything together). Nonzero divergence means *differential* soil movement concentrated at *j* — the classic precursor to joint pull-out. This is a graph-structural signal no single sensor can produce.

### 3.5 The junction defect potential Φ\_Enbi

**Φ\_Enbi(j) \= 100 · clamp( w₁·|δ\_j|/δ\_max \+ w₂·|θ\_j|/θ\_max \+ w₃·ω\_j/ω\_max \+ w₄·max\_{e ∋ j}(κ\_e)/κ\_crit \+ w₅·|div\_j|/div\_max , 0, 1 )**

- Weights w₁…w₅ sum to 1\. Initial calibration proposal: w \= (0.20, 0.20, 0.15, 0.30, 0.15) — the barû residual carries the largest weight because it is the earliest and most calibration-robust precursor. Weights are EAV attributes of the calculus template particle, tunable without code change.  
- Normalization constants (δ\_max, θ\_max, ω\_max, κ\_crit, div\_max) are material- and diameter-class-specific, held as template particles in Hepta Space (each template is itself a particle with a unique real-valued 7D position, per the Hepta Space Uniqueness Law).

### 3.6 TIAMAT alert bands

| Band | Φ\_Enbi range | Meaning |
| :---- | :---- | :---- |
| Stable | Φ \< 40 | Elastic regime, routine monitoring |
| Watch | 40 ≤ Φ \< 60 | Elevated deformation, increase sampling cadence |
| Serious | 60 ≤ Φ \< 80 | Nonlinearity confirmed, inspection dispatch advised |
| **ERRA** | Φ ≥ 80 | Critical — failure trajectory, intervention required |

ERRA (𒀭𒂗𒆳) is the TIAMAT alert level per the sealed naming law (Nergal remains reserved for the BahyWay sovereign AV engine).

### 3.7 The Enbi horizon (leak prediction)

EnkiDW holds the longitudinal series Φ\_Enbi(j, t). The predictive output:

**T\_j \= ( Φ\_crit − Φ\_Enbi(j) ) / ( dΦ\_Enbi/dt )**  with Φ\_crit \= 80 (ERRA threshold)

T\_j — the **Enbi horizon** — is the projected time (weeks) until junction *j* crosses into the ERRA band at its current deformation rate. The grid-wide minimum **T\_grid \= min\_j T\_j** is the grid's countdown clock and the headline KPI.

dΦ/dt is computed by robust linear regression over a sliding window (default 8 weeks) of the EnkiDW series; window length is a template EAV attribute.

### 3.8 Topological layer (Betti numbers)

A complementary network-level detector: build the pressure-correlation graph (junctions as nodes, edges where pressure signals correlate above threshold ρ\_min over window W). Track:

- **β₀** — number of connected components. A cluster splitting \= hydraulic decoupling.  
- **β₁** — number of independent loops. A loop appearing or vanishing \= local hydraulics changing before flow balance does — catches slow leaks that redistribute pressure across a loop and evade single-segment King plots.

Betti anomalies feed the same alert path as Φ\_Enbi, tagged with a distinct EAV attribute (`enbi_topology_anomaly = true`).

---

## 4\. Data architecture

### 4.1 Ingestion and KAKI minting

- Every SCADA reading (strain window, pressure, flow, acoustic amplitude per segment; δ/θ/ω per junction) enters exclusively through **`enkidb-ingest::bridge`** — the only lawful KAKI minting point. No hand-minting, ever.  
- Each reading becomes a KAKI particle with the locked v4.0 16-byte layout; all measured values, units, sensor IDs, and quality flags live exclusively in **EAV Mandatory Attributes** (no quality/state bytes exist in the KAKI key).

### 4.2 EAV Mandatory Attributes (Enbi domain)

| Attribute | Type | Notes |
| :---- | :---- | :---- |
| `enbi_segment_id` / `enbi_junction_id` | ref | Graph element identity |
| `enbi_arc_position_m` | f64 | Position along segment for DFOS windows |
| `enbi_strain_ue` | f64 | Microstrain |
| `enbi_pressure_bar` | f64 | Internal pressure |
| `enbi_delta_mm`, `enbi_theta_mrad`, `enbi_ovality_pct` | f64 | Junction tensor D\_j |
| `enbi_kappa` | f64 | Barû King-plot residual |
| `enbi_divergence` | f64 | Discrete divergence at junction |
| `enbi_phi` | f64 | Computed Φ\_Enbi (derived particle) |
| `enbi_horizon_weeks` | f64 | Computed T\_j (derived particle) |
| `enbi_band` | enum | stable / watch / serious / erra |
| `enbi_topology_anomaly` | bool | Betti-layer flag |
| `ninsun_advisory` | bool | Always true on NINSUN-originated annotations |

### 4.3 Storage and pipeline flow

Standard family flow: **EnkiSDB → EnkiODB → EnkiQDB → EnkiDB → EnkiDW → EnkiMDB → EnkiDDB** (ports 7001–7007). The Enbilulu Calculus computes in **EnkiDW** (the longitudinal warehouse — the King plot is fundamentally a time-series shape test). GeoEngine remains the single source of mathematical truth for all geometry (segment arc-length parameterization, junction coordinates); HeptaShellIndex assigns every junction to a 7-band × 18-sector zone (126 zones per shell) for aggregation. All computed Φ\_Enbi and alert events are WAL-file-first into **NĀRU** before any downstream emission.

### 4.4 Performance envelope

The calculus must operate inside the ecosystem runtime target: **1 billion particles retrieved/processed in under 1 second**. Per-junction Φ\_Enbi is O(deg(j)) over pre-aggregated segment integrals; grid-wide recomputation is embarrassingly parallel across zones and rides the ENLIL four-layer index stack (SurrogateMap / KISPU HeadStore / RoaringStateIndex / BTreeRangeIndex).

---

## 5\. Detection and alert pipeline

Layered exactly as the sealed DataSteward split:

1. **Deterministic layer** — Φ\_Enbi, div\_j, T\_j computed by pure-Rust code in EnbiluluEngine. Schema validation by NARAMSIN. No inference, no heuristics.  
2. **Barû layer** — King-plot residual κ\_e and Betti β₀/β₁ anomaly statistics (King-plot/barû residual per the sealed anomaly-detection role).  
3. **NINSUN layer (advisory only)** — semantic anomaly annotation (`ninsun_advisory = true`); NINSUN never holds cryptographic authority and never changes a band, only annotates.  
4. **TIAMAT escalation** — band transitions Stable → Watch → Serious → ERRA, with τ\_full \+ ŠĀRU dual reporting where opacity applies.  
5. **Nisaba emission** — signed **Alert Event KAKI** particle on any Serious/ERRA transition or topology anomaly.  
6. **Kittu delivery** — ShoWEngine \+ email (Kittu v1 scope; no telephony).  
7. **CSR-08** — no automated intervention. The Architect (or the utility operator in a deployed setting) is the sole decision point; running the response playbook is the confirmation act.

## 6\. Sovereignty constraints

- Pure Rust, `#![forbid(unsafe_code)]`, zero async runtime (stdlib TcpStream \+ std::thread per the sealed v4.0 pattern), zero external API calls at runtime.  
- Fully deterministic and auditable: every gauge value traces to its five Φ terms, each term to its KAKI lineage, each KAKI to its NĀRU WAL entry. A water utility can reproduce any alarm from first principles — the anti-black-box argument is the product argument.  
- ConEngine CSR rules apply unmodified (SargonPassport, tribe isolation, NĀRU audit, KIBRATU event emission).  
- Z3 is design-time only (MUMMU/GeoEngine, Gate G4): usable for proving satisfiability of composite Enbi template shapes in DubSar PDM; never present in the shipped EnbiluluEngine binary.

## 7\. Dashboard contract (Enbilulu panel)

Rendered in **DubSar Theater** (sovereign IDE) as the Enbilulu panel; operational mirror in Grafana \+ Prometheus on `dubsar-workstation` (192.168.122.121), the dedicated monitoring node.

### 7.1 Resolution hierarchy

1. **Zone view (default)** — 126 HeptaShellIndex zones per shell, each zone gauge showing Φ\_zone \= max over its junctions, colored by TIAMAT band.  
2. **Junction view (drill-down)** — per-junction gauges inside a hot zone.  
3. **Segment view** — strain profile ε(s) along the selected segment with the King-plot inset for its κ\_e.

### 7.2 Gauge specification

Each gauge displays: junction/zone ID · Φ\_Enbi (0–100, 240° arc) · TIAMAT band color (Stable green / Watch amber / Serious orange / ERRA red) · **Enbi horizon** in weeks · segment metadata (length, diameter class, material) · provenance affordance opening the KAKI lineage of the five Φ terms.

### 7.3 KPI row

Mean Φ (grid) · Critical junctions (ERRA count) · **Shortest Enbi horizon (T\_grid)** · Active barû residual alerts · Topology anomalies (β-layer).

### 7.4 Projection control

A time slider projecting every gauge forward via Φ(j, t+Δ) \= min(100, Φ \+ Δ·dΦ/dt), letting the operator see which junctions the grid's own deformation rates promote next.

### 7.5 Sample sovereign query (HeptaScript — Anti-SQL)

ORBIT waternet.junctions

  PRESENT enbi\_phi, enbi\_horizon\_weeks, enbi\_band

  PHYSICS enbi\_phi \>= 60

  META    material \= "cast\_iron"

  EMIT    dashboard.enbilulu.serious\_watchlist

  PROVE   lineage(enbi\_phi)

  WITNESS naru

## 8\. Failure modes — covered and not covered

| Failure mode | Covered by Enbi? | Mechanism |
| :---- | :---- | :---- |
| Corrosion wall-thinning | Yes | κ\_e nonlinearity \+ ε trend, weeks–months horizon |
| Differential soil settlement | Yes | div\_j \+ θ\_j, weeks–months horizon |
| Joint degradation / pull-out | Yes | δ\_j, θ\_j, div\_j |
| Fatigue at welds | Yes | κ\_e under pressure cycling |
| Slow leak redistributing pressure | Yes | Betti β₁ topology layer |
| **Third-party strike (excavator)** | **No — by design** | Instantaneous; handled by DAS acoustic detection as a distinct Alert Event KAKI type (seconds–minutes response, not prediction) |

## 9\. Alternatives considered

- **ML anomaly detection (autoencoder / LSTM on SCADA streams)** — rejected: opaque, unauditable, violates sovereignty telos, requires training data the utility cannot verify, and adds runtime model authority contrary to the NINSUN advisory-only law.  
- **Threshold-only SCADA alarms** — rejected: detect leaks, not precursors; no horizon; drift-sensitive.  
- **Uber H3 hexagonal aggregation** — rejected: superseded ecosystem-wide by HeptaShellIndex (E7 lattice, 126 zones/shell) per the sealed HeptaMap law.

## 10\. Consequences

- EnkiDW gains its first formal analytical calculus, establishing the pattern for future domain calculi (the ṬUPŠARRU principle: all derived values mathematically traceable to particle data).  
- PollutionWay-era environmental intelligence (v3.5) gains a v4.0 successor domain under the ENKI-TERRA umbrella: NANSHE (river contamination), ABZU (groundwater), and now Enbilulu (engineered waterways) form a coherent water triad, all children of Enki's domain.  
- Adds one future engine (EnbiluluEngine) to the roadmap and one dashboard panel to DubSar Theater — no changes to any Phase A/B playbook.

## 11\. Roadmap position

QUEUED. Convertible to numbered playbooks only after: Phase A (PB-90–98, PB-98 gate, BLK-1 applied) → Phase B (PB-99–109) → TESTING\_PLAYBOOK\_PHASE1 Blocks A–F (96 unit tests \+ 50-file corpus \+ bench) → BeeMDM 50-zip ETL test. Anticipated playbook set on unlock: EnbiluluEngine crate scaffold · Enbi template particles \+ normalization constants · barû King-plot residual module · Betti topology module · EnkiDW series \+ horizon regression · Nisaba/Kittu alert wiring · DubSar Theater Enbilulu panel · Grafana operational mirror.

---

*Sealed by DUB.SAR 𒁾 · BahyWay.Ecosystem v4.0 · 2026-07-07* *"Enbilulu, the inspector of the canals of heaven and earth, keeps the waters in order."*  
