//! # import::detector — Schema Detection with Confidence Scoring
//! **PollutionWay v3.5 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Scores each `PollutionDomain` (AIR / WATER / OIL) with a confidence
//! value in `[0.0, 1.0]` using three independent signals:
//!
//! | Signal | Weight | Method |
//! |--------|--------|--------|
//! | Field Signature  | 0.60 | Known field name sets per domain |
//! | Source Hint      | 0.25 | URL/filename origin match |
//! | Value Heuristics | 0.15 | Numeric range + label analysis |
//!
//! The winning domain is the one with the highest combined score.
//! If the winner's score is below the caller's threshold, the detection
//! is considered ambiguous and an `ImportError::Ambiguous` is returned.

use super::router::{ImportPayload, SourceHint};
use crate::domain::PollutionDomain;

// ─────────────────────────────────────────────────────────────
//  DOMAIN CONFIDENCE RECORD
// ─────────────────────────────────────────────────────────────

/// Per-domain confidence breakdown
#[derive(Debug, Clone)]
pub struct DomainConfidence {
    pub domain:            PollutionDomain,
    /// Field signature match score [0.0, 1.0]
    pub field_score:       f32,
    /// Source hint match score [0.0, 1.0]
    pub source_score:      f32,
    /// Value heuristic score [0.0, 1.0]
    pub heuristic_score:   f32,
    /// Weighted combined score
    pub combined:          f32,
    /// Which fields matched (for diagnostics)
    pub matched_fields:    Vec<String>,
}

impl DomainConfidence {
    fn new(domain: PollutionDomain) -> Self {
        Self {
            domain,
            field_score:     0.0,
            source_score:    0.0,
            heuristic_score: 0.0,
            combined:        0.0,
            matched_fields:  Vec::new(),
        }
    }

    fn compute_combined(&mut self) {
        self.combined =
            self.field_score     * 0.60
          + self.source_score    * 0.25
          + self.heuristic_score * 0.15;
    }
}

// ─────────────────────────────────────────────────────────────
//  DETECTION RESULT
// ─────────────────────────────────────────────────────────────

/// Full detection result for one import payload
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub air:   DomainConfidence,
    pub water: DomainConfidence,
    pub oil:   DomainConfidence,
    /// Row-level domain breakdown (for multi-domain files)
    pub row_domains: Vec<PollutionDomain>,
}

impl DetectionResult {
    /// Domain with the highest combined confidence
    pub fn top_domain(&self) -> PollutionDomain {
        let scores = [
            (self.air.combined,   PollutionDomain::Air),
            (self.water.combined, PollutionDomain::Water),
            (self.oil.combined,   PollutionDomain::Oil),
        ];
        scores.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, d)| *d)
            .unwrap_or(PollutionDomain::Air)
    }

    /// Confidence of the top domain
    pub fn top_confidence(&self) -> f32 {
        match self.top_domain() {
            PollutionDomain::Air   => self.air.combined,
            PollutionDomain::Water => self.water.combined,
            PollutionDomain::Oil   => self.oil.combined,
        }
    }

    /// True if this payload likely contains multiple domain types
    pub fn is_multi_domain(&self) -> bool {
        let threshold = 0.40;
        let above: usize = [self.air.combined, self.water.combined, self.oil.combined]
            .iter()
            .filter(|&&s| s >= threshold)
            .count();
        above > 1
    }

    /// Human-readable score summary for AKK alarm payload
    pub fn scores_summary(&self) -> String {
        format!(
            "AIR={:.2} WATER={:.2} OIL={:.2}",
            self.air.combined,
            self.water.combined,
            self.oil.combined,
        )
    }

    /// Per-domain confidence for the given domain
    pub fn confidence_for(&self, domain: PollutionDomain) -> f32 {
        match domain {
            PollutionDomain::Air   => self.air.combined,
            PollutionDomain::Water => self.water.combined,
            PollutionDomain::Oil   => self.oil.combined,
        }
    }

    /// Determine the best domain for a single row's field map
    pub fn domain_for_row(&self, row: &std::collections::HashMap<String, String>) -> PollutionDomain {
        // Quick field-presence check per row
        let has = |field: &str| row.contains_key(field);

        let air_hits = ["pm2_5","pm25","pm10","aqi","no2","o3","co","so2"]
            .iter().filter(|&&f| has(f)).count();
        let water_hits = ["turbidity","tds","wqi","do_mg","ph","bod","turbidity_ntu"]
            .iter().filter(|&&f| has(f)).count();
        let oil_hits = ["tphc","pah","sheen","weathering","spill","tpah"]
            .iter().filter(|&&f| has(f)).count();

        if air_hits >= water_hits && air_hits >= oil_hits {
            PollutionDomain::Air
        } else if water_hits >= oil_hits {
            PollutionDomain::Water
        } else {
            PollutionDomain::Oil
        }
    }
}

// ─────────────────────────────────────────────────────────────
//  FIELD SIGNATURE TABLES
// ─────────────────────────────────────────────────────────────

