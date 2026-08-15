use pgrx::prelude::*;
use uuid::Uuid;
use csv::ReaderBuilder;
use std::fs::File;
use std::str::FromStr;

pgrx::pg_module_magic!();

/// BDBWay 1.0: 16-Byte Deterministic Identity
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
#[pg_extern(immutable)]
fn bdb_evaluate_fuzzy_quality(
    completeness: f32,
    validity: f32,
    accuracy: f32,
) -> i32 {
    let score = (completeness * 0.4) + (validity * 0.3) + (accuracy * 0.3);
    (score * 255.0) as i32
}

/// Sovereign Classifier
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

/// KAKI Search
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

/// AlertWay: Flag a node as "Pathogen"
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

/// The StormWay Ingestor: Loads NajafWay records into the Fabric
#[pg_extern]
fn bdb_storm_ingest_csv(file_path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut count = 0;

    Spi::connect(|mut client| {
        for result in rdr.records() {
            let record = result.map_err(|e| format!("Invalid CSV row: {}", e))?;

            // Extract CSV fields
            let uuid_str = &record[1];
            let name_ar = &record[2];
            let city_tribe = &record[6];
            let lat_str = &record[7];
            let lon_str = &record[8];

            // Parse coordinates
            let lat: f32 = lat_str.parse()
                .map_err(|_| format!("Invalid latitude: {}", lat_str))?;
            let lon: f32 = lon_str.parse()
                .map_err(|_| format!("Invalid longitude: {}", lon_str))?;

            // Tribe ID
            let tribe_id: i32 = 101;

            // Quality & Identity
            let quality_score = bdb_evaluate_fuzzy_quality(1.0, 1.0, 1.0);
            let id = bdb_generate_identity(uuid_str, tribe_id, 125, quality_score, 100);

            // Parse UUID
            let uuid_val = Uuid::from_str(uuid_str)
                .map_err(|e| format!("Invalid UUID: {}", e))?;
            let pg_uuid = pgrx::Uuid::from_bytes(*uuid_val.as_bytes());

            // Create JSON data
            let json_data = serde_json::json!({
                "full_name_arabic": name_ar,
                "residence_city": city_tribe,
                "csv_lineage": format!("najaf_batch_001_row_{}", count),
                "quality": quality_score
            }).to_string();

            // **CRITICAL FIX: Use the correct INSERT statement**
            // Insert into spatial.fabric_spatial_quads with real[] array
            let insert_sql = format!(
                "INSERT INTO spatial.fabric_spatial_quads (node_id, stable_uuid, position, data) \
                 VALUES ($1, $2, ARRAY[{}, {}, 0.0]::real[], $3::jsonb)",
                lon, lat
            );

            let params = vec![
                (PgOid::from(pg_sys::BYTEAOID), id.into_datum()),
                (PgOid::from(pg_sys::UUIDOID), pg_uuid.into_datum()),
                (PgOid::from(pg_sys::TEXTOID), json_data.into_datum()),
            ];

            client.update(
                &insert_sql,
                None,
                Some(params),
            )?;

            count += 1;

            // Progress indicator every 1000 records
            if count % 1000 == 0 {
                pgrx::notice!("Inserted {} records...", count);
            }
        }
        Ok(count)
    })
}
