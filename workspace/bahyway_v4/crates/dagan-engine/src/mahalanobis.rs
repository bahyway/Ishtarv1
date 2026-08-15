//! Mahalanobis distance: the anomaly-detection metric that accounts for
//! per-dimension scale and cross-dimension correlation, unlike raw
//! Euclidean distance. Requires the covariance matrix's inverse, computed
//! here via Gauss-Jordan elimination with partial pivoting.

use crate::D;

pub type Mat7 = [[f64; D]; D];
pub type Vec7 = [f64; D];

/// Gauss-Jordan matrix inversion with partial pivoting. Returns `None` if
/// the matrix is singular (or numerically indistinguishable from it) --
/// never silently returns a garbage inverse.
pub fn invert_symmetric(m: &Mat7) -> Option<Mat7> {
    // Augmented [M | I], reduce left half to I, read off the right half.
    let mut aug = [[0.0_f64; 2 * D]; D];
    for i in 0..D {
        for j in 0..D {
            aug[i][j] = m[i][j];
        }
        aug[i][D + i] = 1.0;
    }

    for col in 0..D {
        // Partial pivot: swap in the row with the largest magnitude entry
        // in this column, for numerical stability.
        let mut pivot_row = col;
        let mut pivot_val = aug[col][col].abs();
        for r in (col + 1)..D {
            if aug[r][col].abs() > pivot_val {
                pivot_val = aug[r][col].abs();
                pivot_row = r;
            }
        }
        if pivot_val < 1e-12 {
            return None; // singular
        }
        aug.swap(col, pivot_row);

        let pivot = aug[col][col];
        for j in 0..(2 * D) {
            aug[col][j] /= pivot;
        }
        for r in 0..D {
            if r == col {
                continue;
            }
            let factor = aug[r][col];
            if factor != 0.0 {
                for j in 0..(2 * D) {
                    aug[r][j] -= factor * aug[col][j];
                }
            }
        }
    }

    let mut inv = [[0.0; D]; D];
    for i in 0..D {
        for j in 0..D {
            inv[i][j] = aug[i][D + j];
        }
    }
    Some(inv)
}

/// Mahalanobis distance of `x` from `mean`, given the covariance matrix's
/// inverse: sqrt((x - mean)^T * cov_inv * (x - mean)).
pub fn mahalanobis_distance(x: &Vec7, mean: &Vec7, cov_inv: &Mat7) -> f64 {
    let mut delta = [0.0; D];
    for i in 0..D {
        delta[i] = x[i] - mean[i];
    }
    let mut quad = 0.0;
    for i in 0..D {
        let mut row_sum = 0.0;
        for j in 0..D {
            row_sum += cov_inv[i][j] * delta[j];
        }
        quad += delta[i] * row_sum;
    }
    quad.max(0.0).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> Mat7 {
        let mut m = [[0.0; D]; D];
        for i in 0..D {
            m[i][i] = 1.0;
        }
        m
    }

    #[test]
    fn identity_inverts_to_itself() {
        let inv = invert_symmetric(&identity()).unwrap();
        assert_eq!(inv, identity());
    }

    #[test]
    fn singular_matrix_returns_none() {
        // All-zero matrix is singular.
        let zero: Mat7 = [[0.0; D]; D];
        assert!(invert_symmetric(&zero).is_none());
    }

    #[test]
    fn mahalanobis_reduces_to_euclidean_under_identity_covariance() {
        let mean: Vec7 = [0.0; D];
        let mut x: Vec7 = [0.0; D];
        x[0] = 3.0;
        x[1] = 4.0;
        let d = mahalanobis_distance(&x, &mean, &identity());
        // 3-4-5 triangle -- Euclidean distance is exactly 5.
        assert!((d - 5.0).abs() < 1e-9, "d={d}");
    }

    #[test]
    fn mahalanobis_normalizes_by_per_axis_variance() {
        // Covariance diag(4, 1, 1, ...): dim 0 has stddev 2, others stddev 1.
        let mut cov: Mat7 = identity();
        cov[0][0] = 4.0;
        let cov_inv = invert_symmetric(&cov).unwrap();
        let mean: Vec7 = [0.0; D];

        let mut x_wide: Vec7 = [0.0; D]; // 2 units along the wide (dim 0) axis
        x_wide[0] = 2.0;
        let mut x_narrow: Vec7 = [0.0; D]; // 2 units along a narrow axis
        x_narrow[1] = 2.0;

        let d_wide = mahalanobis_distance(&x_wide, &mean, &cov_inv);
        let d_narrow = mahalanobis_distance(&x_narrow, &mean, &cov_inv);
        // Same raw Euclidean offset (2.0), but the wide axis has more
        // natural spread -- Mahalanobis correctly rates it less anomalous.
        assert!(d_wide < d_narrow, "d_wide={d_wide} d_narrow={d_narrow}");
        assert!((d_wide - 1.0).abs() < 1e-9); // 2 / stddev(2) = 1
        assert!((d_narrow - 2.0).abs() < 1e-9); // 2 / stddev(1) = 2
    }
}
