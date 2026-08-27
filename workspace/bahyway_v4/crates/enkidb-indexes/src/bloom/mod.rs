//! Sovereign Bloom Filter — probabilistic particle presence detection.
//!
//! This is the **ADAD Identity Gatekeeper**: O(1) constant-time check for whether
//! a 16-byte KAKI has been seen before. Used by the Active Orbit Field to avoid
//! expensive JNF matrix operations for unknown particles.
//!
//! # Why sovereign?
//! The concept from the Bloom/LRU documents uses the `bloomfilter` crate (FORBIDDEN).
//! This implementation uses only bitwise operations and FNV-like hash functions —
//! zero external dependencies, `#![forbid(unsafe_code)]` throughout.
//!
//! # False positive rate
//! For m bits and k hash functions, the expected FP rate is approximately
//! (1 - e^(-kn/m))^k where n is the number of inserted elements.
//! With k=4 and m=65536 bits (8 KiB), FP rate ≈ 0.39% for 4000 KAKIs,
//! ≈ 3.5% for 10000 KAKIs. Resize by doubling bit count when needed.
//!
//! # Security note
//! This is not a cryptographic data structure. It reveals membership probability
//! only. The actual KAKI is never stored here.

#![forbid(unsafe_code)]

/// A sovereign Bloom Filter backed by a heap-allocated bit array.
///
/// Parameterised over `N_BITS` (must be a power of 2 and a multiple of 64).
/// Default: 65536 bits (8 KiB) with k=4 hash functions.
pub struct BloomFilter {
    /// The underlying bit array stored as u64 words.
    bits: Vec<u64>,
    /// Number of bits (always `bits.len() * 64`).
    n_bits: usize,
    /// Number of independent hash functions.
    k: usize,
    /// Count of inserted items (approximate — no deduplication tracking).
    count: usize,
}

impl BloomFilter {
    /// Create a new filter with `n_bits` capacity (must be a multiple of 64).
    /// `k` is the number of hash functions (2–8 recommended).
    pub fn new(n_bits: usize, k: usize) -> Self {
        assert!(
            n_bits > 0 && n_bits.is_multiple_of(64),
            "n_bits must be a positive multiple of 64"
        );
        assert!((1..=16).contains(&k), "k must be in [1, 16]");
        BloomFilter {
            bits: vec![0u64; n_bits / 64],
            n_bits,
            k,
            count: 0,
        }
    }

    /// Create a default 8 KiB filter with k=4 (good for ~4000 KAKIs, <1% FP).
    pub fn default_8k() -> Self {
        Self::new(65_536, 4)
    }

    /// Create a filter tuned for ~1 million KAKIs at ~3% FP rate (1 MiB, k=4).
    pub fn default_1m() -> Self {
        Self::new(8_388_608, 4)
    }

    /// Insert a 16-byte KAKI into the filter.
    pub fn insert(&mut self, kaki: &[u8; 16]) {
        for i in 0..self.k {
            let bit = self.hash_bit(kaki, i as u64);
            self.set_bit(bit);
        }
        self.count += 1;
    }

    /// Check whether `kaki` *might* have been inserted.
    ///
    /// Returns `false` → definitely NOT in the set (no false negatives).
    /// Returns `true`  → probably in the set (false positive rate varies).
    pub fn possibly_contains(&self, kaki: &[u8; 16]) -> bool {
        for i in 0..self.k {
            let bit = self.hash_bit(kaki, i as u64);
            if !self.get_bit(bit) {
                return false;
            }
        }
        true
    }

    /// Reset the filter (all bits cleared, count reset to 0).
    pub fn clear(&mut self) {
        for word in &mut self.bits {
            *word = 0;
        }
        self.count = 0;
    }

    /// Approximate number of items inserted (no deduplication).
    pub fn approximate_count(&self) -> usize {
        self.count
    }

    /// Number of bits in the filter.
    pub fn capacity_bits(&self) -> usize {
        self.n_bits
    }

    /// Number of hash functions.
    pub fn k(&self) -> usize {
        self.k
    }

    // ── Private helpers ────────────────────────────────────────────────────────

