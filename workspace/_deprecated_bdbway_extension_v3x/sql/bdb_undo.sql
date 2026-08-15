-- 1. Drop Views
DROP VIEW IF EXISTS bdb_mapway_energy_layer;

-- 2. Drop Procedures and Schemas
DROP PROCEDURE IF EXISTS bdb_mdm.resolve_twin_gems(UUID);
DROP SCHEMA IF EXISTS bdb_mdm CASCADE;

-- 3. Drop Main Fabric (Cascades to Partitions and Indexes)
DROP TABLE IF EXISTS bdb_fabric_master CASCADE;

-- 4. Remove Logic Kernels
DROP EXTENSION IF EXISTS bdbway_extension CASCADE;
-- Note: We usually keep 'vector' because it's a 3rd party tool,
-- but you can uncomment the line below if you want a 100% clean wipe.
-- DROP EXTENSION IF EXISTS vector CASCADE;

SELECT 'UNDO COMPLETE: Fabric Liquidation Successful' as status;
