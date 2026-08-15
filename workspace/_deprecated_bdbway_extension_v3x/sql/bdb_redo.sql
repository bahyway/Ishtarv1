-- ============================================================
-- BDBWay 1.0: Sovereign Object Factory (Unified Edition)
-- ============================================================

-- 1. CLEAN SLATE (Undo phase)
DROP SCHEMA IF EXISTS bdb_fabric CASCADE;
DROP SCHEMA IF EXISTS bdb_mdm CASCADE;
DROP TABLE IF EXISTS bdb_status_logs CASCADE;

-- 2. SCHEMAS & EXTENSIONS
CREATE SCHEMA bdb_fabric;
CREATE SCHEMA bdb_mdm;
CREATE SCHEMA IF NOT EXISTS bdb_registry;


CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS bdbway_extension;

-- 3. SHARED TYPES & SEQUENCES
DO $$ BEGIN
    CREATE TYPE bdb_status AS ENUM ('Node', 'Gem', 'QueenGem', 'Detonated');
EXCEPTION WHEN duplicate_object THEN null; END $$;

CREATE TABLE bdb_status_logs (
    id SERIAL PRIMARY KEY,
    status_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 4. THE FABRIC (Partitioned Anchor Table)
-- We use position vector(3) for Hubble Zooming
-- We use get_byte(id, 13) for the Life-Cycle Partitioning
CREATE TABLE bdb_fabric.nodes (
    id BYTEA,                          -- 16-byte DNA
    stable_uuid UUID NOT NULL,         -- Immutable Life-Link
    tribe_id INT NOT NULL,             -- PartitionID (Sector)
    data JSONB,                        -- 26-column Satellite
    position vector(3),                -- X,Y,Z (Sovereign Coordinates)
    status bdb_status DEFAULT 'Node',
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (bdb_get_quality(id));

-- 5. PARTITIONS (Transient Larvae vs Permanent Gems)
CREATE TABLE bdb_fabric.transient
    PARTITION OF bdb_fabric.nodes FOR VALUES FROM (0) TO (140);

CREATE TABLE bdb_fabric.permanent
    PARTITION OF bdb_fabric.nodes FOR VALUES FROM (140) TO (256);

    -- THE CHROMATIC REGISTRY
    -- This is the "Social Contract" of the BDBWay 1.0
    CREATE TABLE bdb_registry.tribe_map (
        tribe_root_id INT PRIMARY KEY,      -- Matches Byte 12 (Red)
        tribe_name_ar TEXT NOT NULL,         -- Arabic Name
        tribe_name_en TEXT,                  -- English Name (for global demo)
        base_color_hex CHAR(7),              -- The "Official" Color
        ui_glow_intensity FLOAT DEFAULT 1.0, -- For the 3D Engine
        parent_confederation_id INT,         -- For Hierarchical Sets
        is_active BOOLEAN DEFAULT TRUE
    );

-- 6. INDEXES (Akkadian Performance Layer)
CREATE INDEX idx_node_quality ON bdb_fabric.nodes (bdb_get_quality(id));
CREATE INDEX idx_node_spatial ON bdb_fabric.nodes USING hnsw (position vector_l2_ops);
CREATE INDEX idx_stable_link ON bdb_fabric.nodes (stable_uuid);
-- Index for high-speed UI lookups
CREATE INDEX idx_registry_name ON bdb_registry.tribe_map(tribe_name_ar);;

-- 7. INITIAL SOVEREIGN INJECTION (Sample Data)
-- We use bdb_generate_identity (Rust) to ensure the 16-byte PK is perfect
INSERT INTO bdb_fabric.nodes (id, stable_uuid, tribe_id, position, data) VALUES
    (bdb_generate_identity('550e8400-e29b-41d4-a716-446655440000'::text, 101, 255, 240, 100),
     '550e8400-e29b-41d4-a716-446655440000', 101, '[13.4050, 52.5200, 0]',
     '{"name": "Berlin Gem", "type": "Energy_Station"}'),
    (bdb_generate_identity('550e8400-e29b-41d4-a716-446655440001'::text, 101, 255, 50, 100),
     '550e8400-e29b-41d4-a716-446655440001', 101, '[2.3522, 48.8566, 0]',
     '{"name": "Paris Node", "type": "Energy_Station"}');

-- 8. SOVEREIGN ORCHESTRATION PROCEDURE
-- This logs the status of the fabric birth
CREATE OR REPLACE PROCEDURE bdb_fabric.audit_fabric_birth()
LANGUAGE plpgsql AS $$
DECLARE
    rec RECORD;
BEGIN
    FOR rec IN
        SELECT
            id,
            bdb_classify_node(id) as life_stage,
            bdb_get_lon(position::real[]) as lon, -- Calls your Rust function
            bdb_get_lat(position::real[]) as lat, -- Calls your Rust function
            data->>'name' as label
        FROM bdb_fabric.nodes
    LOOP
        INSERT INTO bdb_status_logs (status_message)
        VALUES (format('Birth: %s | DNA: %s | GPS: (%s, %s)',
                rec.life_stage, encode(rec.id, 'hex'), rec.lon, rec.lat));
    END LOOP;

    INSERT INTO bdb_status_logs (status_message)
    VALUES ('REDO COMPLETE: BDBWay 1.0 Fabric is Online');
END;
$$;

-- 9. EXECUTE & VERIFY
CALL bdb_fabric.audit_fabric_birth();
SELECT * FROM bdb_status_logs ORDER BY id;

-- 10. MDM GOLDEN RECORD VIEW
CREATE OR REPLACE VIEW bdb_mdm.golden_records AS
SELECT DISTINCT ON (stable_uuid) * FROM bdb_fabric.nodes
WHERE bdb_get_quality(id) >= 200
ORDER BY stable_uuid, bdb_get_quality(id) DESC;
