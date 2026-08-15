//! enkidb-dictionary — TĒMĒNU Dictionary v4.0
//!
//! The TĒMĒNU is the "Foundation Stone" of the EAV model.  It maps
//! human-readable attribute names to their sovereign u32 hash (CRC-16/CCITT
//! of the canonical name string, extended to u32 by zero-padding the high word).
//!
//! Key design decisions:
//!   - HashMap<u32, DamaAttribute> — O(1) lookup by attr_hash (NOT positional offset)
//!   - HashMap<&str, u32> aliases — secondary lookup by human name
//!   - No positional Vec<f64> offsets — that is the v3.x model; v4.0 uses named keys
//!   - Zero third-party dependencies
//!
//! The mandatory DAMA-DMBOK attributes (ILKUM core) are pre-loaded at startup.
//! Domain teams add SHU-GUR attributes via `register_attr()`.

pub mod attribute;
pub mod dictionary;
pub mod builtin;

pub use attribute::{AttrDataType, DamaAttribute};
pub use dictionary::HeptaDictionary;
pub use builtin::{
    BUILTIN_ATTRS,
    ATTR_HASH_STATE,
    ATTR_HASH_QUALITY,
    ATTR_HASH_COLOR_RGB,
    ATTR_HASH_FRESHNESS,
    ATTR_HASH_SNAPSHOT_DATE,
    ATTR_HASH_SNAPSHOT_STATE,
    ATTR_HASH_SNAPSHOT_FREQUENCY,
    ATTR_HASH_MOMENTUM,
};

pub mod prelude {
    pub use super::attribute::{AttrDataType, DamaAttribute};
    pub use super::dictionary::HeptaDictionary;
    pub use super::builtin::ATTR_HASH_STATE;
}
