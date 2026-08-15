# BahyWay v4 — Developer Manual
## bahyway-web · bahyway-api · bee-watchdog Live Stack

> Branch: `claude/serene-goldberg-tpobS` on `bahyway/EnkiDB`  
> Last updated: 2026-06-02

---

## 1. Quick Start — Full Live Stack

```bash
# 1. Build native binaries (on the Vagrant box or CI)
cd workspace/bahyway_v4
cargo build -p bee-watchdog --release
cargo build -p bahyway-api  --release

# 2. Start the API bridge (reads JSON exports from data_dir)
./target/release/bahyway-api \
    --data-dir /var/enkidb \
    --port 8082

# 3. Start the ETL daemon (writes JSON exports to data_dir)
./target/release/bee-watchdog \
    --shard    /vagrant/shard \
    --data-dir /var/enkidb \
    --tribe-id 0x0001

# 4. Build & serve the WASM frontend
trunk serve   # dev server on http://localhost:8080

# 5. Drop a batch — bee-watchdog picks it up within 2 s
cp my_records.zip /vagrant/shard/
```

After processing, the Dashboard tab shows totals within ~12 s (auto-refresh 10 s).  
The Tribes tab shows the new batch row within ~30 s (auto-refresh 30 s) or click **⟳ Refresh**.

---

## 2. Architecture

```
Browser (localhost:8080)
  bahyway-web WASM
      │
      │  fetch (CORS, every 10 / 30 s)
      ▼
  bahyway-api (localhost:8082)
      │  reads files from data_dir
      ▼
  /var/enkidb/
      ├── live_export.json       ← aggregate totals
      └── batches_export.json    ← ring buffer, last 20 batches
      ▲
  bee-watchdog
      │  writes after each ZIP batch
      ▼
  EnkiDB station chain
  LandingZone → Musarû gate → ZIP → ProcessingZone →
  BatchSchema → ClientDqProfile → FuzzyScore →
  Golden / Fuzzy / Dead → PersistedDb + PermanentStore
```

---

## 3. bahyway-web Panels

| Tab button ID | Panel div ID | What it shows |
|---------------|-------------|---------------|
| `tab-dashboard` | `panel-dashboard` | Live MDM Data card (10 s auto-refresh) · headline metrics · tribe health bars · drift chart · steward alert queue |
| `tab-hubble` | `panel-hubble` | Multi-level zoom: Universe view → click tribe → Tribe zoom → click particle → StoryEngine |
| `tab-notebook` | `panel-notebook` | HeptaScript live query · preset buttons · syntax-highlighted token stream · results table · live corpus banner |
| `tab-pipeline` | `panel-pipeline` | Five-tier Canvas 2D animation: SDB → ValidationSweep → ODB / QDB → Journal · 800 ms auto-tick |
| `tab-probe` | `panel-probe` | SC+GA PROBE — Vietoris-Rips complex at adjustable ε · Betti numbers β₀/β₁ · Cl(2,0) rotor path |
| `tab-tribes` | `panel-tribes` | **Live Batch Feed** (30 s auto-refresh) · Star Map canvas · tribe cards · detail drawer · StoryEngine panel · Compare sub-tab |
| `tab-schema` | `panel-schema` | Client DQ Profile editor · per-attribute nullable/threshold/dim/weight table · .dqprofile export · Particles Data Model canvas |

---

## 4. Live Data API

All endpoints served by `bahyway-api` on port 8082.  
CORS headers: `Access-Control-Allow-Origin: *` on every response.

### `GET /api/v1/live`

Served from `{data_dir}/live_export.json`.  
Written by `bee-watchdog` → `write_live_export()` after each ZIP batch.

```json
{
  "status":     "ok",
  "timestamp":  1748900000,
  "last_batch": "najaf_cemetery_001",
  "total":      130,
  "golden":     112,
  "fuzzy":       18,
  "dead":         0,
  "batches":      3
}
```

