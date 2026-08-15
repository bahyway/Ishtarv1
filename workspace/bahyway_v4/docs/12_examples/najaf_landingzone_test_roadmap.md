# NajafEngine — LandingZone Test Roadmap

> **DubSar Help** | `Docs > 12 Examples > NajafEngine LandingZone` | Test Guide

## Overview

This guide walks through end-to-end testing of the NajafEngine discovery
pipeline using the LandingZone ZIP drop mechanism.  Each scenario below
corresponds to a specific integration path through:

```
LandingZone ZIP → bee-watchdog → GraveDiscoveryStation → GraveRegistry → InscriptionMatcher → Report
```

---

## 1. LandingZone Folder Structure

Create the following directory tree before running tests:

```
workspace/bahyway_v4/
  landing_zone/
    incoming/          ← drop ZIP files here
    processed/         ← bee-watchdog moves ZIPs after processing
    failed/            ← ZIPs that failed to parse are archived here
    reports/           ← GraveDiscoveryReport JSON output
```

### ZIP File Naming Convention

```
<source>_<batch_id>_<date>.zip
```

Examples:
- `drone_b001_1446-03-15.zip`
- `satellite_b002_1446-04-01.zip`
- `historical_b003_1446-04-10.zip`

### ZIP Contents

Each ZIP contains exactly one TSV sidecar file named `discovery.tsv`:

```
drone_b001_1446-03-15.zip
  └── discovery.tsv
```

---

## 2. TSV Sidecar — Sample Files

### Scenario A — Drone Flight (partial and destroyed graves)

`discovery.tsv` (place inside a ZIP named `drone_b001_test.zip`):

```tsv
grave_id	lat_center	lon_center	condition	inscription_text	confidence_pct	death_hijri_year
101	31.9950	44.3200	Partial	عبد الله الحسيني	62	1398
102	32.0050	44.3200	Destroyed		0	
103	32.0030	44.3330	Intact	محمد علي الكاظمي	100	1419
104	31.9980	44.3150	Partial	سيد حسن الموسوي	45	
105	31.9920	44.3280	Destroyed		0	
```

**Expected report:**
- `total_rows = 5`
- `parse_errors = 0`
- `graves_updated = 5`
- `intact_count = 1`, `partial_count = 2`, `destroyed_count = 2`
- `damaged_count = 4`

---

### Scenario B — Satellite Pass (new graves not in registry)

`discovery.tsv` (place inside `satellite_b002_test.zip`):

```tsv
grave_id	lat_center	lon_center	condition	inscription_text	confidence_pct	death_hijri_year
201	31.9910	44.3190	Unknown		0	
202	31.9915	44.3195	Partial		30	1420
203	31.9920	44.3200	Destroyed		0	
```

**Expected behaviour:**
- All three IDs (201, 202, 203) are absent from the registry → created as new particles
- `discovery_source = Satellite` applied to all
- `graves_updated = 3`, `graves_not_found = 0`

---

### Scenario C — Historical Archive (high confidence inscription matching)

`discovery.tsv` (place inside `historical_b003_test.zip`):

```tsv
grave_id	lat_center	lon_center	condition	inscription_text	confidence_pct	death_hijri_year
101	31.9950	44.3200	Intact	عبد الله الحسيني	100	1398
106	31.9960	44.3210	Intact	فاطمة الزهراء الهاشمي	85	1401
```

**Setup before running:**
1. Register `grave_id=101` in the registry with `owner_name_arabic = None`.
2. Register the name `101 → "عبد الله الحسيني"` in the `InscriptionMatcher` via
   `station.register_name(101, "عبد الله الحسيني")`.

**Expected behaviour:**
- Grave 101: `identity_confidence = 240` (exact InscriptionMatcher match), `condition = Intact`
- Grave 106: registered as new particle, `owner_name_arabic = "فاطمة الزهراء الهاشمي"` (raw from CSV)
- `identity_matched ≥ 1`, `candidates` contains at least one entry with `confidence = 240`

---

### Scenario D — Confidence Never Lowered

`discovery.tsv` (place inside `drone_b004_test.zip`):

```tsv
grave_id	lat_center	lon_center	condition	inscription_text	confidence_pct	death_hijri_year
101	31.9950	44.3200	Partial		0	
```

**Setup:** Set `registry.get_mut(101).identity_confidence = 200` before processing.

**Expected behaviour:**
- After processing, `identity_confidence` remains `200` (not lowered to `0`)
- `condition` is updated to `Partial`
- `discovery_source` is updated to Drone

---

### Scenario E — Parse Error Recovery

`discovery.tsv` (place inside `drone_b005_test.zip`):

```tsv
grave_id	lat_center	lon_center	condition	inscription_text	confidence_pct	death_hijri_year
BAD_ID	31.995	44.320	Partial		50	
101	31.9950	44.3200	Intact		80	1398
```

**Expected behaviour:**
- Line 1: parse error (`grave_id: BAD_ID`) — `parse_errors = 1`
- Line 2: processed normally — `graves_updated = 1`
- Processing does not abort on error

---

### Scenario F — Multi-Source Pipeline Chain

Run three ZIPs in sequence against the same registry:

1. `satellite_b010_test.zip` — initial AI scan (low confidence)
2. `drone_b011_test.zip` — drone follow-up (raises confidence)
3. `historical_b012_test.zip` — archive cross-reference (confirms identities)

```tsv
# satellite_b010_test — discovery.tsv
101	31.9950	44.3200	Partial		40	
102	32.0050	44.3200	Destroyed		20	

# drone_b011_test — discovery.tsv
101	31.9950	44.3200	Partial	عبد الله الحسيني	75	1398

# historical_b012_test — discovery.tsv
101	31.9950	44.3200	Intact	عبد الله الحسيني	100	1398
```

