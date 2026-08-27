// ── Repair navigation ─────────────────────────────────────────────

use crate::defect::{DefectClass, DefectEvent};
use crate::error::{WpdError, WpdResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiteAccess {
    Open,
    FootOnly,
    Restricted,
    Dangerous,
}

impl SiteAccess {
    /// Travel cost multiplier relative to Open access.
    pub fn cost_multiplier(self) -> f32 {
        match self {
            SiteAccess::Open => 1.0,
            SiteAccess::FootOnly => 1.8,
            SiteAccess::Restricted => 2.5,
            SiteAccess::Dangerous => 4.0,
        }
    }

    pub fn is_safe(self) -> bool {
        !matches!(self, SiteAccess::Dangerous)
    }

    pub fn name(self) -> &'static str {
        match self {
            SiteAccess::Open => "Open",
            SiteAccess::FootOnly => "Foot Only",
            SiteAccess::Restricted => "Restricted",
            SiteAccess::Dangerous => "Dangerous",
        }
    }
}

// ── Route model ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RepairWaypoint {
    pub lat: f64,
    pub lon: f64,
    pub landmark: String,
    pub access: SiteAccess,
}

#[derive(Debug, Clone)]
pub struct RepairRoute {
    pub segment_id: String,
    pub waypoints: Vec<RepairWaypoint>,
    pub distance_metres: u32,
    pub estimated_minutes: u32,
    pub requires_escort: bool,
    pub blueprint_missing: bool,
    pub directions: Vec<String>,
}

// ── Navigator preferences ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RepairPrefs {
    pub avoid_dangerous: bool,
    pub max_distance_m: Option<u32>,
}

impl Default for RepairPrefs {
    fn default() -> Self {
        Self {
            avoid_dangerous: true,
            max_distance_m: None,
        }
    }
}

// ── Navigator ────────────────────────────────────────────────────

pub struct RepairNavigator;

impl RepairNavigator {
    pub fn route_to(defect: &DefectEvent, prefs: &RepairPrefs) -> WpdResult<RepairRoute> {
        let access = Self::estimate_access(defect);
        if prefs.avoid_dangerous && matches!(access, SiteAccess::Dangerous) {
            return Err(WpdError::RouteNotFound(format!(
                "segment {} classified Dangerous — set avoid_dangerous=false to override",
                defect.segment_id
            )));
        }
        // A placeholder staging waypoint; real coordinates come from drone dispatch.
        let waypoints = vec![
            RepairWaypoint {
                lat: 33.3400,
                lon: 44.3900,
                landmark: "Operations Base Baghdad".to_string(),
                access: SiteAccess::Open,
            },
            RepairWaypoint {
                lat: 33.3400,
                lon: 44.3900,
                landmark: format!("Segment {}", defect.segment_id),
                access,
            },
        ];
        let distance_metres = 1_000u32;
        if let Some(max_d) = prefs.max_distance_m {
            if distance_metres > max_d {
                return Err(WpdError::RouteNotFound(format!(
                    "route {distance_metres}m exceeds max allowed {max_d}m"
                )));
            }
        }
        // speed in m/min adjusted by access cost
        let speed_mpm = 70.0 / access.cost_multiplier();
        let estimated_minutes = ((distance_metres as f32 / speed_mpm) as u32).max(1);
        let requires_escort = matches!(access, SiteAccess::Restricted | SiteAccess::Dangerous);
        Ok(RepairRoute {
            segment_id: defect.segment_id.clone(),
            waypoints,
            distance_metres,
            estimated_minutes,
            requires_escort,
            blueprint_missing: false,
            directions: vec![
                "Proceed to Operations Base Baghdad".to_string(),
                format!(
                    "Navigate to segment {} for inspection and repair",
                    defect.segment_id
                ),
            ],
        })
    }

    fn estimate_access(defect: &DefectEvent) -> SiteAccess {
        if matches!(
            defect.defect_class,
            DefectClass::ActiveLeak | DefectClass::Crack
        ) && defect.composite_score > 0.80
        {
            SiteAccess::Dangerous
        } else if defect.composite_score > 0.70 {
            SiteAccess::Restricted
        } else if defect.confidence < 0.40 {
            SiteAccess::FootOnly
        } else {
            SiteAccess::Open
        }
    }
}

/// Haversine distance in metres between two WGS-84 coordinates.
pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> u32 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    (6_371_000.0 * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defect::{DefectClassifier, DetectionMethod};

    fn make_event(score: f32, confidence: f32) -> DefectEvent {
        DefectClassifier::classify(
            "BGH-SC-001",
            score,
            confidence,
            1_717_000_000_000,
            DetectionMethod::SpectralScan,
        )
        .unwrap()
    }

    #[test]
    fn safe_defect_produces_route() {
        let ev = make_event(0.50, 0.80);
        let prefs = RepairPrefs::default();
        let route = RepairNavigator::route_to(&ev, &prefs).unwrap();
        assert_eq!(route.segment_id, "BGH-SC-001");
        assert!(!route.waypoints.is_empty());
    }

    #[test]
    fn dangerous_defect_blocked_by_default() {
        let ev = make_event(0.95, 0.90);
        let prefs = RepairPrefs::default();
        assert!(RepairNavigator::route_to(&ev, &prefs).is_err());
    }

    #[test]
    fn dangerous_defect_allowed_when_override() {
        let ev = make_event(0.95, 0.90);
        let prefs = RepairPrefs {
            avoid_dangerous: false,
            max_distance_m: None,
        };
        assert!(RepairNavigator::route_to(&ev, &prefs).is_ok());
    }

    #[test]
    fn max_distance_enforced() {
        let ev = make_event(0.30, 0.70);
        let prefs = RepairPrefs {
            avoid_dangerous: true,
            max_distance_m: Some(100),
        };
        assert!(RepairNavigator::route_to(&ev, &prefs).is_err());
    }

    #[test]
    fn haversine_same_point_is_zero() {
        assert_eq!(haversine_m(33.36, 44.44, 33.36, 44.44), 0);
    }

    #[test]
    fn haversine_one_degree_lat_approx_111km() {
        let d = haversine_m(0.0, 0.0, 1.0, 0.0);
        assert!(d > 110_000 && d < 112_000, "haversine 1° lat = {d}m");
    }
}
