//! Index 6 — EAV Attribute Index (§9.3)
//! Tribe-specific inverted index with native bitmaps.
//! Answers: "particles where birth_year < 1960", "graves within polygon".
//!
//! Structure: (attr_hash, value_bytes) → sorted list of uuid_hashes.
//! Per-tribe, per-attribute inverted index.

use bahyway_core::TribeId;
use std::collections::{HashMap, HashSet};

/// Inverted index key: (tribe_id, attribute_hash, value_bytes).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct EavKey {
    tribe_id: TribeId,
    attr_hash: u32,
    value: Vec<u8>,
}

/// EAV inverted index: (tribe, attr, value) → particle uuid_hashes (sorted
/// only on read -- see SovereigntyIndex::insert's doc comment: any (attr,
/// value) shared by many particles in a tribe -- e.g. a "station" attribute
/// -- used to land in one growing sorted-Vec bucket with an O(bucket size)
/// insert per write.
pub struct EavIndex {
    index: HashMap<EavKey, HashSet<u32>>,
}

impl EavIndex {
    pub fn new() -> Self {
        EavIndex {
            index: HashMap::new(),
        }
    }

    /// Index one EAV triple for a particle. O(1) average.
    pub fn insert(&mut self, tribe_id: TribeId, attr_hash: u32, value: Vec<u8>, uuid_hash: u32) {
        let key = EavKey {
            tribe_id,
            attr_hash,
            value,
        };
        self.index.entry(key).or_default().insert(uuid_hash);
    }

    /// Exact-value lookup: all particles with (tribe, attr, value), sorted.
    pub fn lookup_exact(&self, tribe_id: TribeId, attr_hash: u32, value: &[u8]) -> Vec<u32> {
        let key = EavKey {
            tribe_id,
            attr_hash,
            value: value.to_vec(),
        };
        let mut v: Vec<u32> = self
            .index
            .get(&key)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        v.sort_unstable();
        v
    }
}

impl Default for EavIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_lookup() {
        let mut idx = EavIndex::new();
        let t = TribeId::from_u16(0x0001);
        idx.insert(t, 0xABCD, b"GOLDEN".to_vec(), 0x0001);
        idx.insert(t, 0xABCD, b"GOLDEN".to_vec(), 0x0003);
        idx.insert(t, 0xABCD, b"DEAD".to_vec(), 0x0002);
        let golden = idx.lookup_exact(t, 0xABCD, b"GOLDEN");
        assert_eq!(golden, &[0x0001, 0x0003]);
        let dead = idx.lookup_exact(t, 0xABCD, b"DEAD");
        assert_eq!(dead, &[0x0002]);
    }

    #[test]
    fn unknown_attr_is_empty() {
        let idx = EavIndex::new();
        assert!(idx
            .lookup_exact(TribeId::from_u16(0x0001), 0xFFFF, b"x")
            .is_empty());
    }
}
