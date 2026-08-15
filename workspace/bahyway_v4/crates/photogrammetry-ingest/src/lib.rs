//! photogrammetry-ingest — real ingestion for drone-photogrammetry point
//! clouds, built for the concrete first use case named in the session
//! that created this crate: drone-scanned Wadi Al-Salaam grave-marker
//! surveys, feeding the same `bahyway-algebra::persistence` GeoEngine
//! this ecosystem already uses for TDA (H0/H1/H2 — see that crate's own
//! module doc).
//!
//! HONEST SCOPE, stated up front: structure-from-motion (turning raw
//! drone photographs into a 3D point cloud in the first place) is NOT
//! reimplemented here. That is a large, mature field (bundle adjustment,
//! feature matching, dense multi-view stereo) with real existing engines
//! (OpenDroneMap, COLMAP, Meshroom, commercial tools) — reimplementing it
//! would be exactly the kind of fabricated capability this ecosystem's
//! own discipline forbids. What this crate does instead:
//!
//!   1. `sfm` — a real, tested subprocess wrapper that invokes WHATEVER
//!      external SfM engine a deployment configures, and locates the
//!      point-cloud file it produces. No specific engine (ODM/COLMAP/
//!      Meshroom) ships with or was verified installed in the workspace
//!      that built this crate — confirmed absent (`which odm colmap
//!      meshroom` and a repo-wide search all came back empty; only the
//!      `docker` binary exists, no running engine). Matches this
//!      ecosystem's existing `girsu-vscodium-src` pattern: "the real
//!      build/binary lives outside this repo," verified by a stub-script
//!      test double, not a fabricated real run.
//!   2. `ply`/`xyz`/`las` — real parsers for the point-cloud file an SfM
//!      engine (or any other tool) actually produces.
//!   3. `downsample` — voxel-grid downsampling, because a raw drone
//!      point cloud is commonly hundreds of thousands to millions of
//!      points, and `vietoris_rips_persistence`'s tetrahedra enumeration
//!      is O(n^4) — undownsampled input is not analyzable at all, not
//!      merely slow.
//!   4. `voids` — wires a downsampled cloud into GeoEngine
//!      (`bahyway_algebra::persistence`) to detect real voids: a gap in
//!      an otherwise-regular field of headstone-top points reads as a
//!      genuine H2 class — "possible unmarked/missing grave," the same
//!      "void = missing data" narrative this session's PDM-discovery
//!      work established, applied to physical survey data instead of
//!      tabular schema data.
#![forbid(unsafe_code)]

pub mod downsample;
pub mod las;
pub mod ply;
pub mod sfm;
pub mod voids;
pub mod xyz;

use bahyway_algebra::persistence::Point3;

/// A parsed point cloud: positions plus optional per-point RGB color
/// (present when the source format carried it, e.g. PLY vertex colors).
#[derive(Debug, Clone, Default)]
pub struct PointCloud {
    pub points: Vec<Point3>,
    pub colors: Option<Vec<[u8; 3]>>,
}

impl PointCloud {
    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Axis-aligned bounding box: `(min, max)`. `None` for an empty cloud.
    pub fn bounds(&self) -> Option<(Point3, Point3)> {
        let mut iter = self.points.iter();
        let first = *iter.next()?;
        let mut min = first;
        let mut max = first;
        for p in iter {
            for axis in 0..3 {
                if p[axis] < min[axis] {
                    min[axis] = p[axis];
                }
                if p[axis] > max[axis] {
                    max[axis] = p[axis];
                }
            }
        }
        Some((min, max))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ParseError {}

/// Parse a point-cloud file by sniffing its extension (case-insensitive):
/// `.ply` / `.xyz` / `.csv` / `.las`. `.laz` is rejected with a clear
/// error (see `las` module doc) rather than silently misparsed.
pub fn parse_point_cloud_file(path: &std::path::Path) -> Result<PointCloud, ParseError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "ply" => {
            let bytes = std::fs::read(path)
                .map_err(|e| ParseError(format!("reading {}: {e}", path.display())))?;
            ply::parse_ply(&bytes)
        }
        "xyz" | "csv" => {
            let text = std::fs::read_to_string(path)
                .map_err(|e| ParseError(format!("reading {}: {e}", path.display())))?;
            xyz::parse_xyz(&text)
        }
        "las" => {
            let bytes = std::fs::read(path)
                .map_err(|e| ParseError(format!("reading {}: {e}", path.display())))?;
            las::parse_las(&bytes)
        }
        "laz" => Err(ParseError(
            "LAZ (compressed LAS) is not supported: this crate implements the LAS 1.2/1.4 \
             binary spec directly, not LASzip decompression. Decompress to .las first \
             (e.g. via `laszip64 -i file.laz -o file.las`) and parse that."
                .to_string(),
        )),
        other => Err(ParseError(format!(
            "unrecognized point-cloud extension '{other}' (expected .ply/.xyz/.csv/.las)"
        ))),
    }
}
