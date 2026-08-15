#![forbid(unsafe_code)]
//! SLA Governance Engine — fuzzy compliance scoring + stakeholder GUI.
//!
//! Provides configurable compliance governance for every BahyWay Enterprise App.
//!
//! # Architecture
//!
//! ```text
//! Stakeholder (Admin / Data Steward)
//!         │
//!         ▼
//! SlaGuiRenderer::render_selection_html()   ← browser form
//!         │  (submits MaturityLevel per requirement)
//!         ▼
//! SlaEvaluator::evaluate(profile, state)    ← fuzzy scoring
//!         │
//!         ▼
//! SlaReport { domain_scores, overall_b11, go_live_ready, actions }
//!         │
//!         ▼
//! SlaGuiRenderer::render_dashboard_html()   ← compliance dashboard
//! ```
//!
//! # Configurable Topology
//!
//! Use `AppTopology` to select a pre-built requirement profile for each app:
//! ```rust,no_run
//! use sla_engine::{SlaProfile, AppTopology, ComplianceState, SlaEvaluator, SlaGuiRenderer};
//! let profile = SlaProfile::for_topology(AppTopology::NajafEngine);
//! let state   = ComplianceState::new(); // administrator fills this in
//! let report  = SlaEvaluator::evaluate(&profile, &state, 0);
//! let html    = SlaGuiRenderer::render_dashboard_html(&report);
//! ```
//!
//! # Fuzzy Scoring (B11 / QUALITY_DIVISOR=240)
//!
//! Each requirement is scored on a 0.0–1.0 fuzzy scale (MaturityLevel).
//! The domain score = weighted average of all requirement scores × 240.
//! The overall score = weighted average of all domain scores × 240.
//!
//! GEM threshold (go-live safe) = 180/240 (75%).

pub mod requirements;
pub mod profile;
pub mod evaluator;
pub mod gui;

pub use requirements::{ComplianceDomain, SlaRequirement, ALL_REQUIREMENTS, requirement_by_id};
pub use profile::{AppTopology, MaturityLevel, SlaProfile};
pub use evaluator::{ComplianceState, ComplianceCheckResult, DomainScore, SlaReport, SlaEvaluator};
pub use gui::SlaGuiRenderer;

/// B11 quality divisor — must match ADR-001 (QUALITY_DIVISOR = 240).
pub const QUALITY_DIVISOR: u8 = 240;

/// Minimum B11 score to be considered go-live safe (75%).
pub const GO_LIVE_THRESHOLD: u8 = 180;

/// Minimum B11 score for a mandatory requirement to not block go-live.
pub const MANDATORY_THRESHOLD: u8 = 192; // 80% — mandatory items must score higher
