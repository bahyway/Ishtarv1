#!/bin/bash
# Export real data from PostgreSQL to JSON for offline HTML visualization

export PGHOST="/home/akkad/.pgrx"
export PGPORT="28816"

echo "Exporting grave data from database..."

psql -d bdbway_extension -t -c "
COPY (
    SELECT json_agg(row_to_json(t))
    FROM (
        SELECT 
            data->>'full_name_arabic' as name,
            data->>'residence_city' as zone,
            position[1] as lon,
            position[2] as lat,
            (data->>'quality')::INT as quality
        FROM spatial.fabric_spatial_quads
        WHERE data->>'full_name_arabic' IS NOT NULL
        LIMIT 1000
    ) t
) TO STDOUT
" > /tmp/graves_data.json

# Create HTML with embedded data
cat > /tmp/najafway_standalone.html << 'HTMLEOF'
<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>NajafWay - مقبرة وادي السلام</title>
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css" />
    <link href="https://fonts.googleapis.com/css2?family=Cairo:wght@400;600;700&display=swap" rel="stylesheet">
    
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Cairo', Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            direction: rtl;
        }
        .header {
            background: rgba(255, 255, 255, 0.95);
            padding: 20px;
            text-align: center;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }
        .header h1 {
            color: #667eea;
            font-size: 2.5rem;
            margin-bottom: 10px;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            padding: 20px;
            max-width: 1200px;
            margin: 0 auto;
        }
        .stat-card {
            background: white;
            padding: 15px;
            border-radius: 10px;
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
            text-align: center;
        }
        .stat-value {
            font-size: 2rem;
            font-weight: bold;
            color: #764ba2;
        }
        .search-box {
            background: white;
            margin: 20px auto;
            padding: 20px;
            max-width: 600px;
            border-radius: 10px;
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
        }
        .search-input {
            width: 100%;
            padding: 12px;
            font-size: 1.1rem;
            border: 2px solid #667eea;
            border-radius: 8px;
            font-family: 'Cairo', Arial, sans-serif;
        }
        .results {
            margin-top: 15px;
            max-height: 300px;
            overflow-y: auto;
        }
        .result-item {
            padding: 10px;
            border-bottom: 1px solid #eee;
            cursor: pointer;
            transition: background 0.2s;
        }
        .result-item:hover {
            background: #f0f0f0;
        }
        .quality-badge {
            display: inline-block;
            padding: 3px 8px;
            border-radius: 4px;
            font-size: 0.8rem;
            font-weight: bold;
            margin-left: 8px;
        }
        .q-gold { background: #FFD700; color: #333; }
        .q-green { background: #90EE90; color: #333; }
        .q-yellow { background: #FFE4B5; color: #333; }
        .q-gray { background: #D3D3D3; color: #666; }
        .map-container {
            background: white;
            margin: 20px;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
        }
        #map {
            height: 600px;
            border-radius: 8px;
            border: 2px solid #667eea;
        }
        .legend {
            background: white;
            padding: 12px;
            border-radius: 8px;
            position: absolute;
            bottom: 30px;
            left: 10px;
            z-index: 1000;
            box-shadow: 0 2px 8px rgba(0,0,0,0.2);
            font-size: 0.9rem;
        }
        .legend-item {
            margin: 4px 0;
            display: flex;
            align-items: center;
        }
        .legend-color {
            width: 16px;
            height: 16px;
            border-radius: 50%;
            margin-left: 8px;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🕌 مقبرة وادي السلام - النجف الأشرف</h1>
        <p>BDBWay v1.0 Stress Test - 340,000 Records</p>
    </div>
    
    <div class="stats">
        <div class="stat-card">
            <h3>إجمالي السجلات</h3>
            <div class="stat-value" id="totalCount">340,000</div>
        </div>
        <div class="stat-card">
            <h3>السجلات المعروضة</h3>
            <div class="stat-value" id="displayCount">1,000</div>
        </div>
        <div class="stat-card">
            <h3>وقت الاستيراد</h3>
            <div class="stat-value">~20 دقيقة</div>
        </div>
        <div class="stat-card">
            <h3>حجم البيانات</h3>
            <div class="stat-value">151 MB</div>
        </div>
    </div>
    
    <div class="search-box">
        <h3 style="color: #667eea; margin-bottom: 12px;">🔍 البحث عن الأسماء</h3>
        <input type="text" class="search-input" id="searchInput" 
               placeholder="ابحث عن اسم... (محمد، فاطمة، علي، حسن)">
        <div class="results" id="results"></div>
    </div>
    
    <div class="map-container">
        <h3 style="color: #667eea; margin-bottom: 12px; text-align: center;">
            🗺️ خريطة المقبرة
        </h3>
        <div id="map"></div>
        <div class="legend">
            <strong>مستويات الجودة:</strong>
            <div class="legend-item">
                <div class="legend-color" style="background: #FFD700;"></div>
                <span>Sovereign (200-255)</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #90EE90;"></div>
                <span>Active (140-199)</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #FFE4B5;"></div>
                <span>Poor (100-139)</span>
            </div>
            <div class="legend-item">
                <div class="legend-color" style="background: #D3D3D3;"></div>
                <span>Non-Active (0-99)</span>
            </div>
        </div>
    </div>
    
    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
    <script>
        // Load data from external JSON file
        let gravesData = [];
        
        fetch('graves_data.json')
            .then(response => response.json())
            .then(data => {
                gravesData = data[0]; // PostgreSQL returns array with one element
                initializeApp();
            })
            .catch(err => {
                console.error('Error loading data:', err);
                alert('تعذر تحميل البيانات. تأكد من وجود ملف graves_data.json');
            });
        
        function initializeApp() {
            // Update display count
            document.getElementById('displayCount').textContent = gravesData.length.toLocaleString('ar-EG');
            
            // Initialize map
            const map = L.map('map').setView([32.0, 44.325], 13);
            L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                attribution: '© OpenStreetMap'
            }).addTo(map);
            
            // Color function
            function getColor(q) {
                if (q >= 200) return '#FFD700';
                if (q >= 140) return '#90EE90';
                if (q >= 100) return '#FFE4B5';
                return '#D3D3D3';
            }
            
            function getBadgeClass(q) {
                if (q >= 200) return 'q-gold';
                if (q >= 140) return 'q-green';
                if (q >= 100) return 'q-yellow';
                return 'q-gray';
            }
            
            // Add markers
            gravesData.forEach(grave => {
                if (!grave.lon || !grave.lat) return;
                
                const marker = L.circleMarker([grave.lat, grave.lon], {
                    radius: 5,
                    fillColor: getColor(grave.quality),
                    color: '#000',
                    weight: 1,
                    opacity: 0.8,
                    fillOpacity: 0.7
                }).addTo(map);
                
                marker.bindPopup(`
                    <div style="direction: rtl; font-family: 'Cairo';">
                        <strong>${grave.name || 'غير معروف'}</strong><br>
                        المنطقة: ${grave.zone || 'غير محدد'}<br>
                        الجودة: ${grave.quality}<br>
                        <span class="quality-badge ${getBadgeClass(grave.quality)}">
                            ${grave.quality >= 200 ? 'ممتاز' : 
                              grave.quality >= 140 ? 'جيد' :
                              grave.quality >= 100 ? 'مقبول' : 'ضعيف'}
                        </span>
                    </div>
                `);
            });
            
            // Search functionality
            const searchInput = document.getElementById('searchInput');
            const resultsDiv = document.getElementById('results');
            
            searchInput.addEventListener('input', (e) => {
                const query = e.target.value.trim();
                if (query.length < 2) {
                    resultsDiv.innerHTML = '';
                    return;
                }
                
                const results = gravesData.filter(g => 
                    g.name && g.name.includes(query)
                ).slice(0, 20);
                
                if (results.length === 0) {
                    resultsDiv.innerHTML = '<div class="result-item">لم يتم العثور على نتائج</div>';
                    return;
                }
                
                resultsDiv.innerHTML = results.map(r => `
                    <div class="result-item" onclick="flyTo(${r.lat}, ${r.lon})">
                        <span class="quality-badge ${getBadgeClass(r.quality)}">${r.quality}</span>
                        <strong>${r.name}</strong><br>
                        <small>${r.zone || 'غير محدد'}</small>
                    </div>
                `).join('');
            });
            
            window.flyTo = function(lat, lon) {
                map.flyTo([lat, lon], 16, { duration: 1.5 });
            };
            
            console.log('✅ تم تحميل', gravesData.length, 'سجل بنجاح');
        }
    </script>
</body>
</html>
HTMLEOF

echo "Created standalone HTML at /tmp/najafway_standalone.html"
echo "Copy both files to your desktop:"
echo "  - /tmp/graves_data.json"
echo "  - /tmp/najafway_standalone.html"
echo ""
echo "Then open najafway_standalone.html in your browser!"
