//! The 20 canonical topological operators for BahyWay v4.0.
//!
//! These are the *only* topological operators.  Do not invent new ones.
//! Source: established mathematics (point-set topology, DE-9IM, algebraic
//! topology, mathematical morphology, vector calculus).
//!
//! Each function operates on a 7D coordinate vector `[f64; 7]` representing
//! a position in the (E, A, V, T, S, Z, M) Hepta manifold.
//!
//! GPU-parallel versions are in `homt-engine::field` (wgsl shaders).

// ── Type aliases used throughout ─────────────────────────────────────────────

/// A point in the 7D Hepta manifold: (E, A, V, T, S, Z, M).
pub type HeptaPoint = [f64; 7];

/// An open-ball neighbourhood radius in 7D space (always > 0).
pub type Radius = f64;

/// A scalar field value at a point.
pub type Scalar = f64;

/// A 7D vector field value at a point.
pub type VectorField7 = [f64; 7];

// ═════════════════════════════════════════════════════════════════════════════
// GROUP 1: Set-Theoretic Operators (point-set topology)
// ═════════════════════════════════════════════════════════════════════════════

/// Int(A) — open interior: largest open set contained in A.
///
/// Returns `true` if `p` is strictly inside the region defined by `centroid`
/// and `radius` (open ball — boundary excluded).
pub fn interior(p: &HeptaPoint, centroid: &HeptaPoint, radius: Radius) -> bool {
    euclidean_7d(p, centroid) < radius
}

/// ∂A — boundary: points in closure(A) but not in interior(A).
///
/// Returns `true` if `p` is within `epsilon` of the surface at `radius`.
pub fn boundary(p: &HeptaPoint, centroid: &HeptaPoint, radius: Radius, epsilon: f64) -> bool {
    let d = euclidean_7d(p, centroid);
    (d - radius).abs() <= epsilon
}

/// Cl(A) — closure: interior ∪ boundary (closed ball).
///
/// Returns `true` if `p` is inside or on the surface at `radius`.
pub fn closure(p: &HeptaPoint, centroid: &HeptaPoint, radius: Radius) -> bool {
    euclidean_7d(p, centroid) <= radius
}

/// Ext(A) — exterior: complement of the closure.
///
/// Returns `true` if `p` is strictly outside the closed ball.
pub fn exterior(p: &HeptaPoint, centroid: &HeptaPoint, radius: Radius) -> bool {
    euclidean_7d(p, centroid) > radius
}

// ═════════════════════════════════════════════════════════════════════════════
// GROUP 2: Binary Relational Operators (DE-9IM)
// ═════════════════════════════════════════════════════════════════════════════

/// DE-9IM: Disjoint — two regions share no points (no overlap, no touch).
pub fn disjoint(c1: &HeptaPoint, r1: Radius, c2: &HeptaPoint, r2: Radius) -> bool {
    euclidean_7d(c1, c2) > r1 + r2
}

/// DE-9IM: Intersects — not disjoint (interiors overlap).
pub fn intersects(c1: &HeptaPoint, r1: Radius, c2: &HeptaPoint, r2: Radius) -> bool {
    !disjoint(c1, r1, c2, r2)
}

/// DE-9IM: Within — region A is entirely inside region B.
pub fn within(c_a: &HeptaPoint, r_a: Radius, c_b: &HeptaPoint, r_b: Radius) -> bool {
    euclidean_7d(c_a, c_b) + r_a <= r_b
}

/// DE-9IM: Touches — boundaries meet but interiors do not overlap.
pub fn touches(c1: &HeptaPoint, r1: Radius, c2: &HeptaPoint, r2: Radius) -> bool {
    let d = euclidean_7d(c1, c2);
    let sum = r1 + r2;
    (d - sum).abs() < 1e-9
}

/// DE-9IM: Overlaps — interiors share some but not all points.
pub fn overlaps(c1: &HeptaPoint, r1: Radius, c2: &HeptaPoint, r2: Radius) -> bool {
    let d = euclidean_7d(c1, c2);
    d < r1 + r2 && d + r1 > r2 && d + r2 > r1
}

// ═════════════════════════════════════════════════════════════════════════════
// GROUP 3: Algebraic Topology Operators
// ═════════════════════════════════════════════════════════════════════════════

/// ∂_n — boundary operator on an n-simplex.
///
/// Given the vertices of an n-simplex, returns the (n-1)-simplices forming
/// its boundary (alternating-sign chain).  Each returned vec is one face.
pub fn boundary_n(simplex_vertices: &[HeptaPoint]) -> Vec<Vec<HeptaPoint>> {
    let n = simplex_vertices.len();
    (0..n)
        .map(|i| {
            simplex_vertices
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, v)| *v)
                .collect()
        })
        .collect()
}