/// Known field name signatures per domain.
/// Each entry is (field_name_pattern, weight).
/// Weight sums within a domain are normalised to [0.0, 1.0].
const AIR_FIELDS: &[(&str, f32)] = &[
    ("pm2_5",           2.0),
    ("pm25",            2.0),
    ("pm10",            2.0),
    ("aqi",             2.0),
    ("air_quality",     1.5),
    ("no2",             1.5),
    ("no2_ppb",         1.5),
    ("o3",              1.0),
    ("ozone",           1.0),
    ("so2",             1.0),
    ("co",              0.8),
    ("co_ppm",          1.0),
    ("nox",             1.0),
    ("nh3",             0.8),
    ("particulate",     1.0),
    ("pollutant",       0.5),
    // OpenAQ-specific
    ("parameter",       0.5),
    ("value",           0.3),
    // Open-Meteo specific
    ("european_aqi",    2.0),
    ("us_aqi",          2.0),
    ("dust",            0.8),
];

const WATER_FIELDS: &[(&str, f32)] = &[
    ("turbidity",       2.0),
    ("turbidity_ntu",   2.0),
    ("wqi",             2.0),
    ("tds",             2.0),
    ("tds_mg_l",        2.0),
    ("do",              1.5),
    ("dissolved_oxygen",2.0),
    ("do_mg_l",         1.5),
    ("ph",              1.5),
    ("bod",             1.5),
    ("cod",             1.5),
    ("conductivity",    1.2),
    ("nitrate",         1.2),
    ("phosphate",       1.2),
    ("salinity",        1.0),
    ("temperature",     0.4),
    // EEA Waterbase specific
    ("observedproperty",1.0),
    ("resultobserved",  1.0),
    ("waterbase",       2.0),
    ("wfd",             1.5),
    ("river",           0.8),
    ("lake",            0.8),
];

const OIL_FIELDS: &[(&str, f32)] = &[
    ("tphc",            2.0),
    ("tphc_ug_l",       2.0),
    ("pah",             2.0),
    ("pah_ratio",       2.0),
    ("sheen",           2.0),
    ("sheen_detected",  2.0),
    ("spill",           2.0),
    ("oil",             1.5),
    ("crude",           1.5),
    ("hydrocarbon",     1.5),
    ("weathering",      1.5),
    ("dispersant",      1.5),
    ("tar",             1.2),
    ("tpah",            2.0),
    ("petroleum",       1.5),
    // NOAA IncidentNews specific
    ("threat",          1.0),
    ("material_spilled",1.5),
    ("countermeasure",  1.0),
    ("open_date",       0.5),
    // NOAA ADIOS specific
    ("api_gravity",     1.5),
    ("viscosity",       1.2),
    ("emulsification",  1.5),
];

// ─────────────────────────────────────────────────────────────
//  SCHEMA DETECTOR
// ─────────────────────────────────────────────────────────────

/// Stateless schema detector — call `detect()` on any `ImportPayload`
#[derive(Default)]
pub struct SchemaDetector;

impl SchemaDetector {
    /// Run full 3-signal detection on a parsed payload.
    pub fn detect(&self, payload: &ImportPayload) -> DetectionResult {
        let mut air   = DomainConfidence::new(PollutionDomain::Air);
        let mut water = DomainConfidence::new(PollutionDomain::Water);
        let mut oil   = DomainConfidence::new(PollutionDomain::Oil);

        // ── Signal 1: Field Signature (weight 0.60) ─────────
        self.score_field_signature(payload, &mut air, &mut water, &mut oil);

        // ── Signal 2: Source Hint (weight 0.25) ─────────────
        self.score_source_hint(payload, &mut air, &mut water, &mut oil);

        // ── Signal 3: Value Heuristics (weight 0.15) ────────
        self.score_value_heuristics(payload, &mut air, &mut water, &mut oil);

        // Compute weighted combined scores
        air.compute_combined();
        water.compute_combined();
        oil.compute_combined();

        // Row-level domain classification
        let row_domains = payload.rows.iter()
            .map(|row| {
                // Reuse a temporary result to classify per row
                let tmp = DetectionResult {
                    air: air.clone(), water: water.clone(), oil: oil.clone(),
                    row_domains: vec![],
                };
                tmp.domain_for_row(row)
            })
            .collect();

        DetectionResult { air, water, oil, row_domains }
    }

    // ── Signal 1: Field Signature ────────────────────────────

    fn score_field_signature(
        &self,
        payload: &ImportPayload,
        air: &mut DomainConfidence,
        water: &mut DomainConfidence,
        oil: &mut DomainConfidence,
    ) {
        let fields = &payload.field_names;

        let (air_score, air_matched)   = self.match_field_table(fields, AIR_FIELDS);
        let (water_score, water_matched) = self.match_field_table(fields, WATER_FIELDS);
        let (oil_score, oil_matched)   = self.match_field_table(fields, OIL_FIELDS);

        air.field_score     = air_score;
        water.field_score   = water_score;
        oil.field_score     = oil_score;
        air.matched_fields  = air_matched;
        water.matched_fields = water_matched;
        oil.matched_fields  = oil_matched;
    }

