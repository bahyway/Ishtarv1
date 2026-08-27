//! TIAMAT Engine 4 — Opacity Monitor
//!
//! Computes tau_full (transparency score) for a data tribe across 4 dimensions:
//!   D1: data_completeness  — fraction of expected particles received
//!   D2: source_diversity   — number of distinct sovereign sources
//!   D3: time_freshness     — staleness penalty (exponential decay)
//!   D4: schema_consistency — NRM_SCHEMA_DRIFT absence rate
//!
//! tau_full = (D1 x D2 x D3 x D4)^(1/4)   in [0.0, 1.0]
//!   1.0 = fully transparent (GOLDEN)
//!   0.5 = opacity threshold (FUZZY)
//!   0.0 = total opacity (DEAD)
//!
//! SARU = unit of accumulated transparency deficit
//!   1 SARU = 1 deficit-day (tau_full below threshold for one full day)
//!   SAHU  = measurement uncertainty modifier on SARU accumulation
#![forbid(unsafe_code)]

pub mod saru;
pub mod tau;

pub use saru::{SaruAccumulator, SaruReport};
pub use tau::{compute_tau_full, TauInput};
