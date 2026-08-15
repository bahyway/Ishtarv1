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
pub mod landing_zone;
pub mod rotor_partition;
pub mod zip_engine;
pub mod way_file;
pub mod way_compiler;
pub mod kaki_generator;
pub mod etl_pipeline;
pub mod dw_analytics;
pub mod processing_zone;
pub mod batch_schema;
pub mod record_router;

pub use landing_zone::{LandingZone, LandingFile, LandingFileKind};
pub use zip_engine::{ZipEntry, ZipError, extract as zip_extract, extract_ok as zip_extract_ok, build_store_zip};
pub use way_file::WayFile;
pub use way_compiler::{compile as way_compile, CompileResult};
pub use kaki_generator::{RawRecord, GeneratedEntry, parse_tsv, parse_csv, generate as kaki_generate};
pub use etl_pipeline::{EtlPipeline, EtlStats, DwAlert, DwAlertSeverity};
pub use dw_analytics::{DwAnalytics, DwReport, ParticleStat};
pub use processing_zone::{ProcessingZone, StagedEntry};
pub use batch_schema::BatchSchema;
pub use record_router::records_from_entry;
