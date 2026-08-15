# NaviEngine — Developer Manual

> **DubSar Help** | `Manuals > NaviEngine` | Crate Reference

## Overview

`navi-engine` is the sovereign navigation and map layer for all EnkiDB engines.
It provides KAKI-native routing over an offline pure-Rust GIS with HubbleZoom
particle visibility, typed EAV attributes, pipeline engineering data, and
named bookmark storage.

No external GIS. No third-party routing. Everything is a particle.

Coordinates reference: `WADI_AL_SALAM_LAT = 31.995°N, WADI_AL_SALAM_LON = 44.320°E`

---

## Module Map

```
navi-engine/src/
  eav.rs        — EavStore: typed Entity-Attribute-Value with FNV-1a keys
  mapparticle.rs — MapParticle: sovereign map atom + MapKind taxonomy
  tile.rs        — HubbleTile: 7-level zoom system Z0–Z6
  particlemap.rs — ParticleMap: pure-Rust GIS (SpatialGrid + KAKI index + adjacency)
  machrouter.rs  — MachineRouter: KAKI-native A* over ParticleMap
  bookmark.rs    — BookmarkStore: named sovereign locations with 6 categories
  pipeline.rs    — WpdPipelineMap: pipeline engineering domain layer
  particle.rs    — NaviCoord, NaviParticle primitives
  route.rs       — haversine_m, PilgrimRoute, cost utilities
  graph.rs       — NaviGraph (NajafEngine routing graph)
  sensor.rs      — SensorEvent, SensorFeed
```

---

## EAV — Sovereign Attribute Store

Every `MapParticle` carries an `EavStore` of typed attributes.  Keys are
FNV-1a 32-bit hashes of the attribute name, computed at compile time.

### Pre-defined Attribute Keys

| Constant             | Attribute name     | Type        | Usage                                      |
|----------------------|--------------------|-------------|---------------------------------------------|
| `ATTR_NAME`          | `"name"`           | Text        | Latin/English road or POI name              |
| `ATTR_NAME_ARABIC`   | `"name_arabic"`    | Text        | Canonical Arabic name — sovereign label     |
| `ATTR_SPEED_LIMIT`   | `"speed_limit"`    | UInt (km/h) | Road speed limit                            |
| `ATTR_LANES`         | `"lanes"`          | UInt        | Number of traffic lanes                     |
| `ATTR_ONE_WAY`       | `"one_way"`        | Bool        | One-way traffic restriction                 |
| `ATTR_ROAD_CLASS`    | `"road_class"`     | UInt (u8)   | RoadClass discriminant                      |
| `ATTR_DIAMETER_MM`   | `"diameter_mm"`    | UInt        | Pipeline inner diameter in mm               |
| `ATTR_PRESSURE_KPA`  | `"pressure_kpa"`   | UInt        | Pipeline operating pressure in kPa          |
| `ATTR_FLOW_DIR`      | `"flow_dir"`       | UInt (u8)   | FlowDir discriminant                        |
| `ATTR_SACRED_WEIGHT` | `"sacred_weight"`  | Float       | NajafEngine zone routing priority           |
| `ATTR_ELEVATION_M`   | `"elevation_m"`    | Float       | Elevation above WGS-84 ellipsoid            |
| `ATTR_PIPE_MATERIAL` | `"pipe_material"`  | UInt (u8)   | PipeMaterial discriminant                   |
| `ATTR_AGE_YEARS`     | `"age_years"`      | UInt        | Infrastructure age for condition assessment |
| `ATTR_ACCESS_LEVEL`  | `"access_level"`   | UInt        | 0=public 1=permit 2=restricted 3=sovereign  |

### AttrValue

```rust
pub enum AttrValue {
    Text(String),
    Int(i32),
    UInt(u32),
    Float(f32),
    Bool(bool),
    KakiRef(u32),  // points to another sovereign particle by uuid_hash
}
```

### EavStore API

```rust
use navi_engine::{EavStore, EavAttr, AttrValue, ATTR_NAME_ARABIC, ATTR_SPEED_LIMIT};

let mut eav = EavStore::new();
eav.set(EavAttr::required(ATTR_NAME_ARABIC, AttrValue::Text("شارع الإمام".into())));
eav.set(EavAttr::optional(ATTR_SPEED_LIMIT, AttrValue::UInt(60)));

eav.get_text(ATTR_NAME_ARABIC)  // → Some("شارع الإمام")
eav.get_uint(ATTR_SPEED_LIMIT)  // → Some(60)
eav.required_missing()          // → Vec<AttrKey> of required attrs not yet set
eav.attr_count()                // → usize
```

---

## MapParticle — Sovereign Map Atom

