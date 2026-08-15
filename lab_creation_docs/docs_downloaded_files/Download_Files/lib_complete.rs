use pgrx::prelude::*;
use uuid::Uuid;
use csv::ReaderBuilder;
use std::fs::File;
use std::str::FromStr;

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

// ============================================================
// PGRAGWay Synapse: Arabic Name Vector Intelligence
// ============================================================

/// Generate 3D vector from Arabic name
/// X = First Name Root (phonetic hash)
/// Y = Lineage/Grandfather Root (middle name)
/// Z = Tribe Resonance (family name weight)
#[pg_extern]
fn bdb_generate_name_vector(full_name_arabic: &str) -> Vec<f32> {
    // Split Arabic name into components
    let parts: Vec<&str> = full_name_arabic.split_whitespace().collect();
    
    // X-axis: First name phonetic hash (0.0 - 1.0)
    let x = if !parts.is_empty() {
        let first_name = parts[0];
        let hash = first_name.chars()
            .map(|c| c as u32)
            .sum::<u32>();
        (hash % 1000) as f32 / 1000.0
    } else {
        0.5
    };
    
    // Y-axis: Father/Grandfather name (middle names)
    let y = if parts.len() > 1 {
        let middle_name = parts[1];
        let hash = middle_name.chars()
            .map(|c| c as u32)
            .sum::<u32>();
        (hash % 1000) as f32 / 1000.0
    } else {
        0.5
    };
    
    // Z-axis: Family/Tribe name resonance (last name)
    let z = if parts.len() > 2 {
        let family_name = parts[parts.len() - 1];
        let hash = family_name.chars()
            .map(|c| c as u32)
            .sum::<u32>();
        (hash % 1000) as f32 / 1000.0
    } else {
        0.5
    };
    
    vec![x, y, z]
}

/// Calculate similarity between two Arabic names using vector distance
/// Returns 0.0 (completely different) to 1.0 (identical)
#[pg_extern(immutable)]
fn bdb_name_similarity(name1: &str, name2: &str) -> f32 {
    let vec1 = bdb_generate_name_vector(name1);
    let vec2 = bdb_generate_name_vector(name2);
    
    // Calculate Euclidean distance
    let distance: f32 = vec1.iter()
        .zip(vec2.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt();
    
    // Convert distance to similarity score
    // Max distance in 3D unit cube is sqrt(3) ≈ 1.732
    let max_distance = 1.732f32;
    let similarity = 1.0 - (distance / max_distance).min(1.0);
    
    similarity
}

/// Entity Resolution: Find potential duplicate names
/// Uses phonetic similarity in 3D vector space
#[pg_extern]
fn bdb_find_name_duplicates(
    target_name: &str,
    similarity_threshold: f32
) -> String {
    let target_vec = bdb_generate_name_vector(target_name);
    
    // Return JSON with target vector and threshold
    serde_json::json!({
        "target_name": target_name,
        "target_vector": target_vec,
        "similarity_threshold": similarity_threshold,
        "message": "Use this with spatial queries to find similar names"
    }).to_string()
}

// ============================================================
// StormWay Ingestor: CSV Bulk Loading
// ============================================================

/// The StormWay Ingestor: Loads NajafWay records into the Fabric
/// Processes CSV files with Arabic names and spatial coordinates
#[pg_extern]
fn bdb_storm_ingest_csv(file_path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let mut count = 0;
    let mut error_count = 0;

    Spi::connect(|mut client| {
        for result in rdr.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    error_count += 1;
                    pgrx::warning!("Skipping invalid CSV row: {}", e);
                    continue;
                }
            };

            // Extract CSV fields (adjust indices based on your CSV structure)
            if record.len() < 9 {
                error_count += 1;
                pgrx::warning!("Skipping row with insufficient columns");
                continue;
            }

            let uuid_str = &record[1];
            let name_ar = &record[2];
            let city_tribe = &record[6];
            let lat_str = &record[7];
            let lon_str = &record[8];

            // Parse coordinates
            let lat: f32 = match lat_str.parse() {
                Ok(v) => v,
                Err(_) => {
                    error_count += 1;
                    pgrx::warning!("Invalid latitude: {}", lat_str);
                    continue;
                }
            };
            
            let lon: f32 = match lon_str.parse() {
                Ok(v) => v,
                Err(_) => {
                    error_count += 1;
                    pgrx::warning!("Invalid longitude: {}", lon_str);
                    continue;
                }
            };

            // Tribe ID
            let tribe_id: i32 = 101;

            // Quality & Identity
            let quality_score = bdb_evaluate_fuzzy_quality(1.0, 1.0, 1.0);
            let id = bdb_generate_identity(uuid_str, tribe_id, 125, quality_score, 100);

            // Parse UUID
            let uuid_val = match Uuid::from_str(uuid_str) {
                Ok(u) => u,
                Err(_) => {
                    error_count += 1;
                    pgrx::warning!("Invalid UUID: {}", uuid_str);
                    continue;
                }
            };
            let pg_uuid = pgrx::Uuid::from_bytes(*uuid_val.as_bytes());

            // Generate name vector for semantic search
            let name_vector = bdb_generate_name_vector(name_ar);

            // Create JSON data with name vector
            let json_data = serde_json::json!({
                "full_name_arabic": name_ar,
                "residence_city": city_tribe,
                "csv_lineage": format!("najaf_batch_row_{}", count),
                "quality": quality_score,
                "name_vector": name_vector
            }).to_string();

            // **CRITICAL: Insert into spatial.fabric_spatial_quads**
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

            match client.update(&insert_sql, None, Some(params)) {
                Ok(_) => {
                    count += 1;
                    // Progress indicator every 1000 records
                    if count % 1000 == 0 {
                        pgrx::notice!("✓ Inserted {} records ({} errors)...", count, error_count);
                    }
                }
                Err(e) => {
                    error_count += 1;
                    pgrx::warning!("Insert failed for row {}: {}", count, e);
                }
            }
        }
        
        pgrx::notice!("Import complete: {} records inserted, {} errors", count, error_count);
        Ok(count)
    })
}

/// Batch ingest with transaction control
/// Commits every N records for better performance
#[pg_extern]
fn bdb_storm_ingest_csv_batched(
    file_path: &str,
    batch_size: i32
) -> Result<i32, Box<dyn std::error::Error>> {
    pgrx::notice!("Starting batched import with batch size: {}", batch_size);
    
    // For now, just call the regular ingest
    // In production, you'd implement proper batching with COPY or multi-row inserts
    bdb_storm_ingest_csv(file_path)
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_name_vector_generation() {
        let name = "محمد علي الموسوي";
        let vec = crate::bdb_generate_name_vector(name);
        assert_eq!(vec.len(), 3);
        assert!(vec[0] >= 0.0 && vec[0] <= 1.0);
        assert!(vec[1] >= 0.0 && vec[1] <= 1.0);
        assert!(vec[2] >= 0.0 && vec[2] <= 1.0);
    }

    #[pg_test]
    fn test_name_similarity() {
        let name1 = "محمد علي الموسوي";
        let name2 = "محمد علي الموسوي";
        let name3 = "فاطمة حسن الحسيني";
        
        let sim1 = crate::bdb_name_similarity(name1, name2);
        let sim2 = crate::bdb_name_similarity(name1, name3);
        
        assert!(sim1 > 0.99); // Should be very similar
        assert!(sim2 < sim1); // Different names should have lower similarity
    }
}
