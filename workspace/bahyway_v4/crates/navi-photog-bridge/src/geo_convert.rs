//! WGS84 <-> UTM (Universal Transverse Mercator) conversion.
//!
//! Needed because drone-photogrammetry point clouds are very commonly
//! georeferenced in UTM meters (the typical SfM-engine convention for
//! GPS-tagged photo input, e.g. OpenDroneMap's default georeferenced
//! output), while `navi-engine`'s `NaviCoord`/`ParticleMap` operate in
//! WGS84 lat/lon degrees. Nothing in this workspace did this conversion
//! before this crate (confirmed: no `UTM`/`utm_to`/`easting` hits
//! anywhere in `crates/`).
//!
//! HONEST SCOPE: this implements the standard 6th-order Redfearn/Snyder
//! series (the same formula family used by most open UTM libraries),
//! accurate to a few centimeters within a UTM zone — far more precise
//! than grave-marker-scale routing needs. It is verified here via
//! forward-then-inverse round-trip on multiple reference points (several
//! latitudes, both hemispheres) recovering the original coordinate to
//! within 1e-6 degrees (~0.1 m). It has NOT been independently cross-
//! checked against a third-party UTM computation or survey-grade
//! reference (no network reference lookup was available in the session
//! that built this) — treat it as accurate for routing/visualization,
//! and spot-check one real coordinate against a trusted external tool
//! before relying on it for legal or survey-grade positioning.
//!
//! Also out of scope: reading a coordinate reference system out of a
//! point cloud file's own metadata (e.g. a LAS file's CRS-defining VLR).
//! `photogrammetry_ingest::las` does not parse VLRs at all. Callers must
//! know and supply which CRS their point cloud is actually in.

const WGS84_A: f64 = 6_378_137.0; // semi-major axis, meters
const WGS84_F: f64 = 1.0 / 298.257_223_563; // flattening
const K0: f64 = 0.9996; // UTM scale factor

fn ellipsoid_params() -> (f64, f64) {
    // Returns (e2, e_prime2): eccentricity squared and second eccentricity squared.
    let e2 = WGS84_F * (2.0 - WGS84_F);
    let e_prime2 = e2 / (1.0 - e2);
    (e2, e_prime2)
}

/// UTM zone number [1, 60] for a given longitude (degrees).
pub fn zone_number_for_lon(lon_deg: f64) -> u8 {
    (((lon_deg + 180.0) / 6.0).floor() as i32 + 1).clamp(1, 60) as u8
}