No-data fallback:
```json
{"status":"no_data","msg":"No export yet — run bee-watchdog to process a ZIP batch."}
```

### `GET /api/v1/batches`

Served from `{data_dir}/batches_export.json`.  
Written by `bee-watchdog` → `write_batches_export()` — ring buffer of last 20 batches.

```json
{
  "status":   "ok",
  "tribe_id": "0x0001",
  "count":    3,
  "batches": [
    {"name":"najaf_001","timestamp":1748899000,"golden":40,"fuzzy": 8,"total":48},
    {"name":"najaf_002","timestamp":1748899600,"golden":38,"fuzzy":10,"total":48},
    {"name":"najaf_003","timestamp":1748900000,"golden":34,"fuzzy":12,"total":46}
  ]
}
```

No-data fallback:
```json
{"status":"no_data","count":0,"batches":[],"msg":"No batches processed yet."}
```

### `GET /health`

Returns `200 ok` — liveness probe.

---

## 5. bee-watchdog ETL Chain

```
shard/*.zip detected by LandingZone::poll()
    │
    ├─ musaru_scan()          ← SECURITY GATE — blocks malware signatures
    ├─ zip_engine::extract_ok()
    ├─ ProcessingZone::stage()    → Processing/{batch}_{ts}/
    ├─ BatchSchema::infer()       → writes .schema file
    ├─ load or auto-generate ClientDqProfile
    │      from shard/profiles/{batch}.dqprofile
    │
    └─ per record:
         KakiGenerator::generate()       → IdentityKaki + EventKaki + EAV
         structure(template, eav)        → normalised EAV
         compute_dims(eav, profile)      → FuzzyDimensions D1-D8
         fuzzy_score(ScoreInput)         → B11 + ColorRGB + tier
         tier_to_state(tier)             → Golden / Fuzzy / Dead

         Golden (B11 ≥ 140)  → PermanentStore + PersistedDb
         Fuzzy  (B11 100-139) → PersistedDb + StewardStation alert
         Dead   (B11 < 100)   → PersistedDb (log only)

    ├─ batch KAKI event → EnkiDb journal
    ├─ ProcessingZone::complete()  → Moved_To/{batch}_{ts}/
    ├─ write_live_export()         → {data_dir}/live_export.json
    └─ write_batches_export()      → {data_dir}/batches_export.json
```

### bee-watchdog CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--shard <dir>` | required | LandingZone root — shared Vagrant folder |
| `--data-dir <dir>` | required | Where JSON exports and PersistedDb are stored |
| `--tribe-id <hex>` | required | e.g. `0x0001` |
| `--interval-ms <ms>` | `2000` | Poll interval |

---

## 6. Client DQ Profile

A `.dqprofile` file lives at `shard/profiles/{batch_name}.dqprofile`.  
If missing, bee-watchdog auto-generates one from the BatchSchema.

### File format

```
# Client DQ Profile — auto-generated from schema
schema_name = najaf_cemetery_001
format_layer = CivilRegistry
source_trust = Official
gem_threshold = 200
tribe_threshold = 140

[attributes]
  name         nullable=false fill=90 dim=Name     weight=25
  national_id  nullable=false fill=95 dim=Identity weight=30
  date_of_death nullable=true fill=50 dim=Date     weight=15
  notes         nullable=true fill=20 dim=General  weight=5
```

### DimTag auto-detection heuristics

| Column name pattern | Assigned DimTag |
|--------------------|-----------------|
| `name`, `first_name`, `last_name` | `Name` |
| `id`, `national_id`, `passport` | `Identity` |
| `date`, `birth`, `death`, `dob` | `Date` |
| `city`, `address`, `country`, `region` | `Geography` |
| `notes`, `description`, `text` | `Content` |
| (everything else) | `General` |

### Schema Config GUI (Schema tab)

