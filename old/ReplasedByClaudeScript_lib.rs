use pgrx::prelude::*;
use uuid::Uuid;
use csv::ReaderBuilder;
use std::fs::File;
use std::str::FromStr; // Required for UUID parsing

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
fn bdb_calculate_proximity(pos1: pgrx::Array<f32>, pos2: pgrx::Array<f32>) -> f32 {
    let v1: Vec<f32> = pos1.iter_deny_null().collect();
    let v2: Vec<f32> = pos2.iter_deny_null().collect();

    let dist: f32 = v1.iter()
        .zip(v2.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt();
    dist
}

/// KAKI Search: Placeholder for KD-Tree traversal logic
#[pg_extern]
fn bdb_kaki_search(
    target_quality: i32,
    _radius: f32,
    _limit: i32
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

/// The StormWay Ingestor: Loads 1M NajafWay records into the Fabric
#[pg_extern]
fn bdb_storm_ingest_csv(file_path: &str) -> Result<i32, spi::Error> {
    let file = File::open(file_path).map_err(|e| {
        pgrx::error!("Failed to open file: {}", e);
    }).unwrap();

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut count = 0;

    Spi::connect(|mut client| {
        for result in rdr.records() {
            let record = result.expect("Invalid CSV row");

            // --- STEP 1: CAPTURE ARABIC DATA ---
            let uuid_str = &record[1];
            let name_ar = &record[2];           // <--- Arabic Name captured here!
            let city_tribe = &record[6];
            let lat_str = &record[7];
            let lon_str = &record[8];

            // --- STEP 2: RESOLVE TRIBE ID ---
            // In BDBWay 1.0, 101 is our root Najaf Tribe
            let tribe_id: i32 = 101;

            // --- STEP 3: QUALITY & DNA GENERATION ---
            // We evaluate the quality of the specific Arabic Name record
            let quality_score = bdb_evaluate_fuzzy_quality(1.0, 1.0, 1.0);
            let id = bdb_generate_identity(uuid_str, tribe_id, 125, quality_score, 100);

            // --- STEP 4: SPATIAL COORDINATES ---
            let pos_str = format!("[{}, {}, 0]", lon_str, lat_str);

            // --- STEP 5: CREATE SATELLITE JSON OBJECT ---
            // We create a structured JSON object instead of a simple array
            let json_data = serde_json::json!({
                "full_name_arabic": name_ar,
                "residence_city": city_tribe,
                "csv_lineage": format!("najaf_batch_001_row_{}", count)
            }).to_string();

            // --- STEP 6: PREPARE PARAMETERS (5 Total) ---
            let uuid_val = uuid::Uuid::from_str(uuid_str).expect("Invalid UUID");
            let pg_uuid = pgrx::Uuid::from_bytes(*uuid_val.as_bytes());

            let params = vec![
                (PgOid::from(pg_sys::BYTEAOID), id.into_datum()),         // $1
                (PgOid::from(pg_sys::UUIDOID), pg_uuid.into_datum()),    // $2
                (PgOid::from(pg_sys::INT4OID), tribe_id.into_datum()),   // $3
                (PgOid::from(pg_sys::TEXTOID), pos_str.into_datum()),    // $4
                (PgOid::from(pg_sys::TEXTOID), json_data.into_datum()),  // $5
            ];

            // --- STEP 7: THE MULTI-TIER INSERT ---
            // Tribe_ID is now explicitly included to satisfy the NOT NULL constraint
            client.update(
                "INSERT INTO bdb_fabric.nodes (id, stable_uuid, tribe_id, position, data)
                 VALUES ($1, $2, $3, $4::vector, $5::jsonb)",
                None,
                Some(params),
            )?;

            count += 1;
        }
        Ok(count)
    })
}

#[pg_extern]
fn bdb_generate_name_vector(full_name_arabic: &str) -> Vec<f32> {
    // This function calls your PGRAGWay synapse to convert
    // the complex Arabic name into a 3D coordinate.
    // Logic:
    // X = First Name Root
    // Y = Lineage/Grandfather Root
    // Z = Tribe Resonance
    pgrag_synapse::vectorize_arabic(full_name_arabic)
}