/// Central meridian (degrees) of a UTM zone.
fn central_meridian_deg(zone_number: u8) -> f64 {
    (zone_number as f64) * 6.0 - 183.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtmZone {
    pub zone_number: u8,
    pub northern_hemisphere: bool,
}

/// Forward projection: WGS84 lat/lon (degrees) -> UTM easting/northing (meters).
/// The zone is auto-selected from `lon_deg` (the standard UTM convention).
pub fn latlon_to_utm(lat_deg: f64, lon_deg: f64) -> (f64, f64, UtmZone) {
    let zone_number = zone_number_for_lon(lon_deg);
    let zone = UtmZone {
        zone_number,
        northern_hemisphere: lat_deg >= 0.0,
    };
    let (e, n) = latlon_to_utm_zone(lat_deg, lon_deg, zone);
    (e, n, zone)
}

/// Forward projection into a SPECIFIC zone (useful when a point cloud's
/// points must all share one zone even if a few straddle a boundary).
pub fn latlon_to_utm_zone(lat_deg: f64, lon_deg: f64, zone: UtmZone) -> (f64, f64) {
    let (e2, e_prime2) = ellipsoid_params();
    let phi = lat_deg.to_radians();
    let lambda = lon_deg.to_radians();
    let lambda0 = central_meridian_deg(zone.zone_number).to_radians();

    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let tan_phi = phi.tan();

    let n_rad = WGS84_A / (1.0 - e2 * sin_phi * sin_phi).sqrt();
    let t = tan_phi * tan_phi;
    let c = e_prime2 * cos_phi * cos_phi;
    let a = (lambda - lambda0) * cos_phi;

    let e4 = e2 * e2;
    let e6 = e4 * e2;
    let m = WGS84_A
        * ((1.0 - e2 / 4.0 - 3.0 * e4 / 64.0 - 5.0 * e6 / 256.0) * phi
            - (3.0 * e2 / 8.0 + 3.0 * e4 / 32.0 + 45.0 * e6 / 1024.0) * (2.0 * phi).sin()
            + (15.0 * e4 / 256.0 + 45.0 * e6 / 1024.0) * (4.0 * phi).sin()
            - (35.0 * e6 / 3072.0) * (6.0 * phi).sin());

    let easting = K0
        * n_rad
        * (a + (1.0 - t + c) * a.powi(3) / 6.0
            + (5.0 - 18.0 * t + t * t + 72.0 * c - 58.0 * e_prime2) * a.powi(5) / 120.0)
        + 500_000.0;

    let mut northing = K0
        * (m + n_rad
            * tan_phi
            * (a * a / 2.0
                + (5.0 - t + 9.0 * c + 4.0 * c * c) * a.powi(4) / 24.0
                + (61.0 - 58.0 * t + t * t + 600.0 * c - 330.0 * e_prime2) * a.powi(6) / 720.0));

    if !zone.northern_hemisphere {
        northing += 10_000_000.0;
    }

    (easting, northing)
}

/// Inverse projection: UTM easting/northing (meters) in `zone` -> WGS84 lat/lon (degrees).
pub fn utm_to_latlon(easting: f64, northing: f64, zone: UtmZone) -> (f64, f64) {
    let (e2, e_prime2) = ellipsoid_params();
    let e1 = (1.0 - (1.0 - e2).sqrt()) / (1.0 + (1.0 - e2).sqrt());

    let x = easting - 500_000.0;
    let y = if zone.northern_hemisphere {
        northing
    } else {
        northing - 10_000_000.0
    };

    let m = y / K0;
    let e4 = e2 * e2;
    let e6 = e4 * e2;
    let mu = m / (WGS84_A * (1.0 - e2 / 4.0 - 3.0 * e4 / 64.0 - 5.0 * e6 / 256.0));

    let e1_2 = e1 * e1;
    let e1_3 = e1_2 * e1;
    let e1_4 = e1_3 * e1;
    let phi1 = mu
        + (3.0 * e1 / 2.0 - 27.0 * e1_3 / 32.0) * (2.0 * mu).sin()
        + (21.0 * e1_2 / 16.0 - 55.0 * e1_4 / 32.0) * (4.0 * mu).sin()
        + (151.0 * e1_3 / 96.0) * (6.0 * mu).sin()
        + (1097.0 * e1_4 / 512.0) * (8.0 * mu).sin();

    let sin_phi1 = phi1.sin();
    let cos_phi1 = phi1.cos();
    let tan_phi1 = phi1.tan();

    let n1 = WGS84_A / (1.0 - e2 * sin_phi1 * sin_phi1).sqrt();
    let t1 = tan_phi1 * tan_phi1;
    let c1 = e_prime2 * cos_phi1 * cos_phi1;
    let r1 = WGS84_A * (1.0 - e2) / (1.0 - e2 * sin_phi1 * sin_phi1).powf(1.5);
    let d = x / (n1 * K0);

    let lat = phi1
        - (n1 * tan_phi1 / r1)
            * (d * d / 2.0
                - (5.0 + 3.0 * t1 + 10.0 * c1 - 4.0 * c1 * c1 - 9.0 * e_prime2) * d.powi(4) / 24.0
                + (61.0 + 90.0 * t1 + 298.0 * c1 + 45.0 * t1 * t1
                    - 252.0 * e_prime2
                    - 3.0 * c1 * c1)
                    * d.powi(6)
                    / 720.0);

    let lambda0 = central_meridian_deg(zone.zone_number).to_radians();
    let lon = lambda0
        + (d - (1.0 + 2.0 * t1 + c1) * d.powi(3) / 6.0
            + (5.0 - 2.0 * c1 + 28.0 * t1 - 3.0 * c1 * c1 + 8.0 * e_prime2 + 24.0 * t1 * t1)
                * d.powi(5)
                / 120.0)
            / cos_phi1;

    (lat.to_degrees(), lon.to_degrees())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_round_trips(lat: f64, lon: f64) {
        let (e, n, zone) = latlon_to_utm(lat, lon);
        let (lat2, lon2) = utm_to_latlon(e, n, zone);
        assert!(
            (lat - lat2).abs() < 1e-6,
            "lat round-trip failed for ({lat},{lon}): got {lat2} (zone {zone:?}, E={e} N={n})"
        );
        assert!(
            (lon - lon2).abs() < 1e-6,
            "lon round-trip failed for ({lat},{lon}): got {lon2} (zone {zone:?}, E={e} N={n})"
        );
    }

    #[test]
    fn round_trips_najaf_wadi_al_salam() {
        // The reference coordinate this whole ecosystem's cemetery-domain
        // tests already use (navi-engine::particlemap::najaf_map, bookmark.rs).
        assert_round_trips(31.995, 44.320);
    }

    #[test]
    fn round_trips_near_equator() {
        assert_round_trips(0.5, 30.0);
    }

    #[test]
    fn round_trips_southern_hemisphere() {
        assert_round_trips(-33.87, 151.21); // Sydney-like coordinate
    }

    #[test]
    fn round_trips_high_northern_latitude() {
        assert_round_trips(60.17, 24.94); // Helsinki-like coordinate
    }

    #[test]
    fn round_trips_near_a_zone_boundary() {
        assert_round_trips(31.995, 42.0); // near a UTM zone edge
        assert_round_trips(31.995, 48.0);
    }

    #[test]
    fn zone_number_matches_known_bands() {
        assert_eq!(zone_number_for_lon(44.320), 38); // Najaf's real UTM zone
        assert_eq!(zone_number_for_lon(-180.0), 1);
        assert_eq!(zone_number_for_lon(179.999), 60);
    }

    #[test]
    fn utm_coordinates_are_in_the_expected_range() {
        // Easting near the central meridian must be close to the 500km
        // false-easting origin; northing must be positive (northern
        // hemisphere, no false-northing offset needed).
        let (e, n, zone) = latlon_to_utm(31.995, 44.320);
        assert!(zone.northern_hemisphere);
        assert!(
            (e - 500_000.0).abs() < 400_000.0,
            "easting {e} implausible for a point near its zone's central meridian"
        );
        assert!(n > 0.0 && n < 10_000_000.0);
    }
}
