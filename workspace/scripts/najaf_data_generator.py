import random
import csv
import json
import os
from datetime import datetime, timedelta
from faker import Faker
import uuid

# Initialize Faker with Arabic locale
fake_ar = Faker('ar_SA')

# ============================================================
# Configuration
# ============================================================
TOTAL_RECORDS = 1_000_000
BATCH_SIZE = 10_000
OUTPUT_DIR = './najaf_cemetery_data'

# Wadi-us-Salaam Cemetery boundaries
CEMETERY_BOUNDS = {
    'min_lat': 31.9850,
    'max_lat': 32.0150,
    'min_lon': 44.3050,
    'max_lon': 44.3450
}

FIRST_NAMES_MALE = ['محمد', 'علي', 'حسن', 'حسين', 'عباس', 'جعفر', 'موسى', 'إبراهيم', 'أحمد', 'مصطفى']
FIRST_NAMES_FEMALE = ['فاطمة', 'زينب', 'مريم', 'خديجة', 'عائشة', 'سكينة', 'رقية', 'نور', 'سارة']
FAMILY_NAMES = ['الموسوي', 'الحسيني', 'العلوي', 'الهاشمي', 'الطائي', 'الكعبي', 'الربيعي', 'الجنابي']
CITIES = ['النجف الأشرف', 'الكوفة', 'الحيدرية', 'المشخاب', 'بغداد', 'كربلاء', 'البصرة']

# ============================================================
# Helper Functions
# ============================================================

def generate_person_record(record_id):
    gender = random.choice(['male', 'female'])
    first = random.choice(FIRST_NAMES_MALE if gender == 'male' else FIRST_NAMES_FEMALE)
    full_name = f"{first} {random.choice(FIRST_NAMES_MALE)} {random.choice(FAMILY_NAMES)}"

    death_datetime = datetime(1950, 1, 1) + timedelta(days=random.randint(0, 27000))
    lat = random.uniform(CEMETERY_BOUNDS['min_lat'], CEMETERY_BOUNDS['max_lat'])
    lon = random.uniform(CEMETERY_BOUNDS['min_lon'], CEMETERY_BOUNDS['max_lon'])

    return {
        'id': record_id,
        'uuid': str(uuid.uuid4()),
        'full_name_arabic': full_name,
        'gender': gender,
        'birth_year': death_datetime.year - random.randint(0, 90),
        'death_date': death_datetime.strftime('%Y-%m-%d'),
        'residence_city': random.choice(CITIES),
        'grave_latitude': round(lat, 7),
        'grave_longitude': round(lon, 7),
        'record_source': 'test12.zip',
        'notes': 'Verified by Steward' if random.random() > 0.8 else None
    }

def generate_csv_batch(start_id, batch_size, filename):
    with open(filename, 'w', encoding='utf-8-sig', newline='') as csvfile:
        fieldnames = ['id', 'uuid', 'full_name_arabic', 'gender', 'birth_year', 'death_date',
                     'residence_city', 'grave_latitude', 'grave_longitude', 'record_source', 'notes']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        for i in range(start_id, start_id + batch_size):
            writer.writerow(generate_person_record(i))

def generate_sql_batch(start_id, batch_size, filename):
    """Fixed version: Pre-calculates variables to avoid f-string nesting errors"""
    with open(filename, 'w', encoding='utf-8') as sqlfile:
        for i in range(start_id, start_id + batch_size):
            rec = generate_person_record(i)

            # Clean variables before string insertion
            name = rec['full_name_arabic'].replace("'", "''")
            notes = f"'{rec['notes']}'" if rec['notes'] else "NULL"

            sql = (
                f"INSERT INTO bdb_fabric.nodes (id, stable_uuid, tribe_id, position, data) "
                f"VALUES (bdb_generate_identity('{rec['uuid']}', 101, 125, 240, 100), "
                f"'{rec['uuid']}', 101, '[{rec['grave_longitude']}, {rec['grave_latitude']}, 0]', "
                f"'{{\"name\": \"{name}\"}}');\n"
            )
            sqlfile.write(sql)

# ============================================================
# Main Execution
# ============================================================

if __name__ == "__main__":
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    print(f"--- Firing Ingestion Storm: Generating {TOTAL_RECORDS} Records ---")

    num_batches = TOTAL_RECORDS // BATCH_SIZE
    for b in range(num_batches):
        sid = b * BATCH_SIZE
        csv_name = f"{OUTPUT_DIR}/najaf_cemetery_batch_{b+1:03d}.csv"
        generate_csv_batch(sid, BATCH_SIZE, csv_name)

        if b % 10 == 0:
            print(f"✓ Created {sid + BATCH_SIZE} records...")

    print(f"\n[SUCCESS] Files saved in {OUTPUT_DIR}")
