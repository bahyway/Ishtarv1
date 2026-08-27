//! Voxel-grid downsampling — the standard point-cloud reduction
//! technique (bucket points into a 3D grid, replace each occupied cell
//! with its centroid) needed because a raw drone-photogrammetry cloud is
//! commonly hundreds of thousands to millions of points, while
//! `bahyway_algebra::persistence::vietoris_rips_persistence`'s tetrahedra
//! enumeration (H2/voids) is O(n^4) — undownsampled input isn't merely
//! slow to analyze, it's intractable.

use bahyway_algebra::persistence::Point3;

/// Reduce `points` to at most one point (the centroid) per `voxel_size`
/// grid cell. `voxel_size` must be positive; a non-positive value
/// returns the input unchanged rather than dividing by zero or panicking.
pub fn voxel_downsample(points: &[Point3], voxel_size: f64) -> Vec<Point3> {
    if voxel_size <= 0.0 {
        return points.to_vec();
    }

    let mut cells: std::collections::HashMap<(i64, i64, i64), (Point3, usize)> =
        std::collections::HashMap::new();
    for p in points {
        let key = (
            (p[0] / voxel_size).floor() as i64,
            (p[1] / voxel_size).floor() as i64,
            (p[2] / voxel_size).floor() as i64,
        );
        let entry = cells.entry(key).or_insert(([0.0, 0.0, 0.0], 0));
        entry.0[0] += p[0];
        entry.0[1] += p[1];
        entry.0[2] += p[2];
        entry.1 += 1;
    }

    cells
        .into_values()
        .map(|(sum, n)| [sum[0] / n as f64, sum[1] / n as f64, sum[2] / n as f64])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_points_within_the_same_voxel_into_one_centroid() {
        let points = vec![[0.0, 0.0, 0.0], [0.1, 0.1, 0.1], [0.2, 0.0, 0.0]];
        let out = voxel_downsample(&points, 1.0);
        assert_eq!(out.len(), 1, "all three points fall in the same unit voxel");
        let expected = [
            (0.0 + 0.1 + 0.2) / 3.0,
            (0.0 + 0.1 + 0.0) / 3.0,
            (0.0 + 0.1 + 0.0) / 3.0,
        ];
        for i in 0..3 {
            assert!((out[0][i] - expected[i]).abs() < 1e-9);
        }
    }

    #[test]
    fn keeps_points_in_different_voxels_separate() {
        let points = vec![[0.0, 0.0, 0.0], [10.0, 10.0, 10.0]];
        let out = voxel_downsample(&points, 1.0);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn nonpositive_voxel_size_returns_input_unchanged() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let out = voxel_downsample(&points, 0.0);
        assert_eq!(out, points);
    }

    #[test]
    fn empty_input_is_empty_output() {
        assert!(voxel_downsample(&[], 1.0).is_empty());
    }

    #[test]
    fn a_dense_grid_downsamples_to_far_fewer_points() {
        // 1000 points densely packed in a 1x1x1 cube -- a real stand-in
        // for "way too many points for O(n^4) tetrahedra enumeration."
        let mut points = Vec::new();
        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    points.push([i as f64 * 0.1, j as f64 * 0.1, k as f64 * 0.1]);
                }
            }
        }
        assert_eq!(points.len(), 1000);
        let out = voxel_downsample(&points, 1.0);
        assert!(
            out.len() <= 8,
            "a 1x1x1 cube should collapse to a small handful of unit voxels, got {}",
            out.len()
        );
    }
}
