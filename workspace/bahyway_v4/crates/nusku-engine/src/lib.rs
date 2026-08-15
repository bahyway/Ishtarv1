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
pub use types::{KakiPK, KAKI_VOID, Hepta, BodyTribe, ZoneId, BodyType};
pub use kaki::{derive_kaki, kaki_to_hex};
pub use particle::{ParticleSignal, BodyScan};
pub use result::{ScanResult, PrimaryFinding, FaceMatchResult};
pub use pipeline::{
    PipelineContext, PipelineTiming, PipelineBudgets,
    NuskuAlert, PIPELINE_BUDGET_MS,
};
pub use database::{DatabaseConnector, DatabaseStatus, AuthorityLookupResult};
