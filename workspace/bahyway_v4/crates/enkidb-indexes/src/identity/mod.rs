//! Index 1 — Identity Index: D1 uuid_hash → file_offset (§9.3)
//! O(1) point lookup for "fetch this particle".

use std::collections::HashMap;
use bahyway_core::TribeId;

type FileOffset = u64;

/// Identity index: uuid_hash → storage file offset.
pub struct IdentityIndex {
    tribe_id: TribeId,
    map:      HashMap<u32, FileOffset>,
}

impl IdentityIndex {
    pub fn new(tribe_id: TribeId) -> Self {
        IdentityIndex { tribe_id, map: HashMap::new() }
    }

    /// Insert (uuid_hash, file_offset).  Rejects duplicate uuid_hash (Rule II).
    pub fn insert(&mut self, uuid_hash: u32, offset: FileOffset) -> bool {
        if self.map.contains_key(&uuid_hash) {
            return false;
        }
        self.map.insert(uuid_hash, offset);
        true
    }

    pub fn lookup(&self, uuid_hash: u32) -> Option<FileOffset> {
        self.map.get(&uuid_hash).copied()
    }

    pub fn tribe_id(&self) -> TribeId { self.tribe_id }
    pub fn len(&self)      -> usize   { self.map.len() }
    pub fn is_empty(&self) -> bool    { self.map.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup() {
        let mut idx = IdentityIndex::new(TribeId::from_u16(0x0001));
        assert!(idx.insert(0xDEAD_BEEF, 1024));
        assert_eq!(idx.lookup(0xDEAD_BEEF), Some(1024));
    }

    #[test]
    fn duplicate_rejected_rule_ii() {
        let mut idx = IdentityIndex::new(TribeId::from_u16(0x0001));
        assert!(idx.insert(0x1234, 100));
        assert!(!idx.insert(0x1234, 200));
        assert_eq!(idx.lookup(0x1234), Some(100));
    }

    #[test]
    fn unknown_hash_returns_none() {
        let idx = IdentityIndex::new(TribeId::from_u16(0x0001));
        assert!(idx.lookup(0xFFFF_FFFF).is_none());
    }
}
