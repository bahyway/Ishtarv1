// ============================================================================
// hepta/tests/parser_integration.rs
// Integration tests: parse real .hepta map files → verify AST correctness
// ============================================================================

use hepta::{
    ast::{AttribValue, Coord, DistanceUnit},
    kaki::{ChordType, CoordSystem, PlanetNode},
    parse,
};

// ─── Baghdad Primary Map ─────────────────────────────────────────────────────

const BAGHDAD: &str = include_str!("../maps/baghdad_primary.hepta");

#[test]
fn baghdad_header_parses_correctly() {
    let f = parse(BAGHDAD).expect("baghdad_primary.hepta should parse");
    assert_eq!(f.header.version, "3.5.1");
    assert_eq!(f.header.coord_system, CoordSystem::Kaki7d);
    assert_eq!(
        f.header.project_id.as_deref(),
        Some("Baghdad_Heptagram_Primary")
    );
}

#[test]
fn baghdad_has_one_anchor() {
    let f = parse(BAGHDAD).unwrap();
    assert_eq!(f.anchors.len(), 1, "exactly one ANCHOR expected");
    let anchor = &f.anchors[0];
    assert_eq!(anchor.node.label, "SUN");
    assert_eq!(anchor.node.planet, PlanetNode::Sun);
    assert_eq!(anchor.node.sector_index, 0);
}

#[test]
fn baghdad_anchor_center_is_green_zone() {
    let f = parse(BAGHDAD).unwrap();
    let anchor = &f.anchors[0];
    match anchor.center.as_ref().unwrap() {
        Coord::LatLon { lat, lon } => {
            assert!((lat - 33.33).abs() < 0.001);
            assert!((lon - 44.39).abs() < 0.001);
        }
        other => panic!("expected LatLon, got {:?}", other),
    }
}

#[test]
fn baghdad_anchor_radius_is_15km() {
    let f = parse(BAGHDAD).unwrap();
    let dist = f.anchors[0].radius.as_ref().unwrap();
    assert!((dist.to_metres() - 15_000.0).abs() < 1.0);
    assert_eq!(dist.unit, DistanceUnit::Km);
}

#[test]
fn baghdad_has_six_sectors() {
    let f = parse(BAGHDAD).unwrap();
    assert_eq!(f.sectors.len(), 6, "Moon through Saturn = 6 sectors");
}

#[test]
fn baghdad_all_sector_planet_indices_are_sequential() {
    let f = parse(BAGHDAD).unwrap();
    for (i, sector) in f.sectors.iter().enumerate() {
        assert_eq!(
            sector.node.sector_index as usize, i + 1,
            "sector {} has wrong index: {}", i, sector.node.sector_index
        );
    }
}

#[test]
fn baghdad_sadr_city_beam_72_targets_venus() {
    let f = parse(BAGHDAD).unwrap();
    let sadr = f.sector_by_label("MOON").unwrap();
    let beam_72 = sadr.beams.iter().find(|b| b.chord == ChordType::Chord72).unwrap();
    assert_eq!(beam_72.target.label, "VENUS");
    assert_eq!(beam_72.target.planet, PlanetNode::Venus);
}

#[test]
fn baghdad_sadr_city_beam_73_targets_jupiter() {
    let f = parse(BAGHDAD).unwrap();
    let sadr = f.sector_by_label("MOON").unwrap();
    let beam_73 = sadr.beams.iter().find(|b| b.chord == ChordType::Chord73).unwrap();
    assert_eq!(beam_73.target.label, "JUPITER");
    assert_eq!(beam_73.target.planet, PlanetNode::Jupiter);
}

#[test]
fn baghdad_sadr_city_attribs_include_waste_demand() {
    let f = parse(BAGHDAD).unwrap();
    let sadr = f.sector_by_label("MOON").unwrap();
    let waste = sadr.attribs.iter()
        .find(|a| a.domain == "waste" && a.key == "demand")
        .unwrap();
    match &waste.value {
        AttribValue::Percent(v) => assert!((*v - 0.90).abs() < 0.001),
        other => panic!("expected Percent(0.90), got {:?}", other),
    }
}

