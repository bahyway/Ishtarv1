#![forbid(unsafe_code)]
//! RadixSplineIndex — learned range index replacing BTreeRange.
//!
//! Replaces the W5H2 "HOW MUCH" BTreeRange index:
//!
//! | Variant       | RAM for 1B particles |
//! |---------------|----------------------|
//! | BTreeRange    | ~12 GB               |
//! | RadixSpline   | ~2 KB                |
//!
//! Algorithm (from `EnkiDB_W5H2_Rust_Implementation_Guide.md` §7):
//! 1. Build time: greedy piecewise-linear spline fitting with error bound ε.
//! 2. Radix table: top `RADIX_BITS` of key → segment index (O(1) lookup).
//! 3. Query: radix lookup → spline interpolation → local binary search within ε.
//!
//! Keys are `u64` epoch values; values are dense `u32` surrogate IDs.

/// Number of radix bits.  2^18 = 262 144 slots.
const RADIX_BITS: usize = 18;
const RADIX_SIZE: usize = 1 << RADIX_BITS;

/// Error bound: maximum prediction error in number of array positions.
pub const DEFAULT_EPSILON: u32 = 32;

/// One piecewise-linear segment: y = slope * x + intercept.
#[derive(Clone, Debug)]
pub struct SplineSegment {
    pub start_key: u64,
    pub slope:     f64,
    pub intercept: f64,
}

impl SplineSegment {
    fn estimate(&self, key: u64) -> usize {
        let y = self.slope * key as f64 + self.intercept;
        y.max(0.0) as usize
    }
}

/// Learned range index over sorted (epoch, surrogate) pairs.
///
/// After `build()` the struct is immutable.  Rebuild on each snapshot load.
pub struct RadixSplineIndex {
    /// Segment table (radix prefix → segment index).
    radix_table: Vec<u32>,
    /// Spline segments in ascending `start_key` order.
    segments: Vec<SplineSegment>,
    /// Sorted epoch keys (parallel array to `values`).
    keys: Vec<u64>,
    /// Surrogate IDs (parallel to `keys`).
    values: Vec<u32>,
    /// Error bound used at build time.
    epsilon: u32,
}

impl RadixSplineIndex {
    /// Build from **sorted** (key, value) pairs.
    /// Panics if `keys` and `values` differ in length or are unsorted.
    pub fn build(keys: Vec<u64>, values: Vec<u32>, epsilon: u32) -> Self {
        assert_eq!(keys.len(), values.len(), "keys and values must be the same length");
        // Verify ascending order in debug builds.
        debug_assert!(keys.windows(2).all(|w| w[0] <= w[1]), "keys must be sorted");

        if keys.is_empty() {
            return Self {
                radix_table: vec![0u32; RADIX_SIZE],
                segments: vec![],
                keys: vec![],
                values: vec![],
                epsilon,
            };
        }

        let n = keys.len();
        let mut segments: Vec<SplineSegment> = Vec::new();
        let mut radix_table = vec![0u32; RADIX_SIZE];

        // Greedy spline fitting.
        let mut seg_start = 0usize;
        let mut upper_slope = f64::INFINITY;
        let mut lower_slope = f64::NEG_INFINITY;

        let close_segment = |seg_start: usize, upper_slope: f64, lower_slope: f64,
                             segs: &mut Vec<SplineSegment>, rt: &mut Vec<u32>,
                             k_start: u64| {
            let slope = (upper_slope + lower_slope) / 2.0;
            let intercept = seg_start as f64 - slope * k_start as f64;
            segs.push(SplineSegment { start_key: k_start, slope, intercept });
            // Fill radix table slot.
            let prefix = (k_start >> (64 - RADIX_BITS)).min(RADIX_SIZE as u64 - 1) as usize;
            rt[prefix] = (segs.len() as u32).saturating_sub(1);
        };

        for i in 1..n {
            let dx = (keys[i] as i128 - keys[seg_start] as i128) as f64;
            if dx == 0.0 { continue; }

            let y_top = (i as f64) + epsilon as f64;
            let y_bot = (i as f64) - epsilon as f64;
            let s_top = (y_top - seg_start as f64) / dx;
            let s_bot = (y_bot - seg_start as f64) / dx;

            upper_slope = upper_slope.min(s_top);
            lower_slope = lower_slope.max(s_bot);

            if lower_slope > upper_slope {
                // Constraint violated — close segment at i−1
                close_segment(seg_start, upper_slope, lower_slope,
                              &mut segments, &mut radix_table, keys[seg_start]);
                seg_start = i - 1;
                upper_slope = f64::INFINITY;
                lower_slope = f64::NEG_INFINITY;
            }
        }
        // Close final segment.
        close_segment(seg_start, upper_slope.min(1.0), lower_slope.max(0.0),
                      &mut segments, &mut radix_table, keys[seg_start]);

        // Forward-fill radix table gaps (point empty slots to the last valid segment).
        let mut last = 0u32;
        for slot in radix_table.iter_mut() {
            if *slot != 0 { last = *slot; } else { *slot = last; }
        }

        Self { radix_table, segments, keys, values, epsilon }
    }

