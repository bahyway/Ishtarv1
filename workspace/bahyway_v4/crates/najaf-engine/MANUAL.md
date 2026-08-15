# NajafEngine — Developer Manual

> **DubSar Help** | `Manuals > NajafEngine` | Crate Reference

## Overview

`najaf-engine` is the sovereign cemetery navigation engine for Wadi al-Salam, Najaf, Iraq.
It provides KAKI-native pilgrim routing, grave registration, AI-powered discovery
from satellite and drone imagery, and simplicial-complex topology for spatial reasoning.

Coordinates: `WADI_AL_SALAM_LAT = 31.995°N, WADI_AL_SALAM_LON = 44.320°E`

---

## Module Map

```
najaf-engine/src/
  sector.rs    — NajafSector: 7 cemetery zones with Arabic names and sacred weights
  grave.rs     — GraveParticle: sovereign burial site + discovery extension types
  identity.rs  — InscriptionMatcher: FNV-1a Arabic text matching, confidence scoring
  search.rs    — GraveRegistry: in-memory sovereign collection of GraveParticles
  guide.rs     — PilgrimGuide: routes pilgrims from the entrance to a grave
  topology.rs  — SimplicialComplex: 7D barycentric topology, ghost reconstruction
  seed.rs      — seven_grave_registry(): canonical 7-grave test fixture
  error.rs     — NajafError / NajafResult
```

---

## NajafSector — 7 Cemetery Zones

| Sector       | Index | Arabic          | Sacred Weight | Description                    |
|--------------|-------|-----------------|---------------|--------------------------------|
| `Entrance`   | 0     | المدخل          | 1.00          | Main gate — all pilgrims pass here |
| `Shuhadaa`   | 1     | الشهداء         | 0.85          | Martyrs section — elevated priority |
| `Awliya`     | 2     | الأولياء        | 0.90          | Saints section                 |
| `Huffaz`     | 3     | الحفّاظ         | 0.95          | Memorisers of Quran            |
| `Momineen`   | 4     | المؤمنين        | 1.00          | Believers — general section    |
| `Ulamaa`     | 5     | العلماء         | 0.92          | Scholars                       |
| `Anbiya`     | 6     | الأنبياء        | 0.88          | Prophets — most revered zone   |

Sacred weight feeds into `intrinsic_cost()` — lower values mean higher routing priority in
`Pilgrimage` mode.

---

## GraveParticle — Core Type

```rust
pub struct GraveParticle {
    pub id:                  GraveId,        // u32
    pub coord:               NaviCoord,      // lat/lon/alt WGS-84
    pub sector:              NajafSector,
    pub tribe:               TribeId,
    pub epoch:               u32,            // Hijri year of interment
    pub state:               GraveState,     // Occupied/Reserved/Available/Sealed
    pub kaki:                Kaki,           // immutable sovereign identity
    // Discovery fields (populated by AI pipeline):
    pub condition:           GraveCondition, // Intact/Partial/Destroyed/Unknown
    pub identity_confidence: u8,             // 0–240 (QUALITY_DIVISOR=240)
    pub civil_registry_kaki: Option<u32>,    // linked civil registry particle
    pub image_tile_kaki:     Option<u32>,    // linked satellite/drone tile particle
    pub discovery_source:    DiscoverySource,
    pub owner_name_arabic:   Option<String>,
    pub death_hijri_year:    Option<u32>,
}
```

### Construction

```rust
// Minimal — suitable for known registered graves
let grave = GraveParticle::new(id, coord, NajafSector::Shuhadaa, tribe, 1400);

// Full discovery chain (builder pattern)
let grave = GraveParticle::new(id, coord, NajafSector::Awliya, tribe, 1380)
    .with_condition(GraveCondition::Partial)
    .with_confidence(150)                         // 150/240 = probable
    .with_discovery_source(DiscoverySource::Drone)
    .with_image_tile(0x1A2B_3C4D)
    .with_owner_name("عبد الله الحسيني")
    .with_death_year(1378);
```

### GraveCondition

| Value       | Meaning                                      | `is_damaged()` |
|-------------|----------------------------------------------|----------------|
| `Intact`    | Structure fully visible — headstone upright  | false          |
| `Partial`   | Headstone collapsed or surround eroded       | true           |
| `Destroyed` | Only ground disturbance remains              | true           |
| `Unknown`   | Not yet assessed (default on new())          | false          |

### DiscoverySource

| Value           | Description                             |
|-----------------|-----------------------------------------|
| `Manual`        | Physical field inspection (default)     |
| `Satellite`     | AI analysis of satellite imagery        |
| `Drone`         | Drone overhead / oblique photography    |
| `Historical`    | Archive maps and historical records     |
| `CivilRegistry` | Cross-reference with government records |

