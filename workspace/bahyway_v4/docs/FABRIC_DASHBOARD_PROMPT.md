# BahyWay Fabric Dashboard — Implementation Prompt

> Hand this prompt verbatim to a new Claude Code session.
> It is self-contained: no prior conversation context is needed.

---

## TASK

You are working inside the BahyWay.Ecosystem v4.0 monorepo at
`/home/user/EnkiDB/workspace/bahyway_v4/`.

Implement a new tab panel called **"Fabric"** inside the existing
`crates/bahyway-web` Rust-WASM application.

The Fabric panel is the production-level, sovereign dashboard that:
1. **Controls** the Enterprise Data Fabric — register sources, run pipelines,
   enable/disable flows
2. **Shows transparency** — live pipeline animation, per-record lineage chains,
   quality distribution, exception console
3. **Persists** state — all simulation data lives in a Rust `thread_local!`
   struct, same pattern as the existing `ETL`, `TRIBES_STATE`, `HUBBLE_STATE`
   thread-locals

---

## EXISTING CODEBASE — READ THESE FIRST

Before writing a single line, read:

1. `crates/bahyway-web/index.html` — full HTML, CSS variables, panel structure, nav tabs
2. `crates/bahyway-web/src/lib.rs` — main Rust-WASM entry, DOM rendering, all existing panels
3. `crates/bahyway-web/src/etl_sim.rs` — EtlSimState pattern (state machine, tick loop, canvas draw)
4. `crates/bahyway-web/src/tribes.rs` — data definitions pattern (static arrays, structs)
5. `crates/bahyway-web/src/story_engine.rs` — node graph canvas pattern
6. `crates/bahyway-fabric/src/lib.rs` — the Fabric API surface you are visualising
7. `crates/bahyway-fabric/src/pipeline.rs` — Stage enum, PipelineDeclaration, ExceptionPolicy
8. `crates/bahyway-fabric/src/connector.rs` — SourceId, TargetId, DataBatch
9. `crates/bahyway-fabric/src/exception.rs` — FabricException, ExceptionKind (7 variants)
10. `crates/bahyway-fabric/src/lineage.rs` — LineageChain, LineageHop, QualitySnapshot
11. `crates/bahyway-fabric/src/adapters.rs` — all 8 sources, all 7 targets

---

## TECHNOLOGY CONSTRAINTS — NON-NEGOTIABLE

```
✓ Pure Rust + wasm-bindgen + web-sys                (no TypeScript, no Node)
✓ Pure DOM for tables, cards, controls              (no React, Vue, Svelte)
✓ Canvas 2D (CanvasRenderingContext2d) for flows    (no SVG, no D3, no Chart.js)
✓ No CDN imports, no npm packages                  (sovereign — everything compiled)
✓ Same Dracula color scheme as all other panels    (CSS variables defined in index.html)
✓ Monospace font stack everywhere                  (JetBrains Mono → Cascadia → Fira Code)
✓ Same setInterval animation pattern (33ms = 30fps for canvas, 250ms for slow ticks)
✓ Same thread_local! state pattern as ETL, TRIBES_STATE, HUBBLE_STATE
✓ Rust 2021 edition, #![allow(unused)] not permitted — clean code only
```

---

## COLOR SCHEME REFERENCE

All these are already defined as CSS variables in `index.html`. Use them by name:

```
--bg:          #0b0b10    Main background
--surface:     #0e0e18    Card / panel backgrounds
--surface2:    #131322    Button / hover backgrounds
--border:      #2a2a4a    Panel borders
--text:        #cdd6f4    Primary text
--text-muted:  #6c7086    Secondary / dim text
--accent:      #bd93f9    Primary accent (purple — use for selected/active)
--tok-lit:     #50fa7b    Green  — GEM quality, success, active source
--tok-str:     #f1fa8c    Yellow — TRIBE quality, warning
--tok-op:      #ffb86c    Orange — ACTIVE/FUZZY quality, slow
--tok-unk:     #ff5555    Red    — DEAD quality, exception, error
--tok-fld:     #8be9fd    Cyan   — lineage hashes, info, field names
--tok-kw:      #bd93f9    Purple — stage names, keywords
--text-muted:  #6c7086    Gray   — disabled, muted, dead targets
```

**Quality lane → colour mapping (use throughout):**
```
B11 ≥ 200  GEM    →  #50fa7b  (green)
B11 ≥ 140  TRIBE  →  #f1fa8c  (yellow)
B11 ≥ 100  ACTIVE →  #ffb86c  (orange)
B11 ≥  60  FUZZY  →  #bd93f9  (purple)
B11  <  60  DEAD   →  #ff5555  (red)
```

---

## WHAT TO BUILD — FIVE SUB-PANELS

The Fabric tab contains five sub-panels, selectable via a secondary nav bar
inside the Fabric panel (pill buttons, not full page tabs):

```
[ Pipeline Map ]  [ Lineage Explorer ]  [ Exception Console ]
[ Registry ]  [ Quality Report ]
```

---

### Sub-Panel 1: Pipeline Map (Canvas — primary view)

**Purpose:** Animated flow graph showing live data moving from sources through
stages to targets.

**Canvas size:** 960 × 480px, 30fps animation loop.

**Layout (left → right):**

```
┌──────────────┐        ┌──────────────────────────────────┐        ┌──────────────┐
│   SOURCES    │        │          FABRIC STAGES           │        │   TARGETS    │
│              │        │                                  │        │              │
│  ● ERP       │──────▶ │  [Cleanse]─[Validate]─[Enrich]  │──────▶ │  ● DW        │
│  ● CRM       │──────▶ │                   └─[Dedup]─────│──────▶ │  ● Dashboard │
│  ● HR        │        │                                  │        │  ● Notify    │
│  ● Excel     │        │  B11 flow bar ──────────────────│        │  ● Portal    │
│  ● API       │        │  █████████░░  GEM: 42%          │        │              │
└──────────────┘        └──────────────────────────────────┘        └──────────────┘
```

**Rendering details:**

- **Source nodes** (left column): rounded rectangles (8px radius), 120×32px each,
  colored by connection state:
  - Active = `#50fa7b` left border (4px)
  - Idle   = `#6c7086` left border
  - Error  = `#ff5555` left border
  - Label: `source.display_name()`, 11px monospace, `--text`
  - Small dot (6px circle) on right edge = connector port

- **Stage nodes** (center area): rounded rectangles, 80×28px, positioned in a
  horizontal chain with 12px gaps.
  - Fill: `--surface2`
  - Border: 1px `--border`
  - Label: stage name from `Stage::stage_name()`, 10px, `--tok-kw`
  - When active (particles flowing): border glows with 2px `--accent`
  - Stages for a selected pipeline only; dimmed stages shown for inactive pipelines

- **Target nodes** (right column): same as source nodes but with left port dot.

- **Flow lines:** Bezier curves from source port → stage chain → target port.
  - Idle: 1px `--border`
  - Active: 2px `--tok-lit` with animated dashes (offset increments 2px per frame)
  - Error: 1px `--tok-unk`

- **Particle dots on flow lines:** Small circles (5px radius) animating along
  the bezier path at a speed proportional to throughput. Color = B11 quality lane color.
  Each dot carries a mini B11 value label (7px) shown on hover (canvas mouse position check).

- **B11 flow bar** (below stage chain, full width of center area):
  Stacked horizontal bar, 12px height:
  - Segments: GEM (green) | TRIBE (yellow) | ACTIVE (orange) | FUZZY (purple) | DEAD (red)
  - Percentages computed from `FabricSimState.quality_distribution`
  - Label: "Quality Distribution — last run" + counts for each lane

- **Run controls** (below canvas, DOM buttons):
  - `[▶ Run All]` — triggers `FabricSimState::tick_all()`
  - `[⏸ Pause]` — stops auto-tick
  - `[↺ Reset]` — resets all simulation counters
  - Pipeline selector dropdown: select which pipeline to highlight on canvas

---

### Sub-Panel 2: Lineage Explorer (Canvas + DOM)

**Purpose:** Show the complete hop-by-hop audit trail for a selected record.
Answers "where does this data even come from?" visually.

**Canvas size:** 960 × 260px.

**Layout:** Horizontal timeline of hop nodes from left (origin) to right (final target).

**Hop node rendering (80 × 80px rounded rect, 12px radius):**

```
┌─────────────────┐
│  [stage name]   │  ← 10px, --tok-kw (purple)
│                 │
│  B11: 55 → 168  │  ← colour = destination lane colour
│                 │
│  epoch: 42      │  ← 9px, --text-muted
│                 │
│  a1b2 → d4e5    │  ← FNV hash prefix, 8px, --tok-fld (cyan)
└─────────────────┘
```

