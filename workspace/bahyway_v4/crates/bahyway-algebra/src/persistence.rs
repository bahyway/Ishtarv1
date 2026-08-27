//! Persistent homology over a Vietoris-Rips filtration.
//!
//! GeoEngine is the sole math-truth source (design law, sealed 2026-07-17
//! in the Particle Death & Legacy Law discussion): this module is the ONLY
//! place persistent homology is computed anywhere in BahyWay. LamassuEngine
//! (the TDA sentinel/orchestrator crate) calls into this module and must
//! never recompute homology itself — it samples point clouds and reads the
//! diagram this module returns.
//!
//! Pure std, zero external dependencies: the boundary-matrix reduction
//! algorithm (Zomorodian & Carlsson, "Computing Persistent Homology," 2005)
//! is implemented directly over GF(2) using sorted `Vec<usize>` columns
//! (a row index present in the column = a 1-bit; symmetric difference of
//! two sorted vectors is XOR over GF(2)).
//!
//! Scope: H0 (connected components), H1 (loops), and H2 (voids). Vietoris-
//! Rips tetrahedra enumeration (needed for H2) is O(n^4) in the point-cloud
//! size — one order worse than the O(n^3) triangle enumeration H0/H1 alone
//! required — so callers MUST downsample before calling this, and harder
//! than an H0/H1-only caller would need to (that discipline lives in
//! LamassuEngine, per the design's "downsampled representative cloud per
//! Tribe per epoch, capped at H2" rule: H2 is the ceiling, not optional
//! future work).

use std::collections::HashMap;

/// A point in 3D space — the shape LamassuEngine samples orbits into.
pub type Point3 = [f64; 3];

/// One (birth, death) pair from the persistence diagram.
///
/// `dim` is the homology dimension the pair contributes to: 0 = a
/// connected component (H0), 1 = a loop (H1). `death` is `f64::INFINITY`
/// for a class that survives to `max_epsilon` without being filled in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PersistencePair {
    pub dim: u8,
    pub birth: f64,
    pub death: f64,
}

impl PersistencePair {
    /// Lifetime of this class. `f64::INFINITY` if it never dies.
    pub fn persistence(&self) -> f64 {
        self.death - self.birth
    }
}

/// The full persistence diagram for one sampled point cloud.
#[derive(Debug, Clone, Default)]
pub struct PersistenceDiagram {
    pub pairs: Vec<PersistencePair>,
}

impl PersistenceDiagram {
    pub fn h0_pairs(&self) -> impl Iterator<Item = &PersistencePair> {
        self.pairs.iter().filter(|p| p.dim == 0)
    }

    pub fn h1_pairs(&self) -> impl Iterator<Item = &PersistencePair> {
        self.pairs.iter().filter(|p| p.dim == 1)
    }

    pub fn h1_count(&self) -> usize {
        self.h1_pairs().count()
    }

    /// The longest-lived H1 class's lifetime, or 0.0 if no H1 class exists.
    /// A single loud, long bar here is the signature of a genuine orbit
    /// ("the un-circle announcing itself"); several short bars are the
    /// FUZZY seam; none at all is DEAD.
    pub fn max_h1_persistence(&self) -> f64 {
        self.h1_pairs()
            .map(PersistencePair::persistence)
            .fold(0.0_f64, f64::max)
    }

    /// Number of infinite-persistence H0 bars = number of connected
    /// components that never merge within `max_epsilon`.
    pub fn component_count(&self) -> usize {
        self.h0_pairs().filter(|p| p.death.is_infinite()).count()
    }

    pub fn h2_pairs(&self) -> impl Iterator<Item = &PersistencePair> {
        self.pairs.iter().filter(|p| p.dim == 2)
    }

    /// Number of infinite-persistence H2 bars = number of voids (enclosed
    /// cavities) that never get filled in by a tetrahedron within
    /// `max_epsilon`. This is the "missing data / structural hole" signal:
    /// a void born from an incomplete point cloud collapses (this count
    /// drops to 0) once the missing point is supplied and its coning-in
    /// tetrahedra enter the filtration.
    pub fn void_count(&self) -> usize {
        self.h2_pairs().filter(|p| p.death.is_infinite()).count()
    }

