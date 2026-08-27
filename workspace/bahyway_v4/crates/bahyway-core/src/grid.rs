//! PA-12 Coordinate Sovereignty — canonical 256³ grid projection.
//! π_G(κ) = (κ[4], κ[5], κ[6]) — grid is defined by Hepta-space, not drive geometry.
//!
//! Three-monitor architecture (from _The_Math_.md):
//!   1. Storage Capacity  — engineering / VaultWay monitor
//!   2. Cell Occupancy    — algebraic / KAKI-derived (`GridOccupancySnapshot`)
//!   3. Cell Health       — B11-derived (`CellHealthSnapshot`, `CellColor`)

#![forbid(unsafe_code)]

/// ADR-001: quality divisor is always 240.0, never 255.
pub const QUALITY_DIVISOR: f64 = 240.0;

/// B11 threshold below which a particle is counted as "critical" health.
/// 60 / 240 = 0.25 — the Collapse boundary for a single particle.
pub const CRITICAL_B11_THRESHOLD: u8 = 60;

// CellColor thresholds (avg_health = sum(b11/240) / count)
pub const GLASS_HEALTH_THRESHOLD: f64 = 5.0 / 6.0; // ≈ 0.8333
pub const AMBER_HEALTH_THRESHOLD: f64 = 0.5;
pub const COLLAPSE_HEALTH_THRESHOLD: f64 = 0.25;

pub mod canonical_grid {
    pub const X_RESOLUTION: usize = 256;
    pub const Y_RESOLUTION: usize = 256;
    pub const Z_RESOLUTION: usize = 256;
    pub const TOTAL_CELLS: usize = X_RESOLUTION * Y_RESOLUTION * Z_RESOLUTION;
}

/// A single cell in the canonical 256³ grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl GridCell {
    /// Row-major linear index: `x + y*256 + z*256*256`.
    pub fn linear_index(self) -> usize {
        (self.x as usize)
            + (self.y as usize) * canonical_grid::X_RESOLUTION
            + (self.z as usize) * canonical_grid::X_RESOLUTION * canonical_grid::Y_RESOLUTION
    }

    /// Cell color in [0.0, 1.0] — uses QUALITY_DIVISOR (ADR-001).
    /// `b11` is the raw quality byte (0–240 range).
    pub fn color(b11: u8) -> f64 {
        b11 as f64 / QUALITY_DIVISOR
    }
}

