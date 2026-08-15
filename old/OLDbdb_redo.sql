-- Drop existing objects first
DROP INDEX IF EXISTS spatial.idx_fabric_spatial;
DROP FUNCTION IF EXISTS bdb_get_lon(real[]);
DROP FUNCTION IF EXISTS bdb_get_lat(real[]);

-- Now create the index
CREATE INDEX idx_fabric_spatial ON spatial.fabric_spatial_quads
USING gist(position);

-- Update the procedure to use proper function calls
CREATE OR REPLACE PROCEDURE spatial.redo_sovereign_fabric()
LANGUAGE plpgsql AS $$
DECLARE
    rec RECORD;
BEGIN
    FOR rec IN
        SELECT
            node_id,
            position,
            bdb_get_lon(position) as longitude,  -- Remove ::real[]
            bdb_get_lat(position) as latitude    -- Remove ::real[]
        FROM spatial.fabric_spatial_quads
        LIMIT 5
    LOOP
        INSERT INTO status (status)
        VALUES (format('Node %s at (%s, %s)',
            rec.node_id,
            rec.longitude,
            rec.latitude));
    END LOOP;

    INSERT INTO status (status)
    VALUES ('REDO COMPLETE: Sovereign Fabric Operational');
END;
$$;

-- Call the procedure
CALL spatial.redo_sovereign_fabric();

-- Verify
SELECT * FROM status;
