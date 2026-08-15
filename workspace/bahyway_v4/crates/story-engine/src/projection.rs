//! Projection helpers — well-known mandatory EAV attribute hashes.
//!
//! CORRECTION (2026-07-31): these constants were long labeled
//! `// crc16("name")`, but verified directly against the real
//! `bahyway_crc::crc16` function, every single one of those labels is
//! false (e.g. the real `crc16("state")` is `0x3E19`, not `0x1A4B`).
//! They are fixed sovereign attribute-hash constants, not derived CRC16
//! outputs — kept at their existing literal values because they're
//! already depended on across dozens of crates' EAV rows, tests, and
//! disk-persisted formats (see `bahyway_core::mandatory_attrs` for the
//! full accounting). Attribute hashes are also the index keys in Index 6
//! (EAV index, §9.3).

// Mandatory universal EAV attributes — The Hepta of Universal Assessments
pub const ATTR_STATE:              u32 = 0x1A4B; // sovereign constant, NOT crc16("state")
pub const ATTR_QUALITY:            u32 = 0x2C5E; // sovereign constant, NOT crc16("quality")
pub const ATTR_COLOR_RGB:          u32 = 0x3D7F; // sovereign constant, NOT crc16("color_rgb")
pub const ATTR_FRESHNESS:          u32 = 0x4E89; // sovereign constant, NOT crc16("freshness")
pub const ATTR_SNAPSHOT_DATE:      u32 = 0x9A1D; // sovereign constant, NOT crc16("Snapshot_Date")
pub const ATTR_SNAPSHOT_STATE:     u32 = 0x7E32; // sovereign constant, NOT crc16("Snapshot_State")
pub const ATTR_SNAPSHOT_FREQUENCY: u32 = 0x4B0F; // sovereign constant, NOT crc16("Snapshot_Frequency")

// StoryWay temporal query attributes — per-event color and cause tracing
pub const ATTR_COLOR_ID_SNAPSHOT: u32 = 0x5F3A; // sovereign constant, NOT crc16("color_id_snapshot")
pub const ATTR_COLOR_DRIFT:        u32 = 0x6C2B; // sovereign constant, NOT crc16("color_drift")
pub const ATTR_EVENT_CAUSE:        u32 = 0x7D1C; // sovereign constant, NOT crc16("event_cause")
pub const ATTR_SOURCE_KAKI:        u32 = 0x8E0D; // sovereign constant, NOT crc16("source_kaki")

/// Decoded particle state from the `state` EAV attribute value bytes.
pub fn decode_state(bytes: &[u8]) -> bahyway_core::ParticleState {
    match bytes {
        b"GOLDEN" => bahyway_core::ParticleState::Golden,
        b"FUZZY"  => bahyway_core::ParticleState::Fuzzy,
        b"DEAD"   => bahyway_core::ParticleState::Dead,
        _         => bahyway_core::ParticleState::Fuzzy, // unknown → conservative
    }
}

/// Encode a particle state to its canonical EAV bytes.
pub fn encode_state(s: bahyway_core::ParticleState) -> &'static [u8] {
    match s {
        bahyway_core::ParticleState::Golden => b"GOLDEN",
        bahyway_core::ParticleState::Fuzzy  => b"FUZZY",
        bahyway_core::ParticleState::Dead   => b"DEAD",
    }
}
