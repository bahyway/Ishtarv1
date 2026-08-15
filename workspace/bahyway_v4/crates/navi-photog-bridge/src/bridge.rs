//! Places a drone-photogrammetry point cloud into a `navi_engine::ParticleMap`
//! as real, routable `GraveMarker` particles, with candidate shortcut-path
//! edges between nearby markers.
//!
//! Axis convention (documented because lat/lon-vs-lon/lat ordering is a
//! classic GIS bug source): every `Point3` from `photogrammetry_ingest`
//! is read as `[X, Y, Z]` = `[easting-or-longitude, northing-or-latitude,
//! elevation]` — the standard convention for georeferenced survey point
//! clouds (matches typical LAS/PLY exports from GPS-tagged drone photos).

use std::collections::HashSet;

use bahyway_core::TribeId;
use navi_engine::{haversine_m, MapKind, NaviCoord, ParticleMap};
use photogrammetry_ingest::PointCloud;

use crate::geo_convert::{utm_to_latlon, UtmZone};

/// Which coordinate reference system a point cloud's `[X, Y, Z]` values
/// are actually in. `photogrammetry_ingest` does not read this out of a
/// file's own metadata (no VLR/CRS parsing — see `las` module's own
/// doc), so the caller must know and supply it.
#[derive(Debug, Clone, Copy)]
pub enum SourceCrs {
    /// Already WGS84: X = longitude (degrees), Y = latitude (degrees).
    Wgs84,
    /// UTM: X = easting (meters), Y = northing (meters), in the given zone.
    Utm(UtmZone),
}

impl SourceCrs {
    fn to_lat_lon(&self, x: f64, y: f64) -> (f32, f32) {
        match self {
            SourceCrs::Wgs84 => (y as f32, x as f32),
            SourceCrs::Utm(zone) => {
                let (lat, lon) = utm_to_latlon(x, y, *zone);
                (lat as f32, lon as f32)
            }
        }
    }
}

/// Parameters for the candidate shortcut-path edges built between nearby
/// markers. This is a real, honest v1 geometric-proximity heuristic — NOT
/// derived from actual path/trail imagery (identifying an actual paved
/// or worn path from pixels is a separate computer-vision problem, out
/// of scope here). It gives `MachineRouter`'s existing A* something real
/// to route over; a stakeholder can always edit/add/remove edges in
/// `ParticleMap` directly once real path surveys are available.
#[derive(Debug, Clone, Copy)]
pub struct MarkerLinking {
    /// Markers farther apart than this are never linked.
    pub max_link_distance_m: f32,
    /// Each marker links to at most this many of its nearest neighbours
    /// (within `max_link_distance_m`) — caps a dense cluster from
    /// becoming a near-complete graph.
    pub max_links_per_marker: usize,
}

impl Default for MarkerLinking {
    /// 15 m / 4 neighbours: plausible grave-to-grave footpath spacing and
    /// branching in a burial ground. Tune per real survey density.
    fn default() -> Self {
        MarkerLinking { max_link_distance_m: 15.0, max_links_per_marker: 4 }
    }
}

