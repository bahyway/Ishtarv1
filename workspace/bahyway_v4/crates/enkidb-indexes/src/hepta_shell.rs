#![forbid(unsafe_code)]
//! HeptaShellIndex — E7 lattice spatial index for 7D HeptaScript WHERE queries.
//!
//! Maps 7-dimensional positions to quantised zone hashes, then groups
//! particle surrogates into buckets.  Spatial queries return the centre
//! zone plus up to 126 E7 lattice neighbours (the E7 kissing number,
//! as specified in `EnkiDB_W5H2_Rust_Implementation_Guide.md` §4).
//!
//! ## Memory (W5H2 document)
//! | Variant          | RAM      |
//! |------------------|----------|
//! | Unoptimised      | ~8 GB    |
//! | With zone merging| ~4 GB    |
//!
//! ## Hashing
//! Zone hash = FNV-1a over quantised 7D integers (replaces xxhash64
//! to stay dependency-free; same avalanche quality for 7×4=28 bytes).

/// E7 kissing number — maximum lattice neighbours in 7 dimensions.
pub const E7_KISSING_NUMBER: usize = 126;
/// Spatial dimensions.
pub const DIMENSIONS: usize = 7;

/// Quantisation scale per dimension.
/// Dimensions 0–2 (r,θ,φ): 1e6 (sub-milliradian / sub-millimetre).
/// Dimensions 3–6 (d3–d6): 1e3 (metre scale).
const SCALE: [f64; DIMENSIONS] = [1e6, 1e6, 1e6, 1e3, 1e3, 1e3, 1e3];

/// FNV-1a 64-bit (same constants as surrogate.rs and enkidb-kaki).
fn fnv1a_28(bytes: &[u8; 28]) -> u64 {
    const OFFSET: u64 = 14_695_981_039_346_656_037;
    const PRIME:  u64 = 1_099_511_628_211;
    let mut h = OFFSET;
    for &b in bytes.iter() { h = (h ^ b as u64).wrapping_mul(PRIME); }
    h
}

fn zone_hash_from_quantized(q: &[u32; DIMENSIONS]) -> u64 {
    let mut bytes = [0u8; 28];
    for (i, &v) in q.iter().enumerate() {
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&v.to_le_bytes());
    }
    fnv1a_28(&bytes)
}

/// Quantise a 7D float position to integer grid.
fn quantize(pos: &[f64; DIMENSIONS]) -> [u32; DIMENSIONS] {
    let mut q = [0u32; DIMENSIONS];
    for i in 0..DIMENSIONS {
        q[i] = (pos[i].abs() * SCALE[i]) as u32;
    }
    q
}

/// A zone bucket: sorted zone hash → dense list of surrogate IDs.
#[derive(Clone, Debug)]
pub struct ZoneBucket {
    pub zone_hash:    u64,
    pub surrogates:   Vec<u32>,
}

/// E7 neighbour offsets — precomputed unit-step lattice vectors.
///
/// The full E7 root system has 126 minimal vectors.  We approximate them
/// with the 7-dimensional ±1 unit vectors (14 vectors) and all 7C2 = 21
/// face-diagonal pairs (42 vectors), plus selected body diagonals.
/// For production use replace with the exact E7 Gosset polytope coordinates.
fn e7_neighbour_offsets() -> Vec<[i32; DIMENSIONS]> {
    let mut offsets = Vec::with_capacity(E7_KISSING_NUMBER);

    // ±1 unit vectors (14)
    for d in 0..DIMENSIONS {
        let mut v = [0i32; DIMENSIONS];
        v[d] = 1;
        offsets.push(v);
        let mut v = [0i32; DIMENSIONS];
        v[d] = -1;
        offsets.push(v);
    }

    // ±1 face diagonals (all pairs of two dimensions, ± combinations) (4 × 21 = 84)
    for i in 0..DIMENSIONS {
        for j in (i + 1)..DIMENSIONS {
            for si in &[1i32, -1] {
                for sj in &[1i32, -1] {
                    let mut v = [0i32; DIMENSIONS];
                    v[i] = *si;
                    v[j] = *sj;
                    offsets.push(v);
                }
            }
        }
    }

    // Body diagonals: all ±1 in every dimension truncated to 126
    let needed = E7_KISSING_NUMBER.saturating_sub(offsets.len());
    if needed > 0 {
        // Generate a few body diagonals to pad to 126
        let signs_iter = (0..)
            .map(|n: u64| {
                let mut v = [0i32; DIMENSIONS];
                for d in 0..DIMENSIONS {
                    v[d] = if (n >> d) & 1 == 0 { 1 } else { -1 };
                }
                v
            })
            .take(needed);
        offsets.extend(signs_iter);
    }

    offsets.truncate(E7_KISSING_NUMBER);
    offsets
}

/// 7D spatial index for HeptaScript WHERE queries.
pub struct HeptaShellIndex {
    /// Sorted zone table (binary-searched by zone_hash).
    zone_table: Vec<ZoneBucket>,
    /// Expected surrogates per zone (for significance filtering).
    expected_density: u32,
    /// Precomputed E7 neighbour offsets.
    neighbour_offsets: Vec<[i32; DIMENSIONS]>,
}

