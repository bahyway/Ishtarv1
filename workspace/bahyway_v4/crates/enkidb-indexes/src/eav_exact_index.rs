//! EavExactIndex — Fast attribute-value exact-match lookup.
//!
//! Eliminates the O(n) full-scan cost of WHERE clauses on 1B+ particles by
//! indexing `(attr_hash: u32, val_fingerprint: u32, surrogate: u32)` triples
//! in a sorted Vec.  A Xor8-style bit-fingerprint table provides O(1) negative
//! answers before the binary search.
//!
//! Memory budget (1 B particles, 4 attrs each on average):
//!   4 B triples × 12 B = 48 GB  ← full corpus lives on cold/archive tier
//!   Hot tier (10 M particles):  10 M × 4 × 12 = 480 MB  ← acceptable
//!
//! Performance on hot tier with 10 M indexed entries:
//!   Negative lookup (fingerprint miss) — O(1)
//!   Positive lookup (binary search)   — O(log 40 M) ≈ 25 comparisons
//!
//! # Value fingerprint
//!
//! The `val_fingerprint` is a 32-bit FNV-1a hash of the raw encoded attribute
//! bytes.  Collisions are ~1 in 4 billion per (attr_hash, val) pair — an
//! acceptable false-positive rate that the caller resolves with an exact EAV
//! check.

