#!/usr/bin/env python3
"""
BDBWay + NajafWay Integration
Generate 1M cemetery records with BDBWay v1.0 Sovereign Identity
"""

import random
import csv
import json
import uuid
import hashlib
from datetime import datetime, timedelta
from pathlib import Path

# ============================================================
# Configuration
# ============================================================
TOTAL_RECORDS = 1_000_000
BATCH_SIZE = 50_000  # Larger batches for faster insertion
OUTPUT_DIR = Path('./najafway_bdbway_data')

# Wadi-us-Salaam Cemetery boundaries (Najaf, Iraq)
CEMETERY_BOUNDS = {
    'min_lat': 31.9850,
    'max_lat': 32.0150,
    'min_lon': 44.3050,
    'max_lon': 44.3450
}

# BDBWay Tribe IDs
TRIBE_NAJAF = 1001  # NajafWay Tribe
TRIBE_CEMETERY = 1002  # Cemetery Management
TRIBE_HISTORICAL = 1003  # Historical Records

# ============================================================
# Iraqi/Arabic Name Components
# ============================================================
FIRST_NAMES_MALE = [
    'محمد', 'علي', 'حسن', 'حسين', 'عباس', 'جعفر', 'موسى', 'إبراهيم',
    'أحمد', 'مصطفى', 'عمر', 'خالد', 'سعيد', 'كريم', 'رضا', 'طارق',
    'فاضل', 'صادق', 'كاظم', 'جواد', 'باقر', 'تقي', 'نقي', 'هادي'
]

FIRST_NAMES_FEMALE = [
    'فاطمة', 'زينب', 'مريم', 'خديجة', 'عائشة', 'سكينة', 'رقية', 'أم كلثوم',
    'نور', 'سارة', 'ليلى', 'هدى', 'نادية', 'سميرة', 'لبنى', 'رباب'
]

FATHER_NAMES = [
    'عبد الله', 'عبد الرحمن', 'عبد الكريم', 'عبد العزيز', 'عبد الحميد',
    'علي', 'حسن', 'حسين', 'محمد', 'أحمد', 'إبراهيم', 'موسى', 'عيسى'
]

FAMILY_NAMES = [
    'الموسوي', 'الحسيني', 'العلوي', 'الهاشمي', 'الطائي', 'الكعبي',
    'الربيعي', 'الجبوري', 'الدليمي', 'العبيدي', 'النعيمي', 'الشمري'
]

GRAVE_ZONES = [
    'المنطقة الشمالية', 'المنطقة الجنوبية', 'المنطقة الشرقية', 'المنطقة الغربية',
    'منطقة العلماء', 'منطقة الشهداء', 'المنطقة الحديثة', 'المنطقة القديمة',
    'قسم الأطفال', 'قسم النساء', 'القسم العام', 'قسم الأسر'
]

# ============================================================
# BDBWay Identity Generation
# ============================================================

def generate_bdbway_identity(person_id: int, quality_score: int) -> bytes:
    """
    Generate 16-byte BDBWay identity
    [0..7]   - UUID Partial (Root)
    [8..11]  - Tribe ID
    [12]     - Red (Domain: 100=Cemetery)
    [13]     - Green (Quality: ISO-25012)
    [14]     - Blue (Temporal: year offset)
    [15]     - Flags (0x00 = active)
    """
    # Generate deterministic UUID from person_id
    uuid_seed = f"najafway-{person_id}"
    full_uuid = uuid.uuid5(uuid.NAMESPACE_DNS, uuid_seed)
    
    identity = bytearray(16)
    
    # UUID root (first 8 bytes)
    identity[0:8] = full_uuid.bytes[0:8]
    
    # Tribe ID (4 bytes)
    tribe_bytes = TRIBE_NAJAF.to_bytes(4, byteorder='big')
    identity[8:12] = tribe_bytes
    
    # Color channels
    identity[12] = 100  # Red: Cemetery domain
    identity[13] = min(255, max(0, quality_score))  # Green: Quality
    identity[14] = 75   # Blue: Historical temporal marker
    identity[15] = 0x00  # Flags: Active record
    
    return bytes(identity)

def calculate_record_quality(record_data: dict) -> int:
    """
    ISO-25012 Quality Score
    Completeness (40%) + Validity (30%) + Accuracy (30%)
    Returns 0-255
    """
    completeness = 0.0
    validity = 0.0
    accuracy = 0.0
    
    # Completeness: Check required fields
    required_fields = ['full_name', 'death_date', 'grave_location']
    filled_fields = sum(1 for f in required_fields if record_data.get(f))
    completeness = filled_fields / len(required_fields)
    
    # Validity: Check data format
    if record_data.get('death_date'):
        try:
            death_year = int(str(record_data['death_date'])[:4])
            validity = 1.0 if 1900 <= death_year <= 2025 else 0.5
        except:
            validity = 0.3
    
    # Accuracy: Coordinate precision
    if record_data.get('grave_latitude') and record_data.get('grave_longitude'):
        accuracy = 0.9  # High confidence for generated coordinates
    else:
        accuracy = 0.5
    
    # Calculate weighted score
    score = (completeness * 0.4) + (validity * 0.3) + (accuracy * 0.3)
    return int(score * 255)

