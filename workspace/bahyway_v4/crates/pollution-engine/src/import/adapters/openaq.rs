//! # OpenAQ v3 Adapter — Air Quality
//! **PollutionWay v3.5 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Maps OpenAQ v3 API JSON responses to `RawSensorReading`.
//!
//! ## OpenAQ v3 Field Map
//! ```text
//! OpenAQ field                  → RawSensorReading field
//! ─────────────────────────────   ──────────────────────────
//! locationId / location_id      → sensor_id
//! coordinates_latitude          → lat
//! coordinates_longitude         → lon
//! parameter / pollutant         → (selects field: pm2_5 / pm10 / no2 / ...)
//! value                         → (value for selected field)
//! date_utc / date_local         → (recorded_at — metadata only)
//! country                       → (origin_hint context)
//! ```
//!
//! OpenAQ returns one measurement per row (long format),
//! so this adapter groups by locationId and pivots to wide format.

use super::super::{
    registry::SourceAdapter,
    router::{ImportPayload, PayloadFormat},
    ImportError,
};
use crate::sensors::RawSensorReading;
use std::collections::HashMap;

#[derive(Default)]
pub struct OpenAqAdapter;

impl SourceAdapter for OpenAqAdapter {
    fn name(&self) -> &'static str {
        "openaq"
    }
    fn description(&self) -> &'static str {
        "OpenAQ v3 — Global air quality measurements (JSON)"
    }
    fn supported_formats(&self) -> &[PayloadFormat] {
        &[PayloadFormat::JsonArray, PayloadFormat::NdJson]
    }

    fn normalize(
        &self,
        payload: &ImportPayload,
    ) -> (Vec<RawSensorReading>, Vec<(usize, ImportError)>) {
        // OpenAQ v3 is long-format: one row per measurement parameter.
        // Strategy: pivot on (locationId, date) → wide row per location.
        let mut pivoted: HashMap<String, HashMap<String, String>> = HashMap::new();

        for (i, row) in payload.rows.iter().enumerate() {
            // Station / sensor identity
            let location_id = row
                .get("locationid")
                .or_else(|| row.get("location_id"))
                .or_else(|| row.get("sensorid"))
                .or_else(|| row.get("id"))
                .cloned()
                .unwrap_or_else(|| format!("OAQ-{}", i));

            let key = format!(
                "{}-{}",
                location_id,
                row.get("date_utc")
                    .or_else(|| row.get("date_local"))
                    .cloned()
                    .unwrap_or_default()
            );

            let entry = pivoted.entry(key).or_default();

            // Always copy geo + metadata into the pivot group
            if let Some(lat) = row.get("coordinates_latitude").or_else(|| row.get("lat")) {
                entry.insert("lat".into(), lat.clone());
            }
            if let Some(lon) = row.get("coordinates_longitude").or_else(|| row.get("lon")) {
                entry.insert("lon".into(), lon.clone());
            }
            entry
                .entry("sensor_id".into())
                .or_insert_with(|| location_id.clone());
            entry
                .entry("origin_hint".into())
                .or_insert_with(|| "ANTHROPOGENIC".into());
            entry.insert("domain".into(), "AIR".into());

            // Map parameter name → our field name
            let param = row
                .get("parameter")
                .or_else(|| row.get("pollutant"))
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            let value = row
                .get("value")
                .or_else(|| row.get("avg"))
                .cloned()
                .unwrap_or_default();

            match param.as_str() {
                "pm25" | "pm2.5" | "pm2_5" => {
                    entry.insert("pm2_5".into(), value);
                }
                "pm10" => {
                    entry.insert("pm10".into(), value);
                }
                "no2" => {
                    entry.insert("no2_ppb".into(), value);
                }
                "o3" | "ozone" => {
                    entry.insert("o3_ppb".into(), value);
                }
                "so2" => {
                    entry.insert("so2_ppb".into(), value);
                }
                "co" => {
                    entry.insert("co_ppm".into(), value);
                }
                "um003" | "um010" => {} // particle counts — skip
                _ => {
                    // Unknown param — store as-is for generic fallback
                    entry.insert(param, value);
                }
            }
        }

        // Convert pivoted wide-rows to RawSensorReading
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

impl OpenAqAdapter {
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
                reason: format!("missing in row {}", i),
            })?;
        let lon = row
            .get("lon")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| ImportError::FieldMappingFailed {
                field: "lon".into(),
                reason: format!("missing in row {}", i),
            })?;

        let pm2_5 = row.get("pm2_5").and_then(|v| v.parse().ok());
        let pm10 = row.get("pm10").and_then(|v| v.parse().ok());

        // Require at least PM2.5 or PM10
        if pm2_5.is_none() && pm10.is_none() {
            return Err(ImportError::FieldMappingFailed {
                field: "pm2_5 / pm10".into(),
                reason: "at least one PM field required".into(),
            });
        }

        Ok(RawSensorReading {
            sensor_id: row
                .get("sensor_id")
                .cloned()
                .unwrap_or_else(|| format!("OAQ-{}", i)),
            domain: "AIR".into(),
            origin_hint: row
                .get("origin_hint")
                .cloned()
                .unwrap_or_else(|| "ANTHROPOGENIC".into()),
            lat,
            lon,
            altitude_m: None,
            pm2_5,
            pm10,
            no2_ppb: row.get("no2_ppb").and_then(|v| v.parse().ok()),
            o3_ppb: row.get("o3_ppb").and_then(|v| v.parse().ok()),
            so2_ppb: row.get("so2_ppb").and_then(|v| v.parse().ok()),
            co_ppm: row.get("co_ppm").and_then(|v| v.parse().ok()),
            aqi: row.get("aqi").and_then(|v| v.parse().ok()),
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
}
