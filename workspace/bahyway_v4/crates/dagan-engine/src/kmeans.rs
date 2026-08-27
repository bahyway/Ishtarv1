//! Deterministic clustering: farthest-first centroid seeding (a
//! deterministic alternative to probabilistic k-means++ -- same goal,
//! well-spread initial centroids, but fully reproducible across runs and
//! platforms, no RNG anywhere) followed by standard Lloyd iteration.

use crate::D;

pub type Vec7 = [f64; D];

fn sq_dist(a: &Vec7, b: &Vec7) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

/// Deterministic farthest-first seeding: the first centroid is always
/// `points[0]` (a fixed, reproducible choice); each subsequent centroid is
/// the point that maximizes its minimum distance to every centroid chosen
/// so far. No randomness anywhere -- same input always yields the same
/// centroids.
pub fn farthest_first_init(points: &[Vec7], k: usize) -> Vec<Vec7> {
    if points.is_empty() || k == 0 {
        return Vec::new();
    }
    let k = k.min(points.len());
    let mut centroids = vec![points[0]];
    while centroids.len() < k {
        let mut best_idx = 0;
        let mut best_min_dist = -1.0_f64;
        for (i, p) in points.iter().enumerate() {
            let min_dist = centroids
                .iter()
                .map(|c| sq_dist(p, c))
                .fold(f64::MAX, f64::min);
            if min_dist > best_min_dist {
                best_min_dist = min_dist;
                best_idx = i;
            }
        }
        centroids.push(points[best_idx]);
    }
    centroids
}

#[derive(Debug, Clone)]
pub struct KMeansResult {
    pub centroids: Vec<Vec7>,
    /// `assignments[i]` is the centroid index for `points[i]`.
    pub assignments: Vec<usize>,
    pub iterations_run: usize,
    pub converged: bool,
}

/// Standard Lloyd iteration from a given (typically `farthest_first_init`)
/// starting point. Converges when no point changes assignment.
pub fn lloyd_iterate(
    points: &[Vec7],
    initial_centroids: Vec<Vec7>,
    max_iter: usize,
) -> KMeansResult {
    let k = initial_centroids.len();
    let mut centroids = initial_centroids;
    let mut assignments = vec![0usize; points.len()];
    let mut converged = false;
    let mut iterations_run = 0;

    for iter in 0..max_iter.max(1) {
        iterations_run = iter + 1;
        let mut changed = false;
        for (i, p) in points.iter().enumerate() {
            let mut best = 0usize;
            let mut best_dist = f64::MAX;
            for (c_idx, c) in centroids.iter().enumerate() {
                let d = sq_dist(p, c);
                if d < best_dist {
                    best_dist = d;
                    best = c_idx;
                }
            }
            if assignments[i] != best {
                changed = true;
            }
            assignments[i] = best;
        }

        // Recompute centroids as the mean of their assigned points. A
        // centroid with no points keeps its previous position rather than
        // collapsing to the origin.
        let mut sums = vec![[0.0; D]; k];
        let mut counts = vec![0u64; k];
        for (i, p) in points.iter().enumerate() {
            let c = assignments[i];
            counts[c] += 1;
            for d in 0..D {
                sums[c][d] += p[d];
            }
        }
        for c in 0..k {
            if counts[c] > 0 {
                for d in 0..D {
                    centroids[c][d] = sums[c][d] / counts[c] as f64;
                }
            }
        }

        if !changed {
            converged = true;
            break;
        }
    }

    KMeansResult {
        centroids,
        assignments,
        iterations_run,
        converged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x0: f64, x1: f64) -> Vec7 {
        let mut v = [0.0; D];
        v[0] = x0;
        v[1] = x1;
        v
    }

    #[test]
    fn farthest_first_init_is_deterministic_across_calls() {
        let points = vec![
            point(0.0, 0.0),
            point(10.0, 0.0),
            point(5.0, 5.0),
            point(0.0, 10.0),
        ];
        let a = farthest_first_init(&points, 2);
        let b = farthest_first_init(&points, 2);
        assert_eq!(a, b);
    }

    #[test]
    fn farthest_first_init_spreads_centroids_apart() {
        let points = vec![
            point(0.0, 0.0),
            point(1.0, 0.0),
            point(10.0, 10.0),
            point(11.0, 10.0),
        ];
        let seeds = farthest_first_init(&points, 2);
        // The two natural clusters are far apart -- seeding should pick
        // one point from each, not two neighbours.
        let d = sq_dist(&seeds[0], &seeds[1]);
        assert!(d > 50.0, "seeds too close: {seeds:?} dist={d}");
    }

    #[test]
    fn lloyd_separates_two_obvious_clusters() {
        let points = vec![
            point(0.0, 0.0),
            point(0.2, 0.1),
            point(-0.1, 0.2),
            point(10.0, 10.0),
            point(10.2, 9.8),
            point(9.9, 10.1),
        ];
        let seeds = farthest_first_init(&points, 2);
        let result = lloyd_iterate(&points, seeds, 20);
        assert!(result.converged);
        // First three points share one cluster label, last three share
        // the other, and the two labels differ.
        let a = result.assignments[0];
        assert_eq!(result.assignments[1], a);
        assert_eq!(result.assignments[2], a);
        let b = result.assignments[3];
        assert_eq!(result.assignments[4], b);
        assert_eq!(result.assignments[5], b);
        assert_ne!(a, b);
    }

    #[test]
    fn empty_points_is_safe() {
        let seeds = farthest_first_init(&[], 3);
        assert!(seeds.is_empty());
    }
}