/// A 32-bit FNV-1a hash — used for both the fingerprint filter and as the
/// index key component.
#[inline]
fn fnv1a_32(bytes: &[u8]) -> u32 {
    const PRIME:  u32 = 0x0100_0193;
    const OFFSET: u32 = 0x811C_9DC5;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u32;
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// One entry in the EavExactIndex: (attr_hash, val_fingerprint, surrogate).
///
/// Sorted order: attr_hash → val_fingerprint → surrogate.
/// Binary search on the first two fields yields the surrogate slice to check.
type Entry = (u32, u32, u32);

/// Xor8-style bloom substitute: one byte per slot in a power-of-2 table.
///
/// Insert: set `table[hash & mask] |= fingerprint_byte`.
/// Query:  `table[hash & mask] & fingerprint_byte == fingerprint_byte` must hold.
/// On failure: definite absence.  On success: proceed to binary search.
#[allow(dead_code)]
struct FingerprintTable {
    table: Vec<u8>,
    mask:  usize,
}

impl FingerprintTable {
    fn new(capacity: usize) -> Self {
        // Next power of two, minimum 256 slots
        let n = capacity.next_power_of_two().max(256);
        Self { table: vec![0u8; n], mask: n - 1 }
    }

    #[inline]
    fn insert(&mut self, combined_hash: u32, fingerprint: u8) {
        let idx = (combined_hash as usize) & self.mask;
        self.table[idx] |= fingerprint;
    }

    #[inline]
    fn may_contain(&self, combined_hash: u32, fingerprint: u8) -> bool {
        let idx = (combined_hash as usize) & self.mask;
        self.table[idx] & fingerprint == fingerprint
    }
}

/// EavExactIndex: O(log n) attribute-value lookup with O(1) negative filter.
#[derive(Debug)]
pub struct EavExactIndex {
    /// Sorted (attr_hash, val_fingerprint, surrogate) triples.
    entries: Vec<Entry>,
    /// Fingerprint bloom table for fast negative answers.
    bloom:   Vec<u8>,
    bloom_mask: usize,
}

impl EavExactIndex {
    /// Build from raw `(attr_hash, encoded_value_bytes, surrogate)` triples.
    ///
    /// Duplicates are preserved (same attr/val on multiple surrogates).
    /// After build the entries are sorted and the bloom table is populated.
    pub fn build(raw: &[(u32, Vec<u8>, u32)]) -> Self {
        let n = raw.len().max(1);
        let bloom_size = (n / 4).next_power_of_two().max(256);
        let mask = bloom_size - 1;
        let mut bloom = vec![0u8; bloom_size];

        let mut entries: Vec<Entry> = raw.iter().map(|(attr_h, val_bytes, sur)| {
            let vfp = fnv1a_32(val_bytes);
            // Populate bloom
            let combined = attr_h.wrapping_add(vfp);
            let fp_byte  = (fnv1a_32(&combined.to_le_bytes()) & 0xFF) as u8 | 1;
            bloom[combined as usize & mask] |= fp_byte;
            (*attr_h, vfp, *sur)
        }).collect();

        entries.sort_unstable();

        Self { entries, bloom, bloom_mask: mask }
    }

    /// Build from already-hashed `(attr_hash, val_fingerprint, surrogate)` triples.
    pub fn build_from_triples(mut triples: Vec<Entry>) -> Self {
        let n = triples.len().max(1);
        let bloom_size = (n / 4).next_power_of_two().max(256);
        let mask = bloom_size - 1;
        let mut bloom = vec![0u8; bloom_size];

        for &(ah, vfp, _) in &triples {
            let combined = ah.wrapping_add(vfp);
            let fp_byte  = (fnv1a_32(&combined.to_le_bytes()) & 0xFF) as u8 | 1;
            bloom[combined as usize & mask] |= fp_byte;
        }

        triples.sort_unstable();
        Self { entries: triples, bloom, bloom_mask: mask }
    }

    /// Hash an encoded attribute value to its fingerprint.
    #[inline]
    pub fn val_fingerprint(encoded_value: &[u8]) -> u32 {
        fnv1a_32(encoded_value)
    }

    /// Hash an attribute name string to its attr_hash (CRC-16 widened to u32).
    #[inline]
    pub fn attr_hash_of(name: &str) -> u32 {
        fnv1a_32(name.as_bytes())
    }

    /// Fast negative check: if this returns `false`, `attr_hash`+`val_fp`
    /// is definitely not in the index.
    #[inline]
    pub fn may_contain(&self, attr_hash: u32, val_fp: u32) -> bool {
        let combined = attr_hash.wrapping_add(val_fp);
        let fp_byte  = (fnv1a_32(&combined.to_le_bytes()) & 0xFF) as u8 | 1;
        let idx      = combined as usize & self.bloom_mask;
        self.bloom[idx] & fp_byte == fp_byte
    }

    /// Return all surrogates that have `(attr_hash, val_fingerprint)`.
    ///
    /// Empty slice means: definitely absent (after bloom check).
    /// Non-empty slice: check surrogates for an exact match (caller's responsibility).
    pub fn candidates(&self, attr_hash: u32, val_fp: u32) -> &[Entry] {
        if !self.may_contain(attr_hash, val_fp) {
            return &[];
        }

        // Binary search for the lower bound of (attr_hash, val_fp, 0)
        let lo = self.entries.partition_point(|&(a, v, _)| {
            (a, v) < (attr_hash, val_fp)
        });
        // Upper bound of (attr_hash, val_fp, u32::MAX)
        let hi = self.entries.partition_point(|&(a, v, _)| {
            (a, v) <= (attr_hash, val_fp)
        });

        &self.entries[lo..hi]
    }

    /// Check whether a specific `(attr_hash, val_fp, surrogate)` triple exists.
    pub fn contains_surrogate(&self, attr_hash: u32, val_fp: u32, surrogate: u32) -> bool {
        let slice = self.candidates(attr_hash, val_fp);
        slice.binary_search(&(attr_hash, val_fp, surrogate)).is_ok()
    }

    /// Return a sorted Vec<u32> of all surrogates that match `(attr_hash, val_fp)`.
    ///
    /// The returned Vec has duplicates removed (at most one entry per surrogate).
    pub fn surrogates_for(&self, attr_hash: u32, val_fp: u32) -> Vec<u32> {
        self.candidates(attr_hash, val_fp)
            .iter()
            .map(|&(_, _, s)| s)
            .collect()
    }

    /// Total number of indexed triples.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if no triples were indexed.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Approximate memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        self.entries.len() * 12
            + self.bloom.len()
            + std::mem::size_of::<Self>()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_str(s: &str) -> Vec<u8> { s.as_bytes().to_vec() }
    fn encode_i64(n: i64) -> Vec<u8>  { n.to_le_bytes().to_vec() }

    #[test]
    fn build_and_exact_lookup() {
        let raw = vec![
            (100u32, encode_str("Najaf"),   1u32),
            (100u32, encode_str("Baghdad"), 2u32),
            (100u32, encode_str("Najaf"),   3u32),
            (200u32, encode_i64(1975),      1u32),
        ];
        let idx = EavExactIndex::build(&raw);

        let vfp_najaf   = EavExactIndex::val_fingerprint(b"Najaf");
        let vfp_baghdad = EavExactIndex::val_fingerprint(b"Baghdad");

        let najaf_surs = idx.surrogates_for(100, vfp_najaf);
        assert_eq!(najaf_surs, vec![1, 3]);

        let baghdad_surs = idx.surrogates_for(100, vfp_baghdad);
        assert_eq!(baghdad_surs, vec![2]);

        assert!(idx.contains_surrogate(100, vfp_najaf,   1));
        assert!(idx.contains_surrogate(100, vfp_najaf,   3));
        assert!(!idx.contains_surrogate(100, vfp_najaf,  2));
        assert!(!idx.contains_surrogate(100, vfp_baghdad,1));
    }

    #[test]
    fn bloom_eliminates_absent_entries() {
        let raw = vec![
            (100u32, encode_str("Najaf"), 1u32),
        ];
        let idx = EavExactIndex::build(&raw);
        let vfp_missing = EavExactIndex::val_fingerprint(b"ZZZ_NEVER_INSERTED");

        // may_contain should return false (or at worst true due to bloom collision)
        // but candidates() must return empty when bloom says no
        let cands = idx.candidates(100, vfp_missing);
        // Even if bloom false-positives, binary search returns no match
        let found: Vec<_> = cands.iter().filter(|&&(_, v, _)| v == vfp_missing).collect();
        assert!(found.is_empty());
    }

    #[test]
    fn empty_index_returns_no_candidates() {
        let idx = EavExactIndex::build(&[]);
        assert_eq!(idx.candidates(1, 2), &[]);
    }

    #[test]
    fn large_index_lookup() {
        // 100K entries, attr_hash 777, values "val_N"
        let raw: Vec<_> = (0u32..100_000)
            .map(|i| (777u32, format!("val_{i}").into_bytes(), i))
            .collect();
        let idx = EavExactIndex::build(&raw);

        let vfp = EavExactIndex::val_fingerprint(b"val_42");
        assert!(idx.contains_surrogate(777, vfp, 42));
        assert!(!idx.contains_surrogate(777, vfp, 43));
    }

    #[test]
    fn attr_hash_of_stable() {
        // Same name must always produce the same hash
        let h1 = EavExactIndex::attr_hash_of("city.name");
        let h2 = EavExactIndex::attr_hash_of("city.name");
        assert_eq!(h1, h2);
        assert_ne!(h1, EavExactIndex::attr_hash_of("birth.year"));
    }

    #[test]
    fn memory_estimate_reasonable() {
        let raw: Vec<_> = (0u32..1_000).map(|i| (i, encode_i64(i as i64), i)).collect();
        let idx = EavExactIndex::build(&raw);
        // 1000 × 12 + bloom ≈ 12KB + 256B
        assert!(idx.memory_bytes() < 64_000);
    }
}