    /// Sovereign FNV-1a inspired hash: mix the 16 KAKI bytes with a seed.
    /// Returns a bit index in [0, n_bits).
    fn hash_bit(&self, kaki: &[u8; 16], seed: u64) -> usize {
        // FNV-1a basis and prime
        const FNV_BASIS: u64 = 14_695_981_039_346_656_037;
        const FNV_PRIME: u64 = 1_099_511_628_211;

        let mut h = FNV_BASIS ^ seed.wrapping_mul(0x9e3779b97f4a7c15);
        for &byte in kaki.iter() {
            h ^= byte as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        // Mix the upper and lower halves for better distribution
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        (h as usize) % self.n_bits
    }

    fn set_bit(&mut self, bit: usize) {
        let word = bit / 64;
        let shift = bit % 64;
        self.bits[word] |= 1u64 << shift;
    }

    fn get_bit(&self, bit: usize) -> bool {
        let word = bit / 64;
        let shift = bit % 64;
        (self.bits[word] >> shift) & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kaki(seed: u8) -> [u8; 16] {
        let mut k = [0u8; 16];
        k[0] = seed;
        k[7] = seed.wrapping_mul(3);
        k[15] = seed.wrapping_mul(7);
        k
    }

    #[test]
    fn test_new_filter_empty_contains_nothing() {
        let f = BloomFilter::default_8k();
        for seed in 0..10 {
            assert!(!f.possibly_contains(&kaki(seed)));
        }
    }

    #[test]
    fn test_inserted_kaki_always_found() {
        let mut f = BloomFilter::default_8k();
        for seed in 0..50u8 {
            f.insert(&kaki(seed));
        }
        for seed in 0..50u8 {
            assert!(
                f.possibly_contains(&kaki(seed)),
                "seed {seed} must be found"
            );
        }
    }

    #[test]
    fn test_clear_resets_filter() {
        let mut f = BloomFilter::default_8k();
        f.insert(&kaki(42));
        assert!(f.possibly_contains(&kaki(42)));
        f.clear();
        assert!(!f.possibly_contains(&kaki(42)));
        assert_eq!(f.approximate_count(), 0);
    }

    #[test]
    fn test_count_increments_on_insert() {
        let mut f = BloomFilter::default_8k();
        for i in 0..10u8 {
            f.insert(&kaki(i));
            assert_eq!(f.approximate_count(), (i + 1) as usize);
        }
    }

    #[test]
    fn test_different_seeds_produce_different_hashes() {
        let f = BloomFilter::default_8k();
        let h0 = f.hash_bit(&kaki(0), 0);
        let h1 = f.hash_bit(&kaki(1), 0);
        assert_ne!(h0, h1);
    }

    #[test]
    fn test_k_seed_shifts_produce_different_bits() {
        let f = BloomFilter::default_8k();
        let b0 = f.hash_bit(&kaki(7), 0);
        let b1 = f.hash_bit(&kaki(7), 1);
        let b2 = f.hash_bit(&kaki(7), 2);
        let b3 = f.hash_bit(&kaki(7), 3);
        // Very likely all different (could theoretically collide but won't for these values)
        let unique: std::collections::BTreeSet<_> = [b0, b1, b2, b3].iter().copied().collect();
        assert_eq!(
            unique.len(),
            4,
            "hash seeds should produce distinct bit positions"
        );
    }

    #[test]
    fn test_false_positive_rate_acceptable_for_small_set() {
        // Insert 100 KAKIs, check 100 unknown ones — expect very few false positives
        let mut f = BloomFilter::default_8k();
        for i in 0..100u8 {
            f.insert(&kaki(i));
        }

        let mut fp_count = 0usize;
        for i in 100..200u8 {
            if f.possibly_contains(&kaki(i)) {
                fp_count += 1;
            }
        }
        // With 8k bits, k=4, 100 items: FP rate should be < 1%
        assert!(
            fp_count < 5,
            "false positive count {fp_count} out of 100 is too high"
        );
    }

    #[test]
    fn test_capacity_and_k_accessors() {
        let f = BloomFilter::new(128, 3);
        assert_eq!(f.capacity_bits(), 128);
        assert_eq!(f.k(), 3);
    }
}
