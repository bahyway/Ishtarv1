// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  nusku-engine · src/lib.rs
//  𒀭𒉪 Nusku sovereign shared types
//  Canonical home for KakiPK, Hepta, ParticleSignal, BodyScan,
//  pipeline timing, and authority DB — pure Rust, zero external deps.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub mod database;
pub mod kaki;
pub mod particle;
pub mod pipeline;
pub mod result;
pub mod types;

// ── Flat re-exports for ergonomic use ─────────────────────────────
pub use database::{AuthorityLookupResult, DatabaseConnector, DatabaseStatus};
pub use kaki::{derive_kaki, kaki_to_hex};
pub use particle::{BodyScan, ParticleSignal};
pub use pipeline::{
    NuskuAlert, PipelineBudgets, PipelineContext, PipelineTiming, PIPELINE_BUDGET_MS,
};
pub use result::{FaceMatchResult, PrimaryFinding, ScanResult};
pub use types::{BodyTribe, BodyType, Hepta, KakiPK, ZoneId, KAKI_VOID};