**Expected final state for grave 101:**
- `condition = Intact` (most recent)
- `identity_confidence = 240` (100% → 240, raised by archive)
- `discovery_source = Historical` (most recent source)
- `owner_name_arabic = Some("عبد الله الحسيني")`

---

## 3. Rust Test Code Skeleton

```rust
use grave_discovery_station::GraveDiscoveryStation;
use najaf_engine::{
    GraveParticle, GraveRegistry, NajafSector,
    NaviCoord, DiscoverySource, GraveCondition,
};
use bahyway_core::TribeId;

fn tribe() -> TribeId { TribeId::from_u16(0x0001) }

fn base_registry() -> GraveRegistry {
    let mut r = GraveRegistry::new();
    r.register(GraveParticle::new(101, NaviCoord::new(31.995, 44.320, 0.0),
                                  NajafSector::Entrance, tribe(), 1400));
    r.register(GraveParticle::new(102, NaviCoord::new(32.005, 44.320, 0.0),
                                  NajafSector::Shuhadaa, tribe(), 1410));
    r.register(GraveParticle::new(103, NaviCoord::new(32.003, 44.333, 0.0),
                                  NajafSector::Awliya, tribe(), 1420));
    r
}

#[test]
fn scenario_a_drone_flight() {
    let mut reg = base_registry();
    let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);
    let tsv = include_str!("scenarios/drone_b001_test.tsv");
    let report = station.process_csv(tsv, &mut reg, tribe(), NajafSector::Entrance);

    assert_eq!(report.total_rows,     5);
    assert_eq!(report.parse_errors,   0);
    assert_eq!(report.graves_updated, 5);
    assert_eq!(report.intact_count,   1);
    assert_eq!(report.partial_count,  2);
    assert_eq!(report.destroyed_count,2);
    assert_eq!(report.damaged_count,  4);
}

#[test]
fn scenario_d_confidence_never_lowered() {
    let mut reg = base_registry();
    reg.get_mut(101).unwrap().identity_confidence = 200;

    let tsv = "101\t31.9950\t44.3200\tPartial\t\t0\t\n";
    let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);
    station.process_csv(tsv, &mut reg, tribe(), NajafSector::Entrance);

    assert_eq!(reg.get(101).unwrap().identity_confidence, 200);
    assert_eq!(reg.get(101).unwrap().condition, GraveCondition::Partial);
}
```

---

## 4. Checking Registry State After Processing

```rust
// Damaged and unidentified graves — priority restoration targets
let urgent = registry.damaged_and_unidentified(150);
println!("Priority targets: {}", urgent.len());

// All destroyed graves
let destroyed = registry.by_condition(GraveCondition::Destroyed);
println!("Destroyed: {}", destroyed.len());

// Unidentified (speculative or worse)
let unknown = registry.unidentified(60);
println!("Speculative/unidentified: {}", unknown.len());

// Identity candidates from last batch
for candidate in &report.candidates {
    println!("Grave {} → '{}' (confidence {})",
             candidate.grave_id, candidate.matched_name, candidate.confidence);
}
```

---

## 5. Pilgrim Routing After Discovery

After updating the registry, test that `PilgrimGuide` can still route:

```rust
use najaf_engine::{PilgrimGuide, cemetery_nav_graph, seven_grave_registry};

let graph    = cemetery_nav_graph();
let guide    = PilgrimGuide::new(&graph, &registry);

let route = guide.guide_to_grave(1, 103).expect("route to grave 103");
assert!(route.is_valid());
println!("Sector: {:?}, Cost: {}", route.sector, route.total_cost);
```

Graves with `state = Sealed` or `Available` return `NajafError::GraveNotAccessible`.

---

## 6. Topology Sanity Check

After a batch update, verify the simplicial complex still covers new graves:

```rust
use najaf_engine::{SimplicialComplex, is_particle_in_complex};

let vertices: Vec<[f64; 7]> = registry.all_ids()
    .filter_map(|id| registry.get(id))
    .map(|g| [
        g.coord.lat as f64, g.coord.lon as f64, g.coord.alt as f64,
        g.epoch as f64, g.sector.index() as f64,
        g.tribe.as_u16() as f64, g.identity_confidence as f64,
    ])
    .collect();

let complex = SimplicialComplex { vertices, dimension: 7 };

// Newly discovered grave at (31.991, 44.317) should fit in the complex
let point = [31.991, 44.317, 0.0, 1420.0, 1.0, 1.0, 0.0];
assert!(is_particle_in_complex(&complex, &point));
```

---

## 7. Test Checklist

- [ ] Scenario A: drone flight updates 5 graves — condition counts correct
- [ ] Scenario B: 3 new graves registered from satellite pass
- [ ] Scenario C: inscription matching resolves `confidence = 240` for grave 101
- [ ] Scenario D: confidence not lowered from 200 to 0 on re-scan
- [ ] Scenario E: parse error counted, processing continues, valid row applied
- [ ] Scenario F: three-pass pipeline chain — final state correct for grave 101
- [ ] Pilgrim routing still works after registry update
- [ ] `damaged_and_unidentified(150)` returns correct priority target list
- [ ] Simplicial complex covers new graves after batch

---

## See Also

- `crates/najaf-engine/MANUAL.md` — GraveParticle, GraveRegistry, InscriptionMatcher, PilgrimGuide
- `crates/grave-discovery-station/MANUAL.md` — TSV format, GraveDiscoveryStation, GraveDiscoveryReport
- `docs/02_identity/identity_kaki.md` — KAKI sovereignty model