- Border colour = B11 lane of `b11_out`:
  - GEM: `#50fa7b`, TRIBE: `#f1fa8c`, ACTIVE: `#ffb86c`, FUZZY: `#bd93f9`, DEAD: `#ff5555`
- Arrow connector between nodes: 2px `--border`, arrowhead (filled triangle, 6px)
- Quality improvement indicator: small upward triangle (▲ green) or downward (▼ red)
  in top-right of each node if `b11_out > b11_in` or not
- Origin node (first hop): dashed left border = source entry
- Terminal node (last hop): double right border = delivered

**Below canvas — DOM detail pane:**
When a hop node is clicked (canvas hit test by x-position):
- **Full hash display:** `input_hash: 0xa1b2c3d4...` / `output_hash: 0xd4e5f6a7...`
- **Stage annotation** text
- **Quality delta** bar: from `b11_in` to `b11_out` on a 0–240 scale

**Record selector (above canvas, DOM):**
- Dropdown to select which record's lineage to display
- Shows: `record #N — origin: ERP — depth: 5 hops — B11: 210 (GEM)`
- `[← prev]` `[→ next]` buttons to step through records in last run

---

### Sub-Panel 3: Exception Console (DOM — scrollable feed)

**Purpose:** Real-time exception feed. Zero exceptions = zero entries (clean is clean).

**Layout:** Fixed-height (400px) scrollable `<div>` with exception cards, newest on top.

**Each exception card:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│  [QualityRejection]  stage: hepta-score  epoch: 42                     │
│  source: erp.sovereign                                                   │
│  B11=55 below quality threshold 140                                     │
│  ▼ payload  (3 fields)                                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

- **Kind badge** (`[QualityRejection]`): coloured pill
  - SchemaViolation      → `--tok-op`   (orange)
  - MissingRequiredField → `--tok-op`   (orange)
  - QualityRejection     → `--tok-str`  (yellow)
  - DuplicateIdentity    → `--tok-kw`   (purple)
  - DeliveryFailure      → `--tok-unk`  (red)
  - TransformError       → `--tok-unk`  (red)
  - ExtractionError      → `--tok-unk`  (red)
  - InternalFault        → `--tok-unk`  (red, pulsing border animation)

- **Left border** (4px): same colour as badge
- **Expandable payload:** clicking `▼ payload (N fields)` toggles a `<pre>` block
  showing each `(attr_hash: 0xNNNN, value: "...")` pair

- **Top of console controls:**
  - `[🗑 Clear]` — removes all displayed exceptions
  - Filter pills: `[All]  [Quality]  [Schema]  [Delivery]  [Extraction]`
  - Exception count badge: `12 exceptions — 0 critical`

- **Empty state (no exceptions):**
  - Centered text: `✓ No exceptions — last run was clean`  (green, `--tok-lit`)

---

### Sub-Panel 4: Registry (DOM — two-column tables)

**Purpose:** Show all registered sources and targets with their schema fields.
Read-only view in simulation; click a row to highlight it on the Pipeline Map canvas.

**Left column — Sources table:**

| SourceId | Display Name | Required Fields | Optional Fields | Status |
|---|---|---|---|---|
| `erp.sovereign` | ERP System | 3 | 2 | ● Active |
| `crm.sovereign` | CRM System | 3 | 2 | ● Active |
| `hr.sovereign` | HR System | 3 | 2 | ● Idle |
| `legacy.sovereign` | Legacy System | 2 | 1 | ⚠ Error |
| ... | | | | |

- Status indicator colours: Active = `--tok-lit`, Idle = `--text-muted`, Error = `--tok-unk`
- Clicking a row: highlights that source's connections on the Pipeline Map canvas
  (sets `FabricSimState.selected_source`)
- Row hover: `--surface2` background

**Right column — Targets table:**

| TargetId | Display Name | Pipelines | Last Delivery |
|---|---|---|---|
| `dw.central` | Data Warehouse | 3 | epoch 42 |
| `dashboard.sovereign` | Dashboards | 2 | epoch 41 |
| `notify.sovereign` | Notifications | 1 | epoch 40 |
| ... | | | |

**Below both tables — Pipelines table:**

| Pipeline ID | Version | Source | Stages | Targets | Status | Last Run |
|---|---|---|---|---|---|---|
| `erp.invoices → dw` | v1 | erp.sovereign | Cleanse→Validate→Dedup→Enrich | dw.central, notify | ● Enabled | epoch 42 |
| `crm.contacts → dash` | v2 | crm.sovereign | Cleanse→Validate→Dedup | dashboard.sovereign | ● Enabled | epoch 40 |

- `[▶ Run]` button per row → triggers that pipeline's tick
- `[⏸ Disable]` / `[▶ Enable]` toggle
- `[👁 Inspect]` → switches to Lineage Explorer and loads last-run lineage for that pipeline

---

### Sub-Panel 5: Quality Report (Canvas — bar chart + DOM summary)

**Purpose:** Show quality distribution across all processed records in the session.

**Canvas size:** 960 × 200px, static (redraws on each new run result).

**Bar chart layout:**
- X axis: 5 quality lanes (GEM, TRIBE, ACTIVE, FUZZY, DEAD)
- Y axis: record count (0 to max lane count, with 4 gridlines)
- Each bar: filled rounded rectangle, coloured by lane
- Labels: lane name + count + percentage below each bar
- Chart title: "Quality Distribution — Session Total" + total records processed

**Below canvas — DOM summary row (4 stat cards):**

```
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│ SUCCESS RATE    │ │ GEM RATE        │ │ EXCEPTIONS      │ │ TOTAL RECORDS   │
│                 │ │                 │ │                 │ │                 │
│    97.3 %       │ │    42.1 %       │ │      12         │ │     1,847       │
│                 │ │ target: 35.4%↑  │ │  3 critical     │ │  in this session│
└─────────────────┘ └─────────────────┘ └─────────────────┘ └─────────────────┘
```

- Success rate colour: green ≥ 95%, yellow ≥ 85%, red < 85%
- GEM rate colour: green ≥ 35.4% (ADR-004 SLA), yellow ≥ 25%, red < 25%
- GEM rate target line drawn on the bar chart as a dashed horizontal line at 35.4%

---

## SIMULATION STATE — Rust struct to implement

Add this to a new file `crates/bahyway-web/src/fabric_sim.rs`:

```rust
//! FabricSimState — in-browser simulation of bahyway-fabric orchestration.
//! Mirrors the real FabricOrchestrator API surface without actual I/O.

use std::cell::Cell;

/// One source connector in the simulation registry.
pub struct SimSource {
    pub id:            &'static str,
    pub display_name:  &'static str,
    pub required_fields: u8,
    pub optional_fields: u8,
    pub status:        SimSourceStatus,
}

pub enum SimSourceStatus { Active, Idle, Error }

/// One target connector in the simulation registry.
pub struct SimTarget {
    pub id:           &'static str,
    pub display_name: &'static str,
    pub pipeline_count: u8,
    pub last_epoch:   u32,
}

/// One pipeline in the simulation registry.
pub struct SimPipeline {
    pub id:      &'static str,
    pub version: u16,
    pub source:  &'static str,          // SourceId string
    pub stages:  Vec<&'static str>,     // stage names in order
    pub targets: Vec<&'static str>,     // TargetId strings
    pub enabled: bool,
    pub last_epoch: u32,
}

/// One hop in a lineage chain (mirrors LineageHop).
pub struct SimLineageHop {
    pub stage:       &'static str,
    pub b11_in:      u8,
    pub b11_out:     u8,
    pub input_hash:  u64,
    pub output_hash: u64,
    pub epoch:       u32,
    pub annotation:  &'static str,
}

/// One simulated exception (mirrors FabricException).
pub struct SimException {
    pub kind:      &'static str,    // "QualityRejection" etc.
    pub stage:     &'static str,
    pub source_id: &'static str,
    pub message:   String,
    pub epoch:     u32,
    pub payload_field_count: usize,
}

/// Distribution of B11 quality scores across all processed records.
pub struct QualityDistribution {
    pub gem:    usize,   // B11 ≥ 200
    pub tribe:  usize,   // B11 ≥ 140
    pub active: usize,   // B11 ≥ 100
    pub fuzzy:  usize,   // B11 ≥  60
    pub dead:   usize,   // B11  < 60
}

impl QualityDistribution {
    pub fn total(&self) -> usize {
        self.gem + self.tribe + self.active + self.fuzzy + self.dead
    }
    pub fn success_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 { return 1.0; }
        (self.gem + self.tribe + self.active + self.fuzzy) as f64 / t as f64
    }
    pub fn gem_rate(&self) -> f64 {
        let t = self.total();
        if t == 0 { return 0.0; }
        self.gem as f64 / t as f64
    }
}

/// Animation state for particle dots flowing along bezier curves.
pub struct FlowParticle {
    pub pipeline_idx: usize,
    pub t:            f64,   // 0.0 = source, 1.0 = target
    pub b11:          u8,
    pub speed:        f64,   // t-units per frame
}

/// Master state for the Fabric dashboard panel.
pub struct FabricSimState {
    pub sources:       Vec<SimSource>,
    pub targets:       Vec<SimTarget>,
    pub pipelines:     Vec<SimPipeline>,
    pub lineage:       Vec<Vec<SimLineageHop>>,   // one Vec per record
    pub exceptions:    Vec<SimException>,
    pub quality:       QualityDistribution,
    pub flow_particles: Vec<FlowParticle>,
    pub running:       bool,
    pub epoch:         u32,
    pub selected_pipeline: usize,
    pub selected_record:   usize,
    pub selected_source:   Option<usize>,
    pub active_sub_panel:  FabricSubPanel,
}

pub enum FabricSubPanel {
    PipelineMap,
    LineageExplorer,
    ExceptionConsole,
    Registry,
    QualityReport,
}
```

