//! # Pollution Engine — Multi-Domain Sensor Ingestion
//! **PollutionEngine v4.0 | BahyWay.Ecosystem | DUB.SAR 𒁾**
//!
//! Zero external dependencies. Pure Rust sovereign implementation.
//! Migrated from PollutionWay v3.5 per ADR-002 ("Way" suffix removed).

#![forbid(unsafe_code)]

pub mod domain;
pub mod sensors;
pub mod import;

pub use domain::{PollutionDomain, PollutionError};
pub use sensors::RawSensorReading;
pub use import::smart_import;
