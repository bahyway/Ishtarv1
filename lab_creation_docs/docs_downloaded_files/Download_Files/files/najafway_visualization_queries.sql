-- ============================================================
-- NajafWay Visualization Queries for Web Frontend
-- BDBWay v1.0 Spatial Fabric Integration
-- ============================================================

-- 1. Get all graves in viewport (for map rendering)
CREATE OR REPLACE FUNCTION najafway_get_viewport_graves(
    min_lon REAL,
    min_lat REAL,
    max_lon REAL,
    max_lat REAL,
    max_results INT DEFAULT 1000
)
RETURNS TABLE (
    grave_id TEXT,
    name TEXT,
    zone TEXT,
    grave_number TEXT,
    death_date TEXT,
    longitude REAL,
    latitude REAL,
    elevation REAL,
    quality_score INT,
    classification TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        encode(f.node_id, 'hex') as grave_id,
        f.data->>'name' as name,
        f.data->>'zone' as zone,
        f.data->>'grave' as grave_number,
        f.data->>'death' as death_date,
        f.position[1] as longitude,
        f.position[2] as latitude,
        f.position[3] as elevation,
        (f.data->>'quality')::INT as quality_score,
        bdb_classify_node(f.node_id) as classification
    FROM spatial.fabric_spatial_quads f
    WHERE 
        f.position[1] BETWEEN min_lon AND max_lon
        AND f.position[2] BETWEEN min_lat AND max_lat
        AND f.data->>'zone' IS NOT NULL
    ORDER BY (f.data->>'quality')::INT DESC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- 2. Search graves by name (Arabic text search)
CREATE OR REPLACE FUNCTION najafway_search_by_name(
    search_term TEXT,
    max_results INT DEFAULT 50
)
RETURNS TABLE (
    grave_id TEXT,
    name TEXT,
    zone TEXT,
    grave_number TEXT,
    death_date TEXT,
    longitude REAL,
    latitude REAL,
    quality_score INT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        encode(f.node_id, 'hex') as grave_id,
        f.data->>'name' as name,
        f.data->>'zone' as zone,
        f.data->>'grave' as grave_number,
        f.data->>'death' as death_date,
        f.position[1] as longitude,
        f.position[2] as latitude,
        (f.data->>'quality')::INT as quality_score
    FROM spatial.fabric_spatial_quads f
    WHERE 
        f.data->>'name' ILIKE '%' || search_term || '%'
        AND f.data->>'zone' IS NOT NULL
    ORDER BY (f.data->>'quality')::INT DESC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- 3. Get zone statistics (for dashboard)
CREATE OR REPLACE VIEW najafway_zone_stats AS
SELECT 
    data->>'zone' as zone_name,
    COUNT(*) as total_graves,
    AVG((data->>'quality')::INT) as avg_quality,
    MIN(data->>'death') as earliest_burial,
    MAX(data->>'death') as latest_burial,
    ST_Centroid(
        ST_Collect(
            ST_MakePoint(position[1], position[2])
        )
    ) as zone_center
FROM spatial.fabric_spatial_quads
WHERE data->>'zone' IS NOT NULL
GROUP BY data->>'zone'
ORDER BY total_graves DESC;

-- 4. Get quality distribution (for analytics)
CREATE OR REPLACE VIEW najafway_quality_distribution AS
SELECT 
    CASE 
        WHEN (data->>'quality')::INT >= 200 THEN 'Sovereign (200-255)'
        WHEN (data->>'quality')::INT >= 140 THEN 'Active (140-199)'
        WHEN (data->>'quality')::INT >= 100 THEN 'Poor (100-139)'
        ELSE 'Non-Active (0-99)'
    END as quality_tier,
    COUNT(*) as record_count,
    ROUND(AVG((data->>'quality')::INT), 2) as avg_quality
FROM spatial.fabric_spatial_quads
WHERE data->>'zone' IS NOT NULL
GROUP BY 
    CASE 
        WHEN (data->>'quality')::INT >= 200 THEN 'Sovereign (200-255)'
        WHEN (data->>'quality')::INT >= 140 THEN 'Active (140-199)'
        WHEN (data->>'quality')::INT >= 100 THEN 'Poor (100-139)'
        ELSE 'Non-Active (0-99)'
    END
ORDER BY avg_quality DESC;

-- 5. Find nearest graves to a point (for "find nearby" feature)
CREATE OR REPLACE FUNCTION najafway_find_nearest_graves(
    target_lon REAL,
    target_lat REAL,
    radius_km REAL DEFAULT 0.5,
    max_results INT DEFAULT 20
)
RETURNS TABLE (
    grave_id TEXT,
    name TEXT,
    zone TEXT,
    longitude REAL,
    latitude REAL,
    distance_km REAL,
    quality_score INT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        encode(f.node_id, 'hex') as grave_id,
        f.data->>'name' as name,
        f.data->>'zone' as zone,
        f.position[1] as longitude,
        f.position[2] as latitude,
        bdb_calculate_proximity(
            ARRAY[target_lon, target_lat]::real[],
            ARRAY[f.position[1], f.position[2]]::real[]
        ) as distance_km,
        (f.data->>'quality')::INT as quality_score
    FROM spatial.fabric_spatial_quads f
    WHERE 
        f.data->>'zone' IS NOT NULL
        AND bdb_calculate_proximity(
            ARRAY[target_lon, target_lat]::real[],
            ARRAY[f.position[1], f.position[2]]::real[]
        ) <= radius_km
    ORDER BY distance_km ASC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- 6. Heatmap data (grave density for visualization)
CREATE OR REPLACE FUNCTION najafway_heatmap_data(
    grid_size REAL DEFAULT 0.001  -- ~100m grid cells
)
RETURNS TABLE (
    cell_lon REAL,
    cell_lat REAL,
    grave_count BIGINT,
    avg_quality NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        ROUND((position[1] / grid_size)::NUMERIC, 0) * grid_size as cell_lon,
        ROUND((position[2] / grid_size)::NUMERIC, 0) * grid_size as cell_lat,
        COUNT(*) as grave_count,
        ROUND(AVG((data->>'quality')::INT), 2) as avg_quality
    FROM spatial.fabric_spatial_quads
    WHERE data->>'zone' IS NOT NULL
    GROUP BY 
        ROUND((position[1] / grid_size)::NUMERIC, 0) * grid_size,
        ROUND((position[2] / grid_size)::NUMERIC, 0) * grid_size
    HAVING COUNT(*) > 0
    ORDER BY grave_count DESC;
END;
$$ LANGUAGE plpgsql;

-- 7. Timeline data (burials over time)
CREATE OR REPLACE VIEW najafway_burial_timeline AS
SELECT 
    EXTRACT(YEAR FROM (data->>'death')::DATE) as year,
    COUNT(*) as burial_count,
    data->>'zone' as zone
FROM spatial.fabric_spatial_quads
WHERE 
    data->>'zone' IS NOT NULL
    AND data->>'death' IS NOT NULL
GROUP BY 
    EXTRACT(YEAR FROM (data->>'death')::DATE),
    data->>'zone'
ORDER BY year DESC;

-- 8. Export for GeoJSON (for web mapping libraries)
CREATE OR REPLACE FUNCTION najafway_geojson_export(
    min_lon REAL,
    min_lat REAL,
    max_lon REAL,
    max_lat REAL,
    max_features INT DEFAULT 500
)
RETURNS JSON AS $$
DECLARE
    result JSON;
BEGIN
    SELECT json_build_object(
        'type', 'FeatureCollection',
        'features', json_agg(
            json_build_object(
                'type', 'Feature',
                'id', encode(node_id, 'hex'),
                'geometry', json_build_object(
                    'type', 'Point',
                    'coordinates', json_build_array(position[1], position[2])
                ),
                'properties', json_build_object(
                    'name', data->>'name',
                    'zone', data->>'zone',
                    'grave_number', data->>'grave',
                    'death_date', data->>'death',
                    'quality', (data->>'quality')::INT,
                    'classification', bdb_classify_node(node_id)
                )
            )
        )
    )
    INTO result
    FROM spatial.fabric_spatial_quads
    WHERE 
        position[1] BETWEEN min_lon AND max_lon
        AND position[2] BETWEEN min_lat AND max_lat
        AND data->>'zone' IS NOT NULL
    LIMIT max_features;
    
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- Sample Queries for Testing
-- ============================================================

-- Example 1: Get all graves in Wadi-us-Salaam viewport
-- SELECT * FROM najafway_get_viewport_graves(44.305, 31.985, 44.345, 32.015, 100);

-- Example 2: Search for "محمد"
-- SELECT * FROM najafway_search_by_name('محمد', 20);

-- Example 3: Zone statistics
-- SELECT * FROM najafway_zone_stats;

-- Example 4: Quality distribution
-- SELECT * FROM najafway_quality_distribution;

-- Example 5: Find nearest graves to Imam Ali Shrine
-- SELECT * FROM najafway_find_nearest_graves(44.3146, 32.0286, 1.0, 10);

-- Example 6: Heatmap data
-- SELECT * FROM najafway_heatmap_data(0.001);

-- Example 7: Burial timeline
-- SELECT * FROM najafway_burial_timeline WHERE year >= 2000 ORDER BY year DESC;

-- Example 8: GeoJSON export
-- SELECT najafway_geojson_export(44.305, 31.985, 44.345, 32.015, 100);
