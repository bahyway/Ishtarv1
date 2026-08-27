//! # EEA Waterbase Adapter — Water Quality
//! **PollutionWay v3.5 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Maps EEA Waterbase ICM CSV exports to `RawSensorReading`.
//!
//! ## EEA Waterbase v2021 Field Map
//! ```text
//! EEA field                      → RawSensorReading field
//! ──────────────────────────────   ────────────────────────
//! monitoringsiteid               → sensor_id
//! lat / lon                      → lat / lon
//! observedproperty               → (selects turbidity/ph/do/tds...)
//! resultobserved                 → (value for selected property)
//! resultqualitybadge             → (skip if "Bad")
//! phenomenontime*                → (timestamp context)
//! ```
//!
//! Waterbase is also long-format (one row per parameter),
//! so the same pivot strategy is applied as OpenAQ.

use super::super::{
    registry::SourceAdapter,
    router::{ImportPayload, PayloadFormat},
    ImportError,
};
use crate::sensors::RawSensorReading;
use std::collections::HashMap;

/// Known EEA Waterbase parameter URIs → our field names
const EEA_PARAM_MAP: &[(&str, &str)] = &[
    // Turbidity
    ("turbidity", "turbidity_ntu"),
    ("EEA_3169", "turbidity_ntu"),
    // Dissolved Oxygen
    ("dissolvedoxygen", "do_mg_l"),
    ("dissolved_o2", "do_mg_l"),
    ("EEA_54", "do_mg_l"),
    // pH
    ("ph", "ph"),
    ("EEA_1", "ph"),
    // BOD
    ("biochemicaloxygendemand", "bod_mg_l"),
    ("bod", "bod_mg_l"),
    ("EEA_EEA_BOD", "bod_mg_l"),
    // TDS / Conductivity proxy
    ("totaldissolvedsolids", "tds_mg_l"),
    ("conductivity", "tds_mg_l"), // unit-converted below
    ("EEA_conductivity", "tds_mg_l"),
    // Nitrate, Phosphate (stored as raw extras — WQI still computed)
    ("nitrate", "nitrate_mg_l"),
    ("phosphate", "phosphate_mg_l"),
    // Salinity
    ("salinity", "salinity_ppt"),
];

#[derive(Default)]
pub struct EeaWaterbaseAdapter;

impl SourceAdapter for EeaWaterbaseAdapter {
    fn name(&self) -> &'static str {
        "eea_waterbase"
    }
    fn description(&self) -> &'static str {
        "EEA Waterbase ICM — European water quality (CSV)"
    }
    fn supported_formats(&self) -> &[PayloadFormat] {
        &[PayloadFormat::Csv, PayloadFormat::Tsv]
    }

    fn normalize(
        &self,
        payload: &ImportPayload,
    ) -> (Vec<RawSensorReading>, Vec<(usize, ImportError)>) {
        // Pivot long-format rows on monitoringsiteid + phenomenontime
        let mut pivoted: HashMap<String, HashMap<String, String>> = HashMap::new();

        for (i, row) in payload.rows.iter().enumerate() {
            // Skip rows with bad quality flag
            if let Some(quality) = row.get("resultqualitybadge") {
                if quality.to_lowercase().contains("bad") {
                    continue;
                }
            }

            let site_id = row
                .get("monitoringsiteid")
                .or_else(|| row.get("stationid"))
                .or_else(|| row.get("id"))
                .cloned()
                .unwrap_or_else(|| format!("EEA-{}", i));

            let time_key = row
                .get("phenomenontimestart")
                .or_else(|| row.get("phenomenontime"))
                .or_else(|| row.get("date"))
                .cloned()
                .unwrap_or_default();

            let key = format!("{}-{}", site_id, time_key);
            let entry = pivoted.entry(key).or_default();

            // Geo coordinates
            for lat_field in &["lat", "latitude", "point_lat"] {
                if let Some(v) = row.get(*lat_field) {
                    entry.entry("lat".into()).or_insert_with(|| v.clone());
                }
            }
            for lon_field in &["lon", "longitude", "point_lon"] {
                if let Some(v) = row.get(*lon_field) {
                    entry.entry("lon".into()).or_insert_with(|| v.clone());
                }
            }
            entry
                .entry("sensor_id".into())
                .or_insert_with(|| site_id.clone());
            entry.insert("domain".into(), "WATER".into());
            entry.insert("origin_hint".into(), "CHEMICAL".into());

            // Map observed property → our field
            let prop = row
                .get("observedproperty")
                .or_else(|| row.get("parametercategory"))
                .or_else(|| row.get("parameter"))
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            let value = row
                .get("resultobserved")
                .or_else(|| row.get("value"))
                .or_else(|| row.get("result"))
                .cloned()
                .unwrap_or_default();

            // Match against known EEA parameter names
            let mut matched = false;
            for (pattern, target) in EEA_PARAM_MAP {
                if prop.contains(&pattern.to_lowercase()) || prop.contains(pattern) {
                    // Special: conductivity → TDS via 0.64 conversion factor
                    let stored_value = if *target == "tds_mg_l" && prop.contains("conductivity") {
                        value
                            .parse::<f64>()
                            .map(|v| (v * 0.64).to_string())
                            .unwrap_or(value.clone())
                    } else {
                        value.clone()
                    };
                    entry.insert(target.to_string(), stored_value);
                    matched = true;
                    break;
                }
            }

            if !matched && !value.is_empty() {
                // Store unknown parameter as-is for generic WQI fallback
                entry.insert(format!("extra_{}", prop.replace(' ', "_")), value);
            }
        }

        // Convert to RawSensorReading
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();

        for (i, (_key, row)) in pivoted.into_iter().enumerate() {
            match self.row_to_reading(&row, i) {
                Ok(r) => succeeded.push(r),
                Err(e) => failed.push((i, e)),
            }
        }

        (succeeded, failed)
    }
}

