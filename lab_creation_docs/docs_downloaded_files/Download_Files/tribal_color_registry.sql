-- ============================================================
-- BDBWay v1.0: Iraqi Tribal Color Registry
-- Sovereign Identity Enhancement with Tribal DNA
-- ============================================================
-- Purpose: Register all Iraqi tribes (Arab, Kurdish, Turkmen)
-- with hierarchical color coding for identity validation
-- ============================================================

-- DROP EXISTING TABLES (if needed)
DROP TABLE IF EXISTS bdb_tribal_registry CASCADE;
DROP TABLE IF EXISTS bdb_tribal_hierarchy CASCADE;
DROP TABLE IF EXISTS bdb_tribal_colors CASCADE;
DROP TYPE IF EXISTS tribal_ethnicity CASCADE;

-- ============================================================
-- ENUMS & TYPES
-- ============================================================

CREATE TYPE tribal_ethnicity AS ENUM (
    'ARAB',
    'KURDISH', 
    'TURKMEN',
    'ASSYRIAN',
    'YAZIDI',
    'MANDAEAN',
    'SHABAK',
    'MIXED',
    'UNKNOWN'
);

-- ============================================================
-- TABLE 1: TRIBAL COLOR PALETTE
-- ============================================================
-- Defines the color space for tribal identification
-- Root tribes get primary colors (0-50)
-- Sub-tribes get derivative colors (51-200)
-- Non-tribal gets (201-255)

CREATE TABLE bdb_tribal_colors (
    color_id SERIAL PRIMARY KEY,
    color_range_start INT NOT NULL CHECK (color_range_start >= 0 AND color_range_start <= 255),
    color_range_end INT NOT NULL CHECK (color_range_end >= 0 AND color_range_end <= 255),
    color_category VARCHAR(50) NOT NULL,
    description_ar TEXT,
    description_en TEXT,
    rgb_example VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT valid_range CHECK (color_range_end >= color_range_start)
);

-- Insert color ranges
INSERT INTO bdb_tribal_colors (color_range_start, color_range_end, color_category, description_ar, description_en, rgb_example) VALUES
(0, 10, 'ROOT_ARAB_MAJOR', 'القبائل العربية الكبرى', 'Major Arab Tribes', '#8B4513'),
(11, 30, 'ROOT_ARAB_MEDIUM', 'القبائل العربية المتوسطة', 'Medium Arab Tribes', '#A0522D'),
(31, 50, 'ROOT_KURDISH', 'القبائل الكردية', 'Kurdish Tribes', '#228B22'),
(51, 70, 'ROOT_TURKMEN', 'القبائل التركمانية', 'Turkmen Tribes', '#4682B4'),
(71, 90, 'SUB_ARAB', 'عشائر عربية فرعية', 'Arab Sub-tribes', '#D2691E'),
(91, 110, 'SUB_KURDISH', 'عشائر كردية فرعية', 'Kurdish Sub-tribes', '#90EE90'),
(111, 130, 'SUB_TURKMEN', 'عشائر تركمانية فرعية', 'Turkmen Sub-tribes', '#87CEEB'),
(131, 150, 'MINORITY_TRIBES', 'الأقليات والعشائر الصغيرة', 'Minority Tribes', '#DDA0DD'),
(151, 200, 'MIXED_HERITAGE', 'أصول مختلطة', 'Mixed Heritage', '#F0E68C'),
(201, 255, 'NON_TRIBAL', 'غير قبلي', 'Non-Tribal', '#C0C0C0');

-- ============================================================
-- TABLE 2: TRIBAL HIERARCHY
-- ============================================================
-- Hierarchical structure: Root Tribe -> Sub-Tribe -> Branch

