//! # Import Integration Tests — ze kiest EAV schema
//! **PollutionEngine v4.0 | BahyWay.Ecosystem | DUB.SAR 𒁾**

use pollution_engine::import::{
    smart_import,
    detector::SchemaDetector,
    router::{ImportRouter, PayloadFormat, SourceHint},
};
use pollution_engine::domain::PollutionDomain;

// ─────────────────────────────────────────────────────────────
//  ROUTER TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_router_detects_json_array() {
    let json = r#"[{"pm2_5": 12.3, "lat": 52.37, "lon": 4.89}]"#;
    let router = ImportRouter::default();
    let payload = router.parse(json, "https://openaq.org/test").unwrap();
    assert_eq!(payload.format, PayloadFormat::JsonArray);
}

#[test]
fn test_router_detects_csv() {
    let csv = "sensor_id,lat,lon,pm2_5,pm10\nS1,52.37,4.89,12.3,25.0\n";
    let router = ImportRouter::default();
    let payload = router.parse(csv, "generic_file.csv").unwrap();
    assert_eq!(payload.format, PayloadFormat::Csv);
}

#[test]
fn test_router_source_hint_openaq() {
    let json = r#"[{"pm2_5": 5.0, "lat": 1.0, "lon": 1.0}]"#;
    let router = ImportRouter::default();
    let payload = router.parse(json, "https://api.openaq.org/v3/measurements").unwrap();
    assert_eq!(payload.source, SourceHint::OpenAq);
}

#[test]
fn test_router_source_hint_eea() {
    let csv = "monitoringsiteid,lat,lon\nEEA_NL001,52.1,4.5\n";
    let router = ImportRouter::default();
    let payload = router.parse(csv, "https://www.eea.europa.eu/waterbase/data.csv").unwrap();
    assert_eq!(payload.source, SourceHint::EeaWaterbase);
}

#[test]
fn test_router_source_hint_noaa() {
    let csv = "name,lat,lon,threat\nSpill 1,29.5,-90.1,Oil\n";
    let router = ImportRouter::default();
    let payload = router.parse(csv, "https://incidentnews.noaa.gov/raw/index").unwrap();
    assert_eq!(payload.source, SourceHint::NoaaIncident);
}

#[test]
fn test_router_flattens_nested_json() {
    let json = r#"[{
        "coordinates": {"latitude": 52.37, "longitude": 4.89},
        "sensor": {"id": "S-001"}
    }]"#;
    let router = ImportRouter::default();
    let payload = router.parse(json, "unknown").unwrap();
    assert_eq!(payload.rows.len(), 1);
    assert!(payload.has_field("coordinates_latitude") || payload.has_field("sensor_id"));
}

#[test]
fn test_router_ndjson_parses_multiple_rows() {
    let ndjson = "{\"pm2_5\":5.0,\"lat\":52.0,\"lon\":4.8}\n{\"pm2_5\":8.0,\"lat\":52.1,\"lon\":4.9}";
    let router = ImportRouter::default();
    let payload = router.parse(ndjson, "unknown").unwrap();
    assert_eq!(payload.rows.len(), 2);
}

#[test]
fn test_router_empty_payload_returns_error() {
    let router = ImportRouter::default();
    assert!(router.parse("", "unknown").is_err());
    assert!(router.parse("  \n  ", "unknown").is_err());
}

// ─────────────────────────────────────────────────────────────
//  SCHEMA DETECTOR TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_detector_air_from_pm_fields() {
    let csv = "sensor_id,lat,lon,pm2_5,pm10,aqi,no2\nS1,52.37,4.89,12.3,25.0,48,18.0\n";
    let router = ImportRouter::default();
    let payload = router.parse(csv, "unknown_source").unwrap();

    let detector = SchemaDetector::default();
    let result   = detector.detect(&payload);

    assert_eq!(result.top_domain(), PollutionDomain::Air,
        "PM fields should detect AIR domain");
    assert!(result.top_confidence() > 0.50,
        "AIR confidence should exceed 0.50, got {:.2}", result.top_confidence());
}