**Static initial data** — populate `FabricSimState::default()` with all 8
sources from `adapters.rs`, all 7 targets, and the following 3 pipelines:

1. `"erp.invoices → dw"` v1 — source: `erp.sovereign` — stages: Cleanse, Validate(140), Deduplicate, Enrich(erp.v1) — targets: `dw.central`, `notify.sovereign`
2. `"crm.contacts → dashboard"` v2 — source: `crm.sovereign` — stages: Cleanse, Validate(100), Deduplicate — targets: `dashboard.sovereign`
3. `"api.events → portals"` v1 — source: `api.sovereign` — stages: Cleanse, Validate(60), Transform(api.normalize) — targets: `portal.sovereign`, `reporting.tools`

**`do_tick()` method** — called every 800ms when `running = true`:
- Increments `epoch`
- For each enabled pipeline, generates between 5–20 records
- Assigns B11 scores following a realistic distribution:
  - 35% GEM (200–240), 30% TRIBE (140–199), 20% ACTIVE (100–139),
    10% FUZZY (60–99), 5% DEAD (0–59)
- Appends B11 scores to `quality` distribution
- Generates 0–2 `SimException`s per pipeline per tick (probability 15%)
  with realistic kinds and messages
- Generates 3–5 `SimLineageHop` sequences (one per "record") with realistic
  B11 progression (cleanse: same, validate: scoring, enrich: +10–20)
- Spawns new `FlowParticle` entries for each processed record
- Updates `last_epoch` on active pipelines and targets

---

## HTML CHANGES — add to `index.html`

### 1. Navigation tab button (add alongside existing tabs):
```html
<button class="tab-btn" onclick="switch_tab('fabric')" id="tab-fabric">
  𒀭 Fabric
</button>
```

### 2. Panel div (add before the closing `</main>` tag):
```html
<div id="panel-fabric" style="display:none">
  <div class="panel-header">
    <span class="panel-title">𒀭𒂗𒆠 Enterprise Data Fabric</span>
    <span class="panel-subtitle">Sovereign pipeline control · lineage · exceptions · quality</span>
  </div>

  <!-- Secondary sub-panel nav -->
  <div id="fabric-subnav">
    <button class="fabric-pill active" onclick="fabric_switch('pipeline_map')"  id="fpill-pipeline_map">Pipeline Map</button>
    <button class="fabric-pill"        onclick="fabric_switch('lineage')"        id="fpill-lineage">Lineage Explorer</button>
    <button class="fabric-pill"        onclick="fabric_switch('exceptions')"     id="fpill-exceptions">Exception Console</button>
    <button class="fabric-pill"        onclick="fabric_switch('registry')"       id="fpill-registry">Registry</button>
    <button class="fabric-pill"        onclick="fabric_switch('quality')"        id="fpill-quality">Quality Report</button>
  </div>

  <!-- Pipeline Map -->
  <div id="fabric-pipeline_map">
    <div id="fabric-run-controls">
      <button id="fabric-run-all">▶ Run All</button>
      <button id="fabric-pause">⏸ Pause</button>
      <button id="fabric-reset">↺ Reset</button>
      <select id="fabric-pipeline-select"></select>
    </div>
    <canvas id="fabric-map-canvas" width="960" height="480"></canvas>
  </div>

  <!-- Lineage Explorer -->
  <div id="fabric-lineage" style="display:none">
    <div id="fabric-lineage-controls">
      <button id="fabric-prev-record">← prev</button>
      <span id="fabric-record-label"></span>
      <button id="fabric-next-record">→ next</button>
    </div>
    <canvas id="fabric-lineage-canvas" width="960" height="260"></canvas>
    <div id="fabric-hop-detail"></div>
  </div>

  <!-- Exception Console -->
  <div id="fabric-exceptions" style="display:none">
    <div id="fabric-exc-controls">
      <button id="fabric-exc-clear">🗑 Clear</button>
      <span id="fabric-exc-count"></span>
      <div id="fabric-exc-filters">
        <button class="exc-filter active" data-kind="all">All</button>
        <button class="exc-filter" data-kind="quality">Quality</button>
        <button class="exc-filter" data-kind="schema">Schema</button>
        <button class="exc-filter" data-kind="delivery">Delivery</button>
        <button class="exc-filter" data-kind="extraction">Extraction</button>
      </div>
    </div>
    <div id="fabric-exc-feed"></div>
  </div>

  <!-- Registry -->
  <div id="fabric-registry" style="display:none">
    <div id="fabric-registry-cols">
      <div id="fabric-sources-table"></div>
      <div id="fabric-targets-table"></div>
    </div>
    <div id="fabric-pipelines-table"></div>
  </div>

  <!-- Quality Report -->
  <div id="fabric-quality" style="display:none">
    <canvas id="fabric-quality-canvas" width="960" height="200"></canvas>
    <div id="fabric-quality-stats"></div>
  </div>
</div>
```

### 3. CSS to add to the `<style>` block in `index.html`:
```css
/* Fabric sub-panel nav */
#fabric-subnav {
  display: flex;
  gap: 8px;
  padding: 12px 0 16px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 16px;
}
.fabric-pill {
  background: var(--surface2);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 14px;
  padding: 4px 14px;
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.fabric-pill:hover  { border-color: var(--accent); color: var(--text); }
.fabric-pill.active { background: var(--accent); color: var(--accent-fg);
                      border-color: var(--accent); font-weight: 600; }

/* Exception cards */
.exc-card {
  border-left: 4px solid var(--border);
  background: var(--surface);
  border-radius: 0 6px 6px 0;
  padding: 10px 14px;
  margin-bottom: 8px;
  font-size: 12px;
}
.exc-card .exc-kind-badge {
  display: inline-block;
  border-radius: 10px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 600;
  margin-right: 8px;
}
.exc-card .exc-stage  { color: var(--text-muted); }
.exc-card .exc-source { color: var(--tok-fld); }
.exc-card .exc-msg    { color: var(--text); margin-top: 4px; }
.exc-card .exc-payload-toggle { color: var(--text-muted); cursor: pointer;
                                 font-size: 11px; margin-top: 6px; }
.exc-payload-body { background: var(--surface2); border-radius: 4px;
                    padding: 6px 10px; margin-top: 6px; font-size: 11px;
                    color: var(--tok-fld); display: none; }
.exc-payload-body.open { display: block; }

/* Stat cards (quality report) */
.fabric-stat-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-top: 16px;
}
.fabric-stat-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 14px 16px;
  text-align: center;
}
.fabric-stat-card .stat-label { font-size: 10px; color: var(--text-muted);
                                  text-transform: uppercase; letter-spacing: 0.8px; }
.fabric-stat-card .stat-value { font-size: 28px; font-weight: 700;
                                  margin: 6px 0 2px; }
.fabric-stat-card .stat-sub   { font-size: 11px; color: var(--text-muted); }

/* Registry tables */
.fabric-reg-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  margin-bottom: 20px;
}
.fabric-reg-table th {
  text-align: left;
  padding: 6px 10px;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.6px;
}
.fabric-reg-table td {
  padding: 7px 10px;
  border-bottom: 1px solid var(--border);
  color: var(--text);
}
.fabric-reg-table tr:hover td { background: var(--surface2); cursor: pointer; }
.fabric-reg-table tr.selected td { background: var(--surface2);
                                    outline: 1px solid var(--accent) inset; }

/* Fabric run controls */
#fabric-run-controls {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}
#fabric-run-controls button {
  background: var(--surface2);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 5px 14px;
  font-family: inherit;
  font-size: 12px;
  cursor: pointer;
}
#fabric-run-controls button:hover { border-color: var(--accent); }
#fabric-run-controls select {
  background: var(--surface2);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 4px 10px;
  font-family: inherit;
  font-size: 12px;
  margin-left: auto;
}
```