1. Paste `.schema` file contents into the textarea
2. Click **Load Schema** — attributes populate the table
3. Per attribute: toggle Nullable, adjust Fill Threshold slider, cycle DimTag, set Weight
4. Click **⬇ Export .dqprofile** — copy the output to `shard/profiles/{batch}.dqprofile`

---

## 7. FuzzyScore / B11 Thresholds

| B11 range | State | Label | Action |
|-----------|-------|-------|--------|
| 140–240 | `Golden` | Gem / Tribe | PermanentStore + PersistedDb |
| 100–139 | `Fuzzy` | Active | PersistedDb + StewardStation alert |
| 0–99 | `Dead` | NonActive | PersistedDb only |

`QUALITY_DIVISOR = 240.0` — never 255 (ADR-001).

FuzzyDimensions computed by `compute_dims()`:
- **D1** Completeness — fill rate of mandatory attributes
- **D2** Validity — format conformance per dim tag
- **D3** Consistency — cross-field constraints
- **D4** Uniqueness — deduplication signal
- **D5** Timeliness — freshness / SATTATU decay (54 h max)
- **D6** Accuracy — reference match
- **D7** Lineage — provenance / source trust level
- **D8** Integrity — CRC + journal hash

---

## 8. WASM Event Wiring Pattern

All DOM event handlers follow this pattern — **no inline HTML `onclick` attributes**:

```rust
// 1. Render HTML with element IDs
set_html("my-container", &format!(r#"<button id="my-btn-{i}">Click</button>"#, i=i));

// 2. Attach Closure-based listeners after rendering
let cb = Closure::wrap(Box::new(move |_: Event| {
    // handler logic
    render_my_view();
    attach_my_events(n);   // re-attach after full re-render
}) as Box<dyn FnMut(Event)>);
if let Some(el) = doc().get_element_by_id(&format!("my-btn-{i}")) {
    el.add_event_listener_with_callback("click", cb.as_ref().unchecked_ref()).ok();
}
cb.forget();  // leak intentionally — handler lives for page lifetime
```

**Rule**: after any `set_html()` that replaces a container, re-call the corresponding `attach_*_events()` function.

---

## 9. WASM Async Fetch Pattern

```rust
fn fetch_my_data() {
    wasm_bindgen_futures::spawn_local(async {
        let window = match web_sys::window() { Some(w) => w, None => return };
        set_html("my-status", "<span>⟳ fetching…</span>");

        let p = match window.fetch_with_str("http://localhost:8082/api/v1/live") {
            Ok(p) => p,
            Err(_) => { set_html("my-status", "⚠ API unreachable"); return; }
        };
        let resp_val = match JsFuture::from(p).await {
            Ok(v) => v,
            Err(_) => { set_html("my-status", "⚠ network error"); return; }
        };
        let resp: web_sys::Response = match resp_val.dyn_into() { Ok(r) => r, Err(_) => return };
        if !resp.ok() { return; }
        let text_p = match resp.text() { Ok(p) => p, Err(_) => return };
        let text_val = match JsFuture::from(text_p).await { Ok(v) => v, Err(_) => return };
        let text = text_val.as_string().unwrap_or_default();
        parse_and_render(&text);
    });
}
```

Manual JSON parsing (no serde):
```rust
fn num(json: &str, key: &str) -> usize {
    let needle = format!("\"{}\":", key);
    json.find(&needle)
        .and_then(|pos| {
            let rest = &json[pos + needle.len()..];
            let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            rest[..end].trim().parse().ok()
        })
        .unwrap_or(0)
}
```

---

## 10. Completed Milestones (this session series)

