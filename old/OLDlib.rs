use pgrx::prelude::*;
use uuid::Uuid;

pgrx::pg_module_magic!();

/// BDBWay 1.0: 16-Byte Deterministic Identity
/// [0..7]   - UUID Partial (Root)
/// [8..11]  - Tribe ID (Partition)
/// [12]     - Color Red (Domain)
/// [13]     - Color Green (Quality - The Golden Selector)
/// [14]     - Color Blue (Temporal)
/// [15]     - Flags
#[pg_extern(immutable)]
fn bdb_generate_identity(
    uuid_str: &str,
    tribe_id: i32,
    red: i32,
    green: i32,
    blue: i32,
) -> Vec<u8> {
    let mut bytes = [0u8; 16];
    let u = Uuid::parse_str(uuid_str).expect("Invalid UUID");
    bytes[0..8].copy_from_slice(&u.as_bytes()[0..8]);

    let t_bytes = (tribe_id as u32).to_be_bytes();
    bytes[8..12].copy_from_slice(&t_bytes);

    bytes[12] = red as u8;
    bytes[13] = green as u8;
    bytes[14] = blue as u8;
    bytes[15] = 0b00000000;

    bytes.to_vec()
}

/// Akkadian Query Logic: Extract Quality (Byte 13)
#[pg_extern(immutable)]
fn bdb_get_quality(id: Vec<u8>) -> i32 {
    if id.len() < 14 { return 0; }
    id[13] as i32
}

/// Akkadian v3.4 Fuzzy Logic Engine
/// Determines the Green Channel (Quality) based on ISO-25012 Weights
#[pg_extern(immutable)]
fn bdb_evaluate_fuzzy_quality(
    completeness: f32,
    validity: f32,
    accuracy: f32,
) -> i32 {
    let score = (completeness * 0.4) + (validity * 0.3) + (accuracy * 0.3);
    (score * 255.0) as i32
}

/// Sovereign Classifier: Tells the UI how to render the particle
#[pg_extern(immutable)]
fn bdb_classify_node(id: Vec<u8>) -> String {
    let quality = bdb_get_quality(id);
    match quality {
        200..=255 => "SOVEREIGN_GEM (Golden)".to_string(),
        140..=199 => "ACTIVE_TRIBE_NODE".to_string(),
        100..=139 => "POOR_QUALITY_NODE".to_string(),
        _ => "NON_ACTIVE_PATHOGEN".to_string(),
    }
}

/// Geometric Link-less Join: Calculate Proximity
#[pg_extern(immutable)]
fn bdb_calculate_proximity(pos1: Vec<f32>, pos2: Vec<f32>) -> f32 {
    let dist: f32 = pos1.iter()
        .zip(pos2.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt();
    dist
}

/// KAKI Search: Placeholder for KD-Tree traversal logic
#[pg_extern]
fn bdb_kaki_search(
    target_quality: i32,
    _radius: f32, // Added _ to suppress warning
    _limit: i32   // Added _ to suppress warning
) -> Vec<Vec<u8>> {
    pgrx::notice!("KAKI: Scanning 4D Geometric Space for Quality {}", target_quality);
    let mut results = Vec::new();
    results.push(vec![0u8; 16]);
    results
}

/// AlertWay: Flag a node as "Pathogen" (Metamorphosis)
#[pg_extern]
fn bdb_detonate_node(mut id: Vec<u8>) -> Vec<u8> {
    if id.len() == 16 {
        id[13] = 45;
        id[15] = 0b00000001;
    }
    id
}
/// Sovereign GPS: Extract Longitude (X) from position
#[pg_extern(immutable)]
fn bdb_get_lon(pos: Vec<f32>) -> f32 {
    if pos.len() > 0 { pos[0] } else { 0.0 }
}

/// Sovereign GPS: Extract Latitude (Y) from position
#[pg_extern(immutable)]
fn bdb_get_lat(pos: Vec<f32>) -> f32 {
    if pos.len() > 1 { pos[1] } else { 0.0 }
}

/// Sovereign GPS: Extract Longitude (X) from position
#[pg_extern(immutable)]
fn bdb_get_lon(pos: pgrx::Array<f32>) -> f32 {
    pos.iter_deny_null()
        .next()
        .unwrap_or(0.0)
}

/// Sovereign GPS: Extract Latitude (Y) from position
#[pg_extern(immutable)]
fn bdb_get_lat(pos: pgrx::Array<f32>) -> f32 {
    pos.iter_deny_null()
        .nth(1)
        .unwrap_or(0.0)
}