---

## RUST IMPLEMENTATION — `src/lib.rs` additions

### 1. Add thread-local for FabricSimState

```rust
thread_local! {
    static FABRIC: RefCell<FabricSimState> = RefCell::new(FabricSimState::default());
}
```

### 2. Register the Fabric tab in `start()`:

```rust
// In the existing start() function, alongside other tab registrations:
add_click("tab-fabric", |_| { switch_tab("fabric"); });
add_click("fabric-run-all", |_| {
    FABRIC.with(|f| { f.borrow_mut().running = true; });
});
add_click("fabric-pause", |_| {
    FABRIC.with(|f| { f.borrow_mut().running = false; });
});
add_click("fabric-reset", |_| {
    FABRIC.with(|f| { f.borrow_mut().reset(); });
    redraw_fabric();
});
add_click("fabric-exc-clear", |_| {
    FABRIC.with(|f| { f.borrow_mut().exceptions.clear(); });
    render_exception_feed();
});
add_click("fabric-prev-record", |_| {
    FABRIC.with(|f| {
        let mut s = f.borrow_mut();
        if s.selected_record > 0 { s.selected_record -= 1; }
    });
    draw_lineage_canvas();
});
add_click("fabric-next-record", |_| {
    FABRIC.with(|f| {
        let mut s = f.borrow_mut();
        let max = s.lineage.len().saturating_sub(1);
        if s.selected_record < max { s.selected_record += 1; }
    });
    draw_lineage_canvas();
});
```

### 3. Add the `fabric_switch` JS-callable function:

```rust
#[wasm_bindgen]
pub fn fabric_switch(sub: &str) {
    // Hide all fabric sub-panels, show the selected one
    // Update .active class on pill buttons
    // Redraw relevant canvas if needed
}
```

### 4. Animation interval (add to the `start()` interval setup):

```rust
// 33ms interval for pipeline map canvas animation
let cb = Closure::wrap(Box::new(|| {
    FABRIC.with(|f| {
        let mut s = f.borrow_mut();
        // advance flow particle t values
        s.advance_flow_particles(0.004);
    });
    draw_fabric_map_canvas();
}) as Box<dyn FnMut()>);
window.set_interval_with_callback_and_timeout_and_arguments_0(
    cb.as_ref().unchecked_ref(), 33
).unwrap();
cb.forget();

// 800ms interval for simulation ticks
let cb2 = Closure::wrap(Box::new(|| {
    FABRIC.with(|f| {
        let mut s = f.borrow_mut();
        if s.running { s.do_tick(); }
    });
    redraw_fabric();
}) as Box<dyn FnMut()>);
window.set_interval_with_callback_and_timeout_and_arguments_0(
    cb2.as_ref().unchecked_ref(), 800
).unwrap();
cb2.forget();
```

---

## CANVAS DRAWING FUNCTIONS

Implement these in `src/lib.rs` (or in a new `src/fabric_canvas.rs` module):

```rust
/// Draw the pipeline flow map on #fabric-map-canvas.
fn draw_fabric_map_canvas() {
    // 1. Clear canvas with --canvas-bg (#07070f)
    // 2. Draw source nodes (left column, 8 sources)
    // 3. Draw stage chain nodes for the selected pipeline
    // 4. Draw target nodes (right column, relevant targets for selected pipeline)
    // 5. Draw bezier flow lines (source → stages → targets)
    // 6. Draw animated flow particles (dots along bezier paths)
    // 7. Draw B11 quality distribution bar (below stage chain)
    // 8. Draw column headers: "SOURCES", "STAGES", "TARGETS" in --text-muted
}

/// Draw the lineage chain timeline on #fabric-lineage-canvas.
fn draw_lineage_canvas() {
    // 1. Clear canvas
    // 2. Load selected record's lineage from FABRIC state
    // 3. Compute hop node positions (evenly spaced, centred vertically)
    // 4. Draw connecting arrows between nodes
    // 5. Draw hop nodes with B11 lane border colour
    // 6. Draw stage name, B11 before→after, epoch, hash prefix labels
    // 7. Draw quality delta indicators (▲ ▼)
    // 8. Update #fabric-record-label DOM element
}

/// Draw the quality distribution bar chart on #fabric-quality-canvas.
fn draw_quality_canvas() {
    // 1. Clear canvas
    // 2. Draw 4 horizontal gridlines with labels (25%, 50%, 75%, 100% of max)
    // 3. Draw 5 bars (GEM, TRIBE, ACTIVE, FUZZY, DEAD) with lane colours
    // 4. Draw dashed ADR-004 GEM rate target line at 35.4%
    // 5. Draw bar labels below (lane name, count, percentage)
    // 6. Draw chart title above
}

/// Re-render the full exception feed DOM.
fn render_exception_feed() {
    // Clear #fabric-exc-feed
    // For each exception in FABRIC state (newest first):
    //   Create .exc-card div with correct border colour and badge
    //   Append to feed
    // Update #fabric-exc-count
    // If empty: show clean state message
}

/// Re-render registry tables.
fn render_registry_tables() {
    // Render .fabric-reg-table for sources into #fabric-sources-table
    // Render .fabric-reg-table for targets into #fabric-targets-table
    // Render .fabric-reg-table for pipelines into #fabric-pipelines-table
    // Populate #fabric-pipeline-select dropdown
}

/// Render the 4 quality stat cards.
fn render_quality_stats() {
    // Create .fabric-stat-cards div with 4 .fabric-stat-card elements
    // Inject into #fabric-quality-stats
    // Colour stat values based on thresholds described above
}

/// Master redraw — calls all relevant draw functions for the active sub-panel.
fn redraw_fabric() {
    FABRIC.with(|f| {
        let s = f.borrow();
        match s.active_sub_panel {
            FabricSubPanel::PipelineMap      => { drop(s); draw_fabric_map_canvas(); }
            FabricSubPanel::LineageExplorer  => { drop(s); draw_lineage_canvas(); }
            FabricSubPanel::ExceptionConsole => { drop(s); render_exception_feed(); }
            FabricSubPanel::Registry         => { drop(s); render_registry_tables(); }
            FabricSubPanel::QualityReport    => { drop(s); draw_quality_canvas(); render_quality_stats(); }
        }
    });
}
```

---

## BEZIER CURVE HELPER

The flow lines from sources through stages to targets follow cubic bezier paths.
Use this helper for both drawing and animating particle positions:

```rust
/// Evaluate a cubic bezier at parameter t ∈ [0.0, 1.0].
/// p0 = start, p1 = control1, p2 = control2, p3 = end.
fn bezier_point(p0: (f64,f64), p1: (f64,f64), p2: (f64,f64), p3: (f64,f64), t: f64) -> (f64,f64) {
    let u = 1.0 - t;
    let x = u*u*u*p0.0 + 3.0*u*u*t*p1.0 + 3.0*u*t*t*p2.0 + t*t*t*p3.0;
    let y = u*u*u*p0.1 + 3.0*u*u*t*p1.1 + 3.0*u*t*t*p2.1 + t*t*t*p3.1;
    (x, y)
}
```

Control points: `p1 = (p0.x + (p3.x-p0.x)*0.4, p0.y)` and
`p2 = (p3.x - (p3.x-p0.x)*0.4, p3.y)` — this gives smooth S-curves.

---

## SOVEREIGN CONSTANTS TO ENCODE

These must be hardcoded as Rust constants in `fabric_sim.rs`,
matching `hepta-score` and `bahyway-fabric` ADR values exactly:

```rust
pub const QUALITY_DIVISOR: f64 = 240.0;     // ADR-001 — never 255
pub const GEM_B11:   u8 = 200;              // Lane threshold
pub const TRIBE_B11: u8 = 140;
pub const ACTIVE_B11: u8 = 100;
pub const FUZZY_B11:  u8 = 60;
pub const GEM_RATE_TARGET: f64 = 0.354;     // ADR-004 — 35.4% SLA
```

---

## ACCEPTANCE CRITERIA

The implementation is complete when:

