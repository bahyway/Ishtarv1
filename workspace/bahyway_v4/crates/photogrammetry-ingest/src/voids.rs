//! Wires a downsampled point cloud into GeoEngine
//! (`bahyway_algebra::persistence`) to detect real voids: a gap in an
//! otherwise-regular field of points (e.g. headstone-top returns from a
//! drone survey over a burial ground) reads as a genuine H2 class —
//! "possible unmarked/missing grave" — the same "void = missing data"
//! narrative this ecosystem's PDM-discovery work established for tabular
//! schema data (see `pdm-discovery`'s own module doc), applied here to
//! physical survey data instead. This module does not re-derive whether
//! the math is correct (that's `bahyway-algebra::persistence`'s own,
//! already-tested job) — only that the downsample-then-persist pipeline
//! is wired correctly.

use crate::{downsample::voxel_downsample, PointCloud};
use bahyway_algebra::persistence::{vietoris_rips_persistence, PersistenceDiagram};

#[derive(Debug, Clone)]
pub struct VoidReport {
    pub input_point_count: usize,
    pub downsampled_point_count: usize,
    pub diagram: PersistenceDiagram,
}

impl VoidReport {
    pub fn void_count(&self) -> usize {
        self.diagram.void_count()
    }
    pub fn component_count(&self) -> usize {
        self.diagram.component_count()
    }
}

/// Downsample `cloud` to `voxel_size` and run GeoEngine's H0/H1/H2
/// persistence over the result, up to `max_epsilon`. Callers MUST choose
/// `voxel_size` large enough that the downsampled point count stays
/// small — tetrahedra enumeration is O(n^4) (see
/// `bahyway_algebra::persistence`'s own module doc).
pub fn detect_voids(cloud: &PointCloud, voxel_size: f64, max_epsilon: f64) -> VoidReport {
    let downsampled = voxel_downsample(&cloud.points, voxel_size);
    let diagram = vietoris_rips_persistence(&downsampled, max_epsilon);
    VoidReport {
        input_point_count: cloud.points.len(),
        downsampled_point_count: downsampled.len(),
        diagram,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The same octahedron-shell construction `bahyway-algebra::
    /// persistence`'s own tests already proved encloses exactly one
    /// persistent H2 void — reused here (small voxel size so no merging
    /// occurs) to prove THIS crate's downsample-then-persist wiring is
    /// correct, not to re-derive the underlying math.
    fn octahedron_shell() -> Vec<[f64; 3]> {
        vec![
            [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0], [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0], [0.0, 0.0, -1.0],
        ]
    }

    #[test]
    fn a_hollow_shell_of_survey_points_reports_one_void() {
        let cloud = PointCloud { points: octahedron_shell(), colors: None };
        let report = detect_voids(&cloud, 0.01, 1.5);
        assert_eq!(report.input_point_count, 6);
        assert_eq!(report.downsampled_point_count, 6, "voxel size smaller than point spacing must not merge any points");
        assert_eq!(report.component_count(), 1);
        assert_eq!(report.void_count(), 1, "a hollow shell of survey points must report a real void");
    }

    #[test]
    fn supplying_the_missing_center_point_collapses_the_reported_void() {
        let mut points = octahedron_shell();
        points.push([0.0, 0.0, 0.0]);
        let cloud = PointCloud { points, colors: None };
        let report = detect_voids(&cloud, 0.01, 1.5);
        assert_eq!(report.void_count(), 0, "a previously-missing survey point, once present, must close the void");
    }

    #[test]
    fn empty_cloud_reports_no_voids() {
        let cloud = PointCloud::default();
        let report = detect_voids(&cloud, 1.0, 1.0);
        assert_eq!(report.void_count(), 0);
        assert_eq!(report.input_point_count, 0);
    }

    #[test]
    fn aggressive_downsampling_actually_reduces_point_count() {
        // A dense cluster of near-duplicate survey points (realistic for
        // raw drone photogrammetry output) must collapse to far fewer
        // points before persistence ever runs on it.
        let mut points = Vec::new();
        for i in 0..20 {
            points.push([i as f64 * 0.01, 0.0, 0.0]);
        }
        let cloud = PointCloud { points, colors: None };
        let report = detect_voids(&cloud, 1.0, 1.0);
        assert_eq!(report.input_point_count, 20);
        assert!(report.downsampled_point_count < 20);
    }
}
