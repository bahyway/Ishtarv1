//! KakiNucleusBlock — the dense 16-bytes-per-slot KAKI nucleus store (§9.3).
//!
//! Slot assignment: determined by uuid_hash — no gaps.
//! A shard owns a contiguous range of uuid_hash values.

use bahyway_core::Result;
use enkidb_kaki::Kaki;

/// In-memory representation of one block's worth of KAKI nuclei.
/// Backed by a `Vec<[u8; 16]>` for O(1) slot access.
pub struct KakiNucleusBlock {
    /// Raw 16-byte KAKI slots.  Index = relative slot within this block.
    slots: Vec<[u8; 16]>,
}

impl KakiNucleusBlock {
    pub fn new(capacity: usize) -> Self {
        KakiNucleusBlock { slots: vec![[0u8; 16]; capacity] }
    }

    /// Write a KAKI into the given slot (overwrite only allowed for zero slots).
    /// Returns `Err` if the slot already contains a non-zero KAKI (Rule II).
    pub fn write(&mut self, slot: usize, kaki: &Kaki) -> Result<()> {
        if self.slots[slot] != [0u8; 16] {
            return Err(bahyway_core::BahywayError::KakiReassignmentForbidden);
        }
        self.slots[slot].copy_from_slice(kaki.bytes());
        Ok(())
    }

    /// Read and validate the KAKI at the given slot.
    pub fn read(&self, slot: usize) -> Result<Kaki> {
        Kaki::from_bytes(self.slots[slot])
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Serialize all slots to raw bytes (contiguous 16*N byte array).
    pub fn to_bytes(&self) -> Vec<u8> {
        self.slots.iter().flat_map(|s| s.iter().copied()).collect()
    }
}