CREATE TABLE bdb_tribal_hierarchy (
    tribe_id SERIAL PRIMARY KEY,
    tribe_name_ar VARCHAR(200) NOT NULL,
    tribe_name_en VARCHAR(200),
    parent_tribe_id INT REFERENCES bdb_tribal_hierarchy(tribe_id),
    ethnicity tribal_ethnicity NOT NULL,
    tier INT NOT NULL CHECK (tier >= 1 AND tier <= 5), -- 1=Root, 2=Sub, 3=Branch, etc.
    
    -- Color Assignment (Red Channel in BDBWay identity)
    assigned_color INT NOT NULL CHECK (assigned_color >= 0 AND assigned_color <= 255),
    color_range_id INT REFERENCES bdb_tribal_colors(color_id),
    
    -- Geographic Information
    primary_region VARCHAR(100), -- e.g., 'Baghdad', 'Basra', 'Najaf', 'Erbil'
    secondary_regions TEXT[], -- Array of other regions
    
    -- Historical & Cultural Data
    historical_notes_ar TEXT,
    historical_notes_en TEXT,
    estimated_population INT,
    
    -- Metadata
    is_active BOOLEAN DEFAULT TRUE,
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(tribe_name_ar, parent_tribe_id),
    CONSTRAINT root_has_no_parent CHECK (
        (tier = 1 AND parent_tribe_id IS NULL) OR 
        (tier > 1 AND parent_tribe_id IS NOT NULL)
    )
);

-- Create indexes
CREATE INDEX idx_tribal_hierarchy_ethnicity ON bdb_tribal_hierarchy(ethnicity);
CREATE INDEX idx_tribal_hierarchy_region ON bdb_tribal_hierarchy(primary_region);
CREATE INDEX idx_tribal_hierarchy_color ON bdb_tribal_hierarchy(assigned_color);
CREATE INDEX idx_tribal_hierarchy_parent ON bdb_tribal_hierarchy(parent_tribe_id);

-- ============================================================
-- TABLE 3: TRIBAL REGISTRY (Main Lookup)
-- ============================================================
-- Fast lookup for tribal name validation