impl EeaWaterbaseAdapter {
    fn row_to_reading(
        &self,
        row: &HashMap<String, String>,
        i: usize,
    ) -> Result<RawSensorReading, ImportError> {
        let lat = row
            .get("lat")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| ImportError::FieldMappingFailed {
                field: "lat".into(),
                reason: format!("missing in EEA row {}", i),
            })?;
        let lon = row
            .get("lon")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| ImportError::FieldMappingFailed {
                field: "lon".into(),
                reason: format!("missing in EEA row {}", i),
            })?;

        let turbidity_ntu = row.get("turbidity_ntu").and_then(|v| v.parse().ok());
        let tds_mg_l = row.get("tds_mg_l").and_then(|v| v.parse().ok());

        // Need at least one water parameter
        if turbidity_ntu.is_none()
            && tds_mg_l.is_none()
            && row.get("do_mg_l").is_none()
            && row.get("ph").is_none()
        {
            return Err(ImportError::FieldMappingFailed {
                field: "water parameters".into(),
                reason: "no recognised water quality fields found in row".into(),
            });
        }

        // Compute a simple WQI proxy from available fields
        let wqi = self.compute_wqi_proxy(row);

        Ok(RawSensorReading {
            sensor_id: row
                .get("sensor_id")
                .cloned()
                .unwrap_or_else(|| format!("EEA-{}", i)),
            domain: "WATER".into(),
            origin_hint: "CHEMICAL".into(),
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
            tds_mg_l: tds_mg_l.or(Some(300.0)), // default safe value
            do_mg_l: row.get("do_mg_l").and_then(|v| v.parse().ok()),
            ph: row.get("ph").and_then(|v| v.parse().ok()),
            bod_mg_l: row.get("bod_mg_l").and_then(|v| v.parse().ok()),
            wqi: Some(wqi),
            tphc_ug_l: None,
            pah_ratio: None,
            spill_area_km2: None,
            sheen_detected: None,
            weathering_index: None,
        })
    }

    /// Simple WQI proxy from available EEA fields [0–100, higher = cleaner]
    fn compute_wqi_proxy(&self, row: &HashMap<String, String>) -> f32 {
        let mut score = 100.0f32;

        if let Some(turb) = row.get("turbidity_ntu").and_then(|v| v.parse::<f32>().ok()) {
            score -= match turb {
                t if t <= 1.0 => 0.0,
                t if t <= 5.0 => 10.0,
                t if t <= 25.0 => 25.0,
                _ => 45.0,
            };
        }
        if let Some(do_val) = row.get("do_mg_l").and_then(|v| v.parse::<f32>().ok()) {
            score -= match do_val {
                d if d >= 8.0 => 0.0,
                d if d >= 5.0 => 10.0,
                d if d >= 2.0 => 25.0,
                _ => 40.0,
            };
        }
        if let Some(ph) = row.get("ph").and_then(|v| v.parse::<f32>().ok()) {
            let dev = (ph - 7.0).abs();
            score -= (dev * 10.0).min(20.0);
        }
        if let Some(bod) = row.get("bod_mg_l").and_then(|v| v.parse::<f32>().ok()) {
            score -= match bod {
                b if b <= 2.0 => 0.0,
                b if b <= 5.0 => 10.0,
                _ => 25.0,
            };
        }

        score.clamp(0.0, 100.0)
    }
}
