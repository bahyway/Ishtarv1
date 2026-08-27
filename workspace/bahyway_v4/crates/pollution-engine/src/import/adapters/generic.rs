//! # Generic Heuristic Adapter — Unknown Source Fallback
//! **PollutionWay v3.5 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Handles any CSV or JSON payload that did not match a known source.
//! Uses field name pattern matching and value range heuristics to
//! attempt best-effort mapping to `RawSensorReading`.
//!
//! Supports **mixed-domain files**: each row is independently classified,
//! allowing a single import to produce Air + Water + Oil particles.

use super::super::{
    registry::SourceAdapter,
    router::{ImportPayload, PayloadFormat},
    ImportError,
};
use crate::sensors::RawSensorReading;
use std::collections::HashMap;

#[derive(Default)]
pub struct GenericAdapter;

impl SourceAdapter for GenericAdapter {
    fn name(&self) -> &'static str {
        "generic"
    }
    fn description(&self) -> &'static str {
        "Generic heuristic — unknown source, field-pattern fallback"
    }
    fn supported_formats(&self) -> &[PayloadFormat] {
        &[
            PayloadFormat::JsonArray,
            PayloadFormat::NdJson,
            PayloadFormat::Csv,
            PayloadFormat::Tsv,
        ]
    }

    fn normalize(
        &self,
        payload: &ImportPayload,
    ) -> (Vec<RawSensorReading>, Vec<(usize, ImportError)>) {
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();

        for (i, row) in payload.rows.iter().enumerate() {
            // Per-row domain classification
            let domain = self.classify_row(row);

            let result = match domain {
                RowDomain::Air => self.normalize_air_row(row, i),
                RowDomain::Water => self.normalize_water_row(row, i),
                RowDomain::Oil => self.normalize_oil_row(row, i),
                RowDomain::Unknown => Err(ImportError::FieldMappingFailed {
                    field: "domain".into(),
                    reason: format!("row {} has no recognisable pollution fields", i),
                }),
            };

            match result {
                Ok(r) => succeeded.push(r),
                Err(e) => failed.push((i, e)),
            }
        }

        (succeeded, failed)
    }
}

enum RowDomain {
    Air,
    Water,
    Oil,
    Unknown,
}

impl GenericAdapter {
    // ── Row domain classification ────────────────────────────

    fn classify_row(&self, row: &HashMap<String, String>) -> RowDomain {
        // Word-boundary match: pattern must appear at start/end or surrounded by '_'.
        // Prevents "ph" from matching "tphc_ug_l", "co" from matching "conductivity", etc.
        let has = |f: &str| {
            row.iter().any(|(k, v)| {
                if v.is_empty() {
                    return false;
                }
                let k = k.as_str();
                k == f
                    || k.starts_with(&format!("{}_", f))
                    || k.ends_with(&format!("_{}", f))
                    || k.contains(&format!("_{}_", f))
            })
        };

        let air_score: usize = [
            "pm2_5",
            "pm25",
            "pm10",
            "aqi",
            "no2",
            "o3",
            "so2",
            "co",
            "ozone",
            "particulate",
            "european_aqi",
            "us_aqi",
            "pollutant",
        ]
        .iter()
        .filter(|&&f| has(f))
        .count();

        let water_score: usize = [
            "turbidity",
            "tds",
            "wqi",
            "dissolvedoxygen",
            "do_mg",
            "ph",
            "bod",
            "cod",
            "conductivity",
            "nitrate",
            "phosphate",
            "salinity",
            "ntu",
        ]
        .iter()
        .filter(|&&f| has(f))
        .count();

        let oil_score: usize = [
            "tphc",
            "pah",
            "sheen",
            "spill",
            "hydrocarbon",
            "crude",
            "petroleum",
            "weathering",
            "tar",
            "dispersant",
            "api_gravity",
            "tpah",
        ]
        .iter()
        .filter(|&&f| has(f))
        .count();

        if air_score == 0 && water_score == 0 && oil_score == 0 {
            return RowDomain::Unknown;
        }

        if air_score >= water_score && air_score >= oil_score {
            RowDomain::Air
        } else if water_score >= oil_score {
            RowDomain::Water
        } else {
            RowDomain::Oil
        }
    }

    // ── Coordinate extraction (shared) ──────────────────────