- [ ] `cargo build -p bahyway-web` compiles with 0 errors, 0 warnings
- [ ] `trunk serve` launches and the Fabric tab appears in the nav bar
- [ ] Clicking `▶ Run All` starts the simulation — flow particles appear and animate on the canvas
- [ ] The quality bar chart updates on every tick with real B11 distribution numbers
- [ ] Exceptions appear in the Exception Console with correct kind badges and colours
- [ ] The Lineage Explorer shows a horizontal hop chain; clicking prev/next steps through records
- [ ] The Registry tables list all 8 sources, 7 targets, and 3 pipelines
- [ ] Clicking a source row on the Registry highlights its connections on the Pipeline Map
- [ ] `⏸ Pause` stops ticks; `↺ Reset` returns all counters to zero
- [ ] All 5 sub-panels switch correctly via the pill nav
- [ ] The GEM rate stat card goes green when GEM rate ≥ 35.4% (ADR-004 SLA met)
- [ ] The "𒀭 Fabric" tab appearance is indistinguishable in style from existing tabs
- [ ] No JavaScript is written — all logic is in Rust-WASM (web-sys + wasm-bindgen only)

---

## FILES TO CREATE OR MODIFY

| File | Action |
|---|---|
| `crates/bahyway-web/src/fabric_sim.rs` | **Create** — FabricSimState, SimSource, SimTarget, SimPipeline, SimException, QualityDistribution, FlowParticle |
| `crates/bahyway-web/src/lib.rs` | **Modify** — add `mod fabric_sim;`, thread_local FABRIC, register tab, register buttons, add animation intervals, add all draw functions |
| `crates/bahyway-web/index.html` | **Modify** — add Fabric tab button, panel div, sub-panel divs, CSS section |
| `crates/bahyway-web/Cargo.toml` | **No change needed** — bahyway-fabric is a separate crate; simulation data is embedded in fabric_sim.rs |

---

## BRANCH

All changes go to branch: `claude/focused-pascal-Do6ld`
Commit with message: `feat(bahyway-web): add Fabric Dashboard tab — pipeline map, lineage explorer, exception console, registry, quality report`

---

---

## PART 2 — DQM SLA Dashboard Panel (bahyway-dqm integration)

After completing Part 1, add a sixth sub-panel pill to the Fabric tab:

```
[ Pipeline Map ]  [ Lineage Explorer ]  [ Exception Console ]
[ Registry ]  [ Quality Report ]  [ DQM / SLA ]
```

---

### Sub-Panel 6: DQM / SLA (Canvas hex-radar + DOM SLA config table)

**Purpose:** Show all 6 DAMA-DMBOK data quality dimensions scored per record and
per batch, with configurable SLA thresholds and real-time compliance tracking.
This is the "data quality contract" panel — the client sees exactly which
dimension is failing and by how much.

**Read these files before implementing:**
- `crates/bahyway-dqm/src/dimensions.rs` — DqmDimension (6 variants), DimensionScore, DqmSla
- `crates/bahyway-dqm/src/report.rs`     — DqmReport, DqmBatchReport
- `crates/bahyway-dqm/src/algorithms/`  — Levenshtein, Jaro-Winkler, Soundex, Z-score
- `crates/bahyway-dqm/src/rules.rs`     — RuleEngine, rule_not_empty, rule_contains_char, etc.
- `crates/bahyway-dqm/src/merkle.rs`    — MerkleTree (lineage integrity proof)

---

#### Left half: Hexagonal Radar Chart (Canvas 480×480px)

Draw a hexagonal radar chart with 6 axes, one per DQM dimension.
The six axes radiate from the centre at 60° intervals, pointing to:
- Top:          Completeness  (0°)
- Top-right:    Validity      (60°)
- Bottom-right: Accuracy      (120°)
- Bottom:       Consistency   (180°)
- Bottom-left:  Uniqueness    (240°)
- Top-left:     Timeliness    (300°)

**Rendering layers (draw in order):**

1. **Grid rings** — 5 concentric hexagons at 20%, 40%, 60%, 80%, 100% of max radius.
   Color: `--border` (`#2a2a4a`), 1px stroke. Innermost ring is dashed.

2. **Axis lines** — 6 thin lines from centre to outer ring vertex.
   Color: `--border2` (`#44475a`), 1px stroke.

3. **SLA threshold polygon** — hexagon connecting the SLA threshold point on
   each axis. Color: `--tok-str` (`#f1fa8c`, yellow), 1.5px dashed stroke,
   fill with alpha 0.05. This shows the client's SLA contract boundary.

4. **Score polygon** — hexagon connecting the actual mean score on each axis.
   Color: determined by worst-dimension status:
   - All dimensions ≥ SLA: `--tok-lit` (`#50fa7b`, green), fill alpha 0.15
   - Any dimension < SLA but > 0.5×SLA: `--tok-op` (`#ffb86c`, orange), fill alpha 0.15
   - Any dimension < 0.5×SLA: `--tok-unk` (`#ff5555`, red), fill alpha 0.15

5. **Axis labels** — at the tip of each axis (outside the outer ring, 12px clearance).
   Text: dimension label (`"Completeness"` etc.), 11px monospace, `--text`.
   Dimension code badge: `"COMP"` etc., 9px, `--text-muted`, below label.

6. **Score dots** — 6 filled circles (5px radius) at the score position on each axis.
   Color: B11 quality lane colour (GEM=green, TRIBE=yellow, ACTIVE=orange,
   FUZZY=purple, DEAD=red).

7. **SLA breach indicator** — for any axis where score < SLA threshold:
   draw a red exclamation mark (!) 14px outside the dot position.

**Axis calculation:**
```
centre = (240, 240)   // canvas centre
max_r  = 180.0        // outer ring radius in px

axis_angle[i] = -π/2 + i * (2π/6)   // i = 0..5, starting top, clockwise

axis_tip[i]   = (centre.x + max_r * cos(axis_angle[i]),
                 centre.y + max_r * sin(axis_angle[i]))

score_point[i] = (centre.x + score[i] * max_r * cos(axis_angle[i]),
                  centre.y + score[i] * max_r * sin(axis_angle[i]))

sla_point[i]   = (centre.x + sla[i]   * max_r * cos(axis_angle[i]),
                  centre.y + sla[i]   * max_r * sin(axis_angle[i]))
```

---

#### Right half: SLA Configuration Table + Per-Dimension Detail (DOM, 480px wide)

**SLA Configuration table** (editable — the "client contract"):

| Dimension | Current Score | SLA Threshold | Status | B11 |
|---|---|---|---|---|
| Completeness | 0.99 | [0.98 ▲▼] | ✓ PASS | 238 |
| Validity | 0.94 | [0.95 ▲▼] | ✗ FAIL | 226 |
| Accuracy | 1.00 | [0.90 ▲▼] | ✓ PASS | 240 |
| Consistency | 0.97 | [0.95 ▲▼] | ✓ PASS | 233 |
| Uniqueness | 0.99 | [0.99 ▲▼] | ✓ PASS | 238 |
| Timeliness | 0.88 | [0.90 ▲▼] | ✗ FAIL | 211 |

- **SLA Threshold** column: `<input type="number" min="0" max="1" step="0.01">` styled to match
  dark theme. On change → update simulation SLA, redraw radar, update status.
- **Status** column: green "✓ PASS" (`--tok-lit`) or red "✗ FAIL" (`--tok-unk`).
- **B11** column: integer 0–240, coloured by quality lane.
- Row highlight: red background tint (`rgba(255,85,85,0.08)`) for failing rows.

**Three SLA preset buttons** (below the table):
```
[Enterprise Baseline]  [Exploratory]  [Master Data]
```
- `Enterprise Baseline`: completeness=0.98, validity=0.95, accuracy=0.90,
  consistency=0.95, uniqueness=0.99, timeliness=0.90
- `Exploratory`: all thresholds = 0.70–0.90 (relaxed)
- `Master Data`: completeness=1.00, validity=0.99, accuracy=0.99,
  consistency=0.99, uniqueness=1.00, timeliness=0.95

**Batch Compliance Summary** (below presets):

```
┌────────────────────────────────────────────────────────────┐
│  BATCH SLA COMPLIANCE — last 1,284 records                 │
│                                                            │
│  Overall:   97.3% compliant  (1,249 pass / 35 fail)       │
│                                                            │
│  COMP  ██████████ 99.2%   VALD  ████████░░ 86.4%          │
│  ACCY  ██████████ 100%    CONS  █████████░ 97.1%          │
│  UNIQ  ██████████ 99.8%   TIME  █████████░ 91.2%          │
│                                                            │
│  Worst dimension: VALIDITY (86.4% pass rate)               │
└────────────────────────────────────────────────────────────┘
```

- Mini bars: `--tok-lit` (green) fill, `--surface2` background, 8px height, 80px wide.
- SLA pass rate below threshold: bar turns `--tok-unk` (red).
- "Worst dimension" line: bold, coloured red if any dim < 90%.

---

#### Algorithm Explainer Panel (DOM, expandable — collapsed by default)

A collapsible section below the SLA table titled "▶ Algorithm Reference".
When expanded, shows a 2-column card grid with 6 algorithm cards:

| Card | Algorithm | Used For |
|---|---|---|
| Completeness | Field presence counting | Required field check |
| Validity | Rule engine + Z-score | Boolean constraints + outlier detection |
| Accuracy | Merkle tree integrity | Lineage hop verification |
| Consistency | Cross-field rules | Conflict marker detection |
| Uniqueness | Levenshtein · Jaro-Winkler · Soundex | Fuzzy dedup + phonetic match |
| Timeliness | Epoch freshness window | Source-to-pipeline latency |

Each card (120×90px, `--surface` background, `--border` border, 8px radius):
- **Algorithm name(s)** in `--tok-kw` (purple), 11px
- **Used for** label in `--text-muted`, 10px
- **Mini formula or example** in `--tok-fld` (cyan), 9px monospace:
  - Levenshtein: `d("kitten","sitting") = 3`
  - Jaro-Winkler: `JW("MARTHA","MARHTA") = 0.961`
  - Soundex: `soundex("Robert") = R163`
  - Z-score: `z = (x − μ) / σ`
  - Merkle: `root = H(H(A,B), H(C,D))`
  - Completeness: `score = present / required`

---

#### Simulation State additions for DQM

Add these fields to `FabricSimState` in `fabric_sim.rs`:

```rust
/// Current DQM SLA configuration (mirrors DqmSla from bahyway-dqm)
pub dqm_sla: SimDqmSla,

/// Per-dimension mean scores from last batch
pub dqm_scores: [f32; 6],   // index = DqmDimension::ALL order

/// Per-dimension SLA pass rates from last batch
pub dqm_pass_rates: [f32; 6],

/// Total records assessed by DQM this session
pub dqm_records_assessed: usize,

/// Records that failed at least one SLA threshold
pub dqm_sla_violations: usize,
```

```rust
pub struct SimDqmSla {
    pub thresholds: [f32; 6],  // index = DqmDimension::ALL order
}

impl SimDqmSla {
    pub fn enterprise() -> Self {
        SimDqmSla { thresholds: [0.98, 0.95, 0.90, 0.95, 0.99, 0.90] }
    }
    pub fn exploratory() -> Self {
        SimDqmSla { thresholds: [0.80, 0.75, 0.70, 0.75, 0.90, 0.70] }
    }
    pub fn master_data() -> Self {
        SimDqmSla { thresholds: [1.00, 0.99, 0.99, 0.99, 1.00, 0.95] }
    }
}
```

In `do_tick()`, after computing quality distribution, also compute DQM scores:
- Completeness: `0.85 + rng * 0.15` (good sources are mostly complete)
- Validity:     `0.80 + rng * 0.18` (some records fail rules)
- Accuracy:     `0.92 + rng * 0.08` (golden record comparison usually passes)
- Consistency:  `0.88 + rng * 0.12`
- Uniqueness:   `0.95 + rng * 0.05` (idu-prober catches most duplicates)
- Timeliness:   `0.75 + rng * 0.25` (depends on source latency)

where `rng` is a deterministic pseudo-random based on epoch:
```rust
fn tick_rng(epoch: u32, dimension: usize) -> f32 {
    let h = epoch.wrapping_mul(2654435761).wrapping_add(dimension as u32 * 0x9e3779b9);
    (h & 0xFFFF) as f32 / 65536.0
}
```

---

#### DQM Canvas Draw Function

```rust
fn draw_dqm_radar_canvas() {
    // 1. Clear canvas (#07070f)
    // 2. Draw 5 grid hexagons (20%..100% radius)
    // 3. Draw 6 axis lines
    // 4. Draw SLA threshold hexagon (dashed yellow)
    // 5. Draw score hexagon (green/orange/red fill)
    // 6. Draw axis label + code at each tip
    // 7. Draw score dots (coloured by B11 lane)
    // 8. Draw SLA breach (!) markers
    // 9. Update DOM compliance summary
}
```

---

#### Additional HTML for DQM sub-panel

```html
<div id="fabric-dqm" style="display:none">
  <div id="fabric-dqm-layout">
    <canvas id="fabric-dqm-canvas" width="480" height="480"></canvas>
    <div id="fabric-dqm-right">
      <table id="fabric-dqm-sla-table" class="fabric-reg-table"></table>
      <div id="fabric-dqm-presets">
        <button onclick="dqm_preset('enterprise')">Enterprise Baseline</button>
        <button onclick="dqm_preset('exploratory')">Exploratory</button>
        <button onclick="dqm_preset('master_data')">Master Data</button>
      </div>
      <div id="fabric-dqm-compliance"></div>
      <details id="fabric-dqm-algorithms">
        <summary style="cursor:pointer; color:var(--text-muted); font-size:12px">
          ▶ Algorithm Reference
        </summary>
        <div id="fabric-dqm-algo-cards"></div>
      </details>
    </div>
  </div>
</div>
```

CSS additions:
```css
#fabric-dqm-layout {
  display: flex;
  gap: 20px;
  align-items: flex-start;
}
#fabric-dqm-right {
  flex: 1;
  min-width: 0;
}
#fabric-dqm-presets {
  display: flex;
  gap: 8px;
  margin: 10px 0;
}
#fabric-dqm-presets button {
  background: var(--surface2);
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 6px;
  padding: 4px 12px;
  font-family: inherit;
  font-size: 11px;
  cursor: pointer;
}
#fabric-dqm-presets button:hover { border-color: var(--accent); color: var(--text); }
#fabric-dqm-compliance {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px 14px;
  font-size: 11px;
  margin-top: 10px;
}
#fabric-dqm-algo-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-top: 10px;
}
.dqm-algo-card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 10px;
}
.dqm-algo-card .algo-name    { color: var(--tok-kw);  font-size: 11px; font-weight: 600; }
.dqm-algo-card .algo-use     { color: var(--text-muted); font-size: 10px; margin: 2px 0; }
.dqm-algo-card .algo-formula { color: var(--tok-fld); font-size: 9px; font-family: inherit; }
```

---

## UPDATED ACCEPTANCE CRITERIA (adds to Part 1 checklist)

- [ ] DQM sub-panel pill "DQM / SLA" appears and switches to the panel
- [ ] Hexagonal radar chart renders with 5 grid rings, 6 axes, labels at each tip
- [ ] Yellow dashed SLA threshold hexagon matches the values in the SLA table
- [ ] Score polygon colour changes: green when all pass, orange/red when failing
- [ ] Score dots are coloured by B11 lane (GEM/TRIBE/ACTIVE/FUZZY/DEAD colours)
- [ ] Red `!` marker appears on any axis where score < SLA threshold
- [ ] SLA table has editable `<input>` per dimension; changing value redraws radar live
- [ ] "Enterprise Baseline" / "Exploratory" / "Master Data" buttons update all thresholds at once
- [ ] Batch compliance summary shows per-dimension mini bars with correct pass rates
- [ ] Algorithm reference panel expands/collapses; all 6 algorithm cards render with formula
- [ ] `do_tick()` updates DQM scores on each tick; radar redraws after each tick
- [ ] DQM scores in simulation use `tick_rng(epoch, dimension)` — deterministic, no `rand` crate

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | Fabric Dashboard Prompt | Updated 2026-06-04 with DQM/SLA Panel*

---

---

## PART 3 — Particle Motion & Tribe Orbit Dashboard Panel

After completing Parts 1 and 2, add a **seventh sub-panel** pill to the Fabric tab:

**Pill label:** `𒀭 Orbits`

---

### WHAT THIS PANEL SHOWS

The panel visualises how data particles move through the BahyWay quality manifold
and settle into their sovereign orbit lanes.  Three canvases render simultaneously:

| Canvas | ID | Size | Content |
|---|---|---|---|
| Orbit Ring Canvas | `orbit-canvas` | 480×480 | Concentric orbit lanes with animated particles |
| FSV Projection Canvas | `fsv-canvas` | 480×240 | 7D→2D scatter plot of particle quality geometry |
| Density Bar Canvas | `density-canvas` | 480×120 | Per-lane density bars with GEM-rate ADR-004 marker |

---

### ORBIT RING CANVAS — Concentric Orbit Visualisation

**Layout:** Five concentric rings centred at (240, 240), drawn outermost to innermost.

| Ring | Lane | Colour (Dracula) | Radius | Min B11 |
|---|---|---|---|---|
| 1 — outermost | DEAD    | `#6272A4` (comment) | 220 | 0    |
| 2             | FUZZY   | `#FF5555` (red)     | 180 | 60   |
| 3             | ACTIVE  | `#FFB86C` (orange)  | 140 | 100  |
| 4             | TRIBE   | `#8BE9FD` (cyan)    | 100 | 140  |
| 5 — innermost | GEM     | `#50FA7B` (green)   |  55 | 200  |