| # | Milestone | Key files changed | Commit |
|---|-----------|------------------|--------|
| M1 | `client-dq-profile` crate | `crates/client-dq-profile/src/{profile,compute,parse}.rs` | `8fd2d20` |
| M2 | Schema Config GUI + Particles Data Model | `bahyway-web/src/schema_config.rs`, `lib.rs`, `index.html` | prev session |
| M3 | HTTP API bridge | `bin/bahyway-api/src/main.rs`, `bee-watchdog/src/main.rs`, `bahyway-web/src/lib.rs`, `index.html` | `4e41de0` |
| M4 | Live Batch Feed | Same files — extended with batches_export.json, /api/v1/batches, Tribes panel card | current |

---

## 11. Upcoming Milestones

### M5 — Real Particle Data in Tribes / Notebook *(large, ~2 sessions)*

Replace `TRIBES` / `PARTICLES` static arrays with runtime-loaded data.

**Steps:**
1. Add `/api/v1/tribes` endpoint — serves `tribes_export.json` written by bee-watchdog
2. bee-watchdog accumulates per-tribe particle summaries across batches
3. `bahyway-web`: `DynamicTribe` / `DynamicParticle` in `RefCell<Vec<...>>` thread-locals
4. Fetch on tab switch; re-render tribes panel from live data
5. Fallback: demo data when API unreachable

---

### M6 — Multi-Tribe Support *(medium)*

bee-watchdog accepts `--tribe-config tribes.toml`; one thread per tribe;  
`/api/v1/tribes/all` aggregates across all tribe data dirs.

---

### M7 — Persistent StoryEngine *(medium)*

`/api/v1/story/{entity_hash}` reads PersistedDb event journal and returns JSON events.  
StoryEngine canvas unchanged — only data source swaps from static `STORIES[pi]` to fetched events.

---

### M8 — .dqprofile Round-Trip *(small)*

`/api/v1/profiles` lists available profiles; `/api/v1/profiles/{name}` returns text.  
Schema tab gains a "Load from API" dropdown.

---

### M9 — WASM Build CI *(small)*

`.github/workflows/wasm-build.yml` — `trunk build --release` on push.

---

### M10 — Dubsar IDE Integration *(large — Tier 1)*

`dubsar-ide` compiled to WASM32; embedded in Notebook tab as a proper multi-line editor.

---

### M11 — Nigin Topology Scanner + Graph GUI *(large — Tier 1 now, Tier 2 upgrade later)*

**NIGIN** (Akkadian: "to encircle, survey, gather all") — the self-image of the platform.
Renders the full BahyWay ecosystem as an interactive dependency graph for Administrators
and Architects. The same tool that proved the SQL Server SP/UDF call chain, now sovereign.

#### Tier 1 delivery (Vagrant / Canvas 2D — no GPU required)

```
bin/niginway/                   ← new native binary (std::thread, no Tokio)
    src/main.rs                 ← scan orchestrator + HTTP server on :8083
    src/static_scanner.rs       ← reads Cargo.toml files → node + edge list
    src/policy_scanner.rs       ← walks shard/policies/*.akk for declared endpoints
    src/graph_export.rs         ← writes {data_dir}/graph_export.json

bahyway-api                     ← adds /api/v1/graph serving graph_export.json

bahyway-web panel-graph         ← new Canvas 2D tab
    Canvas force-directed layout (repulsion + spring, 60fps)
    Click node  → highlight callers (red) / callees (blue)
    Hover       → tooltip: layer · role · health
    ⟳ Refresh   → re-fetch /api/v1/graph → re-animate diff
    Lane colour: White=confirmed / Gray=static-only / Black=shadow
    Diff pulse:  new=blue-fade / dropped=red-outline / lane-shift=amber
    Filter bar: layer checkboxes (L0–L12)
    Stakeholder gate: istar ABAC — Admin sees lanes+health, Architect sees full provenance
```

**graph_export.json schema:**
```json
{
  "scan_ts": 1748900000,
  "nodes": [
    {"kaki": "...", "name": "enkidb-kaki", "layer": 1,
     "role": "KAKI/Identity", "health": "ok"}
  ],
  "edges": [
    {"from": "bee-watchdog", "to": "enkidb-kaki",
     "lane": "White", "count": 0, "last_seen": 0}
  ],
  "diff": {"added": [], "removed": [], "lane_shifts": []}
}
```

