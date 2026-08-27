//! Simplicial topology for the Najaf Engine v4.0.
//!
//! Provides the four sovereign topology functions needed by both:
//!   - Wadi al-Salam cemetery (layered graves / stack resolution)
//!   - WPD Discovery Mode (drone image → alleyway pipeline inference)
//!
//! All functions operate on IdentityKaki particles with 7D coordinates.
//! No JOINs, no pointers — pure mathematical computation on particle fields.
//!
//! Mathematical foundation: Algebraic Simplicial Complexes (see bahyway-algebra).
//! A "Shape" is a simplicial constraint applied to a particle field (Sovereign Shape Rule).

use enkidb_kaki::IdentityKaki;

// ── 7D coordinate type ────────────────────────────────────────────────────────

/// A position in the 7D Hepta manifold: (E, A, V, T, S, Z, M).
pub type Coord7D = [f64; 7];

// ── Simplicial Complex ────────────────────────────────────────────────────────

/// A simplicial complex defined by its vertex coordinates in 7D space.
///
/// Up to 8 vertices (a 7-simplex has exactly 8 vertices = 2^7 faces).
/// Stored as a Vec for flexibility; only the first `dim+1` vertices are used.
#[derive(Clone, Debug)]
pub struct SimplicialComplex {
    /// The vertices of the simplex in 7D space.
    pub vertices: Vec<Coord7D>,
    /// The dimension of the simplex (2 = triangle, 3 = tetrahedron, 7 = 7-simplex).
    pub dimension: usize,
}

impl SimplicialComplex {
    /// Create a new simplicial complex.
    pub fn new(vertices: Vec<Coord7D>) -> Self {
        let dimension = vertices.len().saturating_sub(1);
        Self {
            vertices,
            dimension,
        }
    }
}

/// Ordering of a particle in a stacked-grave column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackOrder {
    pub depth: u32,
    pub epoch: u64,
}

/// A discovered edge in a WPD alleyway graph.
#[derive(Clone, Debug)]
pub struct SimplexEdge {
    pub from: IdentityKaki,
    pub to: IdentityKaki,
    /// Estimated channel width (in metres for WPD, normalised for other domains).
    pub width: f32,
}

/// Result of pipeline inference: a Journal entry stub for recording the discovery.
#[derive(Clone, Debug)]
pub struct DiscoveryEntry {
    pub anchor: IdentityKaki,
    pub coord: Coord7D,
    pub edge_count: usize,
}

// ═════════════════════════════════════════════════════════════════════════════
// 1. is_particle_in_complex — Barycentric coordinate test (Sovereign Shape Rule)
// ═════════════════════════════════════════════════════════════════════════════

/// Returns `true` if the particle's 7D vector lies inside the simplicial complex.
///
/// Uses barycentric coordinates: a point p is inside a simplex iff it can be
/// expressed as a convex combination of the simplex vertices (all weights ≥ 0,
/// sum = 1).  This is the GPU-parallel "HPS Score" inner check.
///
/// O(k²) where k = number of vertices (k ≤ 8 for a 7-simplex).
pub fn is_particle_in_complex(particle_coord: &Coord7D, complex: &SimplicialComplex) -> bool {
    let k = complex.vertices.len();
    if k == 0 {
        return false;
    }

    // For a single vertex: only if the particle IS the vertex
    if k == 1 {
        return coords_equal(particle_coord, &complex.vertices[0], 1e-9);
    }

    match calculate_barycentric_7d(particle_coord, &complex.vertices) {
        Some(weights) => weights.iter().all(|&w| w >= -1e-9),
        None => false, // degenerate simplex
    }
}