**Centroid dot:** A pulsing white circle (radius 6, opacity 0.9) at the canvas centre
(240, 240) labelled `⊕ centroid` in 9px JetBrains Mono.  It pulses (radius oscillates
4.5–7.5 over 2 s) to indicate the domain centroid is alive and self-calibrating.

**Orbit ring rendering (for each ring):**
```
ctx.beginPath();
ctx.arc(240, 240, ring_radius, 0, 2π);
ctx.strokeStyle = ring_colour;
ctx.lineWidth   = 1.5;
ctx.setLineDash([4, 4]);
ctx.stroke();
// Lane label at top of ring
ctx.fillStyle = ring_colour;
ctx.fillText(lane_label, 240, 240 - ring_radius + 12);
```

**Animated particles:** Each orbit lane holds a pool of `OrbitalParticle` objects.
A particle on lane L orbits at radius `ring_radius[L]` with a unique angular velocity:

```rust
pub struct OrbitalParticle {
    pub lane:        u8,     // 0=GEM 1=TRIBE 2=ACTIVE 3=FUZZY 4=DEAD
    pub angle:       f32,    // radians, updated each tick
    pub speed:       f32,    // radians per tick (lane-dependent)
    pub b11:         u8,     // particle's quality score
    pub radius_jitter: f32,  // ±8 px jitter for visual depth
    pub alpha:       f32,    // 0.4–1.0 depending on b11
}
```

Angular speed by lane (sovereign, do not change):
- GEM: `0.012` rad/tick
- TRIBE: `0.008` rad/tick
- ACTIVE: `0.005` rad/tick
- FUZZY: `0.003` rad/tick
- DEAD: `0.001` rad/tick (barely moves)

Particle dot rendering:
```js
const x = cx + (ring_radius + p.radius_jitter) * Math.cos(p.angle);
const y = cy + (ring_radius + p.radius_jitter) * Math.sin(p.angle);
ctx.beginPath();
ctx.arc(x, y, 3.5, 0, 2 * Math.PI);
ctx.fillStyle = lane_colour_with_alpha(p.lane, p.alpha);
ctx.fill();
```

**Particle migration animation:** When the orbit density changes (after a simulated
batch cleanse), particles smoothly migrate between rings:
- A NEW particle starts at radius 260 (outside all rings) at a random angle
- It spirals inward over 60 ticks until it reaches its target lane radius
- Alien-classified particles pulse red once and disappear (fade to alpha 0)

**Centroid drift indicator:** An arrow from the centroid toward the mean FSV direction
of the last batch.  Length = 20px × deviation from origin.  Colour = GEM green when
the centroid is well-calibrated (gem_depth ≥ 10), orange otherwise.

---

### FSV PROJECTION CANVAS — 7D Quality Geometry

The 7D Feature Score Vector is projected to 2D using the first two most-discriminating
dimensions as the axes:

- **X axis** → `D4 latin_density` (horizontal spread)
- **Y axis** → `D3 arabic_density` (vertical spread)