```rust
pub struct MapParticle {
    pub particle_id: u32,
    pub kaki:        Kaki,            // minted at new()
    pub coord:       NaviCoord,
    pub kind:        MapKind,
    pub tribe:       TribeId,
    pub zoom_min:    u8,              // minimum HubbleZoom level for visibility
    pub eav:         EavStore,
    pub state:       NaviParticleState,
}
```

KAKI is minted automatically via `KakiMinter` in `MapParticle::new()`.

### MapKind

| Variant                              | routing_multiplier | vehicle | pedestrian |
|--------------------------------------|--------------------|---------|------------|
| `Road { class, one_way }`            | per RoadClass      | depends | depends    |
| `Pipeline { kind, flow }`            | 1.00               | false   | false      |
| `Poi { category }`                   | 0.80               | false   | true       |
| `Junction`                           | 0.70               | true    | true       |
| `GraveMarker`                        | 0.95               | false   | true       |

### RoadClass — Cost Multipliers

| Class        | Multiplier | Vehicle | Pedestrian |
|--------------|------------|---------|------------|
| `Motorway`   | 0.60       | true    | false      |
| `Primary`    | 0.70       | true    | false      |
| `Secondary`  | 0.80       | true    | true       |
| `Tertiary`   | 0.90       | true    | true       |
| `Residential`| 1.10       | true    | true       |
| `Service`    | 1.20       | true    | true       |
| `Footway`    | 2.00       | false   | true       |

### PipelineKind

`WaterSupply` / `Sewage` / `Drainage` / `Gas` / `Power` / `Irrigation` / `Telecoms`

### FlowDir

`UpStream` / `DownStream` / `Bidirectional`

---

## HubbleTile — 7-Level Zoom System

```
Z0 Global   10°      Countries, major cities
Z1 Regional  2°      Provinces, roads
Z2 City      0.5°    Districts, streets
Z3 District  0.1°    Blocks, buildings
Z4 Block     0.02°   Cemetery sections, manholes
Z5 Feature   0.005°  Individual graves, pipe joints
Z6 Precision 0.001°  Sub-meter landmarks, inscriptions
```

```rust
use navi_engine::{HubbleTileId, HUBBLE_ZOOM_MAX, zoom_for_radius_m, tiles_in_bbox};

// Tile containing Wadi al-Salam at Z5
let tile = HubbleTileId::from_coord(31.995, 44.320, 5);

// Bounding box of the tile
let bounds = tile.bounds();           // TileBounds { lat_min, lat_max, lon_min, lon_max }
bounds.contains(31.995, 44.320)       // → true

// Navigate the hierarchy
let parent = tile.parent();           // Option<HubbleTileId>
let children = tile.children();       // Vec<HubbleTileId> — n×n, NOT quadtree

// Tile size ratios between levels: [5, 4, 5, 5, 4, 5]
// Z0→Z1=5:1 (25 children), Z1→Z2=4:1 (16 children), ...

// Utility functions
let zoom = zoom_for_radius_m(500.0);  // coarsest zoom whose tile fits radius
let ids  = tiles_in_bbox(31.980, 32.010, 44.300, 44.340, 4);
```

`HubbleTile` (the container object) uses `contains_particle(zoom_min)` to
filter which particles are visible at a given zoom level.

---

## ParticleMap — Sovereign GIS

Pure-Rust offline map: spatial grid + KAKI index + adjacency list.
No third-party GIS, no R-tree, no external spatial indexing.

```rust
use navi_engine::{ParticleMap, MapBounds, MapEdge, MapParticle};

// Create with a geographic bounding box
let bounds = MapBounds { lat_min: 31.95, lat_max: 32.05,
                          lon_min: 44.28, lon_max: 44.37 };
let mut map = ParticleMap::new(bounds, 0.01); // cell_size_deg = 0.01°

// Register a particle
map.add(particle);

// Convenience: register + auto-compute edges to nearby particles
map.place(particle, 100.0); // radius_m for auto-edge detection

// Manual edge with known distance
map.add_edge(from_id, to_id, distance_m);

// Auto-distance edge (haversine computed)
map.add_edge_auto_dist(from_id, to_id);

// Spatial queries
let nearby: Vec<&MapParticle> = map.query_radius(&coord, 250.0);
let nearest: Option<&MapParticle> = map.nearest(&coord);
let in_tile: Vec<&MapParticle> = map.particles_in_tile(&tile_id);

// KAKI lookup
let p: Option<&MapParticle> = map.by_kaki(kaki_hash);
```

Edge cost = `distance_m × average(from.kind.routing_multiplier, to.kind.routing_multiplier)`

---

## MachineRouter — KAKI-Native A*