    fn extract_coords(
        &self,
        row: &HashMap<String, String>,
        i: usize,
    ) -> Result<(f64, f64), ImportError> {
        let lat_keys = [
            "lat",
            "latitude",
            "y",
            "coord_lat",
            "point_lat",
            "decimallatitude",
        ];
        let lon_keys = [
            "lon",
            "longitude",
            "x",
            "coord_lon",
            "point_lon",
            "decimallongitude",
        ];

        let lat = lat_keys
            .iter()
            .find_map(|&k| row.get(k).and_then(|v| v.parse::<f64>().ok()))
            .ok_or_else(|| ImportError::FieldMappingFailed {
                field: "lat".into(),
                reason: format!("generic row {}: tried lat/latitude/y/coord_lat", i),
            })?;

        let lon = lon_keys
            .iter()
            .find_map(|&k| row.get(k).and_then(|v| v.parse::<f64>().ok()))
            .ok_or_else(|| ImportError::FieldMappingFailed {
                field: "lon".into(),
                reason: format!("generic row {}: tried lon/longitude/x/coord_lon", i),
            })?;

        Ok((lat, lon))
    }

    fn extract_sensor_id(&self, row: &HashMap<String, String>, i: usize) -> String {
        let id_keys = [
            "id",
            "sensor_id",
            "stationid",
            "monitoringsiteid",
            "locationid",
            "name",
            "station",
        ];
        id_keys
            .iter()
            .find_map(|&k| row.get(k).filter(|v| !v.is_empty()).cloned())
            .unwrap_or_else(|| format!("GEN-{}", i))
    }

    fn extract_origin_hint(&self, row: &HashMap<String, String>, domain: &str) -> String {
        let hint_keys = ["origin", "origin_hint", "source", "type"];
        let raw = hint_keys
            .iter()
            .find_map(|&k| row.get(k).cloned())
            .unwrap_or_default()
            .to_uppercase();

        if raw.contains("NATURAL") {
            return "NATURAL".into();
        }
        if raw.contains("SPILL") {
            return "SPILL".into();
        }
        if raw.contains("CHEMICAL") {
            return "CHEMICAL".into();
        }
        if raw.contains("ANTHROPOGENIC") {
            return "ANTHROPOGENIC".into();
        }

        // Domain-based default
        match domain {
            "AIR" => "ANTHROPOGENIC".into(),
            "WATER" => "CHEMICAL".into(),
            "OIL" => "SPILL".into(),
            _ => "UNKNOWN".into(),
        }
    }

    // ── Field fuzzy lookup helpers ───────────────────────────

    fn fuzzy_f64(&self, row: &HashMap<String, String>, patterns: &[&str]) -> Option<f64> {
        for pattern in patterns {
            if let Some(val) = row
                .iter()
                .find(|(k, _)| k.contains(pattern))
                .and_then(|(_, v)| v.parse::<f64>().ok())
            {
                return Some(val);
            }
        }
        None
    }

    // ── AIR row normalizer ───────────────────────────────────

    fn normalize_air_row(
        &self,
        row: &HashMap<String, String>,
        i: usize,
    ) -> Result<RawSensorReading, ImportError> {
        let (lat, lon) = self.extract_coords(row, i)?;

        let pm2_5 = self.fuzzy_f64(row, &["pm2_5", "pm25", "pm2.5", "fine_particles"]);
        let pm10 = self.fuzzy_f64(row, &["pm10", "coarse_particles"]);

        if pm2_5.is_none() && pm10.is_none() {
            return Err(ImportError::FieldMappingFailed {
                field: "pm2_5 / pm10".into(),
                reason: format!("generic AIR row {} has no PM fields", i),
            });
        }

        let aqi = self
            .fuzzy_f64(row, &["aqi", "air_quality_index", "european_aqi", "us_aqi"])
            .map(|v| v as u16);

        Ok(RawSensorReading {
            sensor_id: self.extract_sensor_id(row, i),
            domain: "AIR".into(),
            origin_hint: self.extract_origin_hint(row, "AIR"),
            lat,
            lon,
            altitude_m: self
                .fuzzy_f64(row, &["altitude", "elevation", "alt"])
                .map(|v| v as f32),
            pm2_5,
            pm10,
            no2_ppb: self.fuzzy_f64(row, &["no2", "nitrogen_dioxide"]),
            o3_ppb: self.fuzzy_f64(row, &["o3", "ozone"]),
            so2_ppb: self.fuzzy_f64(row, &["so2", "sulphur_dioxide", "sulfur_dioxide"]),
            co_ppm: self.fuzzy_f64(row, &["co_ppm", "carbon_monoxide"]),
            aqi,
            turbidity_ntu: None,
            tds_mg_l: None,
            do_mg_l: None,
            ph: None,
            bod_mg_l: None,
            wqi: None,
            tphc_ug_l: None,
            pah_ratio: None,
            spill_area_km2: None,
            sheen_detected: None,
            weathering_index: None,
        })
    }

