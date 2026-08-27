//! score-engine — Quality scoring and ColorID computation (§4.2, §4.5)
//!
//! Computes the three assessment dimensions for a particle's color_rgb EAV:
//!   RED   (D3) — domain classification (horizontal, set by template)
//!   GREEN (D4) — quality (HPS score 0.0–1.0)
//!   BLUE  (D5) — freshness (δₜ metric, 0.0–1.0, decays over time)
//!
//! ColorID is algebra-free diagnosis — pattern recognition over drift
//! signatures, not algebraic equations (§4.5).

pub mod color_id;
pub mod freshness;
pub mod fuzzy_scorer;
pub mod hps;

pub use color_id::{compute_color, ColorRgb};
pub use freshness::FreshnessDecay;
pub use fuzzy_scorer::{score, tier_to_state, ScoreInput, ScoreResult};
pub use hps::HpsScore;

pub mod prelude {
    pub use super::color_id::{compute_color, ColorRgb};
    pub use super::freshness::FreshnessDecay;
    pub use super::fuzzy_scorer::{score, tier_to_state, ScoreInput, ScoreResult};
    pub use super::hps::HpsScore;
}
