//! OntoGraph — Nasaru instrument (GL-ONT-001).
//! Three rites: Reading -> Closure -> Minting.
//! Law: KAKI is address only; Mandatory EAV is spine; Optional EAV is harvest.
#![forbid(unsafe_code)]

pub mod eav;
pub mod kaki;
pub mod rites;

pub use eav::{Attribute, Layer, DMBOK_OPTIONAL, MANDATORY_FACETS, ONTO_OPTIONAL};
pub use kaki::Kaki;
pub use rites::{
    closure::{Concept, Lattice},
    minting::Nebuchadnezzar,
    reading::FormalContext,
};
