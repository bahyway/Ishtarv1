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

pub mod lane_color;
pub mod file_walker;
pub mod collection;
pub mod analyzer;
pub mod lane_classifier;

// Flat re-exports — the sovereign public surface
pub use lane_color::LaneColor;
pub use file_walker::{FileRecord, ScanSummary, walk_folder, walk_folder_async};
pub use collection::{Collection, CollectionSet, build_collections};
pub use analyzer::{AnalysisReport, DeadEvidence, DeadReason, analyze_scan};
pub use lane_classifier::{classify_collection, classify_file};
