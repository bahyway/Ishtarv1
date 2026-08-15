//! enkidb-kaki — The 16-byte KAKI sovereign identity for BahyWay v4.0
//!
//! The KAKI is born once and never altered.  All "change" lives in the Orbit.
//! See KAKI_v4.0.pdf §1–§2 for the canonical specification.

pub mod kaki;
pub mod types;
pub mod newtypes;
pub mod mint;
pub mod pattern;

pub use kaki::Kaki;
pub use types::{KakiType, KakiRole};
pub use newtypes::{IdentityKaki, EventKaki, CrossTribeKaki, PatternKaki};
pub use mint::KakiMinter;
pub use pattern::{
    FixedCoord7D, PatternType, PatternLifecycle,
    derive_pattern_kaki, pattern_kaki_confidence,
};

pub mod prelude {
    pub use super::kaki::Kaki;
    pub use super::types::{KakiType, KakiRole};
    pub use super::newtypes::{IdentityKaki, EventKaki, CrossTribeKaki, PatternKaki};
    pub use super::mint::KakiMinter;
    pub use super::pattern::{
        FixedCoord7D, PatternType, PatternLifecycle,
        derive_pattern_kaki, pattern_kaki_confidence,
    };
}
