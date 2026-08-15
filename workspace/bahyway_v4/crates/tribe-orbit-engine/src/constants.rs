//! Sovereign constants for tribe orbital mechanics.
//!
//! These values are eternal — never change them.
//! Any PR modifying these must be rejected.

/// 1 Particle Unit = 7 KAKI particles — the Heptagon constant.
pub const PARTICLES_PER_TRIBE: usize = 7;

/// Sovereign Heptagon Constant — normalisation factor.
pub const SHC: f32 = 1.0 / 255.0;

/// Rule 7 trigger threshold — overflow when density ≥ 11 particles.
pub const TAU_R7: usize = 11;

/// Minimum cluster size before density effects apply.
pub const CLUSTER_THRESH: usize = 3;

/// SPH smoothing length (resonance radius) — neighbourhood kernel bandwidth.
pub const RESONANCE_RADIUS: f32 = 0.15;

/// Isolated density band upper bound (0–2 neighbours).
pub const DENSITY_ISOLATED_MAX: usize = 2;

/// Cluster density band (3–10 neighbours).
pub const DENSITY_CLUSTER_MAX: usize = 10;
// ≥ 11 = Rule7Overflow (use TAU_R7)

/// Maximum orbital radius (outermost dead ring).
pub const ORBIT_RADIUS_MAX: f32 = 4.0;

/// Inner orbit radius (GEM particles).
pub const ORBIT_RADIUS_GEM: f32 = 1.0;

/// Tribal law priority order (ascending — lowest index = lowest priority).
///
/// Law indices (1-based): L3 < L7 < L1 < L2 < L5 < L4 < L6
/// Encoded as 0-based indices: [2, 6, 0, 1, 4, 3, 5]
pub const LAW_PRIORITY_ORDER: [usize; 7] = [2, 6, 0, 1, 4, 3, 5];
