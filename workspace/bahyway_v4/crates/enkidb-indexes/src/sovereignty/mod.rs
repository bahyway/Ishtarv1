//! Index 2 — Sovereignty Index: tribe_id → sorted list of uuid_hashes (§9.3)
//! Answers "enumerate all particles in tribe X" in O(k) where k is tribe size.

use bahyway_core::TribeId;
use std::collections::{HashMap, HashSet};

/// Sovereignty index: tribe_id → set of uuid_hashes (sorted only on read).
pub struct SovereigntyIndex {
    tribes: HashMap<TribeId, HashSet<u32>>,
}

impl SovereigntyIndex {
    pub fn new() -> Self {
        SovereigntyIndex {
            tribes: HashMap::new(),
        }
    }

    /// O(1) average. Previously a sorted `Vec<u32>` with `Vec::insert(pos,
    /// ..)` -- correct, but every particle in a tribe landed in the same
    /// one growing bucket, so a bulk single-tribe seed (e.g. bin/enkidb-seed)
    /// paid an O(bucket size) shift per insert: O(n^2) overall, discovered
    /// when seeding 10M particles into one tribe visibly decelerated
    /// (18.9K/s -> 3.9K/s by 1M written). A `HashSet` makes insert O(1)
    /// average; sorting is deferred to `tribe_members()`, a read-time cost
    /// instead of a write-time one.
    pub fn insert(&mut self, tribe_id: TribeId, uuid_hash: u32) {
        self.tribes.entry(tribe_id).or_default().insert(uuid_hash);
    }

    /// All uuid_hashes in this tribe, sorted. O(k log k) per call.
    pub fn tribe_members(&self, tribe_id: TribeId) -> Vec<u32> {
        let mut v: Vec<u32> = self
            .tribes
            .get(&tribe_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default();
        v.sort_unstable();
        v
    }

    pub fn tribe_count(&self, tribe_id: TribeId) -> usize {
        self.tribes.get(&tribe_id).map(HashSet::len).unwrap_or(0)
    }
}

impl Default for SovereigntyIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_insert() {
        let mut idx = SovereigntyIndex::new();
        let t = TribeId::from_u16(0x0001);
        idx.insert(t, 300);
        idx.insert(t, 100);
        idx.insert(t, 200);
        assert_eq!(idx.tribe_members(t), &[100, 200, 300]);
    }

    #[test]
    fn no_duplicates() {
        let mut idx = SovereigntyIndex::new();
        let t = TribeId::from_u16(0x0002);
        idx.insert(t, 42);
        idx.insert(t, 42);
        assert_eq!(idx.tribe_count(t), 1);
    }

    #[test]
    fn unknown_tribe_is_empty() {
        let idx = SovereigntyIndex::new();
        assert!(idx.tribe_members(TribeId::from_u16(0xFFFF)).is_empty());
    }

    /// Regression for the O(n^2) bulk-insert bug: 200K inserts into one
    /// tribe (the enkidb-seed pattern) must finish in well under a second.
    /// The old sorted-Vec implementation made this quadratic -- this test
    /// would have taken tens of seconds, not milliseconds, before the fix.
    #[test]
    fn bulk_single_tribe_insert_stays_fast() {
        let mut idx = SovereigntyIndex::new();
        let t = TribeId::from_u16(0x0003);
        let t0 = std::time::Instant::now();
        for i in 0..200_000u32 {
            idx.insert(t, i);
        }
        let elapsed = t0.elapsed();
        assert_eq!(idx.tribe_count(t), 200_000);
        assert!(
            elapsed.as_secs() < 2,
            "200K single-tribe inserts took {elapsed:?} -- likely regressed to O(n^2)"
        );
    }
}
