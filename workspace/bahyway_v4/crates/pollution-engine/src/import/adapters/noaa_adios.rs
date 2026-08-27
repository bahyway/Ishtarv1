//! # NOAA ADIOS Oil Database Adapter — Chemical Fingerprint
//! **PollutionWay v3.5 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Maps NOAA ADIOS v2+ JSON records to `RawSensorReading`.
//! ADIOS records describe oil *types* (not spill events), so this adapter
//! is primarily used to enrich PAH ratio, weathering, and TPHC estimates.
//!
//! ## ADIOS Field Map
//! ```text
//! adios_oil_id      → sensor_id
//! location          → (geo fallback to 0,0 — ADIOS has no spill coords)
//! physical_properties.api_gravity → (used for PAH estimation)
//! sara.saturates / .aromatics     → pah_ratio (aromatics / total)
//! weathering        → weathering_index
//! viscosity         → (informational)
//! ```

use super::super::{
    registry::SourceAdapter,
    router::{ImportPayload, PayloadFormat},
    ImportError,
};
use crate::sensors::RawSensorReading;
use std::collections::HashMap;

#[derive(Default)]
pub struct NoaaAdiosAdapter;

impl SourceAdapter for NoaaAdiosAdapter {
    fn name(&self) -> &'static str {
        "noaa_adios"
    }
    fn description(&self) -> &'static str {
        "NOAA ADIOS Oil DB — Chemical fingerprint data (JSON)"
    }
    fn supported_formats(&self) -> &[PayloadFormat] {
        &[PayloadFormat::JsonArray, PayloadFormat::NdJson]
    }

    fn normalize(
        &self,
        payload: &ImportPayload,
    ) -> (Vec<RawSensorReading>, Vec<(usize, ImportError)>) {
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();

        for (i, row) in payload.rows.iter().enumerate() {
            match self.row_to_reading(row, i) {
                Ok(r) => succeeded.push(r),
                Err(e) => failed.push((i, e)),
            }
        }

        (succeeded, failed)
    }
}

impl NoaaAdiosAdapter {
    fn row_to_reading(
        &self,
        row: &HashMap<String, String>,
        i: usize,
    ) -> Result<RawSensorReading, ImportError> {
        // ADIOS has no spill coordinates — use 0.0 as placeholder
        // In production, the caller should enrich with incident coords
        let lat = row
            .get("lat")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0f64);
        let lon = row
            .get("lon")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0f64);

        // PAH ratio from SARA fractions: aromatics / (saturates + aromatics + resins + asphaltenes)
        let aromatics = row
            .get("sara_aromatics")
            .or_else(|| row.get("sara_fraction_aromatics"))
            .and_then(|v| v.parse::<f64>().ok());
        let saturates = row
            .get("sara_saturates")
            .or_else(|| row.get("sara_fraction_saturates"))
            .and_then(|v| v.parse::<f64>().ok());
        let resins = row.get("sara_resins").and_then(|v| v.parse::<f64>().ok());
        let asphaltenes = row
            .get("sara_asphaltenes")
            .and_then(|v| v.parse::<f64>().ok());

        let pah_ratio = match (aromatics, saturates) {
            (Some(a), Some(s)) => {
                let total = s + a + resins.unwrap_or(0.0) + asphaltenes.unwrap_or(0.0);
                if total > 0.0 {
                    Some((a / total).clamp(0.0, 1.0))
                } else {
                    None
                }
            }
            (Some(a), None) => Some((a / 100.0).clamp(0.0, 1.0)),
            _ => None,
        };

        // Weathering index: 0 = fresh, 1 = fully weathered
        let weathering_index = row
            .get("weathering")
            .and_then(|v| v.parse::<f32>().ok())
            .map(|w| (w / 100.0).clamp(0.0, 1.0));

        // TPHC proxy from API gravity: lower API = heavier = higher TPHC
        let api_gravity = row
            .get("physical_properties_api_gravity")
            .or_else(|| row.get("api_gravity"))
            .and_then(|v| v.parse::<f64>().ok());
        let tphc_ug_l = api_gravity.map(|api| {
            // Heavy crude (API < 20) → ~8000 µg/L, Light (API > 40) → ~500 µg/L
            let clamped = api.clamp(10.0, 60.0);
            8000.0 - ((clamped - 10.0) / 50.0) * 7500.0
        });

        let sensor_id = row
            .get("adios_oil_id")
            .or_else(|| row.get("name"))
            .or_else(|| row.get("oil_name"))
            .cloned()
            .unwrap_or_else(|| format!("ADIOS-{}", i));

        if pah_ratio.is_none() && tphc_ug_l.is_none() {
            return Err(ImportError::FieldMappingFailed {
                field: "pah_ratio / tphc_ug_l".into(),
                reason: "ADIOS record missing SARA fractions and API gravity".into(),
            });
        }

        Ok(RawSensorReading {
            sensor_id,
            domain: "OIL".into(),
            origin_hint: "SPILL".into(),
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
            spill_area_km2: None,
            sheen_detected: Some(pah_ratio.map(|p| p > 0.3).unwrap_or(false)),
            weathering_index: weathering_index.or(Some(0.1)),
        })
    }
}
