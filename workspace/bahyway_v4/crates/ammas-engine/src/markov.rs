//! Markov Chain / Steady-State Distribution / Mean First Passage Time —
//! ALGEBRA_GLOSSARY.md Part V, Layer 8, items 31-33. Confirmed genuinely
//! absent before this file: no hit for Markov/steady_state/MFPT anywhere
//! in the workspace.

/// A finite, discrete-time Markov chain given by a row-stochastic
/// transition matrix P (P[i][j] = probability of moving from state i to
/// state j in one step).
pub struct MarkovChain {
    pub p: Vec<Vec<f64>>,
    pub n: usize,
}

impl MarkovChain {
    /// # Panics
    /// Panics if `p` is not square, or any row does not sum to ~1.0.
    pub fn new(p: Vec<Vec<f64>>) -> Self {
        let n = p.len();
        for row in &p {
            assert_eq!(row.len(), n, "transition matrix must be square");
            let sum: f64 = row.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-9,
                "each row must sum to 1.0, got {sum}"
            );
        }
        MarkovChain { p, n }
    }

    /// Steady-state distribution pi such that pi P = pi, via power
    /// iteration on pi^T (starting from the uniform distribution).
    pub fn steady_state(&self, tol: f64, max_iter: usize) -> Vec<f64> {
        let n = self.n;
        let mut pi = vec![1.0 / n as f64; n];

        for _ in 0..max_iter {
            let mut next = vec![0.0; n];
            for (j, next_j) in next.iter_mut().enumerate() {
                for (i, &pi_i) in pi.iter().enumerate() {
                    *next_j += pi_i * self.p[i][j];
                }
            }
            let delta: f64 = next.iter().zip(pi.iter()).map(|(a, b)| (a - b).abs()).sum();
            pi = next;
            if delta < tol {
                break;
            }
        }
        pi
    }

    /// n x n matrix inverse via Gauss-Jordan elimination with partial pivoting.
    fn invert(m: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = m.len();
        let mut a: Vec<Vec<f64>> = m.to_vec();
        let mut inv: Vec<Vec<f64>> = (0..n)
            .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
            .collect();

        for col in 0..n {
            let mut pivot = col;
            for r in (col + 1)..n {
                if a[r][col].abs() > a[pivot][col].abs() {
                    pivot = r;
                }
            }
            a.swap(col, pivot);
            inv.swap(col, pivot);
            let d = a[col][col];
            for j in 0..n {
                a[col][j] /= d;
                inv[col][j] /= d;
            }
            for r in 0..n {
                if r == col {
                    continue;
                }
                let factor = a[r][col];
                for j in 0..n {
                    a[r][j] -= factor * a[col][j];
                    inv[r][j] -= factor * inv[col][j];
                }
            }
        }
        inv
    }

    /// Mean First Passage Time matrix M, M[i][j] = expected number of
    /// steps to first reach state j starting from state i (M[i][i] is
    /// the mean *recurrence* time, 1/pi_i).
    ///
    /// Uses the fundamental matrix method: Z = (I - P + 1·pi^T)^-1,
    /// M[i][j] = (Z[j][j] - Z[i][j]) / pi[j] for i != j, M[i][i] = 1/pi[i].
    pub fn mean_first_passage_time(&self, pi: &[f64]) -> Vec<Vec<f64>> {
        let n = self.n;
        // A = I - P + 1·pi^T
        let mut a = vec![vec![0.0; n]; n];
        for (i, row) in a.iter_mut().enumerate() {
            for j in 0..n {
                let identity = if i == j { 1.0 } else { 0.0 };
                row[j] = identity - self.p[i][j] + pi[j];
            }
        }
        let z = Self::invert(&a);

        let mut m = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    m[i][j] = 1.0 / pi[j];
                } else {
                    m[i][j] = (z[j][j] - z[i][j]) / pi[j];
                }
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steady_state_satisfies_balance_equation() {
        let chain = MarkovChain::new(vec![vec![0.9, 0.1], vec![0.2, 0.8]]);
        let pi = chain.steady_state(1e-12, 10_000);
        // pi P = pi
        for j in 0..2 {
            let sum: f64 = (0..2).map(|i| pi[i] * chain.p[i][j]).sum();
            assert!(
                (sum - pi[j]).abs() < 1e-9,
                "balance equation failed at {j}: {sum} vs {pi_j}",
                pi_j = pi[j]
            );
        }
        let total: f64 = pi.iter().sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[test]
    fn steady_state_matches_known_analytic_two_state_result() {
        // For a 2-state chain with p_12=a, p_21=b:
        // pi = [b/(a+b), a/(a+b)]  (standard closed-form result)
        let a = 0.3;
        let b = 0.4;
        let chain = MarkovChain::new(vec![vec![1.0 - a, a], vec![b, 1.0 - b]]);
        let pi = chain.steady_state(1e-14, 10_000);
        let expected_0 = b / (a + b);
        let expected_1 = a / (a + b);
        assert!(
            (pi[0] - expected_0).abs() < 1e-6,
            "pi[0]={} expected {}",
            pi[0],
            expected_0
        );
        assert!(
            (pi[1] - expected_1).abs() < 1e-6,
            "pi[1]={} expected {}",
            pi[1],
            expected_1
        );
    }

    #[test]
    fn mfpt_matches_known_analytic_two_state_result() {
        // For a 2-state chain, mean first passage time m_12 = 1/a, m_21 = 1/b
        // exactly, regardless of the other transition probability -- a
        // special, independently-checkable property of 2-state chains.
        let a = 0.3;
        let b = 0.4;
        let chain = MarkovChain::new(vec![vec![1.0 - a, a], vec![b, 1.0 - b]]);
        let pi = chain.steady_state(1e-14, 10_000);
        let m = chain.mean_first_passage_time(&pi);
        assert!(
            (m[0][1] - 1.0 / a).abs() < 1e-6,
            "m_12={} expected {}",
            m[0][1],
            1.0 / a
        );
        assert!(
            (m[1][0] - 1.0 / b).abs() < 1e-6,
            "m_21={} expected {}",
            m[1][0],
            1.0 / b
        );
    }

    #[test]
    fn mfpt_recurrence_time_is_inverse_steady_state() {
        let chain = MarkovChain::new(vec![
            vec![0.5, 0.3, 0.2],
            vec![0.1, 0.6, 0.3],
            vec![0.2, 0.2, 0.6],
        ]);
        let pi = chain.steady_state(1e-14, 10_000);
        let m = chain.mean_first_passage_time(&pi);
        for i in 0..3 {
            assert!((m[i][i] - 1.0 / pi[i]).abs() < 1e-6);
        }
    }

    #[test]
    #[should_panic(expected = "must sum to 1.0")]
    fn rejects_non_stochastic_row() {
        MarkovChain::new(vec![vec![0.5, 0.4], vec![0.5, 0.5]]);
    }
}
