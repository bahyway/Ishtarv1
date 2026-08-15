# GraveDiscoveryStation — Developer Manual

> **DubSar Help** | `Manuals > GraveDiscoveryStation` | Crate Reference

## Overview

`grave-discovery-station` is the AI imagery ingestion pipeline for Wadi al-Salam.
It parses tab-separated sidecar files produced by satellite or drone analysis,
applies them to an existing `GraveRegistry`, and returns a `GraveDiscoveryReport`
with statistics on condition, identity resolution, and parse errors.

No external crates. No floats in confidence arithmetic. Pure Rust, offline.

---

## TSV Sidecar Format

Files are UTF-8 tab-separated, one row per discovered grave.

```
grave_id  lat_center  lon_center  condition  inscription_text  confidence_pct  death_hijri_year
101       31.9950     44.3200     Partial    عبد الله الحسيني  62              1398
102       32.0050     44.3200     Destroyed                    0
103       32.0030     44.3330     Intact     محمد علي الكاظمي  100             1419
```

| Field              | Type   | Required | Notes                                     |
|--------------------|--------|----------|-------------------------------------------|
| `grave_id`         | u32    | Yes      | Must match existing `GraveParticle.id`    |
| `lat_center`       | f32    | Yes      | WGS-84 latitude                           |
| `lon_center`       | f32    | Yes      | WGS-84 longitude                          |
| `condition`        | string | Yes      | `Intact` \| `Partial` \| `Destroyed` \| `Unknown` |
| `inscription_text` | UTF-8  | No       | Arabic headstone text — may be empty      |
| `confidence_pct`   | u8     | Yes      | 0–100; scaled internally to 0–240         |
| `death_hijri_year` | u32    | No       | Hijri year — may be empty                 |

Lines beginning with `#` or starting with `grave_id` (header) are skipped.

---

## Confidence Scaling

`confidence_pct` (0–100) is mapped to the sovereign `QUALITY_DIVISOR = 240` scale:

```
scaled = (confidence_pct × 240) / 100
```

| Input % | Scaled (0–240) | Meaning                      |
|---------|----------------|------------------------------|
| 100     | 240            | AI fully confirmed            |
| 62      | 148            | Probable                      |
| 50      | 120            | Significant                   |
| 0       | 0              | Unidentified                  |

**Confidence is never lowered.** A subsequent scan reporting 0% will not
overwrite a particle that already holds a higher score.

---

## GraveDiscoveryStation — API

```rust
use grave_discovery_station::GraveDiscoveryStation;
use najaf_engine::{DiscoverySource, GraveRegistry, NajafSector};

// Create station for a drone-flight batch
let mut station = GraveDiscoveryStation::new(DiscoverySource::Drone);

// Optionally seed inscription matcher from known graves
station.seed_matcher_from_registry(&registry);

// Or register a specific known name
station.register_name(101, "عبد الله الحسيني");

// Process raw TSV text
let report = station.process_csv(csv_text, &mut registry, tribe, NajafSector::Shuhadaa);

// Or process pre-parsed rows (for programmatic use)
let report = station.process_rows(&rows, &mut registry, tribe, NajafSector::Shuhadaa);
```

One `GraveDiscoveryStation` corresponds to one discovery source (one drone
flight, one satellite pass, one archive digitisation batch).

---

## GraveDiscoveryReport

```rust
pub struct GraveDiscoveryReport {
    pub total_rows:       usize,
    pub parse_errors:     usize,
    pub graves_updated:   usize,
    pub graves_not_found: usize,
    pub identity_matched: usize,
    pub damaged_count:    usize,      // Partial + Destroyed
    pub destroyed_count:  usize,
    pub partial_count:    usize,
    pub intact_count:     usize,
    pub candidates:       Vec<IdentityCandidate>,
}
```

- `parse_errors` — rows that could not be parsed; processing continues
- `graves_updated` — rows applied to the registry (update or new registration)
- `candidates` — identity candidates with confidence ≥ 150 (from `InscriptionMatcher`)
- `unresolved_count()` — helper: `total_rows − parse_errors − graves_not_found`

---

## DiscoveryRow — Programmatic API

```rust
use grave_discovery_station::DiscoveryRow;
use najaf_engine::GraveCondition;

let rows = vec![
    DiscoveryRow {
        grave_id:         201,
        lat_center:       31.991,
        lon_center:       44.317,
        condition:        GraveCondition::Partial,
        inscription_text: "سيد حسن الموسوي".into(),
        confidence_pct:   75,
        death_hijri_year: Some(1402),
    },
];
```

---

## Behaviour Rules

1. **New grave**: if `grave_id` is not in the registry, a new `GraveParticle`
   is created with `DiscoverySource` from the station and registered.
2. **Existing grave**: `condition`, `discovery_source`, and `death_hijri_year`
   are updated; `identity_confidence` is raised only, never lowered.
3. **InscriptionMatcher**: if `inscription_text` is non-empty and the matcher
   holds any entries, matching runs and the best candidate (confidence ≥ 150)
   is applied to `owner_name_arabic` only if it was previously `None`.
4. **Parse errors**: a malformed row increments `parse_errors` and is skipped;
   subsequent rows are still processed.

---

## Dependencies

- `najaf-engine` — `GraveParticle`, `GraveRegistry`, `InscriptionMatcher`, `QUALITY_DIVISOR`
- `bahyway-core` — `TribeId`

---

## See Also

- `crates/najaf-engine/MANUAL.md` — GraveParticle, GraveCondition, DiscoverySource, InscriptionMatcher
- `docs/12_examples/najaf_landingzone_test_roadmap.md` — end-to-end LandingZone ZIP test guide