/// d — exterior derivative of a scalar field f at point p.
///
/// Numerically approximated by finite differences in each of the 7 axes.
pub fn exterior_derivative<F>(f: F, p: &HeptaPoint, h: f64) -> VectorField7
where
    F: Fn(&HeptaPoint) -> Scalar,
{
    let mut grad = [0.0f64; 7];
    for i in 0..7 {
        let mut p_plus = *p;
        let mut p_minus = *p;
        p_plus[i] += h;
        p_minus[i] -= h;
        grad[i] = (f(&p_plus) - f(&p_minus)) / (2.0 * h);
    }
    grad
}

/// ∪ — cup product: combines two cochain classes (integer coefficients).
///
/// For 7D KAKI manifold use: pointwise product of two scalar field samples.
pub fn cup_product(a: Scalar, b: Scalar) -> Scalar {
    a * b
}

/// ∩ — cap product: pairs a cohomology class with a homology class.
///
/// Implemented as the Frobenius inner product ⟨a, b⟩ in 7D.
pub fn cap_product(a: &VectorField7, b: &VectorField7) -> Scalar {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// ★ — Hodge star operator in flat 7D Euclidean space.
///
/// Maps a k-form to a (7-k)-form.  For a 1-form (vector), returns the dual
/// 6-form represented as the vector itself (isomorphism in flat space).
/// Full exterior algebra implementation is deferred to `enlil-algebra`.
pub fn hodge_star(form: &VectorField7) -> VectorField7 {
    *form
}

// ═════════════════════════════════════════════════════════════════════════════
// GROUP 4: Morphological Operators (Mathematical Morphology)
// ═════════════════════════════════════════════════════════════════════════════

/// ⊕ — Dilation: expand a region's radius by structuring element `se_radius`.
pub fn dilation(centroid: HeptaPoint, radius: Radius, se_radius: Radius) -> (HeptaPoint, Radius) {
    (centroid, radius + se_radius)
}

/// ⊖ — Erosion: shrink a region's radius by structuring element `se_radius`.
pub fn erosion(centroid: HeptaPoint, radius: Radius, se_radius: Radius) -> (HeptaPoint, Radius) {
    (centroid, (radius - se_radius).max(0.0))
}

/// ○ — Opening: erosion followed by dilation (removes small protrusions).
pub fn opening(centroid: HeptaPoint, radius: Radius, se_radius: Radius) -> (HeptaPoint, Radius) {
    let (c, r) = erosion(centroid, radius, se_radius);
    dilation(c, r, se_radius)
}

/// ● — Closing: dilation followed by erosion (fills small gaps).
pub fn closing(centroid: HeptaPoint, radius: Radius, se_radius: Radius) -> (HeptaPoint, Radius) {
    let (c, r) = dilation(centroid, radius, se_radius);
    erosion(c, r, se_radius)
}

// ═════════════════════════════════════════════════════════════════════════════
// GROUP 5: Field Operators (Vector Calculus)
// ═════════════════════════════════════════════════════════════════════════════

/// ∇ — gradient of a scalar field f at point p.
///
/// Returns the 7D gradient vector (∂f/∂x₁, …, ∂f/∂x₇).
pub fn gradient<F>(f: F, p: &HeptaPoint, h: f64) -> VectorField7
where
    F: Fn(&HeptaPoint) -> Scalar,
{
    exterior_derivative(f, p, h)
}

/// ∇· — divergence of a vector field at point p.
///
/// Each component of `v` is a fn mapping p → scalar (the i-th component).
/// Returns Σ ∂vᵢ/∂xᵢ.
pub fn divergence<F>(v_components: &[F; 7], p: &HeptaPoint, h: f64) -> Scalar
where
    F: Fn(&HeptaPoint) -> Scalar,
{
    let mut div = 0.0;
    for (i, vi) in v_components.iter().enumerate() {
        let mut p_plus = *p;
        let mut p_minus = *p;
        p_plus[i] += h;
        p_minus[i] -= h;
        div += (vi(&p_plus) - vi(&p_minus)) / (2.0 * h);
    }
    div
}

/// ∇× — curl (rotation) of a 3D sub-field embedded in the 7D manifold.
///
/// Operates on dimensions (0,1,2) = (E,A,V) — the first three Hepta axes.
/// Returns the 3-component curl vector for those axes.
pub fn curl<Fx, Fy, Fz>(
    fx: Fx, fy: Fy, fz: Fz,
    p: &HeptaPoint,
    h: f64,
) -> [f64; 3]
where
    Fx: Fn(&HeptaPoint) -> Scalar,
    Fy: Fn(&HeptaPoint) -> Scalar,
    Fz: Fn(&HeptaPoint) -> Scalar,
{
    let dfz_dy = partial_deriv(&fz, p, 1, h);
    let dfy_dz = partial_deriv(&fy, p, 2, h);
    let dfx_dz = partial_deriv(&fx, p, 2, h);
    let dfz_dx = partial_deriv(&fz, p, 0, h);
    let dfy_dx = partial_deriv(&fy, p, 0, h);
    let dfx_dy = partial_deriv(&fx, p, 1, h);
    [
        dfz_dy - dfy_dz,
        dfx_dz - dfz_dx,
        dfy_dx - dfx_dy,
    ]
}

/// Δ — Laplacian of a scalar field f at point p.
///
/// Returns Σ ∂²f/∂xᵢ².  Used by VGCA-Δ smoothing operator.
pub fn laplacian<F>(f: F, p: &HeptaPoint, h: f64) -> Scalar
where
    F: Fn(&HeptaPoint) -> Scalar + Copy,
{
    let f0 = f(p);
    let mut lap = -14.0 * f0; // 7 dimensions × 2
    for i in 0..7 {
        let mut p_plus = *p;
        let mut p_minus = *p;
        p_plus[i] += h;
        p_minus[i] -= h;
        lap += f(&p_plus) + f(&p_minus);
    }
    lap / (h * h)
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn euclidean_7d(a: &HeptaPoint, b: &HeptaPoint) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn partial_deriv<F: Fn(&HeptaPoint) -> Scalar>(f: &F, p: &HeptaPoint, axis: usize, h: f64) -> f64 {
    let mut p_plus = *p;
    let mut p_minus = *p;
    p_plus[axis] += h;
    p_minus[axis] -= h;
    (f(&p_plus) - f(&p_minus)) / (2.0 * h)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGIN: HeptaPoint = [0.0; 7];
    const H: f64 = 1e-5;

    #[test]
    fn interior_excludes_boundary() {
        let c = ORIGIN;
        let p_inside = [0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let p_on     = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!( interior(&p_inside, &c, 1.0));
        assert!(!interior(&p_on,     &c, 1.0)); // strictly < radius
    }

    #[test]
    fn closure_includes_boundary() {
        let c = ORIGIN;
        let p_on = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(closure(&p_on, &c, 1.0));
    }

    #[test]
    fn exterior_is_complement_of_closure() {
        let c = ORIGIN;
        let p_out = [2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(exterior(&p_out, &c, 1.0));
        assert!(!interior(&p_out, &c, 1.0));
        assert!(!closure(&p_out, &c, 1.0));
    }

    #[test]
    fn disjoint_vs_overlaps() {
        let c1 = ORIGIN;
        let c2 = [3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(disjoint(&c1, 1.0, &c2, 1.0));
        assert!(!intersects(&c1, 1.0, &c2, 1.0));

        let c3 = [1.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(overlaps(&c1, 1.0, &c3, 1.0));
    }

    #[test]
    fn gradient_of_linear_field() {
        // f(x) = x[0] + 2*x[1] → ∇f = [1, 2, 0, 0, 0, 0, 0]
        let f = |p: &HeptaPoint| p[0] + 2.0 * p[1];
        let g = gradient(f, &ORIGIN, H);
        assert!((g[0] - 1.0).abs() < 1e-6);
        assert!((g[1] - 2.0).abs() < 1e-6);
        for i in 2..7 { assert!(g[i].abs() < 1e-6); }
    }

    #[test]
    fn laplacian_of_quadratic_is_constant() {
        // f(x) = x[0]^2 + x[1]^2 → Δf = ∂²/∂x₀² + ∂²/∂x₁² = 2 + 2 = 4
        let f = |p: &HeptaPoint| p[0].powi(2) + p[1].powi(2);
        let lap = laplacian(f, &ORIGIN, 1e-4);
        assert!((lap - 4.0).abs() < 1e-3);
    }

    #[test]
    fn boundary_n_of_triangle() {
        let v0 = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let faces = boundary_n(&[v0, v1, v2]);
        assert_eq!(faces.len(), 3); // triangle has 3 edges
        for face in &faces { assert_eq!(face.len(), 2); }
    }

    #[test]
    fn dilation_increases_radius() {
        let (_, r) = dilation(ORIGIN, 1.0, 0.5);
        assert!((r - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn opening_erodes_then_dilates() {
        // opening(r=0.3, se=1.0): erosion → max(0.3-1.0, 0) = 0.0, then dilation → 0+1.0 = 1.0
        let (_, r) = opening(ORIGIN, 0.3, 1.0);
        assert!((r - 1.0).abs() < 1e-9);
    }
}
