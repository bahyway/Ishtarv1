//! OntoGraph — Nasaru instrument (GL-ONT-001).
//! Three rites: Reading -> Closure -> Minting.
//! Law: KAKI is address only; Mandatory EAV is spine; Optional EAV is harvest.
#![forbid(unsafe_code)]

pub mod kaki;
pub mod eav;
pub mod rites;

pub use eav::{Layer, Attribute, MANDATORY_FACETS, ONTO_OPTIONAL, DMBOK_OPTIONAL};
pub use kaki::Kaki;
pub use rites::{reading::FormalContext, closure::{Concept, Lattice}, minting::Nebuchadnezzar};
