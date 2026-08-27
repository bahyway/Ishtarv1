//! Index 4 — Temporal Index (§9.3)
//! Answers "what happened in this time window" — supports time-travel queries (§3.4).
//!
//! Tree-free by construction: the timestamp is a `u16` (κ[12..13], compressed
//! birth epoch), a fixed 65,536-value domain regardless of particle count. That
//! bound means a direct-indexed array outperforms any tree here -- O(1) bucket
//! access instead of O(log 65536), zero pointer-chasing, and it can never grow
//! into the failure mode that made the old BTreeRange index cost 12GB at 1B
//! particles (a tree with one node per *particle*, not per timestamp value).
//! This index was previously described as a "native LSM-tree variant" and
//! implemented with `BTreeMap<u16, Vec<u32>>` -- accurate in structure (bounded,
//! read-only after batch build, safe under concurrent reads with zero lock
//! contention) but the LSM/MemTable/SSTable language overclaimed what was
//! actually there: a single-level bucket map, no compaction, no disk levels.
//! Replaced with the array form both for honesty and because it is strictly
//! faster with no downside -- matching the flat, direct/computed-index idiom
//! `surrogate`/`radix_spline`/`hepta_shell` already use elsewhere in this crate.

const BUCKET_COUNT: usize = 1 << 16; // u16 domain: 65,536 timestamp buckets

/// A (timestamp, uuid_hash) entry.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemporalEntry {
    pub timestamp: u16,
    pub uuid_hash: u32,
}

/// Direct-indexed epoch bucket array. No tree anywhere.
pub struct TemporalIndex {
    buckets: Vec<Vec<u32>>,
}

impl TemporalIndex {
    pub fn new() -> Self {
        TemporalIndex {
            buckets: vec![Vec::new(); BUCKET_COUNT],
        }
    }

    pub fn insert(&mut self, timestamp: u16, uuid_hash: u32) {
        self.buckets[timestamp as usize].push(uuid_hash);
    }

    /// Return all uuid_hashes born in the timestamp range [from, to] inclusive.
    pub fn range_query(&self, from: u16, to: u16) -> Vec<u32> {
        self.buckets[from as usize..=to as usize]
            .iter()
            .flat_map(|hashes| hashes.iter().copied())
            .collect()
    }

    /// Point query: uuid_hashes born at exactly `timestamp`.
    pub fn at(&self, timestamp: u16) -> &[u32] {
        &self.buckets[timestamp as usize]
    }
}

impl Default for TemporalIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_query() {
        let mut idx = TemporalIndex::new();
        idx.insert(100, 0xAAAA);
        idx.insert(200, 0xBBBB);
        idx.insert(300, 0xCCCC);
        let r = idx.range_query(100, 200);
        assert!(r.contains(&0xAAAA));
        assert!(r.contains(&0xBBBB));
        assert!(!r.contains(&0xCCCC));
    }

    #[test]
    fn point_query() {
        let mut idx = TemporalIndex::new();
        idx.insert(42, 0x1111);
        idx.insert(42, 0x2222);
        assert_eq!(idx.at(42).len(), 2);
        assert!(idx.at(99).is_empty());
    }

    #[test]
    fn boundary_timestamps_are_valid() {
        let mut idx = TemporalIndex::new();
        idx.insert(0, 0x0001);
        idx.insert(u16::MAX, 0x0002);
        assert_eq!(idx.at(0), &[0x0001]);
        assert_eq!(idx.at(u16::MAX), &[0x0002]);
        assert_eq!(idx.range_query(0, u16::MAX).len(), 2);
    }
}