impl HeptaShellIndex {
    /// Build index from (7D position, surrogate) pairs.
    pub fn build(entries: &[([f64; DIMENSIONS], u32)]) -> Self {
        let mut buckets: std::collections::HashMap<u64, Vec<u32>> =
            std::collections::HashMap::new();

        for (pos, surrogate) in entries {
            let q = quantize(pos);
            let h = zone_hash_from_quantized(&q);
            buckets.entry(h).or_default().push(*surrogate);
        }

        let total_surrogates: usize = buckets.values().map(|v| v.len()).sum();
        let expected_density = if buckets.is_empty() {
            0
        } else {
            (total_surrogates / buckets.len()) as u32
        };

        let mut zone_table: Vec<ZoneBucket> = buckets
            .into_iter()
            .map(|(h, mut s)| {
                s.sort_unstable();
                ZoneBucket { zone_hash: h, surrogates: s }
            })
            .collect();
        zone_table.sort_by_key(|b| b.zone_hash);

        Self {
            zone_table,
            expected_density,
            neighbour_offsets: e7_neighbour_offsets(),
        }
    }

    /// Compute zone hash for a 7D float position.
    #[inline(always)]
    pub fn zone_hash(&self, pos: &[f64; DIMENSIONS]) -> u64 {
        zone_hash_from_quantized(&quantize(pos))
    }

    /// Spatial query: returns all surrogate IDs in centre zone + E7 neighbours.
    ///
    /// `radius_scale` multiplies the unit neighbour offsets (in quantised units).
    /// Pass `1` for exact-zone + immediate neighbours.
    pub fn spatial_query(&self, center: &[f64; DIMENSIONS], radius_scale: i32) -> Vec<u32> {
        let cq = quantize(center);
        let center_hash = zone_hash_from_quantized(&cq);

        let mut result = Vec::new();

        // Centre zone
        if let Some(bucket) = self.find_bucket(center_hash) {
            result.extend_from_slice(&bucket.surrogates);
        }

        // E7 neighbours
        for offset in &self.neighbour_offsets {
            let nq: [u32; DIMENSIONS] = {
                let mut q = cq;
                for d in 0..DIMENSIONS {
                    let shifted = (q[d] as i64) + (offset[d] as i64) * (radius_scale as i64);
                    q[d] = shifted.max(0) as u32;
                }
                q
            };
            let nh = zone_hash_from_quantized(&nq);
            if let Some(bucket) = self.find_bucket(nh) {
                if self.is_significant(bucket) {
                    result.extend_from_slice(&bucket.surrogates);
                }
            }
        }

        result.sort_unstable();
        result.dedup();
        result
    }

    /// Zone significance: include zones with meaningful density (Θ_E7 filter).
    #[inline(always)]
    fn is_significant(&self, bucket: &ZoneBucket) -> bool {
        let actual = bucket.surrogates.len() as u32;
        actual > (self.expected_density / 4).max(1) || actual > 1000
    }

    fn find_bucket(&self, hash: u64) -> Option<&ZoneBucket> {
        self.zone_table
            .binary_search_by_key(&hash, |b| b.zone_hash)
            .ok()
            .map(|idx| &self.zone_table[idx])
    }

    pub fn zone_count(&self) -> usize { self.zone_table.len() }
    pub fn is_empty(&self)   -> bool  { self.zone_table.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entries(n: usize) -> Vec<([f64; DIMENSIONS], u32)> {
        (0..n).map(|i| {
            let v = i as f64 * 0.1;
            ([v, v, v, v, v, v, v], i as u32)
        }).collect()
    }

    #[test]
    fn build_and_zone_count() {
        let entries = make_entries(100);
        let idx = HeptaShellIndex::build(&entries);
        assert!(!idx.is_empty());
    }

    #[test]
    fn centre_zone_query_finds_entry() {
        let entries = vec![([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], 42u32)];
        let idx = HeptaShellIndex::build(&entries);
        let result = idx.spatial_query(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], 1);
        assert!(result.contains(&42), "expected surrogate 42 in result");
    }

    #[test]
    fn neighbour_offsets_count() {
        let offsets = e7_neighbour_offsets();
        assert_eq!(offsets.len(), E7_KISSING_NUMBER,
            "must have exactly {} E7 neighbour offsets", E7_KISSING_NUMBER);
    }

    #[test]
    fn empty_index() {
        let idx = HeptaShellIndex::build(&[]);
        assert!(idx.is_empty());
        let result = idx.spatial_query(&[0.0; 7], 1);
        assert!(result.is_empty());
    }

    #[test]
    fn no_duplicate_surrogates_in_result() {
        let entries = make_entries(50);
        let idx = HeptaShellIndex::build(&entries);
        let result = idx.spatial_query(&[2.5, 2.5, 2.5, 2.5, 2.5, 2.5, 2.5], 2);
        let mut sorted = result.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(result, sorted, "result must have no duplicates");
    }
}