    /// The longest-lived H2 class's lifetime, or 0.0 if no H2 class exists.
    pub fn max_h2_persistence(&self) -> f64 {
        self.h2_pairs()
            .map(PersistencePair::persistence)
            .fold(0.0_f64, f64::max)
    }
}

fn euclidean(a: &Point3, b: &Point3) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// One simplex in the filtration: a sorted vertex-index set plus the
/// Vietoris-Rips filtration value at which it enters (max pairwise
/// distance among its vertices; 0.0 for a single vertex).
#[derive(Debug, Clone)]
struct Simplex {
    verts: Vec<usize>,
    value: f64,
}

/// Compute the H0/H1/H2 persistence diagram of a point cloud's Vietoris-Rips
/// filtration, up to `max_epsilon`.
///
/// Downsample the point cloud before calling this — tetrahedra enumeration
/// (needed for H2) is O(n^4); callers sampling whole tribes must use a
/// landmark subset, more aggressively than an H0/H1-only cap would need.
pub fn vietoris_rips_persistence(points: &[Point3], max_epsilon: f64) -> PersistenceDiagram {
    let n = points.len();
    if n == 0 {
        return PersistenceDiagram::default();
    }

    // ── Build the filtration: vertices, then edges, then triangles ──────
    let mut simplices: Vec<Simplex> = Vec::with_capacity(n);
    for i in 0..n {
        simplices.push(Simplex {
            verts: vec![i],
            value: 0.0,
        });
    }
    for i in 0..n {
        for j in (i + 1)..n {
            let d = euclidean(&points[i], &points[j]);
            if d <= max_epsilon {
                simplices.push(Simplex {
                    verts: vec![i, j],
                    value: d,
                });
            }
        }
    }
    for i in 0..n {
        for j in (i + 1)..n {
            let dij = euclidean(&points[i], &points[j]);
            if dij > max_epsilon {
                continue;
            }
            for k in (j + 1)..n {
                let dik = euclidean(&points[i], &points[k]);
                let djk = euclidean(&points[j], &points[k]);
                let v = dij.max(dik).max(djk);
                if v <= max_epsilon {
                    simplices.push(Simplex {
                        verts: vec![i, j, k],
                        value: v,
                    });
                }
            }
        }
    }
    // H2 (voids): the same enumeration shape as triangles, one dimension
    // up. This is the only piece H0/H1 was missing -- the reduction
    // algorithm below is already dimension-generic (it derives a class's
    // dimension from verts.len(), not from a hardcoded triangle/edge
    // case), so no changes are needed past this enumeration step. Real
    // cost, not reused from triangles: O(n^4) worst case versus
    // triangles' O(n^3) -- callers must downsample harder than an H0/H1-
    // only scan would require.
    for i in 0..n {
        for j in (i + 1)..n {
            let dij = euclidean(&points[i], &points[j]);
            if dij > max_epsilon {
                continue;
            }
            for k in (j + 1)..n {
                let dik = euclidean(&points[i], &points[k]);
                let djk = euclidean(&points[j], &points[k]);
                if dik > max_epsilon || djk > max_epsilon {
                    continue;
                }
                for l in (k + 1)..n {
                    let dil = euclidean(&points[i], &points[l]);
                    let djl = euclidean(&points[j], &points[l]);
                    let dkl = euclidean(&points[k], &points[l]);
                    let v = dij.max(dik).max(djk).max(dil).max(djl).max(dkl);
                    if v <= max_epsilon {
                        simplices.push(Simplex {
                            verts: vec![i, j, k, l],
                            value: v,
                        });
                    }
                }
            }
        }
    }

    reduce_filtration(simplices)
}

