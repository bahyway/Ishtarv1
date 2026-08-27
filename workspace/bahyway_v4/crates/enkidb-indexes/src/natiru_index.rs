//! NatiruIndex — Orbital-range temporal pruning index.
//!
//! Guards against full O(n) EAV scans on 1B+ particles by answering
//! "which surrogate IDs have at least one journal entry in orbital range
//! [start, end]?" in O(log n + result_size) time.
//!
//! Architecture
//! ─────────────────────────────────────────────────────────────────────
//! Stores a sorted `Vec<(orbital: u64, surrogate: u32)>` (12 bytes/entry).
//! Binary search on orbital bounds → collect unique surrogates in range.
//!
//! Hot tier budget at 10 M particles × 64 entries/particle average:
//!   640 M entries × 12 B = 7.68 GB  ← use cold bucket grouping instead
//!
//! For hot session data (≤ 1 M particles, ≤ 100 entries each):
//!   100 M entries × 12 B = 1.2 GB   ← acceptable
//!
//! Bucket variant (BUCKET_ORBITALS = 10):
//!   Stores (bucket_id: u32, surrogate: u32) pairs instead.
//!   Reduces entry count by BUCKET_ORBITALS factor.

/// Orbitals grouped into one bucket to reduce index size.
/// Tune this based on how granular orbital filtering needs to be.
pub const BUCKET_ORBITALS: u64 = 10;

/// NatiruIndex: temporal surrogate index for orbital-range pruning.
#[derive(Debug, Default)]
pub struct NatiruIndex {
    /// Sorted by (bucket_id, surrogate).  One entry per (bucket, particle).
    entries: Vec<(u32, u32)>,
    /// Orbital value of the very first bucket (orbit 0 offset).
    epoch_orbital: u64,
}

impl NatiruIndex {
    /// Create an empty index with the given epoch offset.
    pub fn new(epoch_orbital: u64) -> Self {
        Self {
            entries: Vec::new(),
            epoch_orbital,
        }
    }

    /// Insert a (orbital, surrogate) pair.
    ///
    /// Call this for every journal entry written.  Duplicates within the
    /// same bucket are collapsed at `seal()` time.
    pub fn insert(&mut self, orbital: u64, surrogate: u32) {
        let bucket = self.bucket_for(orbital);
        self.entries.push((bucket, surrogate));
    }

    /// Seal the index: sort and deduplicate.
    ///
    /// Must be called once after all `insert()` calls and before any query.
    pub fn seal(&mut self) {
        self.entries.sort_unstable();
        self.entries.dedup();
    }

    /// Build from a batch of `(orbital, surrogate)` pairs (already sorted or not).
    pub fn build(epoch_orbital: u64, pairs: &[(u64, u32)]) -> Self {
        let mut idx = Self::new(epoch_orbital);
        for &(orb, sur) in pairs {
            idx.insert(orb, sur);
        }
        idx.seal();
        idx
    }

    /// Return all unique surrogates that have data in the orbital range
    /// `[start_orbital, end_orbital]` (inclusive).
    ///
    /// Returns `None` if the index is empty (caller should fall back to full scan).
    pub fn surrogates_in_range(&self, start_orbital: u64, end_orbital: u64) -> Option<Vec<u32>> {
        if self.entries.is_empty() {
            return None;
        }

        let lo_bucket = self.bucket_for(start_orbital);
        let hi_bucket = self.bucket_for(end_orbital);

        // Binary-search for the first entry with bucket >= lo_bucket
        let start_idx = self.entries.partition_point(|&(b, _)| b < lo_bucket);
        // Binary-search for the first entry with bucket > hi_bucket
        let end_idx = self.entries.partition_point(|&(b, _)| b <= hi_bucket);

        if start_idx >= end_idx {
            return Some(Vec::new());
        }

        // Collect unique surrogates from the slice
        let mut result: Vec<u32> = self.entries[start_idx..end_idx]
            .iter()
            .map(|&(_, s)| s)
            .collect();

        result.sort_unstable();
        result.dedup();
        Some(result)
    }

