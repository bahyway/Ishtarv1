//! AkkValue — Sovereign EAV value type (31 variants).
//! 𒀊𒆪 *akkû* — "sovereign quality / precious thing".
//!
//! Zero external dependencies. No serde. No uuid crate. Pure Rust.

#![forbid(unsafe_code)]
#![allow(dead_code)]

pub mod codec;
pub mod types;
pub mod value;

pub use codec::{decode as decode_value, encode as encode_value, CodecError};
pub use types::{
    AkkCoordinate, AkkDate, AkkHijriDate, AkkNamePartLabel, AkkNameVector, AkkNationalId, AkkPhone,
    CipherAlgorithm, PipelineStatus, PolicyVerdict,
};
pub use value::AkkValue;

/// Sovereign EAV triple: entity + attribute + value.
#[derive(Debug, Clone)]
pub struct AkkTriple {
    pub entity: String,
    pub attribute: String,
    pub value: AkkValue,
}

impl AkkTriple {
    pub fn new(entity: impl Into<String>, attribute: impl Into<String>, value: AkkValue) -> Self {
        Self {
            entity: entity.into(),
            attribute: attribute.into(),
            value,
        }
    }
}