#[test]
fn test_detector_water_from_turbidity_ph() {
    let csv = "stationid,lat,lon,turbidity_ntu,ph,do_mg_l,bod_mg_l\nW1,51.9,4.4,1.5,7.2,8.5,2.0\n";
    let router = ImportRouter::default();
    let payload = router.parse(csv, "unknown").unwrap();

    let detector = SchemaDetector::default();
    let result   = detector.detect(&payload);

    assert_eq!(result.top_domain(), PollutionDomain::Water);
    assert!(result.top_confidence() > 0.50);
}

#[test]
fn test_detector_oil_from_tphc_pah() {
    let csv = "name,lat,lon,tphc_ug_l,pah_ratio,sheen_detected\nSpill1,51.8,4.1,3500.0,0.45,true\n";
    let router = ImportRouter::default();
    let payload = router.parse(csv, "unknown").unwrap();

    let detector = SchemaDetector::default();
    let result   = detector.detect(&payload);

    assert_eq!(result.top_domain(), PollutionDomain::Oil);
    assert!(result.top_confidence() > 0.50);
}

#[test]
fn test_detector_source_hint_boosts_confidence() {
    let csv = "id,lat,lon,value\nS1,52.0,4.8,42.0\n";
    let router = ImportRouter::default();

    let payload_unknown = router.parse(csv, "unknown_source").unwrap();
    let payload_openaq  = router.parse(csv, "https://api.openaq.org/v3/").unwrap();

    let detector = SchemaDetector::default();
    let det_unknown = detector.detect(&payload_unknown);
    let det_openaq  = detector.detect(&payload_openaq);

    assert!(
        det_openaq.air.combined > det_unknown.air.combined,
        "OpenAQ source hint should increase AIR confidence: {:.2} vs {:.2}",
        det_openaq.air.combined, det_unknown.air.combined
    );
}

#[test]
fn test_detector_multi_domain_detection() {
    let csv = "id,lat,lon,pm2_5,turbidity_ntu\nMix1,52.0,4.8,15.0,3.5\n";
    let router = ImportRouter::default();
    let payload = router.parse(csv, "unknown").unwrap();

    let detector = SchemaDetector::default();
    let result   = detector.detect(&payload);

    assert!(result.air.field_score   > 0.0);
    assert!(result.water.field_score > 0.0);
}

// ─────────────────────────────────────────────────────────────
//  OPENAQ ADAPTER TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_openaq_json_ingestion() {
    let json = r#"[
        {
            "locationId": "AMS-001",
            "coordinates": {"latitude": 52.370, "longitude": 4.895},
            "parameter": "pm25",
            "value": 12.5
        },
        {
            "locationId": "AMS-001",
            "coordinates": {"latitude": 52.370, "longitude": 4.895},
            "parameter": "pm10",
            "value": 28.0
        },
        {
            "locationId": "AMS-001",
            "coordinates": {"latitude": 52.370, "longitude": 4.895},
            "parameter": "no2",
            "value": 22.0
        }
    ]"#;

    let result = smart_import(json, "https://api.openaq.org/v3/measurements", 0.40).unwrap();
    assert!(result.success_count() >= 1, "OpenAQ pivot should produce at least 1 location reading");
    assert_eq!(result.adapter_used, "openaq");
    assert_eq!(result.detection.top_domain(), PollutionDomain::Air);
}

// ─────────────────────────────────────────────────────────────
//  EEA WATERBASE ADAPTER TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_eea_waterbase_csv_ingestion() {
    let csv = "\
monitoringsiteid,lat,lon,observedproperty,resultobserved,phenomenontimestart\n\
NL001,52.1,4.5,Turbidity,1.8,2024-01-01\n\
NL001,52.1,4.5,Ph,7.1,2024-01-01\n\
NL001,52.1,4.5,DissolvedOxygen,8.5,2024-01-01\n\
NL002,51.9,4.4,Turbidity,45.0,2024-01-02\n\
NL002,51.9,4.4,Ph,5.8,2024-01-02\n";

    let result = smart_import(csv, "https://www.eea.europa.eu/waterbase/data.csv", 0.50).unwrap();
    assert!(result.success_count() >= 2, "Should produce at least 2 pivoted station readings");
    assert_eq!(result.adapter_used, "eea_waterbase");
}

