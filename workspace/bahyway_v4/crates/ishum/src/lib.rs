//! ishum — GL-ISM-001 (IshumEngine Founding Tablet): the Ishum skeleton
//! kernel — chamfer clearance transform, H0 components, monotone
//! connectivity + disconnect-width bisection, constellation fingerprint.
//! PB-329. Pure Rust, zero dependencies.
#![forbid(unsafe_code)]

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
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
    pub fn set_blocked(&mut self, x: usize, y: usize) {
        let i = self.idx(x, y);
        self.blocked[i] = true;
    }
    pub fn is_blocked(&self, x: usize, y: usize) -> bool {
        self.blocked[self.idx(x, y)]
    }
}

/// Organ I — the chamfer clearance transform: distance to the nearest
/// blocked (grave) cell. Two-pass chessboard sweep, pure Rust.
pub fn clearance_transform(grid: &Grid) -> Vec<f64> {
    let (w, h) = (grid.width, grid.height);
    let mut dist = vec![f64::INFINITY; w * h];
    for y in 0..h {
        for x in 0..w {
            if grid.is_blocked(x, y) {
                dist[grid.idx(x, y)] = 0.0;
            }
        }
    }
    for y in 0..h {
        for x in 0..w {
            let i = grid.idx(x, y);
            let mut d = dist[i];
            if x > 0 {
                d = d.min(dist[grid.idx(x - 1, y)] + 1.0);
            }
            if y > 0 {
                d = d.min(dist[grid.idx(x, y - 1)] + 1.0);
            }
            dist[i] = d;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = grid.idx(x, y);
            let mut d = dist[i];
            if x + 1 < w {
                d = d.min(dist[grid.idx(x + 1, y)] + 1.0);
            }
            if y + 1 < h {
                d = d.min(dist[grid.idx(x, y + 1)] + 1.0);
            }
            dist[i] = d;
        }
    }
    dist
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self { parent: (0..n).collect() }
    }
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            let root = self.find(self.parent[x]);
            self.parent[x] = root;
        }
        self.parent[x]
    }
    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            self.parent[ra] = rb;
        }
    }
}

/// Organ II — "can a traveler of width w pass?": erode free space by
/// w/2, test H0 connectivity via union-find. A blocked cell (grave OR
/// refusal buffer, §V "buffers block") has clearance 0 and is never
/// walkable for any w > 0.
pub fn connected_at_width(
    grid: &Grid,
    clearance: &[f64],
    a: (usize, usize),
    b: (usize, usize),
    w: f64,
) -> bool {
    let req = w / 2.0;
    let (width, height) = (grid.width, grid.height);
    let walkable = |x: usize, y: usize| clearance[y * width + x] >= req;
    if !walkable(a.0, a.1) || !walkable(b.0, b.1) {
        return false;
    }
    let mut uf = UnionFind::new(width * height);
    for y in 0..height {
        for x in 0..width {
            if !walkable(x, y) {
                continue;
            }
            let i = y * width + x;
            if x + 1 < width && walkable(x + 1, y) {
                uf.union(i, y * width + x + 1);
            }
            if y + 1 < height && walkable(x, y + 1) {
                uf.union(i, (y + 1) * width + x);
            }
        }
    }
    uf.find(a.1 * width + a.0) == uf.find(b.1 * width + b.0)
}

/// Bisection over the monotone severance predicate (L1: once severed at
/// w, severed for all w' > w). O(log n) evaluations of
/// `connected_at_width` against a sorted candidate list.
pub fn disconnect_width_bisection(
    grid: &Grid,
    clearance: &[f64],
    a: (usize, usize),
    b: (usize, usize),
    candidate_widths: &[f64],
) -> Option<f64> {
    let n = candidate_widths.len();
    if n == 0 {
        return None;
    }
    let (mut lo, mut hi) = (0usize, n);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if !connected_at_width(grid, clearance, a, b, candidate_widths[mid]) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    if lo == n {
        None
    } else {
        Some(candidate_widths[lo])
    }
}

pub fn disconnect_width_linear_scan(
    grid: &Grid,
    clearance: &[f64],
    a: (usize, usize),
    b: (usize, usize),
    candidate_widths: &[f64],
) -> Option<f64> {
    candidate_widths
        .iter()
        .find(|&&w| !connected_at_width(grid, clearance, a, b, w))
        .copied()
}

/// Organ III — a k-nearest constellation fingerprint. Distances alone
/// are rotation-invariant by construction: rotating the whole scene
/// around the query point leaves every pairwise distance unchanged.
pub fn constellation_fingerprint(query: (f64, f64), stars: &[(f64, f64)], k: usize) -> Vec<f64> {
    let mut dists: Vec<f64> = stars
        .iter()
        .map(|&(sx, sy)| ((sx - query.0).powi(2) + (sy - query.1).powi(2)).sqrt())
        .collect();
    dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
    dists.truncate(k);
    dists
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_grid(w: usize, h: usize) -> Grid {
        Grid::new(w, h)
    }

    // L1 — monotone severance: once severed at w, severed for all w' > w.
    #[test]
    fn l1_monotone_severance() {
        let mut grid = open_grid(10, 3);
        // A single-cell-wide gap at (5,1) between two grave walls.
        for y in 0..3usize {
            if y != 1 {
                grid.set_blocked(5, y);
            }
        }
        let clearance = clearance_transform(&grid);
        let a = (0, 1);
        let b = (9, 1);
        let widths = [0.5, 1.0, 1.5, 2.0, 3.0];
        let mut seen_disconnect = false;
        for &w in &widths {
            let connected = connected_at_width(&grid, &clearance, a, b, w);
            if !connected {
                seen_disconnect = true;
            } else {
                assert!(!seen_disconnect, "connectivity must not return after being severed (w={w})");
            }
        }
        assert!(seen_disconnect, "the scripted grid must sever at some tested width");
    }

    // L2 — buffers block: a refusal buffer cell is never walkable and a
    // path never routes through it.
    #[test]
    fn l2_buffers_block() {
        let mut grid = open_grid(5, 5);
        grid.set_blocked(2, 2); // treated identically whether grave or refusal buffer
        let clearance = clearance_transform(&grid);
        assert_eq!(clearance[2 * 5 + 2], 0.0);
        assert!(!connected_at_width(&grid, &clearance, (2, 2), (2, 2), 0.1));
    }

    // L3 — bisection equals linear scan.
    #[test]
    fn l3_bisection_matches_scan() {
        let mut grid = open_grid(12, 3);
        for y in 0..3usize {
            if y != 1 {
                grid.set_blocked(6, y);
            }
        }
        let clearance = clearance_transform(&grid);
        let a = (0, 1);
        let b = (11, 1);
        let widths = [0.2, 0.6, 1.0, 1.4, 1.8, 2.2, 2.6, 3.0];
        assert_eq!(
            disconnect_width_bisection(&grid, &clearance, a, b, &widths),
            disconnect_width_linear_scan(&grid, &clearance, a, b, &widths)
        );
    }

    // L4 — fingerprints distinguish neighbors.
    #[test]
    fn l4_fingerprints_distinguish_neighbors() {
        let stars = vec![(0.0, 0.0), (2.0, 0.0), (0.0, 3.0), (5.0, 5.0), (-2.0, 1.0)];
        let fp_a = constellation_fingerprint((0.1, 0.1), &stars, 3);
        let fp_b = constellation_fingerprint((4.9, 4.9), &stars, 3);
        assert_ne!(fp_a, fp_b, "two distinct neighborhoods must not share a fingerprint");
        assert_eq!(fp_a.len(), 3);
    }
}
