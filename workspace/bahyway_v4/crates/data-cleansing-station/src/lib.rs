//! data-cleansing-station — Sovereign VGCA-powered Data Cleansing.
//!
//! Combines VGCA geometric analysis (FSV 7D, BFV 6D) with DAMA-DMBOK
//! DQ dimension checks and the bahyway-dqm 6-dimension assessment engine
//! to produce a fully auditable `CleansingReport` per particle.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use data_cleansing_station::{VgcaCleansingStation, ParticleLane};
//! use template_engine::{Template, FieldSpec};
//!
//! let mut station = VgcaCleansingStation::new();
//! let template = Template::default_template(
//!     "t.customer", "Customer",
//!     &[
//!         FieldSpec::new(0x1001u32, "name",   true),
//!         FieldSpec::new(0x1002u32, "email",  true),
//!     ],
//! );
//!
//! let record = vec![
//!     (0x1001u32, b"Bahaa Fadam".to_vec()),
//!     (0x1002u32, b"bahaa@bahyway.com".to_vec()),
//! ];
//!
//! let report = station.cleanse(&record, &template, 100, 99, 10);
//! println!("{}", report.summary());
//! // [CLEANSE] lane=GEM B11=240 vgca_clean=100% dqm=SLA_PASS
//! ```

pub mod cleanse;

pub use cleanse::{
    // Legacy shim
    cleanse,
    CleansingReport,
    CleansingReportLegacy,
    DqFinding,
    FieldVgcaResult,
    OrbitDensitySnapshot,
    ParticleLane,
    // New VGCA-powered API
    VgcaCleansingStation,
    ACTIVE_B11,
    FUZZY_B11,
    // Sovereign lane thresholds
    GEM_B11,
    TRIBE_B11,
};