# ============================================================
# Data Generation
# ============================================================

def generate_arabic_name(gender='male'):
    """Generate realistic Iraqi Arabic name"""
    if gender == 'male':
        first_name = random.choice(FIRST_NAMES_MALE)
    else:
        first_name = random.choice(FIRST_NAMES_FEMALE)
    
    father_name = random.choice(FATHER_NAMES)
    family_name = random.choice(FAMILY_NAMES)
    
    return f"{first_name} {father_name} {family_name}"

def generate_death_date():
    """Generate death date between 1950-2025"""
    start_date = datetime(1950, 1, 1)
    end_date = datetime(2025, 12, 31)
    
    time_between = end_date - start_date
    days_between = time_between.days
    random_days = random.randint(0, days_between)
    
    return start_date + timedelta(days=random_days)

def generate_grave_location():
    """Generate grave coordinates within Wadi-us-Salaam"""
    lat = random.uniform(CEMETERY_BOUNDS['min_lat'], CEMETERY_BOUNDS['max_lat'])
    lon = random.uniform(CEMETERY_BOUNDS['min_lon'], CEMETERY_BOUNDS['max_lon'])
    elevation = random.uniform(30, 50)  # Najaf elevation
    
    zone = random.choice(GRAVE_ZONES)
    row = random.randint(1, 1000)
    plot = random.randint(1, 500)
    grave_number = f"{row:04d}-{plot:03d}"
    
    return {
        'zone': zone,
        'grave_number': grave_number,
        'latitude': round(lat, 7),
        'longitude': round(lon, 7),
        'elevation': round(elevation, 2)
    }

def generate_person_record(person_id: int) -> dict:
    """Generate complete person record with BDBWay identity"""
    gender = random.choice(['male', 'female'])
    full_name = generate_arabic_name(gender)
    
    death_date = generate_death_date()
    grave_loc = generate_grave_location()
    
    # Birth year (age 1-100)
    age = random.randint(1, 100)
    birth_year = death_date.year - age
    
    # Generate stable UUID
    stable_uuid = str(uuid.uuid5(uuid.NAMESPACE_DNS, f"najafway-{person_id}"))
    
    # Build record data for quality calculation
    record_data = {
        'full_name': full_name,
        'death_date': death_date,
        'grave_location': grave_loc,
        'grave_latitude': grave_loc['latitude'],
        'grave_longitude': grave_loc['longitude']
    }
    
    # Calculate quality score
    quality_score = calculate_record_quality(record_data)
    
    # Generate BDBWay identity
    bdb_identity = generate_bdbway_identity(person_id, quality_score)
    
    return {
        'id': person_id,
        'bdb_identity': bdb_identity.hex(),
        'stable_uuid': stable_uuid,
        'full_name_arabic': full_name,
        'gender': gender,
        'birth_year': birth_year,
        'age_at_death': age,
        'death_date': death_date.strftime('%Y-%m-%d'),
        'death_datetime': death_date.isoformat(),
        'grave_zone': grave_loc['zone'],
        'grave_number': grave_loc['grave_number'],
        'grave_latitude': grave_loc['latitude'],
        'grave_longitude': grave_loc['longitude'],
        'grave_elevation': grave_loc['elevation'],
        'quality_score': quality_score,
        'tribe_id': TRIBE_NAJAF,
        'created_at': datetime.now().isoformat()
    }

# ============================================================
# Batch Generation
# ============================================================

def generate_csv_batch(start_id: int, batch_size: int, filename: Path):
    """Generate CSV batch"""
    print(f"  📝 Generating CSV batch {start_id} to {start_id + batch_size - 1}...")
    
    with open(filename, 'w', encoding='utf-8', newline='') as csvfile:
        fieldnames = [
            'id', 'bdb_identity', 'stable_uuid', 'full_name_arabic', 'gender',
            'birth_year', 'age_at_death', 'death_date', 'death_datetime',
            'grave_zone', 'grave_number', 'grave_latitude', 'grave_longitude',
            'grave_elevation', 'quality_score', 'tribe_id', 'created_at'
        ]
        
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        
        for i in range(start_id, start_id + batch_size):
            record = generate_person_record(i)
            writer.writerow(record)
    
    print(f"  ✓ Saved CSV: {filename}")