### Identity Confidence Scale (QUALITY_DIVISOR = 240)

| Score | Meaning              | How to reach it                          |
|-------|----------------------|------------------------------------------|
| 240   | Confirmed            | Civil registry cross-reference exact     |
| 200   | High confidence      | All inscription words match (reordered)  |
| 150   | Probable             | First name (given name) prefix match     |
| 60    | Speculative          | Partial similarity, no definitive match  |
| 0     | Unidentified         | Default — no identity information        |

---

## GraveRegistry — Search API

```rust
let mut registry = GraveRegistry::new();
registry.register(grave);

// Basic queries (pre-existing)
registry.get(101)                          // → Option<&GraveParticle>
registry.by_sector(NajafSector::Shuhadaa) // → Vec<&GraveParticle>
registry.by_tribe(tribe)
registry.by_epoch_range(1400, 1450)
registry.accessible()
registry.nearest(&coord)                   // haversine distance
registry.nearest_accessible(&coord)

// Discovery queries (new)
registry.by_condition(GraveCondition::Destroyed)
registry.unidentified(60)                  // confidence < 60 → speculative or worse
registry.damaged()                         // Partial or Destroyed
registry.damaged_and_unidentified(150)     // damaged AND confidence < 150
```

---

## InscriptionMatcher — Identity Resolution

Matches Arabic headstone inscriptions against a registry of known names using
FNV-1a hashing of normalized text.

```rust
let mut matcher = InscriptionMatcher::new();

// Seed from civil registry or known graves
matcher.register(101, "عبد الله الحسيني");
matcher.register(102, "محمد علي الكاظمي");

// Match a field-captured inscription (may have whitespace noise)
let candidates = matcher.match_inscription("  عبد الله   الحسيني  ");
// → [IdentityCandidate { grave_id: 101, confidence: 240, matched_name: "..." }]
```

### Match confidence levels

| Confidence | Condition                                     |
|------------|-----------------------------------------------|
| 240        | Exact normalized text hash match              |
| 200        | All words present, order-independent          |
| 150        | First word (given name) matches               |
| (absent)   | No meaningful similarity                      |

Results are sorted descending by confidence.  Only candidates ≥ 1 are returned.

---

## PilgrimGuide — Routing

```rust
let graph = cemetery_nav_graph();
let registry = seven_grave_registry();
let guide = PilgrimGuide::new(&graph, &registry);

// Route from entrance (node 1) to a specific grave
let route = guide.guide_to_grave(1, 107)?;  // → PilgrimRoute
assert!(route.is_valid());
println!("Sector: {:?}, Cost: {}", route.sector, route.total_cost);
```

Routes use the NaviCode A* pipeline from `navi-engine`.  Graves with
`state = Sealed` or `Available` return `NajafError::GraveNotAccessible`.

---

## SimplicialComplex — 7D Topology

Used by the AI discovery pipeline to map partially-destroyed graves into existing
topological structures (i.e., identify which "gap" in the spatial complex a
newly-found grave belongs to).

```rust
// Build complex from known grave coordinates
let vertices: Vec<[f64; 7]> = graves.iter()
    .map(|g| [g.coord.lat as f64, g.coord.lon as f64, g.coord.alt as f64,
              g.epoch as f64, g.sector.index() as f64,
              g.tribe.as_u16() as f64, g.identity_confidence as f64])
    .collect();
let complex = SimplicialComplex { vertices, dimension: 7 };

// Test if a newly-discovered grave fits in the complex
let point = [31.992, 44.325, 0.0, 1380.0, 1.0, 1.0, 0.0];
let inside = is_particle_in_complex(&complex, &point);

// Reconstruct ghost grave coordinates from its neighbours
let ghost = reconstruct_ghost(&complex, vec![101, 102, 103]);

// Infer pipeline alleyways from known paths
let path = infer_pipeline(&complex, start_vertex, end_vertex);
```

---

## Sovereign Constants

```rust
pub const WADI_AL_SALAM_LAT: f32 = 31.995;
pub const WADI_AL_SALAM_LON: f32 = 44.320;
pub const NAJAF_SECTORS:     usize = 7;
pub const QUALITY_DIVISOR:   u8    = 240;   // ADR-001 — never 255
```

---

## Dependencies

- `navi-engine` — `NaviCoord`, `NaviGraph`, `haversine_m`, routing
- `enkidb-kaki` — `Kaki`, `KakiMinter`, `KakiRole::Zikru`
- `bahyway-core` — `TribeId`

---

## See Also

- `grave-discovery-station/MANUAL.md` — AI batch CSV processing
- `docs/12_examples/najaf_landingzone_test_roadmap.md` — end-to-end test guide
- `docs/02_identity/identity_kaki.md` — KAKI sovereignty model
