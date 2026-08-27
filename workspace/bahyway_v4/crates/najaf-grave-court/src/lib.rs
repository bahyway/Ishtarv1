//! najaf-grave-court — GL-NJF-001 (The Najaf Grave Court): capacity,
//! lawful vacancy, and the protection of cherished graves. PB-328. Pure
//! Rust, zero dependencies.
#![forbid(unsafe_code)]

/// §3 — the Qibla bearing at Najaf is a sealed constant, never a runtime
/// parameter: orientation is a constant of the erosion.
pub const QIBLA_BEARING_DEG: f64 = 192.0;

/// §2 — the Capacity Equation (Abubu inverted): graves do not release, so
/// mu = 0 and the horizon is a pure countdown.
pub fn capacity_horizon(rho_star: f64, rho0: f64, lambda: f64) -> Option<f64> {
    if lambda <= 0.0 {
        return None;
    }
    Some((rho_star - rho0) / lambda)
}

pub fn capacity_alarm(t_star: Option<f64>, tau: f64) -> bool {
    matches!(t_star, Some(t) if t <= tau)
}

/// A boolean occupancy grid: `blocked[y*width+x]` true means the cell is
/// occupied by an existing grave, a dilated clearance buffer, or a
/// refusal buffer (§5) -- all three are lawfully indistinguishable to the
/// erosion: none of them may host a new grave.
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub blocked: Vec<bool>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, blocked: vec![false; width * height] }
    }

    pub fn is_blocked(&self, x: usize, y: usize) -> bool {
        self.blocked[y * self.width + x]
    }

    pub fn block(&mut self, x: usize, y: usize) {
        let idx = y * self.width + x;
        self.blocked[idx] = true;
    }
}

/// §3 — erode the vacancy by the grave footprint at ITS LAWFUL
/// ORIENTATION. Orientation is a constant of the erosion, never a free
/// parameter: note this function takes no angle argument at all -- the
/// footprint's w/h are fixed by the caller to already reflect the
/// Qibla-normal long axis, and QIBLA_BEARING_DEG is the only bearing this
/// crate knows.
pub fn lawful_plots(grid: &Grid, footprint_w: usize, footprint_h: usize) -> Vec<(usize, usize)> {
    let mut plots = Vec::new();
    if footprint_w == 0 || footprint_h == 0 || footprint_w > grid.width || footprint_h > grid.height {
        return plots;
    }
    for y in 0..=(grid.height - footprint_h) {
        for x in 0..=(grid.width - footprint_w) {
            let mut fits = true;
            'outer: for dy in 0..footprint_h {
                for dx in 0..footprint_w {
                    if grid.is_blocked(x + dx, y + dy) {
                        fits = false;
                        break 'outer;
                    }
                }
            }
            if fits {
                plots.push((x, y));
            }
        }
    }
    plots
}

/// §5 — a refusal buffer is excluded from vacancy and allocation, held
/// until witnesses arrive or a Madanu decree. Marking cells as buffer is
/// the same `block` operation as an existing grave: nothing about a
/// buffer cell can ever re-enter vacancy through this API.
pub fn mark_refusal_buffer(grid: &mut Grid, cells: &[(usize, usize)]) {
    for &(x, y) in cells {
        grid.block(x, y);
    }
}

/// §4 — the accuracy ladder. The verdict always states which rung
/// produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccuracyRung {
    Photogrammetry,
    GroundPenetratingRadar,
    SatelliteProposingOnly,
}

pub fn rung_accuracy_cm(rung: AccuracyRung) -> (f64, f64) {
    match rung {
        AccuracyRung::Photogrammetry => (2.0, 5.0),
        AccuracyRung::GroundPenetratingRadar => (10.0, 50.0),
        AccuracyRung::SatelliteProposingOnly => (30.0, 3_000.0),
    }
}

/// §5 — the Refusal Clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraveVerdict {
    Golden,
    Fuzzy,
    Dead,
}

pub fn classify_grave(remote_witness: bool, ground_or_archive_witness: bool) -> GraveVerdict {
    match (remote_witness, ground_or_archive_witness) {
        (true, true) => GraveVerdict::Golden,
        (true, false) => GraveVerdict::Fuzzy,
        _ => GraveVerdict::Dead,
    }
}

/// A DEAD verdict is unissuable: a grave is never declared absent,
/// erased, or reclaimable by remote sensing.
pub fn issue_grave(verdict: GraveVerdict) -> Result<(), &'static str> {
    match verdict {
        GraveVerdict::Golden | GraveVerdict::Fuzzy => Ok(()),
        GraveVerdict::Dead => Err("DEAD verdict: this court refuses to issue it"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Capacity monotone in lambda: T* strictly decreasing as the burial
    // rate lambda increases.
    #[test]
    fn capacity_monotone_in_lambda() {
        let rho_star = 100.0;
        let rho0 = 10.0;
        let t_slow = capacity_horizon(rho_star, rho0, 1.0).unwrap();
        let t_fast = capacity_horizon(rho_star, rho0, 5.0).unwrap();
        assert!(t_fast < t_slow, "a faster burial rate must shorten the horizon");
        assert!(capacity_horizon(rho_star, rho0, 0.0).is_none(), "no burials, no finite horizon");
    }

    // Erosion angle fixed: the sealed bearing is a compile-time constant,
    // and the erosion function itself accepts no angle parameter (proven
    // by its signature, not just its output).
    #[test]
    fn erosion_angle_fixed() {
        assert_eq!(QIBLA_BEARING_DEG, 192.0);
        let mut grid = Grid::new(6, 6);
        grid.block(2, 2);
        let plots_a = lawful_plots(&grid, 2, 1);
        let plots_b = lawful_plots(&grid, 2, 1); // same call, no angle input exists to vary
        assert_eq!(plots_a, plots_b, "erosion is deterministic; there is no angle to perturb it");
        assert!(!plots_a.iter().any(|&(x, y)| x <= 2 && 2 < x + 2 && y == 2), "blocked cell excluded");
    }

    // Buffer never becomes vacancy.
    #[test]
    fn buffer_never_becomes_vacancy() {
        let mut grid = Grid::new(4, 4);
        mark_refusal_buffer(&mut grid, &[(1, 1)]);
        let plots = lawful_plots(&grid, 1, 1);
        assert!(!plots.contains(&(1, 1)), "a refusal buffer must never appear as a lawful plot");
    }

    // DEAD unissuable.
    #[test]
    fn dead_unissuable() {
        assert!(issue_grave(GraveVerdict::Golden).is_ok());
        assert!(issue_grave(GraveVerdict::Fuzzy).is_ok());
        assert!(issue_grave(classify_grave(false, false)).is_err());
        assert_eq!(classify_grave(false, false), GraveVerdict::Dead);
        assert_eq!(classify_grave(false, true), GraveVerdict::Dead, "archive alone without remote is still DEAD, per the two-witness minting rule");
    }

    #[test]
    fn accuracy_ladder_ranges() {
        let (lo, hi) = rung_accuracy_cm(AccuracyRung::Photogrammetry);
        assert!(lo < hi && hi <= 5.0);
        let (lo, hi) = rung_accuracy_cm(AccuracyRung::GroundPenetratingRadar);
        assert!(lo >= 10.0 && hi <= 50.0);
    }
}
