//! enkidb-kaki — The 16-byte KAKI sovereign identity for BahyWay v4.0
//!
//! The KAKI is born once and never altered.  All "change" lives in the Orbit.
//! See KAKI_v4.0.pdf §1–§2 for the canonical specification.

pub mod kaki;
pub mod mint;
pub mod newtypes;
pub mod pattern;
pub mod types;

pub use kaki::Kaki;
pub use mint::KakiMinter;
pub use newtypes::{CrossTribeKaki, EventKaki, IdentityKaki, PatternKaki};
pub use pattern::{
    derive_pattern_kaki, pattern_kaki_confidence, FixedCoord7D, PatternLifecycle, PatternType,
};
pub use types::{KakiRole, KakiType};

pub mod prelude {
    pub use super::kaki::Kaki;
    pub use super::mint::KakiMinter;
    pub use super::newtypes::{CrossTribeKaki, EventKaki, IdentityKaki, PatternKaki};
    pub use super::pattern::{
        derive_pattern_kaki, pattern_kaki_confidence, FixedCoord7D, PatternLifecycle, PatternType,
    };
    pub use super::types::{KakiRole, KakiType};
}
