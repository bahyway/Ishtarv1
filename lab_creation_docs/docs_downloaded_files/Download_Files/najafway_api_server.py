#!/usr/bin/env python3
"""
NajafWay API Server
Serves data from BDBWay PostgreSQL to web visualization
"""

from flask import Flask, jsonify, request
from flask_cors import CORS
import psycopg2
import psycopg2.extras
import json

app = Flask(__name__)
CORS(app)  # Enable CORS for all routes

# Database configuration
DB_CONFIG = {
    'host': '/home/akkad/.pgrx',
    'port': 28816,
    'database': 'bdbway_extension',
    'user': 'akkad'
}

def get_db_connection():
    """Create database connection"""
    return psycopg2.connect(**DB_CONFIG)

@app.route('/api/stats', methods=['GET'])
def get_stats():
    """Get overall statistics"""
    conn = get_db_connection()
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    # Total records
    cursor.execute("""
        SELECT COUNT(*) as total_records
        FROM spatial.fabric_spatial_quads
        WHERE data->>'full_name_arabic' IS NOT NULL
    """)
    total = cursor.fetchone()['total_records']
    
    # Database size
    cursor.execute("""
        SELECT pg_size_pretty(pg_total_relation_size('spatial.fabric_spatial_quads')) as table_size
    """)
    size = cursor.fetchone()['table_size']
    
    conn.close()
    
    return jsonify({
        'total_records': total,
        'table_size': size,
        'import_time': '~20 minutes',
        'import_speed': f'{int(total / 20)} records/minute'
    })

@app.route('/api/viewport', methods=['GET'])
def get_viewport_graves():
    """Get graves within viewport bounds"""
    min_lon = float(request.args.get('min_lon', 44.305))
    min_lat = float(request.args.get('min_lat', 31.985))
    max_lon = float(request.args.get('max_lon', 44.345))
    max_lat = float(request.args.get('max_lat', 32.015))
    max_results = int(request.args.get('max_results', 500))
    
    conn = get_db_connection()
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    cursor.execute("""
        SELECT 
            encode(node_id, 'hex') as id,
            data->>'full_name_arabic' as name,
            data->>'residence_city' as zone,
            position[1] as lon,
            position[2] as lat,
            (data->>'quality')::INT as quality
        FROM spatial.fabric_spatial_quads
        WHERE 
            position[1] BETWEEN %s AND %s
            AND position[2] BETWEEN %s AND %s
            AND data->>'full_name_arabic' IS NOT NULL
        ORDER BY (data->>'quality')::INT DESC
        LIMIT %s
    """, (min_lon, max_lon, min_lat, max_lat, max_results))
    
    graves = cursor.fetchall()
    conn.close()
    
    return jsonify(graves)

@app.route('/api/search', methods=['GET'])
def search_names():
    """Search for names"""
    query = request.args.get('q', '')
    max_results = int(request.args.get('limit', 20))
    
    if len(query) < 2:
        return jsonify([])
    
    conn = get_db_connection()
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    cursor.execute("""
        SELECT 
            encode(node_id, 'hex') as id,
            data->>'full_name_arabic' as name,
            data->>'residence_city' as zone,
            position[1] as lon,
            position[2] as lat,
            (data->>'quality')::INT as quality
        FROM spatial.fabric_spatial_quads
        WHERE 
            data->>'full_name_arabic' ILIKE %s
            AND data->>'full_name_arabic' IS NOT NULL
        ORDER BY (data->>'quality')::INT DESC
        LIMIT %s
    """, (f'%{query}%', max_results))
    
    results = cursor.fetchall()
    conn.close()
    
    return jsonify(results)

@app.route('/api/zones', methods=['GET'])
def get_zone_stats():
    """Get zone distribution"""
    conn = get_db_connection()
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    cursor.execute("""
        SELECT 
            data->>'residence_city' as zone,
            COUNT(*) as count
        FROM spatial.fabric_spatial_quads
        WHERE data->>'residence_city' IS NOT NULL
        GROUP BY data->>'residence_city'
        ORDER BY count DESC
        LIMIT 10
    """)
    
    zones = cursor.fetchall()
    conn.close()
    
    return jsonify(zones)

