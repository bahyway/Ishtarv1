//! SnapshotRecord — the three mandatory EAV snapshot attributes for one particle.

use enkidb_kaki::IdentityKaki;
use enkidb_vector_id::VectorId;

/// Attribute name hashes for the three snapshot-related mandatory EAV attributes.
/// These are CRC-16/CCITT of the canonical attribute name strings.
pub const ATTR_SNAPSHOT_DATE:  u32 = 0x9A1D; // crc16("Snapshot_Date")
pub const ATTR_SNAPSHOT_STATE: u32 = 0x7E32; // crc16("Snapshot_State")
pub const ATTR_SNAPSHOT_FREQ:  u32 = 0x4B0F; // crc16("Snapshot_Frequency")

/// The current snapshot state for one particle.
#[derive(Debug, Clone)]
pub struct SnapshotRecord {
    /// Which particle this snapshot belongs to.
    pub particle:           IdentityKaki,
    /// Epoch (seconds since Unix epoch) when this snapshot was captured.
    pub snapshot_date:      u64,
    /// Full projected EAV state at snapshot_date — serialized as raw bytes.
    pub snapshot_state:     Vec<u8>,
    /// VectorId of the Snapshot_Job that governs this particle.
    pub snapshot_frequency: VectorId,
}

impl SnapshotRecord {
    pub fn new(
        particle:           IdentityKaki,
        snapshot_date:      u64,
        snapshot_state:     Vec<u8>,
        snapshot_frequency: VectorId,
    ) -> Self {
        SnapshotRecord { particle, snapshot_date, snapshot_state, snapshot_frequency }
    }

    /// Create a birth-time snapshot (no events yet, empty projected state).
    pub fn at_birth(particle: IdentityKaki, birth_epoch: u64) -> Self {
        SnapshotRecord {
            particle,
            snapshot_date:      birth_epoch,
            snapshot_state:     vec![],
            snapshot_frequency: VectorId::NEVER_SNAPSHOT,
        }
    }
}
