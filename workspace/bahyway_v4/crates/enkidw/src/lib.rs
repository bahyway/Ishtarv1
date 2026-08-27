//! enkidw — EnkiDB Data Warehouse layer (§12).
//!
//! Provides the full ETL pipeline:
//!   LandingZone → ArchiveEngine → RecordRouter → KakiGenerator → PersistedDb
//!
//! The RecordRouter normalises ALL ingested content — CSV, JSON, XML, Parquet,
//! Excel, PDF, Word, BSON, images (PNG/JPEG/TIFF/GeoTIFF/HDF5/NetCDF/hyperspectral/
//! drone/satellite), point clouds (LAS/LAZ), video/audio — into Vec<RawRecord>
//! so the ETL station chain (adad-gate → structure → cleanse → VGCA → score)
//! works unchanged for every media type.
//!
//! Plus analytical read path:
//!   DwAnalytics → DwReport (state counts, epoch ranges, top-N)
//!
//! And sovereign security compilation:
//!   WayFile → WayCompiler → AAOL (.akk) source

pub mod ars;
pub mod batch_schema;
pub mod dw_analytics;
pub mod etl_pipeline;
pub mod kaki_generator;
pub mod landing_zone;
pub mod processing_zone;
pub mod record_router;
pub mod rotor_partition;
pub mod way_compiler;
pub mod way_file;
pub mod zip_engine;

pub use batch_schema::BatchSchema;
pub use dw_analytics::{DwAnalytics, DwReport, ParticleStat};
pub use etl_pipeline::{DwAlert, DwAlertSeverity, EtlPipeline, EtlStats};
pub use kaki_generator::{
    generate as kaki_generate, parse_csv, parse_tsv, GeneratedEntry, RawRecord,
};
pub use landing_zone::{LandingFile, LandingFileKind, LandingZone};
pub use processing_zone::{ProcessingZone, StagedEntry};
pub use record_router::records_from_entry;
pub use way_compiler::{compile as way_compile, CompileResult};
pub use way_file::WayFile;
pub use zip_engine::{
    build_store_zip, extract as zip_extract, extract_ok as zip_extract_ok, ZipEntry, ZipError,
};