// ─────────────────────────────────────────────────────────────
//  NOAA INCIDENT ADAPTER TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_noaa_incident_csv_ingestion() {
    let csv = "\
name,lat,lon,threat,material_spilled,max_ptl_release,tags\n\
Gulf Spill 2010,28.7,-88.4,Oil,crude oil,210000000,\"sheen|dispersant\"\n\
River Chemical,39.1,-84.5,Chemical,acetone,5000,\n\
Tanker Leak,51.9,4.1,Oil,diesel,50000,\"sheen\"\n";

    let result = smart_import(csv, "https://incidentnews.noaa.gov/raw/index", 0.40).unwrap();
    assert_eq!(result.success_count(), 2, "Should only import Oil-threat incidents");
    assert_eq!(result.adapter_used, "noaa_incident");
}

// ─────────────────────────────────────────────────────────────
//  GENERIC ADAPTER TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_generic_adapter_air_row() {
    let csv = "\
station_id,latitude,longitude,pm2_5_ug_m3,pm10_ug_m3,aqi_value\n\
RIVM-001,52.37,4.89,9.5,22.0,38\n";

    let result = smart_import(csv, "https://rivm.nl/custom_data.csv", 0.20).unwrap();
    assert_eq!(result.success_count(), 1);
    assert_eq!(result.adapter_used, "generic");
    assert_eq!(result.readings[0].domain, "AIR");
}

#[test]
fn test_generic_adapter_multi_domain_rows() {
    let csv = "\
id,lat,lon,pm2_5,turbidity_ntu,tphc_ug_l\n\
R1,52.0,4.8,15.0,,\n\
R2,52.1,4.9,,3.5,\n\
R3,52.2,5.0,,,2500.0\n";

    let result = smart_import(csv, "https://unknown.nl/mixed.csv", 0.15).unwrap();
    assert_eq!(result.success_count(), 3);

    let domains: Vec<&str> = result.readings.iter().map(|r| r.domain.as_str()).collect();
    assert!(domains.contains(&"AIR"),   "Row 1 should be AIR");
    assert!(domains.contains(&"WATER"), "Row 2 should be WATER");
    assert!(domains.contains(&"OIL"),   "Row 3 should be OIL");
}

#[test]
fn test_generic_adapter_unknown_rows_fail_gracefully() {
    let csv = "col_a,col_b\nfoo,bar\nbaz,qux\n";
    let result = smart_import(csv, "unknown", 0.0).unwrap();
    assert_eq!(result.success_count(), 0);
    assert_eq!(result.error_count(), 2);
}

// ─────────────────────────────────────────────────────────────
//  CONFIDENCE THRESHOLD TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_ambiguous_detection_returns_error() {
    let csv = "col_a,col_b,col_c\n1,2,3\n4,5,6\n";
    let result = smart_import(csv, "truly_unknown_source.csv", 0.70);
    assert!(result.is_err(), "No recognisable fields should trigger Ambiguous error");

    if let Err(pollution_engine::import::ImportError::Ambiguous { confidence, .. }) = result {
        assert!(confidence < 0.70);
    }
}

#[test]
fn test_threshold_zero_always_succeeds_for_valid_data() {
    let csv = "sensor_id,lat,lon,pm2_5\nS1,52.0,4.8,10.0\n";
    let result = smart_import(csv, "unknown", 0.0);
    assert!(result.is_ok());
}

// ─────────────────────────────────────────────────────────────
//  AKK SUMMARY TESTS
// ─────────────────────────────────────────────────────────────

#[test]
fn test_import_result_akk_summary_format() {
    let json = r#"[{"locationId":"S1","coordinates":{"latitude":52.0,"longitude":4.8},"parameter":"pm25","value":10.0}]"#;
    let result = smart_import(json, "https://api.openaq.org/v3/", 0.30).unwrap();
    let summary = result.akk_summary();

    assert!(summary.starts_with("IMPORT source="),  "AKK summary must start with IMPORT");
    assert!(summary.contains("adapter="),            "AKK summary must contain adapter");
    assert!(summary.contains("confidence="),         "AKK summary must contain confidence");
    assert!(summary.contains("ok="),                 "AKK summary must contain ok count");
}

#[test]
fn test_import_result_is_clean_flag() {
    let csv = "sensor_id,lat,lon,pm2_5,pm10\nS1,52.37,4.89,12.0,25.0\n";
    let result = smart_import(csv, "unknown", 0.0).unwrap();

    if result.success_count() > 0 && result.error_count() == 0 {
        assert!(result.is_clean());
    }
}