```rust
use navi_engine::{MachineRouter, RoutingMode, MachRoute};

let router = MachineRouter::new(&map);

// Route between two particle IDs
let route: Option<MachRoute> = router.route(from_id, to_id, RoutingMode::Pedestrian);

// Inspect result
if let Some(r) = route {
    println!("Total cost: {}", r.total_cost);
    println!("Segments: {}", r.segments.len());
    for seg in &r.segments {
        println!("  → particle {} bearing {:.0}° dist {:.1}m cost {:.2}",
                 seg.to_particle_id, seg.bearing_deg, seg.distance_m, seg.edge_cost);
    }
}
```

### RoutingMode

| Mode                       | Effect                                                     |
|----------------------------|------------------------------------------------------------|
| `Pedestrian`               | Footways + roads; vehicle-only roads blocked               |
| `Vehicle`                  | Vehicle-accessible roads; footways blocked                 |
| `Pilgrimage`               | Pedestrian + uses `ATTR_SACRED_WEIGHT` EAV for pilgrimage priority |
| `Pipeline { follow_flow }` | Pipeline particles only; optionally enforces flow direction|
| `Emergency`                | All particles passable; edge cost × 0.5                    |

A* uses integer priority: `f_cost = (g + h) × 10 000` stored as `u64` in a
min-heap (`BinaryHeap<Reverse<PqEntry>>`).  Heuristic is haversine distance
to goal.

---

## BookmarkStore — Named Sovereign Locations

```rust
use navi_engine::{BookmarkStore, BookmarkCategory};

let mut store = BookmarkStore::new();

let id = store.add("مرقد الإمام علي", coord, BookmarkCategory::Sacred, tribe, 1445);
store.set_latin(id, "Maqam al-Imam Ali");
store.set_notes(id, "Major pilgrimage site — always accessible");

// Queries
store.get(id)
store.by_category(BookmarkCategory::Grave)
store.nearest(coord)
store.nearest_in_category(coord, BookmarkCategory::Infrastructure)
store.within_radius(coord, 500.0)
store.remove(id)
```

### BookmarkCategory

`Grave` / `Infrastructure` / `Waypoint` / `Hazard` / `Sacred` / `Personal`

Every bookmark is minted a KAKI via `KakiMinter` at `add()` time.

---

## WpdPipelineMap — Pipeline Domain Layer

Sidecar to `ParticleMap` — attaches pipeline engineering metadata.

```rust
use navi_engine::{WpdPipelineMap, PipeMaterial, LeakDetectionSource, PIPELINE_QUALITY_DIVISOR};

let mut wpd = WpdPipelineMap::new();

// Register segment engineering data
wpd.register_segment(particle_id, PipelineSegmentData {
    kind:         PipelineKind::WaterSupply,
    flow:         FlowDir::DownStream,
    diameter_mm:  300,
    pressure_kpa: Some(400),
    material:     PipeMaterial::DuctileIron,
    age_years:    Some(22),
    flow_rate_ls: Some(45),
});

// Add leak candidate (severity on 0–240 scale)
wpd.add_leak(particle_id, severity, LeakDetectionSource::DroneInfraRed, now_epoch);

// Query
let critical = wpd.critical_leaks(200);       // severity ≥ 200
let deteriorated = wpd.deteriorated_segments(0.3); // life_factor < 0.3
```

### PipeMaterial — Life Factor

| Material       | Lifespan | `life_factor(25 yr)` |
|----------------|----------|----------------------|
| `Steel`        | 50 yr    | 0.50                 |
| `DuctileIron`  | 80 yr    | 0.69                 |
| `Concrete`     | 60 yr    | 0.58                 |
| `PVC`          | 40 yr    | 0.38                 |
| `Clay`         | 100 yr   | 0.75                 |
| `HDPE`         | 50 yr    | 0.50                 |

### LeakDetectionSource

`PressureDrop` / `VisualInspection` / `SoilMoisture` / `DroneInfraRed` / `AcousticSensor`

### Leak Severity Scale (QUALITY_DIVISOR = 240)

| Severity | Meaning           |
|----------|-------------------|
| 240      | Critical rupture  |
| ≥ 200    | `is_critical()`   |
| ≥ 100    | `is_significant()`|
| 60       | Minor seepage     |
| 0        | Suspected only    |

---

## Sovereign Constants

```rust
pub const HUBBLE_ZOOM_MAX:        u8  = 6;
pub const PIPELINE_QUALITY_DIVISOR: u8 = 240;  // re-export of ADR-001
```

---

## Dependencies

- `enkidb-kaki` — `Kaki`, `KakiMinter`, `KakiRole`
- `bahyway-core` — `TribeId`

---

## See Also

- `crates/najaf-engine/MANUAL.md` — NajafEngine routing over NaviEngine
- `docs/12_examples/najaf_landingzone_test_roadmap.md` — end-to-end test guide
