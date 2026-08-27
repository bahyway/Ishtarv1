//! # vault-engine — Sovereign File System Health Analyzer
//!
//! Walks the OS file tree, detects Gray-Rot patterns, and classifies
//! every file and collection into BahyWay quality lanes.
//!
//! ## The Sovereign Rule
//!
//! **No Gray without evidence.**
//! A collection cannot be `Dead` unless the analyzer found real problems in it.
//! A file cannot be `Dead` unless it has `DeadEvidence`.
//!
//! ## Pipeline
//!
//! ```text
//! walk_folder(root)
//!     → Vec<FileRecord> + ScanSummary
//!         → build_collections()
//!             → CollectionSet
//!                 → analyze_scan()
//!                     → AnalysisReport (with DeadEvidence)
//!                         → classify_collection() / classify_file()
//!                             → LaneColor
//! ```
//!
//! ## Modules
//!
//! - [`lane_color`]       — `LaneColor` enum (Gem / Active / Fuzzy / Sec / Dead / Critical)
//! - [`file_walker`]      — Pure-std recursive file system walker
//! - [`collection`]       — Groups files into sovereign collections
//! - [`analyzer`]         — Evidence-backed dead file detection
//! - [`lane_classifier`]  — Maps analysis evidence to `LaneColor`
//!
//! DUB.SAR 𒁾

#![forbid(unsafe_code)]

pub mod analyzer;
pub mod collection;
pub mod file_walker;
pub mod lane_classifier;
pub mod lane_color;

// Flat re-exports — the sovereign public surface
pub use analyzer::{analyze_scan, AnalysisReport, DeadEvidence, DeadReason};
pub use collection::{build_collections, Collection, CollectionSet};
pub use file_walker::{walk_folder, walk_folder_async, FileRecord, ScanSummary};
pub use lane_classifier::{classify_collection, classify_file};
pub use lane_color::LaneColor;
