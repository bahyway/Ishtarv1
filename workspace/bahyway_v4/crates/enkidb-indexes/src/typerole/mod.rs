//! Index 3 — Type+Role Index: 3×3 KAKI taxonomy (§9.3)
//! Answers "all Event-Kakis in tribe X", "all PARZU-KAKIs in templates", etc.

use std::collections::{HashMap, HashSet};
use bahyway_core::TribeId;
use enkidb_kaki::{KakiRole, KakiType};

/// 3×3 taxonomy cell: (KakiType, KakiRole) → set of uuid_hashes (sorted only
/// on read -- see SovereigntyIndex::insert's doc comment for why this isn't
/// a sorted Vec: every particle sharing a (tribe, type, role) -- the common
/// case for a bulk single-tribe seed -- used to land in one growing bucket
/// with an O(bucket size) `Vec::insert(pos, ..)` per write.
pub struct TypeRoleIndex {
    cells: HashMap<(TribeId, KakiType, KakiRole), HashSet<u32>>,
}

impl TypeRoleIndex {
    pub fn new() -> Self {
        TypeRoleIndex { cells: HashMap::new() }
    }

    pub fn insert(&mut self, tribe_id: TribeId, kaki_type: KakiType, role: KakiRole, uuid_hash: u32) {
        self.cells.entry((tribe_id, kaki_type, role)).or_default().insert(uuid_hash);
    }

    /// All uuid_hashes with a given type+role within a tribe, sorted.
    pub fn query(&self, tribe_id: TribeId, kaki_type: KakiType, role: KakiRole) -> Vec<u32> {
        let mut v: Vec<u32> = self
            .cells
            .get(&(tribe_id, kaki_type, role))
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        v.sort_unstable();
        v
    }
}

impl Default for TypeRoleIndex { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taxonomy_separation() {
        let mut idx = TypeRoleIndex::new();
        let t = TribeId::from_u16(0x0001);
        idx.insert(t, KakiType::Identity,   KakiRole::Zikru,  0x0001);
        idx.insert(t, KakiType::Identity,   KakiRole::Parzu,  0x0002);
        idx.insert(t, KakiType::Event,      KakiRole::Zikru,  0x0003);

        assert_eq!(idx.query(t, KakiType::Identity, KakiRole::Zikru).len(),  1);
        assert_eq!(idx.query(t, KakiType::Identity, KakiRole::Parzu).len(),  1);
        assert_eq!(idx.query(t, KakiType::Event,    KakiRole::Zikru).len(),  1);
        assert_eq!(idx.query(t, KakiType::Event,    KakiRole::Parzu).len(),  0);
    }
}