CREATE TABLE bdb_tribal_registry (
    registry_id SERIAL PRIMARY KEY,
    tribe_id INT NOT NULL REFERENCES bdb_tribal_hierarchy(tribe_id),
    
    -- All possible name variations for fuzzy matching
    name_variations TEXT[] NOT NULL,
    
    -- Phonetic keys for Arabic fuzzy matching
    soundex_ar VARCHAR(10),
    metaphone_ar VARCHAR(10),
    
    -- Quick lookup fields
    ethnicity tribal_ethnicity NOT NULL,
    assigned_color INT NOT NULL,
    tier INT NOT NULL,
    
    -- Fast search
    search_vector tsvector,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_tribal_registry_color ON bdb_tribal_registry(assigned_color);
CREATE INDEX idx_tribal_registry_ethnicity ON bdb_tribal_registry(ethnicity);
CREATE INDEX idx_tribal_registry_search ON bdb_tribal_registry USING gin(search_vector);

-- ============================================================
-- SAMPLE DATA: MAJOR IRAQI TRIBES
-- ============================================================

-- ============================================================
-- ARAB TRIBES (Tier 1 - Root Tribes)
-- ============================================================

-- Major Arab Tribes (Color 0-10)
INSERT INTO bdb_tribal_hierarchy (tribe_name_ar, tribe_name_en, ethnicity, tier, assigned_color, color_range_id, primary_region, estimated_population) VALUES
('شمر', 'Shammar', 'ARAB', 1, 1, 1, 'Mosul', 1500000),
('الدليم', 'Al-Dulaim', 'ARAB', 1, 2, 1, 'Anbar', 2000000),
('العبيد', 'Al-Ubaid', 'ARAB', 1, 3, 1, 'Najaf', 800000),
('بني تميم', 'Bani Tamim', 'ARAB', 1, 4, 1, 'Diyala', 500000),
('الجبور', 'Al-Jubur', 'ARAB', 1, 5, 1, 'Salah ad-Din', 900000),
('ربيعة', 'Rabi''a', 'ARAB', 1, 6, 1, 'Mosul', 700000),
('بني لام', 'Bani Lam', 'ARAB', 1, 7, 1, 'Maysan', 600000),
('خزاعة', 'Khuzaa', 'ARAB', 1, 8, 1, 'Basra', 450000),
('الخزاعل', 'Al-Khaza''il', 'ARAB', 1, 9, 1, 'Dhi Qar', 550000),
('بني مالك', 'Bani Malik', 'ARAB', 1, 10, 1, 'Babylon', 400000);

-- Medium Arab Tribes (Color 11-30)
INSERT INTO bdb_tribal_hierarchy (tribe_name_ar, tribe_name_en, ethnicity, tier, assigned_color, color_range_id, primary_region) VALUES
('بني حسن', 'Bani Hassan', 'ARAB', 1, 11, 2, 'Wasit'),
('عنزة', 'Anaza', 'ARAB', 1, 12, 2, 'Anbar'),
('الزبيد', 'Al-Zubaid', 'ARAB', 1, 13, 2, 'Najaf'),
('السعدون', 'Al-Sa''dun', 'ARAB', 1, 14, 2, 'Dhi Qar'),
('بني ركاب', 'Bani Rikab', 'ARAB', 1, 15, 2, 'Babylon'),
('ألبو محمد', 'Albu Mohammed', 'ARAB', 1, 16, 2, 'Diyala'),
('الكعبي', 'Al-Ka''bi', 'ARAB', 1, 17, 2, 'Basra'),
('الطائي', 'Al-Tai', 'ARAB', 1, 18, 2, 'Najaf'),
('النعيم', 'Al-Na''im', 'ARAB', 1, 19, 2, 'Anbar'),
('البو نمر', 'Al-Bu Nimr', 'ARAB', 1, 20, 2, 'Anbar');

-- Sub-tribes of Shammar (Tier 2, Color 71-74)
INSERT INTO bdb_tribal_hierarchy (tribe_name_ar, tribe_name_en, parent_tribe_id, ethnicity, tier, assigned_color, color_range_id, primary_region) VALUES
('الصايح', 'Al-Sayeh', 1, 'ARAB', 2, 71, 5, 'Mosul'),
('عبدة', 'Abda', 1, 'ARAB', 2, 72, 5, 'Mosul'),
('الأسلم', 'Al-Aslam', 1, 'ARAB', 2, 73, 5, 'Mosul'),
('الصلتة', 'Al-Salta', 1, 'ARAB', 2, 74, 5, 'Mosul');

-- ============================================================
-- KURDISH TRIBES (Tier 1 - Root Tribes, Color 31-50)
-- ============================================================

INSERT INTO bdb_tribal_hierarchy (tribe_name_ar, tribe_name_en, ethnicity, tier, assigned_color, color_range_id, primary_region, estimated_population) VALUES
('بارزاني', 'Barzani', 'KURDISH', 1, 31, 3, 'Erbil', 500000),
('طالباني', 'Talabani', 'KURDISH', 1, 32, 3, 'Sulaymaniyah', 450000),
('سوراني', 'Sorani', 'KURDISH', 1, 33, 3, 'Sulaymaniyah', 800000),
('بادينان', 'Badinan', 'KURDISH', 1, 34, 3, 'Dohuk', 700000),
('جاف', 'Jaff', 'KURDISH', 1, 35, 3, 'Sulaymaniyah', 300000),
('البرواري', 'Al-Barwari', 'KURDISH', 1, 36, 3, 'Dohuk', 250000),
('الزنغانة', 'Zangana', 'KURDISH', 1, 37, 3, 'Erbil', 200000),
('دوسكي', 'Doski', 'KURDISH', 1, 38, 3, 'Dohuk', 180000);

-- Kurdish Sub-tribes (Tier 2, Color 91-95)
INSERT INTO bdb_tribal_hierarchy (tribe_name_ar, tribe_name_en, parent_tribe_id, ethnicity, tier, assigned_color, color_range_id, primary_region) VALUES
('الزيباري', 'Zebari', (SELECT tribe_id FROM bdb_tribal_hierarchy WHERE tribe_name_ar = 'بارزاني'), 'KURDISH', 2, 91, 6, 'Erbil'),
('البرادوستي', 'Bradosti', (SELECT tribe_id FROM bdb_tribal_hierarchy WHERE tribe_name_ar = 'بارزاني'), 'KURDISH', 2, 92, 6, 'Erbil');

-- ============================================================
-- TURKMEN TRIBES (Tier 1, Color 51-70)
-- ============================================================

INSERT INTO bdb_tribal_hierarchy (tribe_name_ar, tribe_name_en, ethnicity, tier, assigned_color, color_range_id, primary_region, estimated_population) VALUES
('بيات', 'Bayat', 'TURKMEN', 1, 51, 4, 'Kirkuk', 300000),
('آلبو صالح', 'Albu Salih', 'TURKMEN', 1, 52, 4, 'Kirkuk', 150000),
('جاووش', 'Chawish', 'TURKMEN', 1, 53, 4, 'Tal Afar', 100000),
('كايي', 'Kayi', 'TURKMEN', 1, 54, 4, 'Erbil', 80000);

-- Turkmen Sub-tribes (Tier 2, Color 111-115)
INSERT INTO bdb_tribal_hierarchy (tribe_name_ar, tribe_name_en, parent_tribe_id, ethnicity, tier, assigned_color, color_range_id, primary_region) VALUES
('البهادرلي', 'Bahadarli', (SELECT tribe_id FROM bdb_tribal_hierarchy WHERE tribe_name_ar = 'بيات'), 'TURKMEN', 2, 111, 7, 'Kirkuk'),
('الموصللي', 'Mosuli', (SELECT tribe_id FROM bdb_tribal_hierarchy WHERE tribe_name_ar = 'بيات'), 'TURKMEN', 2, 112, 7, 'Kirkuk');

-- ============================================================
-- POPULATE REGISTRY WITH NAME VARIATIONS
-- ============================================================

INSERT INTO bdb_tribal_registry (tribe_id, name_variations, ethnicity, assigned_color, tier)
SELECT 
    tribe_id,
    ARRAY[tribe_name_ar, tribe_name_en, 
          'قبيلة ' || tribe_name_ar,
          'عشيرة ' || tribe_name_ar,
          tribe_name_ar || 'ي',  -- With nisba suffix
          'ال' || tribe_name_ar
    ],
    ethnicity,
    assigned_color,
    tier
FROM bdb_tribal_hierarchy;

-- Update search vectors
UPDATE bdb_tribal_registry 
SET search_vector = to_tsvector('arabic', array_to_string(name_variations, ' '));

-- ============================================================
-- FUNCTIONS FOR TRIBAL VALIDATION
-- ============================================================

-- Function 1: Extract tribe from full name
CREATE OR REPLACE FUNCTION bdb_extract_tribal_name(full_name_arabic TEXT)
RETURNS TEXT AS $$
DECLARE
    name_parts TEXT[];
    potential_tribe TEXT;
BEGIN
    -- Split name by spaces
    name_parts := string_to_array(full_name_arabic, ' ');
    
    -- Typically the last name is the tribal/family name
    IF array_length(name_parts, 1) >= 3 THEN
        potential_tribe := name_parts[array_length(name_parts, 1)];
        RETURN potential_tribe;
    END IF;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function 2: Validate and assign tribal color
CREATE OR REPLACE FUNCTION bdb_validate_tribal_identity(
    full_name_arabic TEXT,
    fallback_color INT DEFAULT 220  -- Default non-tribal color
)
RETURNS TABLE (
    tribe_id INT,
    tribe_name_ar TEXT,
    assigned_color INT,
    ethnicity tribal_ethnicity,
    confidence REAL
) AS $$
DECLARE
    extracted_tribe TEXT;
BEGIN
    extracted_tribe := bdb_extract_tribal_name(full_name_arabic);
    
    IF extracted_tribe IS NULL THEN
        RETURN QUERY
        SELECT NULL::INT, 'غير قبلي'::TEXT, fallback_color, 'UNKNOWN'::tribal_ethnicity, 0.0::REAL;
        RETURN;
    END IF;
    
    -- Fuzzy match against tribal registry
    RETURN QUERY
    SELECT 
        r.tribe_id,
        h.tribe_name_ar,
        r.assigned_color,
        r.ethnicity,
        CASE 
            WHEN extracted_tribe = ANY(r.name_variations) THEN 1.0
            WHEN similarity(extracted_tribe, h.tribe_name_ar) > 0.6 THEN similarity(extracted_tribe, h.tribe_name_ar)
            ELSE 0.3
        END as confidence
    FROM bdb_tribal_registry r
    JOIN bdb_tribal_hierarchy h ON r.tribe_id = h.tribe_id
    WHERE extracted_tribe ILIKE '%' || h.tribe_name_ar || '%'
       OR h.tribe_name_ar ILIKE '%' || extracted_tribe || '%'
       OR extracted_tribe = ANY(r.name_variations)
    ORDER BY confidence DESC
    LIMIT 1;
    
    -- If no match found, return non-tribal
    IF NOT FOUND THEN
        RETURN QUERY
        SELECT NULL::INT, 'غير قبلي'::TEXT, fallback_color, 'UNKNOWN'::tribal_ethnicity, 0.0::REAL;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function 3: Get tribal hierarchy path
CREATE OR REPLACE FUNCTION bdb_get_tribal_hierarchy(input_tribe_id INT)
RETURNS TEXT AS $$
WITH RECURSIVE tribe_path AS (
    -- Base case
    SELECT tribe_id, tribe_name_ar, parent_tribe_id, 1 as level
    FROM bdb_tribal_hierarchy
    WHERE tribe_id = input_tribe_id
    
    UNION ALL
    
    -- Recursive case
    SELECT h.tribe_id, h.tribe_name_ar, h.parent_tribe_id, tp.level + 1
    FROM bdb_tribal_hierarchy h
    JOIN tribe_path tp ON h.tribe_id = tp.parent_tribe_id
)
SELECT string_agg(tribe_name_ar, ' -> ' ORDER BY level DESC)
FROM tribe_path;
$$ LANGUAGE sql;

-- ============================================================
-- VIEWS FOR ANALYTICS
-- ============================================================

CREATE OR REPLACE VIEW bdb_tribal_statistics AS
SELECT 
    ethnicity,
    tier,
    COUNT(*) as tribe_count,
    MIN(assigned_color) as color_start,
    MAX(assigned_color) as color_end,
    SUM(estimated_population) as total_population
FROM bdb_tribal_hierarchy
GROUP BY ethnicity, tier
ORDER BY ethnicity, tier;

-- ============================================================
-- EXAMPLE USAGE
-- ============================================================

-- Example 1: Validate a name
SELECT * FROM bdb_validate_tribal_identity('محمد علي الموسوي');
SELECT * FROM bdb_validate_tribal_identity('أحمد حسن الدليمي');
SELECT * FROM bdb_validate_tribal_identity('فاطمة بارزاني');

-- Example 2: Get tribal hierarchy
SELECT bdb_get_tribal_hierarchy(
    (SELECT tribe_id FROM bdb_tribal_hierarchy WHERE tribe_name_ar = 'الصايح')
);

-- Example 3: Statistics
SELECT * FROM bdb_tribal_statistics;

-- Example 4: Find all sub-tribes of Shammar
SELECT 
    h.tribe_name_ar,
    h.assigned_color,
    h.tier,
    bdb_get_tribal_hierarchy(h.tribe_id) as full_path
FROM bdb_tribal_hierarchy h
WHERE parent_tribe_id = (SELECT tribe_id FROM bdb_tribal_hierarchy WHERE tribe_name_ar = 'شمر')
   OR tribe_id = (SELECT tribe_id FROM bdb_tribal_hierarchy WHERE tribe_name_ar = 'شمر');

-- ============================================================
-- SUCCESS MESSAGE
-- ============================================================

SELECT '✅ Iraqi Tribal Color Registry Created Successfully!' as status,
       COUNT(*) as total_tribes
FROM bdb_tribal_hierarchy;
