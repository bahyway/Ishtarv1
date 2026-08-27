//! Built-in ILKUM (mandatory DAMA-DMBOK core) attributes.
//!
//! These are the seven universal assessments that every particle must carry.
//!
//! CORRECTION (2026-07-31): the header here used to claim these hashes
//! are "CRC-16/CCITT of the canonical name" — verified directly against
//! `bahyway_crc::crc16`, that's false for every one of them (the real
//! `crc16("state")` is `0x3E19`, not `0x1A4B`). They're fixed sovereign
//! attribute-hash constants, matching `story-engine::projection`'s same
//! (equally mislabeled) values — see that module and
//! `bahyway_core::mandatory_attrs` for the full accounting.

use crate::attribute::{AttrDataType, DamaAttribute};

// Re-export the well-known attr hashes (same values as story-engine::projection).
pub const ATTR_HASH_STATE: u32 = 0x1A4B; // sovereign constant, NOT crc16("state")
pub const ATTR_HASH_QUALITY: u32 = 0x2C5E; // sovereign constant, NOT crc16("quality")
pub const ATTR_HASH_COLOR_RGB: u32 = 0x3D7F; // sovereign constant, NOT crc16("color_rgb")
pub const ATTR_HASH_FRESHNESS: u32 = 0x4E89; // sovereign constant, NOT crc16("freshness")
pub const ATTR_HASH_SNAPSHOT_DATE: u32 = 0x9A1D; // sovereign constant, NOT crc16("Snapshot_Date")
pub const ATTR_HASH_SNAPSHOT_STATE: u32 = 0x7E32; // sovereign constant, NOT crc16("Snapshot_State")
pub const ATTR_HASH_SNAPSHOT_FREQUENCY: u32 = 0x4B0F; // sovereign constant, NOT crc16("Snapshot_Frequency")
pub const ATTR_HASH_MOMENTUM: u32 = 0xD3A1; // sovereign constant, NOT crc16("momentum") — PA-13

/// The complete set of built-in ILKUM attributes.
pub const BUILTIN_ATTRS: &[DamaAttribute] = &[
    DamaAttribute::new(
        ATTR_HASH_STATE,
        "state",
        AttrDataType::Text,
        true,
        "Lifecycle state: GOLDEN | FUZZY | DEAD",
    ),
    DamaAttribute::new(
        ATTR_HASH_QUALITY,
        "quality",
        AttrDataType::Float64,
        true,
        "Normalised quality score [0.0, 1.0]; thresholds: 0.2=DEAD, 0.6=GOLDEN",
    ),
    DamaAttribute::new(
        ATTR_HASH_COLOR_RGB,
        "color_rgb",
        AttrDataType::Bytes,
        true,
        "24-bit RGB color for DubSar particle visualisation",
    ),
    DamaAttribute::new(
        ATTR_HASH_FRESHNESS,
        "freshness",
        AttrDataType::Float64,
        true,
        "Temporal freshness score [0.0, 1.0]; decays via energy model E(t)=E₀e^(-λΔt)",
    ),
    DamaAttribute::new(
        ATTR_HASH_SNAPSHOT_DATE,
        "Snapshot_Date",
        AttrDataType::Int64,
        false,
        "Unix epoch of the last snapshot capture",
    ),
    DamaAttribute::new(
        ATTR_HASH_SNAPSHOT_STATE,
        "Snapshot_State",
        AttrDataType::Text,
        false,
        "State at snapshot time: GOLDEN | FUZZY | DEAD",
    ),
    DamaAttribute::new(
        ATTR_HASH_SNAPSHOT_FREQUENCY,
        "Snapshot_Frequency",
        AttrDataType::Int64,
        false,
        "Target snapshot interval in seconds",
    ),
    DamaAttribute::new(
        ATTR_HASH_MOMENTUM,
        "momentum",
        AttrDataType::Float64,
        false,
        "PA-13 Trajectory Momentum θ(p) = |Δhepta/Δt|; derived, never stored in KAKI",
    ),
];