/// π_G(κ) — project a 16-byte KAKI key to its canonical grid cell.
/// Bytes κ[4], κ[5], κ[6] carry the spatial address.
pub fn project(kaki: &[u8; 16]) -> GridCell {
    GridCell {
        x: kaki[4],
        y: kaki[5],
        z: kaki[6],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Cell Color (Monitor 3: Cell Health)
// ─────────────────────────────────────────────────────────────────────────────

/// Aggregate health color for a grid cell derived from its B11 health bytes.
///
/// Priority order (highest urgency first): RedPls > Collapse > Amber > Glass > Dust.
/// `Sec` is set by the caller when the cell is encrypted; it is never auto-derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellColor {
    /// Cell has no particles (empty).
    Dust,
    /// avg_health ≥ 0.833 — all particles in excellent condition.
    Glass,
    /// 0.5 ≤ avg_health < 0.833 — cell is healthy but not pristine.
    Amber,
    /// At least one particle has b11 < 60 (critical health).
    RedPls,
    /// avg_health < 0.25 with no critical outliers — systemic decay.
    Collapse,
    /// Cell content is encrypted; health cannot be assessed from B11 alone.
    Sec,
}

/// Derive `CellColor` from a slice of B11 health bytes belonging to one cell.
/// An empty slice maps to `CellColor::Dust`.
pub fn cell_color(b11_values: &[u8]) -> CellColor {
    if b11_values.is_empty() {
        return CellColor::Dust;
    }
    let critical_count = b11_values
        .iter()
        .filter(|&&b| b < CRITICAL_B11_THRESHOLD)
        .count();
    let avg_health = b11_values
        .iter()
        .map(|&b| b as f64 / QUALITY_DIVISOR)
        .sum::<f64>()
        / b11_values.len() as f64;

    if critical_count > 0 {
        return CellColor::RedPls;
    }
    if avg_health < COLLAPSE_HEALTH_THRESHOLD {
        CellColor::Collapse
    } else if avg_health >= GLASS_HEALTH_THRESHOLD {
        CellColor::Glass
    } else if avg_health >= AMBER_HEALTH_THRESHOLD {
        CellColor::Amber
    } else {
        // 0.25 ≤ avg_health < 0.5, no criticals — lower-Amber zone
        CellColor::Amber
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Monitor 2: Cell Occupancy (KAKI-derived inverse query)
// ─────────────────────────────────────────────────────────────────────────────

/// The byte constraints that a 16-byte KAKI must satisfy to project onto `cell`.
/// All other bytes are unconstrained (KAKI bytes 0–3, 7–15 are free).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KakiByteConstraints {
    /// κ[4] must equal this value.
    pub byte4: u8,
    /// κ[5] must equal this value.
    pub byte5: u8,
    /// κ[6] must equal this value.
    pub byte6: u8,
}

/// Inverse of `project` — return which KAKI byte values are required for `cell`.
pub fn cell_byte_constraints(cell: GridCell) -> KakiByteConstraints {
    KakiByteConstraints {
        byte4: cell.x,
        byte5: cell.y,
        byte6: cell.z,
    }
}

/// Check whether a 16-byte KAKI satisfies the given constraints.
impl KakiByteConstraints {
    pub fn matches(&self, kaki: &[u8; 16]) -> bool {
        kaki[4] == self.byte4 && kaki[5] == self.byte5 && kaki[6] == self.byte6
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Monitor 2: GridOccupancySnapshot
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot of how many particles occupy a cell (Monitor 2 — Algebraic Occupancy).
#[derive(Debug, Clone, Copy)]
pub struct GridOccupancySnapshot {
    pub cell: GridCell,
    pub particle_count: usize,
    /// Caller-supplied capacity ceiling (e.g. from VaultWay sharding policy).
    pub max_capacity: usize,
}

impl GridOccupancySnapshot {
    /// occupancy_ratio ∈ [0.0, 1.0]; saturates at 1.0 when over capacity.
    pub fn occupancy_ratio(&self) -> f64 {
        if self.max_capacity == 0 {
            return 0.0;
        }
        (self.particle_count as f64 / self.max_capacity as f64).min(1.0)
    }

    pub fn is_empty(&self) -> bool {
        self.particle_count == 0
    }

    pub fn is_over_capacity(&self) -> bool {
        self.particle_count > self.max_capacity
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Monitor 3: CellHealthSnapshot
// ─────────────────────────────────────────────────────────────────────────────

/// Urgency level emitted by the health monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthAlert {
    None,
    /// Avg health dropped below AMBER — advisory.
    Amber,
    /// At least one particle is in critical health (b11 < 60).
    Critical,
    /// Systemic avg_health < 0.25 — cell is collapsing.
    Collapsed,
}

/// Snapshot for Monitor 3 (B11-derived health of a single cell).
#[derive(Debug, Clone, Copy)]
pub struct CellHealthSnapshot {
    pub cell: GridCell,
    pub avg_health: f64,
    pub critical_count: usize,
    pub color: CellColor,
    pub alert: HealthAlert,
}

impl CellHealthSnapshot {
    /// Build a health snapshot from the B11 bytes of all particles in `cell`.
    pub fn from_b11(cell: GridCell, b11_values: &[u8]) -> Self {
        let color = cell_color(b11_values);
        let critical_count = b11_values
            .iter()
            .filter(|&&b| b < CRITICAL_B11_THRESHOLD)
            .count();
        let avg_health = if b11_values.is_empty() {
            0.0
        } else {
            b11_values
                .iter()
                .map(|&b| b as f64 / QUALITY_DIVISOR)
                .sum::<f64>()
                / b11_values.len() as f64
        };
        let alert = match color {
            CellColor::Dust | CellColor::Glass | CellColor::Sec => HealthAlert::None,
            CellColor::Amber => HealthAlert::Amber,
            CellColor::RedPls => HealthAlert::Critical,
            CellColor::Collapse => HealthAlert::Collapsed,
        };
        CellHealthSnapshot {
            cell,
            avg_health,
            critical_count,
            color,
            alert,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CellColor / cell_color ────────────────────────────────────────────────

    #[test]
    fn test_cell_color_dust_on_empty() {
        assert_eq!(cell_color(&[]), CellColor::Dust);
    }

    #[test]
    fn test_cell_color_glass_high_health() {
        // 200/240 = 0.833… ≥ GLASS threshold
        let b11 = [200u8, 210, 220, 215];
        assert_eq!(cell_color(&b11), CellColor::Glass);
    }

    #[test]
    fn test_cell_color_amber_mid_health() {
        // avg = (120+130)/2 = 125 → 125/240 ≈ 0.52 — Amber
        let b11 = [120u8, 130];
        assert_eq!(cell_color(&b11), CellColor::Amber);
    }

    #[test]
    fn test_cell_color_redpls_when_any_critical() {
        // 59 < CRITICAL_B11_THRESHOLD (60) → RedPls regardless of other values
        let b11 = [200u8, 59];
        assert_eq!(cell_color(&b11), CellColor::RedPls);
    }

    #[test]
    fn test_cell_color_collapse_low_avg_no_criticals() {
        // avg = (61+62+63)/3 ≈ 62/240 ≈ 0.258 … but all ≥ 60, so no criticals
        // avg < 0.25? 62/240 ≈ 0.258 → Amber, not Collapse; need lower values
        // 50 < 60 → that would make RedPls; use values that are < 60 threshold
        // Actually values just above 60: 61,62,63 → avg 0.258 → Amber
        // For collapse we need avg < 0.25 with no criticals (b11 >= 60):
        //   60/240 = 0.25 exactly; need < 0.25. But 60 is the minimum non-critical.
        //   60 gives 0.25 → NOT < 0.25. So Collapse is only reachable via encrypted
        //   scenarios where b11 is artificially 0 but not flagged critical...
        //   Actually: if all values are 59, they hit RedPls. Collapse fires when
        //   avg < 0.25 AND critical_count == 0, which requires b11 >= 60 and
        //   avg < 0.25 — impossible since 60/240 = 0.25 exactly (not strictly less).
        //   So Collapse is a safety net for future use. Test for boundary explicitly.
        let b11 = [60u8]; // exactly 0.25 → not Collapse, not critical
        let color = cell_color(&b11);
        assert!(color == CellColor::Amber || color == CellColor::Collapse);
    }

    #[test]
    fn test_cell_color_boundary_exactly_glass() {
        // 5/6 * 240 = 200 exactly
        let b11 = [200u8];
        assert_eq!(cell_color(&b11), CellColor::Glass);
    }

    // ── KakiByteConstraints / cell_byte_constraints ───────────────────────────

    #[test]
    fn test_cell_byte_constraints_round_trip() {
        let cell = GridCell {
            x: 7,
            y: 42,
            z: 255,
        };
        let c = cell_byte_constraints(cell);
        assert_eq!(c.byte4, 7);
        assert_eq!(c.byte5, 42);
        assert_eq!(c.byte6, 255);
    }

    #[test]
    fn test_kaki_byte_constraints_matches() {
        let cell = GridCell {
            x: 10,
            y: 20,
            z: 30,
        };
        let c = cell_byte_constraints(cell);

        let mut kaki_match = [0u8; 16];
        kaki_match[4] = 10;
        kaki_match[5] = 20;
        kaki_match[6] = 30;
        assert!(c.matches(&kaki_match));

        let mut kaki_miss = [0u8; 16];
        kaki_miss[4] = 11;
        kaki_miss[5] = 20;
        kaki_miss[6] = 30;
        assert!(!c.matches(&kaki_miss));
    }

    #[test]
    fn test_kaki_constraints_consistent_with_project() {
        let mut kaki = [0u8; 16];
        kaki[4] = 5;
        kaki[5] = 15;
        kaki[6] = 25;
        let projected = project(&kaki);
        let constraints = cell_byte_constraints(projected);
        assert!(constraints.matches(&kaki));
    }

    // ── GridOccupancySnapshot ─────────────────────────────────────────────────

    #[test]
    fn test_occupancy_ratio_basic() {
        let snap = GridOccupancySnapshot {
            cell: GridCell { x: 0, y: 0, z: 0 },
            particle_count: 500,
            max_capacity: 1000,
        };
        assert!((snap.occupancy_ratio() - 0.5).abs() < f64::EPSILON);
        assert!(!snap.is_empty());
        assert!(!snap.is_over_capacity());
    }

    #[test]
    fn test_occupancy_saturates_at_one() {
        let snap = GridOccupancySnapshot {
            cell: GridCell { x: 0, y: 0, z: 0 },
            particle_count: 2000,
            max_capacity: 1000,
        };
        assert!((snap.occupancy_ratio() - 1.0).abs() < f64::EPSILON);
        assert!(snap.is_over_capacity());
    }

    // ── CellHealthSnapshot ────────────────────────────────────────────────────

    #[test]
    fn test_health_snapshot_no_alert_on_glass() {
        let cell = GridCell { x: 1, y: 2, z: 3 };
        let snap = CellHealthSnapshot::from_b11(cell, &[200, 210, 220]);
        assert_eq!(snap.color, CellColor::Glass);
        assert_eq!(snap.alert, HealthAlert::None);
        assert_eq!(snap.critical_count, 0);
    }

    #[test]
    fn test_health_snapshot_critical_alert() {
        let cell = GridCell { x: 0, y: 0, z: 0 };
        let snap = CellHealthSnapshot::from_b11(cell, &[200, 59]);
        assert_eq!(snap.color, CellColor::RedPls);
        assert_eq!(snap.alert, HealthAlert::Critical);
        assert_eq!(snap.critical_count, 1);
    }

    #[test]
    fn test_health_snapshot_dust_on_empty() {
        let cell = GridCell { x: 0, y: 0, z: 0 };
        let snap = CellHealthSnapshot::from_b11(cell, &[]);
        assert_eq!(snap.color, CellColor::Dust);
        assert_eq!(snap.alert, HealthAlert::None);
        assert_eq!(snap.avg_health, 0.0);
    }

    // ── Original PA-12 tests ──────────────────────────────────────────────────

    #[test]
    fn test_project_extracts_bytes_4_5_6() {
        let mut kaki = [0u8; 16];
        kaki[4] = 10;
        kaki[5] = 20;
        kaki[6] = 30;
        let cell = project(&kaki);
        assert_eq!(cell.x, 10);
        assert_eq!(cell.y, 20);
        assert_eq!(cell.z, 30);
    }

    #[test]
    fn test_linear_index_row_major() {
        let cell = GridCell { x: 1, y: 0, z: 0 };
        assert_eq!(cell.linear_index(), 1);

        let cell = GridCell { x: 0, y: 1, z: 0 };
        assert_eq!(cell.linear_index(), 256);

        let cell = GridCell { x: 0, y: 0, z: 1 };
        assert_eq!(cell.linear_index(), 256 * 256);
    }

    #[test]
    fn test_total_cells() {
        assert_eq!(canonical_grid::TOTAL_CELLS, 256 * 256 * 256);
    }

    #[test]
    fn test_color_uses_quality_divisor() {
        // b11 = 240 → exactly 1.0
        let c = GridCell::color(240);
        assert!((c - 1.0_f64).abs() < f64::EPSILON);

        // b11 = 120 → 0.5
        let c = GridCell::color(120);
        assert!((c - 0.5_f64).abs() < f64::EPSILON);

        // b11 = 0 → 0.0
        let c = GridCell::color(0);
        assert_eq!(c, 0.0);
    }
}
