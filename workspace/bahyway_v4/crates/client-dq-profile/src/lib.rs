#![forbid(unsafe_code)]
//! client-dq-profile — Client-configurable Data Quality profile (BeeMDM §SLA).
//!
//! Bridges the gap between a client's uploaded schema and the fuzzy scoring engine.
//! Each client organisation defines:
//!   - Which attributes are nullable (empty = acceptable)
//!   - Minimum fill-rate threshold per attribute across the batch
//!   - Which fuzzy dimension each attribute contributes to (D1–D8)
//!   - Attribute weight in the overall quality score
//!
//! The profile is stored as a human-readable `.dqprofile` text file in the shard
//! folder (`{shard}/profiles/{batch_name}.dqprofile`).
//!
//! Pipeline:
//!   BatchSchema + client settings → ClientDqProfile
//!   ClientDqProfile + EAV triples → FuzzyDimensions
//!   FuzzyDimensions → score_engine::score() → ScoreResult (B11, State, ColorRGB, Freshness)

pub mod compute;
pub mod parse;
pub mod profile;

pub use compute::compute_dims;
pub use parse::{parse_profile, to_profile_text};
pub use profile::{AttrProfile, ClientDqProfile, DimTag, FormatLayerCode, SourceTrustLevel};
