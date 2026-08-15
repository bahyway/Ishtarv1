//! Index 7 — Snapshot Index (§9.3, §3.6)
//! Accelerates StoryEngine projection by jumping to the latest snapshot before time T.
//!
//! Tree-free by construction. The previous design was `BTreeMap<u32, BTreeMap<u64,
//! SnapshotIndexEntry>>` -- an OUTER tree keyed by `uuid_hash`, meaning one tree
//! node per *particle*, not per snapshot. That is the exact failure shape that
//! made the old BTreeRange index cost 12GB at 1B particles: a tree whose node
//! count scales with particle count has no bound and no cache locality.
//!
//! Replaced with `HashMap<uuid_hash, Vec<(epoch, entry)>>`, each particle's list
//! kept sorted by epoch. "Sparse" (§9.3's own word for this index) means each
//! list is typically tiny, so `latest_before` is a binary search over a handful
//! of contiguous entries -- never a tree descent over the whole index, and never
//! proportional to total particle count.

use std::collections::HashMap;

/// Entry in the snapshot index.
#[derive(Clone, Debug)]
pub struct SnapshotIndexEntry {
    /// Storage offset of the snapshot Event-Kaki / snapshot state block.
    pub cold_storage_offset: u64,
    /// Number of events replayed to produce this snapshot.
    pub events_count:        u32,
}

/// Sparse snapshot index for all particles. Tree-free by construction.
pub struct SnapshotIndex {
    particles: HashMap<u32, Vec<(u64, SnapshotIndexEntry)>>,
}

impl SnapshotIndex {
    pub fn new() -> Self {
        SnapshotIndex { particles: HashMap::new() }
    }

    /// Insert a snapshot record, keeping this particle's list sorted by epoch.
    /// An existing entry at the same epoch is replaced (matches the original
    /// BTreeMap::insert's overwrite-on-duplicate-key semantics).
    pub fn insert(&mut self, uuid_hash: u32, epoch: u64, entry: SnapshotIndexEntry) {
        let list = self.particles.entry(uuid_hash).or_default();
        match list.binary_search_by_key(&epoch, |(e, _)| *e) {
            Ok(pos) => list[pos] = (epoch, entry),
            Err(pos) => list.insert(pos, (epoch, entry)),
        }
    }

    /// Find the most recent snapshot at or before `at_epoch` for the given particle.
    /// O(log k) binary search over that one particle's own sparse list, where k
    /// is that particle's snapshot count -- never a function of total particles.
    pub fn latest_before(&self, uuid_hash: u32, at_epoch: u64) -> Option<(u64, &SnapshotIndexEntry)> {
        let list = self.particles.get(&uuid_hash)?;
        let idx = list.partition_point(|(e, _)| *e <= at_epoch);
        if idx == 0 {
            None
        } else {
            let (e, entry) = &list[idx - 1];
            Some((*e, entry))
        }
    }
}

impl Default for SnapshotIndex { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(offset: u64) -> SnapshotIndexEntry {
        SnapshotIndexEntry { cold_storage_offset: offset, events_count: 10 }
    }

    #[test]
    fn latest_snapshot_before_query_time() {
        let mut idx = SnapshotIndex::new();
        idx.insert(0x0001, 100, entry(1000));
        idx.insert(0x0001, 200, entry(2000));
        idx.insert(0x0001, 300, entry(3000));

        let (epoch, e) = idx.latest_before(0x0001, 250).unwrap();
        assert_eq!(epoch, 200);
        assert_eq!(e.cold_storage_offset, 2000);
    }

    #[test]
    fn no_snapshot_before_earliest() {
        let mut idx = SnapshotIndex::new();
        idx.insert(0x0001, 500, entry(5000));
        assert!(idx.latest_before(0x0001, 499).is_none());
    }

    #[test]
    fn unknown_particle_returns_none() {
        let idx = SnapshotIndex::new();
        assert!(idx.latest_before(0xFFFF_FFFF, 9999).is_none());
    }

    #[test]
    fn out_of_order_inserts_still_sort_correctly() {
        let mut idx = SnapshotIndex::new();
        idx.insert(0x0001, 300, entry(3000));
        idx.insert(0x0001, 100, entry(1000));
        idx.insert(0x0001, 200, entry(2000));

        assert_eq!(idx.latest_before(0x0001, 150).unwrap().0, 100);
        assert_eq!(idx.latest_before(0x0001, 250).unwrap().0, 200);
        assert_eq!(idx.latest_before(0x0001, 999).unwrap().0, 300);
    }

    #[test]
    fn duplicate_epoch_insert_overwrites_not_duplicates() {
        let mut idx = SnapshotIndex::new();
        idx.insert(0x0001, 100, entry(1111));
        idx.insert(0x0001, 100, entry(2222));

        let (epoch, e) = idx.latest_before(0x0001, 100).unwrap();
        assert_eq!(epoch, 100);
        assert_eq!(e.cold_storage_offset, 2222, "second insert at the same epoch must replace, not duplicate");
    }

    #[test]
    fn particles_are_isolated_from_each_other() {
        let mut idx = SnapshotIndex::new();
        idx.insert(0x0001, 100, entry(1000));
        idx.insert(0x0002, 100, entry(9999));

        assert_eq!(idx.latest_before(0x0001, 100).unwrap().1.cold_storage_offset, 1000);
        assert_eq!(idx.latest_before(0x0002, 100).unwrap().1.cold_storage_offset, 9999);
    }
}
