//! # Open-Meteo Air Quality Adapter
//! **PollutionWay v3.5 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Maps Open-Meteo Air Quality API JSON responses to `RawSensorReading`.
//!
//! ## Open-Meteo Response Shape
//! ```json
//! {
//!   "latitude": 52.37, "longitude": 4.89,
//!   "hourly": {
//!     "time": ["2024-01-01T00:00", ...],
//!     "pm10": [12.3, ...],
//!     "pm2_5": [5.1, ...],
//!     "nitrogen_dioxide": [18.2, ...],
//!     "ozone": [42.1, ...],
//!     "european_aqi": [23, ...]
//!   }
//! }
//! ```
//! After `ImportRouter` flattening, rows appear as:
//! `{ "hourly_time_0": "2024-01-01T00:00", "hourly_pm10_0": "12.3", ... }`
//!
//! This adapter reconstructs the time-series from the flattened map.

use super::super::{
    registry::SourceAdapter,
    router::{ImportPayload, PayloadFormat},
    ImportError,
};
use crate::sensors::RawSensorReading;
use std::collections::HashMap;

#[derive(Default)]
pub struct OpenMeteoAdapter;

impl SourceAdapter for OpenMeteoAdapter {
    fn name(&self) -> &'static str {
        "open_meteo"
    }
    fn description(&self) -> &'static str {
        "Open-Meteo AQ — Hourly air quality forecast/history (JSON)"
    }
    fn supported_formats(&self) -> &[PayloadFormat] {
        &[
            PayloadFormat::JsonArray,
            PayloadFormat::NdJson,
            PayloadFormat::Csv,
        ]
    }

    fn normalize(
        &self,
        payload: &ImportPayload,
    ) -> (Vec<RawSensorReading>, Vec<(usize, ImportError)>) {
        // Open-Meteo has one row per JSON object (one location).
        // Each row has flattened arrays: "hourly_pm10" = "12.3,14.1,..."
        // OR after router flattening: "hourly_pm10_0", "hourly_pm10_1", ...
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();

        for (loc_idx, row) in payload.rows.iter().enumerate() {
            let lat = ImportPayload::parse_f64(row, "latitude")
                .or_else(|| ImportPayload::parse_f64(row, "lat"));
            let lon = ImportPayload::parse_f64(row, "longitude")
                .or_else(|| ImportPayload::parse_f64(row, "lon"));

            let (lat, lon) = match (lat, lon) {
                (Some(la), Some(lo)) => (la, lo),
                _ => {
                    failed.push((
                        loc_idx,
                        ImportError::FieldMappingFailed {
                            field: "latitude/longitude".into(),
                            reason: format!("Open-Meteo row {}", loc_idx),
                        },
                    ));
                    continue;
                }
            };

            // Detect if arrays are comma-joined strings or indexed keys
            let pm10_raw = row
                .get("hourly_pm10")
                .cloned()
                .unwrap_or_else(|| self.collect_indexed(row, "hourly_pm10_"));

            let pm25_raw = row
                .get("hourly_pm2_5")
                .cloned()
                .or_else(|| row.get("hourly_pm25").cloned())
                .unwrap_or_else(|| self.collect_indexed(row, "hourly_pm2_5_"));

            let no2_raw = row
                .get("hourly_nitrogen_dioxide")
                .cloned()
                .unwrap_or_else(|| self.collect_indexed(row, "hourly_nitrogen_dioxide_"));

            let aqi_raw = row
                .get("hourly_european_aqi")
                .cloned()
                .or_else(|| row.get("hourly_us_aqi").cloned())
                .unwrap_or_else(|| self.collect_indexed(row, "hourly_european_aqi_"));

            let times_raw = row
                .get("hourly_time")
                .cloned()
                .unwrap_or_else(|| self.collect_indexed(row, "hourly_time_"));

            let pm10_vals = self.parse_array(&pm10_raw);
            let pm25_vals = self.parse_array(&pm25_raw);
            let no2_vals = self.parse_array(&no2_raw);
            let aqi_vals: Vec<Option<f64>> = self.parse_array(&aqi_raw);
            let time_strs = self.split_array(&times_raw);

            let n = pm10_vals.len().max(pm25_vals.len());
            if n == 0 {
                failed.push((
                    loc_idx,
                    ImportError::FieldMappingFailed {
                        field: "hourly_pm10 / hourly_pm2_5".into(),
                        reason: "no time-series data found".into(),
                    },
                ));
                continue;
            }

            for t in 0..n {
                let pm2_5 = pm25_vals.get(t).and_then(|v| *v);
                let pm10 = pm10_vals.get(t).and_then(|v| *v);
                let no2 = no2_vals.get(t).and_then(|v| *v);
                let aqi = aqi_vals.get(t).and_then(|v| *v).map(|v| v as u16);
                let _time = time_strs.get(t).cloned().unwrap_or_default();

                if pm2_5.is_none() && pm10.is_none() {
                    continue;
                }

                succeeded.push(RawSensorReading {
                    sensor_id: format!("METEO-{}-T{}", loc_idx, t),
                    domain: "AIR".into(),
                    origin_hint: "ANTHROPOGENIC".into(),
                    lat,
                    lon,
                    altitude_m: None,
                    pm2_5,
                    pm10,
                    no2_ppb: no2,
                    o3_ppb: None,
                    so2_ppb: None,
                    co_ppm: None,
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
                });
            }
        }

        (succeeded, failed)
    }
}

impl OpenMeteoAdapter {
    /// Collect indexed keys like "prefix_0", "prefix_1" → comma-joined string
    fn collect_indexed(&self, row: &HashMap<String, String>, prefix: &str) -> String {
        let mut vals = Vec::new();
        let mut i = 0usize;
        loop {
            let key = format!("{}{}", prefix, i);
            if let Some(v) = row.get(&key) {
                vals.push(v.clone());
                i += 1;
            } else {
                break;
            }
        }
        vals.join(",")
    }

    fn parse_array(&self, raw: &str) -> Vec<Option<f64>> {
        raw.split(',')
            .map(|s| {
                let t = s.trim();
                if t.is_empty() || t == "null" {
                    None
                } else {
                    t.parse().ok()
                }
            })
            .collect()
    }

    fn split_array(&self, raw: &str) -> Vec<String> {
        raw.split(',').map(|s| s.trim().to_string()).collect()
    }
}
