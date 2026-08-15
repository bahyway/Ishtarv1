//! PartitionKey — the four-axis partition system per §9.1.
//!
//! Axis 1: tribe_id (κ[4..5]) — PA-15 sovereignty
//! Axis 2: uuid_hash shard    — κ[0..3] consistent hash
//! Axis 3: time epoch         — Event-Kaki birth timestamp quarter
//! Axis 4: state tier         — Hot / Warm / Cold (policy layer)

use bahyway_core::TribeId;

/// Identifies one Journal partition — the logical path for a particle's events.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartitionKey {
    pub tribe_id:    TribeId,
    pub shard:       u16,
    pub time_epoch:  u32,
}

impl PartitionKey {
    pub fn new(tribe_id: TribeId, uuid_hash: u32, time_epoch: u32, shard_count: u16) -> Self {
        let shard = (uuid_hash % shard_count as u32) as u16;
        PartitionKey { tribe_id, shard, time_epoch }
    }

    /// Format as a directory path component: "tribe_XXXX/shard_YYYY/epoch_ZZZZ"
    pub fn as_path(&self) -> String {
        format!(
            "tribe_{:04x}/shard_{:04x}/epoch_{:08x}",
            self.tribe_id.as_u16(),
            self.shard,
            self.time_epoch,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_within_range() {
        let pk = PartitionKey::new(TribeId::from_u16(0x0001), 0xDEAD_BEEF, 100, 64);
        assert!(pk.shard < 64);
    }

    #[test]
    fn deterministic_shard() {
        let pk1 = PartitionKey::new(TribeId::from_u16(0x0001), 0xDEAD_BEEF, 100, 64);
        let pk2 = PartitionKey::new(TribeId::from_u16(0x0001), 0xDEAD_BEEF, 100, 64);
        assert_eq!(pk1.shard, pk2.shard);
    }
}
