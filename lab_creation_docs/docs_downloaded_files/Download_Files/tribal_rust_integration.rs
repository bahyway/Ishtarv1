// ============================================================
// BDBWay Tribal Color Registry Integration
// Add to your lib.rs
// ============================================================

use pgrx::prelude::*;

/// Tribal Identity Validation Result
#[derive(PostgresType, Serialize, Deserialize, Debug)]
pub struct TribalIdentity {
    pub tribe_id: Option<i32>,
    pub tribe_name_ar: String,
    pub assigned_color: i32,
    pub ethnicity: String,
    pub confidence: f32,
}

/// Enhanced BDBWay Identity Generator with Tribal Color Validation
/// Automatically assigns correct tribal color based on family name
#[pg_extern]
fn bdb_generate_tribal_identity(
    uuid_str: &str,
    full_name_arabic: &str,
    tribe_id: i32,
    green: i32,  // Quality score
    blue: i32    // Temporal marker
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Query database for tribal color
    let tribal_color = Spi::connect(|client| {
        let query = "
            SELECT assigned_color 
            FROM bdb_validate_tribal_identity($1) 
            LIMIT 1
        ";
        
        let result = client.select(
            query, 
            None, 
            Some(vec![(PgBuiltInOids::TEXTOID.oid(), full_name_arabic.into_datum())])
        )?;
        
        if let Some(row) = result.first() {
            if let Ok(Some(color)) = row.get::<i32>(1) {
                return Ok(color);
            }
        }
        
        // Fallback: non-tribal color
        Ok(220)
    })?;
    
    // Generate identity with tribal-validated color
    let mut bytes = [0u8; 16];
    let u = uuid::Uuid::parse_str(uuid_str)?;
    bytes[0..8].copy_from_slice(&u.as_bytes()[0..8]);
    
    let t_bytes = (tribe_id as u32).to_bytes();
    bytes[8..12].copy_from_slice(&t_bytes);
    
    bytes[12] = tribal_color as u8;  // RED: Tribal color
    bytes[13] = green as u8;          // GREEN: Quality
    bytes[14] = blue as u8;           // BLUE: Temporal
    bytes[15] = 0b00000000;           // FLAGS
    
    Ok(bytes.to_vec())
}

/// Validate tribal affiliation and return full details
#[pg_extern]
fn bdb_validate_tribe_from_name(
    full_name_arabic: &str
) -> Result<JsonB, Box<dyn std::error::Error>> {
    Spi::connect(|client| {
        let query = "
            SELECT row_to_json(t)
            FROM (
                SELECT 
                    tribe_id,
                    tribe_name_ar,
                    assigned_color,
                    ethnicity::TEXT,
                    confidence
                FROM bdb_validate_tribal_identity($1)
            ) t
        ";
        
        let result = client.select(
            query,
            None,
            Some(vec![(PgBuiltInOids::TEXTOID.oid(), full_name_arabic.into_datum())])
        )?;
        
        if let Some(row) = result.first() {
            if let Ok(Some(json)) = row.get::<JsonB>(1) {
                return Ok(json);
            }
        }
        
        // Return default non-tribal
        Ok(JsonB(serde_json::json!({
            "tribe_id": null,
            "tribe_name_ar": "غير قبلي",
            "assigned_color": 220,
            "ethnicity": "UNKNOWN",
            "confidence": 0.0
        })))
    })
}

/// Get tribal hierarchy as formatted string
#[pg_extern]
fn bdb_get_tribe_path(tribe_id: i32) -> Result<String, Box<dyn std::error::Error>> {
    Spi::connect(|client| {
        let query = "SELECT bdb_get_tribal_hierarchy($1)";
        
        let result = client.select(
            query,
            None,
            Some(vec![(PgBuiltInOids::INT4OID.oid(), tribe_id.into_datum())])
        )?;
        
        if let Some(row) = result.first() {
            if let Ok(Some(path)) = row.get::<String>(1) {
                return Ok(path);
            }
        }
        
        Ok("Unknown".to_string())
    })
}

/// Enhanced CSV Ingestor with Tribal Validation
#[pg_extern]
fn bdb_storm_ingest_csv_with_tribal_validation(
    file_path: &str
) -> Result<i32, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    
    let mut count = 0;
    
    Spi::connect(|mut client| {
        for result in rdr.records() {
            let record = result?;
            
            let uuid_str = &record[1];
            let name_ar = &record[2];
            let lat_str = &record[7];
            let lon_str = &record[8];
            
            let lat: f32 = lat_str.parse()?;
            let lon: f32 = lon_str.parse()?;
            
            let tribe_id: i32 = 101;
            
            // AUTO-DETECT TRIBAL COLOR
            let tribal_result = bdb_validate_tribe_from_name(name_ar)?;
            let tribal_data: serde_json::Value = serde_json::from_str(&tribal_result.0.to_string())?;
            
            let tribal_color = tribal_data["assigned_color"]
                .as_i64()
                .unwrap_or(220) as i32;
            
            let tribe_name = tribal_data["tribe_name_ar"]
                .as_str()
                .unwrap_or("غير قبلي");
            
            let ethnicity = tribal_data["ethnicity"]
                .as_str()
                .unwrap_or("UNKNOWN");
            
            // Quality score
            let quality_score = bdb_evaluate_fuzzy_quality(1.0, 1.0, 1.0);
            
            // Generate identity with validated tribal color
            let id = bdb_generate_tribal_identity(
                uuid_str,
                name_ar,
                tribe_id,
                quality_score,
                75
            )?;
            
            let uuid_val = Uuid::from_str(uuid_str)?;
            let pg_uuid = pgrx::Uuid::from_bytes(*uuid_val.as_bytes());
            
            // Enhanced JSON with tribal info
            let json_data = serde_json::json!({
                "full_name_arabic": name_ar,
                "quality": quality_score,
                "tribal_affiliation": tribe_name,
                "tribal_color": tribal_color,
                "ethnicity": ethnicity,
                "csv_lineage": format!("row_{}", count)
            }).to_string();
            
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
            
            client.update(&insert_sql, None, Some(params))?;
            
            count += 1;
            
            if count % 1000 == 0 {
                pgrx::notice!(
                    "✓ {} records | Tribal colors assigned automatically", 
                    count
                );
            }
        }
        
        Ok(count)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[pg_test]
    fn test_tribal_validation() {
        // Test Arab tribe
        let result = bdb_validate_tribe_from_name("محمد علي الدليمي").unwrap();
        let data: serde_json::Value = serde_json::from_str(&result.0.to_string()).unwrap();
        assert_eq!(data["tribe_name_ar"], "الدليم");
        assert!(data["assigned_color"].as_i64().unwrap() >= 1);
        assert!(data["assigned_color"].as_i64().unwrap() <= 10);
        
        // Test Kurdish tribe
        let result = bdb_validate_tribe_from_name("احمد بارزاني").unwrap();
        let data: serde_json::Value = serde_json::from_str(&result.0.to_string()).unwrap();
        assert_eq!(data["ethnicity"], "KURDISH");
        
        // Test non-tribal
        let result = bdb_validate_tribe_from_name("علي حسن").unwrap();
        let data: serde_json::Value = serde_json::from_str(&result.0.to_string()).unwrap();
        assert_eq!(data["tribe_name_ar"], "غير قبلي");
        assert_eq!(data["assigned_color"], 220);
    }
}
