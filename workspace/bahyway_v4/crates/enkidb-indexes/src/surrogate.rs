#![forbid(unsafe_code)]
//! SurrogateMap — KAKI → dense u32 surrogate ID mapping.
//!
//! In the W5H2 query pipeline every particle is referenced internally via
//! a compact surrogate u32 rather than its full 16-byte KAKI, saving
//! significant RAM compared to a `HashMap<[u8;16], u32>`.
//!
//! ## Implementation
//! This module implements a **sorted-array binary-search map**:
//! - Build: O(N log N) — sort once at snapshot load.
//! - Lookup: O(log N) — binary search; cache-friendly for 1B KAKIs in sorted order.
//! - Memory: 20 bytes/entry (16B KAKI + 4B surrogate) = ~20 GB for 1B particles
//!   vs ~24 GB for HashMap.  Production deployments replace with PTHash MPHF for
//!   750 MB, pending the `pthash` crate stabilisation.
//!
//! ## Memory model (W5H2 document §CRITICAL OPTIMIZATION)
//! | Variant              | RAM for 1B particles |
//! |----------------------|----------------------|
//! | HashMap<[u8;16],u32> | ~24 GB               |
//! | SurrogateMap (sorted)| ~20 GB               |
//! | PTHash MPHF (target) | ~750 MB              |

use enkidb_kaki::Kaki;

/// Sorted-array surrogate map.
///
/// The map is **immutable after build** — snapshot-level, rebuilt on each
/// Read-node snapshot load.  No insert/delete API is intentional.
pub struct SurrogateMap {
    /// Sorted entries: (kaki_bytes, surrogate).
    entries: Vec<([u8; 16], u32)>,
}

impl SurrogateMap {
    /// Build the map from a slice of KAKI byte arrays.
    /// `kakis[i]` is assigned surrogate `i as u32`.
    pub fn build(kakis: &[[u8; 16]]) -> Self {
        let mut entries: Vec<([u8; 16], u32)> = kakis
            .iter()
            .enumerate()
            .map(|(i, k)| (*k, i as u32))
            .collect();
        // Sort by KAKI bytes for binary search.
        entries.sort_unstable_by_key(|(k, _)| *k);
        Self { entries }
    }

    /// Build from `Kaki` values.
    pub fn build_from_kakis(kakis: &[Kaki]) -> Self {
        let raw: Vec<[u8; 16]> = kakis.iter().map(|k| *k.bytes()).collect();
        Self::build(&raw)
    }

    /// Resolve a KAKI byte array to its surrogate ID.
    /// Returns `None` for KAKIs not registered during `build`.
    #[inline(always)]
    pub fn resolve(&self, kaki: &[u8; 16]) -> Option<u32> {
        self.entries
            .binary_search_by_key(kaki, |(k, _)| *k)
            .ok()
            .map(|idx| self.entries[idx].1)
    }

    /// Resolve a `Kaki`.
    #[inline(always)]
    pub fn resolve_kaki(&self, kaki: &Kaki) -> Option<u32> {
        self.resolve(kaki.bytes())
    }

    /// Batch resolution: returns `u32::MAX` for unknown KAKIs.
    pub fn resolve_batch(&self, kakis: &[[u8; 16]]) -> Vec<u32> {
        kakis
            .iter()
            .map(|k| self.resolve(k).unwrap_or(u32::MAX))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kaki(n: u32) -> [u8; 16] {
        let mut k = [0u8; 16];
        k[..4].copy_from_slice(&n.to_le_bytes());
        k[14] = 0x04; // pattern KAKI type prefix
        k
    }

    #[test]
    fn build_and_resolve() {
        let kakis: Vec<[u8; 16]> = (0..128).map(make_kaki).collect();
        let map = SurrogateMap::build(&kakis);
        assert_eq!(map.len(), 128);

        // Every registered KAKI must resolve to its original index.
        for (i, kaki) in kakis.iter().enumerate() {
            let s = map.resolve(kaki).expect("registered KAKI must resolve");
            assert_eq!(s as usize, i, "surrogate mismatch for kaki {i}");
        }
    }

    #[test]
    fn unknown_kaki_returns_none() {
        let kakis: Vec<[u8; 16]> = (0..16).map(make_kaki).collect();
        let map = SurrogateMap::build(&kakis);
        let unknown = make_kaki(9999);
        assert!(map.resolve(&unknown).is_none());
    }

    #[test]
    fn empty_map() {
        let map = SurrogateMap::build(&[]);
        assert!(map.is_empty());
        assert!(map.resolve(&[0u8; 16]).is_none());
    }

    #[test]
    fn batch_resolves_match_single() {
        let kakis: Vec<[u8; 16]> = (0..32).map(make_kaki).collect();
        let map = SurrogateMap::build(&kakis);
        let singles: Vec<u32> = kakis.iter().map(|k| map.resolve(k).unwrap()).collect();
        let batch = map.resolve_batch(&kakis);
        assert_eq!(singles, batch);
    }

    #[test]
    fn large_map() {
        let kakis: Vec<[u8; 16]> = (0u32..10_000).map(make_kaki).collect();
        let map = SurrogateMap::build(&kakis);
        assert_eq!(map.len(), 10_000);
        // Spot-check a few
        for i in [0u32, 1, 42, 999, 9999] {
            assert!(map.resolve(&make_kaki(i)).is_some());
        }
    }
}