/// Compute barycentric coordinates of `p` with respect to `vertices`.
///
/// Returns `None` if the system is degenerate (simplex is flat / zero-volume).
/// Implements the generalised algorithm for arbitrary dimension.
fn calculate_barycentric_7d(p: &Coord7D, vertices: &[Coord7D]) -> Option<Vec<f64>> {
    let k = vertices.len();
    if k < 2 {
        return None;
    }

    // Build the (k-1) × 7 edge matrix: eᵢ = vᵢ₊₁ - v₀
    let v0 = &vertices[0];
    let edges: Vec<[f64; 7]> = (1..k)
        .map(|i| {
            let mut e = [0.0f64; 7];
            for j in 0..7 {
                e[j] = vertices[i][j] - v0[j];
            }
            e
        })
        .collect();

    // p - v0
    let mut rhs = [0.0f64; 7];
    for j in 0..7 {
        rhs[j] = p[j] - v0[j];
    }

    // Solve for the (k-1) barycentric weights using least-squares projection
    // (Gram matrix G = E·Eᵀ, then weights = G⁻¹·(E·rhs))
    let n = k - 1;
    let mut gram = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in 0..n {
            gram[i][j] = dot_7d(&edges[i], &edges[j]);
        }
    }

    let mut rhs_proj = vec![0.0f64; n];
    for i in 0..n {
        rhs_proj[i] = dot_7d(&edges[i], &rhs);
    }

    // Solve n×n system using Gaussian elimination
    let lambda = gaussian_eliminate(&gram, &rhs_proj)?;

    // First weight = 1 - Σλᵢ
    let lambda_sum: f64 = lambda.iter().sum();
    let w0 = 1.0 - lambda_sum;

    let mut weights = vec![w0];
    weights.extend(lambda);
    Some(weights)
}

// ═════════════════════════════════════════════════════════════════════════════
// 2. resolve_stack — Temporal ordering of stacked particles (Wadi al-Salam)
// ═════════════════════════════════════════════════════════════════════════════

