//! Ninazu (GL-NAV-003): offline navigation over a heptagonal
//! graveyard field. Reads only the daily tile snapshot; routes use
//! only road cells; every grave is one distinct cell.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Grave(u32),
    Road,
    Landmark,
}

/// The daily snapshot — the ONLY source read at navigation time.
pub struct Snapshot {
    pub date: &'static str,
    pub cells: Vec<Vec<Cell>>,
}

/// L69: navigation reads ONLY the snapshot. This function has no
/// network access and no argument that could carry a live handle.
pub fn navigate(
    snap: &Snapshot,
    from: (usize, usize),
    to: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    // BFS over ROAD cells only (never step through a Grave cell, L70)
    use std::collections::{HashMap, VecDeque};
    let h = snap.cells.len();
    let w = snap.cells[0].len();
    let is_walk = |r: usize, c: usize| {
        matches!(snap.cells[r][c], Cell::Road) || (r, c) == from || (r, c) == to
    };
    let mut q = VecDeque::new();
    let mut prev: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    q.push_back(from);
    prev.insert(from, from);
    while let Some((r, c)) = q.pop_front() {
        if (r, c) == to {
            let mut path = vec![to];
            let mut cur = to;
            while cur != from {
                cur = prev[&cur];
                path.push(cur);
            }
            path.reverse();
            return Some(path);
        }
        let nbrs = [
            (r.wrapping_sub(1), c),
            (r + 1, c),
            (r, c.wrapping_sub(1)),
            (r, c + 1),
        ];
        for (nr, nc) in nbrs {
            if nr < h && nc < w && !prev.contains_key(&(nr, nc)) && is_walk(nr, nc) {
                prev.insert((nr, nc), (r, c));
                q.push_back((nr, nc));
            }
        }
    }
    None
}

/// L70 check: a route must never stand on a Grave cell (except endpoints).
pub fn route_avoids_graves(snap: &Snapshot, route: &[(usize, usize)]) -> bool {
    if route.len() < 2 {
        return true;
    }
    route[1..route.len() - 1]
        .iter()
        .all(|&(r, c)| snap.cells[r][c] == Cell::Road)
}

/// L71: every grave id appears in exactly one cell.
pub fn each_grave_one_cell(snap: &Snapshot) -> bool {
    use std::collections::HashMap;
    let mut seen: HashMap<u32, usize> = HashMap::new();
    for row in &snap.cells {
        for cell in row {
            if let Cell::Grave(id) = cell {
                *seen.entry(*id).or_default() += 1;
            }
        }
    }
    seen.values().all(|&n| n == 1)
}
