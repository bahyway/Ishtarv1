//! Position verb (BC-MRD-001 §0, §3.1): H(P) distance to a behavioral
//! template under the Hepta Space metric g = diag(w1..w7).
//!
//! At a single instant, with a fixed (not-yet-recalibrated) weight
//! vector, g is a CONSTANT diagonal metric — flat. On a flat metric the
//! geodesic between two points is the straight line, so the radial
//! coordinate r is exactly the direct weighted-Euclidean distance; no
//! ODE integration is needed to compute it. (`tests::position_matches_geodesic_on_flat_metric`
//! below cross-checks this claim against `vgca_engine::riemannian`'s
//! independently-validated geodesic integrator, rather than asserting
//! it un-tested.)
//!
//! What DOES need the covariant machinery is the Motion verb (∇r/dt
//! across a *recalibrated* metric/template — BC-MRD-001 §3.2) and the
//! Curvature verb (geodesic deviation across a cohort's orbits — §3.4
//! Jacobi equation). Both are deliberately deferred; see lib.rs.

/// Hepta Space radial coordinate: geodesic distance from a particle P
/// to a template T under weights w (BC-MRD-001 §3.1, g = diag(w)).
pub fn radial_coordinate(p: &[f64; 7], t: &[f64; 7], w: &[f64; 7]) -> f64 {
    let sum_sq: f64 = (0..7).map(|i| w[i] * (p[i] - t[i]).powi(2)).sum();
    sum_sq.sqrt()
}

/// H(P) = 1/(1 + r): the Hepta Space membership score, 1.0 at the
/// template itself, falling toward 0 with distance.
pub fn template_score(p: &[f64; 7], t: &[f64; 7], w: &[f64; 7]) -> f64 {
    1.0 / (1.0 + radial_coordinate(p, t, w))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vgca_engine::riemannian::RiemannianManifold;

    #[test]
    fn template_score_is_one_at_the_template() {
        let t = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let w = [1.0; 7];
        assert_eq!(template_score(&t, &t, &w), 1.0);
    }

    #[test]
    fn template_score_falls_with_distance() {
        let t = [0.0; 7];
        let w = [1.0; 7];
        let near = [0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let far = [5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert!(template_score(&near, &t, &w) > template_score(&far, &t, &w));
    }

    /// The claim this module rests on: under a flat diagonal metric,
    /// straight-line weighted distance IS the geodesic distance.
    /// Verified against vgca-engine's independently-validated
    /// covariant/geodesic integrator rather than assumed.
    #[test]
    fn position_matches_geodesic_on_flat_metric() {
        let w = [1.0, 2.0, 1.0, 0.5, 1.0, 3.0, 1.0];
        let p = [0.0; 7];
        let t = [1.0, 0.5, -1.0, 2.0, 0.0, 0.3, -0.5];

        let w_clone = w;
        let manifold = RiemannianManifold::new(7, move |_x: &[f64]| {
            let mut g = vec![vec![0.0; 7]; 7];
            for i in 0..7 {
                g[i][i] = w_clone[i];
            }
            g
        });

        // Integrate the geodesic from p with initial velocity (t - p);
        // on a flat metric this traces a straight line, reaching t at
        // parameter s=1 with dt=1/steps.
        let v0: Vec<f64> = (0..7).map(|i| t[i] - p[i]).collect();
        let steps = 100;
        let traj = manifold.geodesic(&p, &v0, 1.0 / steps as f64, steps);
        let endpoint = traj.last().unwrap();

        for i in 0..7 {
            assert!((endpoint[i] - t[i]).abs() < 1e-6, "component {i}: {} vs {}", endpoint[i], t[i]);
        }

        // And the direct radial_coordinate formula matches the metric
        // arc-length of that same straight segment.
        let direct = radial_coordinate(&p, &t, &w);
        let arc_length: f64 = w.iter().zip(v0.iter()).map(|(wi, vi)| wi * vi * vi).sum::<f64>().sqrt();
        assert!((direct - arc_length).abs() < 1e-9);
    }
}
