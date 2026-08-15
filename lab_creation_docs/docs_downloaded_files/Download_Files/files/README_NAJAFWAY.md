# NajafWay + BDBWay v1.0: Stress Test Suite

## 🎯 Overview

This stress test suite generates **1 MILLION cemetery records** for the NajafWay application, fully integrated with BDBWay v1.0's Sovereign Identity system and spatial fabric.

### What's Included

1. **najafway_bdbway_generator.py** - Python script to generate 1M records with:
   - BDBWay 16-byte deterministic identities
   - ISO-25012 quality scores (Green channel)
   - Real Wadi-us-Salaam cemetery coordinates (Najaf, Iraq)
   - Arabic names, dates, and locations
   - Batch processing (50k records per batch)

2. **najafway_visualization_queries.sql** - PostgreSQL functions for web visualization:
   - Viewport queries (for map rendering)
   - Name search (Arabic text search)
   - Zone statistics and analytics
   - Nearest neighbor search
   - Heatmap data generation
   - GeoJSON export
   - Timeline visualization

3. **setup_najafway_stress_test.sh** - Automated setup script that:
   - Installs Python dependencies
   - Creates visualization functions
   - Generates 1M records
   - Imports data into PostgreSQL
   - Creates performance indexes
   - Runs test queries

## 🚀 Quick Start

### Prerequisites

- RustLAB Docker container (running)
- PostgreSQL with BDBWay extension installed
- Python 3 with pip
- ~5GB free disk space

### Installation

```bash
# 1. Make scripts executable
chmod +x setup_najafway_stress_test.sh najafway_bdbway_generator.py

# 2. Run the complete setup
./setup_najafway_stress_test.sh
```

The setup script will guide you through:
1. Installing dependencies (faker, arabic-reshaper, python-bidi)
2. Creating visualization functions
3. Generating 1M records (5-10 minutes)
4. Importing data (10-20 minutes)
5. Creating indexes
6. Running test queries

## 📊 Generated Data Structure

### BDBWay Identity (16 bytes)
```
[0..7]   - UUID Partial (deterministic from person_id)
[8..11]  - Tribe ID (1001 = NajafWay)
[12]     - Red Channel (100 = Cemetery domain)
[13]     - Green Channel (ISO-25012 quality: 0-255)
[14]     - Blue Channel (75 = Historical temporal marker)
[15]     - Flags (0x00 = Active record)
```

### Record Fields
- **Identity**: 16-byte BDBWay identity + stable UUID
- **Personal**: Arabic name, gender, birth year, age at death
- **Death Info**: Date, datetime (1950-2025)
- **Grave Location**: Zone, number, lat/lon/elevation
- **Quality**: ISO-25012 score based on completeness, validity, accuracy
- **Spatial**: Real coordinates within Wadi-us-Salaam bounds

## 🗺️ Cemetery Coordinates

**Wadi-us-Salaam Cemetery, Najaf, Iraq**
- Latitude: 31.9850° to 32.0150° N
- Longitude: 44.3050° to 44.3450° E
- Elevation: 30-50 meters
- Area: World's largest cemetery (~6 million burials)

## 📈 Visualization Functions

### 1. Viewport Query (for map rendering)
```sql
SELECT * FROM najafway_get_viewport_graves(
    44.305,  -- min_lon
    31.985,  -- min_lat
    44.345,  -- max_lon
    32.015,  -- max_lat
    1000     -- max_results
);
```

Returns: grave_id, name, zone, grave_number, death_date, lon/lat, quality, classification

### 2. Name Search (Arabic text)
```sql
SELECT * FROM najafway_search_by_name('محمد', 50);
```

### 3. Zone Statistics
```sql
SELECT * FROM najafway_zone_stats;
```

Returns: zone_name, total_graves, avg_quality, earliest/latest burial, zone_center

### 4. Quality Distribution
```sql
SELECT * FROM najafway_quality_distribution;
```

Returns quality tiers:
- Sovereign (200-255) - High quality, complete records
- Active (140-199) - Good quality
- Poor (100-139) - Incomplete data
- Non-Active (0-99) - Low quality

