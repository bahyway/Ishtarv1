//! Mesopotamian spatial context — the ancient cities and the
//! Najaf Sumerian context grounding the sacred ground of Wadi al-Salam.
//! Coordinates are modern WGS84 approximations of the tells.
//! Fine-grained cell assignment is GeoEngine law — this module
//! provides the HISTORICAL layer only.

/// Broad periodization of southern Mesopotamia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SumerianPeriod {
    Ubaid,
    Uruk,
    EarlyDynastic,
    Akkadian,
    UrIII,
    OldBabylonian,
}

/// An ancient Mesopotamian city with its sovereign identity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AncientCity {
    pub name_sumerian: &'static str,
    pub name_akkadian: &'static str,
    pub name_modern: &'static str,
    pub patron_deity: &'static str,
    pub period: SumerianPeriod,
    pub lat: f64,
    pub lon: f64,
}

/// Ten Phase-1 anchor cities of Sumer and Akkad.
pub const ANCIENT_CITIES: [AncientCity; 10] = [
    AncientCity { name_sumerian: "Eridug", name_akkadian: "Eridu",   name_modern: "Tell Abu Shahrain", patron_deity: "Enki",     period: SumerianPeriod::Ubaid,         lat: 30.8158, lon: 45.9960 },
    AncientCity { name_sumerian: "Urim",   name_akkadian: "Uru",     name_modern: "Tell el-Muqayyar",  patron_deity: "Nanna",    period: SumerianPeriod::UrIII,         lat: 30.9626, lon: 46.1032 },
    AncientCity { name_sumerian: "Unug",   name_akkadian: "Uruk",    name_modern: "Warka",             patron_deity: "Inanna",   period: SumerianPeriod::Uruk,          lat: 31.3241, lon: 45.6367 },
    AncientCity { name_sumerian: "Nibru",  name_akkadian: "Nippur",  name_modern: "Nuffar",            patron_deity: "Enlil",    period: SumerianPeriod::EarlyDynastic, lat: 32.1266, lon: 45.2306 },
    AncientCity { name_sumerian: "Lagash", name_akkadian: "Lagash",  name_modern: "Al-Hiba",           patron_deity: "Ningirsu", period: SumerianPeriod::EarlyDynastic, lat: 31.4183, lon: 46.4075 },
    AncientCity { name_sumerian: "Girsu",  name_akkadian: "Girsu",   name_modern: "Telloh",            patron_deity: "Ningirsu", period: SumerianPeriod::EarlyDynastic, lat: 31.5619, lon: 46.1776 },
    AncientCity { name_sumerian: "Kish",   name_akkadian: "Kishatu", name_modern: "Tell al-Uhaymir",   patron_deity: "Zababa",   period: SumerianPeriod::EarlyDynastic, lat: 32.5399, lon: 44.6089 },
    AncientCity { name_sumerian: "Larsam", name_akkadian: "Larsa",   name_modern: "Tell as-Senkereh",  patron_deity: "Utu",      period: SumerianPeriod::OldBabylonian, lat: 31.2857, lon: 45.8534 },
    AncientCity { name_sumerian: "Isin",   name_akkadian: "Isin",    name_modern: "Ishan al-Bahriyat", patron_deity: "Gula",     period: SumerianPeriod::OldBabylonian, lat: 31.8839, lon: 45.2697 },
    AncientCity { name_sumerian: "Ka-dingirra", name_akkadian: "Babilim", name_modern: "Babylon",      patron_deity: "Marduk",   period: SumerianPeriod::OldBabylonian, lat: 32.5423, lon: 44.4211 },
];

/// Great-circle distance in kilometres (pure-std haversine).
pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().asin()
}

/// The Sumerian context of the Najaf region.
/// Wadi al-Salam lies on ancient Ki (𒆠) — sacred ground within
/// the cultural orbit of Nippur, the religious center of Sumer.
#[derive(Debug, Clone, Copy)]
pub struct NajafSumerianContext {
    pub lat: f64,
    pub lon: f64,
}

impl NajafSumerianContext {
    /// Wadi al-Salam cemetery, Najaf.
    pub fn wadi_al_salam() -> Self {
        NajafSumerianContext { lat: 32.0190, lon: 44.3150 }
    }

    /// Nearest ancient city and its distance in km.
    pub fn nearest_ancient_city(&self) -> (&'static AncientCity, f64) {
        let mut best = &ANCIENT_CITIES[0];
        let mut best_km = f64::MAX;
        for c in ANCIENT_CITIES.iter() {
            let d = haversine_km(self.lat, self.lon, c.lat, c.lon);
            if d < best_km {
                best_km = d;
                best = c;
            }
        }
        (best, best_km)
    }

    /// Distance to Nippur — the religious center whose cultural
    /// territory grounds the sacred status of Wadi al-Salam.
    pub fn distance_to_nippur_km(&self) -> f64 {
        let nippur = ANCIENT_CITIES
            .iter()
            .find(|c| c.name_akkadian == "Nippur")
            .expect("Nippur is a Phase-1 anchor city");
        haversine_km(self.lat, self.lon, nippur.lat, nippur.lon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_cities_with_unique_modern_names() {
        let mut names: Vec<&str> =
            ANCIENT_CITIES.iter().map(|c| c.name_modern).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 10);
    }

    #[test]
    fn haversine_zero_and_symmetry() {
        let d0 = haversine_km(31.0, 45.0, 31.0, 45.0);
        assert!(d0.abs() < 1e-9);
        let ab = haversine_km(30.9626, 46.1032, 32.1266, 45.2306);
        let ba = haversine_km(32.1266, 45.2306, 30.9626, 46.1032);
        assert!((ab - ba).abs() < 1e-9);
    }

    #[test]
    fn najaf_sits_within_the_sumerian_heartland() {
        let ctx = NajafSumerianContext::wadi_al_salam();
        let (_city, km) = ctx.nearest_ancient_city();
        assert!(km < 100.0, "an anchor city lies within 100 km of Najaf");
        assert!(ctx.distance_to_nippur_km() < 150.0, "Nippur within 150 km");
    }

    #[test]
    fn ur_matches_the_sealed_coordinates() {
        let ur = ANCIENT_CITIES.iter().find(|c| c.name_akkadian == "Uru").unwrap();
        assert!((ur.lat - 30.9626).abs() < 1e-6);
        assert!((ur.lon - 46.1032).abs() < 1e-6);
    }
}
