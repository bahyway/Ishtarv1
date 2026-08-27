//! AnuGovernor (formerly Shakkanakku) — the governor's engine, exposed as a
//! library so both the native egui face (`app`, feature "gui") and the
//! browser-facing HTTP server (`bin/anu_governor_web.rs`, feature "web")
//! drive the exact same config/runner/remedy/report/chronicle code. Neither
//! face owns the law; they only render it.

pub mod chronicle;
pub mod config;
pub mod crate_mint;
pub mod domain_review;
pub mod gate_review;
pub mod model;
pub mod pb_catalog;
pub mod pb_catalog_rebuild;
pub mod pb_dependency_review;
pub mod pb_doc_mint;
pub mod pb_mint;
pub mod pb_story;
pub mod remedy;
pub mod report;
pub mod resource_check;
pub mod runner;
pub mod tablet_mint;

#[cfg(feature = "gui")]
pub mod app;
// Hala's docs Reform & Pulse engine (sovereign name as of Phase 2
// reconciliation, 2026-08-15; prior name "Uruinimgina" -- see
// docs/99_index/BAHYWAY_PHASE2_GLOSSARY.md). Historically driven only from the
// Hala tab in `app` (feature "gui") -- ungated 2026-08-01 so
// `bin/uruinimgina_cli.rs` (no "gui" feature required, binary name kept
// unchanged for compatibility with playbook_278/281) can drive the same
// engine headlessly, e.g. on a host running document ingestion without a
// windowing session. docpulse itself has no GUI dependency of its own.
pub mod docpulse;

#[cfg(feature = "web")]
pub mod web_auth;
#[cfg(feature = "web")]
pub mod web_tls;
