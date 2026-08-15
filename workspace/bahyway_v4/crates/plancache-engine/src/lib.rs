//! plancache-engine — sealed, compile-once query execution plans + journal
//! projection cache. From the iPhone-session PB-158/159 review (TPL-001
//! sections A/B), landed with two corrections found during review:
//!
//! 1. Renamed "Template" -> `SealedPlan`. The original design called a
//!    compiled query-execution-plan artifact a "Template" — but this
//!    workspace's real `template-engine::Template` already means something
//!    different (an EAV field-spec / data-model definition, e.g. "what
//!    attributes does a Heart particle have"). Reusing the word for a
//!    compiled query plan would collide in code and conversation.
//! 2. The original admission law required "every access path is
//!    index-covered (Anu four-layer stack or HeptaShellIndex)" before a
//!    plan may seal. `HeptaShellIndex` is real but not yet wired to any
//!    live consumer in this workspace (confirmed by reading its own
//!    module), so hard-coding a dependency on it would block sealing
//!    nearly every 7D-spatial plan today. This crate keeps the actual LAW
//!    — a plan that would degrade to a raw full-journal scan may never be
//!    sealed — but the coverage check is a caller-supplied fact per access
//!    path (`AccessPath::IndexCovered { index_name }` vs.
//!    `AccessPath::FullScan`), not a hard dependency on one specific index.
//!    Wiring a real index-coverage checker in is separate, later work.
//!
//! Sealing itself (Ed25519) is a hook (`seal_with`), matching the same
//! pattern used elsewhere in this workspace (Tupsimati, kupru) — this
//! crate does not depend on `kupru` directly to keep it minimal; callers
//! supply the signing function.

pub mod plan;
pub mod segment;

pub use plan::{AccessPath, PlanSealError, SealedPlan, UnsealedPlan};
pub use segment::{federated_staleness_ms, Segment};