    /// Score field presence against a signature table.
    /// Returns (normalised_score [0,1], matched field names).
    fn match_field_table(
        &self,
        fields: &[String],
        table: &[(&str, f32)],
    ) -> (f32, Vec<String>) {
        // Normalize by top-10 weights: prevents score dilution from large signal tables.
        // A payload with 6 strong domain-specific fields should score near 1.0,
        // not 0.3 because the table has 22 theoretical patterns.
        let mut sorted: Vec<f32> = table.iter().map(|(_, w)| *w).collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let top_n = sorted.len().min(10);
        let max_possible: f32 = sorted[..top_n].iter().sum();

        let mut matched_weight = 0.0f32;
        let mut matched_names  = Vec::new();

        for (pattern, weight) in table {
            if fields.iter().any(|f| f.contains(pattern) || pattern.contains(f.as_str())) {
                matched_weight += weight;
                matched_names.push(pattern.to_string());
            }
        }

        let score = if max_possible > 0.0 {
            (matched_weight / max_possible).min(1.0)
        } else {
            0.0
        };

        (score, matched_names)
    }

    // ── Signal 2: Source Hint ────────────────────────────────

    fn score_source_hint(
        &self,
        payload: &ImportPayload,
        air: &mut DomainConfidence,
        water: &mut DomainConfidence,
        oil: &mut DomainConfidence,
    ) {
        match &payload.source {
            SourceHint::OpenAq | SourceHint::OpenMeteo => {
                air.source_score = 1.0;
            }
            SourceHint::EeaWaterbase => {
                water.source_score = 1.0;
            }
            SourceHint::NoaaIncident | SourceHint::NoaaAdios => {
                oil.source_score = 1.0;
            }
            SourceHint::Unknown(_) => {
                // No strong hint — all domains get 0.33
                air.source_score   = 0.33;
                water.source_score = 0.33;
                oil.source_score   = 0.33;
            }
        }
    }

    // ── Signal 3: Value Heuristics ───────────────────────────

    fn score_value_heuristics(
        &self,
        payload: &ImportPayload,
        air: &mut DomainConfidence,
        water: &mut DomainConfidence,
        oil: &mut DomainConfidence,
    ) {
        // Sample first 20 rows for efficiency
        let sample: Vec<&std::collections::HashMap<String, String>> =
            payload.rows.iter().take(20).collect();

        let mut air_hits   = 0u32;
        let mut water_hits = 0u32;
        let mut oil_hits   = 0u32;

        for row in &sample {
            // AQI range check: 0–500
            if let Some(v) = self.parse_any_of(row, &["aqi","air_quality_index","us_aqi","european_aqi"]) {
                if (0.0..=500.0).contains(&v) { air_hits += 2; }
            }
            // PM2.5 range: 0–500 µg/m³
            if let Some(v) = self.parse_any_of(row, &["pm2_5","pm25","pm2.5"]) {
                if (0.0..=500.0).contains(&v) { air_hits += 2; }
            }
            // pH range: 0–14
            if let Some(v) = self.parse_any_of(row, &["ph","ph_value"]) {
                if (0.0..=14.0).contains(&v) { water_hits += 2; }
            }
            // Turbidity: 0–1000 NTU
            if let Some(v) = self.parse_any_of(row, &["turbidity","turbidity_ntu","ntu"]) {
                if (0.0..=1000.0).contains(&v) { water_hits += 2; }
            }
            // PAH ratio: 0–1
            if let Some(v) = self.parse_any_of(row, &["pah","pah_ratio"]) {
                if (0.0..=1.0).contains(&v) { oil_hits += 3; } // strong signal
            }
            // TPHC: typically > 1 µg/L for oil readings
            if let Some(v) = self.parse_any_of(row, &["tphc","tphc_ug_l","tph"]) {
                if v > 0.0 { oil_hits += 3; }
            }

            // Keyword scan in string values
            for val in row.values() {
                let v = val.to_lowercase();
                if v.contains("oil") || v.contains("spill") || v.contains("petroleum") {
                    oil_hits += 1;
                }
                if v.contains("river") || v.contains("lake") || v.contains("water") {
                    water_hits += 1;
                }
                if v.contains("air") || v.contains("particulate") || v.contains("emission") {
                    air_hits += 1;
                }
            }
        }

        let total = (air_hits + water_hits + oil_hits) as f32;
        if total > 0.0 {
            air.heuristic_score   = air_hits   as f32 / total;
            water.heuristic_score = water_hits as f32 / total;
            oil.heuristic_score   = oil_hits   as f32 / total;
        }
    }

    /// Try to parse a numeric value from any of a list of field names
    fn parse_any_of(
        &self,
        row: &std::collections::HashMap<String, String>,
        fields: &[&str],
    ) -> Option<f64> {
        for &field in fields {
            if let Some(v) = row.get(field).and_then(|s| s.trim().parse::<f64>().ok()) {
                return Some(v);
            }
        }
        None
    }
}