(This pair gives the widest natural separation for BahyWay's mixed Arabic/Latin data.)

Canvas coordinate mapping:
```
canvas_x = padding + fsv.latin_density  * (480 - 2*padding)
canvas_y = padding + (1.0 - fsv.arabic_density) * (240 - 2*padding)
padding  = 24
```

**Grid:** Light grey `#44475A` dashed grid at 0.0, 0.25, 0.5, 0.75, 1.0 on both axes.

**Particle dots:**
- Dot radius = 4px
- Fill colour = lane colour (same palette as orbit rings)
- Opacity = b11 / 240

**Centroid crosshair:** A `+` crosshair at the projected centroid position, 12px arm
length, 1px stroke, white `#F8F8F2`.

**Axis labels:** `latin_density →` at bottom, `↑ arabic_density` at left, 9px font.

**Legend** (top-right, 9px):
```
● GEM    ● TRIBE  ● ACTIVE  ● FUZZY  ● DEAD
```

---

### DENSITY BAR CANVAS — Lane Distribution

A horizontal stacked bar chart showing the current particle count per lane as a
fraction of total, with the ADR-004 GEM-rate target line.

```
Canvas: 480 × 120
Bar Y:  36 to 72  (36px tall)
Bar X:  padding=40 to 440  (400px wide)
```

Bars are drawn left-to-right in this order: GEM | TRIBE | ACTIVE | FUZZY | DEAD
Each bar segment width = `lane_density[L] × 400`.

**ADR-004 GEM-rate target line:** A vertical dashed line at
`x = 40 + 0.354 × 400 = 181.6` px, white `#F8F8F2`, 1px dashed, labelled
`ADR-004 target 35.4%` in 8px above the bar.

**Current GEM rate label:** Below bar at GEM segment right-edge:
`GEM rate: {rate:.1}%` in lane colour (green if ≥ 35.4%, orange otherwise).

**Per-lane count labels** inside each bar segment (if ≥ 24px wide):
`{count}` in 9px dark text.

---

### SIMULATION STATE ADDITIONS

Add to `FabricSimState` in `fabric_sim.rs`:

```rust
pub struct SimOrbitState {
    pub particles:       Vec<OrbitalParticle>,
    pub density:         [usize; 5],   // [GEM, TRIBE, ACTIVE, FUZZY, DEAD]
    pub total:           usize,
    pub mean_quality:    f32,
    pub centroid_depth:  u32,
    pub centroid_fsv:    [f32; 7],     // current domain centroid 7D coordinates
    pub gem_rate:        f32,
    pub last_batch_size: usize,
}

pub struct OrbitalParticle {
    pub lane:          u8,
    pub angle:         f32,
    pub speed:         f32,
    pub b11:           u8,
    pub radius_jitter: f32,
    pub alpha:         f32,
    pub migrating:     bool,
    pub migrate_ticks: u8,  // countdown from 60 → 0
}
```

**Initial state** (no data yet):
```rust
SimOrbitState {
    particles:      Vec::new(),
    density:        [0; 5],
    total:          0,
    mean_quality:   0.0,
    centroid_depth: 0,
    centroid_fsv:   [1.0; 7],  // starts at ideal point
    gem_rate:       0.0,
    last_batch_size: 0,
}
```

**Tick function** — on each `do_tick(epoch)`:

```rust
fn tick_orbit(state: &mut SimOrbitState, epoch: u32) {
    // 1. Advance all particle angles
    for p in &mut state.particles {
        p.angle += p.speed;
        if p.migrating && p.migrate_ticks > 0 {
            p.migrate_ticks -= 1;
            if p.migrate_ticks == 0 { p.migrating = false; }
        }
    }

    // 2. Every 12 ticks simulate a new incoming batch of 8 particles
    if epoch % 12 == 0 {
        let batch = simulate_incoming_batch(epoch, 8);
        for (lane, b11) in batch {
            state.density[lane as usize] += 1;
            state.total += 1;
            state.particles.push(OrbitalParticle {
                lane, b11, angle: tick_rng(epoch, lane as usize) * 2.0 * PI,
                speed: lane_speed(lane), radius_jitter: (tick_rng(epoch + 1, lane as usize) - 0.5) * 16.0,
                alpha: 0.4 + (b11 as f32 / 240.0) * 0.6,
                migrating: true, migrate_ticks: 60,
            });
        }
        // Cap at 120 particles for performance
        if state.particles.len() > 120 {
            state.particles.drain(0..8);
        }
        // Update centroid: GEM particles drift centroid toward ideal
        let gem_count = state.density[0];
        state.centroid_depth = gem_count as u32;
        if gem_count > 0 {
            let g = (gem_count as f32 / state.total as f32).min(1.0);
            for d in 0..7 {
                state.centroid_fsv[d] = state.centroid_fsv[d] * (1.0 - g * 0.05)
                    + 1.0 * (g * 0.05);
            }
        }
        state.gem_rate = state.density[0] as f32 / state.total as f32;
        state.mean_quality = state.particles.iter()
            .map(|p| p.b11 as f32 / 240.0)
            .sum::<f32>() / state.particles.len().max(1) as f32;
    }
}

fn simulate_incoming_batch(epoch: u32, n: usize) -> Vec<(u8, u8)> {
    (0..n).map(|i| {
        // Deterministic quality score
        let raw = tick_rng(epoch, i);
        let b11 = (raw * 240.0) as u8;
        let lane = match b11 {
            b if b >= 200 => 0,  // GEM
            b if b >= 140 => 1,  // TRIBE
            b if b >= 100 => 2,  // ACTIVE
            b if b >= 60  => 3,  // FUZZY
            _             => 4,  // DEAD
        };
        (lane, b11)
    }).collect()
}

fn lane_speed(lane: u8) -> f32 {
    match lane { 0 => 0.012, 1 => 0.008, 2 => 0.005, 3 => 0.003, _ => 0.001 }
}
```

---

### ORBIT PANEL CSS

Add to the `<style>` block (Dracula palette):

```css
/* ── Orbit Panel ─────────────────────────── */
#orbit-panel { display: none; padding: 12px; }
#orbit-panel.active { display: flex; flex-wrap: wrap; gap: 12px; }

.orbit-canvas-wrap {
  background: #282A36;
  border: 1px solid #44475A;
  border-radius: 6px;
  padding: 8px;
}
.orbit-canvas-wrap h4 {
  color: #BD93F9;
  font-size: 10px;
  margin: 0 0 6px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
#orbit-canvas  { display: block; width: 480px; height: 480px; }
#fsv-canvas    { display: block; width: 480px; height: 240px; }
#density-canvas{ display: block; width: 480px; height: 120px; }

.orbit-stats-row {
  display: flex; gap: 10px; flex-wrap: wrap; width: 100%;
}
.orbit-stat-card {
  background: #282A36;
  border: 1px solid #44475A;
  border-radius: 6px;
  padding: 8px 14px;
  min-width: 110px;
}
.orbit-stat-card .label { color: #6272A4; font-size: 9px; text-transform: uppercase; }
.orbit-stat-card .value { color: #F8F8F2; font-size: 18px; font-weight: bold; margin-top: 2px; }
.orbit-stat-card .sub   { color: #6272A4; font-size: 8px; }
.orbit-stat-card.gem-met .value { color: #50FA7B; }
.orbit-stat-card.gem-miss .value { color: #FF5555; }
```

---

### ORBIT PANEL HTML

```html
<!-- Orbit Panel (pill: 𒀭 Orbits) -->
<div id="orbit-panel">

  <!-- Stat Cards Row -->
  <div class="orbit-stats-row">
    <div class="orbit-stat-card" id="osc-total">
      <div class="label">Total Particles</div>
      <div class="value" id="osc-total-v">0</div>
      <div class="sub">in all orbit lanes</div>
    </div>
    <div class="orbit-stat-card" id="osc-gem">
      <div class="label">GEM Rate</div>
      <div class="value" id="osc-gem-v">0.0%</div>
      <div class="sub">ADR-004 target 35.4%</div>
    </div>
    <div class="orbit-stat-card" id="osc-centroid">
      <div class="label">Centroid Depth</div>
      <div class="value" id="osc-centroid-v">0</div>
      <div class="sub">GEM particles seen</div>
    </div>
    <div class="orbit-stat-card" id="osc-quality">
      <div class="label">Mean Quality</div>
      <div class="value" id="osc-quality-v">—</div>
      <div class="sub">composite B11</div>
    </div>
  </div>

  <!-- Orbit Ring Canvas -->
  <div class="orbit-canvas-wrap">
    <h4>𒀭 Tribe Orbit Lanes — Particle Motion</h4>
    <canvas id="orbit-canvas" width="480" height="480"></canvas>
  </div>

  <!-- FSV Projection Canvas -->
  <div class="orbit-canvas-wrap">
    <h4>𒁾 VGCA 7D → 2D Quality Manifold Projection</h4>
    <canvas id="fsv-canvas" width="480" height="240"></canvas>
  </div>

  <!-- Density Bar Canvas -->
  <div class="orbit-canvas-wrap" style="width:100%">
    <h4>⬛ Orbit Lane Density Distribution</h4>
    <canvas id="density-canvas" width="480" height="120"></canvas>
  </div>

</div>
```

---

### DRAWING FUNCTIONS

Add these functions to `lib.rs` (called each animation frame when orbit panel is active):

```rust
fn draw_orbit_canvas(state: &SimOrbitState, epoch: u32) {
    // canvas: orbit-canvas 480×480
    // 1. Clear to #1E1F29 (slightly darker than Dracula bg)
    // 2. Draw 5 concentric dashed rings (outermost=DEAD first, innermost=GEM last)
    //    — ring radii: [220, 180, 140, 100, 55]
    //    — ring colours: ["#6272A4","#FF5555","#FFB86C","#8BE9FD","#50FA7B"]
    //    — lane labels drawn at top arc of each ring
    // 3. Draw all particles (OrbitalParticle::lane → radius lookup)
    //    — migrating particles: draw at lerped radius (260 → target_radius, migrate_ticks/60)
    //    — non-migrating: draw at target_radius + radius_jitter
    // 4. Draw centroid pulse (pulsing dot at 240,240)
    //    — pulse_r = 4.5 + 3.0 * sin(epoch as f32 * 0.05)
    // 5. Draw centroid drift arrow toward centroid_fsv mean
    // 6. Update stat card DOM elements
}

fn draw_fsv_canvas(state: &SimOrbitState) {
    // canvas: fsv-canvas 480×240
    // 1. Clear background
    // 2. Draw dashed grid (5 lines each axis, colour #44475A)
    // 3. For each particle, compute (canvas_x, canvas_y) from (latin_density, arabic_density)
    //    Use last-known FSV approximation: fsv[3]=latin, fsv[2]=arabic from centroid_fsv
    //    For individual particles: use tick_rng(particle_id, dim) as proxy FSV dimensions
    // 4. Draw centroid crosshair at projected centroid position
    // 5. Draw axis labels and legend
}

fn draw_density_canvas(state: &SimOrbitState) {
    // canvas: density-canvas 480×120
    // 1. Clear background
    // 2. Draw stacked bar: for each lane L, width = density[L]/total * 400
    //    — start x = 40, bar spans y=36..72
    //    — colours: GEM=#50FA7B, TRIBE=#8BE9FD, ACTIVE=#FFB86C, FUZZY=#FF5555, DEAD=#6272A4
    // 3. Draw ADR-004 vertical dashed line at x=40+0.354*400=181.6
    //    — label "ADR-004 ▶ 35.4%" above
    // 4. Draw count labels inside each bar segment (if width ≥ 24px)
    // 5. Draw GEM rate label below GEM segment, coloured green/orange by ADR-004
}
```

---

### ANIMATION

The orbit panel animation runs in a `setInterval` at **30 ms** (≈33 fps) when the
orbit pill is active.  It calls:
1. `tick_orbit(&mut orbit_state, epoch)` — advances particles, simulates new batch every 12 ticks
2. `draw_orbit_canvas(&orbit_state, epoch)`
3. `draw_fsv_canvas(&orbit_state)`
4. `draw_density_canvas(&orbit_state)`

Use the same `tick_rng(epoch, dim)` function from Part 2 for all deterministic simulation.

---

### FSIM STATE INTEGRATION

`FabricSimState` in `fabric_sim.rs` gains one field:
```rust
pub orbit: SimOrbitState,
```
Initialised in `FabricSimState::new()` with `SimOrbitState::new()`.

---

### UPDATED ACCEPTANCE CRITERIA (adds to Parts 1+2 checklist)

- [ ] `𒀭 Orbits` pill appears and switches to the orbit panel
- [ ] Five concentric dashed rings render in the correct Dracula lane colours
- [ ] Particles animate clockwise on each ring at the correct lane speed
- [ ] New particles enter from the outer edge (r=260) and spiral inward to their lane
- [ ] Alien particles flash red and disappear (not placed on any ring)
- [ ] Centroid dot pulses at the canvas centre; pulsing stops if gem_depth = 0
- [ ] Centroid drift arrow points toward the mean quality direction
- [ ] FSV projection canvas renders scatter dots coloured by lane
- [ ] Centroid crosshair appears on FSV canvas at the correct projected position
- [ ] Density bar canvas shows correct stacked proportions per lane
- [ ] ADR-004 35.4% target line renders on density canvas
- [ ] GEM rate stat card turns green (`#50FA7B`) when GEM rate ≥ 35.4%
- [ ] GEM rate stat card turns red (`#FF5555`) when GEM rate < 35.4%
- [ ] All three canvases update every 30 ms when orbit panel is visible
- [ ] No JavaScript — all canvas drawing logic is in Rust-WASM

---

## OVERALL PANEL PILL ORDER (final)

| # | Pill Label | Panel ID |
|---|---|---|
| 1 | `🗺 Pipeline Map`    | `pipeline-panel`   |
| 2 | `🔗 Lineage`         | `lineage-panel`    |
| 3 | `⚠ Exceptions`       | `exceptions-panel` |
| 4 | `📋 Registry`        | `registry-panel`   |
| 5 | `📊 Quality Report`  | `quality-panel`    |
| 6 | `DQM / SLA`          | `dqm-panel`        |
| 7 | `𒀭 Orbits`          | `orbit-panel`      |

---

*𒁾 DUB.SAR — BahyWay.Ecosystem v4.0 | Fabric Dashboard Prompt | Updated 2026-06-04 with Particle Motion + Tribe Orbit Panel*
