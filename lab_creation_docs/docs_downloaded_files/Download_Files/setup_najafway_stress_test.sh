#!/bin/bash
# ============================================================
# NajafWay + BDBWay v1.0: Complete Setup Script
# Stress Test: 1 Million Cemetery Records
# ============================================================

set -e  # Exit on error

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DB_NAME="bdbway_extension"
export PGHOST="/home/akkad/.pgrx"
export PGPORT="28816"

echo "============================================================"
echo " BDBWay + NajafWay Setup"
echo "============================================================"
echo " Database: $DB_NAME"
echo " Host: $PGHOST"
echo " Port: $PGPORT"
echo "============================================================"

# ============================================================
# Step 1: Install Python dependencies
# ============================================================
echo ""
echo "📦 Step 1: Installing Python dependencies..."
pip install --break-system-packages faker arabic-reshaper python-bidi 2>/dev/null || {
    echo "⚠️  pip install failed, but continuing..."
}

# ============================================================
# Step 2: Create visualization functions
# ============================================================
echo ""
echo "🔧 Step 2: Creating visualization functions..."
psql -d $DB_NAME -f najafway_visualization_queries.sql

if [ $? -eq 0 ]; then
    echo "✅ Visualization functions created successfully"
else
    echo "❌ Failed to create visualization functions"
    exit 1
fi

# ============================================================
# Step 3: Generate data
# ============================================================
echo ""
echo "🏭 Step 3: Generating 1 million records..."
echo ""
read -p "Generate 1M records now? This will take 5-10 minutes (yes/no): " generate_choice

if [[ "$generate_choice" == "yes" || "$generate_choice" == "y" ]]; then
    python3 najafway_bdbway_generator.py <<EOF
yes
EOF
    
    if [ $? -eq 0 ]; then
        echo "✅ Data generation complete"
    else
        echo "❌ Data generation failed"
        exit 1
    fi
else
    echo "⏭️  Skipping data generation"
    echo "   Run manually: python3 najafway_bdbway_generator.py"
    exit 0
fi

# ============================================================
# Step 4: Import data
# ============================================================
echo ""
echo "📥 Step 4: Importing data into PostgreSQL..."
echo ""
read -p "Import data now? This will take 10-20 minutes (yes/no): " import_choice

if [[ "$import_choice" == "yes" || "$import_choice" == "y" ]]; then
    cd najafway_bdbway_data
    
    echo "Starting import..."
    start_time=$(date +%s)
    
    ./import_all_najafway.sh
    
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    
    echo ""
    echo "✅ Import complete in ${duration} seconds"
    cd ..
else
    echo "⏭️  Skipping data import"
    echo "   Run manually: cd najafway_bdbway_data && ./import_all_najafway.sh"
fi

# ============================================================
# Step 5: Create indexes for performance
# ============================================================
echo ""
echo "🚀 Step 5: Creating performance indexes..."

psql -d $DB_NAME <<SQL
-- Index on quality for fast classification queries
CREATE INDEX IF NOT EXISTS idx_najafway_quality 
ON spatial.fabric_spatial_quads (((data->>'quality')::INT))
WHERE data->>'zone' IS NOT NULL;

-- Index on zone for aggregation queries
CREATE INDEX IF NOT EXISTS idx_najafway_zone 
ON spatial.fabric_spatial_quads ((data->>'zone'))
WHERE data->>'zone' IS NOT NULL;

-- Index on name for text search
CREATE INDEX IF NOT EXISTS idx_najafway_name 
ON spatial.fabric_spatial_quads USING gin ((data->>'name') gin_trgm_ops);

-- Create trigram extension if needed
CREATE EXTENSION IF NOT EXISTS pg_trgm;

ANALYZE spatial.fabric_spatial_quads;

SELECT 'Indexes created successfully' as status;
SQL

if [ $? -eq 0 ]; then
    echo "✅ Performance indexes created"
else
    echo "⚠️  Some indexes may have failed, but continuing..."
fi

# ============================================================
# Step 6: Display statistics
# ============================================================
echo ""
echo "============================================================"
echo " 📊 Database Statistics"
echo "============================================================"

psql -d $DB_NAME <<SQL
-- Total records
SELECT 
    'Total Cemetery Records' as metric,
    COUNT(*) as value
FROM spatial.fabric_spatial_quads
WHERE data->>'zone' IS NOT NULL;

-- Quality distribution
SELECT 
    'Quality Tier' as category,
    quality_tier,
    record_count
FROM najafway_quality_distribution;

-- Top zones
SELECT 
    'Top Zones' as category,
    zone_name,
    total_graves
FROM najafway_zone_stats
LIMIT 5;

-- Database size
SELECT 
    'Database Size' as metric,
    pg_size_pretty(pg_database_size('$DB_NAME')) as value;

-- Table size
SELECT 
    'fabric_spatial_quads Size' as metric,
    pg_size_pretty(pg_total_relation_size('spatial.fabric_spatial_quads')) as value;
SQL

# ============================================================
# Step 7: Test queries
# ============================================================
echo ""
echo "============================================================"
echo " 🧪 Testing Queries"
echo "============================================================"

echo ""
echo "Test 1: Viewport query (100 records)..."
psql -d $DB_NAME -c "SELECT COUNT(*) as graves_found FROM najafway_get_viewport_graves(44.305, 31.985, 44.345, 32.015, 100);"

echo ""
echo "Test 2: Name search (محمد)..."
psql -d $DB_NAME -c "SELECT name, zone, death_date FROM najafway_search_by_name('محمد', 5);"

echo ""
echo "Test 3: Nearest graves to Imam Ali Shrine..."
psql -d $DB_NAME -c "SELECT name, zone, ROUND(distance_km::NUMERIC, 4) as distance_km FROM najafway_find_nearest_graves(44.3146, 32.0286, 1.0, 5);"

# ============================================================
# Success!
# ============================================================
echo ""
echo "============================================================"
echo " ✅ Setup Complete!"
echo "============================================================"
echo ""
echo "📋 Next Steps:"
echo ""
echo "1. View zone statistics:"
echo "   psql -d $DB_NAME -c 'SELECT * FROM najafway_zone_stats;'"
echo ""
echo "2. Search for graves:"
echo "   psql -d $DB_NAME -c \"SELECT * FROM najafway_search_by_name('علي', 10);\""
echo ""
echo "3. Get viewport data for map:"
echo "   psql -d $DB_NAME -c 'SELECT * FROM najafway_get_viewport_graves(44.305, 31.985, 44.345, 32.015, 100);'"
echo ""
echo "4. Export GeoJSON for web:"
echo "   psql -d $DB_NAME -c \"SELECT najafway_geojson_export(44.305, 31.985, 44.345, 32.015, 500);\" > graves.geojson"
echo ""
echo "5. View quality distribution:"
echo "   psql -d $DB_NAME -c 'SELECT * FROM najafway_quality_distribution;'"
echo ""
echo "============================================================"
echo " 🌐 Ready for Web Visualization!"
echo "============================================================"
