//! ninsun-steward-bridge — NINSUN's advisory inbox for the Data Steward
//! (Stage 2b, draft §BC-NINSUN-001).
//!
//! NINSUN (`ninsun-agent`) detects pattern and semantic anomalies — stuck
//! sensors, sustained FUZZY/DEAD drift, schema/semantic drift — that
//! deterministic rules cannot easily express, and emits `RefineProposal`
//! particles: advisory only, carrying a confidence score and evidence.
//! NINSUN never modifies a committed particle and never blocks the
//! pipeline; this crate turns a batch of its proposals into the
//! confidence-ordered queue the Data Steward actually works through, and
//! journals every confirm/reject decision so the Steward's judgment stays
//! auditable in the NĀRU (Journal).
//!
//! This crate is deliberately the only place that depends on both
//! `ninsun-agent` (AI-adjacent) and the deterministic pipeline crates —
//! keeping the AI dependency out of `data-steward-station` itself.
//!
//! NINSUN surfaces, the Data Steward decides, KISPU commits.

pub mod advisory_queue;
pub mod hazard;

pub use advisory_queue::{AdvisoryCase, AdvisoryDecision, NinsunAdvisoryQueue};
pub use hazard::{classify_hazard, HazardDomain, UrgencyKey, ESARHADDON_TRIBE_RANGE};

pub mod prelude {
    pub use super::advisory_queue::{AdvisoryCase, AdvisoryDecision, NinsunAdvisoryQueue};
    pub use super::hazard::{classify_hazard, HazardDomain, UrgencyKey};
}
