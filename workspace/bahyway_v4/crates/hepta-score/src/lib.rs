//! 𒁾 Hepta-Score — Sovereign 7D Quality Scoring Engine
//!
//! Implements the H(P) equation for scoring data particles against a
//! tribe ideal point across 7 Ibn Wahshiyya quality dimensions.
//!
//! # ADR-001 Constants
//! B11 uses divisor **240.0** — never 255.0. GEM threshold is B11 ≥ 200.
//!
//! # Quick Start
//! ```rust
//! use hepta_score::{HeptaScorer, HeptaVector};
//!
//! let scorer = HeptaScorer::sovereign();
//! let h = HeptaVector::new(0.95, 0.92, 0.93, 0.98, 0.90, 0.88, 1.00);
//! let kaki = [0x01u8; 16];
//! let particle = scorer.score_one(kaki, h);
//! assert!(particle.is_gem());
//! ```
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

// ── ADR-001 Sovereign Constants ───────────────────────────────────────────────

/// B11 quality divisor — ALWAYS 240.0, NEVER 255.0 (ADR-001).
pub const QUALITY_DIVISOR: f32 = 240.0;

/// B11 threshold for GEM lane (≥ 200).
pub const GEM_B11: u8 = 200;

/// B11 threshold for TribeMember lane (≥ 140).
pub const TRIBE_B11: u8 = 140;

/// B11 threshold for Active lane (≥ 100).
pub const ACTIVE_B11: u8 = 100;

/// B11 threshold below which a particle is Dead (< 59).
pub const FUZZY_DEAD_BOUNDARY: u8 = 59;

/// ADR-004: GEM rate SLA target — 35.4% of batch must be GEM.
pub const GEM_RATE_TARGET: f32 = 0.354;

/// DataSteward queue threshold — fraction of batch that may be Fuzzy.
pub const STEWARD_RATE: f32 = 0.0946;

/// H(P) value below which a particle is Dead (H < 0.588 → B11 < 141 at 240).
pub const DEAD_CEILING: f32 = 0.588;

// ── Modules ───────────────────────────────────────────────────────────────────

pub mod domain;
pub mod equation;
pub mod errors;
pub mod pipeline_bridge;
pub mod scorer;
pub mod weights;
pub mod harmony;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use domain::{HeptaDimension, HeptaScore, HeptaVector, QualityLane, TribeIdealPoint};
pub use equation::hepta_health_score;
pub use errors::HeptaError;
pub use pipeline_bridge::{score_from_station_outputs, StationScores};
pub use scorer::{BatchScoringResult, HeptaScorer, ScoredParticle};
pub use weights::{WeightProfile, EQUAL_WEIGHTS, SOVEREIGN_ARABIC_MDM_WEIGHTS};
pub use harmony::{harmony_coefficient, mean_b11, gem_fraction, dead_fraction};