@app.route('/api/quality', methods=['GET'])
def get_quality_distribution():
    """Get quality distribution"""
    conn = get_db_connection()
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    cursor.execute("""
        SELECT 
            CASE 
                WHEN (data->>'quality')::INT >= 200 THEN 'Sovereign (200-255)'
                WHEN (data->>'quality')::INT >= 140 THEN 'Active (140-199)'
                WHEN (data->>'quality')::INT >= 100 THEN 'Poor (100-139)'
                ELSE 'Non-Active (0-99)'
            END as tier,
            COUNT(*) as count
        FROM spatial.fabric_spatial_quads
        WHERE data->>'quality' IS NOT NULL
        GROUP BY 
            CASE 
                WHEN (data->>'quality')::INT >= 200 THEN 'Sovereign (200-255)'
                WHEN (data->>'quality')::INT >= 140 THEN 'Active (140-199)'
                WHEN (data->>'quality')::INT >= 100 THEN 'Poor (100-139)'
                ELSE 'Non-Active (0-99)'
            END
        ORDER BY count DESC
    """)
    
    quality = cursor.fetchall()
    conn.close()
    
    return jsonify(quality)

@app.route('/api/geojson', methods=['GET'])
def export_geojson():
    """Export data as GeoJSON"""
    min_lon = float(request.args.get('min_lon', 44.305))
    min_lat = float(request.args.get('min_lat', 31.985))
    max_lon = float(request.args.get('max_lon', 44.345))
    max_lat = float(request.args.get('max_lat', 32.015))
    max_features = int(request.args.get('max_features', 500))
    
    conn = get_db_connection()
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    cursor.execute("""
        SELECT 
            encode(node_id, 'hex') as id,
            data->>'full_name_arabic' as name,
            data->>'residence_city' as zone,
            position[1] as lon,
            position[2] as lat,
            (data->>'quality')::INT as quality
        FROM spatial.fabric_spatial_quads
        WHERE 
            position[1] BETWEEN %s AND %s
            AND position[2] BETWEEN %s AND %s
            AND data->>'full_name_arabic' IS NOT NULL
        LIMIT %s
    """, (min_lon, max_lon, min_lat, max_lat, max_features))
    
    graves = cursor.fetchall()
    conn.close()
    
    # Convert to GeoJSON
    features = []
    for grave in graves:
        features.append({
            'type': 'Feature',
            'id': grave['id'],
            'geometry': {
                'type': 'Point',
                'coordinates': [grave['lon'], grave['lat']]
            },
            'properties': {
                'name': grave['name'],
                'zone': grave['zone'],
                'quality': grave['quality']
            }
        })
    
    geojson = {
        'type': 'FeatureCollection',
        'features': features
    }
    
    return jsonify(geojson)

@app.route('/')
def index():
    """Root endpoint"""
    return jsonify({
        'message': 'NajafWay API Server',
        'version': 'BDBWay v1.0',
        'endpoints': [
            '/api/stats',
            '/api/viewport',
            '/api/search',
            '/api/zones',
            '/api/quality',
            '/api/geojson'
        ]
    })

if __name__ == '__main__':
    print("=" * 60)
    print(" NajafWay API Server")
    print(" BDBWay v1.0 Integration")
    print("=" * 60)
    print(" Starting server on http://localhost:5000")
    print(" API Endpoints:")
    print("   GET /api/stats")
    print("   GET /api/viewport?min_lon=...&max_lon=...&min_lat=...&max_lat=...")
    print("   GET /api/search?q=محمد")
    print("   GET /api/zones")
    print("   GET /api/quality")
    print("   GET /api/geojson")
    print("=" * 60)
    
    app.run(host='0.0.0.0', port=5000, debug=True)