**Edge lanes (White / Gray / Black axiom):**
| Lane | Meaning | Action |
|------|---------|--------|
| White | Declared in Cargo.toml AND observed in journal | Confirmed — no action |
| Gray | Declared in Cargo.toml, never observed at runtime | Dead path — needs evidence |
| Black | Observed at runtime, NOT declared | Shadow call — policy violation |

#### Tier 2 upgrade (Bare Metal / Bevy — when on MSI Prestige 15 + Fedora SilverBlue)

Replace Canvas 2D panel with Bevy 3D scene:
- ServiceParticle and CalleeEdge as first-class KAKI particles in EnkiDB
- Force-directed layout in 3D space — FuzzyDimensions D1-D8 as axes
- Tokio-async scan orchestrator (Static + Runtime + Policy in parallel)
- `petgraph` for topology algorithms (shortest path, cycle detection)
- Animated diff: Hanabi particle effects on lane shifts, new/dropped services
- Stakeholder camera: Admin view (health + lanes) vs Architect view (full provenance)

**Three-week Tier 2 shape:**
- Week 1: `niginway` static scanner + ServiceParticle/CalleeEdge in EnkiDB
- Week 2: Tokio runtime scanner (journal events) + policy scanner (.akk files)
- Week 3: Bevy 3D scene + Refresh button + diff rendering + stakeholder gate

> This tool is the self-image of the platform — BahyWay rendering itself.
> Build before sealing v4.0; the screenshots anchor IP filing and investor demos.

---

## 12. Constraints

### 12a. Universal — all deployments, all crates

```
#![forbid(unsafe_code)]          — every crate, no exceptions
QUALITY_DIVISOR = 240.0          — ADR-001, never 255
SATTATU_MAX = 54 hours           — max credential / record age
"Way" suffix                     — reserved for WAYv2.0 Language files only
```

**Forbidden external crates (universal):**  
`serde`, `thiserror`, `csv`, `tokio`, `tonic`, `lalrpop`, `nom`, `sha2`, `rand`,
`prometheus`, `openraft`, `logos`, `blake3`, `rayon`, `lru`, `ndarray`, `nalgebra`,
`bincode`, `uuid`, `crc16`, `bloomfilter`

---

### 12b. ADR-005 — UI Deployment Tiers (see ROADMAP.md)

> **The "pure DOM + Canvas 2D" rule is a Vagrant environment constraint,
> NOT a permanent architectural constraint.**

| Tier | Environment | UI Stack | Status |
|------|-------------|----------|--------|
| **Tier 1 — Vagrant** | Windows host · Vagrant Fedora box · no GPU passthrough | Pure DOM + HTML Canvas 2D (bahyway-web WASM) | ✅ Current |
| **Tier 2 — Bare Metal** | MSI Prestige 15 · Fedora SilverBlue · full GPU | Bevy + egui · 7D (3D spatial) rendering | Planned |

The real architectural target is **7-dimensional 3D** — FuzzyDimensions D1-D8 rendered as spatial geometry in a Bevy ECS scene. The Canvas 2D panels in `bahyway-web` are the Vagrant-compatible transport layer and remain available as a fallback / web-embed layer in all tiers.

Crates that will become available in Tier 2 (bare-metal only):
- `bevy` — ECS game engine for 3D scene rendering
- `egui` / `bevy_egui` — immediate-mode GUI panels
- `tokio` — async runtime for parallel scan orchestrators (e.g. Nigin scanner)
- `petgraph` — graph topology algorithms

These are **not forbidden** — they are **Vagrant-incompatible** and therefore deferred to Tier 2.

---

*𒁾 DUB.SAR — BahyWay v4 Developer Manual | 2026-06-02*
