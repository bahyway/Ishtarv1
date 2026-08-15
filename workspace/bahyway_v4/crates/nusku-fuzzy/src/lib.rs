//! 𒀭𒉪 Nusku Fuzzy Engine — Sovereign Mamdani Thermal Threat Disambiguation
//!
//! Classifies thermal body scans into sovereign states:
//! Clear / Fever / Stress / MedicalOther / Ambiguous / SecurityThreat
//!
//! Note: kept separate from `fuzzy-engine` (MDM data-quality scoring).
//! This crate is Nusku-specific; `fuzzy-engine` is the generic MDM engine.
//!
//! DUB.SAR 𒁾 — BahyWay.Ecosystem v4.0 | Pure Rust

pub mod defuzz;
pub mod engine;
pub mod membership;
pub mod rules;

pub use defuzz::centroid_defuzz;
pub use engine::{FuzzyEngine, FuzzyState, FuzzyVerdict};
pub use membership::{LinguisticVar, MembershipFn};
pub use rules::{Antecedent, Connective, Consequent, FuzzyRule, nusku_rule_base};