    /// Intersect the range result with an existing surrogate candidate set.
    ///
    /// Both slices must be sorted.  Returns the intersection (AND semantics).
    pub fn intersect_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().min(b.len()));
        let (mut i, mut j) = (0, 0);
        while i < a.len() && j < b.len() {
            match a[i].cmp(&b[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    out.push(a[i]);
                    i += 1;
                    j += 1;
                }
            }
        }
        out
    }

    /// Check whether a specific (orbital, surrogate) pair is indexed.
    pub fn contains(&self, orbital: u64, surrogate: u32) -> bool {
        let bucket = self.bucket_for(orbital);
        self.entries.binary_search(&(bucket, surrogate)).is_ok()
    }

    /// Number of (bucket, surrogate) entries after sealing.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if no entries have been inserted.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Approximate memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        self.entries.len() * 8 + std::mem::size_of::<Self>()
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    #[inline]
    fn bucket_for(&self, orbital: u64) -> u32 {
        let relative = orbital.saturating_sub(self.epoch_orbital);
        (relative / BUCKET_ORBITALS) as u32
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_range_query() {
        // Particles 0-9 exist at orbital 100
        // Particles 5-14 exist at orbital 200
        let mut idx = NatiruIndex::new(0);
        for s in 0..10u32 {
            idx.insert(100, s);
        }
        for s in 5..15u32 {
            idx.insert(200, s);
        }
        idx.seal();

        // Query [95..105] → bucket covers orbital 100 → surrogates 0-9
        let r = idx.surrogates_in_range(95, 105).unwrap();
        assert!(r.contains(&0));
        assert!(r.contains(&9));
        assert!(!r.contains(&10));

        // Query [95..205] → both buckets → surrogates 0-14
        let r2 = idx.surrogates_in_range(95, 205).unwrap();
        assert_eq!(r2, (0..15).collect::<Vec<_>>());
    }

    #[test]
    fn range_outside_index_returns_empty() {
        let idx = NatiruIndex::build(0, &[(100, 1), (100, 2)]);
        let r = idx.surrogates_in_range(500, 600).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn empty_index_returns_none() {
        let idx = NatiruIndex::new(0);
        assert!(idx.surrogates_in_range(0, 1000).is_none());
    }

    #[test]
    fn intersect_sorted_works() {
        let a = vec![1u32, 3, 5, 7, 9];
        let b = vec![2u32, 3, 5, 8, 9];
        let r = NatiruIndex::intersect_sorted(&a, &b);
        assert_eq!(r, vec![3, 5, 9]);
    }

    #[test]
    fn dedup_same_bucket() {
        // Multiple insertions of same (orbital, surrogate) should be collapsed
        let mut idx = NatiruIndex::new(0);
        for _ in 0..100 {
            idx.insert(50, 42);
        }
        idx.seal();
        assert_eq!(idx.len(), 1);
    }

    #[test]
    fn contains_exact() {
        let idx = NatiruIndex::build(0, &[(10, 5), (20, 7)]);
        assert!(idx.contains(10, 5));
        assert!(idx.contains(20, 7));
        assert!(!idx.contains(10, 7));
        // Orbital 15 is in the same bucket as orbital 10 (bucket = 15/10 = 1),
        // so contains() returns true — bucket-level granularity is by design.
        assert!(idx.contains(15, 5));
        // A completely different bucket (orbital 50 → bucket 5) must not match.
        assert!(!idx.contains(50, 5));
    }

    #[test]
    fn memory_estimate_reasonable() {
        let pairs: Vec<(u64, u32)> = (0..1_000u32).map(|i| (i as u64 * 10, i)).collect();
        let idx = NatiruIndex::build(0, &pairs);
        // Expect: 1000 entries × 8 bytes ≈ 8 KB
        assert!(idx.memory_bytes() < 64_000);
    }
}
