//! 𒀭𒄿𒍪𒉡 Azuga Engine — Sovereign Medical Thermal Condition Inference
//!
//! Scores IR thermal body scans against a 9-entry medical catalogue to infer
//! medical conditions and detect security-threat torso cold-shadow overlap.
//!
//! Pipeline budget: ≤ 50ms per frame.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

pub mod body;
pub mod conditions;
pub mod inference;
pub mod symmetry;

pub use body::{BodyScan, BodyTribe, BodyType, ZoneId};
pub use conditions::{EscalationLevel, MedicalCondition, ThermalSignature};
pub use inference::{AzugaEngine, CandidateScore, MedicalInferenceResult};
pub use symmetry::SymmetryAnalysis;