#[test]
fn baghdad_mars_sector_has_elevated_risk() {
    let f = parse(BAGHDAD).unwrap();
    let mars = f.sector_by_label("MARS").unwrap();
    assert_eq!(mars.condition.as_deref(), Some("ELEVATED_RISK"));
}

#[test]
fn baghdad_total_beam_count() {
    let f = parse(BAGHDAD).unwrap();
    // Each of 6 sectors has 2 BEAM statements = 12
    assert_eq!(f.beam_count(), 12);
}

// ─── NajafWay / Wadi al-Salam Map ────────────────────────────────────────────

const NAJAF: &str = include_str!("../maps/wadi_al_salam_primary.hepta");

#[test]
fn najaf_header_parses_correctly() {
    let f = parse(NAJAF).expect("wadi_al_salam_primary.hepta should parse");
    assert_eq!(f.header.coord_system, CoordSystem::Kaki7dSacred);
    assert_eq!(
        f.header.project_id.as_deref(),
        Some("NajafWay_Wadi_Al_Salam")
    );
}

#[test]
fn najaf_anchor_is_imam_ali_shrine() {
    let f = parse(NAJAF).unwrap();
    assert_eq!(f.anchors.len(), 1);
    let anchor = &f.anchors[0];
    assert_eq!(anchor.node.planet, PlanetNode::Sun);
    assert!((anchor.sacred_weight.unwrap() - 1.0).abs() < 0.001);
    match anchor.center.as_ref().unwrap() {
        Coord::LatLon { lat, lon } => {
            assert!((lat - 31.9957).abs() < 0.001);
            assert!((lon - 44.3148).abs() < 0.001);
        }
        _ => panic!("expected LatLon"),
    }
}

#[test]
fn najaf_anchor_radius_is_2km() {
    let f = parse(NAJAF).unwrap();
    let dist = f.anchors[0].radius.as_ref().unwrap();
    assert!((dist.to_metres() - 2_000.0).abs() < 1.0);
}

#[test]
fn najaf_has_six_sectors() {
    let f = parse(NAJAF).unwrap();
    assert_eq!(f.sectors.len(), 6);
}

#[test]
fn najaf_has_one_node_block() {
    let f = parse(NAJAF).unwrap();
    assert_eq!(f.nodes.len(), 1, "the example family vault NODE block");
}

#[test]
fn najaf_saturn_sector_has_two_beams() {
    let f = parse(NAJAF).unwrap();
    let saturn = f.sector_by_label("SATURN").unwrap();
    assert_eq!(saturn.beams.len(), 2);
}

#[test]
fn najaf_saturn_chord_73_has_high_hidden_path_probability() {
    let f = parse(NAJAF).unwrap();
    let saturn = f.sector_by_label("SATURN").unwrap();
    let beam_73 = saturn.beams.iter()
        .find(|b| b.chord == ChordType::Chord73).unwrap();
    let prob = beam_73.properties.hidden_path_probability.unwrap();
    assert!(prob > 0.90, "Saturn {{7/3}} beam should have prob > 0.90, got {}", prob);
}

#[test]
fn najaf_saturn_condition_is_ruins() {
    let f = parse(NAJAF).unwrap();
    let saturn = f.sector_by_label("SATURN").unwrap();
    assert_eq!(saturn.condition.as_deref(), Some("HIGH_DENSITY_RUINS"));
}

#[test]
fn najaf_family_vault_node_is_family_key_access() {
    let f = parse(NAJAF).unwrap();
    let vault_node = &f.nodes[0];
    let attrib = vault_node.attribs.iter()
        .find(|a| a.domain == "vault" && a.key == "access_level")
        .unwrap();
    match &attrib.value {
        AttribValue::Symbol(s) => assert_eq!(s, "FAMILY_KEY"),
        other => panic!("expected Symbol('FAMILY_KEY'), got {:?}", other),
    }
}

#[test]
fn najaf_total_attrib_count_is_substantial() {
    let f = parse(NAJAF).unwrap();
    // Each of 6 sectors has 5–8 attribs plus the node block has 7
    assert!(f.attrib_count() >= 30, "expected ≥30 ATTRIBs, got {}", f.attrib_count());
}