def generate_sql_batch(start_id: int, batch_size: int, filename: Path):
    """Generate SQL INSERT batch with BDBWay integration"""
    print(f"  📝 Generating SQL batch {start_id} to {start_id + batch_size - 1}...")
    
    with open(filename, 'w', encoding='utf-8') as sqlfile:
        sqlfile.write("""-- NajafWay + BDBWay v1.0 Integration
-- Cemetery Records with Sovereign Identity

-- Insert into BDBWay spatial fabric
INSERT INTO spatial.fabric_spatial_quads (node_id, stable_uuid, position, data) VALUES
""")
        
        records = []
        for i in range(start_id, start_id + batch_size):
            record = generate_person_record(i)
            
            # SQL VALUES format
            sql_value = f"""(
    decode('{record['bdb_identity']}', 'hex'),
    '{record['stable_uuid']}',
    ARRAY[{record['grave_longitude']}, {record['grave_latitude']}, {record['grave_elevation']}]::real[],
    '{{"name": "{record['full_name_arabic']}", "zone": "{record['grave_zone']}", "grave": "{record['grave_number']}", "death": "{record['death_date']}", "quality": {record['quality_score']}}}'::jsonb
)"""
            records.append(sql_value)
        
        sqlfile.write(',\n'.join(records))
        sqlfile.write(';\n')
    
    print(f"  ✓ Saved SQL: {filename}")

# ============================================================
# Main Generation
# ============================================================

def generate_all_data():
    """Generate 1 million records in batches"""
    OUTPUT_DIR.mkdir(exist_ok=True)
    
    print("=" * 70)
    print("  BDBWay + NajafWay: Cemetery Records Generator")
    print("=" * 70)
    print(f"  Total records: {TOTAL_RECORDS:,}")
    print(f"  Batch size: {BATCH_SIZE:,}")
    print(f"  Output directory: {OUTPUT_DIR}")
    print(f"  Cemetery: Wadi-us-Salaam, Najaf, Iraq")
    print("=" * 70)
    
    num_batches = TOTAL_RECORDS // BATCH_SIZE
    
    for batch_num in range(num_batches):
        start_id = batch_num * BATCH_SIZE
        
        print(f"\n📦 Batch {batch_num + 1}/{num_batches}")
        
        # CSV
        csv_filename = OUTPUT_DIR / f"najafway_batch_{batch_num + 1:03d}.csv"
        generate_csv_batch(start_id, BATCH_SIZE, csv_filename)
        
        # SQL (every batch for bulk insert)
        sql_filename = OUTPUT_DIR / f"najafway_insert_{batch_num + 1:03d}.sql"
        generate_sql_batch(start_id, BATCH_SIZE, sql_filename)
    
    # Create master import script
    create_master_import_script(num_batches)
    
    print("\n" + "=" * 70)
    print("  ✅ Generation Complete!")
    print("=" * 70)
    print(f"  Total records: {TOTAL_RECORDS:,}")
    print(f"  CSV files: {num_batches}")
    print(f"  SQL files: {num_batches}")
    print(f"\n  📁 Files in: {OUTPUT_DIR}/")
    print(f"  📊 Next: Run import_all_najafway.sh")
    print("=" * 70)

def create_master_import_script(num_batches: int):
    """Create shell script to import all batches"""
    script_path = OUTPUT_DIR / 'import_all_najafway.sh'
    
    with open(script_path, 'w') as f:
        f.write("""#!/bin/bash
# BDBWay + NajafWay: Master Import Script

echo "======================================"
echo " Importing 1M Cemetery Records"
echo "======================================"

DB_NAME="bdbway_extension"
PGHOST="/home/akkad/.pgrx"
PGPORT="28816"

export PGHOST PGPORT

""")
        
        for i in range(1, num_batches + 1):
            f.write(f"""
echo "Batch {i}/{num_batches}..."
psql -d $DB_NAME -f najafway_insert_{i:03d}.sql
""")
        
        f.write("""
echo "======================================"
echo " ✅ Import Complete!"
echo "======================================"

# Show summary
psql -d $DB_NAME -c "SELECT COUNT(*) as total_records FROM spatial.fabric_spatial_quads WHERE data->>'zone' IS NOT NULL;"
psql -d $DB_NAME -c "SELECT data->>'zone' as zone, COUNT(*) FROM spatial.fabric_spatial_quads WHERE data->>'zone' IS NOT NULL GROUP BY 1 ORDER BY 2 DESC LIMIT 10;"
""")
    
    script_path.chmod(0o755)
    print(f"\n  ✓ Created master import script: {script_path}")

# ============================================================
# Run
# ============================================================

if __name__ == "__main__":
    print("""
    ╔══════════════════════════════════════════════════════════╗
    ║   BDBWay v1.0 + NajafWay Integration                    ║
    ║   Generate 1 Million Cemetery Records                   ║
    ║   With Sovereign Identity & Spatial Fabric              ║
    ╚══════════════════════════════════════════════════════════╝
    """)
    
    response = input("Generate 1 million records? (yes/no): ")
    
    if response.lower() in ['yes', 'y']:
        generate_all_data()
        print("\n🎉 Ready to import into PostgreSQL!")
        print("\n📋 Next steps:")
        print(f"   cd {OUTPUT_DIR}")
        print("   ./import_all_najafway.sh")
    else:
        print("Cancelled.")