/// One vertex per named entity (e.g. one per table, for the PDM-discovery
/// use case) plus a caller-supplied relationship graph, gated at
/// `max_weight` instead of a Euclidean `max_epsilon`. This is the
/// "relationship-based complex-builder" GeoEngine was missing: the exact
/// same clique-enumeration and boundary-matrix reduction as
/// [`vietoris_rips_persistence`], just triggered by an arbitrary caller-
/// supplied graph (e.g. discovered foreign-key/join-key relationships)
/// instead of a 3D point cloud's pairwise distance. `edges` need not be
/// symmetric or deduplicated; `(i, j, weight)` and `(j, i, weight)` are
/// treated the same. A lower `weight` enters the filtration earlier —
/// callers should map "more confident relationship" to "lower weight"
/// (e.g. `1.0 - overlap_confidence`), mirroring "closer points merge
/// first" in the geometric case.
pub fn clique_complex_persistence(
    n_vertices: usize,
    edges: &[(usize, usize, f64)],
    max_weight: f64,
) -> PersistenceDiagram {
    if n_vertices == 0 {
        return PersistenceDiagram::default();
    }

    let mut adjacency: HashMap<usize, HashMap<usize, f64>> = HashMap::new();
    for &(a, b, w) in edges {
        if a == b || w > max_weight {
            continue;
        }
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        adjacency.entry(lo).or_default().insert(hi, w);
    }
    let edge_weight = |a: usize, b: usize| -> Option<f64> {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        adjacency.get(&lo).and_then(|row| row.get(&hi)).copied()
    };

    let mut simplices: Vec<Simplex> = Vec::with_capacity(n_vertices);
    for i in 0..n_vertices {
        simplices.push(Simplex {
            verts: vec![i],
            value: 0.0,
        });
    }
    for i in 0..n_vertices {
        for j in (i + 1)..n_vertices {
            if let Some(w) = edge_weight(i, j) {
                simplices.push(Simplex {
                    verts: vec![i, j],
                    value: w,
                });
            }
        }
    }
    for i in 0..n_vertices {
        for j in (i + 1)..n_vertices {
            let Some(wij) = edge_weight(i, j) else {
                continue;
            };
            for k in (j + 1)..n_vertices {
                let (Some(wik), Some(wjk)) = (edge_weight(i, k), edge_weight(j, k)) else {
                    continue;
                };
                simplices.push(Simplex {
                    verts: vec![i, j, k],
                    value: wij.max(wik).max(wjk),
                });
            }
        }
    }
    for i in 0..n_vertices {
        for j in (i + 1)..n_vertices {
            let Some(wij) = edge_weight(i, j) else {
                continue;
            };
            for k in (j + 1)..n_vertices {
                let (Some(wik), Some(wjk)) = (edge_weight(i, k), edge_weight(j, k)) else {
                    continue;
                };
                for l in (k + 1)..n_vertices {
                    let (Some(wil), Some(wjl), Some(wkl)) =
                        (edge_weight(i, l), edge_weight(j, l), edge_weight(k, l))
                    else {
                        continue;
                    };
                    let v = wij.max(wik).max(wjk).max(wil).max(wjl).max(wkl);
                    simplices.push(Simplex {
                        verts: vec![i, j, k, l],
                        value: v,
                    });
                }
            }
        }
    }

    reduce_filtration(simplices)
}