### 5. Find Nearest Graves
```sql
-- Near Imam Ali Shrine (44.3146°E, 32.0286°N)
SELECT * FROM najafway_find_nearest_graves(
    44.3146,  -- target_lon
    32.0286,  -- target_lat
    1.0,      -- radius_km
    20        -- max_results
);
```

### 6. Heatmap Data
```sql
SELECT * FROM najafway_heatmap_data(0.001);  -- ~100m grid
```

### 7. Timeline Data
```sql
SELECT * FROM najafway_burial_timeline WHERE year >= 2000;
```

### 8. GeoJSON Export (for Leaflet/Mapbox)
```sql
SELECT najafway_geojson_export(44.305, 31.985, 44.345, 32.015, 500);
```

## 🔍 Sample Queries

### Get high-quality graves in specific zone
```sql
SELECT 
    encode(node_id, 'hex') as id,
    data->>'name' as name,
    data->>'grave' as grave_number,
    (data->>'quality')::INT as quality
FROM spatial.fabric_spatial_quads
WHERE 
    data->>'zone' = 'منطقة العلماء'
    AND (data->>'quality')::INT >= 200
LIMIT 20;
```

### Count graves by decade
```sql
SELECT 
    FLOOR(EXTRACT(YEAR FROM (data->>'death')::DATE) / 10) * 10 as decade,
    COUNT(*) as grave_count
FROM spatial.fabric_spatial_quads
WHERE data->>'zone' IS NOT NULL
GROUP BY decade
ORDER BY decade DESC;
```

### Find graves with specific names
```sql
SELECT 
    data->>'name' as name,
    data->>'zone' as zone,
    data->>'grave' as grave_number,
    position[1] as lon,
    position[2] as lat
FROM spatial.fabric_spatial_quads
WHERE data->>'name' LIKE '%علي%'
LIMIT 10;
```

## 🌐 Web Visualization Integration

### Leaflet.js Example
```javascript
// Fetch viewport data
const response = await fetch(`/api/najafway/viewport?` + new URLSearchParams({
    min_lon: 44.305,
    min_lat: 31.985,
    max_lon: 44.345,
    max_lat: 32.015,
    max_results: 1000
}));

const graves = await response.json();

// Add markers to map
graves.forEach(grave => {
    const marker = L.marker([grave.latitude, grave.longitude])
        .bindPopup(`
            <b>${grave.name}</b><br>
            Zone: ${grave.zone}<br>
            Grave: ${grave.grave_number}<br>
            Death: ${grave.death_date}<br>
            Quality: ${grave.quality_score}
        `)
        .addTo(map);
        
    // Color by quality
    if (grave.quality_score >= 200) {
        marker.setIcon(goldenIcon);  // Sovereign
    } else if (grave.quality_score >= 140) {
        marker.setIcon(greenIcon);   // Active
    } else {
        marker.setIcon(yellowIcon);  // Poor quality
    }
});
```

### GeoJSON Layer Example
```javascript
// Fetch GeoJSON
const geojson = await fetch('/api/najafway/geojson?...');
const data = await geojson.json();

// Add to map
L.geoJSON(data, {
    pointToLayer: (feature, latlng) => {
        return L.circleMarker(latlng, {
            radius: 5,
            fillColor: getColorByQuality(feature.properties.quality),
            color: "#000",
            weight: 1,
            opacity: 1,
            fillOpacity: 0.8
        });
    },
    onEachFeature: (feature, layer) => {
        layer.bindPopup(`
            <b>${feature.properties.name}</b><br>
            ${feature.properties.grave_number}
        `);
    }
}).addTo(map);
```

## 📊 Performance Benchmarks

Expected performance with 1M records:

- **Viewport Query** (1000 results): ~50-100ms
- **Name Search** (50 results): ~100-200ms
- **Zone Statistics**: ~200-500ms
- **Nearest Neighbor** (20 results): ~100-200ms
- **Heatmap Generation**: ~500ms-1s
- **GeoJSON Export** (500 features): ~100-300ms

Indexes created automatically for optimal performance:
- Quality score index
- Zone index
- Name trigram index (for fuzzy search)
- Spatial indexes on coordinates

