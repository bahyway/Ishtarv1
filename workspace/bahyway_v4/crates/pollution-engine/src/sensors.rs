//! # RawSensorReading — Universal sensor reading record
//! **PollutionEngine v4.0 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! One struct covers all three domains.
//! Fields not applicable to the domain are `None`.

#[derive(Debug, Clone)]
pub struct RawSensorReading {
    pub sensor_id:   String,
    /// "AIR" | "WATER" | "OIL"
    pub domain:      String,
    /// "ANTHROPOGENIC" | "NATURAL" | "SPILL" | "CHEMICAL"
    pub origin_hint: String,
    pub lat:         f64,
    pub lon:         f64,
    pub altitude_m:  Option<f32>,
    // ── AIR ─────────────────────────────────────────────────
    pub pm2_5:   Option<f64>,
    pub pm10:    Option<f64>,
    pub no2_ppb: Option<f64>,
    pub o3_ppb:  Option<f64>,
    pub so2_ppb: Option<f64>,
    pub co_ppm:  Option<f64>,
    pub aqi:     Option<u16>,
    // ── WATER ────────────────────────────────────────────────
    pub turbidity_ntu: Option<f64>,
    pub tds_mg_l:      Option<f64>,
    pub do_mg_l:       Option<f64>,
    pub ph:            Option<f64>,
    pub bod_mg_l:      Option<f64>,
    pub wqi:           Option<f32>,
    // ── OIL ──────────────────────────────────────────────────
    pub tphc_ug_l:        Option<f64>,
    pub pah_ratio:        Option<f64>,
    pub spill_area_km2:   Option<f64>,
    pub sheen_detected:   Option<bool>,
    pub weathering_index: Option<f32>,
}