/// Place every point in `cloud` as a `MapKind::GraveMarker` particle in
/// `map`, then connect nearby markers with candidate shortcut-path edges
/// per `linking`. Returns the assigned particle IDs in `cloud.points`
/// order. `zoom_min` should normally be 5, matching `navi_engine::tile`'s
/// own documented convention ("Z5 Feature — individual graves").
pub fn place_point_cloud_as_grave_markers(
    map: &mut ParticleMap,
    cloud: &PointCloud,
    crs: SourceCrs,
    tribe: TribeId,
    zoom_min: u8,
    linking: &MarkerLinking,
) -> Vec<u32> {
    let mut ids = Vec::with_capacity(cloud.points.len());
    for p in &cloud.points {
        let (lat, lon) = crs.to_lat_lon(p[0], p[1]);
        let coord = NaviCoord::new(lat, lon, p[2] as f32);
        ids.push(map.place(coord, MapKind::GraveMarker, tribe, zoom_min));
    }

    let mut link_pairs: HashSet<(u32, u32)> = HashSet::new();
    for &id in &ids {
        let Some(p) = map.get(id) else { continue };
        let (lat, lon) = (p.coord.lat, p.coord.lon);
        let mut candidates: Vec<(u32, f32)> = ids
            .iter()
            .copied()
            .filter(|&other| other != id)
            .filter_map(|other| {
                map.get(other)
                    .map(|op| (other, haversine_m(lat, lon, op.coord.lat, op.coord.lon)))
            })
            .filter(|&(_, dist)| dist <= linking.max_link_distance_m)
            .collect();
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates.truncate(linking.max_links_per_marker);
        for (other, _) in candidates {
            link_pairs.insert(if id < other { (id, other) } else { (other, id) });
        }
    }

    for (a, b) in link_pairs {
        map.add_edge_auto_dist(a, b, true);
    }

    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use navi_engine::{MachineRouter, MapBounds, RoutingMode};

    fn tribe() -> TribeId {
        TribeId::from_u16(0x0001)
    }

    fn empty_najaf_map() -> ParticleMap {
        ParticleMap::new(MapBounds::new(31.960, 32.040, 44.290, 44.360), 0.001)
    }

    #[test]
    fn places_every_point_as_a_grave_marker() {
        let mut map = empty_najaf_map();
        let cloud = PointCloud {
            points: vec![
                [44.320, 31.995, 0.0],
                [44.3201, 31.9951, 0.0],
                [44.3202, 31.9952, 0.0],
            ],
            colors: None,
        };
        let ids = place_point_cloud_as_grave_markers(
            &mut map,
            &cloud,
            SourceCrs::Wgs84,
            tribe(),
            5,
            &MarkerLinking::default(),
        );
        assert_eq!(ids.len(), 3);
        assert_eq!(map.particle_count(), 3);
        for id in &ids {
            assert!(matches!(map.get(*id).unwrap().kind, MapKind::GraveMarker));
        }
    }

    #[test]
    fn wgs84_axis_convention_is_x_equals_lon_y_equals_lat() {
        let mut map = empty_najaf_map();
        let cloud = PointCloud { points: vec![[44.320, 31.995, 0.0]], colors: None };
        let ids = place_point_cloud_as_grave_markers(
            &mut map, &cloud, SourceCrs::Wgs84, tribe(), 5, &MarkerLinking::default(),
        );
        let p = map.get(ids[0]).unwrap();
        assert!((p.coord.lat - 31.995).abs() < 1e-4, "lat={}", p.coord.lat);
        assert!((p.coord.lon - 44.320).abs() < 1e-4, "lon={}", p.coord.lon);
    }

    #[test]
    fn utm_source_crs_recovers_the_original_lat_lon() {
        let (easting, northing, zone) = crate::geo_convert::latlon_to_utm(31.995, 44.320);
        let mut map = empty_najaf_map();
        let cloud = PointCloud { points: vec![[easting, northing, 0.0]], colors: None };
        let ids = place_point_cloud_as_grave_markers(
            &mut map, &cloud, SourceCrs::Utm(zone), tribe(), 5, &MarkerLinking::default(),
        );
        let p = map.get(ids[0]).unwrap();
        assert!((p.coord.lat - 31.995).abs() < 1e-4, "lat={}", p.coord.lat);
        assert!((p.coord.lon - 44.320).abs() < 1e-4, "lon={}", p.coord.lon);
    }

    #[test]
    fn markers_farther_than_max_link_distance_are_never_linked() {
        let mut map = empty_najaf_map();
        let cloud = PointCloud {
            points: vec![
                [44.320, 31.995, 0.0],
                [44.3201, 31.9951, 0.0], // ~15 m away
                [44.400, 32.030, 0.0],   // far away
            ],
            colors: None,
        };
        let ids = place_point_cloud_as_grave_markers(
            &mut map,
            &cloud,
            SourceCrs::Wgs84,
            tribe(),
            5,
            &MarkerLinking { max_link_distance_m: 50.0, max_links_per_marker: 8 },
        );
        assert!(!map.neighbours(ids[0]).is_empty(), "the two close markers must be linked");
        assert!(map.neighbours(ids[2]).is_empty(), "the far marker must not be linked to anything");
    }

    #[test]
    fn max_links_per_marker_caps_a_dense_hub() {
        // One central marker with 10 close neighbours, cap of 3.
        let mut map = empty_najaf_map();
        let mut points = vec![[44.320, 31.995, 0.0]];
        for i in 0..10 {
            points.push([44.320 + (i as f64) * 0.00002, 31.995, 0.0]);
        }
        let cloud = PointCloud { points, colors: None };
        let ids = place_point_cloud_as_grave_markers(
            &mut map,
            &cloud,
            SourceCrs::Wgs84,
            tribe(),
            5,
            &MarkerLinking { max_link_distance_m: 100.0, max_links_per_marker: 3 },
        );
        assert!(
            map.neighbours(ids[0]).len() <= 3,
            "hub marker must not exceed max_links_per_marker, got {}",
            map.neighbours(ids[0]).len()
        );
    }

    #[test]
    fn end_to_end_pilgrimage_route_between_drone_surveyed_graves() {
        // A short chain of "drone-surveyed" grave markers ~15m apart --
        // proves the real path from a point cloud all the way to a real
        // MachineRouter A* route, with no test doubles anywhere in the
        // chain (ParticleMap and MachineRouter are navi-engine's own,
        // already-tested real code).
        let mut map = empty_najaf_map();
        let mut points = Vec::new();
        for i in 0..5 {
            points.push([44.320 + (i as f64) * 0.00015, 31.995, 0.0]);
        }
        let cloud = PointCloud { points, colors: None };
        let ids = place_point_cloud_as_grave_markers(
            &mut map,
            &cloud,
            SourceCrs::Wgs84,
            tribe(),
            5,
            &MarkerLinking { max_link_distance_m: 25.0, max_links_per_marker: 2 },
        );

        let router = MachineRouter::new(&map);
        let route = router
            .route(ids[0], ids[4], &RoutingMode::Pilgrimage { prefer_sacred: false })
            .expect("a real route must be found across the drone-surveyed marker chain");
        assert!(route.is_valid());
        assert_eq!(route.start(), Some(ids[0]));
        assert_eq!(route.end(), Some(ids[4]));
        assert!(route.total_dist_m > 0.0);
    }
}