/// Filtration order, boundary-matrix assembly, and the standard
/// Zomorodian-Carlsson reduction — shared by [`vietoris_rips_persistence`]
/// (Euclidean-distance-triggered simplices) and
/// [`clique_complex_persistence`] (arbitrary-graph-triggered simplices).
/// Dimension-generic: derives a class's dimension from `verts.len()`, not
/// a hardcoded triangle/edge case, so it needed no changes to gain H2.
fn reduce_filtration(mut simplices: Vec<Simplex>) -> PersistenceDiagram {
    // Filtration order: by value, ties broken by dimension (faces must
    // precede their cofaces for the boundary matrix to be well-formed).
    simplices.sort_by(|a, b| {
        a.value
            .partial_cmp(&b.value)
            .unwrap()
            .then(a.verts.len().cmp(&b.verts.len()))
    });

    let mut index_of: HashMap<Vec<usize>, usize> = HashMap::with_capacity(simplices.len());
    for (idx, s) in simplices.iter().enumerate() {
        index_of.insert(s.verts.clone(), idx);
    }

    // Boundary matrix: column j lists the (sorted, ascending) row indices
    // of its codimension-1 faces. Vertices have an empty boundary.
    let m = simplices.len();
    let mut reduced: Vec<Vec<usize>> = vec![Vec::new(); m];
    for (idx, s) in simplices.iter().enumerate() {
        if s.verts.len() < 2 {
            continue;
        }
        let mut faces = Vec::with_capacity(s.verts.len());
        for skip in 0..s.verts.len() {
            let mut face = s.verts.clone();
            face.remove(skip);
            faces.push(index_of[&face]);
        }
        faces.sort_unstable();
        reduced[idx] = faces;
    }

    // ── Standard persistence algorithm (Zomorodian-Carlsson) ────────────
    let mut low_to_col: HashMap<usize, usize> = HashMap::new();
    let mut column_empty: Vec<bool> = vec![false; m];
    let mut got_paired: Vec<bool> = vec![false; m];
    let mut pairs = Vec::new();

    for j in 0..m {
        loop {
            match reduced[j].last().copied() {
                None => break,
                Some(low_idx) => match low_to_col.get(&low_idx) {
                    Some(&j2) => reduced[j] = xor_sorted(&reduced[j], &reduced[j2]),
                    None => {
                        low_to_col.insert(low_idx, j);
                        break;
                    }
                },
            }
        }
        match reduced[j].last().copied() {
            Some(low_idx) => {
                got_paired[low_idx] = true;
                let birth_val = simplices[low_idx].value;
                let death_val = simplices[j].value;
                if death_val > birth_val {
                    let dim = (simplices[low_idx].verts.len() - 1) as u8;
                    pairs.push(PersistencePair {
                        dim,
                        birth: birth_val,
                        death: death_val,
                    });
                }
            }
            None => column_empty[j] = true,
        }
    }

    // Births that never found a death within max_epsilon survive forever.
    for j in 0..m {
        if column_empty[j] && !got_paired[j] {
            let dim = (simplices[j].verts.len() - 1) as u8;
            pairs.push(PersistencePair {
                dim,
                birth: simplices[j].value,
                death: f64::INFINITY,
            });
        }
    }

    PersistenceDiagram { pairs }
}