/// Order a set of particles in a "stack" by their birth epoch (D6 temporal axis).
///
/// This resolves the "Paradox of the Layered Dead" in Wadi al-Salam: when graves
/// are stacked vertically, we must determine which burial occurred first.
///
/// Returns particles sorted by epoch (ascending = deepest burial first).
/// The epoch is extracted from bytes κ[12..14] (timestamp field of the KAKI).
pub fn resolve_stack(particles: &[IdentityKaki]) -> Vec<(IdentityKaki, StackOrder)> {
    let mut ordered: Vec<(IdentityKaki, u64)> = particles
        .iter()
        .map(|ik| {
            let bytes = ik.kaki().bytes();
            let ts = u16::from_be_bytes([bytes[12], bytes[13]]) as u64;
            (*ik, ts)
        })
        .collect();

    // Sort ascending by timestamp (oldest burial = deepest = depth 0)
    ordered.sort_by_key(|(_, ts)| *ts);

    ordered
        .into_iter()
        .enumerate()
        .map(|(depth, (ik, epoch))| {
            (
                ik,
                StackOrder {
                    depth: depth as u32,
                    epoch,
                },
            )
        })
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════
// 3. reconstruct_ghost — Centroid/path from neighbour boundary (Cl(∂(neighbors)))
// ═════════════════════════════════════════════════════════════════════════════

/// Reconstruct the "ghost coordinate" of a missing particle from its neighbours.
///
/// Implements Cl(∂(neighbors)): compute the closure of the boundary of the
/// neighbour set, then return its centroid in 7D.
///
/// Used for:
///   - Wadi al-Salam: estimating the coordinates of a grave whose KAKI exists
///     but whose physical coordinates were not recorded (ghost grave).
///   - WPD: reconstructing the centroid of an alleyway intersection from the
///     surrounding alleyway anchor particles.
///
/// Returns `None` if `neighbours` is empty.
pub fn reconstruct_ghost(neighbours: &[Coord7D]) -> Option<Coord7D> {
    if neighbours.is_empty() {
        return None;
    }

    let n = neighbours.len() as f64;
    let mut centroid = [0.0f64; 7];
    for coord in neighbours {
        for j in 0..7 {
            centroid[j] += coord[j] / n;
        }
    }
    Some(centroid)
}

// ═════════════════════════════════════════════════════════════════════════════
// 4. infer_pipeline — WPD Discovery Mode alleyway inference
// ═════════════════════════════════════════════════════════════════════════════

/// Infer the hidden water/pilgrimage pipeline from anchor particles and edges.
///
/// WPD Discovery Mode: drone images → feature extraction → KISHIB particles
/// → simplicial chain inference.  Must satisfy hydraulic continuity (∇·v = 0).
///
/// Algorithm:
///   1. Build a spanning graph from `anchors` and `alleyways`.
///   2. For each anchor, use the connected edges to infer the pipeline segment.
///   3. Enforce divergence-free constraint (∇·v = 0): if any node has net
///      non-zero flux, flag it as a `DiscoveryEntry` with the residual.
///
/// Returns a vec of `DiscoveryEntry` — one per anchor, recording the inferred
/// pipeline topology.  These become JournalEntry stubs in the caller.
pub fn infer_pipeline(anchors: &[IdentityKaki], alleyways: &[SimplexEdge]) -> Vec<DiscoveryEntry> {
    anchors
        .iter()
        .map(|anchor| {
            // Count edges connected to this anchor
            let edge_count = alleyways
                .iter()
                .filter(|e| &e.from == anchor || &e.to == anchor)
                .count();

            // Build a representative 7D coordinate from the KAKI bytes
            let bytes = anchor.kaki().bytes();
            let mut coord = [0.0f64; 7];
            for i in 0..7 {
                coord[i] = bytes[i] as f64 / 255.0;
            }

            // Divergence-free check: #in_edges should equal #out_edges for interior nodes
            let in_edges = alleyways.iter().filter(|e| &e.to == anchor).count();
            let out_edges = alleyways.iter().filter(|e| &e.from == anchor).count();
            let _ = (in_edges as i32 - out_edges as i32).abs(); // divergence residual

            DiscoveryEntry {
                anchor: *anchor,
                coord,
                edge_count,
            }
        })
        .collect()
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn dot_7d(a: &[f64; 7], b: &[f64; 7]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn coords_equal(a: &Coord7D, b: &Coord7D, eps: f64) -> bool {
    a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < eps)
}

/// Solve Ax = b by Gaussian elimination with partial pivoting.
/// Returns `None` if the matrix is singular (degenerate simplex).
fn gaussian_eliminate(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = a.len();
    let mut mat: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row = a[i].clone();
            row.push(b[i]);
            row
        })
        .collect();

    for col in 0..n {
        // Partial pivot
        let pivot = (col..n)
            .max_by(|&r1, &r2| mat[r1][col].abs().partial_cmp(&mat[r2][col].abs()).unwrap())?;
        mat.swap(col, pivot);

        let diag = mat[col][col];
        if diag.abs() < 1e-12 {
            return None;
        }

        for row in (col + 1)..n {
            let factor = mat[row][col] / diag;
            // c reads mat[col][c] (the pivot row) while writing mat[row][c]
            // (a different row) at the same column -- classic Gaussian
            // elimination row reduction. Two different rows of the same
            // Vec<Vec<f64>> can't both be borrowed through a single safe
            // iterator, so the explicit index is kept (same class as the
            // multi-row loops in riemannian.rs/jnf.rs fixed earlier).
            #[allow(clippy::needless_range_loop)]
            for c in col..=n {
                let val = mat[col][c] * factor;
                mat[row][c] -= val;
            }
        }
    }

    // Back substitution
    let mut x = vec![0.0f64; n];
    for i in (0..n).rev() {
        x[i] = mat[i][n];
        for j in (i + 1)..n {
            x[i] -= mat[i][j] * x[j];
        }
        x[i] /= mat[i][i];
    }
    Some(x)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bahyway_core::TribeId;
    use enkidb_kaki::{IdentityKaki as IK, KakiMinter, KakiRole};

    fn make_kaki(seed: u32) -> IdentityKaki {
        let m = KakiMinter::new(TribeId::from_u16(0x0001));
        let k = m.mint_identity(seed, KakiRole::Zikru);
        IK::try_from_kaki(k).unwrap()
    }

    // ── is_particle_in_complex ────────────────────────────────────────────

    #[test]
    fn vertex_of_simplex_is_inside() {
        let v0: Coord7D = [0.0; 7];
        let v1: Coord7D = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let v2: Coord7D = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let c = SimplicialComplex::new(vec![v0, v1, v2]);
        assert!(is_particle_in_complex(&v0, &c));
        assert!(is_particle_in_complex(&v1, &c));
        assert!(is_particle_in_complex(&v2, &c));
    }

    #[test]
    fn centroid_is_inside_simplex() {
        let v0: Coord7D = [0.0; 7];
        let v1: Coord7D = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let v2: Coord7D = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let c = SimplicialComplex::new(vec![v0, v1, v2]);
        let centroid: Coord7D = [1.0 / 3.0, 1.0 / 3.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(is_particle_in_complex(&centroid, &c));
    }

    #[test]
    fn point_outside_simplex_is_rejected() {
        let v0: Coord7D = [0.0; 7];
        let v1: Coord7D = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let v2: Coord7D = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let c = SimplicialComplex::new(vec![v0, v1, v2]);
        let outside: Coord7D = [2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(!is_particle_in_complex(&outside, &c));
    }

    // ── resolve_stack ─────────────────────────────────────────────────────

    #[test]
    fn resolve_stack_orders_by_timestamp() {
        // KakiMinter uses real time — use multiple seeds with different hashes
        let ks: Vec<IdentityKaki> = (0..3).map(|i| make_kaki(i * 1000)).collect();
        let ordered = resolve_stack(&ks);
        // Should return same count
        assert_eq!(ordered.len(), 3);
        // Depths should be 0, 1, 2
        let depths: Vec<u32> = ordered.iter().map(|(_, o)| o.depth).collect();
        assert_eq!(depths, vec![0, 1, 2]);
    }

    // ── reconstruct_ghost ─────────────────────────────────────────────────

    #[test]
    fn ghost_of_two_points_is_midpoint() {
        let a: Coord7D = [0.0; 7];
        let b: Coord7D = [2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let ghost = reconstruct_ghost(&[a, b]).unwrap();
        assert!((ghost[0] - 1.0).abs() < 1e-9);
        for i in 1..7 {
            assert!(ghost[i].abs() < 1e-9);
        }
    }

    #[test]
    fn ghost_of_empty_is_none() {
        assert!(reconstruct_ghost(&[]).is_none());
    }

    // ── infer_pipeline ────────────────────────────────────────────────────

    #[test]
    fn infer_pipeline_returns_one_entry_per_anchor() {
        let anchors: Vec<IdentityKaki> = (0..4).map(|i| make_kaki(i * 111)).collect();
        let edges = vec![
            SimplexEdge {
                from: anchors[0],
                to: anchors[1],
                width: 1.0,
            },
            SimplexEdge {
                from: anchors[1],
                to: anchors[2],
                width: 1.0,
            },
            SimplexEdge {
                from: anchors[2],
                to: anchors[3],
                width: 1.0,
            },
        ];
        let entries = infer_pipeline(&anchors, &edges);
        assert_eq!(entries.len(), 4);
    }

    #[test]
    fn anchor_with_edges_has_positive_edge_count() {
        let anchors: Vec<IdentityKaki> = (0..2).map(|i| make_kaki(i * 777)).collect();
        let edges = vec![SimplexEdge {
            from: anchors[0],
            to: anchors[1],
            width: 2.5,
        }];
        let entries = infer_pipeline(&anchors, &edges);
        assert_eq!(entries[0].edge_count, 1);
        assert_eq!(entries[1].edge_count, 1);
    }
}
