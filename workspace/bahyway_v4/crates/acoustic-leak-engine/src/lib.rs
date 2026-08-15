// ============================================================================
// acoustic-leak-engine — Sovereign Water Pipe Acoustic Leak Localization
// BahyWay.Ecosystem v4.0
//
// From the iPhone-session PB-150 review ("WPD-Engine v4.0"), landed under a
// distinct crate name: a real, different crates/wpd-engine already exists
// in this workspace (defect classification, 12-band multispectral scoring,
// Baghdad sector repair-priority scheduling) with zero conceptual overlap
// with this crate's acoustic/correlation leak-localization design. Same
// working name, two unrelated engines -- this one keeps its own identity.
//
// SOVEREIGN LAWS ENCODED HERE:
//   LAW-ALE-1  A position verdict is PROVEN, never asserted: sigma_d <=
//              0.10 m or the verdict is refused (escalate stage).
//   LAW-ALE-2  Pipe material selects the solver. One universal solver is
//              false universality -- the baseline model must match the
//              medium.
//   LAW-ALE-3  Wave speed v is calibrated per segment at measurement time.
//              A stored textbook v is a lie carried forward.
//   LAW-ALE-4  This engine never mints KAKI. It emits LeakAlertEvent to the
//              real enkidb-ingest bridge, the only lawful minting site.
//   LAW-ALE-5  Mathematical primitives here are candidates for GeoEngine
//              (bahyway-algebra) once proven; vendored here until then.
// ============================================================================
#![forbid(unsafe_code)]

pub mod calibration;
pub mod crlb;
pub mod event;
pub mod material;
pub mod solver;
pub mod stage1_kingplot;
pub mod verdict;

/// The sovereign localization margin of BahyWay v4.0: ten centimetres.
pub const SOVEREIGN_MARGIN_M: f64 = 0.10;

/// A correlation peak closer than this to a node forces Stage-3 confirmation
/// (junction multipath cannot be resolved by correlation alone).
pub const JUNCTION_GUARD_M: f64 = 2.0;
