#!/bin/bash
# ============================================================
# BDBWay StormWay: Import 1 Million Najaf Cemetery Records
# ============================================================

set -e

DB_NAME="bdbway_extension"
export PGHOST="/home/akkad/.pgrx"
export PGPORT="28816"

DATA_DIR="/workspace/scripts/najaf_cemetery_data"
TOTAL_BATCHES=38

echo "============================================================"
echo " BDBWay StormWay Ingestor"
echo " Importing 1 Million Cemetery Records"
echo "============================================================"
echo " Database: $DB_NAME"
echo " Data Directory: $DATA_DIR"
echo " Total Batches: $TOTAL_BATCHES"
echo "============================================================"
echo ""

# Record start time
start_time=$(date +%s)

# Check if table exists and get initial count
echo "📊 Checking initial record count..."
initial_count=$(psql -d $DB_NAME -t -c "SELECT COUNT(*) FROM spatial.fabric_spatial_quads WHERE data->>'full_name_arabic' IS NOT NULL;")
echo "   Current records: $initial_count"
echo ""

# Import each batch
total_imported=0
total_errors=0

for i in $(seq 6 38); do
    batch_file="$DATA_DIR/najaf_cemetery_batch_$(printf '%03d' $i).csv"
    
    if [ ! -f "$batch_file" ]; then
        echo "⚠️  Batch $i: File not found - $batch_file"
        continue
    fi
    
    echo "📦 Batch $i/$TOTAL_BATCHES: $(basename $batch_file)"
    
    # Import batch and capture result
    result=$(psql -d $DB_NAME -t -c "SELECT bdb_storm_ingest_csv('$batch_file');" 2>&1)
    
    if [ $? -eq 0 ]; then
        records=$(echo "$result" | grep -oP '\d+' | tail -1)
        total_imported=$((total_imported + records))
        echo "   ✅ Imported: $records records (Total: $total_imported)"
    else
        total_errors=$((total_errors + 1))
        echo "   ❌ Error importing batch $i"
        echo "   Error: $result"
    fi
    
    # Show progress every 5 batches
    if [ $((i % 5)) -eq 0 ]; then
        current_count=$(psql -d $DB_NAME -t -c "SELECT COUNT(*) FROM spatial.fabric_spatial_quads WHERE data->>'full_name_arabic' IS NOT NULL;")
        echo ""
        echo "   📈 Progress: $current_count total records in database"
        echo ""
    fi
done

# Calculate duration
end_time=$(date +%s)
duration=$((end_time - start_time))
minutes=$((duration / 60))
seconds=$((duration % 60))

# Final statistics
echo ""
echo "============================================================"
echo " ✅ Import Complete!"
echo "============================================================"

final_count=$(psql -d $DB_NAME -t -c "SELECT COUNT(*) FROM spatial.fabric_spatial_quads WHERE data->>'full_name_arabic' IS NOT NULL;")
new_records=$((final_count - initial_count))

echo " Duration: ${minutes}m ${seconds}s"
echo " Initial records: $initial_count"
echo " Final records: $final_count"
echo " New records imported: $new_records"
echo " Batches processed: $TOTAL_BATCHES"
echo " Errors: $total_errors"
echo "============================================================"
echo ""

# Display database statistics
echo "📊 Database Statistics:"
echo "============================================================"

psql -d $DB_NAME <<SQL
-- Total records
SELECT 
    'Total Cemetery Records' as metric,
    COUNT(*) as value
FROM spatial.fabric_spatial_quads
WHERE data->>'full_name_arabic' IS NOT NULL;

-- Zone distribution
SELECT 
    'Zone Distribution' as category,
    data->>'residence_city' as zone,
    COUNT(*) as count
FROM spatial.fabric_spatial_quads
WHERE data->>'full_name_arabic' IS NOT NULL
GROUP BY data->>'residence_city'
ORDER BY count DESC
LIMIT 10;

-- Database size
SELECT 
    'Database Size' as metric,
    pg_size_pretty(pg_database_size('$DB_NAME')) as value;

-- Table size
SELECT 
    'Table Size' as metric,
    pg_size_pretty(pg_total_relation_size('spatial.fabric_spatial_quads')) as value;
SQL

echo ""
echo "============================================================"
echo " 🎉 1 Million Records Successfully Imported!"
echo "============================================================"
echo ""
echo "📋 Next Steps:"
echo "   1. Test queries: psql -d $DB_NAME"
echo "   2. View data: SELECT * FROM najafway_zone_stats;"
echo "   3. Search names: SELECT * FROM najafway_search_by_name('محمد', 10);"
echo "   4. Export GeoJSON for web visualization"
echo ""
echo "============================================================"
