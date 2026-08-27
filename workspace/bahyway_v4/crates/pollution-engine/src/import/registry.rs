//! # import::registry — Adapter Registry & Selection
//! **PollutionEngine v4.0 | BahyWay.Ecosystem | DUB.SAR 𒁾**

use super::{
    adapters::{
        eea_waterbase::EeaWaterbaseAdapter, generic::GenericAdapter, noaa_adios::NoaaAdiosAdapter,
        noaa_incident::NoaaIncidentAdapter, open_meteo::OpenMeteoAdapter, openaq::OpenAqAdapter,
    },
    detector::DetectionResult,
    router::ImportPayload,
    ImportError,
};
use crate::sensors::RawSensorReading;
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────
//  SOURCE ADAPTER TRAIT
// ─────────────────────────────────────────────────────────────

pub trait SourceAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn supported_formats(&self) -> &[super::router::PayloadFormat];
    fn normalize(
        &self,
        payload: &ImportPayload,
    ) -> (Vec<RawSensorReading>, Vec<(usize, ImportError)>);
}

// ─────────────────────────────────────────────────────────────
//  ADAPTER REGISTRY
// ─────────────────────────────────────────────────────────────

pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn SourceAdapter>>,
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        let mut r = Self {
            adapters: HashMap::new(),
        };
        r.register(Box::new(OpenAqAdapter));
        r.register(Box::new(EeaWaterbaseAdapter));
        r.register(Box::new(NoaaIncidentAdapter));
        r.register(Box::new(NoaaAdiosAdapter));
        r.register(Box::new(OpenMeteoAdapter));
        r.register(Box::new(GenericAdapter));
        r
    }
}

impl AdapterRegistry {
    pub fn register(&mut self, adapter: Box<dyn SourceAdapter>) {
        self.adapters.insert(adapter.name().to_string(), adapter);
    }

    pub fn get(&self, name: &str) -> Option<&dyn SourceAdapter> {
        self.adapters.get(name).map(|a| a.as_ref())
    }

    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.adapters.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Select best adapter name for a detection result + payload.
    ///
    /// Priority:
    /// 1. Source hint maps to a known adapter → use directly
    /// 2. Top domain + format → best-matching registered adapter
    /// 3. Fallback → "generic"
    pub fn select(
        &self,
        detection: &DetectionResult,
        payload: &ImportPayload,
    ) -> Result<String, ImportError> {
        // Priority 1: source hint
        let hint_name = payload.source.adapter_name();
        if hint_name != "generic" && self.adapters.contains_key(hint_name) {
            return Ok(hint_name.to_string());
        }

        // Unknown sources: always use generic — domain-heuristic adapters expect
        // specific wire formats from known sources and will produce no readings otherwise.
        if matches!(payload.source, super::router::SourceHint::Unknown(_)) {
            return Ok("generic".to_string());
        }

        // Priority 2: domain + format heuristic (known source without direct hint match)
        use super::router::PayloadFormat;
        use crate::domain::PollutionDomain;

        let top = detection.top_domain();
        let candidate = match (&top, &payload.format) {
            (PollutionDomain::Air, PayloadFormat::JsonArray | PayloadFormat::NdJson) => "openaq",
            (PollutionDomain::Air, PayloadFormat::Csv | PayloadFormat::Tsv) => "open_meteo",
            (PollutionDomain::Water, PayloadFormat::Csv | PayloadFormat::Tsv) => "eea_waterbase",
            (PollutionDomain::Water, PayloadFormat::JsonArray | PayloadFormat::NdJson) => "generic",
            (PollutionDomain::Oil, PayloadFormat::Csv | PayloadFormat::Tsv) => "noaa_incident",
            (PollutionDomain::Oil, PayloadFormat::JsonArray | PayloadFormat::NdJson) => {
                "noaa_adios"
            }
        };

        if self.adapters.contains_key(candidate) {
            return Ok(candidate.to_string());
        }

        Ok("generic".to_string())
    }
}