    // ── WATER row normalizer ─────────────────────────────────

    fn normalize_water_row(
        &self,
        row: &HashMap<String, String>,
        i: usize,
    ) -> Result<RawSensorReading, ImportError> {
        let (lat, lon) = self.extract_coords(row, i)?;

        let turbidity_ntu = self.fuzzy_f64(row, &["turbidity", "ntu"]);
        let tds_mg_l = self.fuzzy_f64(row, &["tds", "total_dissolved"]);
        let do_mg_l = self.fuzzy_f64(row, &["dissolved_oxygen", "do_mg", "oxygen"]);
        let ph = self.fuzzy_f64(row, &["ph"]);

        if turbidity_ntu.is_none() && tds_mg_l.is_none() && do_mg_l.is_none() && ph.is_none() {
            return Err(ImportError::FieldMappingFailed {
                field: "water parameters".into(),
                reason: format!("generic WATER row {} has no recognisable water fields", i),
            });
        }

        let wqi = self
            .fuzzy_f64(row, &["wqi", "water_quality"])
            .map(|v| v as f32);

        Ok(RawSensorReading {
            sensor_id: self.extract_sensor_id(row, i),
            domain: "WATER".into(),
            origin_hint: self.extract_origin_hint(row, "WATER"),
            lat,
            lon,
            altitude_m: None,
            pm2_5: None,
            pm10: None,
            no2_ppb: None,
            o3_ppb: None,
            so2_ppb: None,
            co_ppm: None,
            aqi: None,
            turbidity_ntu,
            tds_mg_l: tds_mg_l.or(Some(300.0)),
            do_mg_l,
            ph,
            bod_mg_l: self.fuzzy_f64(row, &["bod", "biochemical_oxygen"]),
            wqi,
            tphc_ug_l: None,
            pah_ratio: None,
            spill_area_km2: None,
            sheen_detected: None,
            weathering_index: None,
        })
    }

    // ── OIL row normalizer ───────────────────────────────────

    fn normalize_oil_row(
        &self,
        row: &HashMap<String, String>,
        i: usize,
    ) -> Result<RawSensorReading, ImportError> {
        let (lat, lon) = self.extract_coords(row, i)?;

        let tphc_ug_l = self.fuzzy_f64(row, &["tphc", "tph", "petroleum_hydrocarbon"]);
        let pah_ratio = self.fuzzy_f64(row, &["pah", "polycyclic_aromatic"]);
        let sheen = row
            .iter()
            .find(|(k, _)| k.contains("sheen") || k.contains("visible_oil"))
            .map(|(_, v)| matches!(v.to_lowercase().trim(), "true" | "1" | "yes" | "t"))
            .or_else(|| Some(pah_ratio.map(|p| p > 0.3).unwrap_or(false)));

        if tphc_ug_l.is_none() && pah_ratio.is_none() {
            return Err(ImportError::FieldMappingFailed {
                field: "tphc_ug_l / pah_ratio".into(),
                reason: format!("generic OIL row {} has no oil-specific fields", i),
            });
        }

        Ok(RawSensorReading {
            sensor_id: self.extract_sensor_id(row, i),
            domain: "OIL".into(),
            origin_hint: self.extract_origin_hint(row, "OIL"),
            lat,
            lon,
            altitude_m: None,
            pm2_5: None,
            pm10: None,
            no2_ppb: None,
            o3_ppb: None,
            so2_ppb: None,
            co_ppm: None,
            aqi: None,
            turbidity_ntu: None,
            tds_mg_l: None,
            do_mg_l: None,
            ph: None,
            bod_mg_l: None,
            wqi: None,
            tphc_ug_l,
            pah_ratio,
            spill_area_km2: self.fuzzy_f64(row, &["spill_area", "area_km2", "coverage"]),
            sheen_detected: sheen,
            weathering_index: self
                .fuzzy_f64(row, &["weathering", "evaporation"])
                .map(|v| v as f32),
        })
    }
}