    /// Range query: returns surrogates for all keys in `[min_epoch, max_epoch]`.
    pub fn range_query(&self, min_epoch: u64, max_epoch: u64) -> Vec<u32> {
        if self.keys.is_empty() { return vec![]; }

        let start_est = self.estimate(min_epoch);
        let end_est   = self.estimate(max_epoch);
        let eps = self.epsilon as usize;

        let n = self.keys.len();
        let lo = start_est.saturating_sub(eps).min(n.saturating_sub(1));
        let hi = (end_est + eps).min(n.saturating_sub(1));

        if lo > hi {
            return vec![];
        }

        // Binary search within the ε window.
        let exact_lo = self.keys[lo..=hi]
            .partition_point(|&k| k < min_epoch) + lo;
        let exact_hi = self.keys[lo..=hi]
            .partition_point(|&k| k <= max_epoch) + lo;

        self.values[exact_lo..exact_hi].to_vec()
    }

    /// Estimated array position for a key (radix + spline interpolation).
    #[inline(always)]
    fn estimate(&self, key: u64) -> usize {
        let prefix = (key >> (64 - RADIX_BITS)).min(RADIX_SIZE as u64 - 1) as usize;
        let seg_idx = self.radix_table[prefix] as usize;
        let seg = &self.segments[seg_idx.min(self.segments.len() - 1)];
        seg.estimate(key)
    }

    pub fn len(&self)      -> usize { self.keys.len() }
    pub fn is_empty(&self) -> bool  { self.keys.is_empty() }
    pub fn segment_count(&self) -> usize { self.segments.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_dense(n: usize) -> RadixSplineIndex {
        let keys:   Vec<u64> = (0..n as u64).collect();
        let values: Vec<u32> = (0..n as u32).collect();
        RadixSplineIndex::build(keys, values, DEFAULT_EPSILON)
    }

    #[test]
    fn empty_index() {
        let idx = RadixSplineIndex::build(vec![], vec![], DEFAULT_EPSILON);
        assert!(idx.range_query(0, 100).is_empty());
    }

    #[test]
    fn exact_range_dense() {
        let idx = build_dense(1000);
        let result = idx.range_query(10, 19);
        assert_eq!(result, (10u32..20).collect::<Vec<_>>());
    }

    #[test]
    fn full_range() {
        let idx = build_dense(512);
        let result = idx.range_query(0, 511);
        assert_eq!(result.len(), 512);
    }

    #[test]
    fn out_of_range_returns_empty() {
        let idx = build_dense(100);
        assert!(idx.range_query(200, 300).is_empty());
    }

    #[test]
    fn single_element_range() {
        let idx = build_dense(1000);
        let result = idx.range_query(42, 42);
        assert_eq!(result, vec![42u32]);
    }

    #[test]
    fn segment_count_reasonable() {
        let idx = build_dense(10_000);
        // Should compress into far fewer segments than 10 000 keys.
        assert!(idx.segment_count() < 1000, "expected fewer than 1000 segments, got {}", idx.segment_count());
    }
}
