//! # NOAA IncidentNews Adapter — Oil Spill Data
//! **PollutionWay v3.5 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Maps NOAA IncidentNews CSV (public domain) to `RawSensorReading`.
//!
//! ## NOAA IncidentNews Field Map
//! ```text
//! open_date       → recorded_at context
//! name            → sensor_id (incident name)
//! lat / lon       → lat / lon
//! threat          → origin_hint (Oil → SPILL)
//! material_spilled→ (free text — used for PAH estimation)
//! max_ptl_release → tphc_ug_l proxy (gallons → µg/L conversion)
//! sheen_present   → sheen_detected
//! ```

use super::super::{
    registry::SourceAdapter,
    router::{ImportPayload, PayloadFormat},
    ImportError,
};
use crate::sensors::RawSensorReading;
use std::collections::HashMap;

#[derive(Default)]
pub struct NoaaIncidentAdapter;

impl SourceAdapter for NoaaIncidentAdapter {
    fn name(&self)        -> &'static str { "noaa_incident" }
    fn description(&self) -> &'static str { "NOAA IncidentNews — Oil & chemical spills (CSV, public domain)" }
    fn supported_formats(&self) -> &[PayloadFormat] {
        &[PayloadFormat::Csv, PayloadFormat::Tsv]
    }

    fn normalize(
        &self,
        payload: &ImportPayload,
    ) -> (Vec<RawSensorReading>, Vec<(usize, ImportError)>) {
        let mut succeeded = Vec::new();
        let mut failed    = Vec::new();

        for (i, row) in payload.rows.iter().enumerate() {
            // Only process Oil threat rows
            let threat = row.get("threat").map(|s| s.to_lowercase()).unwrap_or_default();
            if !threat.is_empty() && threat != "oil" && !threat.contains("oil") {
                continue; // Skip chemical/other incidents
            }

            match self.row_to_reading(row, i) {
                Ok(r)  => succeeded.push(r),
                Err(e) => failed.push((i, e)),
            }
        }

        (succeeded, failed)
    }
}

impl NoaaIncidentAdapter {
    fn row_to_reading(
        &self,
        row: &HashMap<String, String>,
        i: usize,
    ) -> Result<RawSensorReading, ImportError> {
        let lat = row.get("lat")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| ImportError::FieldMappingFailed {
                field: "lat".into(),
                reason: format!("NOAA row {}", i),
            })?;
        let lon = row.get("lon")
            .and_then(|v| v.parse::<f64>().ok())
            .ok_or_else(|| ImportError::FieldMappingFailed {
                field: "lon".into(),
                reason: format!("NOAA row {}", i),
            })?;

        // Convert max_ptl_release (gallons) to µg/L proxy
        // 1 US gallon crude oil ≈ 3,785 mL × ~850 mg/mL density ≈ 3.2M µg
        // TPHC µg/L = gallons × 3,218,750 (rough field-level proxy)
        let tphc_ug_l = row.get("max_ptl_release")
            .and_then(|v| v.parse::<f64>().ok())
            .map(|gal| gal * 3_218_750.0)
            .or(Some(500.0)); // default: trace level

        // PAH ratio heuristic from material_spilled text
        let material = row.get("material_spilled")
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        let pah_ratio = Self::estimate_pah_from_material(&material);

        // Sheen detection from countermeasure tags
        let tags = row.get("tags").map(|s| s.to_lowercase()).unwrap_or_default();
        let sheen_detected = tags.contains("sheen") || material.contains("crude");

        // Origin
        let origin_hint = if material.contains("chemical") {
            "CHEMICAL"
        } else {
            "SPILL"
        };

        let sensor_id = row.get("name")
            .cloned()
            .unwrap_or_else(|| format!("NOAA-INC-{}", i));

        Ok(RawSensorReading {
            sensor_id,
            domain:           "OIL".into(),
            origin_hint:      origin_hint.into(),
            lat, lon,
            altitude_m:       None,
            pm2_5:            None,
            pm10:             None,
            no2_ppb:          None,
            o3_ppb:           None,
            so2_ppb:          None,
            co_ppm:           None,
            aqi:              None,
            turbidity_ntu:    None,
            tds_mg_l:         None,
            do_mg_l:          None,
            ph:               None,
            bod_mg_l:         None,
            wqi:              None,
            tphc_ug_l,
            pah_ratio:        Some(pah_ratio),
            spill_area_km2:   row.get("area").and_then(|v| v.parse().ok()),
            sheen_detected:   Some(sheen_detected),
            weathering_index: Some(0.1), // fresh spill assumption
        })
    }

    /// Estimate PAH ratio from material description text
    fn estimate_pah_from_material(material: &str) -> f64 {
        if material.contains("crude") || material.contains("heavy") {
            0.45
        } else if material.contains("diesel") || material.contains("fuel") {
            0.30
        } else if material.contains("gasoline") || material.contains("refined") {
            0.20
        } else if material.contains("bunker") {
            0.55
        } else {
            0.25 // unknown — moderate estimate
        }
    }
}