## 🎨 Quality Tier Visualization

Use these color codes for web visualization:

```javascript
function getColorByQuality(quality) {
    if (quality >= 200) return '#FFD700';  // Gold - Sovereign
    if (quality >= 140) return '#90EE90';  // Light Green - Active
    if (quality >= 100) return '#FFE4B5';  // Moccasin - Poor
    return '#D3D3D3';  // Light Gray - Non-Active
}
```

## 📁 Output Files

After generation, you'll have:

```
najafway_bdbway_data/
├── najafway_batch_001.csv           (50k records CSV)
├── najafway_batch_002.csv           (50k records CSV)
├── ...
├── najafway_batch_020.csv           (50k records CSV)
├── najafway_insert_001.sql          (50k SQL inserts)
├── najafway_insert_002.sql          (50k SQL inserts)
├── ...
├── najafway_insert_020.sql          (50k SQL inserts)
└── import_all_najafway.sh           (master import script)
```

Total: 20 CSV files, 20 SQL files (~2-3 GB total)

## 🔧 Troubleshooting

### PostgreSQL Connection Issues
```bash
export PGHOST="/home/akkad/.pgrx"
export PGPORT="28816"
psql -d bdbway_extension -c "SELECT 1;"
```

### Python Dependencies Failed
```bash
pip install --break-system-packages faker arabic-reshaper python-bidi
```

### Import Taking Too Long
- Normal import time: 10-20 minutes for 1M records
- Monitor progress: `watch -n 5 "psql -d bdbway_extension -c 'SELECT COUNT(*) FROM spatial.fabric_spatial_quads'"`

### Out of Disk Space
- Reduce TOTAL_RECORDS in generator script
- Or clean up after each batch import

## 📞 Integration with Web API

Create a simple API endpoint (Flask/FastAPI example):

```python
from flask import Flask, jsonify, request
import psycopg2

app = Flask(__name__)

@app.route('/api/najafway/viewport')
def get_viewport():
    min_lon = request.args.get('min_lon', type=float)
    min_lat = request.args.get('min_lat', type=float)
    max_lon = request.args.get('max_lon', type=float)
    max_lat = request.args.get('max_lat', type=float)
    max_results = request.args.get('max_results', 1000, type=int)
    
    conn = psycopg2.connect(
        host="/home/akkad/.pgrx",
        port=28816,
        database="bdbway_extension"
    )
    
    cursor = conn.cursor()
    cursor.execute("""
        SELECT * FROM najafway_get_viewport_graves(%s, %s, %s, %s, %s)
    """, (min_lon, min_lat, max_lon, max_lat, max_results))
    
    columns = [desc[0] for desc in cursor.description]
    results = [dict(zip(columns, row)) for row in cursor.fetchall()]
    
    conn.close()
    return jsonify(results)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

## 🎉 Success Criteria

After setup, verify:

1. ✅ 1 million records in `spatial.fabric_spatial_quads`
2. ✅ Quality distribution shows ~5% Sovereign (200-255)
3. ✅ All 12 cemetery zones have graves
4. ✅ Viewport queries return results in <100ms
5. ✅ GeoJSON export generates valid output
6. ✅ Name search works with Arabic text

Run verification:
```bash
psql -d bdbway_extension <<SQL
SELECT 
    'Total Records' as check,
    COUNT(*) as value,
    CASE WHEN COUNT(*) = 1000000 THEN '✅' ELSE '❌' END as status
FROM spatial.fabric_spatial_quads
WHERE data->>'zone' IS NOT NULL;
SQL
```

## 📚 Next Steps

1. **Build Web Interface**: Use Leaflet.js or Mapbox GL JS
2. **Add Search UI**: Arabic text input with autocomplete
3. **Create Dashboard**: Zone statistics, quality distribution charts
4. **Implement Filters**: By date range, zone, quality tier
5. **Add Heatmap Layer**: Density visualization
6. **Timeline Animation**: Show burials over decades
7. **3D Visualization**: Use elevation data for 3D map

---

**Created by:** Bahaa Fadam  
**Project:** BahyWay Ecosystem - NajafWay Integration  
**Date:** January 2026  
**License:** Proprietary