/// Symmetric difference of two sorted, deduplicated index lists — XOR
/// over GF(2) for the boundary-matrix reduction.
fn xor_sorted(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            out.push(a[i]);
            i += 1;
        } else if a[i] > b[j] {
            out.push(b[j]);
            j += 1;
        } else {
            i += 1;
            j += 1;
        }
    }
    out.extend_from_slice(&a[i..]);
    out.extend_from_slice(&b[j..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;

    #[test]
    fn empty_cloud_has_no_pairs() {
        let diag = vietoris_rips_persistence(&[], 1.0);
        assert!(diag.pairs.is_empty());
    }

    #[test]
    fn single_point_is_one_infinite_h0_bar_no_h1() {
        let diag = vietoris_rips_persistence(&[[0.0, 0.0, 0.0]], 1.0);
        assert_eq!(diag.component_count(), 1);
        assert_eq!(diag.h1_count(), 0);
    }

    #[test]
    fn tight_cluster_has_no_persistent_h1_dead_signature() {
        // A small blob with no ring structure: everything merges into one
        // component (H0) and no loop should persist (H1 empty or trivial).
        let points: Vec<Point3> = vec![
            [0.0, 0.0, 0.0],
            [0.05, 0.02, 0.0],
            [0.02, 0.05, 0.0],
            [0.04, 0.04, 0.0],
            [0.01, 0.01, 0.01],
        ];
        let diag = vietoris_rips_persistence(&points, 0.2);
        assert_eq!(
            diag.component_count(),
            1,
            "tight cluster fully merges into one component"
        );
        assert_eq!(
            diag.h1_count(),
            0,
            "DEAD signature: no loop in a solid blob"
        );
    }

    #[test]
    fn ring_of_points_produces_one_loud_persistent_h1_golden_signature() {
        // 12 points evenly spaced on a circle of radius 1. Neighbour
        // spacing ~0.52; the diagonal (opposite-side) distance is 2.0.
        // At epsilon just above neighbour spacing, edges form the ring
        // (an H1 cycle) but not the chords that would fill it in — so
        // exactly one H1 class should be born, and it should persist for
        // a long stretch of epsilon before a triangle finally caps it.
        let n = 12;
        let points: Vec<Point3> = (0..n)
            .map(|i| {
                let a = TAU * (i as f64) / (n as f64);
                [a.cos(), a.sin(), 0.0]
            })
            .collect();
        let diag = vietoris_rips_persistence(&points, 1.2);
        assert_eq!(
            diag.component_count(),
            1,
            "the ring is one connected component"
        );
        assert!(
            diag.h1_count() >= 1,
            "a ring must produce at least one H1 class"
        );
        // The un-circle announcing itself: one dot far above the diagonal.
        let max_persist = diag.max_h1_persistence();
        assert!(
            max_persist > 0.3,
            "the ring's H1 class must be long-lived (GOLDEN signature), got {max_persist}"
        );
    }

    #[test]
    fn two_separated_clusters_are_two_infinite_h0_bars() {
        let points: Vec<Point3> = vec![
            [0.0, 0.0, 0.0],
            [0.05, 0.0, 0.0],
            [0.0, 0.05, 0.0],
            [10.0, 0.0, 0.0],
            [10.05, 0.0, 0.0],
            [10.0, 0.05, 0.0],
        ];
        // Epsilon smaller than the inter-cluster gap: the two blobs never merge.
        let diag = vietoris_rips_persistence(&points, 0.2);
        assert_eq!(
            diag.component_count(),
            2,
            "two far-apart blobs stay two components"
        );
    }

    /// The 6 vertices of an octahedron: (+-1,0,0), (0,+-1,0), (0,0,+-1).
    /// Adjacent (non-antipodal) vertices are distance sqrt(2) apart;
    /// antipodal pairs are distance 2 apart. At an epsilon between those
    /// two values, the Vietoris-Rips flag complex is exactly the
    /// octahedron's boundary surface (its graph is K_{2,2,2}: every vertex
    /// adjacent to all but its antipode) — 6 vertices, 12 edges, 8
    /// triangular faces, and *no* tetrahedra, since a 4-clique in
    /// K_{2,2,2} would need two vertices from the same antipodal pair,
    /// which are never adjacent. That boundary surface is topologically a
    /// hollow sphere: a genuine enclosed void with nothing sampled at its
    /// center.
    fn octahedron_vertices() -> Vec<Point3> {
        vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ]
    }

    #[test]
    fn hollow_octahedron_shell_has_one_persistent_h2_void() {
        let points = octahedron_vertices();
        let diag = vietoris_rips_persistence(&points, 1.5);
        assert_eq!(
            diag.component_count(),
            1,
            "the shell is one connected surface"
        );
        assert_eq!(
            diag.h1_count(),
            0,
            "the closed surface has no un-filled loop, only a void"
        );
        assert_eq!(
            diag.void_count(),
            1,
            "a hollow shell encloses exactly one void — the missing center"
        );
        assert!(
            diag.max_h2_persistence().is_infinite(),
            "nothing in the cloud ever fills the void within max_epsilon"
        );
    }

    #[test]
    fn filling_the_center_point_collapses_the_void() {
        // Same shell, plus the one point that was missing: the center.
        // Every vertex is distance 1.0 from the center (< the 1.414
        // adjacent-vertex spacing), so the center cones over every face,
        // turning the hollow shell into a solid, contractible ball. This
        // is the concrete "structural hole -> supply the missing data ->
        // hole closes" story the void-detection feature exists for.
        let mut points = octahedron_vertices();
        points.push([0.0, 0.0, 0.0]);
        let diag = vietoris_rips_persistence(&points, 1.5);
        assert_eq!(diag.component_count(), 1);
        assert_eq!(
            diag.void_count(),
            0,
            "coning in the missing center fills the void completely"
        );
    }

    #[test]
    fn clique_complex_empty_graph_has_no_pairs() {
        let diag = clique_complex_persistence(0, &[], 1.0);
        assert!(diag.pairs.is_empty());
    }

    #[test]
    fn clique_complex_five_cycle_with_no_chords_is_persistent_h1() {
        // A 5-vertex ring with only the ring edges (no diagonals): the
        // same "un-circle" shape as ring_of_points_produces_one_loud_..,
        // but driven by an explicit relationship graph instead of
        // Euclidean coordinates -- this is the "relationship-based
        // complex-builder" the PDM-discovery paradigm needs.
        let edges = [
            (0, 1, 0.1),
            (1, 2, 0.1),
            (2, 3, 0.1),
            (3, 4, 0.1),
            (4, 0, 0.1),
        ];
        let diag = clique_complex_persistence(5, &edges, 1.0);
        assert_eq!(diag.component_count(), 1);
        assert_eq!(
            diag.h1_count(),
            1,
            "no chords ever fill the ring in, so exactly one H1 class is born"
        );
        assert!(
            diag.max_h1_persistence().is_infinite(),
            "nothing in the graph ever fills it"
        );
    }

    #[test]
    fn clique_complex_two_disjoint_edges_are_two_components() {
        let edges = [(0, 1, 0.1), (2, 3, 0.1)];
        let diag = clique_complex_persistence(4, &edges, 1.0);
        assert_eq!(diag.component_count(), 2);
    }

    fn octahedron_edges() -> Vec<(usize, usize, f64)> {
        // Same graph as the octahedron_vertices() geometric test, but
        // expressed directly as a relationship graph: 6 vertices, every
        // pair adjacent except the 3 antipodal pairs (0,1) (2,3) (4,5).
        let antipodes = [(0, 1), (2, 3), (4, 5)];
        let mut edges = Vec::new();
        for i in 0..6 {
            for j in (i + 1)..6 {
                if !antipodes.contains(&(i, j)) {
                    edges.push((i, j, 1.0));
                }
            }
        }
        edges
    }

    #[test]
    fn clique_complex_octahedron_graph_has_one_persistent_h2_void() {
        let diag = clique_complex_persistence(6, &octahedron_edges(), 1.0);
        assert_eq!(diag.component_count(), 1);
        assert_eq!(
            diag.h1_count(),
            0,
            "the closed surface has no un-filled loop, only a void"
        );
        assert_eq!(
            diag.void_count(),
            1,
            "the discovered relationship graph encloses one void"
        );
    }

    #[test]
    fn clique_complex_adding_the_center_vertex_collapses_the_void() {
        // Vertex 6 = the missing "center" relationship: connected to every
        // other vertex, so it cones over every face, exactly as adding
        // the center *point* did in filling_the_center_point_collapses_the_void.
        let mut edges = octahedron_edges();
        for v in 0..6 {
            edges.push((6, v, 1.0));
        }
        let diag = clique_complex_persistence(7, &edges, 1.0);
        assert_eq!(diag.component_count(), 1);
        assert_eq!(
            diag.void_count(),
            0,
            "the missing relationship, once supplied, fills the void"
        );
    }

    #[test]
    fn xor_sorted_is_symmetric_difference() {
        assert_eq!(xor_sorted(&[1, 2, 3], &[2, 3, 4]), vec![1, 4]);
        assert_eq!(xor_sorted(&[], &[1, 2]), vec![1, 2]);
        assert_eq!(xor_sorted(&[1, 2], &[1, 2]), Vec::<usize>::new());
    }
}
